//! Module for interfacing with the emulator
use std::collections::HashMap;
use std::iter;

use sdl2::SdlResult;
use sdl2::surface::Surface;
use sdl2::render::{Renderer, Texture};

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::graphics;

use common::{PlayerData, MovementData, PlayerId};
use common::data::{self, Party, PokemonData};


mod offsets;
mod values;

fn load_party(party: data::Party, mem: &mut Memory) {
    let pokemon = party.pokemon;
    let pokemon_array = [pokemon.0, pokemon.1, pokemon.2, pokemon.3, pokemon.4, pokemon.5];

    let mut addr = (offsets::PROF_OAK_DATA_ADDR & 0x3FFF) as usize;
    let bank = offsets::PROF_OAK_DATA_BANK;

    mem.cart.rom[bank][addr] = 0xFF;
    addr += 1;
    for mon in pokemon_array.iter().take(party.num_pokemon as usize) {
        mem.cart.rom[bank][addr] = mon.level;
        mem.cart.rom[bank][addr + 1] = mon.species;
        addr += 2;
    }
    mem.cart.rom[bank][addr] = 0;
}

#[derive(PartialEq)]
enum DataState {
    Normal,
    Hacked,
}

#[derive(PartialEq)]
pub enum GameState {
    Normal,
    Waiting,
}

#[derive(PartialEq)]
pub enum NetworkRequest {
    None,
    Battle(PlayerId),
}

pub struct GameData {
    pub game_state: GameState,
    pub network_request: NetworkRequest,
    pub other_players: HashMap<u32, PlayerData>,
    last_interaction: u32,
    sprite_id_state: DataState,
    text_state: DataState,
    current_message: Vec<u8>,
}

impl GameData {
    pub fn new() -> GameData {
        GameData {
            game_state: GameState::Normal,
            network_request: NetworkRequest::None,
            other_players: HashMap::new(),
            last_interaction: 0,
            sprite_id_state: DataState::Normal,
            text_state: DataState::Normal,
            current_message: Vec::new(),
        }
    }

    pub fn load_message(&mut self, input: &str) {
        // The expected format of the text in the game is *not* ascii. Here we convert the input
        // UTF-8 string into the proper format.
        // TODO: Clean up and refactor this code
        self.current_message.push(0x50);
        self.current_message.push(0x57);
        for byte in input.bytes().rev() {
            if 'a' as u8 <= byte && byte <= 'z' as u8 || 'A' as u8 <= byte && byte <= 'Z' as u8 {
                self.current_message.push(byte + 0x3F);
            }
            else if byte == ',' as u8 {
                self.current_message.push(0xF4);
            }
            else if byte == '.' as u8 {
                self.current_message.push(0xE8);
            }
            else if byte == ' ' as u8 {
                self.current_message.push(0x7F);
            }
            else if byte == '\n' as u8 {
                self.current_message.push(0x4F);
            }
            else {
                panic!("Unsupported character");
            }
        }
        self.current_message.push(0x00);
    }
}

pub fn sprite_check_hack(cpu: &mut Cpu, mem: &mut Memory, game_data: &mut GameData) {
    if cpu.pc == offsets::OVERWORLD_LOOP_START {
        game_data.sprite_id_state = DataState::Normal;
    }

    if (cpu.pc == offsets::SPRITE_CHECK_EXIT_1 && mem.lb(offsets::NUM_SPRITES) == 0) ||
        cpu.pc == offsets::SPRITE_CHECK_EXIT_2
    {
        let map_id = mem.lb(offsets::MAP_ID);

        // Determine the tile that the player is trying to move into.
        let mut x = mem.lb(offsets::MAP_X);
        let mut y = mem.lb(offsets::MAP_Y);
        match mem.lb(offsets::PLAYER_DIR) {
            0x00 => y += 1, // Down
            0x04 => y -= 1, // Up
            0x0C => x += 1, // Right
            _    => x -= 1, // Left
        }

        // Check if there are any other players that occupy this tile
        for (id, player) in game_data.other_players.iter() {
            if player.movement_data.map_id == map_id && player.check_collision(x, y) {
                // If there was a player set a sentinel value so the game thinks that there is
                // something in the way.
                mem.sb(offsets::SPRITE_INDEX, 0xFF);
                game_data.sprite_id_state = DataState::Hacked;
                game_data.last_interaction = *id;
                break;
            }
        }
    }
}

pub fn display_text_hack(cpu: &mut Cpu, mem: &mut Memory, game_data: &mut GameData) {
    if game_data.sprite_id_state == DataState::Hacked &&
        cpu.pc == offsets::DISPLAY_TEXT_ID_AFTER_INIT
    {
        // Skip unnecessary parts of the DISPLAY_TEXT_ID routine releated to finding the correct
        // message address when we are interacting with a hacked object.
        cpu.jump(offsets::DISPLAY_TEXT_SETUP_DONE);
        // Set the delay time (this is normally set in the middle of the code we just skipped)
        mem.sb(offsets::FRAME_COUNTER, 30);

        game_data.text_state = DataState::Hacked;
        game_data.load_message("PLAYER has nothing\nto say.");

        game_data.network_request = NetworkRequest::Battle(game_data.last_interaction);
        // We probably want to defer this until as late as possible, to avoid latency causing too
        // much of an issue
        game_data.game_state = GameState::Waiting;
    }

    // If the text state is hacked when running the text processor, read from our message buffer
    // instead of from the emulator's memory
    if game_data.text_state == DataState::Hacked && (cpu.pc == offsets::TEXT_PROCESSOR_NEXT_CHAR_1
        || cpu.pc == offsets::TEXT_PROCESSOR_NEXT_CHAR_2)
    {
        cpu.a = game_data.current_message.pop().unwrap_or(0x50);
        cpu.pc += 1;
    }

    // Ensure that when we leave the text processor, we reset the text state so that the next call
    // to the text processor will correctly read from the game.
    if cpu.pc == offsets::TEXT_PROCESSOR_END {
        game_data.text_state = DataState::Normal;
    }
}

// A temporary method to set a battle. In future we probably want to do more of the setup manually
// so that we can do things like set the pokemon moves, EVs, DVs etc.
pub fn set_battle(mem: &mut Memory, party: Party) {
    mem.sb(offsets::BATTLE_TYPE, values::BattleType::Normal as u8);
    mem.sb(offsets::ACTIVE_BATTLE, values::ActiveBattle::Trainer as u8);
    mem.sb(offsets::TRAINER_NUM, 1);
    mem.sb(offsets::CURRRENT_OPPONENT, values::TrainerClass::ProfOak as u8 + values::TRAINER_TAG);

    load_party(party, mem);
}

fn extract_movement_data(mem: &Memory) -> MovementData {
    MovementData {
        map_id: mem.lb(offsets::MAP_ID),
        map_x: mem.lb(offsets::MAP_X),
        map_y: mem.lb(offsets::MAP_Y),
        direction: mem.lb(offsets::PLAYER_DIR),
        walk_counter: mem.lb(offsets::WALK_COUNTER),
    }
}

pub fn extract_player_data(mem: &Memory) -> PlayerData {
    let mut name = vec![];

    // TODO: Load player name from memory
    // let mut offset = offsets::PLAYER_NAME_START;
    // loop {
    //     match mem.lb(offset) {
    //         values::END_CHAR => break,
    //         val => name.push(val),
    //     }
    // }

    PlayerData {
        name: name,
        movement_data: extract_movement_data(mem),
    }
}

fn extract_pokemon(mem: &Memory, addr: u16) -> PokemonData {
    PokemonData {
        species: mem.lb(addr+0),
        hp: mem.lw(addr+1),
        unknown: mem.lb(addr+3),
        status: mem.lb(addr+4),
        type1: mem.lb(addr+5),
        type2: mem.lb(addr+6),
        catch_rate: mem.lb(addr+7),
        moves: (mem.lb(addr+8), mem.lb(addr+9), mem.lb(addr+10), mem.lb(addr+11)),
        ot_id: mem.lw(addr+12),

        exp: (mem.lb(addr+14), mem.lb(addr+15), mem.lb(addr+16)),
        hp_ev: mem.lw(addr+17),
        attack_ev: mem.lw(addr+19),
        defense_ev: mem.lw(addr+21),
        speed_ev: mem.lw(addr+23),
        special_ev: mem.lw(addr+25),
        individual_values: (mem.lb(addr+27), mem.lb(addr+28)),
        move_pp: (mem.lb(addr+29), mem.lb(addr+30), mem.lb(addr+31), mem.lb(addr+32)),

        level: mem.lb(addr+33),
        max_hp: mem.lw(addr+34),
        attack: mem.lw(addr+36),
        defense: mem.lw(addr+38),
        speed: mem.lw(addr+40),
        special: mem.lw(addr+42),
    }
}

pub fn extract_player_party(mem: &Memory) -> Party {
    Party {
        num_pokemon: mem.lb(offsets::PARTY_COUNT),
        pokemon: (extract_pokemon(mem, offsets::PARTY_POKE_1),
            extract_pokemon(mem, offsets::PARTY_POKE_2),
            extract_pokemon(mem, offsets::PARTY_POKE_3),
            extract_pokemon(mem, offsets::PARTY_POKE_4),
            extract_pokemon(mem, offsets::PARTY_POKE_5),
            extract_pokemon(mem, offsets::PARTY_POKE_6)),
    }
}

pub fn extract_player_texture(renderer: &Renderer, mem: &Memory) -> SdlResult<Texture> {
    extract_2bpp_sprite_texture(renderer, mem, offsets::PLAYER_SPRITE_BANK,
        offsets::PLAYER_SPRITE_ADDR, 16, 16 * 6)
}

pub fn extract_font_texture(renderer: &Renderer, mem: &Memory) -> SdlResult<Texture> {
    const BLACK: [u8; 4] = [0, 0, 0, 255];
    const WHITE: [u8; 4] = [255, 255, 255, 255];

    extract_1bpp_texture(renderer, mem, offsets::FONT_BANK, offsets::FONT_ADDR, 8 * 16 * 8, 8,
        [BLACK, WHITE])
}

const RMASK: u32 = 0x000000FF;
const GMASK: u32 = 0x0000FF00;
const BMASK: u32 = 0x00FF0000;
const AMASK: u32 = 0xFF000000;

const TILE_SIZE: usize = 8;
const BYTES_PER_PIXEL: usize = 4;

pub fn extract_2bpp_sprite_texture(renderer: &Renderer, mem: &Memory, bank: usize, addr: u16,
    width: usize, height: usize) -> SdlResult<Texture>
{
    let num_x_tiles = width / TILE_SIZE;
    let num_y_tiles = height / TILE_SIZE;

    const BYTES_PER_PIXEL: usize = 4;
    let buffer_size = width * height * BYTES_PER_PIXEL;

    let mut output_buffer: Vec<_> = iter::repeat(0).take(buffer_size).collect();
    let mut sprite_offset = (addr & 0x3FFF) as usize;

    let (mut tile_x, mut tile_y) = (0, 0);
    while tile_y < num_y_tiles {
        for y in 0..8 {
            // Colors stored in the 2bpp format are split over two bytes. The color's lower bit is
            // stored in the first byte and the high bit is stored in the second byte.
            let color_low = mem.cart.rom[bank][sprite_offset];
            let color_high = mem.cart.rom[bank][sprite_offset + 1];
            sprite_offset += 2;

            for x in 0..8 {
                let color_id = graphics::get_color_id(color_low, color_high, x);
                let color = graphics::palette_lookup(208, color_id);

                // Compute the offset of where to place this pixel in the output buffer
                let offset = ((((num_x_tiles - tile_x - 1) * TILE_SIZE) + x) +
                    ((tile_y * TILE_SIZE) + y) * width) * BYTES_PER_PIXEL;

                output_buffer[offset + 0] = color[0];
                output_buffer[offset + 1] = color[1];
                output_buffer[offset + 2] = color[2];
                output_buffer[offset + 3] = if color_id == 0 { 0 } else { 255 };
            }
        }

        // Step to the next tile
        tile_x += 1;
        if tile_x >= num_x_tiles {
            tile_x = 0;
            tile_y += 1;
        }
    }

    let surface = try!(Surface::from_data(&mut *output_buffer, width as isize,
        height as isize, 32, (width * BYTES_PER_PIXEL) as isize, RMASK, GMASK, BMASK, AMASK));
    renderer.create_texture_from_surface(&surface)
}

pub fn extract_1bpp_texture(renderer: &Renderer, mem: &Memory, bank: usize, addr: u16, width: usize,
    height: usize, colors: [[u8; 4]; 2]) -> SdlResult<Texture>
{
    const TILE_SIZE: usize = 8;
    let num_x_tiles = width / TILE_SIZE;
    let num_y_tiles = height / TILE_SIZE;
    let buffer_size = width * height * BYTES_PER_PIXEL;

    let mut output_buffer: Vec<_> = iter::repeat(0).take(buffer_size).collect();
    let mut sprite_offset = (addr & 0x3FFF) as usize;

    let (mut tile_x, mut tile_y) = (0, 0);
    while tile_y < num_y_tiles {
        for y in 0..8 {
            let color_row = mem.cart.rom[bank][sprite_offset];
            sprite_offset += 1;

            for x in 0..8 {
                // Compute the offset of where to place this pixel in the output buffer
                let offset = (((tile_x * TILE_SIZE) + x) +
                    ((tile_y * TILE_SIZE) + y) * width) * BYTES_PER_PIXEL;

                if color_row & 1 << (8 - x) != 0 {
                    output_buffer[offset + 0] = colors[0][0];
                    output_buffer[offset + 1] = colors[0][1];
                    output_buffer[offset + 2] = colors[0][2];
                    output_buffer[offset + 3] = colors[0][3];
                }
                else {
                    output_buffer[offset + 0] = colors[1][0];
                    output_buffer[offset + 1] = colors[1][1];
                    output_buffer[offset + 2] = colors[1][2];
                    output_buffer[offset + 3] = colors[1][3];
                }
            }
        }

        // Step to the next tile
        tile_x += 1;
        if tile_x >= num_x_tiles {
            tile_x = 0;
            tile_y += 1;
        }
    }

    let surface = try!(Surface::from_data(&mut *output_buffer, width as isize,
        height as isize, 32, (width * BYTES_PER_PIXEL) as isize, RMASK, GMASK, BMASK, AMASK));
    renderer.create_texture_from_surface(&surface)
}

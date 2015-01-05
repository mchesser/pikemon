//! Module for interfacing with the emulator
use std::collections::HashMap;

use sdl2::SdlResult;
use sdl2::surface::Surface;
use sdl2::render::{Renderer, Texture};

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::graphics;

use common::PlayerData;

use data::Pokemon;

mod offsets {
    #![allow(dead_code)]

    // Player positional data
    pub const MAP_ID: u16 = 0xD35E;
    pub const MAP_Y: u16 = 0xD361;
    pub const MAP_X: u16 = 0xD362;
    pub const PLAYER_DY: u16 = 0xC103;
    pub const PLAYER_DX: u16 = 0xC105;
    // The direction which the player is facing (0: down, 4: up, 8: left, 12: right)
    pub const PLAYER_DIR: u16 = 0xC109;
    // When a player moves, this value counts down from 8 to 0
    pub const WALK_COUNTER: u16 = 0xCFC5;

    // The address of the player spritesheet encoded as 2bpp in the rom
    pub const PLAYER_SPRITE_ADDR: u16 = 0x4180;
    pub const PLAYER_SPRITE_BANK: uint = 5;

    // Useful addresses for hacks
    pub const LOADED_ROM_BANK: u16 = 0xFFB8;
    pub const FRAME_COUNTER: u16 = 0xFFD5;
    pub const BANK_SWITCH: u16 = 0x35D6;

    // Addresses for sprite check hack
    pub const NUM_SPRITES: u16 = 0xD4E1;
    pub const OVERWORLD_LOOP_START: u16 = 0x03FF;
    pub const SPRITE_CHECK_START: u16 = 0x0B23;
    pub const SPRITE_CHECK_EXIT_1: u16 = 0x0BA0;
    pub const SPRITE_CHECK_EXIT_2: u16 = 0x0BC4;
    pub const SPRITE_INDEX: u16 = 0xFF8C;

    // Addresses for display text hack
    pub const DISPLAY_TEXT_ID: u16 = 0x2920;
    pub const DISPLAY_TEXT_ID_AFTER_INIT: u16 = 0x292B;
    pub const DISPLAY_TEXT_SETUP_DONE: u16 = 0x29CD;
    pub const TEXT_PROCESSOR_NEXT_CHAR_1: u16 = 0x1B55;
    pub const TEXT_PROCESSOR_NEXT_CHAR_2: u16 = 0x1956;
    pub const TEXT_PROCESSOR_END: u16 = 0x1B5E;

    // Addresses for battle hack
    pub const TRAINER_CLASS: u16 = 0xD031;
    pub const TRAINER_NAME: u16 = 0xD04A;
    pub const TRAINER_NUM: u16 = 0xD05D;
    pub const ACTIVE_BATTLE: u16 = 0xD057;
    pub const CURRRENT_OPPONENT: u16 = 0xD059;
    pub const CURRENT_ENEMY_LEVEL: u16 = 0xD127;
    pub const CURRENT_ENEMY_NICK: u16 = 0x0000;
    pub const BATTLE_TYPE: u16 = 0xD05A;
    pub const IS_LINK_BATTLE: u16 = 0xD12B;

    // The Prof. Oak battle is unused by the game, so it is a convenient place to replace with our
    // battle data.
    pub const PROF_OAK_DATA_ADDR: u16 = 0x621D;
    pub const PROF_OAK_DATA_BANK: uint = 0xE;
}

mod values {
    #![allow(dead_code)]

    pub const FALSE: u8 = 0;
    pub const TRUE: u8 = 1;

    pub enum PlayerDir {
        Down = 0,
        Up = 4,
        Left = 8,
        Right = 12,
    }

    pub enum BattleType {
        Normal = 0,
        OldMan = 1,
        Safari = 2,
    }

    pub enum ActiveBattle {
        None = 0,
        Wild = 1,
        Trainer = 2,
    }

    pub enum TrainerClass {
        Unknown = 0x00,
        ProfOak = 0x1A,
    }

    // This gets added to the trainer class when setting CURRENT_OPPONENT
    pub const TRAINER_TAG: u8 = 0xC8;
}

struct Party {
    pokemon: [(u8, Pokemon); 6],
}

fn load_party(party: &Party, mem: &mut Memory) {
    let mut addr = (offsets::PROF_OAK_DATA_ADDR & 0x3FFF) as uint;
    let bank = offsets::PROF_OAK_DATA_BANK;

    mem.cart.rom[bank][addr] = 0xFF;
    addr += 1;
    for &(lvl, mon) in party.pokemon.iter() {
        mem.cart.rom[bank][addr] = lvl;
        mem.cart.rom[bank][addr + 1] = mon as u8;
        addr += 2;
    }
}

#[derive(PartialEq)]
enum DataState {
    Normal,
    Hacked,
}

pub struct GameData {
    pub other_players: HashMap<u32, PlayerData>,
    sprite_id_state: DataState,
    text_state: DataState,
    opponent_state: DataState,
    current_message: Vec<u8>,
}

impl GameData {
    pub fn new() -> GameData {
        GameData {
            other_players: HashMap::new(),
            sprite_id_state: DataState::Normal,
            text_state: DataState::Normal,
            opponent_state: DataState::Normal,
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
        for player in game_data.other_players.values() {
            if player.map_id == map_id {
                if player.map_x == x && player.map_y == y {
                    // If there was a player set a sentinel value so the game thinks that there is
                    // something in the way.
                    mem.sb(offsets::SPRITE_INDEX, 0xFF);
                    game_data.sprite_id_state = DataState::Hacked;
                }
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
        set_battle(cpu, mem, game_data);
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
fn set_battle(cpu: &mut Cpu, mem: &mut Memory, game_data: &mut GameData) {
    mem.sb(offsets::BATTLE_TYPE, values::BattleType::Normal as u8);
    mem.sb(offsets::ACTIVE_BATTLE, values::ActiveBattle::Trainer as u8);
    mem.sb(offsets::TRAINER_NUM, 1);
    mem.sb(offsets::CURRRENT_OPPONENT, values::TrainerClass::ProfOak as u8 + values::TRAINER_TAG);

    let party = Party {
        pokemon: [(1, Pokemon::Weedle), (2, Pokemon::Weedle), (3, Pokemon::Weedle),
        (4, Pokemon::Weedle), (5, Pokemon::Weedle), (6, Pokemon::Weedle)],
    };
    load_party(&party, mem);
}

// FIXME: We need to adjust the approach here to allow for network latency
pub fn extract_player_data(id: u32, mem: &Memory) -> PlayerData {
    // Determine the offset of the player between tiles:
    // When a player begins walking, the delta corresponding to direction the player is moving in is
    // set, and the walk counter is set to 8. For each step of the walk counter, the player's
    // position is moved by two pixels in the specified direction, until the walk counter is 0. When
    // we reach this point, the players map coordinate updated, and the movement delta is cleared.
    //
    // Therefore to determine the player's tile offset, we adjust the walk counter so that it that
    // it starts at 0 and goes to 15, and multiply it by the movement delta.
    let walk_counter = mem.lb(offsets::WALK_COUNTER) as i32;
    let movement = if walk_counter == 0 { 0 } else { 8 - walk_counter } * 2;
    let dx = mem.lb(offsets::PLAYER_DX) as i8 as i32 * movement;
    let dy = mem.lb(offsets::PLAYER_DY) as i8 as i32 * movement;

    PlayerData {
        player_id: id,
        map_id: mem.lb(offsets::MAP_ID),
        pos_x: mem.lb(offsets::MAP_X) as i32 * 16 + dx,
        pos_y: mem.lb(offsets::MAP_Y) as i32 * 16 + dy,
        map_x: mem.lb(offsets::MAP_X),
        map_y: mem.lb(offsets::MAP_Y),
        sprite_index: mem.lb(0xC102),
        direction: mem.lb(offsets::PLAYER_DIR),
    }
}

pub fn extract_player_texture(renderer: &Renderer, mem: &Memory) -> SdlResult<Texture> {
    const SPRITESHEET_WIDTH: uint = 16;
    const SPRITESHEET_HEIGHT: uint = 16 * 6;

    const TILE_SIZE: uint = 8;
    const NUM_X_TILES: uint = SPRITESHEET_WIDTH / TILE_SIZE;
    const NUM_Y_TILES: uint = SPRITESHEET_HEIGHT / TILE_SIZE;

    const BYTES_PER_PIXEL: uint = 4;
    const BUFFER_SIZE: uint = SPRITESHEET_WIDTH * SPRITESHEET_HEIGHT * BYTES_PER_PIXEL;

    let mut output_buffer = [0; BUFFER_SIZE];
    let mut sprite_offset = (offsets::PLAYER_SPRITE_ADDR & 0x3FFF) as uint;

    let (mut tile_x, mut tile_y) = (0, 0);
    while tile_y < NUM_Y_TILES {
        for y in 0..8 {
            // Colors stored in the 2bpp format are split over two bytes. The color's lower bit is
            // stored in the first byte and the high bit is stored in the second byte.
            let color_low = mem.cart.rom[offsets::PLAYER_SPRITE_BANK][sprite_offset];
            let color_high = mem.cart.rom[offsets::PLAYER_SPRITE_BANK][sprite_offset + 1];
            sprite_offset += 2;

            for x in 0..8 {
                let color_id = graphics::get_color_id(color_low, color_high, x);
                let color = graphics::palette_lookup(208, color_id);

                // Compute the offset of where to place this pixel in the output buffer
                let offset = ((((1 - tile_x) * TILE_SIZE) + x) +
                    ((tile_y * TILE_SIZE) + y) * SPRITESHEET_WIDTH) * BYTES_PER_PIXEL;

                output_buffer[offset + 0] = color[0];
                output_buffer[offset + 1] = color[1];
                output_buffer[offset + 2] = color[2];
                output_buffer[offset + 3] = if color_id == 0 { 0 } else { 255 };
            }
        }

        // Step to the next tile
        tile_x += 1;
        if tile_x >= NUM_X_TILES {
            tile_x = 0;
            tile_y += 1;
        }
    }

    let surface = try!(Surface::from_data(&mut output_buffer, SPRITESHEET_WIDTH as int,
        SPRITESHEET_HEIGHT as int, 32, (SPRITESHEET_WIDTH * BYTES_PER_PIXEL) as int, 0, 0, 0, 0));
    renderer.create_texture_from_surface(&surface)
}

//! Module for interfacing with the emulator
use std::collections::RingBuf;
use std::collections::HashMap;

use gb_emu::mmu::Memory;
use gb_emu::graphics;

use common::{SpriteData, PlayerData, PlayerId};
use common::data::{self, Party};

pub mod offsets;
pub mod values;
pub mod extract;
pub mod text;
pub mod hacks;

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
    current_message: RingBuf<u8>,
    sprites_enabled: bool,
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
            current_message: RingBuf::new(),
            sprites_enabled: false,
        }
    }

    pub fn sprites_enabled(&self) -> bool {
        self.sprites_enabled
    }

    pub fn create_message_box(&mut self, input: &str) {
        self.current_message.push_back(text::special::TEXT_START);
        self.current_message.extend(text::Encoder::new(input));
        self.current_message.push_back(text::special::END_MSG);
        self.current_message.push_back(text::special::TERMINATOR);
    }
}

pub fn get_tile_id_addr(x: u8, y: u8) -> u16 {
    let y_offset = (((y + 4) & 0xF0) >> 3) as u16;
    let x_offset = ((x >> 3) + 0x14) as u16;

    offsets::TILE_MAP + 20 * y_offset + x_offset
}

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

// A temporary method to set a battle. In future we probably want to do more of the setup manually
// so that we can do things like set the pokemon moves, EVs, DVs etc.
pub fn set_battle(mem: &mut Memory, party: Party) {
    mem.sb(offsets::BATTLE_TYPE, values::BattleType::Normal as u8);
    mem.sb(offsets::ACTIVE_BATTLE, values::ActiveBattle::Trainer as u8);
    mem.sb(offsets::TRAINER_NUM, 1);
    mem.sb(offsets::CURRRENT_OPPONENT, values::TrainerClass::ProfOak as u8 + values::TRAINER_TAG);

    load_party(party, mem);
}

/// Render a 16x16 sprite
/// Returns true if the sprite was drawn to the screen
pub fn render_sprite(mem: &mut Memory, spritesheet: &[u8], sprite_data: &SpriteData) -> bool {
    const SPRITE_HEIGHT: usize = 16;
    const SPRITE_WIDTH: usize = 16;

    // Check if the sprite actually appears on the screen
    if sprite_data.y >= graphics::HEIGHT as isize || sprite_data.y + SPRITE_HEIGHT as isize <= 0 ||
        sprite_data.x >= graphics::WIDTH as isize || sprite_data.x + SPRITE_WIDTH as isize <= 0
    {
        return false;
    }

    // Check if the sprite is hidden under a menu or a tile
    let tile_addr = get_tile_id_addr(sprite_data.x as u8, sprite_data.y as u8);
    if mem.lb(tile_addr) > values::MAX_MAP_TILE ||
        mem.lb(tile_addr + 1) > values::MAX_MAP_TILE ||
        mem.lb(tile_addr - 20) > values::MAX_MAP_TILE ||
        mem.lb(tile_addr - 19) > values::MAX_MAP_TILE
    {
        return false;
    }

    let sprite_start = sprite_data.index * SPRITE_WIDTH * SPRITE_HEIGHT;
    let sprite = &spritesheet[sprite_start..(sprite_start + SPRITE_WIDTH * SPRITE_HEIGHT)];

    let gpu = &mut mem.gpu;
    let flags = sprite_data.flags;
    let palette = if flags & 0x10 == 0 { gpu.obp0 } else { gpu.obp1 };

    // Render the sprite to the framebuffer
    // Note: Much of this code is similar to gb_emu::graphics::Gpu::render_sprite_scanline, the main
    // differences is that it draws the entire sprite at once.
    for dy in (0..SPRITE_HEIGHT as isize) {
        if sprite_data.y + dy < 0 || sprite_data.y + dy >= graphics::HEIGHT as isize {
            continue;
        }
        let mut current_pos = (sprite_data.y + dy) * graphics::WIDTH as isize + sprite_data.x;
        let tile_y = if flags & 0x40 == 0 { dy as usize } else { SPRITE_HEIGHT - dy as usize - 1 };

        for dx in (0..SPRITE_WIDTH as isize) {
            // Check if this pixel is on the screen
            if sprite_data.x + dx < 0 || sprite_data.x + dx >= graphics::WIDTH as isize {
                current_pos += 1;
                continue;
            }

            let px_priority = gpu.pixel_priorities[current_pos as usize];
            let tile_x = if flags & 0x20 == 0 { SPRITE_WIDTH - dx as usize - 1 }
                         else { dx as usize };
            let color_id = sprite[tile_y * SPRITE_WIDTH + tile_x];

            if color_id != 0 && (flags & 0x80 == 0 || px_priority == 0) && px_priority <= 3 {
                let color = graphics::palette_lookup(palette, color_id as usize);
                graphics::write_pixel(&mut gpu.framebuffer,
                    current_pos as usize * graphics::BYTES_PER_PIXEL, color);
            }

            current_pos += 1;
        }
    }

    true
}


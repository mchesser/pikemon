use std::iter;
use num::FromPrimitive;

use gb_emu::mmu::Memory;
use gb_emu::graphics;

use values::Direction;
use data::{Party, BATTLE_DATA_SIZE, BattleData, PokemonData, MovementData};

use offsets;
use text;

pub fn movement_data(mem: &Memory) -> MovementData {
    MovementData {
        map_id: mem.lb(offsets::MAP_ID),
        map_x: mem.lb(offsets::MAP_X),
        map_y: mem.lb(offsets::MAP_Y),
        direction: Direction::from_u8(mem.lb(offsets::PLAYER_DIR)).unwrap_or(Direction::Down),
        walk_counter: mem.lb(offsets::WALK_COUNTER),
    }
}

pub fn player_name(mem: &Memory) -> Vec<u8> {
    let mut name = vec![];

    let mut offset = offsets::PLAYER_NAME_START;
    for _ in 0..11 {
        match mem.lb(offset) {
            text::special::TERMINATOR => break,
            val => name.push(val),
        }
        offset += 1;
    }
    name
}

pub fn battle_data(mem: &Memory) -> BattleData {
    let base_offset = offsets::PLAYER_BATTLE_DATA_START;
    (0..BATTLE_DATA_SIZE as u16).map(|i| mem.lb(base_offset + i)).collect()
}

fn pokemon_data(mem: &Memory, addr: u16) -> PokemonData {
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

// Currently this has been changed to use a more specific method, however we may want to use this
// for other things in the future. (e.g. server trainers)
pub fn player_party(mem: &Memory) -> Party {
    Party {
        num_pokemon: mem.lb(offsets::PARTY_COUNT),
        pokemon: (pokemon_data(mem, offsets::PARTY_POKE_1),
            pokemon_data(mem, offsets::PARTY_POKE_2),
            pokemon_data(mem, offsets::PARTY_POKE_3),
            pokemon_data(mem, offsets::PARTY_POKE_4),
            pokemon_data(mem, offsets::PARTY_POKE_5),
            pokemon_data(mem, offsets::PARTY_POKE_6)),
    }
}

pub fn default_sprite(mem: &Memory) -> Vec<u8> {
    extract_sprite(mem, offsets::BLUE_SPRITE_BANK, offsets::BLUE_SPRITE_ADDR)
}

const TILE_SIZE: usize = 8;

fn extract_sprite(mem: &Memory, bank: usize, addr: u16) -> Vec<u8> {
    const SPRITE_SIZE: usize = 16;
    const NUM_ELEMENTS: usize = 6;
    const NUM_TILES: usize = 4 * NUM_ELEMENTS;
    const BUFFER_SIZE: usize = SPRITE_SIZE * SPRITE_SIZE * NUM_ELEMENTS;

    let mut buffer: Vec<_> = iter::repeat(0).take(BUFFER_SIZE).collect();
    let mut sprite_offset = (addr & 0x3FFF) as usize;

    let (mut tile_x, mut tile_y) = (0, 0);
    while tile_x + 2 * tile_y  < NUM_TILES {
        for y in (0..TILE_SIZE) {
            // Colors stored in the 2bpp format are split over two bytes. The color's lower bit is
            // stored in the first byte and the high bit is stored in the second byte.
            let color_low = mem.cart.rom[bank][sprite_offset];
            let color_high = mem.cart.rom[bank][sprite_offset + 1];
            sprite_offset += 2;

            for x in (0..TILE_SIZE) {
                let color_id = graphics::get_color_id(color_low, color_high, x) as u8;

                // Each 16x16 block has the tiles store in reverse, so we have to be careful when
                // assembling the complete sprite
                buffer[(((1 - tile_x) * TILE_SIZE) + x) +
                    (tile_y * TILE_SIZE + y) * 2 * TILE_SIZE] = color_id;
            }
        }

        // Step to the next tile
        tile_x += 1;
        if tile_x >= 2 {
            tile_x = 0;
            tile_y += 1;
        }
    }

    buffer
}

#[derive(Clone, Copy)]
pub enum TextureFormat {
    Bpp1 = 1,
    Bpp2 = 2,
}

pub fn extract_texture(mem: &Memory, bank: usize, addr: u16, width: usize, height: usize,
    format: TextureFormat, palette: &[graphics::Color]) -> Vec<u8>
{
    const BYTES_PER_PIXEL_OUT: usize = 4;

    let num_x_tiles = width / TILE_SIZE;
    let num_y_tiles = height / TILE_SIZE;

    let mut output_buffer = vec![0; width * height * BYTES_PER_PIXEL_OUT];
    let mut sprite_offset = (addr & 0x3FFF) as usize;

    let (mut tile_x, mut tile_y) = (0, 0);
    while tile_y < num_y_tiles {
        for y in (0..8) {
            let color_low = mem.cart.rom[bank][sprite_offset];
            let color_high = mem.cart.rom[bank][sprite_offset + 1];

            sprite_offset += format as usize;

            for x in (0..8) {
                let color = match format {
                    TextureFormat::Bpp1 => {
                        if color_low & (1 << (7 - x)) != 0 { palette[0] } else { palette[1] }
                    },
                    TextureFormat::Bpp2 => {
                        palette[graphics::get_color_id(color_low, color_high, 7 - x)]
                    },
                };

                // Compute the offset of where to place this pixel in the output buffer
                let offset = (((tile_x * TILE_SIZE) + x) +
                    ((tile_y * TILE_SIZE) + y) * width) * BYTES_PER_PIXEL_OUT;
                output_buffer[offset + 0] = color[0];
                output_buffer[offset + 1] = color[1];
                output_buffer[offset + 2] = color[2];
                output_buffer[offset + 3] = color[3];
            }
        }

        // Step to the next tile
        tile_x += 1;
        if tile_x >= num_x_tiles {
            tile_x = 0;
            tile_y += 1;
        }
    }

    output_buffer
}

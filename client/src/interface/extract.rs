use std::iter;

use sdl2::SdlResult;
use sdl2::surface::Surface;
use sdl2::render::{Renderer, Texture};

use gb_emu::mmu::Memory;
use gb_emu::graphics;

use common::{PlayerData, MovementData};
use common::data::{Party, PokemonData};

use interface::offsets;
use interface::text;

fn movement_data(mem: &Memory) -> MovementData {
    MovementData {
        map_id: mem.lb(offsets::MAP_ID),
        map_x: mem.lb(offsets::MAP_X),
        map_y: mem.lb(offsets::MAP_Y),
        direction: mem.lb(offsets::PLAYER_DIR),
        walk_counter: mem.lb(offsets::WALK_COUNTER),
    }
}

pub fn player_data(mem: &Memory) -> PlayerData {
    let mut name = vec![];

    let mut offset = offsets::PLAYER_NAME_START;
    for _ in 0..11 {
        match mem.lb(offset) {
            text::special::TERMINATOR => break,
            val => name.push(val),
        }
        offset += 1;
    }

    PlayerData {
        name: name,
        movement_data: movement_data(mem),
    }
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

pub fn player_texture(renderer: &Renderer, mem: &Memory) -> SdlResult<Texture> {
    extract_2bpp_sprite_texture(renderer, mem, offsets::PLAYER_SPRITE_BANK,
        offsets::PLAYER_SPRITE_ADDR, 16, 16 * 6)
}

pub fn font_texture(renderer: &Renderer, mem: &Memory) -> SdlResult<Texture> {
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

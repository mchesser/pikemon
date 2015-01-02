//! Module for interfacing with the emulator
use std::collections::HashMap;

use sdl2::SdlResult;
use sdl2::surface::Surface;
use sdl2::render::{Renderer, Texture};

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::graphics;

use common::PlayerData;

mod offsets {
    // The current map id
    pub const MAP_ID: u16 = 0xD35E;
    // The player's Y coordinate on the current map
    pub const MAP_Y: u16 = 0xD361;
    // The player's Y coordinate on the current map
    pub const MAP_X: u16 = 0xD362;
    // The player's Y movement delta
    pub const PLAYER_DY: u16 = 0xC103;
    // The player's X movement delta
    pub const PLAYER_DX: u16 = 0xC105;
    // The direction which the player is facing (0: down, 4: up, 8: left, 10: right)
    pub const PLAYER_DIR: u16 = 0xC109;
    // When a player moves, this value counts down from 8 to 0
    pub const WALK_COUNTER: u16 = 0xCFC5;

    pub const PLAYER_SPRITE_ADDR: u16 = 0x4180;
    pub const PLAYER_SPRITE_BANK: uint = 5;

    pub const NUM_SPRITES: u16 = 0xD4E1;
    pub const COLLISION_CHECKING_EXIT_1: u16 = 0x0BA0;
    pub const COLLISION_CHECKING_EXIT_2: u16 = 0x0BC4;

    pub const SPRITE_INDEX: u16 = 0xFF8C;
}

pub fn collision_manager(cpu: &mut Cpu, mem: &mut Memory,
    other_players: &mut HashMap<u32, PlayerData>)
{
    if (cpu.pc == offsets::COLLISION_CHECKING_EXIT_1 && mem.lb(offsets::NUM_SPRITES) == 0) ||
        cpu.pc == offsets::COLLISION_CHECKING_EXIT_2
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
        for player in other_players.values() {
            if player.map_id == map_id {
                if player.map_x == x && player.map_y == y {
                    mem.sb(offsets::SPRITE_INDEX, 0xFF);
                }
            }
        }
    }
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

    let mut output_buffer = [0, ..BUFFER_SIZE];
    let mut sprite_offset = (offsets::PLAYER_SPRITE_ADDR & 0x3FFF) as uint;

    let (mut tile_x, mut tile_y) = (0, 0);
    while tile_y < NUM_Y_TILES {
        for y in (0..8) {
            // Colors stored in the 2bpp format are split over two bytes. The color's lower bit is
            // stored in the first byte and the high bit is stored in the second byte.
            let color_low = mem.cart.rom[offsets::PLAYER_SPRITE_BANK][sprite_offset];
            let color_high = mem.cart.rom[offsets::PLAYER_SPRITE_BANK][sprite_offset + 1];
            sprite_offset += 2;

            for x in (0..8) {
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

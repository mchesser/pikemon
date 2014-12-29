use std::slice::bytes::copy_memory;
use std::collections::HashMap;

use sdl2;
use sdl2::event::Event;
use sdl2::event::poll_event;
use sdl2::keycode::KeyCode;
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;
use sdl2::pixels::Color;
use sdl2::render;
use sdl2::render::{Renderer, RenderDriverIndex, RendererFlip};
use sdl2::surface::Surface;
use sdl2::rect::Rect;

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::joypad;

use common::PlayerData;

use timer::Timer;
use net::ClientDataManager;
use sprite::Sprite;

const SCALE: int = 1;
const WIDTH: int = graphics::WIDTH as int * SCALE;
const HEIGHT: int = graphics::HEIGHT as int * SCALE;

const SRC_WIDTH: uint = graphics::WIDTH as uint;
const SRC_HEIGHT: uint = graphics::HEIGHT as uint;

pub fn run<F>(mut data: ClientDataManager, mut emulator: Box<Emulator<F>>)
    where F: FnMut(&mut Cpu, &mut Memory)
{
    sdl2::init(sdl2::INIT_EVERYTHING);

    let window = match Window::new("Pikemon", PosCentered, PosCentered, WIDTH, HEIGHT, OPENGL) {
        Ok(window) => window,
        Err(e) => panic!("failed to create window: {}", e)
    };

    let renderer = match Renderer::from_window(window, RenderDriverIndex::Auto, render::ACCELERATED)
    {
        Ok(window) => window,
        Err(e) => panic!("failed to create renderer: {}", e)
    };
    let _ = renderer.set_draw_color(Color::RGB(0xFF, 0, 0));

    let mut sprite_data = extract_player_sprite(&mut emulator.mem);
    let sprite_surface = Surface::from_data(&mut sprite_data, SPRITESHEET_WIDTH as int,
                SPRITESHEET_HEIGHT as int, 32, SPRITESHEET_WIDTH as int * 4, 0, 0, 0, 0).unwrap();
    let sprite_tex = renderer.create_texture_from_surface(&sprite_surface).unwrap();
    let sprite = Sprite::new(sprite_tex, 16, 16, SCALE as i32);

    let mut fast_mode = false;

    let mut timer = Timer::new();
    'main: loop {
        'event: loop {
            match poll_event() {
                Event::Quit(_) => break 'main,

                Event::KeyDown(_, _, code, _, _, _) => {
                    handle_joypad_event(&mut emulator.mem.joypad, code, joypad::State::Pressed);
                    if code == KeyCode::Space { fast_mode = true; }
                },

                Event::KeyUp(_, _, code, _, _, _) => {
                    handle_joypad_event(&mut emulator.mem.joypad, code, joypad::State::Released);
                    if code == KeyCode::Space { fast_mode = false; }
                },

                Event::None => break,
                _ => continue,
            }
        }


        if fast_mode || timer.elapsed_seconds() >= 1.0 / 60.0 {
            timer.reset();
            emulator.frame();

            let id = data.last_state.player_id;
            data.update(extract_player_data(id, &emulator.mem));
        }

        if emulator.mem.gpu.ready_flag {
            emulator.mem.gpu.ready_flag = false;
            let emulator_surface = Surface::from_data(emulator.display_mut(), SRC_WIDTH as int,
                SRC_HEIGHT as int, 32, SRC_WIDTH as int * 4, 0, 0, 0, 0).unwrap();
            let emulator_texture = renderer.create_texture_from_surface(&emulator_surface).unwrap();

            let _ = renderer.clear();
            let _ = renderer.copy(&emulator_texture, None, None);

            let self_data = data.last_state;
            for player in data.other_players.values() {
                if player.map_id == self_data.map_id {
                    let x = (player.pos_x - self_data.pos_x) + (WIDTH / 2) as i32 - 16 * SCALE as i32;
                    let y = (player.pos_y - self_data.pos_y) + (HEIGHT / 2) as i32  - 12 * SCALE  as i32;
                    if player.sprite_index != 0xFF {
                        let (frame, flip) = determine_frame_index_and_flip(player.sprite_index);
                        sprite.draw(&renderer, x, y, frame, flip);
                    }
                }
            }

            renderer.present();
        }
    }
}

fn determine_frame_index_and_flip(sprite_index: u8) -> (i32, RendererFlip) {
    let (mut index, flip) = match sprite_index & 0xC {
        0  => (0, RendererFlip::None),          // Down
        4  => (1, RendererFlip::None),          // Up
        8  => (2, RendererFlip::Horizontal),    // Right
        12 => (2, RendererFlip::None),          // Left

        // Unreachable
        _  => (0, RendererFlip::None),
    };

    index += match sprite_index & 0x3 {
        0 => 0,
        1 => 3,
        2 => 0,
        3 => 3,

        // Unreachable
        _ => 0,
    };

    (index, flip)
}

const SPRITESHEET_WIDTH: uint = 16;
const SPRITESHEET_HEIGHT: uint = 16 * 6;
const BUFFER_SIZE: uint = SPRITESHEET_WIDTH * SPRITESHEET_HEIGHT * 4;

fn extract_player_sprite(mem: &Memory) -> [u8, ..BUFFER_SIZE] {
    let mut decode_buffer = [0, ..BUFFER_SIZE];

    let bank_num = 5;
    let mut sprite_offset = 0x4180 & 0x3FFF;

    let mut tile_x = 0;
    let mut tile_y = 0;
    for _ in range(0, SPRITESHEET_WIDTH / 8 * SPRITESHEET_HEIGHT / 8) {
        for y in range(0, 8) {
            let low = mem.rom[bank_num][sprite_offset];
            let high = mem.rom[bank_num][sprite_offset + 1];
            sprite_offset += 2;
            for x in range(0, 8) {
                let color_id = ((((high >> x) & 1) << 1) | ((low >> x) & 1)) as uint;
                let color = graphics::palette_lookup(208, color_id);

                let offset = ((((1 - tile_x) * 8) + x) + ((tile_y * 8) + y) * 16) * 4;
                decode_buffer[offset + 0] = color[0];
                decode_buffer[offset + 1] = color[1];
                decode_buffer[offset + 2] = color[2];
                decode_buffer[offset + 3] = if color_id == 0 { 0 } else { 255 };
            }
        }
        tile_x += 1;
        if tile_x == 2 {
            tile_x = 0;
            tile_y += 1;
        }
    }
    decode_buffer
}

fn extract_player_data(id: u32, mem: &Memory) -> PlayerData {
    // The current map id
    const MAP_ID: u16 = 0xD35E;
    // The player's Y coordinate on the current map
    const MAP_Y: u16 = 0xD361;
    // The player's Y coordinate on the current map
    const MAP_X: u16 = 0xD362;
    // The player's Y movement delta
    const PLAYER_DY: u16 = 0xC103;
    // The player's X movement delta
    const PLAYER_DX: u16 = 0xC105;
    // The direction which the player is facing (0: down, 4: up, 8: left, 16: right)
    const PLAYER_DIR: u16 = 0xC109;
    // When a player moves, this value counts down from 8 to 0
    const WALK_COUNTER: u16 = 0xCFC5;

    // Determine the offset of the player between tiles:
    // When a player begins walking, the delta corresponding to direction the player is moving in is
    // set, and the walk counter is set to 8. For each step of the walk counter, the player's
    // position is moved by two pixels in the specified direction, until the walk counter is 0. When
    // we reach this point, the players map coordinate updated, and the movement delta is cleared.
    //
    // Therefore to determine the player's tile offset, we adjust the walk counter so that it that
    // it starts at 0 and goes to 15, and multiply it by the movement delta.
    let walk_counter = mem.lb(WALK_COUNTER) as i32;
    let movement = if walk_counter == 0 { 0 } else { 8 - walk_counter } * 2;
    let dx = mem.lb(PLAYER_DX) as i8 as i32 * movement;
    let dy = mem.lb(PLAYER_DY) as i8 as i32 * movement;

    PlayerData {
        player_id: id,
        map_id: mem.lb(MAP_ID),
        pos_x: mem.lb(MAP_X) as i32 * 16 + dx,
        pos_y: mem.lb(MAP_Y) as i32 * 16 + dy,
        sprite_index: mem.lb(0xC102),
        direction: mem.lb(PLAYER_DIR),
    }
}

fn handle_joypad_event(joypad: &mut joypad::Joypad, keycode: KeyCode, state: joypad::State) {
    // TODO: Add custom keybindings
    match keycode {
        KeyCode::Up => joypad.up = state,
        KeyCode::Down => joypad.down = state,
        KeyCode::Left => joypad.left = state,
        KeyCode::Right => joypad.right = state,

        KeyCode::Z => joypad.a = state,
        KeyCode::X => joypad.b = state,
        KeyCode::Return => joypad.start = state,
        KeyCode::RShift => joypad.select = state,

        _ => {},
    }
}

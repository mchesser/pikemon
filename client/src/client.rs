use std::slice::bytes::copy_memory;
use std::collections::HashMap;

use sdl2;
use sdl2::event::Event;
use sdl2::event::poll_event;
use sdl2::keycode::KeyCode;
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;
use sdl2::surface::Surface;

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::joypad;

use common::PlayerData;

use timer::Timer;
use net::ClientDataManager;

const SCALE: int = 1;
const WIDTH: int = graphics::WIDTH as int * SCALE;
const HEIGHT: int = graphics::HEIGHT as int * SCALE;

const SRC_WIDTH: uint = graphics::WIDTH as uint;
const SRC_HEIGHT: uint = graphics::HEIGHT as uint;

pub fn run<F>(mut data: ClientDataManager, mut emulator: Box<Emulator<F>>)
    where F: FnMut(&mut Cpu, &mut Memory)
{
    sdl2::init(sdl2::INIT_EVERYTHING);

    let window = match Window::new("GameBoy Emulator", PosCentered, PosCentered,
        WIDTH, HEIGHT, OPENGL)
    {
        Ok(window) => window,
        Err(err) => panic!("failed to create window: {}", err)
    };

    let mut surface = match window.get_surface() {
        Ok(surface) => surface,
        Err(err) => panic!("failed to get window surface: {}", err)
    };

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
            render_screen(&mut surface, emulator.display());

            let self_data = data.last_state;
            for player in data.other_players.values() {
                if player.map_id == self_data.map_id {
                    let x = (player.pos_x as i32 - self_data.pos_x as i32) * 16 +
                            (WIDTH / 2) as i32 - 8;
                    let y = (player.pos_y as i32 - self_data.pos_y as i32) * 16 +
                            (HEIGHT / 2) as i32  - 8;

                    if x >= 0 && y >= 0 && x + 16 < WIDTH as i32 && y + 16 < HEIGHT as i32 {
                        draw_square(&mut surface, x as uint, y as uint, 16, 16);
                    }
                }
            }

            window.update_surface();
        }
    }
}

fn extract_player_data(id: u32, mem: &Memory) -> PlayerData {
    PlayerData {
        player_id: id,
        map_id: mem.lb(0xD35E),
        pos_x: mem.lb(0xD362),
        pos_y: mem.lb(0xD361),
        direction: mem.lb(0xC109),
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

fn render_screen(surface: &mut Surface, display: &[u8]) {
    surface.with_lock(|pixels| copy_memory(pixels, display));
}

fn draw_square(surface: &mut Surface, x: uint, y: uint, width: uint, height: uint) {
    surface.with_lock(|pixels|
        for dy in range(0, height) {
            for dx in range(0, width) {
                let draw_pos = ((y + dy) * SRC_WIDTH + (x + dx)) * 4;
                pixels[draw_pos + 0] = 0xFF;
                pixels[draw_pos + 1] = 0x00;
                pixels[draw_pos + 2] = 0x00;
                pixels[draw_pos + 3] = 0xFF;
            }
        }
    );
}

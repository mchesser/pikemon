use std::slice::bytes::copy_memory;

use sdl2;
use sdl2::SdlResult;
use sdl2::event::{Event, poll_event};
use sdl2::keycode::KeyCode;
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;
use sdl2::render::{self, Renderer, RenderDriverIndex, RendererFlip, BlendMode, TextureAccess};
use sdl2::pixels::PixelFormatFlag;

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::joypad;

use common::PlayerData;

use timer::Timer;
use net::ClientDataManager;
use sprite::Sprite;
use interface::{extract_player_data, extract_player_texture, GameState};

const SCALE: i32 = 2;
const WIDTH: int = graphics::WIDTH as int * (SCALE as int);
const HEIGHT: int = graphics::HEIGHT as int * (SCALE as int);

pub fn run<F>(mut data: ClientDataManager, mut emulator: Box<Emulator<F>>) -> SdlResult<()>
    where F: FnMut(&mut Cpu, &mut Memory)
{
    sdl2::init(sdl2::INIT_EVERYTHING);

    let window = try!(Window::new("Pikemon", PosCentered, PosCentered, WIDTH, HEIGHT, OPENGL));
    let renderer = try!(Renderer::from_window(window, RenderDriverIndex::Auto,
        render::ACCELERATED));

    let player_texture = try!(extract_player_texture(&renderer, &mut emulator.mem));
    try!(player_texture.set_blend_mode(BlendMode::Blend));
    let player_sprite = Sprite::new(player_texture, 16, 16, SCALE as i32);

    let screen_texture = try!(renderer.create_texture(PixelFormatFlag::ARGB8888,
        TextureAccess::Streaming, graphics::WIDTH as int, graphics::HEIGHT as int));

    let mut fast_mode = false;

    let mut emulator_timer = Timer::new();
    let mut network_timer = Timer::new();

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

        if fast_mode || emulator_timer.elapsed_seconds() >= 1.0 / 60.0 {
            emulator_timer.reset();
            let game_ready = data.game_data.borrow().game_state == GameState::Normal;
            if game_ready {
                emulator.frame();
                data.update_player_data(extract_player_data(&emulator.mem));
            }
        }

        if network_timer.elapsed_seconds() >= 1.0 / 60.0 {
            network_timer.reset();
            data.send_update();
            data.recv_update(&mut emulator.mem);
        }

        // If there is a new screen ready, copy the internal framebuffer to the screen texture
        if emulator.poll_screen() {
            try!(screen_texture.with_lock(None, |mut pixels, _| {
                copy_memory(pixels.as_mut_slice(), emulator.front_buffer());
            }));
        }

        try!(renderer.clear());

        // Draw the screen
        try!(renderer.copy(&screen_texture, None, None));

        // Draw the players
        let self_data = data.last_state;
        for player in data.game_data.borrow().other_players.values() {
            if player.map_id == self_data.map_id {
                let (x, y) = get_player_draw_position(&self_data, player);
                let (frame, flip) = determine_frame_index_and_flip(player.direction,
                    player.walk_counter);
                try!(player_sprite.draw(&renderer, x * SCALE, y * SCALE, frame, flip));
            }
        }

        renderer.present();
    }
    Ok(())
}

fn get_player_draw_position(self_player: &PlayerData, other_player: &PlayerData) -> (i32, i32) {
    let base_x = (graphics::WIDTH as i32) / 2 - 16;
    let base_y = (graphics::HEIGHT as i32) / 2 - 12;

    let (self_x, self_y) = get_player_position(self_player);
    let (other_x, other_y) = get_player_position(other_player);

    (other_x - self_x + base_x, other_y - self_y + base_y)
}

fn get_player_position(player: &PlayerData) -> (i32, i32) {
    let x = player.map_x as i32 * 16;
    let y = player.map_y as i32 * 16;

    // Determine the offset of the player between tiles:
    // When a player begins walking, the walk counter is set to 8. For each step the walk counter
    // decreases by one, and the player is moved by two pixels, until the walk counter is 0. When
    // we reach this point, the players map coordinate updated.
    let offset = if player.walk_counter == 0 { 0 } else { (8 - player.walk_counter) * 2 } as i32;

    match player.direction {
        0  => (x, y + offset),
        4  => (x, y - offset),
        8  => (x - offset, y),
        12 => (x + offset, y),

        _  => (x, y), // Usually unreachable
    }
}

fn determine_frame_index_and_flip(direction: u8, walk_counter: u8) -> (i32, RendererFlip) {
    let (mut index, flip) = match direction {
        0  => (0, RendererFlip::None),          // Down
        4  => (1, RendererFlip::None),          // Up
        8  => (2, RendererFlip::Horizontal),    // Right
        12 => (2, RendererFlip::None),          // Left

        _  => (0, RendererFlip::None),          // Usually unreachable
    };

    index += match walk_counter / 4 {
        0 => 0,
        1 => 3,

        _ => 0, // Usually unreachable
    };

    (index, flip)
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

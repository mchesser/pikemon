use std::slice::bytes::copy_memory;

use sdl2;
use sdl2::SdlResult;
use sdl2::event::{Event, poll_event};
use sdl2::keycode::KeyCode;
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;
use sdl2::render;
use sdl2::render::{Renderer, RenderDriverIndex, RendererFlip, TextureAccess};
use sdl2::pixels::PixelFormatFlag;

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::joypad;

// use common::PlayerData;

use timer::Timer;
use net::ClientDataManager;
use sprite::Sprite;
use interface::{extract_player_data, extract_player_texture};

const SCALE: int = 2;
const WIDTH: int = graphics::WIDTH as int * SCALE;
const HEIGHT: int = graphics::HEIGHT as int * SCALE;

pub fn run<F>(mut data: ClientDataManager, mut emulator: Box<Emulator<F>>) -> SdlResult<()>
    where F: FnMut(&mut Cpu, &mut Memory)
{
    sdl2::init(sdl2::INIT_EVERYTHING);

    let window = try!(Window::new("Pikemon", PosCentered, PosCentered, WIDTH, HEIGHT, OPENGL));
    let renderer = try!(Renderer::from_window(window, RenderDriverIndex::Auto,
        render::ACCELERATED));

    let player_texture = try!(extract_player_texture(&renderer, &mut emulator.mem));
    let player_sprite = Sprite::new(player_texture, 16, 16, SCALE as i32);

    let screen_texture = try!(renderer.create_texture(PixelFormatFlag::ARGB8888,
        TextureAccess::Streaming, graphics::WIDTH as int, graphics::HEIGHT as int));

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
        for player in data.other_players.borrow().values() {
            if player.map_id == self_data.map_id {
                let x = (player.pos_x - self_data.pos_x - 16) * SCALE as i32 +
                    (WIDTH / 2) as i32;
                let y = (player.pos_y - self_data.pos_y - 12) * SCALE as i32 +
                    (HEIGHT / 2) as i32;

                if player.sprite_index != 0xFF {
                    let (frame, flip) = determine_frame_index_and_flip(player.sprite_index);
                    try!(player_sprite.draw(&renderer, x, y, frame, flip));
                }
            }
        }

        renderer.present();
    }
    Ok(())
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

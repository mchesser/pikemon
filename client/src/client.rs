use std::slice::bytes::copy_memory;

use sdl2;
use sdl2::rect::Rect;
use sdl2::SdlResult;
use sdl2::event::{Event, poll_event};
use sdl2::keycode::KeyCode;
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;
use sdl2::render::{self, Renderer, RenderDriverIndex, TextureAccess};
use sdl2::pixels::{PixelFormatFlag, Color};

use gb_emu::mmu::Memory;
use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::joypad;

use common::{PlayerData, SpriteData};

use timer::Timer;
use net::ClientDataManager;
use interface::{self, extract, hacks, GameState};
use font::Font;

const EMU_SCALE: i32 = 3;
const EMU_WIDTH: i32 = graphics::WIDTH as i32 * EMU_SCALE;
const EMU_HEIGHT: i32 = graphics::HEIGHT as i32 * EMU_SCALE;

const CHAT_WIDTH: i32 = 250;
const FONT_SCALE: i32 = 1;

enum KeyboardTarget {
    Emulator,
    ChatBox,
    Menu,
}

pub fn run(mut data: ClientDataManager, mut emulator: Box<Emulator>) -> SdlResult<()> {
    const WHITE: Color = Color::RGB(0xFF, 0xFF, 0xFF);

    sdl2::init(sdl2::INIT_EVERYTHING);

    let window = try!(Window::new("Pikemon", PosCentered, PosCentered, EMU_WIDTH + CHAT_WIDTH,
        EMU_HEIGHT, OPENGL));
    let renderer = try!(Renderer::from_window(window, RenderDriverIndex::Auto,
        render::ACCELERATED));

    let player_sprite = extract::default_sprite(&mut emulator.mem);

    let font_texture = try!(extract::font_texture(&renderer, &mut emulator.mem));
    let font_data = Font::new(font_texture, 8, 8, FONT_SCALE);

    let emu_dest_rect = Rect::new(0, 0, EMU_WIDTH, EMU_HEIGHT);
    let emu_texture = try!(renderer.create_texture(PixelFormatFlag::ARGB8888,
        TextureAccess::Streaming, graphics::WIDTH as i32, graphics::HEIGHT as i32));

    let mut keyboard_target = KeyboardTarget::Emulator;
    let mut fast_mode = false;

    let mut emulator_timer = Timer::new();
    let mut network_timer = Timer::new();

    'main: loop {
        'event: loop {
            match poll_event() {
                Event::Quit(_) => break 'main,

                Event::KeyDown(_, _, code, _, _, _) => {
                    match keyboard_target {
                        KeyboardTarget::Emulator => {
                            handle_joypad_event(&mut emulator.mem.joypad, code,
                                joypad::State::Pressed);
                            if code == KeyCode::Space { fast_mode = true; }
                        },

                        KeyboardTarget::ChatBox => {
                            handle_keyboard_chat(&mut data, code);
                        },

                        _ => unimplemented!(),
                    }
                },

                Event::KeyUp(_, _, code, _, _, _) => {
                    match keyboard_target {
                        KeyboardTarget::Emulator => {
                            handle_joypad_event(&mut emulator.mem.joypad, code,
                                joypad::State::Released);
                            if code == KeyCode::Space { fast_mode = false; }
                            if code == KeyCode::T { keyboard_target = KeyboardTarget::ChatBox; }
                        },

                        KeyboardTarget::ChatBox => {
                            if code == KeyCode::Return {
                                keyboard_target = KeyboardTarget::Emulator;
                            }
                        },

                        _ => unimplemented!(),
                    }
                },

                Event::None => break,
                _ => continue,
            }
        }

        if fast_mode || emulator_timer.elapsed_seconds() >= 1.0 / 60.0 {
            emulator_timer.reset();
            let game_ready = data.game_data.borrow().game_state == GameState::Normal;
            if game_ready {
                emulator.frame(
                    |cpu, mem| {
                        hacks::sprite_check(cpu, mem, &mut *data.game_data.borrow_mut());
                        hacks::display_text(cpu, mem, &mut *data.game_data.borrow_mut());
                        hacks::sprite_update_tracker(cpu, mem, &mut *data.game_data.borrow_mut());
                    },

                    |_, mem| {
                        if data.game_data.borrow().sprites_enabled() {
                            draw_other_players(&data, mem, &*player_sprite);
                        }

                        // Copy the screen to the emulator texture
                        let _ = emu_texture.with_lock(None, |mut pixels, _| {
                            copy_memory(pixels.as_mut_slice(), &mem.gpu.framebuffer);
                        });
                    }
                );

                data.update_player_data(extract::player_data(&emulator.mem));
            }
        }

        if network_timer.elapsed_seconds() >= 1.0 / 60.0 {
            network_timer.reset();
            data.send_update();
            data.recv_update(&mut emulator.mem);
        }

        try!(renderer.set_draw_color(WHITE));
        try!(renderer.clear());

        // Draw the screen
        try!(renderer.copy(&emu_texture, None, Some(emu_dest_rect)));

        try!(data.chat_box.draw(&renderer, &font_data, &Rect::new(EMU_WIDTH, 0, CHAT_WIDTH,
            EMU_HEIGHT)));

        renderer.present();
    }
    Ok(())
}

fn draw_other_players(data: &ClientDataManager, mem: &mut Memory, sprite: &[u8]) {
    let self_data = &data.last_state;
    for player in data.game_data.borrow().other_players.values() {
        if player.is_visible_to(self_data) {
            let (x, y) = get_player_draw_position(self_data, player);
            let (index, flags) = get_sprite_index_and_flags(player.movement_data.direction,
                player.movement_data.walk_counter);
            let sprite_data = SpriteData {
                x: x as isize,
                y: y as isize,
                index: index as usize,
                flags: flags,
            };
            interface::render_sprite(mem, sprite, &sprite_data);
        }
    }
}

fn get_player_draw_position(self_player: &PlayerData, other_player: &PlayerData) -> (i32, i32) {
    let base_x = (graphics::WIDTH as i32) / 2 - 16;
    let base_y = (graphics::HEIGHT as i32) / 2 - 12;

    let (self_x, self_y) = get_player_position(self_player);
    let (other_x, other_y) = get_player_position(other_player);

    (other_x - self_x + base_x, other_y - self_y + base_y)
}

fn get_player_position(player: &PlayerData) -> (i32, i32) {
    let x = player.movement_data.map_x as i32 * 16;
    let y = player.movement_data.map_y as i32 * 16;

    // Determine the offset of the player between tiles:
    // When a player begins walking, the walk counter is set to 8. For each step the walk counter
    // decreases by one, and the player is moved by two pixels, until the walk counter is 0. When
    // we reach this point, the players map coordinate updated.
    let ticks = player.movement_data.walk_counter;
    let offset = if ticks == 0 { 0 } else { (8 - ticks) * 2 } as i32;

    match player.movement_data.direction {
        0  => (x, y + offset),
        4  => (x, y - offset),
        8  => (x - offset, y),
        12 => (x + offset, y),

        _  => (x, y), // Usually unreachable
    }
}

fn get_sprite_index_and_flags(direction: u8, walk_counter: u8) -> (isize, u8) {
    let (mut index, mut flags) = match direction {
        0  => (0, 0x00),    // Down
        4  => (1, 0x00),    // Up
        8  => (2, 0x00),    // Right
        12 => (2, 0x20),    // Left

        _  => (0, 0x00),    // Usually unreachable
    };

    flags |= 0x80;

    index += match walk_counter / 4 {
        0 => 0,
        1 => 3,

        _ => 0, // Usually unreachable
    };

    (index, flags)
}

fn handle_keyboard_chat(client_data: &mut ClientDataManager, key_code: KeyCode) {
    let letter = match key_code {
        KeyCode::Return => { client_data.send_message(); return },
        KeyCode::Backspace => { client_data.chat_box.remove_char(); return },
        KeyCode::Space => ' ',
        KeyCode::A => 'a',
        KeyCode::B => 'b',
        KeyCode::C => 'c',
        KeyCode::D => 'd',
        KeyCode::E => 'e',
        KeyCode::F => 'f',
        KeyCode::G => 'g',
        KeyCode::H => 'h',
        KeyCode::I => 'i',
        KeyCode::J => 'j',
        KeyCode::K => 'k',
        KeyCode::L => 'l',
        KeyCode::M => 'm',
        KeyCode::N => 'n',
        KeyCode::O => 'o',
        KeyCode::P => 'p',
        KeyCode::Q => 'q',
        KeyCode::R => 'r',
        KeyCode::S => 's',
        KeyCode::T => 't',
        KeyCode::U => 'u',
        KeyCode::V => 'v',
        KeyCode::W => 'w',
        KeyCode::X => 'x',
        KeyCode::Y => 'y',
        KeyCode::Z => 'z',
        _ => return,
    };

    client_data.chat_box.push_char(letter);
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

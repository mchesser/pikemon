use std::mem;
use std::cell::RefCell;
use std::slice::bytes::copy_memory;

use sdl2::rect::Rect;
use sdl2::keycode::KeyCode;
use sdl2::render::{RenderDrawer, Texture};
use sdl2::keyboard as sdl_keyboard;

use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;
use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::joypad;

use interface::data::{PlayerData, SpriteData};
use interface::values::Direction;

use client;
use interface::{self, extract, hacks, InterfaceData, InterfaceState};
use chat::ChatBox;
use font::Font;
use border::BorderRenderer;

#[derive(PartialEq)]
enum GameState {
    Emulator,
    ChatBox,
    Menu,
}

pub struct Game<'a> {
    pub emulator: Box<Emulator>,
    pub emu_texture: Texture<'a>,
    pub font: &'a Font<'a>,
    pub border_renderer: &'a BorderRenderer<'a>,

    pub game_state: GameState,
    pub interface_data: RefCell<InterfaceData>,
    pub chat_box: ChatBox<'a>,
    pub player_data: PlayerData,
    pub fast_mode: bool,
}

impl<'a> Game<'a> {
    pub fn new(emulator: Box<Emulator>, emu_texture: Texture<'a>, font: &'a Font<'a>,
        border_renderer: &'a BorderRenderer<'a>) -> Game<'a>
    {
        let player_data = PlayerData::new(&emulator.mem);
        let chat_box_rect = Rect::new(client::EMU_WIDTH, 0, client::CHAT_WIDTH, client::EMU_HEIGHT);
        Game {
            emulator: emulator,
            emu_texture: emu_texture,
            font: font,
            border_renderer: border_renderer,

            game_state: GameState::Emulator,
            interface_data: RefCell::new(InterfaceData::new()),
            chat_box: ChatBox::new(font, border_renderer, chat_box_rect),
            player_data: player_data,
            fast_mode: false,
        }
    }

    pub fn update(&mut self) {
        if self.interface_data.borrow().state == InterfaceState::Normal {
            // Individually borrow elements of self that we need so that we pass Rust's borrow
            // checker. (Hopefully we won't need to do this in the future)
            let interface_data = &mut self.interface_data;
            let player_data = &mut self.player_data;
            let emu_texture = &mut self.emu_texture;
            let emulator = &mut self.emulator;

            // After each tick we run all the hacks on the game. Most of the hacks do not actually
            // do anything for most of the cycles but wait for the program to reach a certain point.
            let on_tick = |cpu: &mut Cpu, mem: &mut Memory| {
                let interface_data = &mut interface_data.borrow_mut();
                hacks::sprite_check(cpu, mem, interface_data);
                hacks::display_text(cpu, mem, interface_data);
                hacks::sprite_update_tracker(cpu, mem, interface_data);
            };

            // On each vblank we draw other players to the screen and copy the internal framebuffer
            // to a texture. It is important do this during the vblank period to ensure that we
            // don't get partially redrawn lines affecting the result.
            let on_vblank = |_: &mut Cpu, mem: &mut Memory| {
                let new_player_data = PlayerData {
                    name: extract::player_name(mem),
                    sprite: mem::replace(&mut player_data.sprite, vec![]),
                    movement_data: extract::movement_data(mem),
                };
                *player_data = new_player_data;

                let interface_data = &interface_data.borrow();
                if interface_data.sprites_enabled() {
                    draw_other_players(interface_data, player_data, mem);
                }

                let _ = emu_texture.with_lock(None, |mut pixels, _| {
                    copy_memory(&mut pixels, &mem.gpu.framebuffer);
                });
            };

            emulator.frame(on_tick, on_vblank);
        }

    }

    pub fn render(&self, drawer: &mut RenderDrawer) {
        drawer.copy(&self.emu_texture, None, Some(Rect::new(0, 0, client::EMU_WIDTH,
            client::EMU_HEIGHT)));
        self.chat_box.draw(drawer);
    }

    pub fn key_down(&mut self, keycode: KeyCode) {
        match self.game_state {
            GameState::Emulator => {
                self.write_to_joypad(keycode, joypad::State::Pressed);
                if keycode == KeyCode::Space { self.fast_mode = true; }
            },

            GameState::ChatBox => match keycode {
                // TODO: Possible handle other editing
                KeyCode::Backspace => { self.chat_box.message_buffer.pop(); },
                _ => {},
            },

            GameState::Menu => unimplemented!(),
        }
    }

    pub fn key_up(&mut self, keycode: KeyCode) {
        match self.game_state {
            GameState::Emulator => {
                self.write_to_joypad(keycode, joypad::State::Released);
                if keycode == KeyCode::Space { self.fast_mode = false; }
                else if keycode == KeyCode::T {
                    self.game_state = GameState::ChatBox;
                    sdl_keyboard::start_text_input();
                }
            },

            GameState::ChatBox => {
                if keycode == KeyCode::Return {
                    self.chat_box.message_ready = true;
                    self.game_state = GameState::Emulator;
                    sdl_keyboard::stop_text_input();
                }
            },

            GameState::Menu => unimplemented!(),
        }
    }

    pub fn text_input(&mut self, text: String) {
        if self.game_state == GameState::ChatBox {
            self.chat_box.message_buffer.push_str(&text);
        }
    }

    fn write_to_joypad(&mut self, keycode: KeyCode, state: joypad::State) {
        let joypad = &mut self.emulator.mem.joypad;
        // TODO: Add custom key bindings
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
}

fn draw_other_players(interface_data: &InterfaceData, self_data: &PlayerData, mem: &mut Memory) {
    for player in interface_data.players.values() {
        if player.is_visible_to(self_data) {
            let (x, y) = get_player_draw_position(self_data, player);
            let (index, flags) = get_sprite_index_and_flags(player);
            let sprite_data = SpriteData {
                x: x as isize,
                y: y as isize,
                index: index as usize,
                flags: flags,
            };
            interface::render_sprite(mem, &player.sprite, &sprite_data);
        }
    }
}

/// Get the screen coordinates of where to draw a target player adjusted relative to the local
/// player's screen
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
        Direction::Down  => (x, y + offset),
        Direction::Up    => (x, y - offset),
        Direction::Left  => (x - offset, y),
        Direction::Right => (x + offset, y),
    }
}

fn get_sprite_index_and_flags(player: &PlayerData) -> (isize, u8) {
    // Determine the base sprite index and flags that need to be set based on the direction the
    // player is currently facing.
    let (mut index, mut flags) = match player.movement_data.direction {
        Direction::Down  => (0, 0x00),
        Direction::Up    => (1, 0x00),
        Direction::Left  => (2, 0x00),
        Direction::Right => (2, 0x20),
    };

    // Set the flag that indicates background data may be drawn on top. I'm not sure if this is
    // strictly necessary, however it seems to be set by most sprites.
    flags |= 0x80;

    // Change the frame which is displayed based on
    index += match (player.movement_data.walk_counter / 4) & 1 {
        0 => 0,
        1 => 3,
        _ => unreachable!(),
    };

    (index, flags)
}

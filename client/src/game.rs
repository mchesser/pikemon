use std::cell::RefCell;
use std::slice::bytes::copy_memory;

use sdl2::rect::Rect;
use sdl2::keycode::KeyCode;
use sdl2::render::{RenderDrawer, Texture};

use gb_emu::mmu::Memory;
use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::joypad;

use common::{PlayerData, SpriteData, Direction};

use client;
use interface::{self, extract, hacks, InterfaceData, InterfaceState};
use chat::ChatBox;
use font::Font;

enum GameState {
    Emulator,
    ChatBox,
    Menu,
}

pub struct Game<'a> {
    pub emulator: Box<Emulator>,
    pub emu_texture: Texture<'a>,
    pub default_sprite: Vec<u8>,
    pub font: Font<'a>,

    pub game_state: GameState,
    pub interface_data: RefCell<InterfaceData>,
    pub chat_box: ChatBox,
    pub player_data: PlayerData,
    pub fast_mode: bool,
}

impl<'a> Game<'a> {
    pub fn new(emulator: Box<Emulator>, emu_texture: Texture<'a>, default_sprite: Vec<u8>,
        font: Font<'a>) -> Game<'a>
    {
        Game {
            emulator: emulator,
            emu_texture: emu_texture,
            default_sprite: default_sprite,
            font: font,

            game_state: GameState::Emulator,
            interface_data: RefCell::new(InterfaceData::new()),
            chat_box: ChatBox::new(),
            player_data: PlayerData::new(),
            fast_mode: false,
        }
    }

    pub fn update(&mut self) {
        if self.interface_data.borrow().state == InterfaceState::Normal {
            let interface_data = &mut self.interface_data;
            let mut emulator = &mut self.emulator;
            let emu_texture = &mut self.emu_texture;
            let default_sprite = &*self.default_sprite;
            let player_data = &mut self.player_data;

            emulator.frame(
                |cpu, mem| {
                    let interface_data = &mut *interface_data.borrow_mut();
                    hacks::sprite_check(cpu, mem, interface_data);
                    hacks::display_text(cpu, mem, interface_data);
                    hacks::sprite_update_tracker(cpu, mem, interface_data);
                },

                |_, mem| {
                    let interface_data = &*interface_data.borrow();
                    if interface_data.sprites_enabled() {
                        draw_other_players(interface_data, player_data, mem, default_sprite);
                    }

                    // Copy the screen to the emulator texture
                    let _ = emu_texture.with_lock(None, |mut pixels, _| {
                        copy_memory(&mut *pixels, &mem.gpu.framebuffer);
                    });

                    *player_data = extract::player_data(mem);
                }
            );
        }

    }

    pub fn render(&self, drawer: &mut RenderDrawer) {
        drawer.copy(&self.emu_texture, None, Some(Rect::new(0, 0, client::EMU_WIDTH,
            client::EMU_HEIGHT)));

        self.chat_box.draw(drawer, &self.font, Rect::new(client::EMU_WIDTH, 0,
            client::CHAT_WIDTH, client::EMU_HEIGHT));
    }

    pub fn key_down(&mut self, keycode: KeyCode) {
        match self.game_state {
            GameState::Emulator => {
                self.write_to_joypad(keycode, joypad::State::Pressed);
                if keycode == KeyCode::Space { self.fast_mode = true; }
            },

            GameState::ChatBox => self.write_to_chatbox(keycode),

            GameState::Menu => unimplemented!(),
        }
    }

    pub fn key_up(&mut self, keycode: KeyCode) {
        match self.game_state {
            GameState::Emulator => {
                self.write_to_joypad(keycode, joypad::State::Released);
                if keycode == KeyCode::Space { self.fast_mode = false; }
                else if keycode == KeyCode::T { self.game_state = GameState::ChatBox; }
            },

            GameState::ChatBox => {
                if keycode == KeyCode::Return { self.game_state = GameState::Emulator; }
            },

            GameState::Menu => unimplemented!(),
        }
    }

    fn write_to_joypad(&mut self, keycode: KeyCode, state: joypad::State) {
        let joypad = &mut self.emulator.mem.joypad;
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

    fn write_to_chatbox(&mut self, keycode: KeyCode) {
        let letter = match keycode {
            KeyCode::Return => { self.chat_box.message_ready = true; return },
            KeyCode::Backspace => { self.chat_box.message_buffer.pop(); return },
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

        self.chat_box.message_buffer.push(letter);
    }
}

fn draw_other_players(interface_data: &InterfaceData, self_data: &PlayerData, mem: &mut Memory,
    sprite: &[u8])
{
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
        Direction::Down  => (x, y + offset),
        Direction::Up    => (x, y - offset),
        Direction::Left  => (x - offset, y),
        Direction::Right => (x + offset, y),
    }
}

//fn get_sprite_index_and_flags(direction: u8, walk_counter: u8) -> (isize, u8) {
fn get_sprite_index_and_flags(player: &PlayerData) -> (isize, u8) {
    let (mut index, mut flags) = match player.movement_data.direction {
        Direction::Down  => (0, 0x00),
        Direction::Up    => (1, 0x00),
        Direction::Left  => (2, 0x00),
        Direction::Right => (2, 0x20),
    };

    flags |= 0x80;

    index += match player.movement_data.walk_counter / 4 {
        0 => 0,
        1 => 3,

        _ => 0, // Usually unreachable
    };

    (index, flags)
}

use std::{cell::RefCell, mem};

use gb_emu::{cpu::Cpu, emulator::Emulator, graphics, joypad, mmu::Memory};

use interface::{
    self,
    data::{PlayerData, SpriteData},
    extract, hacks,
    values::Direction,
    InterfaceData, InterfaceState,
};
use macroquad::{
    prelude::{KeyCode, WHITE},
    texture::{render_target, FilterMode, Image, Texture2D},
};

use crate::{
    border::BorderRenderer,
    chat::ChatBox,
    client,
    common::{Rect, Renderer},
    font::Font,
    menu::ItemBox,
};

#[derive(PartialEq, Eq)]
pub enum GameState {
    Emulator,
    ChatBox,
    Menu,
}

pub struct Game<'a> {
    pub emulator: Box<Emulator>,
    pub screen: Image,
    pub screen_texture: Texture2D,
    pub font: &'a Font,
    pub border_renderer: &'a BorderRenderer,

    pub game_state: GameState,
    pub interface_data: RefCell<InterfaceData>,
    pub chat_box: ChatBox<'a>,
    pub menu: ItemBox<'a>,
    pub player_data: PlayerData,
    pub fast_mode: bool,
    pub exit_requested: bool,
}

impl<'a> Game<'a> {
    pub fn new(
        emulator: Box<Emulator>,
        font: &'a Font,
        border_renderer: &'a BorderRenderer,
    ) -> Game<'a> {
        let player_data = PlayerData::new(&emulator.mem);

        let chat_box_rect = Rect::new(
            client::EMU_WIDTH as i32,
            0,
            client::CHAT_WIDTH as i32,
            client::EMU_HEIGHT as i32,
        );
        let menu_rect = Rect::new(
            ((client::EMU_WIDTH - client::MENU_WIDTH) / 2) as i32,
            ((client::EMU_HEIGHT - client::MENU_HEIGHT) / 2) as i32,
            client::MENU_WIDTH as i32,
            client::MENU_HEIGHT as i32,
        );

        let screen_texture = render_target(graphics::WIDTH as u32, graphics::HEIGHT as u32).texture;
        screen_texture.set_filter(FilterMode::Nearest);

        Game {
            emulator,
            screen: Image::gen_image_color(graphics::WIDTH as u16, graphics::HEIGHT as u16, WHITE),
            screen_texture,
            font,
            border_renderer,

            game_state: GameState::Emulator,
            interface_data: RefCell::new(InterfaceData::new()),
            chat_box: ChatBox::new(font, border_renderer, chat_box_rect),
            menu: ItemBox::new(
                vec!["CONNECT".to_string(), "SHOW PLAYERS".to_string(), "EXIT".to_string()],
                font,
                border_renderer,
                menu_rect,
            ),
            player_data,
            fast_mode: false,
            exit_requested: false,
        }
    }

    pub fn update(&mut self) {
        if self.interface_data.borrow().state == InterfaceState::Normal {
            // Individually borrow elements of self that we need so that we pass Rust's borrow
            // checker. (Hopefully we won't need to do this in the future)
            let interface_data = &mut self.interface_data;
            let player_data = &mut self.player_data;
            let screen = &mut self.screen;
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

                screen.bytes.copy_from_slice(&mem.gpu.framebuffer);
                self.screen_texture.update(&screen);
            };

            emulator.frame(on_tick, on_vblank);
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.copy(
            self.screen_texture,
            None,
            Some(Rect::new(0, 0, client::EMU_WIDTH as i32, client::EMU_HEIGHT as i32)),
        );
        self.chat_box.draw(renderer);

        if self.game_state == GameState::Menu {
            self.menu.draw(renderer);
        }
    }

    pub fn key_down(&mut self, keycode: KeyCode) {
        match self.game_state {
            GameState::Emulator => {
                self.write_to_joypad(keycode, joypad::State::Pressed);
                if keycode == KeyCode::Space {
                    self.fast_mode = true;
                }
            }

            GameState::ChatBox => match keycode {
                // TODO: Possible handle other editing
                KeyCode::Backspace => {
                    self.chat_box.message_buffer.pop();
                }
                _ => {}
            },

            GameState::Menu => match keycode {
                KeyCode::Up => self.menu.move_up(),
                KeyCode::Down => self.menu.move_down(),
                _ => {}
            },
        }
    }

    pub fn key_up(&mut self, keycode: KeyCode) {
        match self.game_state {
            GameState::Emulator => {
                self.write_to_joypad(keycode, joypad::State::Released);
                if keycode == KeyCode::Space {
                    self.fast_mode = false;
                }
                else if keycode == KeyCode::T {
                    self.game_state = GameState::ChatBox;
                    // sdl_keyboard::start_text_input();
                }
                else if keycode == KeyCode::Escape {
                    self.game_state = GameState::Menu;
                }
            }

            GameState::ChatBox => {
                match keycode {
                    KeyCode::Enter => {
                        self.chat_box.message_ready = true;
                        self.game_state = GameState::Emulator;
                        // sdl_keyboard::stop_text_input();
                    }

                    KeyCode::Escape => {
                        self.game_state = GameState::Emulator;
                        // sdl_keyboard::stop_text_input();
                    }

                    _ => {}
                }
            }

            GameState::Menu => match keycode {
                KeyCode::Escape => {
                    self.game_state = GameState::Emulator;
                }
                _ => {}
            },
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
            KeyCode::Enter => joypad.start = state,
            KeyCode::RightShift => joypad.select = state,

            _ => {}
        }
    }
}

fn draw_other_players(interface_data: &InterfaceData, self_data: &PlayerData, mem: &mut Memory) {
    for player in interface_data.players.values() {
        if player.is_visible_to(self_data) {
            let (x, y) = get_player_draw_position(self_data, player);
            let (index, flags) = get_sprite_index_and_flags(player);
            let sprite_data =
                SpriteData { x: x as isize, y: y as isize, index: index as usize, flags };
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
        Direction::Down => (x, y + offset),
        Direction::Up => (x, y - offset),
        Direction::Left => (x - offset, y),
        Direction::Right => (x + offset, y),
    }
}

fn get_sprite_index_and_flags(player: &PlayerData) -> (isize, u8) {
    // Determine the base sprite index and flags that need to be set based on the direction the
    // player is currently facing.
    let (mut index, mut flags) = match player.movement_data.direction {
        Direction::Down => (0, 0x00),
        Direction::Up => (1, 0x00),
        Direction::Left => (2, 0x00),
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

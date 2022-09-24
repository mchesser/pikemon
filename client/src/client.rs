use std::{error::Error, time::Instant};

use gb_emu::{emulator::Emulator, graphics, mmu::Memory};

use interface::{self, extract};
use macroquad::{
    miniquad::EventHandler,
    prelude::utils,
    texture::{FilterMode, Texture2D},
    window::{next_frame, request_new_screen_size},
};

use crate::{border::BorderRenderer, common::Renderer, font::Font, game::Game, net::ClientManager};

const EMU_SCALE: u32 = 3;
pub const EMU_WIDTH: u32 = graphics::WIDTH as u32 * EMU_SCALE;
pub const EMU_HEIGHT: u32 = graphics::HEIGHT as u32 * EMU_SCALE;

pub const MENU_WIDTH: u32 = 128 * EMU_SCALE;
pub const MENU_HEIGHT: u32 = EMU_HEIGHT / 2;

pub const CHAT_WIDTH: u32 = 208;
pub const CHAT_SCALE: u32 = 1;

impl<'a> EventHandler for Game<'a> {
    fn update(&mut self, _ctx: &mut macroquad::miniquad::Context) {}
    fn draw(&mut self, _ctx: &mut macroquad::miniquad::Context) {}

    fn char_event(
        &mut self,
        _ctx: &mut macroquad::miniquad::Context,
        character: char,
        _keymods: macroquad::miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.text_input(character.to_string())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut macroquad::miniquad::Context,
        keycode: macroquad::prelude::KeyCode,
        _keymods: macroquad::miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.key_down(keycode);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut macroquad::miniquad::Context,
        keycode: macroquad::prelude::KeyCode,
        _keymods: macroquad::miniquad::KeyMods,
    ) {
        self.key_up(keycode)
    }

    fn quit_requested_event(&mut self, _ctx: &mut macroquad::miniquad::Context) {
        eprintln!("exit requested");
        self.exit_requested = true;
    }
}

pub async fn run(
    mut client_manager: ClientManager,
    emulator: Box<Emulator>,
) -> Result<(), Box<dyn Error>> {
    request_new_screen_size(EMU_WIDTH as f32, EMU_HEIGHT as f32);

    let mut renderer = Renderer;
    let font_data = load_font(&renderer, &emulator.mem);
    let border_renderer = load_border_renderer(&renderer, &emulator.mem);

    let mut game = Game::new(emulator, &font_data, &border_renderer);

    let mut prev_time = Instant::now();
    let mut frame_time = 0;

    let events_subscriber = utils::register_input_subscriber();
    while !game.exit_requested {
        utils::repeat_all_miniquad_input(&mut game, events_subscriber);

        game.render(&mut renderer);

        client_manager.update_player(&game.player_data);
        client_manager.send_update(&mut game).unwrap();
        client_manager.recv_update(&mut game).unwrap();

        let current_time = Instant::now();
        frame_time += (current_time - prev_time).as_nanos() as u64;
        prev_time = current_time;

        if !game.fast_mode {
            const TARGET_TIME_STEP: u64 = 16666667;
            while frame_time >= TARGET_TIME_STEP {
                frame_time -= TARGET_TIME_STEP;
                game.update();
            }
            // thread::sleep(Duration::new(0, (TARGET_TIME_STEP - frame_time) as u32));
        }
        else {
            for _ in 0..10 {
                game.update();
            }
        }

        next_frame().await
    }

    Ok(())
}

fn load_font(_renderer: &Renderer, mem: &Memory) -> Font {
    const BLACK: [u8; 4] = [0, 0, 0, 255];
    const WHITE: [u8; 4] = [255, 255, 255, 255];

    const FONT_TEX_WIDTH: usize = 8 * 16 * 8;
    const FONT_TEX_HEIGHT: usize = 8;

    // Extract the font data from the game
    let data = extract::extract_texture(
        mem,
        interface::offsets::FONT_BANK,
        interface::offsets::FONT_ADDR,
        FONT_TEX_WIDTH,
        FONT_TEX_HEIGHT,
        extract::TextureFormat::Bpp1,
        &[BLACK, WHITE],
    );

    // Build a texture from the extracted data
    let texture = Texture2D::from_rgba8(FONT_TEX_WIDTH as u16, FONT_TEX_HEIGHT as u16, &data);
    texture.set_filter(FilterMode::Nearest);
    Font::new(texture, 8, 8, CHAT_SCALE as i32)
}

fn load_border_renderer(_renderer: &Renderer, mem: &Memory) -> BorderRenderer {
    const BORDER_TEX_WIDTH: usize = 8 * 7;
    const BORDER_TEX_HEIGHT: usize = 8;

    // Extract the border data from the game
    let data = extract::extract_texture(
        mem,
        interface::offsets::FONT_BANK,
        interface::offsets::BORDER_ADDR,
        BORDER_TEX_WIDTH,
        BORDER_TEX_HEIGHT,
        extract::TextureFormat::Bpp2,
        graphics::GB_COLOR_TABLE,
    );

    // Build a texture from the extracted data
    let texture = Texture2D::from_rgba8(BORDER_TEX_WIDTH as u16, BORDER_TEX_HEIGHT as u16, &data);
    texture.set_filter(FilterMode::Nearest);
    BorderRenderer::new(texture, 8, CHAT_SCALE as i32)
}

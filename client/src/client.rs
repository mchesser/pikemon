extern crate clock_ticks;

use std::error::Error;
use std::thread;
use std::time::Duration;

use sdl2;
use sdl2::event::Event;
use sdl2::surface::Surface;
use sdl2::render::{Renderer, TextureAccess};
use sdl2::pixels::{PixelFormatEnum, Color};

use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::mmu::Memory;

use game::Game;
use net::ClientManager;
use interface::{self, extract};

use font::Font;
use border::BorderRenderer;

const EMU_SCALE: u32 = 2;
pub const EMU_WIDTH: u32 = graphics::WIDTH as u32 * EMU_SCALE;
pub const EMU_HEIGHT: u32 = graphics::HEIGHT as u32 * EMU_SCALE;

pub const MENU_WIDTH: u32 = 128 * EMU_SCALE;
pub const MENU_HEIGHT: u32 = EMU_HEIGHT / 2;

pub const CHAT_WIDTH: u32 = 208;
pub const CHAT_SCALE: u32 = 1;

pub fn run(mut client_manager: ClientManager, emulator: Box<Emulator>) -> Result<(), Box<Error>> {
    const WHITE: Color = Color::RGB(0xFF, 0xFF, 0xFF);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Pikemon", EMU_WIDTH, EMU_HEIGHT).position_centered()
        .opengl().build()?;
    let mut renderer = window.renderer().accelerated().build()?;

    let font_data = load_font(&renderer, &emulator.mem)?;
    let border_renderer = load_border_renderer(&renderer, &emulator.mem)?;

    let emu_texture = renderer.create_texture_streaming(PixelFormatEnum::ARGB8888,
        graphics::WIDTH as u32, graphics::HEIGHT as u32)?;

    let mut game = Game::new(emulator, emu_texture, &font_data, &border_renderer);

    let mut prev_time = clock_ticks::precise_time_ns();
    let mut frame_time = 0;

    let mut events = sdl_context.event_pump()?;

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} => break 'main,
                Event::KeyDown{ keycode: Some(keycode), .. } => game.key_down(keycode),
                Event::KeyUp{ keycode: Some(keycode), .. } => game.key_up(keycode),
                Event::TextInput{ text, .. } => game.text_input(text),

                _ => {},
            }
        }

        renderer.set_draw_color(WHITE);
        renderer.clear();
        game.render(&mut renderer);
        renderer.present();

        client_manager.update_player(&game.player_data);
        client_manager.send_update(&mut game).unwrap();
        client_manager.recv_update(&mut game).unwrap();

        let current_time = clock_ticks::precise_time_ns();
        frame_time += current_time - prev_time;
        prev_time = current_time;

        const TARGET_TIME_STEP: u64 = 16666667;
        while frame_time >= TARGET_TIME_STEP {
            frame_time -= TARGET_TIME_STEP;
            game.update();
        }

        thread::sleep(Duration::new(0, (TARGET_TIME_STEP - frame_time) as u32));
    }
    Ok(())
}

fn load_font(renderer: &Renderer, mem: &Memory) -> Result<Font, Box<Error>> {
    const BLACK: [u8; 4] = [0, 0, 0, 255];
    const WHITE: [u8; 4] = [255, 255, 255, 255];

    const FONT_TEX_WIDTH: usize = 8 * 16 * 8;
    const FONT_TEX_HEIGHT: usize = 8;

    // Extract the font data from the game
    let mut data = extract::extract_texture(mem, interface::offsets::FONT_BANK,
        interface::offsets::FONT_ADDR, FONT_TEX_WIDTH, FONT_TEX_HEIGHT,
        extract::TextureFormat::Bpp1, &[BLACK, WHITE]);

    // Build a texture from the extracted data
    let surface = Surface::from_data(&mut data, FONT_TEX_WIDTH as u32, FONT_TEX_HEIGHT as u32,
        32, PixelFormatEnum::ARGB8888)?;
    let texture = try!(renderer.create_texture_from_surface(&surface));

    Ok(Font::new(texture, 8, 8, CHAT_SCALE as i32))
}

fn load_border_renderer(renderer: &Renderer, mem: &Memory) -> Result<BorderRenderer, Box<Error>> {
    const BORDER_TEX_WIDTH: usize = 8 * 7;
    const BORDER_TEX_HEIGHT: usize = 8;

    // Extract the border data from the game
    let mut data = extract::extract_texture(mem, interface::offsets::FONT_BANK,
        interface::offsets::BORDER_ADDR, BORDER_TEX_WIDTH, BORDER_TEX_HEIGHT,
        extract::TextureFormat::Bpp2, graphics::GB_COLOR_TABLE);

    // Build a texture from the extracted data
    let surface = try!(Surface::from_data(&mut data, BORDER_TEX_WIDTH as u32, BORDER_TEX_HEIGHT as u32,
        32, PixelFormatEnum::ARGB8888));
    let texture = try!(renderer.create_texture_from_surface(&surface));

    Ok(BorderRenderer::new(texture, 8, CHAT_SCALE as i32))
}

extern crate clock_ticks;

use std::old_io::timer;
use std::time::duration::Duration;

use sdl2;
use sdl2::SdlResult;
use sdl2::event::Event;
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;
use sdl2::surface::Surface;
use sdl2::render::{self, Renderer, RenderDriverIndex, TextureAccess};
use sdl2::pixels::{PixelFormatEnum, Color};

use gb_emu::emulator::Emulator;
use gb_emu::graphics;
use gb_emu::mmu::Memory;

use game::Game;
use net::ClientManager;
use interface::{self, extract};

use font::Font;
use border::BorderRenderer;

const EMU_SCALE: i32 = 3;
pub const EMU_WIDTH: i32 = graphics::WIDTH as i32 * EMU_SCALE;
pub const EMU_HEIGHT: i32 = graphics::HEIGHT as i32 * EMU_SCALE;

pub const CHAT_WIDTH: i32 = 250;
pub const FONT_SCALE: i32 = 1;

pub fn run(mut client_manager: ClientManager, emulator: Box<Emulator>) -> SdlResult<()> {
    const WHITE: Color = Color::RGB(0xFF, 0xFF, 0xFF);

    let sdl_context = sdl2::init(sdl2::INIT_EVERYTHING).unwrap();
    let mut events = sdl_context.event_pump();

    let window = try!(Window::new("Pikemon", PosCentered, PosCentered, EMU_WIDTH + CHAT_WIDTH,
        EMU_HEIGHT, OPENGL));
    let renderer = try!(Renderer::from_window(window, RenderDriverIndex::Auto,
        render::ACCELERATED));

    let player_sprite = extract::default_sprite(&emulator.mem);
    let font_data = try!(load_font(&renderer, &emulator.mem));
    let border_renderer = try!(load_border_renderer(&renderer, &emulator.mem));

    let emu_texture = try!(renderer.create_texture(PixelFormatEnum::ARGB8888,
        TextureAccess::Streaming, (graphics::WIDTH as i32, graphics::HEIGHT as i32)));

    let mut game = Game::new(emulator, emu_texture, player_sprite, font_data, border_renderer);

    let mut prev_time = clock_ticks::precise_time_ns();
    let mut frame_time = 0;

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} => break 'main,
                Event::KeyDown{ keycode: code, .. } => game.key_down(code),
                Event::KeyUp{ keycode: code, .. } => game.key_up(code),

                _ => {},
            }
        }

        let mut drawer = renderer.drawer();
        drawer.set_draw_color(WHITE);
        drawer.clear();
        game.render(&mut drawer);
        drawer.present();

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

        timer::sleep(Duration::nanoseconds((TARGET_TIME_STEP - frame_time) as i64));
    }
    Ok(())
}

const RMASK: u32 = 0x000000FF;
const GMASK: u32 = 0x0000FF00;
const BMASK: u32 = 0x00FF0000;
const AMASK: u32 = 0xFF000000;

const SDL_BYTES_PER_PIXEL: usize = 4;

fn load_font<'a>(renderer: &'a Renderer, mem: &Memory) -> SdlResult<Font<'a>> {
    const BLACK: [u8; 4] = [0, 0, 0, 255];
    const WHITE: [u8; 4] = [255, 255, 255, 255];

    const FONT_TEX_WIDTH: usize = 8 * 16 * 8;
    const FONT_TEX_HEIGHT: usize = 8;

    // Extract the font data from the game
    let mut data = extract::extract_texture(mem, interface::offsets::FONT_BANK,
        interface::offsets::FONT_ADDR, FONT_TEX_WIDTH, FONT_TEX_HEIGHT,
        extract::TextureFormat::Bpp1, &[BLACK, WHITE]);

    // Build a texture from the extracted data
    let surface = try!(Surface::from_data(&mut data, FONT_TEX_WIDTH as i32, FONT_TEX_HEIGHT as i32,
        32, (FONT_TEX_WIDTH * SDL_BYTES_PER_PIXEL) as i32, RMASK, GMASK, BMASK, AMASK));
    let texture = try!(renderer.create_texture_from_surface(&surface));

    Ok(Font::new(texture, 8, 8, FONT_SCALE))
}

fn load_border_renderer<'a>(renderer: &'a Renderer, mem: &Memory) -> SdlResult<BorderRenderer<'a>> {
    const BORDER_TEX_WIDTH: usize = 8 * 7;
    const BORDER_TEX_HEIGHT: usize = 8;

    // Extract the border data from the game
    let mut data = extract::extract_texture(mem, interface::offsets::FONT_BANK,
        interface::offsets::BORDER_ADDR, BORDER_TEX_WIDTH, BORDER_TEX_HEIGHT,
        extract::TextureFormat::Bpp2, graphics::GB_COLOR_TABLE);

    // Build a texture from the extracted data
    let surface = try!(Surface::from_data(&mut data, BORDER_TEX_WIDTH as i32, BORDER_TEX_HEIGHT as i32,
        32, (BORDER_TEX_WIDTH * SDL_BYTES_PER_PIXEL) as i32, RMASK, GMASK, BMASK, AMASK));
    let texture = try!(renderer.create_texture_from_surface(&surface));

    Ok(BorderRenderer::new(texture, 8))
}

use sdl2;
use sdl2::SdlResult;
use sdl2::event::{Event, poll_event};
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;
use sdl2::render::{self, Renderer, RenderDriverIndex, TextureAccess};
use sdl2::pixels::{PixelFormatFlag, Color};

use gb_emu::emulator::Emulator;
use gb_emu::graphics;

use game::Game;
use timer::Timer;
use net::ClientManager;
use interface::extract;
use font::Font;

const EMU_SCALE: i32 = 3;
pub const EMU_WIDTH: i32 = graphics::WIDTH as i32 * EMU_SCALE;
pub const EMU_HEIGHT: i32 = graphics::HEIGHT as i32 * EMU_SCALE;

pub const CHAT_WIDTH: i32 = 250;
pub const FONT_SCALE: i32 = 1;

pub fn run(mut client_manager: ClientManager, mut emulator: Box<Emulator>) -> SdlResult<()> {
    const WHITE: Color = Color::RGB(0xFF, 0xFF, 0xFF);

    sdl2::init(sdl2::INIT_EVERYTHING);

    let window = try!(Window::new("Pikemon", PosCentered, PosCentered, EMU_WIDTH + CHAT_WIDTH,
        EMU_HEIGHT, OPENGL));
    let renderer = try!(Renderer::from_window(window, RenderDriverIndex::Auto,
        render::ACCELERATED));

    let player_sprite = extract::default_sprite(&mut emulator.mem);

    let font_texture = try!(extract::font_texture(&renderer, &mut emulator.mem));
    let font_data = Font::new(font_texture, 8, 8, FONT_SCALE);

    let emu_texture = try!(renderer.create_texture(PixelFormatFlag::ARGB8888,
        TextureAccess::Streaming, graphics::WIDTH as i32, graphics::HEIGHT as i32));

    let mut game = Game::new(emulator, emu_texture, player_sprite, font_data);

    let mut emulator_timer = Timer::new();
    let mut network_timer = Timer::new();

    'main: loop {
        'event: loop {
            match poll_event() {
                Event::Quit(_) => break 'main,

                Event::KeyDown(_, _, code, _, _, _) => game.key_down(code),
                Event::KeyUp(_, _, code, _, _, _) => game.key_up(code),

                Event::None => break,
                _ => continue,
            }
        }

        if emulator_timer.elapsed_seconds() >= 1.0 / 60.0 {
            emulator_timer.reset();
            game.update();
        }

        if network_timer.elapsed_seconds() >= 1.0 / 60.0 {
            network_timer.reset();
            client_manager.update_player(&game.player_data);
            let _ = client_manager.send_update(&mut game);
            let _ = client_manager.recv_update(&mut game);
        }

        try!(renderer.set_draw_color(WHITE));
        try!(renderer.clear());
        try!(game.render(&renderer));
        renderer.present();
    }
    Ok(())
}

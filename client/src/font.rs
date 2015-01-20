use sdl2::SdlResult;
use sdl2::rect::Rect;
use sdl2::render::{Texture, Renderer, RendererFlip};

pub struct Font {
    texture: Texture,
    char_height: i32,
    char_width: i32,
    scale: i32,
}
impl Font {
    pub fn new(texture: Texture, char_height: i32, char_width: i32, scale: i32) -> Font {
        Font {
            texture: texture,
            char_height: char_height,
            char_width: char_width,
            scale: scale,
        }
    }

    pub fn line_height(&self) -> i32 {
        self.char_height * self.scale
    }

    pub fn char_width(&self) -> i32 {
        self.char_width * self.scale
    }

    pub fn draw_char(&self, renderer: &Renderer, value: i32, x: i32, y: i32) -> SdlResult<()> {
        let offset = value * self.char_width;
        let source_rect = Rect::new(offset, 0, self.char_width, self.char_height);
        let dest_rect = Rect::new(x, y, self.char_width(), self.line_height());

        renderer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None,
            RendererFlip::None)
    }
}


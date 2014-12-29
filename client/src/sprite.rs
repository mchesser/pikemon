use sdl2::render::{Texture, Renderer, RendererFlip};
use sdl2::rect::Rect;
use sdl2::SdlResult;

pub struct Sprite {
    texture: Texture,
    frame_width: i32,
    frame_height: i32,
    scale: i32,
}

impl Sprite {
    pub fn new(tex: Texture, frame_width: i32, frame_height: i32, scale: i32) -> Sprite {
        Sprite {
            texture: tex,
            frame_width: frame_width,
            frame_height: frame_height,
            scale: scale,
        }
    }

    pub fn draw(&self, renderer: &Renderer, x: i32, y: i32, frame: i32, flip: RendererFlip)
        -> SdlResult<()>
    {
        let source_rect = Rect::new(0, frame * self.frame_height,
            self.frame_width, self.frame_height);
        let dest_rect = Rect::new(x, y, self.frame_width * self.scale,
            self.frame_height * self.scale);

        renderer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip)
    }
}

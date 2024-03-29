use interface::text::special;
use macroquad::texture::Texture2D;

use crate::common::{Rect, Renderer};

pub struct Font {
    texture: Texture2D,
    char_height: i32,
    char_width: i32,
    scale: i32,
}

impl Font {
    pub fn new(texture: Texture2D, char_height: i32, char_width: i32, scale: i32) -> Font {
        Font { texture, char_height, char_width, scale }
    }

    pub fn line_height(&self) -> i32 {
        self.char_height * self.scale
    }

    pub fn char_width(&self) -> i32 {
        self.char_width * self.scale
    }

    pub fn draw_char(&self, renderer: &mut Renderer, val: i32, x: i32, y: i32) {
        let offset = val * self.char_width;
        let source_rect = Rect::new(offset, 0, self.char_width, self.char_height);
        let dest_rect = Rect::new(x, y, self.char_width(), self.line_height());
        renderer.copy(self.texture, Some(source_rect), Some(dest_rect));
    }
}

/// Draw text, returning the total height of the text drawn
pub fn draw_text(renderer: &mut Renderer, font: &Font, text: &[u8], target: &Rect) -> i32 {
    let (mut x, mut y) = (target.x, target.y);
    for &char_ in text {
        match char_ {
            // These are all control characters, and so do not matter when we are manually rendering
            // the text
            special::TEXT_START | special::SCROLL_LINE | special::END_MSG | special::END_PROMPT => {
            }

            special::SPACE => x += font.char_width(),

            special::LINE_DOWN => {
                x = target.x;
                y += font.line_height();
            }

            special::TERMINATOR => break,

            normal_char => {
                // The index of normal characters in the font is their value - 0x80
                font.draw_char(renderer, (normal_char - 0x80) as i32, x, y);
                x += font.char_width();
            }
        }

        // Check for wrapping
        if x - target.x + font.char_width() > target.width {
            x = target.x;
            y += font.line_height();
        }
        if y - target.y + font.line_height() > target.height {
            break;
        }
    }

    // Return the height of the text drawn
    y - target.y + font.line_height()
}

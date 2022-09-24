//! File for managing borders in the game
use macroquad::texture::Texture2D;

use crate::common::{Rect, Renderer};

/// A struct for managing a border
pub struct BorderRenderer {
    texture: Texture2D,
    piece_size: i32,
    scale: i32,
}

impl BorderRenderer {
    /// Create a new border renderer.
    ///
    /// # Arguments
    ///
    /// * texture - an sdl texture where the pieces are store horizontally in the order:
    /// top-left, top, top-right, left, bottom-left, bottom-right.
    /// * piece_size - The horizontal width of one of the pieces.
    /// * scale - The scale to draw the border at
    pub fn new(texture: Texture2D, piece_size: i32, scale: i32) -> BorderRenderer {
        BorderRenderer { texture, piece_size, scale }
    }

    fn scaled_size(&self) -> i32 {
        self.piece_size * self.scale
    }

    /// Renders a box to a specified rectangle
    pub fn draw_box(&self, renderer: &mut Renderer, rect: Rect) {
        let mut src_rect = Rect::new(0, 0, self.piece_size, self.piece_size);

        // Top-left border
        src_rect.x = 0 * self.piece_size;
        let mut dst_rect = Rect::new(rect.x, rect.y, self.scaled_size(), self.scaled_size());
        renderer.copy(self.texture, Some(src_rect), Some(dst_rect));

        // Top border
        src_rect.x = 1 * self.piece_size;
        dst_rect.x += self.scaled_size();
        while dst_rect.x + self.scaled_size() < rect.x + rect.width {
            renderer.copy(self.texture, Some(src_rect), Some(dst_rect));
            dst_rect.x += self.scaled_size();
        }

        // Top-right border
        src_rect.x = 2 * self.piece_size;
        renderer.copy(self.texture, Some(src_rect), Some(dst_rect));

        // Left border
        src_rect.x = 3 * self.piece_size;
        dst_rect.y = rect.y + self.scaled_size();
        dst_rect.x = rect.x;
        while dst_rect.y + self.scaled_size() < rect.y + rect.height {
            renderer.copy(self.texture, Some(src_rect), Some(dst_rect));
            dst_rect.y += self.scaled_size();
        }

        // Right border
        src_rect.x = 3 * self.piece_size;
        dst_rect.y = rect.y + self.scaled_size();
        dst_rect.x = rect.x + rect.width - self.scaled_size();
        while dst_rect.y + self.scaled_size() < rect.y + rect.height {
            renderer.copy(self.texture, Some(src_rect), Some(dst_rect));
            dst_rect.y += self.scaled_size();
        }

        // Bottom-left border
        src_rect.x = 4 * self.piece_size;
        dst_rect.x = rect.x;
        renderer.copy(self.texture, Some(src_rect), Some(dst_rect));

        // Bottom border
        src_rect.x = 1 * self.piece_size;
        dst_rect.x += self.scaled_size();
        while dst_rect.x + self.scaled_size() < rect.x + rect.width {
            renderer.copy(self.texture, Some(src_rect), Some(dst_rect));
            dst_rect.x += self.scaled_size();
        }

        // Bottom-right border
        src_rect.x = 5 * self.piece_size;
        renderer.copy(self.texture, Some(src_rect), Some(dst_rect));
    }
}

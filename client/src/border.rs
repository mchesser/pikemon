//! File for managing borders in the game
use sdl2::render::{Texture, RenderDrawer};
use sdl2::rect::Rect;

/// A struct for managing a border
pub struct BorderRenderer<'a> {
    texture: Texture<'a>,
    piece_size: i32,
}

impl<'a> BorderRenderer<'a> {
    /// Create a new border drawer.
    ///
    /// # Arguments
    ///
    /// * texture - an sdl texture where the pieces are store horizontally in the order:
    /// top-left, top, top-right, left, right, bottom-left, bottom, bottom-right.
    /// * piece_size - The horizontal width of one of the pieces.
    pub fn new(texture: Texture<'a>, piece_size: i32) -> BorderRenderer {
        BorderRenderer {
            texture: texture,
            piece_size: piece_size,
        }
    }

    /// Renders a box to a specified rectangle
    pub fn draw_box(&self, drawer: &mut RenderDrawer, rect: Rect) {
        let flip = (false, false);

        // Top-left border
        let mut source_rect = Rect::new(0, 0, self.piece_size, self.piece_size);
        let mut dest_rect = Rect::new(rect.x, rect.y, self.piece_size, self.piece_size);
        drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
        source_rect.x += self.piece_size;

        // Top border
        dest_rect.x += self.piece_size;
        while dest_rect.x + 2 * self.piece_size < rect.x + rect.w {
            drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
            dest_rect.x += self.piece_size;
        }
        source_rect.x += self.piece_size;

        // Top-right border
        drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
        source_rect.x += self.piece_size;

        // Left border
        dest_rect.y = self.piece_size;
        dest_rect.x = rect.x;
        while dest_rect.y + 2 * self.piece_size < rect.y + rect.h {
            drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
            dest_rect.y += self.piece_size;
        }
        source_rect.x += self.piece_size;

        // Right border
        dest_rect.y = self.piece_size;
        dest_rect.x = rect.x + rect.w - self.piece_size;
        while dest_rect.y + 2 * self.piece_size < rect.y + rect.h {
            drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
            dest_rect.y += self.piece_size;
        }
        source_rect.x += self.piece_size;

        // Bottom-left border
        dest_rect.x = rect.x;
        drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
        source_rect.x += self.piece_size;

        // Bottom border
        dest_rect.x += self.piece_size;
        while dest_rect.x + 2 * self.piece_size < rect.x + rect.w {
            drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
            dest_rect.x += self.piece_size;
        }
        source_rect.x += self.piece_size;

        // Bottom-right border
        drawer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None, flip);
    }
}

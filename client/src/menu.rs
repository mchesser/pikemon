use sdl2::render::Renderer;

use interface::text;

use common::Rect;
use font::{Font, draw_text};
use border::BorderRenderer;

pub struct ItemBox<'a> {
    items: Vec<String>,
    selection: usize,

    font: &'a Font,
    border: &'a BorderRenderer,

    outer_rect: Rect,
    inner_rect: Rect,
}

impl<'a> ItemBox<'a> {
    pub fn new(items: Vec<String>, font: &'a Font, border: &'a BorderRenderer, rect: Rect)
        -> ItemBox<'a>
    {
        let inner_rect = Rect::new(rect.x + 2 * font.char_width(), rect.y + 2 * font.line_height(),
            rect.width - 3 * font.char_width(), rect.height - 3 * font.line_height());

        ItemBox {
            items: items,
            selection: 0,

            font: font,
            border: border,

            outer_rect: rect,
            inner_rect: inner_rect,
        }
    }

    /// Draws the item box to the screen.
    /// TODO: Cache the render result
    pub fn draw(&self, renderer: &mut Renderer) {
        renderer.fill_rect(self.outer_rect.to_sdl());

        let text_spacing = self.font.line_height();
        let mut y = self.inner_rect.y;

        let mut text_buffer = vec![];
        for (i, item) in self.items.iter().enumerate() {
            if i == self.selection {
                self.font.draw_char(renderer, text::encode_char('>') as i32 - 0x80,
                    self.inner_rect.x - self.font.char_width(), y);
            }

            text_buffer.extend(text::Encoder::new(&item));
            y += draw_text(renderer, &self.font, &text_buffer,
                &Rect::new(self.inner_rect.x, y, self.inner_rect.width, self.inner_rect.height));
            y += text_spacing;
            text_buffer.clear();
        }

        // Draw the chat border
        self.border.draw_box(renderer, self.outer_rect);
    }

    pub fn move_down(&mut self) {
        self.selection += 1;
        if self.selection == self.items.len() {
            self.selection = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.selection == 0 {
            self.selection = self.items.len();
        }
        self.selection -= 1;
    }
}




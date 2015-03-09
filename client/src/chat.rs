use std::mem;

use sdl2::render::RenderDrawer;
use sdl2::rect::Rect;

use interface::text;
use font::{Font, draw_text};
use border::BorderRenderer;

struct Message {
    user_name: Vec<u8>,
    data: Vec<u8>,
}

pub struct ChatBox<'a> {
    pub message_ready: bool,
    pub message_buffer: String,
    messages: Vec<Message>,

    font: &'a Font<'a>,
    border: &'a BorderRenderer<'a>,

    outer_rect: Rect,
    inner_rect: Rect,
}

impl<'a> ChatBox<'a> {
    pub fn new(font: &'a Font<'a>, border: &'a BorderRenderer<'a>, rect: Rect) -> ChatBox<'a> {
        let inner_rect = Rect::new(rect.x + font.char_width(), rect.y + font.line_height(),
            rect.w - 2 * font.char_width(), rect.h - 2 * font.line_height());

        ChatBox {
            message_ready: false,
            message_buffer: String::new(),
            messages: Vec::new(),

            font: font,
            border: border,

            outer_rect: rect,
            inner_rect: inner_rect,
        }
    }

    pub fn get_message_buffer(&mut self) -> String {
        self.message_ready = false;
        mem::replace(&mut self.message_buffer, String::new())
    }

    pub fn add_message(&mut self, user_name: Vec<u8>, msg: Vec<u8>) {
        self.messages.push(Message {
            user_name: user_name,
            data: msg,
        });
    }

    /// Draws the chat box to the screen.
    /// TODO: Cache the render result
    pub fn draw(&self, drawer: &mut RenderDrawer) {
        let mut y = self.inner_rect.y;
        let msg_padding = self.font.char_width() / 2;

        // Draw the text that the player is currently typing
        let encoded_buffer: Vec<_> = text::Encoder::new(&self.message_buffer).collect();
        y += draw_text(drawer, &self.font, &encoded_buffer, &self.inner_rect);
        y += self.font.line_height();

        // Draw the rest of the chat messages
        for message in self.messages.iter().rev() {
            y += draw_text(drawer, &self.font, &message.user_name,
                &Rect::new(self.inner_rect.x, y, self.inner_rect.w, self.inner_rect.h));

            y += draw_text(drawer, &self.font, &message.data,
                &Rect::new(self.inner_rect.x + msg_padding, y,
                self.inner_rect.w - msg_padding, self.inner_rect.h));

            y += self.font.line_height();
        }

        // Draw the chat border
        self.border.draw_box(drawer, self.outer_rect);
    }
}

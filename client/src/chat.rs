use std::mem;

use sdl2::render::RenderDrawer;
use sdl2::rect::Rect;

use interface::text;
use font::{Font, draw_text};

struct Message {
    user_name: Vec<u8>,
    data: Vec<u8>,
}

pub struct ChatBox {
    pub message_ready: bool,
    pub message_buffer: String,
    messages: Vec<Message>,
}

impl ChatBox {
    pub fn new() -> ChatBox {
        ChatBox {
            message_ready: false,
            message_buffer: String::new(),
            messages: Vec::new(),
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

    pub fn draw(&self, drawer: &mut RenderDrawer, font: &Font, rect: Rect) {
        let mut y = rect.y;
        let msg_padding = 1 * font.char_width();

        // Draw the text that the player is currently typing
        let encoded_buffer: Vec<_> = text::Encoder::new(&self.message_buffer).collect();
        y += draw_text(drawer, font, &encoded_buffer, &rect);

        for message in self.messages.iter().rev() {
            y += draw_text(drawer, font, &message.user_name,
                &Rect::new(rect.x, y, rect.w, rect.h));

            y += draw_text(drawer, font, &message.data,
                &Rect::new(rect.x + msg_padding, y, rect.w - msg_padding, rect.h));

            y += font.line_height() / 2;
        }
    }
}

use std::mem;

use sdl2::render::Renderer;
use sdl2::rect::Rect;
use sdl2::SdlResult;

use interface::text::{self, draw_text};
use font::Font;

struct Message {
    user_name: Vec<u8>,
    data: Vec<u8>,
}

pub struct ChatBox {
    messages: Vec<Message>,
    message_buffer: String,
}

impl ChatBox {
    pub fn new() -> ChatBox {
        ChatBox {
            messages: Vec::new(),
            message_buffer: String::new(),
        }
    }

    pub fn push_char(&mut self, value: char) {
        self.message_buffer.push(value);
    }

    pub fn remove_char(&mut self) {
        self.message_buffer.pop();
    }

    pub fn get_message_buffer(&mut self) -> String {
        mem::replace(&mut self.message_buffer, String::new())
    }

    pub fn add_message(&mut self, user_name: Vec<u8>, msg: Vec<u8>) {
        self.messages.push(Message {
            user_name: user_name,
            data: msg,
        });
    }

    pub fn draw(&self, renderer: &Renderer, font: &Font, rect: Rect) -> SdlResult<()> {
        let mut y = rect.y;
        let msg_padding = 2 * font.char_width();

        // Draw the text that the player is currently typing
        let encoded_buffer: Vec<_> = text::Encoder::new(&*self.message_buffer).collect();
        y += try!(draw_text(renderer, font, &*encoded_buffer, &rect));

        for message in self.messages.iter().rev() {
            y += try!(draw_text(renderer, font, &*message.user_name,
                &Rect::new(rect.x, y, rect.w, rect.h)));

            y += try!(draw_text(renderer, font, &*message.data,
                &Rect::new(rect.x + msg_padding, y, rect.w - msg_padding, rect.h)));

            y += font.line_height() / 2;
        }

        Ok(())
    }
}

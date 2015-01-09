use std::mem;

use sdl2::render::{Texture, Renderer, RendererFlip};
use sdl2::rect::Rect;
use sdl2::SdlResult;

use font::{self, Font, FontEncodedString};

struct Message {
    data: FontEncodedString
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

    pub fn add_message(&mut self, message: &str) {
        self.messages.push(Message { data: font::encode_string(message) });
    }

    pub fn draw(&self, renderer: &Renderer, font: &Font, rect: Rect) -> SdlResult<()> {
        let mut y = rect.y;

        let encoded_message_buffer = font::encode_string(&*self.message_buffer);
        y += try!(font.draw_text(renderer, &encoded_message_buffer, rect.x, y, Some(rect.w)));

        for message in self.messages.iter().rev() {
            y += try!(font.draw_text(renderer, &message.data, rect.x, y, Some(rect.w)));
        }

        Ok(())
    }
}

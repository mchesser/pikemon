use sdl2::render::{Texture, Renderer, RendererFlip};
use sdl2::rect::Rect;
use sdl2::SdlResult;

/// Special characters
pub mod special {
    pub const SPACE: u8 = 200;
    pub const NEW_LINE: u8 = 201;
    pub const NONE: u8 = 202;
    pub const TAB: u8 = 203;
}

pub struct FontEncodedString(Vec<u8>);

pub fn encode_string(input: &str) -> FontEncodedString {
    let mut buffer = Vec::with_capacity(input.len());
    for character in input.chars() {
        buffer.push(match character {
            'A'...'Z' => 0 + character as u8 - 'A' as u8,
            '('       => 26,
            ')'       => 27,
            ':'       => 28,
            ';'       => 29,
            '['       => 30,
            ']'       => 31,

            'a'...'z' => 32 + character as u8 - 'a' as u8,
            'é'       => 58,

            '\''      => 96, // TODO: We might want to handle this case more carefully
            '-'       => 98,
            '?'       => 101,
            '!'       => 102,
            '.'       => 103,
            '/'       => 115,
            ','       => 116,

            '0'...'9' => 118 + character as u8 - '0' as u8,

            ' '       => special::SPACE,
            '\n'      => special::NEW_LINE,
            '\r'      => special::NONE,
            '\t'      => special::TAB,

            _         => 101,
        });
    }
    FontEncodedString(buffer)
}

pub struct Font {
    texture: Texture,
    char_height: i32,
    char_width: i32,
}
impl Font {
    pub fn new(texture: Texture, char_height: i32, char_width: i32) -> Font {
        Font {
            texture: texture,
            char_height: char_height,
            char_width: char_width,
        }
    }

    fn draw_char(&self, renderer: &Renderer, value: u8, x: i32, y: i32) -> SdlResult<()> {
        let offset = value as i32 * self.char_width;
        let source_rect = Rect::new(offset, 0, self.char_width, self.char_height);
        let dest_rect = Rect::new(x, y, self.char_width, self.char_height);

        renderer.copy_ex(&self.texture, Some(source_rect), Some(dest_rect), 0.0, None,
            RendererFlip::None)
    }

    /// Draw text, returning the total height of the text drawn
    pub fn draw_text(&self, renderer: &Renderer, text: &FontEncodedString, start_x: i32,
        start_y: i32, wrap_width: Option<i32>) -> SdlResult<i32>
    {
        const TAB_SIZE: i32 = 4;

        let &FontEncodedString(ref string) = text;
        let mut x = start_x;
        let mut y = start_y;

        for &character in string.iter() {
            match character {
                special::SPACE => x += self.char_width,
                special::NEW_LINE => {
                    x = 0;
                    y += self.char_height;
                },
                special::NONE => {},
                special::TAB => x += TAB_SIZE * self.char_width,

                normal => {
                    try!(self.draw_char(renderer, normal, x, y));
                    x += self.char_width;
                },
            }

            // Check for wrapping
            if let Some(max_width) = wrap_width {
                if x + self.char_width > max_width {
                    x = 0;
                    y += self.char_height;
                }
            }
        }

        Ok(y - start_y + self.char_height)
    }
}


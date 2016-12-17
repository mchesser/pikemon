use sdl2::rect::Rect as SdlRect;

#[derive(Copy, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect {
            x: x,
            y: y,
            width: width,
            height: height
        }
    }

    pub fn to_sdl(&self) -> SdlRect {
        SdlRect::new(self.x, self.y, self.width as u32, self.height as u32)
    }
}
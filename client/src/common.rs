use macroquad::{
    prelude::{Vec2, WHITE},
    texture::{draw_texture_ex, Texture2D},
};

#[derive(Copy, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect { x, y, width, height }
    }
}

pub struct Renderer;

impl Renderer {
    pub fn copy(&self, texture: Texture2D, from: Option<Rect>, to: Option<Rect>) {
        let to_point = to.map_or(Vec2::ZERO, |v| Vec2::new(v.x as f32, v.y as f32));
        draw_texture_ex(
            texture,
            to_point.x,
            to_point.y,
            WHITE,
            macroquad::texture::DrawTextureParams {
                dest_size: to.map(|r| Vec2::new(r.width as f32, r.height as f32)),
                source: from.map(|r| {
                    macroquad::prelude::Rect::new(
                        r.x as f32,
                        r.y as f32,
                        r.width as f32,
                        r.height as f32,
                    )
                }),
                ..Default::default()
            },
        )
    }
}

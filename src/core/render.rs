use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, Event, HtmlImageElement, HtmlCanvasElement};
use strum_macros::{EnumIter, EnumCount};
use strum::{IntoEnumIterator, EnumCount};
use glam::Vec2;
use std::cell::OnceCell;

use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Debug, Copy, Clone, EnumIter)]
#[repr(usize)]
pub enum Sprite {
    CrabOrange,
    CrabGreen,
    Parallax,
    Tileset,
    HealthFrame
}

struct SpriteParams {
    path: &'static str,
    h_frames: u32,
    v_frames: u32
}

#[derive(Debug)]
struct SpriteData {
    image: HtmlImageElement,
    pub frame_width: u32,
    pub frame_height: u32
}

#[derive(Copy, Clone)]
pub enum BitmapFont {
    Numbers28,
    Numbers16
}

#[derive(EnumCount, Copy, Clone)]
#[repr(u32)]
pub enum BitmapFontColor {
    White,
    Yellow,
    Red
}

struct Renderer {
    context: CanvasRenderingContext2d,
    sprite_data: Vec<SpriteData>,

    font_numbers28: HtmlImageElement,
    font_numbers16: HtmlImageElement
}

fn render_get_sprite_params(sprite: Sprite) -> SpriteParams {
    match sprite {
        Sprite::CrabOrange => SpriteParams {
            path: "res/crab_orange.png",
            h_frames: 8,
            v_frames: 3
        },
        Sprite::CrabGreen => SpriteParams {
            path: "res/crab_green.png",
            h_frames: 8,
            v_frames: 3
        },
        Sprite::Parallax => SpriteParams {
            path: "res/parallax.png",
            h_frames: 1,
            v_frames: 1
        },
        Sprite::Tileset => SpriteParams {
            path: "res/tileset.png",
            h_frames: 13,
            v_frames: 3
        },
        Sprite::HealthFrame => SpriteParams {
            path: "res/health_frame.png",
            h_frames: 2,
            v_frames: 1
        }
    }
}

async fn render_load_sprite(sprite: Sprite) -> Result<SpriteData, JsValue> {
    let sprite_params = render_get_sprite_params(sprite);
    let message = format!("Loading image {}...", sprite_params.path);
    web_sys::console::debug_1(&message.into());

    let sprite_image = render_load_image(sprite_params.path).await?;
    let sprite_width = sprite_image.width();
    let sprite_height = sprite_image.height();

    Ok(SpriteData {
        image: sprite_image,
        frame_width: sprite_width / sprite_params.h_frames,
        frame_height: sprite_height / sprite_params.v_frames,
    })
}

async fn render_load_image(path: &str) -> Result<HtmlImageElement, JsValue> {
    let image = HtmlImageElement::new()?;
    let image_for_load = image.clone();

    let promise = web_sys::js_sys::Promise::new(&mut move |resolve, reject| {
        let success_image = image_for_load.clone();

        let onload = Closure::once(move |_e: Event| {
            let _ = resolve.call1(&JsValue::NULL, success_image.as_ref());
        });
        let onerror = Closure::once(move |_e: Event| {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("Failed to load image."));
        });

        image_for_load.set_onload(Some(onload.as_ref().unchecked_ref()));
        image_for_load.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        onload.forget();
        onerror.forget();
    });

    image.set_src(path);
    JsFuture::from(promise).await?;

    Ok(image)
}

thread_local! {
    static RENDERER: OnceCell<Renderer> = OnceCell::new();
}

pub async fn render_init() {
    // Get the global window and document objects
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Create canvas
    let canvas = document
        .create_element("canvas").unwrap()
        .dyn_into::<HtmlCanvasElement>().unwrap();
    canvas.set_id("game-canvas");

    // Set canvas style
    let style = canvas.style();
    style.set_property("width", "100vw").unwrap();
    style.set_property("height", "100vh").unwrap();
    style.set_property("display", "block").unwrap();
    style.set_property("image-rendering", "pixelated").unwrap();
    style.set_property("image-rendering", "-moz-crisp-edges").unwrap();
    style.set_property("image-rendering", "crisp-edges").unwrap();

    // Add canvas to body
    let body = document.body().unwrap();
    body.append_child(&canvas).unwrap();

    canvas.set_width(canvas.client_width() as u32);
    canvas.set_height(canvas.client_height() as u32);

    // Get context
    let context = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();
    context.set_image_smoothing_enabled(false);

    // Load sprites
    let mut sprite_data: Vec<SpriteData> = Vec::new();
    for sprite_name in Sprite::iter() {
        sprite_data.push(render_load_sprite(sprite_name).await.unwrap());
    }

    // Load bitmap font
    let font_numbers28 = render_load_image("res/numbers28.png").await.unwrap();
    let font_numbers16 = render_load_image("res/numbers16.png").await.unwrap();

    RENDERER.with(|cell| {
        cell.get_or_init(|| Renderer {
            context,
            sprite_data,
            font_numbers28,
            font_numbers16
        });
    });
}

pub fn render_get_sprite_frame_size(sprite: Sprite) -> Vec2 {
    RENDERER.with(|cell| {
        let renderer = cell.get().unwrap();
        let sprite_data: &SpriteData = &renderer.sprite_data[sprite as usize];
        Vec2::new(sprite_data.frame_width as f32, sprite_data.frame_height as f32)
    })
}

pub fn render_clear() {
    RENDERER.with(|cell| {
        let renderer = cell.get().unwrap();
        renderer.context.set_fill_style_str("#f0f0f0");
        renderer.context.fill_rect(0.0, 0.0, SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
    });
}

pub fn render_fill_rect(color: &str, position: Vec2, size: Vec2) {
    RENDERER.with(|cell| {
        let renderer = cell.get().unwrap();
        renderer.context.set_fill_style_str(color);
        renderer.context.fill_rect(position.x as f64, position.y as f64, size.x as f64, size.y as f64);
    });
}

pub fn render_draw_rect(color: &str, position: Vec2, size: Vec2) {
    RENDERER.with(|cell| {
        let renderer = cell.get().unwrap();

        renderer.context.set_stroke_style_str(color);
        renderer.context.stroke_rect(position.x as f64, position.y as f64, size.x as f64, size.y as f64);
    });
}

pub fn render_text(text: &str, position: Vec2) {
    RENDERER.with(|cell| {
        let renderer = cell.get().unwrap();

        renderer.context.set_font("10px sans-serif");
        renderer.context.set_fill_style_str("#fff");
        renderer.context.fill_text(text, position.x as f64, position.y as f64).unwrap();
    });
}

pub fn render_sprite(sprite: Sprite, position: Vec2, h_frame: u32, v_frame: u32, flip_h: bool) {
    RENDERER.with(|cell| {
        let renderer = cell.get().unwrap();
        let sprite_data: &SpriteData = &renderer.sprite_data[sprite as usize];

        let mut position_x = position.x as f64;
        let mut position_y = position.y as f64;

        if flip_h {
            renderer.context.save();
            renderer.context.translate(position_x + (sprite_data.frame_width as f64), position.y as f64).unwrap();
            renderer.context.scale(-1.0, 1.0).unwrap();
            position_x = 0.0;
            position_y = 0.0;
        }

        renderer.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &sprite_data.image, // Image
            (h_frame * sprite_data.frame_width) as f64, // Source X
            (v_frame * sprite_data.frame_height) as f64, // Source Y
            sprite_data.frame_width as f64, // Source Width
            sprite_data.frame_height as f64, // Source Height
            position_x,
            position_y,
            sprite_data.frame_width as f64, // Dest Width
            sprite_data.frame_height as f64) // Dest Height
        .unwrap();

        if flip_h {
            renderer.context.restore();
        }
    });
}

// Returns the width of the rendered text
pub fn render_bitmap_text(text: &str, font: BitmapFont, color: BitmapFontColor, position: Vec2) -> f32 {
    RENDERER.with(|cell| {
        let renderer = cell.get().unwrap();
        let image = match font {
            BitmapFont::Numbers28 => &renderer.font_numbers28,
            BitmapFont::Numbers16 => &renderer.font_numbers16,
        };
        let glyph_height = (image.height() / (BitmapFontColor::COUNT as u32)) as f64;
        let source_y = glyph_height * (color as u32 as f64);

        let mut text_width: f32 = 0.0;
        for character in text.chars() {
            let (glyph_offset, glyph_width) = render_get_bitmap_glyph_offset_and_width(font, character);
            renderer.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                glyph_offset as f64,
                source_y,
                glyph_width as f64,
                glyph_height,
                (position.x + text_width) as f64,
                position.y as f64,
                glyph_width as f64,
                glyph_height as f64)
            .unwrap();
            text_width += glyph_width as f32;
        }

        text_width
    })
}

fn render_get_bitmap_glyph_offset_and_width(font: BitmapFont, character: char) -> (u32, u32) {
    match font {
        BitmapFont::Numbers28 => {
            match character {
                '0' => (0, 12),
                '1' => (12, 9),
                '2' => (21, 12),
                '3' => (33, 12),
                '4' => (45, 14),
                '5' => (59, 11),
                '6' => (70, 12),
                '7' => (82, 12),
                '8' => (94, 14),
                '9' => (108, 10),
                _ => {
                    assert!(false);
                    (0, 0)
                }
            }
        },
        BitmapFont::Numbers16 => {
            match character {
                '0' => (0, 7),
                '1' => (7, 5),
                '2' => (12, 7),
                '3' => (19, 7),
                '4' => (26, 8),
                '5' => (34, 6),
                '6' => (40, 7),
                '7' => (47, 7),
                '8' => (54, 8),
                '9' => (62, 6),
                '.' => (68, 3),
                '%' => (71, 8),
                _ => {
                    assert!(false);
                    (0, 0)
                }
            }
        }
    }
}

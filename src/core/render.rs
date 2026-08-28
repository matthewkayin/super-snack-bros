use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, Event, HtmlImageElement, HtmlCanvasElement};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use glam::Vec2;
use std::cell::OnceCell;

use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Debug, Copy, Clone, EnumIter)]
#[repr(usize)]
pub enum Sprite {
    Crab
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

struct Renderer {
    context: CanvasRenderingContext2d,
    sprite_data: Vec<SpriteData>
}

fn render_get_sprite_params(sprite: Sprite) -> SpriteParams {
    match sprite {
        Sprite::Crab => SpriteParams {
            path: "res/crab.png",
            h_frames: 13,
            v_frames: 1
        },
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
    let canvas = document.get_element_by_id("game-canvas")
        .unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
    let context = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

    // Load sprites
    let mut sprite_data: Vec<SpriteData> = Vec::new();
    for sprite_name in Sprite::iter() {
        sprite_data.push(render_load_sprite(sprite_name).await.unwrap());
    }

    RENDERER.with(|cell| {
        cell.get_or_init(|| Renderer {
            context,
            sprite_data
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
            sprite_data.frame_height as f64).unwrap(); // Dest Height

        if flip_h {
            renderer.context.restore();
        }
    });
}

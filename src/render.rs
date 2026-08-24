use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, Event, HtmlImageElement, HtmlCanvasElement};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use glam::Vec2;

use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Debug, Copy, Clone, EnumIter)]
#[repr(usize)]
pub enum SpriteName {
    Crab
}

struct SpriteParams {
    path: &'static str,
    h_frames: u32,
    v_frames: u32
}

#[derive(Debug)]
pub struct SpriteData {
    image: HtmlImageElement,
    pub frame_width: u32,
    pub frame_height: u32
}

pub struct Renderer {
    context: CanvasRenderingContext2d,
    sprite_data: Vec<SpriteData>
}

fn render_get_sprite_params(sprite_name: SpriteName) -> SpriteParams {
    match sprite_name {
        SpriteName::Crab => SpriteParams {
            path: "res/crab.png",
            h_frames: 17,
            v_frames: 1
        },
    }
}

async fn render_load_sprite(sprite_name: SpriteName) -> Result<SpriteData, JsValue> {
    let sprite_params = render_get_sprite_params(sprite_name);
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

impl Renderer {
    pub fn new() -> Self {
        // Get the global window and document objects
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("game-canvas")
            .unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
        let context = canvas.get_context("2d").unwrap().unwrap()
            .dyn_into::<CanvasRenderingContext2d>().unwrap();

        Renderer {
            context,
            sprite_data: Vec::new()
        }
    }

    pub async fn load_sprites(&mut self) {
        // Load sprites
        for sprite_name in SpriteName::iter() {
            self.sprite_data.push(render_load_sprite(sprite_name).await.unwrap());
        }
    }

    pub fn render_clear(&self) {
        self.context.set_fill_style_str("#f0f0f0");
        self.context.fill_rect(0.0, 0.0, SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
    }

    pub fn render_sprite(&self, sprite_name: SpriteName, position: Vec2, h_frame: u32, v_frame: u32) {
        let sprite_data: &SpriteData = &self.sprite_data[sprite_name as usize];
        let result = self.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &sprite_data.image, // Image
            (h_frame * sprite_data.frame_width) as f64, // Source X
            (v_frame * sprite_data.frame_height) as f64, // Source Y
            sprite_data.frame_width as f64, // Source Width
            sprite_data.frame_height as f64, // Source Height
            position.x as f64, // Dest X
            position.y as f64, // Dest Y
            sprite_data.frame_width as f64, // Dest Width
            sprite_data.frame_height as f64); // Dest Height
        match result {
            Ok(_) => {},
            Err(js_value) => web_sys::console::error_1(&js_value)
        }
    }
}

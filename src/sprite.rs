use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlImageElement, Event};
use strum_macros::EnumIter;

struct SpriteParams {
    path: &'static str,
    h_frames: u32,
    v_frames: u32
}

#[derive(Debug, Copy, Clone, EnumIter)]
#[repr(usize)]
pub enum SpriteName {
    Crab
}

#[derive(Debug)]
pub struct SpriteData {
    pub image: HtmlImageElement,
    pub h_frames: u32,
    pub v_frames: u32,
    pub frame_width: u32,
    pub frame_height: u32
}

fn get_params(sprite_name: SpriteName) -> SpriteParams {
    match sprite_name {
        SpriteName::Crab => SpriteParams {
            path: "res/crab.png",
            h_frames: 17,
            v_frames: 1
        },
    }
}

pub async fn load(sprite_name: SpriteName) -> Result<SpriteData, JsValue> {
    let sprite_params = get_params(sprite_name);
    let message = format!("Loading image {}...", sprite_params.path);
    web_sys::console::debug_1(&message.into());

    let sprite_image = load_image(sprite_params.path).await?;
    let sprite_width = sprite_image.width();
    let sprite_height = sprite_image.height();

    Ok(SpriteData {
        image: sprite_image,
        h_frames: sprite_params.h_frames,
        v_frames: sprite_params.v_frames,
        frame_width: sprite_width / sprite_params.h_frames,
        frame_height: sprite_height / sprite_params.v_frames,
    })
}

async fn load_image(path: &str) -> Result<HtmlImageElement, JsValue> {
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

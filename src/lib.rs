mod constants;
mod game;

use constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, HtmlImageElement, Event};
use std::rc::Rc;
use std::cell::{RefCell};
use game::GameState;

struct Renderer {
    context: CanvasRenderingContext2d,
    sprite_crab: HtmlImageElement
}

async fn run() -> Result<(), JsValue> {
    // Get the global window and document objects
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("game-canvas")
        .unwrap().dyn_into::<HtmlCanvasElement>()?;

    let context = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

    // Load images
    let sprite_crab = load_image("res/crab.png").await?;

    // Bundle renderer
    let renderer = Renderer {
        context,
        sprite_crab
    };

    // Init game state

    let mut state = GameState::new();

    // GAME LOOP

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let f_for_loop = f.clone();

    const UPDATES_PER_SECOND: f64 = 60.0;
    const UPDATE_DURATION: f64 = 1000.0 / UPDATES_PER_SECOND;
    let mut last_time: f64 = 0.0;
    let mut accumulator: f64 = 0.0;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |current_time: f64| {
        let elapsed = current_time - last_time;
        last_time = current_time;
        accumulator += elapsed;

        while accumulator >= UPDATE_DURATION {
            update(&mut state);
            accumulator -= UPDATE_DURATION;
        }

        render(&renderer, &state);

        // Schedule next frame
        request_animation_frame(f_for_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    // Kick off the initial frame
    web_sys::window().unwrap().request_animation_frame(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
    )?;

    Ok(())
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


#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    web_sys::console::debug_1(&"Main started!".into());

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(err) = run().await {
            web_sys::console::error_1(&err);
        }
    });

    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

fn update(state: &mut GameState) {
    state.update();
}

fn render(renderer: &Renderer, state: &GameState) {
    renderer.context.set_fill_style_str("#000000");
    renderer.context.fill_rect(0.0, 0.0, SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);

    renderer.context.set_fill_style_str("#ffffff");
    renderer.context.draw_image_with_html_image_element(&renderer.sprite_crab, 0.0, 0.0).unwrap();
    // context.fill_rect(state.rect_x as f64, state.rect_y as f64, game::RECT_SIZE as f64, game::RECT_SIZE as f64);
}

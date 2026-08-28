mod constants;
mod core;
mod game;

use std::panic;
use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::cell::{RefCell};
use core::render::*;
use core::input::*;
use core::animation::*;
use game::state::GameState;

async fn run() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    render_init().await;
    web_sys::console::debug_1(&"Initialized renderer.".into());

    input_init().await;
    web_sys::console::debug_1(&"Initialized input.".into());

    animation_init();
    web_sys::console::debug_1(&"Initialized animation.".into());

    // Init game state

    let mut game_state = GameState::new();

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
            input_update();
            game_state.update();
            accumulator -= UPDATE_DURATION;
        }

        render_clear();
        game_state.render();

        // Schedule next frame
        request_animation_frame(f_for_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    // Kick off the initial frame
    web_sys::window().unwrap().request_animation_frame(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
    )?;

    Ok(())
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

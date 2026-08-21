mod constants;
use constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use std::rc::Rc;
use std::cell::RefCell;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    web_sys::console::debug_1(&"Main started!".into());

    // INIT

    // Get the global window and document objects
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("game-canvas")
        .unwrap().dyn_into::<HtmlCanvasElement>()?;

    web_sys::console::debug_1(&"Created HTML canvas.".into());

    let context = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

    // GAME LOOP

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let f_for_loop = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        render(&context);

        // Schedule next frame
        request_animation_frame(f_for_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    // Kick off the initial frame
    web_sys::window().unwrap().request_animation_frame(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
    )?;

    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

fn render(context: &CanvasRenderingContext2d) {
    context.set_fill_style_str("#000000");
    context.fill_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);

    context.set_fill_style_str("#ff0000");

    web_sys::console::log_1(&(format!("screen width {} screen height {}", SCREEN_WIDTH - 25.0, SCREEN_HEIGHT - 25.0).into()));
    context.fill_rect(0.0, 0.0, SCREEN_WIDTH - 25.0, SCREEN_HEIGHT - 25.0);
}

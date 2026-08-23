mod constants;
mod game;

use constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, HtmlImageElement};
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use game::GameState;

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

    // Load images

    let image_element = HtmlImageElement::new()?;
    let image_loaded = Rc::new(Cell::new(false));
    let image_loaded_clone = image_loaded.clone();
    let image_onload_closure = Closure::<dyn FnMut()>::new(move || {
        image_loaded_clone.set(true);
        web_sys::console::debug_1(&"Loaded image.".into());
    });
    image_element.set_onload(Some(image_onload_closure.as_ref().unchecked_ref()));
    image_onload_closure.forget();
    image_element.set_src("res/crab.png");

    web_sys::console::debug_1(&"Loading image...".into());

    // TODO: make this more general purpose and wait until loading is finished before rendering or updating

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

        render(&context, &state, &image_element);

        // Schedule next frame
        request_animation_frame(f_for_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    // Kick off the initial frame
    web_sys::window().unwrap().request_animation_frame(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
    )?;

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

fn render(context: &CanvasRenderingContext2d, state: &GameState, image: &HtmlImageElement) {
    context.set_fill_style_str("#000000");
    context.fill_rect(0.0, 0.0, SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);

    context.set_fill_style_str("#ffffff");
    context.draw_image_with_html_image_element(image, 0.0, 0.0).unwrap();
    // context.fill_rect(state.rect_x as f64, state.rect_y as f64, game::RECT_SIZE as f64, game::RECT_SIZE as f64);
}

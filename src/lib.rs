mod constants;
mod game;
mod sprite;
mod animation;

use constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use std::rc::Rc;
use std::cell::{RefCell};
use game::GameState;
use sprite::{SpriteData, SpriteName};
use strum::IntoEnumIterator;

struct Renderer {
    context: CanvasRenderingContext2d,
    sprite_data: Vec<SpriteData>,
}

struct RenderSpriteParams {
    sprite_name: SpriteName,
    x: f32,
    y: f32,
    h_frame: u32,
    v_frame: u32
}

async fn run() -> Result<(), JsValue> {
    // Get the global window and document objects
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("game-canvas")
        .unwrap().dyn_into::<HtmlCanvasElement>()?;

    let context = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

    // Load sprites
    let mut sprite_data = Vec::new();
    for sprite_name in SpriteName::iter() {
        sprite_data.push(sprite::load(sprite_name).await?);
    }

    // Bundle renderer
    let renderer = Renderer {
        context,
        sprite_data: sprite_data
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
    renderer.context.set_fill_style_str("#f0f0f0");
    renderer.context.fill_rect(0.0, 0.0, SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);

    render_sprite(renderer, &RenderSpriteParams {
        sprite_name: SpriteName::Crab,
        x: 10.0,
        y: 10.0,
        h_frame: state.crab_anim.h_frame,
        v_frame: state.crab_anim.v_frame
    });
}

fn render_sprite(renderer: &Renderer, params: &RenderSpriteParams) {
    let sprite_data: &SpriteData = &renderer.sprite_data[params.sprite_name as usize];
    let result = renderer.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &sprite_data.image,
        (params.h_frame * sprite_data.frame_width) as f64,
        (params.v_frame * sprite_data.frame_height) as f64,
        sprite_data.frame_width as f64,
        sprite_data.frame_height as f64,
        params.x as f64,
        params.y as f64,
        sprite_data.frame_width as f64,
        sprite_data.frame_height as f64);
    match result {
        Ok(_) => {},
        Err(js_value) => web_sys::console::error_1(&js_value)
    }
}

use rcade_plugin_input_classic::ClassicController;
use strum_macros::EnumCount;
use std::cell::{RefCell, OnceCell};

#[derive(EnumCount, Debug, Copy, Clone)]
#[repr(usize)]
pub enum InputAction {
    PlayerOneUp,
    PlayerOneRight,
    PlayerOneDown,
    PlayerOneLeft,
    PlayerOneA,
    PlayerOneB,
    PlayerTwoUp,
    PlayerTwoRight,
    PlayerTwoDown,
    PlayerTwoLeft,
    PlayerTwoA,
    PlayerTwoB,
    Count
}

const ACTION_COUNT: usize = InputAction::Count as usize;

pub struct Input {
    controller: ClassicController,
    current: RefCell<[bool; InputAction::Count as usize]>,
    previous: RefCell<[bool; InputAction::Count as usize]>,
}

fn input_controller_state_to_action_state(controller: &ClassicController) -> [bool; ACTION_COUNT] {
    let mut current = [false; ACTION_COUNT];
    let state = controller.state();

    current[InputAction::PlayerOneUp as usize] = state.player1_up;
    current[InputAction::PlayerOneRight as usize] = state.player1_right;
    current[InputAction::PlayerOneDown as usize] = state.player1_down;
    current[InputAction::PlayerOneLeft as usize] = state.player1_left;
    current[InputAction::PlayerOneA as usize] = state.player1_a;
    current[InputAction::PlayerOneB as usize] = state.player1_b;
    current[InputAction::PlayerTwoUp as usize] = state.player2_up;
    current[InputAction::PlayerTwoRight as usize] = state.player2_right;
    current[InputAction::PlayerTwoDown as usize] = state.player2_down;
    current[InputAction::PlayerTwoLeft as usize] = state.player2_left;
    current[InputAction::PlayerTwoA as usize] = state.player2_a;
    current[InputAction::PlayerTwoB as usize] = state.player2_b;

    current
}

thread_local! {
    static INPUT_STATE: OnceCell<Input> = OnceCell::new();
}

pub async fn input_init() {
    let controller = ClassicController::acquire().await.unwrap();
    let current = input_controller_state_to_action_state(&controller);

    INPUT_STATE.with(|cell| {
        cell.get_or_init(|| Input {
            controller,
            current: RefCell::new(current),
            previous: RefCell::new([false; ACTION_COUNT])
        });
    });
}

pub fn input_update() {
    INPUT_STATE.with(|cell| {
        let input = cell.get().unwrap();
        let current = input_controller_state_to_action_state(&input.controller);
        *input.previous.borrow_mut() = input.current.borrow().clone();
        *input.current.borrow_mut() = current;
    });
}

pub fn input_is_action_pressed(action: InputAction) -> bool {
    INPUT_STATE.with(|cell| {
        let input = cell.get().unwrap();
        let current = input.current.borrow();
        (*current)[action as usize]
    })
}

pub fn input_is_action_just_pressed(action: InputAction) -> bool {
    INPUT_STATE.with(|cell| {
        let input = cell.get().unwrap();
        let current = input.current.borrow();
        let previous = input.previous.borrow();
        (*current)[action as usize] && !(*previous)[action as usize]
    })
}

pub fn input_is_action_just_released(action: InputAction) -> bool {
    INPUT_STATE.with(|cell| {
        let input = cell.get().unwrap();
        let current = input.current.borrow();
        let previous = input.previous.borrow();
        !(*current)[action as usize] && (*previous)[action as usize]
    })
}

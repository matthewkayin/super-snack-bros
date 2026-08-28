use rcade_plugin_input_classic::ClassicController;
use strum::EnumCount;
use strum_macros::EnumCount;
use std::cell::{RefCell, OnceCell};

#[derive(Debug, EnumCount, Copy, Clone)]
#[repr(usize)]
pub enum InputAction {
    Up,
    Right,
    Down,
    Left,
    A,
    B
}

#[derive(Debug, EnumCount, Copy, Clone)]
#[repr(usize)]
pub enum InputPlayer {
    One,
    Two
}

const INPUT_ACTION_COUNT: usize = InputAction::COUNT as usize;
const INPUT_PLAYER_COUNT: usize = InputPlayer::COUNT as usize;
const INPUT_TOTAL_COUNT: usize = INPUT_ACTION_COUNT * INPUT_PLAYER_COUNT;

struct Input {
    controller: ClassicController,
    current: RefCell<[bool; INPUT_TOTAL_COUNT]>,
    previous: RefCell<[bool; INPUT_TOTAL_COUNT]>,
}

fn input_controller_state_to_action_state(controller: &ClassicController) -> [bool; INPUT_TOTAL_COUNT] {
    let mut current = [false; INPUT_TOTAL_COUNT];
    let state = controller.state();

    current[input_action_index(InputPlayer::One, InputAction::Up)] = state.player1_up;
    current[input_action_index(InputPlayer::One, InputAction::Right)] = state.player1_right;
    current[input_action_index(InputPlayer::One, InputAction::Down)] = state.player1_down;
    current[input_action_index(InputPlayer::One, InputAction::Left)] = state.player1_left;
    current[input_action_index(InputPlayer::One, InputAction::A)] = state.player1_a;
    current[input_action_index(InputPlayer::One, InputAction::B)] = state.player1_b;

    current[input_action_index(InputPlayer::Two, InputAction::Up)] = state.player2_up;
    current[input_action_index(InputPlayer::Two, InputAction::Right)] = state.player2_right;
    current[input_action_index(InputPlayer::Two, InputAction::Down)] = state.player2_down;
    current[input_action_index(InputPlayer::Two, InputAction::Left)] = state.player2_left;
    current[input_action_index(InputPlayer::Two, InputAction::A)] = state.player2_a;
    current[input_action_index(InputPlayer::Two, InputAction::B)] = state.player2_b;

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
            previous: RefCell::new([false; INPUT_TOTAL_COUNT])
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

fn input_action_index(player: InputPlayer, action: InputAction) -> usize {
    ((player as usize) * INPUT_ACTION_COUNT) + (action as usize)
}

pub fn input_is_action_pressed(player: InputPlayer, action: InputAction) -> bool {
    INPUT_STATE.with(|cell| {
        let input = cell.get().unwrap();
        let current = input.current.borrow();
        let index = input_action_index(player, action);
        (*current)[index]
    })
}

pub fn input_is_action_just_pressed(player: InputPlayer, action: InputAction) -> bool {
    INPUT_STATE.with(|cell| {
        let input = cell.get().unwrap();
        let current = input.current.borrow();
        let previous = input.previous.borrow();
        let index = input_action_index(player, action);
        (*current)[index] && !(*previous)[index]
    })
}

pub fn input_is_action_just_released(player: InputPlayer, action: InputAction) -> bool {
    INPUT_STATE.with(|cell| {
        let input = cell.get().unwrap();
        let current = input.current.borrow();
        let previous = input.previous.borrow();
        let index = input_action_index(player, action);
        !(*current)[index] && (*previous)[index]
    })
}

use rcade_plugin_input_classic::ClassicController;

#[derive(Debug, Copy, Clone)]
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
    current: [bool; ACTION_COUNT],
    previous: [bool; ACTION_COUNT],
}

pub async fn init() -> Input {
    let controller = ClassicController::acquire().await.unwrap();
    let current = input_controller_state_to_action_state(&controller);

    Input {
        controller,
        current,
        previous: [false; ACTION_COUNT]
    }
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

impl Input {
    pub fn update(&mut self) {
        let current = input_controller_state_to_action_state(&self.controller);
        self.previous = self.current;
        self.current = current;
    }

    pub fn is_action_pressed(&self, action: InputAction) -> bool {
        self.current[action as usize]
    }

    pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        self.current[action as usize] && !self.previous[action as usize]
    }

    pub fn is_action_just_released(&self, action: InputAction) -> bool {
        !self.current[action as usize] && self.previous[action as usize]
    }
}

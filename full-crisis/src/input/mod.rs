/// Game controller input abstraction
/// 
/// This module provides a unified interface for game controller input across
/// different platforms (native desktop via gilrs, web via Gamepad API)

use crate::gui::types::GameMessage;

/// Standard controller input events that can be generated on any platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerInput {
    /// D-pad or left stick up
    Up,
    /// D-pad or left stick down
    Down,
    /// D-pad or left stick left
    Left,
    /// D-pad or left stick right
    Right,
    /// A button (Xbox) / Cross button (PlayStation) / B button (Nintendo)
    ActionPrimary,
    /// B button (Xbox) / Circle button (PlayStation) / A button (Nintendo)
    ActionSecondary,
    /// X button (Xbox) / Square button (PlayStation) / Y button (Nintendo)
    ActionTertiary,
    /// Y button (Xbox) / Triangle button (PlayStation) / X button (Nintendo)
    ActionQuaternary,
    /// Left shoulder button (LB/L1)
    LeftShoulder,
    /// Right shoulder button (RB/R1)
    RightShoulder,
    /// Left trigger (LT/L2) - treated as digital for simplicity
    LeftTrigger,
    /// Right trigger (RT/R2) - treated as digital for simplicity
    RightTrigger,
    /// Start/Menu/Plus button
    Start,
    /// Select/View/Minus button
    Select,
}

/// Controller input state and manager
pub trait ControllerManager: Send {
    /// Poll for new controller input events
    /// Returns None if no events are available
    fn poll_events(&mut self) -> Option<ControllerInput>;
    
    /// Check if any controllers are connected
    fn has_connected_controllers(&self) -> bool;
    
    /// Get the number of connected controllers
    fn controller_count(&self) -> usize;
    
    /// Update controller state (called each frame)
    fn update(&mut self);
}

/// Input timing constants to match keyboard behavior
const INITIAL_REPEAT_DELAY_MS: u64 = 400;  // 400ms before first repeat (slightly faster than typical keyboard)
const REPEAT_INTERVAL_MS: u64 = 100;       // 100ms between repeats (faster for UI navigation)

/// Tracks timing state for input debouncing
#[derive(Debug, Clone)]
struct InputTimingState {
    last_input_time: crate::time::PlatformInstant,
    initial_delay_passed: bool,
}

impl InputTimingState {
    fn new() -> Self {
        Self {
            last_input_time: crate::time::now(),
            initial_delay_passed: false,
        }
    }
    
    /// Check if enough time has passed to allow a repeat input
    fn should_allow_input(&mut self, is_first_press: bool) -> bool {
        let now = crate::time::now();
        
        if is_first_press {
            // First press is always allowed
            self.last_input_time = now;
            self.initial_delay_passed = false;
            return true;
        }
        
        // For held inputs, check timing
        if !self.initial_delay_passed {
            if now.duration_since(self.last_input_time).as_millis() >= INITIAL_REPEAT_DELAY_MS as u128 {
                self.initial_delay_passed = true;
                self.last_input_time = now;
                return true;
            }
        } else {
            if now.duration_since(self.last_input_time).as_millis() >= REPEAT_INTERVAL_MS as u128 {
                self.last_input_time = now;
                return true;
            }
        }
        
        false
    }
}

/// Convert controller input to appropriate GameMessage
pub fn controller_input_to_game_message(input: ControllerInput) -> GameMessage {
    match input {
        ControllerInput::Up => GameMessage::Focus_NavigateUp,
        ControllerInput::Down => GameMessage::Focus_NavigateDown,
        ControllerInput::Left => GameMessage::Focus_NavigateLeft,     // Left panel navigation
        ControllerInput::Right => GameMessage::Focus_NavigateRight,   // Right panel navigation
        ControllerInput::ActionPrimary => GameMessage::Focus_Activate, // Enter key equivalent
        ControllerInput::ActionSecondary => GameMessage::QuitGameRequested, // Back/escape functionality
        ControllerInput::LeftShoulder => GameMessage::Focus_ShiftTabInteract, // Shift+Tab equivalent (L1/LB)
        ControllerInput::RightShoulder => GameMessage::Focus_TabInteract,     // Tab equivalent (R1/RB)
        ControllerInput::Start => GameMessage::Menu_SettingsRequested, // Menu access
        _ => GameMessage::Nop, // Other inputs not mapped yet
    }
}

/// Create a controller manager for the current platform
pub fn create_controller_manager() -> Box<dyn ControllerManager> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(web::WebControllerManager::new())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(native::NativeControllerManager::new())
    }
}

/// Native desktop controller implementation using gilrs
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use super::{ControllerInput, ControllerManager, InputTimingState};
    use gilrs::{Gilrs, Event, EventType, Button, Axis};
    
    pub struct NativeControllerManager {
        gilrs: Gilrs,
        // Track axis states for digital conversion
        left_stick_x_pressed_left: bool,
        left_stick_x_pressed_right: bool,
        left_stick_y_pressed_up: bool,
        left_stick_y_pressed_down: bool,
        // Track trigger states for digital conversion
        left_trigger_pressed: bool,
        right_trigger_pressed: bool,
        // Input timing management for keyboard-like behavior
        input_timing: std::collections::HashMap<ControllerInput, InputTimingState>,
    }
    
    impl NativeControllerManager {
        pub fn new() -> Self {
            let gilrs = Gilrs::new().unwrap_or_else(|_| {
                eprintln!("Warning: Failed to initialize game controller support");
                // Create a minimal Gilrs instance that won't panic
                Gilrs::new().unwrap()
            });
            
            Self {
                gilrs,
                left_stick_x_pressed_left: false,
                left_stick_x_pressed_right: false,
                left_stick_y_pressed_up: false,
                left_stick_y_pressed_down: false,
                left_trigger_pressed: false,
                right_trigger_pressed: false,
                input_timing: std::collections::HashMap::new(),
            }
        }
        
        /// Convert gilrs button to our ControllerInput
        fn button_to_input(button: Button) -> Option<ControllerInput> {
            match button {
                Button::DPadUp => Some(ControllerInput::Up),
                Button::DPadDown => Some(ControllerInput::Down),
                Button::DPadLeft => Some(ControllerInput::Left),
                Button::DPadRight => Some(ControllerInput::Right),
                Button::South => Some(ControllerInput::ActionPrimary),    // Xbox A, PS Cross
                Button::East => Some(ControllerInput::ActionSecondary),   // Xbox B, PS Circle  
                Button::West => Some(ControllerInput::ActionTertiary),    // Xbox X, PS Square
                Button::North => Some(ControllerInput::ActionQuaternary), // Xbox Y, PS Triangle
                Button::LeftTrigger => Some(ControllerInput::LeftShoulder),
                Button::RightTrigger => Some(ControllerInput::RightShoulder),
                Button::Start => Some(ControllerInput::Start),
                Button::Select => Some(ControllerInput::Select),
                _ => None,
            }
        }
        
        /// Handle analog stick input with deadzone
        fn handle_axis_input(&mut self, axis: Axis, value: f32) -> Option<ControllerInput> {
            const DEADZONE: f32 = 0.3;
            const RELEASE_DEADZONE: f32 = 0.2; // Smaller deadzone for releasing to prevent flicker
            
            match axis {
                Axis::LeftStickX => {
                    if value < -DEADZONE {
                        // Moving left
                        if !self.left_stick_x_pressed_left {
                            self.left_stick_x_pressed_left = true;
                            self.left_stick_x_pressed_right = false;
                            // Remove right timing state
                            self.input_timing.remove(&ControllerInput::Right);
                            Some(ControllerInput::Left)
                        } else {
                            // Already pressed, let timing system handle repeats
                            None
                        }
                    } else if value > DEADZONE {
                        // Moving right  
                        if !self.left_stick_x_pressed_right {
                            self.left_stick_x_pressed_right = true;
                            self.left_stick_x_pressed_left = false;
                            // Remove left timing state
                            self.input_timing.remove(&ControllerInput::Left);
                            Some(ControllerInput::Right)
                        } else {
                            // Already pressed, let timing system handle repeats
                            None
                        }
                    } else if value.abs() < RELEASE_DEADZONE {
                        // Released
                        if self.left_stick_x_pressed_left {
                            self.input_timing.remove(&ControllerInput::Left);
                        }
                        if self.left_stick_x_pressed_right {
                            self.input_timing.remove(&ControllerInput::Right);
                        }
                        self.left_stick_x_pressed_left = false;
                        self.left_stick_x_pressed_right = false;
                        None
                    } else {
                        None
                    }
                }
                Axis::LeftStickY => {
                    // Note: Y axis is usually inverted (negative = up)
                    if value < -DEADZONE {
                        // Moving up
                        if !self.left_stick_y_pressed_up {
                            self.left_stick_y_pressed_up = true;
                            self.left_stick_y_pressed_down = false;
                            // Remove down timing state
                            self.input_timing.remove(&ControllerInput::Down);
                            Some(ControllerInput::Up)
                        } else {
                            None
                        }
                    } else if value > DEADZONE {
                        // Moving down
                        if !self.left_stick_y_pressed_down {
                            self.left_stick_y_pressed_down = true;
                            self.left_stick_y_pressed_up = false;
                            // Remove up timing state
                            self.input_timing.remove(&ControllerInput::Up);
                            Some(ControllerInput::Down)
                        } else {
                            None
                        }
                    } else if value.abs() < RELEASE_DEADZONE {
                        // Released
                        if self.left_stick_y_pressed_up {
                            self.input_timing.remove(&ControllerInput::Up);
                        }
                        if self.left_stick_y_pressed_down {
                            self.input_timing.remove(&ControllerInput::Down);
                        }
                        self.left_stick_y_pressed_up = false;
                        self.left_stick_y_pressed_down = false;
                        None
                    } else {
                        None
                    }
                }
                Axis::LeftZ => { // Left trigger
                    if value > 0.5 && !self.left_trigger_pressed {
                        self.left_trigger_pressed = true;
                        Some(ControllerInput::LeftTrigger)
                    } else if value < 0.3 {
                        if self.left_trigger_pressed {
                            self.input_timing.remove(&ControllerInput::LeftTrigger);
                        }
                        self.left_trigger_pressed = false;
                        None
                    } else {
                        None
                    }
                }
                Axis::RightZ => { // Right trigger
                    if value > 0.5 && !self.right_trigger_pressed {
                        self.right_trigger_pressed = true;
                        Some(ControllerInput::RightTrigger)
                    } else if value < 0.3 {
                        if self.right_trigger_pressed {
                            self.input_timing.remove(&ControllerInput::RightTrigger);
                        }
                        self.right_trigger_pressed = false;
                        None
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
    
    impl ControllerManager for NativeControllerManager {
        fn poll_events(&mut self) -> Option<ControllerInput> {
            // First, process any new events to update our state
            while let Some(Event { event, .. }) = self.gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(button, _) => {
                        if let Some(input) = Self::button_to_input(button) {
                            // For button presses, always register as first press
                            let timing = self.input_timing.entry(input).or_insert_with(InputTimingState::new);
                            if timing.should_allow_input(true) {
                                return Some(input);
                            }
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        if let Some(input) = Self::button_to_input(button) {
                            // Remove timing state when button is released
                            self.input_timing.remove(&input);
                        }
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        if let Some(input) = self.handle_axis_input(axis, value) {
                            // Check if this is the first time this axis direction was pressed
                            let is_first_press = !self.input_timing.contains_key(&input);
                            let timing = self.input_timing.entry(input).or_insert_with(InputTimingState::new);
                            if timing.should_allow_input(is_first_press) {
                                return Some(input);
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Check for held inputs that should repeat
            let mut inputs_to_repeat = Vec::new();
            for (&input, timing) in &mut self.input_timing {
                if timing.should_allow_input(false) {
                    inputs_to_repeat.push(input);
                }
            }
            
            // Return the first repeatable input (if any)
            inputs_to_repeat.first().copied()
        }
        
        fn has_connected_controllers(&self) -> bool {
            self.gilrs.gamepads().any(|(_, gamepad)| gamepad.is_connected())
        }
        
        fn controller_count(&self) -> usize {
            self.gilrs.gamepads().filter(|(_, gamepad)| gamepad.is_connected()).count()
        }
        
        fn update(&mut self) {
            // Gilrs handles its own internal updates when we call next_event()
        }
    }
}

/// Web controller implementation using the Gamepad API
#[cfg(target_arch = "wasm32")]
pub mod web {
    use super::{ControllerInput, ControllerManager, InputTimingState};
    use wasm_bindgen::prelude::*;
    use web_sys::{window, Gamepad};
    
    pub struct WebControllerManager {
        // Track button states to detect button press events (not just held)
        prev_button_states: Vec<Vec<bool>>,
        // Track axis states for digital conversion
        prev_axis_states: Vec<Vec<f32>>,
        axis_pressed_states: Vec<Vec<bool>>, // Track which axes are currently "pressed"
        // Input timing management for keyboard-like behavior
        input_timing: std::collections::HashMap<ControllerInput, InputTimingState>,
    }
    
    impl WebControllerManager {
        pub fn new() -> Self {
            Self {
                prev_button_states: Vec::new(),
                prev_axis_states: Vec::new(),
                axis_pressed_states: Vec::new(),
                input_timing: std::collections::HashMap::new(),
            }
        }
        
        /// Get gamepads from browser API
        fn get_gamepads(&self) -> Vec<Gamepad> {
            let window = window().expect("should have window");
            let navigator = window.navigator();
            
            let gamepads_result = navigator.get_gamepads();
            if let Ok(gamepads_array) = gamepads_result {
                let mut gamepads = Vec::new();
                
                for i in 0..gamepads_array.length() {
                    let gamepad_value = gamepads_array.get(i);
                    if !gamepad_value.is_null() {
                        if let Ok(gamepad) = gamepad_value.dyn_into::<Gamepad>() {
                            if gamepad.connected() {
                                gamepads.push(gamepad);
                            }
                        }
                    }
                }
                
                gamepads
            } else {
                Vec::new()
            }
        }
        
        /// Convert standard gamepad button index to ControllerInput
        fn button_index_to_input(button_index: usize) -> Option<ControllerInput> {
            match button_index {
                0 => Some(ControllerInput::ActionPrimary),    // A/Cross
                1 => Some(ControllerInput::ActionSecondary),  // B/Circle
                2 => Some(ControllerInput::ActionTertiary),   // X/Square
                3 => Some(ControllerInput::ActionQuaternary), // Y/Triangle
                4 => Some(ControllerInput::LeftShoulder),     // LB/L1
                5 => Some(ControllerInput::RightShoulder),    // RB/R1
                6 => Some(ControllerInput::LeftTrigger),      // LT/L2
                7 => Some(ControllerInput::RightTrigger),     // RT/R2
                8 => Some(ControllerInput::Select),           // Select/View/Share
                9 => Some(ControllerInput::Start),            // Start/Menu/Options
                12 => Some(ControllerInput::Up),              // D-pad up
                13 => Some(ControllerInput::Down),            // D-pad down
                14 => Some(ControllerInput::Left),            // D-pad left
                15 => Some(ControllerInput::Right),           // D-pad right
                _ => None,
            }
        }
        
        /// Handle axis input with deadzone conversion
        fn handle_axis_input(&mut self, gamepad_index: usize, axis_index: usize, value: f32) -> Option<ControllerInput> {
            const DEADZONE: f32 = 0.3;
            
            // Ensure we have enough space in our tracking vectors
            while self.axis_pressed_states.len() <= gamepad_index {
                self.axis_pressed_states.push(Vec::new());
            }
            while self.axis_pressed_states[gamepad_index].len() <= axis_index {
                self.axis_pressed_states[gamepad_index].push(false);
            }
            
            let was_pressed = self.axis_pressed_states[gamepad_index][axis_index];
            
            match axis_index {
                0 => { // Left stick X
                    if value < -DEADZONE && !was_pressed {
                        self.axis_pressed_states[gamepad_index][axis_index] = true;
                        // Clear any previous opposing direction timing
                        self.input_timing.remove(&ControllerInput::Right);
                        Some(ControllerInput::Left)
                    } else if value > DEADZONE && !was_pressed {
                        self.axis_pressed_states[gamepad_index][axis_index] = true;
                        // Clear any previous opposing direction timing
                        self.input_timing.remove(&ControllerInput::Left);
                        Some(ControllerInput::Right)
                    } else if value.abs() < DEADZONE {
                        self.axis_pressed_states[gamepad_index][axis_index] = false;
                        // Clear timing state for both directions when axis is released
                        self.input_timing.remove(&ControllerInput::Left);
                        self.input_timing.remove(&ControllerInput::Right);
                        None
                    } else {
                        None
                    }
                }
                1 => { // Left stick Y (inverted)
                    if value < -DEADZONE && !was_pressed {
                        self.axis_pressed_states[gamepad_index][axis_index] = true;
                        // Clear any previous opposing direction timing
                        self.input_timing.remove(&ControllerInput::Down);
                        Some(ControllerInput::Up)
                    } else if value > DEADZONE && !was_pressed {
                        self.axis_pressed_states[gamepad_index][axis_index] = true;
                        // Clear any previous opposing direction timing
                        self.input_timing.remove(&ControllerInput::Up);
                        Some(ControllerInput::Down)
                    } else if value.abs() < DEADZONE {
                        self.axis_pressed_states[gamepad_index][axis_index] = false;
                        // Clear timing state for both directions when axis is released
                        self.input_timing.remove(&ControllerInput::Up);
                        self.input_timing.remove(&ControllerInput::Down);
                        None
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        
        /// Resize tracking vectors to match gamepad count
        fn ensure_gamepad_tracking_size(&mut self, gamepad_count: usize) {
            self.prev_button_states.resize(gamepad_count, Vec::new());
            self.prev_axis_states.resize(gamepad_count, Vec::new());
            self.axis_pressed_states.resize(gamepad_count, Vec::new());
        }
    }
    
    impl ControllerManager for WebControllerManager {
        fn poll_events(&mut self) -> Option<ControllerInput> {
            let gamepads = self.get_gamepads();
            if gamepads.is_empty() {
                return None;
            }
            
            self.ensure_gamepad_tracking_size(gamepads.len());
            
            // Check all gamepads for input changes
            for (gamepad_index, gamepad) in gamepads.iter().enumerate() {
                let buttons = gamepad.buttons();
                let axes = gamepad.axes();
                
                // Ensure our tracking vectors are the right size
                let button_count = buttons.length() as usize;
                let axis_count = axes.length() as usize;
                
                if self.prev_button_states[gamepad_index].len() != button_count {
                    self.prev_button_states[gamepad_index].resize(button_count, false);
                }
                if self.prev_axis_states[gamepad_index].len() != axis_count {
                    self.prev_axis_states[gamepad_index].resize(axis_count, 0.0);
                }
                
                // Check for button press/release events
                for button_index in 0..button_count {
                    let button_obj = buttons.get(button_index as u32);
                    if let Ok(button) = button_obj.dyn_into::<web_sys::GamepadButton>() {
                        let is_pressed = button.pressed();
                        let was_pressed = self.prev_button_states[gamepad_index][button_index];
                        
                        if is_pressed && !was_pressed {
                            // Button was just pressed
                            self.prev_button_states[gamepad_index][button_index] = true;
                            if let Some(input) = Self::button_index_to_input(button_index) {
                                let timing = self.input_timing.entry(input).or_insert_with(InputTimingState::new);
                                if timing.should_allow_input(true) {
                                    return Some(input);
                                }
                            }
                        } else if !is_pressed && was_pressed {
                            // Button was just released
                            self.prev_button_states[gamepad_index][button_index] = false;
                            if let Some(input) = Self::button_index_to_input(button_index) {
                                self.input_timing.remove(&input);
                            }
                        }
                    }
                }
                
                // Check for axis changes
                for axis_index in 0..axis_count {
                    let axis_value = axes.get(axis_index as u32);
                    let value = axis_value.as_f64().unwrap_or(0.0) as f32;
                    let prev_value = self.prev_axis_states[gamepad_index][axis_index];
                    
                    if (value - prev_value).abs() > 0.1 { // Threshold to avoid tiny changes
                        self.prev_axis_states[gamepad_index][axis_index] = value;
                        if let Some(input) = self.handle_axis_input(gamepad_index, axis_index, value) {
                            let is_first_press = !self.input_timing.contains_key(&input);
                            let timing = self.input_timing.entry(input).or_insert_with(InputTimingState::new);
                            if timing.should_allow_input(is_first_press) {
                                return Some(input);
                            }
                        }
                    }
                }
            }
            
            // Check for held inputs that should repeat (similar to native implementation)
            let mut inputs_to_repeat = Vec::new();
            for (&input, timing) in &mut self.input_timing {
                if timing.should_allow_input(false) {
                    inputs_to_repeat.push(input);
                }
            }
            
            // Return the first repeatable input (if any)
            inputs_to_repeat.first().copied()
        }
        
        fn has_connected_controllers(&self) -> bool {
            !self.get_gamepads().is_empty()
        }
        
        fn controller_count(&self) -> usize {
            self.get_gamepads().len()
        }
        
        fn update(&mut self) {
            // Web gamepad API is polled, no additional updates needed
        }
    }
}
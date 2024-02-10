use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlyAction {
    // Movement
    LateralMove,
    Jump,
    Look, // Mouse look motion
    Fast, // Speed up key
    NoClip,
    Down, // Move down in no-clip mode
    // Left click
    Fire,
    // Pause
    Pause,
}

pub fn create_input_manager_bundle() -> InputManagerBundle<PlyAction> {
    InputManagerBundle {
        action_state: default(),
        input_map: InputMap::default()
            // Movement
            .insert_many_to_one(
                [VirtualDPad::arrow_keys(), VirtualDPad::wasd()],
                PlyAction::LateralMove,
            )
            .insert(MouseButton::Left, PlyAction::Fire)
            .insert(Modifier::Shift, PlyAction::Fast)
            .insert(DualAxis::mouse_motion(), PlyAction::Look)
            .insert(KeyCode::Escape, PlyAction::Pause)
            .insert(KeyCode::Space, PlyAction::Jump)
            .insert(KeyCode::V, PlyAction::NoClip)
            .insert(KeyCode::ControlLeft, PlyAction::Down)
            // Finish
            .build(),
    }
}

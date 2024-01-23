use bevy::prelude::*;
use leafwing_input_manager::{axislike::VirtualAxis, prelude::*};

#[derive(Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlyAction {
    // Movement
    LateralMove,
    UpDown,
    // Left click
    Fire,
    // Speed up key
    Fast,
    // Mouse look motion
    Look,
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
            .insert(
                VirtualAxis {
                    negative: Modifier::Control.into(),
                    positive: KeyCode::Space.into(),
                },
                PlyAction::UpDown,
            )
            .insert(MouseButton::Left, PlyAction::Fire)
            .insert(Modifier::Shift, PlyAction::Fast)
            .insert(DualAxis::mouse_motion(), PlyAction::Look)
            .insert(KeyCode::Escape, PlyAction::Pause)
            // Finish
            .build(),
    }
}

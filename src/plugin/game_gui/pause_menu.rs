use super::{
    make_btn, menu_node, menu_title_text_bundle, menu_wrapper_node, update_state_button,
    update_state_system, MenuState,
};
use crate::{
    plugin::control::{input::PlyAction, pause::PauseState},
    FontAssets,
};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::Paused),
            (
                spawn_pause_menu_system,
                update_state_system(PauseState::Paused),
            ),
        )
        .add_systems(
            OnExit(MenuState::Paused),
            (despawn_pause_menu_system, exit_pause_menu_system),
        )
        .add_systems(
            Update,
            (
                update_state_button::<MainMenuButton, _>(MenuState::Paused, MenuState::MainMenu),
                update_state_button::<PauseSettingsButton, _>(
                    MenuState::Paused,
                    MenuState::PauseSettings,
                ),
                update_state_button::<UnpauseButton, _>(MenuState::Paused, MenuState::None),
                toggle_pause_menu_system,
            ),
        );
    }
}

fn exit_pause_menu_system(
    state: Res<State<MenuState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if *state.get() == MenuState::None {
        next_pause_state.set(PauseState::Playing);
    }
}

fn toggle_pause_menu_system(
    pause_state: Res<State<PauseState>>,
    menu_state: Res<State<MenuState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    query: Query<&ActionState<PlyAction>>,
) {
    if let Ok(ctrl) = query.get_single() {
        if ctrl.just_pressed(PlyAction::Pause) {
            match (pause_state.get(), menu_state.get()) {
                (PauseState::Playing, MenuState::None) => next_menu_state.set(MenuState::Paused),
                (PauseState::Paused, MenuState::Paused) => next_menu_state.set(MenuState::None),
                _ => {}
            }
        }
    }
}

#[derive(Component)]
struct PauseMenu;

#[derive(Component)]
struct UnpauseButton;

#[derive(Component)]
struct PauseSettingsButton;

#[derive(Component)]
struct MainMenuButton;

fn spawn_pause_menu_system(mut commands: Commands, font_assets: Res<FontAssets>) {
    // Entire screen node
    commands
        .spawn((PauseMenu, menu_wrapper_node()))
        .with_children(|commands| {
            // New world menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Menu title text
                commands.spawn(menu_title_text_bundle(&font_assets, "Game Paused"));

                // Buttons
                make_btn(commands, &font_assets, "Unpause", Some(UnpauseButton), true);
                make_btn(
                    commands,
                    &font_assets,
                    "Settings",
                    Some(PauseSettingsButton),
                    true,
                );
                make_btn(
                    commands,
                    &font_assets,
                    "Main Menu",
                    Some(MainMenuButton),
                    true,
                );
            });
        });
}

fn despawn_pause_menu_system(mut commands: Commands, query: Query<Entity, With<PauseMenu>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

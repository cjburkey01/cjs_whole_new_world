use super::{
    make_btn, menu_node, menu_title_text_bundle, menu_wrapper_node, update_state_button, MenuState,
};
use crate::FontAssets;
use bevy::prelude::*;

pub struct PauseSettingsMenuPlugin;

impl Plugin for PauseSettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::PauseSettings),
            spawn_pause_settings_menu_system,
        )
        .add_systems(
            OnExit(MenuState::PauseSettings),
            despawn_pause_settings_menu_system,
        )
        .add_systems(
            Update,
            update_state_button::<BackButton, _>(MenuState::PauseSettings, MenuState::Paused),
        );
    }
}

#[derive(Component)]
struct PauseSettingsMenu;

#[derive(Component)]
struct BackButton;

fn spawn_pause_settings_menu_system(mut commands: Commands, font_assets: Res<FontAssets>) {
    // Entire screen node
    commands
        .spawn((PauseSettingsMenu, menu_wrapper_node()))
        .with_children(|commands| {
            // New world menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Menu title text
                commands.spawn(menu_title_text_bundle(&font_assets, "Settings"));

                // Buttons
                make_btn(commands, &font_assets, "Back", Some(BackButton), true);
            });
        });
}

fn despawn_pause_settings_menu_system(
    mut commands: Commands,
    query: Query<Entity, With<PauseSettingsMenu>>,
) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

use super::{
    make_btn, menu_node, menu_title_text_bundle, menu_wrapper_node, update_event_button,
    update_state_button, MenuState, FULL_BACK_COVER_COLOR,
};
use crate::plugin::asset::FontAssets;
use bevy::{app::AppExit, prelude::*};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::MainMenu), spawn_game_menu_system)
            .add_systems(OnExit(MenuState::MainMenu), despawn_game_menu_system)
            .add_systems(
                Update,
                (
                    update_state_button::<NewWorldButton, _>(
                        MenuState::MainMenu,
                        MenuState::NewWorldMenu,
                    ),
                    update_event_button::<ExitButton, _>(AppExit)
                        .run_if(in_state(MenuState::MainMenu)),
                ),
            );
    }
}

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct NewWorldButton;

#[derive(Component)]
struct LoadWorldButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct ExitButton;

fn spawn_game_menu_system(mut commands: Commands, font_assets: Res<FontAssets>) {
    // Entire screen node
    commands
        .spawn((MainMenu, menu_wrapper_node(FULL_BACK_COVER_COLOR)))
        .with_children(|commands| {
            // Menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Game title text
                commands.spawn(menu_title_text_bundle(&font_assets, "Hello World!!"));

                make_btn(
                    commands,
                    &font_assets,
                    "New World",
                    Some(NewWorldButton),
                    true,
                );

                make_btn(
                    commands,
                    &font_assets,
                    "Load World",
                    Some(LoadWorldButton),
                    false,
                );

                make_btn(
                    commands,
                    &font_assets,
                    "Game Settings",
                    Some(SettingsButton),
                    false,
                );

                make_btn(commands, &font_assets, "Exit Game", Some(ExitButton), true);
            });
        });
}

fn despawn_game_menu_system(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

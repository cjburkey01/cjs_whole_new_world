use super::{
    label_bundle, menu_node, menu_title_text_bundle, menu_wrapper_node, MenuState,
    FULL_BACK_COVER_COLOR,
};
use crate::plugin::{asset::FontAssets, voxel_world::world_info::WorldInfo};
use bevy::prelude::*;

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::LoadingScreen),
            spawn_loading_screen_system,
        )
        .add_systems(
            OnExit(MenuState::LoadingScreen),
            despawn_loading_screen_system,
        );
    }
}

#[derive(Component)]
struct LoadingScreen;

fn spawn_loading_screen_system(
    mut commands: Commands,
    world_info: Res<WorldInfo>,
    font_assets: Res<FontAssets>,
) {
    let world_name = world_info.name();
    let world_seed = world_info.seed();

    // Entire screen node
    commands
        .spawn((LoadingScreen, menu_wrapper_node(FULL_BACK_COVER_COLOR)))
        .with_children(|commands| {
            // Menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Game title text
                commands.spawn(menu_title_text_bundle(
                    &font_assets,
                    "Yo world is generating",
                ));

                commands.spawn(label_bundle(
                    &font_assets.fira_sans_regular,
                    format!("Praise {world_name}!"),
                ));

                commands.spawn(label_bundle(
                    &font_assets.fira_sans_regular,
                    format!("Spawned of the finest of hash seeds, {world_seed}"),
                ));
            });
        });
}

fn despawn_loading_screen_system(
    mut commands: Commands,
    query: Query<Entity, With<LoadingScreen>>,
) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

use super::{
    input_text_bundle, label_bundle, make_btn, menu_node, menu_title_text_bundle,
    menu_wrapper_node, update_state_button, was_button_just_pressed, MenuState,
};
use crate::{
    plugin::{
        beef::FixedChunkWorld,
        chunk_loader::ChunkLoader,
        control::{pause::PauseState, PlyCamControl},
        game_gui::text_input::{TextInput, TextInputInactive, TextValue},
    },
    voxel::{world_noise::WorldNoiseSettings, BiomeTable},
    FontAssets,
};
use bevy::prelude::*;
use stable_hash::fast_stable_hash;

pub struct NewWorldMenuPlugin;

impl Plugin for NewWorldMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::NewWorldMenu),
            (spawn_new_world_menu_system, disable_text_inputs_system).chain(),
        )
        .add_systems(
            OnExit(MenuState::NewWorldMenu),
            despawn_new_world_menu_system,
        )
        .add_systems(
            Update,
            update_state_button::<ReturnToMainMenuButton, _>(
                MenuState::NewWorldMenu,
                MenuState::MainMenu,
            ),
        )
        .add_systems(
            Update,
            on_pressed_create_button_system.run_if(was_button_just_pressed::<CreateWorldButton>()),
        );
    }
}

fn disable_text_inputs_system(mut query: Query<&mut TextInputInactive, With<TextInput>>) {
    for mut inactive in query.iter_mut() {
        inactive.0 = true;
    }
}

#[derive(Component)]
struct NewWorldMenu;

#[derive(Component)]
struct CreateWorldButton;

#[derive(Component)]
struct WorldNameValueMarker;

#[derive(Component)]
struct WorldSeedValueMarker;

#[derive(Component)]
struct ReturnToMainMenuButton;

fn spawn_new_world_menu_system(mut commands: Commands, font_assets: Res<FontAssets>) {
    // Entire screen node
    commands
        .spawn((NewWorldMenu, menu_wrapper_node()))
        .with_children(|commands| {
            // New world menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Menu title text
                commands.spawn(menu_title_text_bundle(&font_assets, "New World"));

                // World name input label
                commands.spawn(label_bundle(&font_assets, "World name:"));

                // World name text input
                commands.spawn((WorldNameValueMarker, input_text_bundle(&font_assets)));

                // World seed input label
                commands.spawn(label_bundle(
                    &font_assets,
                    "Seed (just type something or nothing idk):",
                ));

                // Seed text input
                commands.spawn((WorldSeedValueMarker, input_text_bundle(&font_assets)));

                // Buttons
                make_btn(
                    commands,
                    &font_assets,
                    "Create!",
                    Some(CreateWorldButton),
                    true,
                );
                make_btn(
                    commands,
                    &font_assets,
                    "Back",
                    Some(ReturnToMainMenuButton),
                    true,
                );
            });
        });
}

fn despawn_new_world_menu_system(mut commands: Commands, query: Query<Entity, With<NewWorldMenu>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

fn on_pressed_create_button_system(
    mut commands: Commands,
    world_name_text: Query<&TextValue, With<WorldNameValueMarker>>,
    world_seed_text: Query<&TextValue, With<WorldSeedValueMarker>>,
    player_controller: Query<Entity, With<PlyCamControl>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut pause_state: ResMut<NextState<PauseState>>,
) {
    let Ok(name) = world_name_text
        .get_single()
        .map(|txt| txt.get().to_string())
    else {
        return;
    };

    // If seed is empty, 42069 is the default value! This must stay stable!
    let seed = fast_stable_hash(
        &world_seed_text
            .get_single()
            .map(|txt| txt.get())
            .unwrap_or("42069"),
    ) as u32;

    if let Ok(ply_entity) = player_controller.get_single() {
        commands.entity(ply_entity).insert(ChunkLoader::new(4));
        menu_state.set(MenuState::None);
        pause_state.set(PauseState::Playing);
    }

    info!("Creating world \"{name}\" with seed {seed}");

    commands.insert_resource(WorldNoiseSettings::new(seed, BiomeTable::new()));
    commands.insert_resource(FixedChunkWorld::new(name, seed));
}

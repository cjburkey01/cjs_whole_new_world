use super::{
    input_text_bundle, label_bundle, make_btn, menu_node, menu_title_text_bundle,
    menu_wrapper_node, update_state_button, was_button_just_pressed, ActiveMenuButton, MenuState,
    FULL_BACK_COVER_COLOR,
};
use crate::plugin::{
    asset::FontAssets,
    control::pause::PauseState,
    game_gui::text_input::TextValue,
    voxel_world::{world_info::WorldInfo, world_state::WorldState},
};
use bevy::prelude::*;
use stable_hash::fast_stable_hash;

pub struct NewWorldMenuPlugin;

impl Plugin for NewWorldMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::NewWorldMenu),
            spawn_new_world_menu_system,
        )
        .add_systems(
            OnExit(MenuState::NewWorldMenu),
            despawn_new_world_menu_system,
        )
        .add_systems(
            Update,
            (
                update_state_button::<ReturnToMainMenuButton, _>(
                    MenuState::NewWorldMenu,
                    MenuState::MainMenu,
                ),
                on_pressed_create_button_system
                    .run_if(was_button_just_pressed::<CreateWorldButton>()),
                toggle_new_world_button_system,
            ),
        );
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

fn toggle_new_world_button_system(
    mut commands: Commands,
    input: Query<&TextValue, With<WorldNameValueMarker>>,
    button: Query<(Entity, Option<&ActiveMenuButton>), With<CreateWorldButton>>,
) {
    if let (Ok(input), Ok((btn_entity, active))) = (
        input.get_single().map(|i| i.get().trim()),
        button.get_single(),
    ) {
        match (active.is_some(), input.is_empty()) {
            (true, true) => {
                commands.entity(btn_entity).remove::<ActiveMenuButton>();
            }
            (false, false) => {
                commands.entity(btn_entity).insert(ActiveMenuButton);
            }
            _ => {}
        }
    }
}

fn spawn_new_world_menu_system(mut commands: Commands, font_assets: Res<FontAssets>) {
    // Entire screen node
    commands
        .spawn((NewWorldMenu, menu_wrapper_node(FULL_BACK_COVER_COLOR)))
        .with_children(|commands| {
            // New world menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Menu title text
                commands.spawn(menu_title_text_bundle(&font_assets, "New World"));

                // World name input label
                commands.spawn(label_bundle(&font_assets.fira_sans_regular, "World name:"));

                // World name text input
                commands.spawn((
                    WorldNameValueMarker,
                    input_text_bundle(&font_assets.fira_sans_regular),
                ));

                // World seed input label
                commands.spawn(label_bundle(
                    &font_assets.fira_sans_regular,
                    "Seed (just type something or nothing idk):",
                ));

                // Seed text input
                commands.spawn((
                    WorldSeedValueMarker,
                    input_text_bundle(&font_assets.fira_code_regular),
                ));

                // Buttons
                make_btn(
                    commands,
                    &font_assets,
                    "Create!",
                    Some(CreateWorldButton),
                    false,
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
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut next_world_state: ResMut<NextState<WorldState>>,
    world_name_text: Query<&TextValue, With<WorldNameValueMarker>>,
    world_seed_text: Query<&TextValue, With<WorldSeedValueMarker>>,
) {
    let Ok(name) = world_name_text
        .get_single()
        .map(|txt| txt.get().trim().to_string())
    else {
        return;
    };

    // If seed is empty, 42069 is the default value! At some point make this
    // random!
    let seed = fast_stable_hash(
        &world_seed_text
            .get_single()
            .map(|txt| txt.get())
            .unwrap_or("42069"),
    ) as u32;

    commands.insert_resource(WorldInfo::new(name, seed));

    next_menu_state.set(MenuState::LoadingScreen);
    next_pause_state.set(PauseState::Playing);
    next_world_state.set(WorldState::LoadingStartArea);

    debug!("updating world state to loading");
}

use super::{
    label_bundle, make_btn, menu_node, menu_title_text_bundle, menu_wrapper_node,
    update_state_button, was_button_just_pressed, MenuState, DEFAULT_BACK_COVER_COLOR,
};
use crate::plugin::{asset::FontAssets, game_settings::GameSettings};
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
            (
                update_radius_text_system.run_if(resource_changed::<GameSettings>()),
                increment_radius_system(false)
                    .run_if(in_state(MenuState::PauseSettings))
                    .run_if(was_button_just_pressed::<RadiusDownButton>()),
                increment_radius_system(true)
                    .run_if(in_state(MenuState::PauseSettings))
                    .run_if(was_button_just_pressed::<RadiusUpButton>()),
                update_state_button::<BackButton, _>(MenuState::PauseSettings, MenuState::Paused),
            ),
        );
    }
}

#[derive(Component)]
struct PauseSettingsMenu;

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct RadiusUpButton;

#[derive(Component)]
struct RadiusDownButton;

#[derive(Component)]
struct RadiusText;

fn spawn_pause_settings_menu_system(
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    font_assets: Res<FontAssets>,
) {
    // Entire screen node
    commands
        .spawn((
            PauseSettingsMenu,
            menu_wrapper_node(DEFAULT_BACK_COVER_COLOR),
        ))
        .with_children(|commands| {
            // New world menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Menu title text
                commands.spawn(menu_title_text_bundle(&font_assets, "Settings"));

                // Load radius input label
                commands.spawn(label_bundle(
                    &font_assets.fira_sans_regular,
                    "Chunk loading radius:",
                ));

                // Load radius input
                commands
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Stretch,
                            column_gap: Val::Px(5.0),
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|commands| {
                        make_btn(commands, &font_assets, "+", Some(RadiusUpButton), true);

                        commands.spawn((
                            RadiusText,
                            TextBundle {
                                style: Style {
                                    align_self: AlignSelf::Center,
                                    justify_self: JustifySelf::Stretch,
                                    flex_grow: 0.0,
                                    ..default()
                                },
                                ..label_bundle(
                                    &font_assets.fira_code_regular,
                                    game_settings.load_radius.to_string(),
                                )
                            },
                        ));

                        make_btn(commands, &font_assets, "-", Some(RadiusDownButton), true);
                    });

                // Back button
                make_btn(commands, &font_assets, "Back", Some(BackButton), true);
            });
        });
}

fn update_radius_text_system(
    settings: Res<GameSettings>,
    mut text: Query<&mut Text, With<RadiusText>>,
) {
    if let Ok(mut text) = text.get_single_mut() {
        text.sections[0].value = settings.load_radius.to_string();
    }
}

fn increment_radius_system(increase: bool) -> impl Fn(ResMut<GameSettings>) {
    move |mut settings: ResMut<GameSettings>| match increase {
        true => settings.load_radius += 1,
        false => settings.load_radius = settings.load_radius.max(2) - 1, // Minimum of 1
    }
}

fn despawn_pause_settings_menu_system(
    mut commands: Commands,
    query: Query<Entity, With<PauseSettingsMenu>>,
) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

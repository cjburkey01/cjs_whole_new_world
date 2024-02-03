mod debug_ui;
mod main_menu;
mod new_world;
mod pause_menu;
mod pause_settings_menu;
pub mod text_input;

pub use debug_ui::*;
pub use main_menu::*;
pub use new_world::*;
pub use pause_menu::*;
pub use pause_settings_menu::*;

use crate::{plugin::control::pause::PauseState, AssetState, FontAssets};
use bevy::{ecs::schedule::SystemConfigs, prelude::*};
use text_input::{TextInputBundle, TextInputInactive};

const BORDER_COLOR_ACTIVE: Color = Color::VIOLET;
const BORDER_COLOR_INACTIVE: Color = Color::BLACK;

pub struct GameGuiPlugin;

impl Plugin for GameGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            .add_plugins((
                GameDebugUIPlugin,
                MainMenuPlugin,
                NewWorldMenuPlugin,
                PauseMenuPlugin,
                PauseSettingsMenuPlugin,
            ))
            .add_systems(
                OnEnter(AssetState::Ready),
                update_state_system(MenuState::MainMenu),
            )
            .add_systems(
                OnExit(MenuState::None),
                update_state_system(PauseState::Paused),
            )
            .add_systems(
                Update,
                update_menu_buttons_system.run_if(in_state(AssetState::Ready)),
            )
            .add_systems(Update, focus_text_input);
    }
}

fn update_state_system<S: States>(next_state: S) -> impl Fn(ResMut<NextState<S>>) {
    move |mut state: ResMut<NextState<S>>| state.set(next_state.clone())
}

fn update_state_button<BtnComp: Component, S: States>(
    current_state: S,
    next_state: S,
) -> SystemConfigs {
    update_state_system(next_state)
        .run_if(in_state(current_state))
        .run_if(was_button_just_pressed::<BtnComp>())
}

fn update_event_button<BtnComp: Component, E: Event + Clone>(event: E) -> SystemConfigs {
    (move |mut event_writer: EventWriter<E>| event_writer.send(event.clone()))
        .run_if(was_button_just_pressed::<BtnComp>())
}

#[derive(Default, States, PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum MenuState {
    #[default]
    None,
    MainMenu,
    NewWorldMenu,
    PauseSettings,
    Paused,
}

#[derive(Component)]
struct ActiveMenuButton;

#[allow(clippy::type_complexity)]
fn update_menu_buttons_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ActiveMenuButton>),
    >,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match interaction {
            Interaction::Hovered => *bg_color = Color::rgb(0.4, 0.8, 0.5).into(),
            Interaction::Pressed => *bg_color = Color::rgb(0.5, 0.6, 0.8).into(),
            Interaction::None => *bg_color = Color::WHITE.into(),
        }
    }
}

fn make_btn(
    commands: &mut ChildBuilder,
    font_assets: &FontAssets,
    text: impl Into<String>,
    bundle: Option<impl Bundle>,
    enabled: bool,
) {
    let mut e = commands.spawn(ButtonBundle {
        style: Style {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: Color::rgb(0.65, 0.65, 0.65).into(),
        ..default()
    });
    if let Some(bundle) = bundle {
        e.insert(bundle);
    }
    if enabled {
        e.insert(ActiveMenuButton);
    }
    e.with_children(|commands| {
        commands.spawn(TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font: Handle::clone(&font_assets.fira_sans_regular),
                    font_size: 30.0,
                    color: Color::BLACK,
                },
            ),
            ..default()
        });
    });
}

pub fn menu_wrapper_node() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: Color::BLACK.with_a(0.75).into(),
        ..default()
    }
}

pub fn menu_node() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(30.0),
            min_width: Val::Px(300.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Stretch,
            row_gap: Val::Px(20.0),
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        background_color: Color::BLACK.into(),
        ..default()
    }
}

pub fn menu_title_text_bundle(font_assets: &FontAssets, value: impl Into<String>) -> TextBundle {
    TextBundle {
        style: Style {
            align_self: AlignSelf::Center,
            ..default()
        },
        text: Text::from_section(
            value,
            TextStyle {
                font: Handle::clone(&font_assets.fira_sans_regular),
                font_size: 40.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    }
}

pub fn input_text_bundle(font: &Handle<Font>) -> (NodeBundle, TextInputBundle) {
    (
        NodeBundle {
            style: Style {
                align_self: AlignSelf::Stretch,
                align_content: AlignContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            border_color: BORDER_COLOR_INACTIVE.into(),
            background_color: Color::INDIGO.into(),
            ..default()
        },
        TextInputBundle::new(TextStyle {
            font: Handle::clone(font),
            font_size: 20.0,
            color: Color::WHITE,
        }),
    )
}

pub fn label_bundle(font: &Handle<Font>, text: impl Into<String>) -> TextBundle {
    TextBundle {
        style: Style {
            align_self: AlignSelf::FlexStart,
            justify_self: JustifySelf::End,
            ..default()
        },
        text: Text::from_section(
            text,
            TextStyle {
                font: Handle::clone(font),
                font_size: 18.0,
                color: Color::rgb(0.75, 0.75, 0.75),
            },
        ),
        ..default()
    }
}

pub fn was_button_just_pressed<BtnComp: Component>(
) -> impl Fn(Query<&Interaction, (Changed<Interaction>, With<BtnComp>)>) -> bool {
    |query: Query<&Interaction, (Changed<Interaction>, With<BtnComp>)>| {
        query
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    }
}

fn focus_text_input(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut text_input_query: Query<(Entity, &mut TextInputInactive, &mut BorderColor)>,
) {
    for (interaction_entity, interaction) in &query {
        if *interaction == Interaction::Pressed {
            for (entity, mut inactive, mut border_color) in &mut text_input_query {
                if entity == interaction_entity {
                    inactive.0 = false;
                    *border_color = BORDER_COLOR_ACTIVE.into();
                } else {
                    inactive.0 = true;
                    *border_color = BORDER_COLOR_INACTIVE.into();
                }
            }
        }
    }
}

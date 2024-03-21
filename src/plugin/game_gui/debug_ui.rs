use crate::{
    init_world_system,
    plugin::{
        asset::{AssetState, FontAssets},
        control::controller_2::{CharControl2, PlayerLookAtRes},
    },
    voxel::{ChunkPos, REGION_WIDTH},
};
use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::common_conditions::on_timer,
};
use std::time::Duration;

pub struct GameDebugUIPlugin;

impl Plugin for GameDebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AssetState::Ready),
            (init_world_system, init_ui_system),
        )
        .add_systems(
            Update,
            update_ui_system.run_if(on_timer(Duration::from_millis(100))),
        );
    }
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct LookAtGlobalPosText;

#[derive(Component)]
struct LookAtInChunkPosText;

#[derive(Component)]
struct LookAtChunkPosText;

#[derive(Component)]
struct LookAtVoxelText;

#[derive(Component)]
struct RequiredGenText;

#[derive(Component)]
struct RequiredRenderText;

#[derive(Component)]
struct RequiredDeleteText;

#[derive(Component)]
struct GeneratedText;

#[derive(Component)]
struct RenderedText;

#[derive(Component)]
struct DirtyText;

#[derive(Component)]
struct VisibleChunksText;

#[derive(Component)]
struct NonCulledChunksText;

#[derive(Component)]
struct PosText;

fn init_ui_system(mut commands: Commands, fonts: Res<FontAssets>) {
    let font_size = 20.0;
    let space_margin = 10.0;

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|cmds| {
            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                    TextSection::new(
                        " ",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                    TextSection::new(
                        "FPS",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                ]),
                FpsText,
            ));

            cmds.spawn(TextBundle {
                style: Style {
                    margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(space_margin), Val::ZERO),
                    ..default()
                },
                text: Text::from_section(
                    "Voxel stuff:",
                    TextStyle {
                        font: fonts.fira_sans_regular.clone(),
                        font_size,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Voxel Pos: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "frank",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                    TextSection::new(
                        " | Chunk Pos: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "was",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                    TextSection::new(
                        " | Region Pos: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "here!",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                PosText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Looking at voxel pos: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "your",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                LookAtGlobalPosText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Looking in chunk: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "mom",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                LookAtChunkPosText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Looking at voxel pos in chunk: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "was",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                LookAtInChunkPosText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Looking at voxel type: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "here",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                LookAtVoxelText,
            ));

            cmds.spawn(TextBundle {
                style: Style {
                    margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(space_margin), Val::ZERO),
                    ..default()
                },
                text: Text::from_section(
                    "Chunk stuff:",
                    TextStyle {
                        font: fonts.fira_sans_regular.clone(),
                        font_size,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Chunks needing generated: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                RequiredGenText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Chunks needing rendered: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                RequiredRenderText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Chunks needing deleted: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                RequiredDeleteText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Chunks generated: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                GeneratedText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Chunks rendered: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                RenderedText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Chunks needing update: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                DirtyText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Visible chunks: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                VisibleChunksText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Non-culled chunks: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                NonCulledChunksText,
            ));
        });
}

#[allow(clippy::type_complexity)]
fn update_ui_system(
    diagnostics: Res<DiagnosticsStore>,
    looking_at: Res<PlayerLookAtRes>,
    mut queries: ParamSet<(
        Query<&mut Text, With<FpsText>>,
        Query<&mut Text, With<PosText>>,
        Query<&mut Text, With<LookAtGlobalPosText>>,
        Query<&mut Text, With<LookAtChunkPosText>>,
        Query<&mut Text, With<LookAtInChunkPosText>>,
        Query<&mut Text, With<LookAtVoxelText>>,
        Query<&mut Text, With<NonCulledChunksText>>,
    )>,
    ply_query: Query<(&Transform, &ChunkPos), (Without<Text>, With<CharControl2>)>,
) {
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average);
    if let Some(fps) = fps {
        if let Ok(mut text) = queries.p0().get_single_mut() {
            text.sections[0].value = format!("{fps:.2}");
        }
    }

    if let (Ok(mut text), Ok((transform, chunk_pos))) =
        (queries.p1().get_single_mut(), ply_query.get_single())
    {
        text.sections[1].value = format!("{}", transform.translation.floor());
        text.sections[3].value = format!("{}", chunk_pos.0);
        text.sections[5].value = format!(
            "{}",
            chunk_pos
                .0
                .div_euclid(UVec3::splat(REGION_WIDTH).as_ivec3())
        );
    }

    if let Ok(mut text) = queries.p2().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.global_voxel_pos))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p3().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.chunk_pos))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p4().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.voxel_pos_in_chunk.pos()))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p5().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{:?}", pla.voxel))
            .unwrap_or_else(|| "Nothing".to_string());
    }
}

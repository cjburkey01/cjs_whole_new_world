use crate::{
    init_world_system,
    plugin::{
        beef::{
            DIAG_DELETE_REQUIRED, DIAG_DIRTY_CHUNKS, DIAG_GENERATED_CHUNKS, DIAG_GENERATE_REQUIRED,
            DIAG_NON_CULLED_CHUNKS, DIAG_RENDERED_CHUNKS, DIAG_RENDER_REQUIRED,
            DIAG_VISIBLE_CHUNKS,
        },
        controller_2::PlayerLookAtRes,
    },
    AssetState, FontAssets,
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
            (
                update_chunk_info_ui_system,
                update_ui_system.run_if(on_timer(Duration::from_millis(100))),
            ),
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

fn init_ui_system(mut commands: Commands, fonts: Res<FontAssets>) {
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
                            font_size: 26.0,
                            color: Color::YELLOW,
                        },
                    ),
                    TextSection::new(
                        " ",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
                            color: Color::YELLOW,
                        },
                    ),
                    TextSection::new(
                        "FPS",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                ]),
                FpsText,
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Looking at voxel pos: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
                            color: Color::YELLOW,
                        },
                    ),
                ]),
                LookAtVoxelText,
            ));

            cmds.spawn(TextBundle::from_section(
                "Chunk stuff:",
                TextStyle {
                    font: fonts.fira_sans_regular.clone(),
                    font_size: 26.0,
                    color: Color::WHITE,
                },
            ));

            cmds.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Chunks needing generated: ",
                        TextStyle {
                            font: fonts.fira_sans_regular.clone(),
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
                            font_size: 26.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font: fonts.fira_code_bold.clone(),
                            font_size: 26.0,
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
        Query<&mut Text, With<LookAtGlobalPosText>>,
        Query<&mut Text, With<LookAtChunkPosText>>,
        Query<&mut Text, With<LookAtInChunkPosText>>,
        Query<&mut Text, With<LookAtVoxelText>>,
        Query<&mut Text, With<NonCulledChunksText>>,
    )>,
) {
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average);
    if let Some(fps) = fps {
        if let Ok(mut text) = queries.p0().get_single_mut() {
            text.sections[0].value = format!("{fps:.2}");
        }
    }

    if let Ok(mut text) = queries.p1().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.global_voxel_pos))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p2().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.chunk_pos))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p3().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.voxel_pos_in_chunk.pos()))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p4().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{:?}", pla.voxel))
            .unwrap_or_else(|| "Nothing".to_string());
    }
}

#[allow(clippy::type_complexity)]
fn update_chunk_info_ui_system(
    diagnostics: Res<DiagnosticsStore>,
    mut queries: ParamSet<(
        Query<&mut Text, With<RequiredGenText>>,
        Query<&mut Text, With<RequiredRenderText>>,
        Query<&mut Text, With<RequiredDeleteText>>,
        Query<&mut Text, With<GeneratedText>>,
        Query<&mut Text, With<RenderedText>>,
        Query<&mut Text, With<DirtyText>>,
        Query<&mut Text, With<VisibleChunksText>>,
        Query<&mut Text, With<NonCulledChunksText>>,
    )>,
) {
    let required_gen_count = diagnostics
        .get(DIAG_GENERATE_REQUIRED)
        .and_then(Diagnostic::value);
    if let Some(required_gen_count) = required_gen_count {
        if let Ok(mut text) = queries.p0().get_single_mut() {
            text.sections[1].value = format!("{}", required_gen_count as u32);
        }
    }

    let required_render_count = diagnostics
        .get(DIAG_RENDER_REQUIRED)
        .and_then(Diagnostic::value);
    if let Some(required_render_count) = required_render_count {
        if let Ok(mut text) = queries.p1().get_single_mut() {
            text.sections[1].value = format!("{}", required_render_count as u32);
        }
    }

    let required_delete_count = diagnostics
        .get(DIAG_DELETE_REQUIRED)
        .and_then(Diagnostic::value);
    if let Some(required_delete_count) = required_delete_count {
        if let Ok(mut text) = queries.p2().get_single_mut() {
            text.sections[1].value = format!("{}", required_delete_count as u32);
        }
    }

    let generated_count = diagnostics
        .get(DIAG_GENERATED_CHUNKS)
        .and_then(Diagnostic::value);
    if let Some(generated_count) = generated_count {
        if let Ok(mut text) = queries.p3().get_single_mut() {
            text.sections[1].value = format!("{}", generated_count as u32);
        }
    }

    let rendered_count = diagnostics
        .get(DIAG_RENDERED_CHUNKS)
        .and_then(Diagnostic::value);
    if let Some(rendered_count) = rendered_count {
        if let Ok(mut text) = queries.p4().get_single_mut() {
            text.sections[1].value = format!("{}", rendered_count as u32);
        }
    }

    let dirty_count = diagnostics
        .get(DIAG_DIRTY_CHUNKS)
        .and_then(Diagnostic::value);
    if let Some(dirty_count) = dirty_count {
        if let Ok(mut text) = queries.p5().get_single_mut() {
            text.sections[1].value = format!("{}", dirty_count as u32);
        }
    }

    let visible_count = diagnostics
        .get(DIAG_VISIBLE_CHUNKS)
        .and_then(Diagnostic::value);
    if let Some(visible_count) = visible_count {
        if let Ok(mut text) = queries.p6().get_single_mut() {
            text.sections[1].value = format!("{}", visible_count as u32);
        }
    }

    let non_culled_count = diagnostics
        .get(DIAG_NON_CULLED_CHUNKS)
        .and_then(Diagnostic::value);
    if let Some(non_culled_count) = non_culled_count {
        if let Ok(mut text) = queries.p7().get_single_mut() {
            text.sections[1].value = format!("{}", non_culled_count as u32);
        }
    }
}

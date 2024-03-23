use super::LodPos;
use bevy::{prelude::*, tasks::Task};

#[derive(Debug, Component)]
pub struct LodRenderTask(pub LodPos, pub Task<LodRenderTaskReturn>);

#[derive(Debug)]
pub struct LodRenderTaskReturn;

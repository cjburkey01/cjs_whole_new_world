use bevy::{prelude::*, tasks::Task};

#[derive(Debug, Component)]
pub struct LodRenderTask(pub Task<LodRenderTaskReturn>);

#[derive(Debug)]
pub struct LodRenderTaskReturn;

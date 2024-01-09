use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "wgpu=warn,naga=warn,bevy_render=info,bevy_app::plugin_group=info".to_string(),
            level: Level::DEBUG,
        }))
        .run();
}

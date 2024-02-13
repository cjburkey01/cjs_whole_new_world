use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection, loading_state::LoadingState, prelude::*,
};

pub struct CwnwAssetPlugin;

impl Plugin for CwnwAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AssetState>().add_loading_state(
            LoadingState::new(AssetState::Loading)
                .continue_to_state(AssetState::Ready)
                .load_collection::<FontAssets>(),
        );
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum AssetState {
    #[default]
    Loading,
    Ready,
}

#[allow(unused)]
#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraCode6.2/FiraCode-Bold.ttf")]
    pub fira_code_bold: Handle<Font>,
    #[asset(path = "fonts/FiraCode6.2/FiraCode-Regular.ttf")]
    pub fira_code_regular: Handle<Font>,

    #[asset(path = "fonts/FiraSans/FiraSans-Bold.ttf")]
    pub fira_sans_bold: Handle<Font>,
    #[asset(path = "fonts/FiraSans/FiraSans-Regular.ttf")]
    pub fira_sans_regular: Handle<Font>,
}

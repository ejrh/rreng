use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Handle};
use bevy::prelude::{Font, Res, ResMut, Resource};

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Theme>()
            .add_systems(Startup, setup_theme);
    }
}

#[derive(Default, Resource)]
pub struct Theme {
    pub font: Handle<Font>,
}

pub fn setup_theme(
    mut theme: ResMut<Theme>,
    asset_server: Res<AssetServer>,
) {
    theme.font = asset_server.load("fonts/FiraMono-Medium.ttf");
}

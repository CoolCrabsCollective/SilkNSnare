use crate::flying_insect::flying_insect::FlyingInsectPlugin;
use crate::flying_obstacle::flying_obstacle::FlyingObstaclePlugin;
use crate::game::GamePlugin;
use crate::health::HealthPlugin;
use crate::mesh_loader::MeshLoaderPlugin;
use crate::spider::SpiderPlugin;
use crate::title_screen::TitleScreenPlugin;
use crate::ui::progress_bar::ProgressBarPlugin;
use bevy::app::{App, PluginGroup};
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, FilterMode};
use bevy::render::texture::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use bevy::render::RenderPlugin;
use bevy::DefaultPlugins;
use pumpkin::PumpkinPlugin;
use tree::TreePlugin;

mod config;
mod game;
mod mesh_loader;
mod pumpkin;
mod spider;
mod tree;
mod web;

mod flying_insect;
mod flying_obstacle;
mod health;
mod skybox;
mod title_screen;
mod ui;

fn main() {
    let mut app = App::new();

    let default_sampler = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::from(AddressMode::Repeat),
        address_mode_v: ImageAddressMode::from(AddressMode::Repeat),
        address_mode_w: ImageAddressMode::from(AddressMode::Repeat),
        mag_filter: ImageFilterMode::from(FilterMode::Linear),
        min_filter: ImageFilterMode::from(FilterMode::Linear),
        mipmap_filter: ImageFilterMode::from(FilterMode::Linear),
        ..default()
    };
    if cfg!(target_arch = "wasm32") {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        title: "Silk & Snare".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin { default_sampler }),
        );
    } else {
        app.add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: Default::default(),
                    synchronous_pipeline_compilation: false,
                })
                .set(ImagePlugin { default_sampler }),
        );
    }

    app.add_plugins(TitleScreenPlugin);
    app.add_plugins(GamePlugin);
    app.add_plugins(MeshLoaderPlugin);
    app.add_plugins(TreePlugin);
    app.add_plugins(PumpkinPlugin);
    app.add_plugins(SpiderPlugin);
    app.add_plugins(FlyingInsectPlugin);
    app.add_plugins(HealthPlugin);
    app.add_plugins(FlyingObstaclePlugin);
    app.add_plugins(ProgressBarPlugin);

    app.run();
}

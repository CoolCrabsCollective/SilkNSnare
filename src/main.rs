use crate::game::GamePlugin;
use crate::spider::SpiderPlugin;
use bevy::app::{App, PluginGroup};
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, FilterMode};
use bevy::render::texture::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use bevy::render::RenderPlugin;
use bevy::DefaultPlugins;
use tree::TreePlugin;

mod config;
mod game;
mod level_loaders;
mod spider;
mod tree;
mod web;

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
                        ..default()
                    }),
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

    app.add_plugins(GamePlugin);
    app.add_plugins(TreePlugin);
    app.add_plugins(SpiderPlugin);

    app.run();
}

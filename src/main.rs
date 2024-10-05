
use bevy::prelude::*;
use bevy::app::{App, PluginGroup};
use bevy::{window, DefaultPlugins};
use bevy::render::render_resource::{AddressMode, FilterMode, SamplerDescriptor};
use bevy::render::RenderPlugin;
use bevy::render::settings::{WgpuFeatures, WgpuSettings};
use bevy::render::texture::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use crate::game::GamePlugin;

mod game;
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
                .set(RenderPlugin { render_creation: Default::default(), synchronous_pipeline_compilation: false })
                .set(ImagePlugin { default_sampler }),
        );
    }

    app.add_plugins(GamePlugin);

    app.run();
}

use bevy::asset::Handle;
use bevy::prelude::{Image, Resource};
use bevy::render::texture::CompressedImageFormats;

pub const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[
    ("moon.png", CompressedImageFormats::NONE),
    // ("moon_purple.png", CompressedImageFormats::NONE),
];

#[derive(Resource)]
pub struct Cubemap {
    pub(crate) is_loaded: bool,
    pub(crate) index: usize,
    pub(crate) image_handle: Handle<Image>,
}

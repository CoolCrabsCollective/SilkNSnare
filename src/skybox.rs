use bevy::asset::Handle;
use bevy::prelude::{Image, Resource};
use bevy::render::texture::CompressedImageFormats;

const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[("skybox.png", CompressedImageFormats::NONE)];

#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}

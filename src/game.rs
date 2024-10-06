use crate::skybox::{Cubemap, CUBEMAPS};
use crate::web::WebSimulationPlugin;
use bevy::asset::LoadState;
use bevy::audio::PlaybackMode::Loop;
use bevy::audio::Volume;
use bevy::color::palettes::basic::RED;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::core_pipeline::Skybox;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::Projection::Perspective;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::CompressedImageFormats;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (cycle_cubemap_asset, asset_loaded.after(cycle_cubemap_asset)),
        );
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default().disabled(),
            WebSimulationPlugin,
        ))
        .add_systems(Update, debug_render_toggle)
        .insert_resource(ClearColor(Color::srgb(0.3, 0.6, 0.9)))
        .insert_resource(DirectionalLightShadowMap { size: 4096 });
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    //load_level("map.glb#Scene0", &mut commands, &asset_server);

    commands.spawn(AudioBundle {
        source: asset_server.load("web_pressure.ogg"),
        settings: PlaybackSettings {
            mode: Loop,
            volume: Volume::new(1.0f32),
            ..default()
        },
        ..default()
    });

    let orange_light_color = Color::srgb(0.97, 0.75, 0.6);

    commands.insert_resource(AmbientLight {
        // color: Color::WHITE,
        // color: Color::srgb(1.0, 0.6, 0.0),
        // color: Color::srgb(1.0, 0.77, 0.59),
        color: orange_light_color,
        brightness: 100.0,
    });

    commands.spawn(PointLightBundle {
        // transform: Transform::from_xyz(5.0, 8.0, 2.0),
        transform: Transform::from_xyz(-2.0, 1.0, 3.0),
        point_light: PointLight {
            intensity: 100_000.0,
            color: orange_light_color,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_MOON_NIGHT,
            shadows_enabled: true,
            color: orange_light_color,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });

    let skybox_handle = asset_server.load(CUBEMAPS[0].0);
    // camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 70,
                ..default()
            },
            transform: get_initial_camera_transform(),
            projection: Perspective(PerspectiveProjection {
                fov: 60.0f32.to_radians(),
                ..default()
            }),

            ..default()
        },
        Skybox {
            image: skybox_handle.clone(),
            brightness: 1000.0,
        },
    ));

    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });

    commands.spawn((SceneBundle {
        scene: asset_server.load("outdoor_scene.glb#Scene0"),
        transform: Transform {
            translation: Vec3::new(-4.0, -2.0, -10.0),
            rotation: Quat::from_rotation_y(PI),
            scale: Vec3::ONE,
        },
        global_transform: Default::default(),
        visibility: Default::default(),
        inherited_visibility: Default::default(),
        view_visibility: Default::default(),
    },));
}

pub fn get_initial_camera_transform() -> Transform {
    Transform::from_xyz(0.0, 0.0, 5.0).with_rotation(Quat::from_axis_angle(Vec3::Y, 0.0))
}

fn get_initial_sun_transform() -> Transform {
    let res = get_initial_camera_transform();
    res.with_translation(res.translation + Vec3::new(0.0, 25.0, 1.75))
        .looking_at(Vec3::ZERO, Vec3::Y)
}

fn debug_render_toggle(mut context: ResMut<DebugRenderContext>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_released(KeyCode::F12) {
        context.enabled = !context.enabled;
    }
}

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle) == LoadState::Loaded {
        info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}
const CUBEMAP_SWAP_DELAY: f32 = 3.0;
fn cycle_cubemap_asset(
    time: Res<Time>,
    mut next_swap: Local<f32>,
    mut cubemap: ResMut<Cubemap>,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {
    let now = time.elapsed_seconds();
    if *next_swap == 0.0 {
        *next_swap = now + CUBEMAP_SWAP_DELAY;
        return;
    } else if now < *next_swap {
        return;
    }
    *next_swap += CUBEMAP_SWAP_DELAY;

    let supported_compressed_formats =
        CompressedImageFormats::from_features(render_device.features());

    let mut new_index = cubemap.index;
    for _ in 0..CUBEMAPS.len() {
        new_index = (new_index + 1) % CUBEMAPS.len();
        if supported_compressed_formats.contains(CUBEMAPS[new_index].1) {
            break;
        }
        info!(
            "Skipping format which is not supported by current hardware: {:?}",
            CUBEMAPS[new_index]
        );
    }

    // Skip swapping to the same texture. Useful for when ktx2, zstd, or compressed texture support
    // is missing
    if new_index == cubemap.index {
        return;
    }

    cubemap.index = new_index;
    cubemap.image_handle = asset_server.load(CUBEMAPS[cubemap.index].0);
    cubemap.is_loaded = false;
}

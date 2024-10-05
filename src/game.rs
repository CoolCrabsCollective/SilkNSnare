use crate::web::WebSimulationPlugin;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::Projection::Perspective;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        //app.add_systems(OnEnter(GameState::FightingInArena), reset_camera);
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default().disabled(),
            WebSimulationPlugin,
        ))
        //.add_systems(Update, debug_render_toggle)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(ClearColor(Color::srgb(0.3, 0.6, 0.9)))
        .insert_resource(DirectionalLightShadowMap { size: 4096 });

        //app.add_plugins(DebugCameraControllerPlugin);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut _asset_server: ResMut<AssetServer>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    //load_level("map.glb#Scene0", &mut commands, &asset_server);
    /*
    commands.spawn(AudioBundle {
        source: asset_server.load("song.ogg"),
        settings: PlaybackSettings {
            mode: Loop,
            volume: Relative(VolumeLevel::new(0.1f32)),
            ..default()
        },
        ..default()
    });*/

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
    });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
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

    // camera
    commands.spawn(Camera3dBundle {
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
    });
    /*.insert(PostProcessSettings {
        time: 0.0,
        enable: 1.0,
        ..default()
    })*/
    //.insert(HolyCam);
}

pub fn get_initial_camera_transform() -> Transform {
    Transform::from_xyz(0.0, 0.0, 5.0).with_rotation(Quat::from_axis_angle(Vec3::Y, 0.0))
}

fn get_initial_sun_transform() -> Transform {
    let res = get_initial_camera_transform();
    res.with_translation(res.translation + Vec3::new(0.0, 25.0, 1.75))
        .looking_at(Vec3::ZERO, Vec3::Y)
}

/*
fn reset_camera(
    mut camera_query: Query<(&mut Transform, &mut Projection)>,
    mut color: ResMut<ClearColor>,
) {
    let mut a = camera_query.single_mut();
    (*a.0) = get_camera_position();

    if let Perspective(pers_proj) = a.1.as_mut() {
        pers_proj.fov = 10.0f32.to_radians();
    }
    color.0 = Color::rgb(0.3, 0.6, 0.9);
}*/

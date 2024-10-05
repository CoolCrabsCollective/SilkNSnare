use std::f32::consts::PI;

use bevy::math::vec3;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::Projection::Perspective;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;


pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        //app.add_systems(OnEnter(GameState::FightingInArena), reset_camera);
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default().disabled(),
        ))
            //.add_systems(Update, debug_render_toggle)
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1.0 / 5.0f32,
            })
            .insert_resource(ClearColor(Color::rgb(0.3, 0.6, 0.9)))
            .insert_resource(DirectionalLightShadowMap { size: 4096 });

        //app.add_plugins(DebugCameraControllerPlugin);
    }
}


/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 50000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-0.75 * PI)),
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            maximum_distance: 100.0,
            ..default()
        }
            .into(),
        ..default()
    });
    // camera
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                order: 70,
                ..default()
            },
            transform: get_camera_position(),
            projection: Perspective(PerspectiveProjection {
                fov: 10.0f32.to_radians(),
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

fn get_camera_position() -> Transform {
    Transform::from_xyz(0.0, 60.0, 28.0).looking_at(vec3(0.0, 0.0, 0.0), Vec3::Y)
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
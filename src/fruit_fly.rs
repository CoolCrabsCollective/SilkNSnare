use bevy::{prelude::*};
use std::f32::consts::FRAC_1_SQRT_2;
use std::sync::Arc;
use rand::Rng;


pub struct FruitFlyPlugin;

struct BezierCurve {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3
}

impl BezierCurve {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        BezierCurve {
            p0,
            p1,
            p2,
            p3
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        if t < 0.0 || t>1.0 {
           panic!("CRINGE INVALID t USED FOR FLY BEZIER CURVE");
        }

        f32::powf(1.0-t, 3.0)*self.p0 + 3.0*(1.0-t)*(1.0-t)*t*self.p1 + 3.0*(1.0-t)*t*t*self.p2 + t*t*t*self.p3
    }
}

fn generate_bezier_handles(p0: Vec3, p3: Vec3) -> (Vec3, Vec3) {
    let mut rng = rand::thread_rng();

    let p1 = Vec3::new(
        rng.gen_range(p0.x..p3.x),
        rng.gen_range(p0.y..p3.y),
        rng.gen_range(p0.z..p3.z)
    );

    let p2 = Vec3::new(
        rng.gen_range(p0.x..p3.x),
        rng.gen_range(p0.y..p3.y),
        rng.gen_range(p0.z..p3.z)
    );

    (p1, p2)
}

#[derive(Component)]
struct FruitFly {
    path: BezierCurve,
    speed: f32,
    progress: f32
}

impl FruitFly {
    pub fn new(curve: BezierCurve, speed: f32) -> Self {

        FruitFly {
            path: curve,
            speed,
            progress: 0.0
        }
    }
}

impl Plugin for FruitFlyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_fruit_fly);
        app.add_systems(Update, move_fruit_fly);
    }
}

fn spawn_fruit_fly(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let start_pos = Vec3::new(-2.5, 0.0, 0.0);
    let end_pos = Vec3::new(50.0, 0.0, 0.0);

    // let handles = generate_bezier_handles(start_pos, end_pos);
    let handles = (Vec3::new(0., 0., 0.), Vec3::new(0., 0., 0.));

    commands.spawn((
        FruitFly::new(BezierCurve::new(start_pos, handles.0, handles.1, end_pos), 0.2),
        SceneBundle {
            scene: asset_server.load("fruit_fly.glb#Scene0"),
            transform: Transform {
                translation: start_pos,
                rotation: Quat::from_xyzw(0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2),
                scale: Vec3::new(0.02, 0.02, 0.02),
            },
            global_transform: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
        },
    ));
}

fn move_fruit_fly(
    mut fly_query: Query<(&mut FruitFly, &mut Transform, Entity, &Handle<Mesh>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut fly, mut transform, entity, mesh_handle) in &mut fly_query {
        fly.progress += time.delta_seconds() * fly.speed;

        if fly.progress > 1.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation = fly.path.at(fly.progress);
        }
    }
}

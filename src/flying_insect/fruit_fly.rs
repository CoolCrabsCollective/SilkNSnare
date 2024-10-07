use crate::config::COLLISION_GROUP_ENEMIES;
use crate::flying_insect::flying_insect::{
    fly_timer_value, BezierCurve, FlyingInsect, FruitFlySpawnTimer,
};
use crate::tree::GameStart;
use crate::ui::progress_bar::CookingInsect;
use bevy::prelude::*;
use bevy_health_bar3d::configuration::BarHeight;
use bevy_health_bar3d::prelude::BarSettings;
use bevy_rapier3d::geometry::{CollisionGroups, Group};
use bevy_rapier3d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider};
use rand::Rng;
use std::time::Duration;

pub const DAVID_DEBUG: bool = false;

#[derive(Component)]
struct FruitFly;

#[derive(Resource)]
pub struct Animation {
    pub animation_list: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}

pub fn spawn_fruit_fly(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    time: Res<Time>,
    mut ff_spawn_timer: ResMut<FruitFlySpawnTimer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut animation_res: ResMut<Animation>,
    start_query: Query<&GameStart>,
) {
    ff_spawn_timer.timer.tick(time.delta());
    if ff_spawn_timer.timer.just_finished() {
        let game_start = start_query.single();
        ff_spawn_timer.timer = Timer::new(
            Duration::from_secs_f32(fly_timer_value(
                time.elapsed_seconds() - game_start.game_start,
            )),
            TimerMode::Repeating,
        );
        let mut rng = rand::thread_rng();
        let x_begin = rng.gen_range(-4.0..0.0);
        let x_end = rng.gen_range(-3.0..-1.0);
        let y_begin = rng.gen_range(0.0..1.0);
        let y_end = rng.gen_range(0.0..1.0);

        let start_pos = Vec3::new(x_begin, y_begin, -2.0);
        let end_pos = Vec3::new(x_end, y_end, 3.5);

        let david_debug_pos = Vec2::new(-2.0, 0.1);

        let mut graph = AnimationGraph::new();
        let animations: Vec<_> = graph
            .add_clips(
                [
                    GltfAssetLabel::Animation(0).from_asset("fruit_fly.glb"),
                    GltfAssetLabel::Animation(1).from_asset("fruit_fly.glb"),
                ]
                .into_iter()
                .map(|path| asset_server.load(path)),
                1.0,
                graph.root,
            )
            .collect();

        let graph = graphs.add(graph);
        animation_res.animation_list = animations;
        animation_res.graph = graph;

        commands
            .spawn((
                FlyingInsect::new(
                    0.1,
                    0.01,
                    if DAVID_DEBUG {
                        BezierCurve::new(
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, -1.0),
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, -1.0),
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, 3.0),
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, 3.0),
                        )
                    } else {
                        BezierCurve::random_from_endpoints(start_pos, end_pos)
                    },
                ),
                FruitFly,
                SceneBundle {
                    scene: asset_server.load("fruit_fly.glb#Scene0"),
                    transform: Transform {
                        translation: start_pos,
                        rotation: Quat::default(),
                        scale: Vec3::new(0.02, 0.02, 0.02) * 1.5,
                    },
                    global_transform: Default::default(),
                    visibility: Default::default(),
                    inherited_visibility: Default::default(),
                    view_visibility: Default::default(),
                },
                Collider::capsule_y(1.0, 1.0),
                BarSettings::<CookingInsect> {
                    offset: 2.0,
                    width: 3.0,
                    height: BarHeight::Static(0.5),
                    ..default()
                },
            ))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(CollisionGroups {
                memberships: COLLISION_GROUP_ENEMIES,
                filters: Group::ALL,
            });
    }
}

pub fn fly_hentai_anime_setup(
    mut commands: Commands,
    animations: Res<Animation>,
    mut player_query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in player_query.iter_mut() {
        player.play(animations.animation_list[0]).repeat();
        player.play(animations.animation_list[1]).repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(player.clone());
    }
}

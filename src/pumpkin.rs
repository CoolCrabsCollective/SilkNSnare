use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

pub struct PumpkinPlugin;

impl Plugin for PumpkinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_pumpkin_glow);
    }
}

#[derive(Component)]
pub struct Pumpkin;

pub fn update_pumpkin_glow(
    mut pumpkin_lights_query: Query<(&mut PointLight, &Pumpkin)>,
    time: Res<Time>,
) {
    for (i, (mut pumpkin_light, _)) in pumpkin_lights_query.iter_mut().enumerate() {
        let get_noise = |t| Perlin::new(i as u32).get([t]) as f32 * 0.5 + 0.5;

        let slow_noise = get_noise(0.5 * time.elapsed_seconds_f64());
        let fast_noise = get_noise(4.0 * time.elapsed_seconds_f64());

        pumpkin_light.intensity = 250_000.0 + 500_000.0 * slow_noise + 200_000.0 * fast_noise;
    }
}

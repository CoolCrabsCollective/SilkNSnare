use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

pub struct PumpkinPlugin;

impl Plugin for PumpkinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_pumpkin_glow);
    }
}

pub const PUMPKIN_LIGHT_INTENSITY: f32 = 250_000.0;

#[derive(Component)]
pub struct Pumpkin;

pub fn update_pumpkin_glow(
    mut pumpkin_lights_query: Query<(&mut PointLight, &Pumpkin)>,
    time: Res<Time>,
) {
    for (i, (mut pumpkin_light, _)) in pumpkin_lights_query.iter_mut().enumerate() {
        let get_noise = |t| Perlin::new(i as u32).get([t]) as f32 * 0.5 + 0.5;

        let slow_noise = get_noise(0.5 * time.elapsed_seconds_f64());
        let fast_noise = get_noise(3.0 * time.elapsed_seconds_f64());

        pumpkin_light.intensity =
            PUMPKIN_LIGHT_INTENSITY * (0.333 + 0.6777 * slow_noise + 0.3333 * fast_noise);
    }
}

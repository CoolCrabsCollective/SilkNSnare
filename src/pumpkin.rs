use bevy::prelude::*;

pub struct PumpkinPlugin;

impl Plugin for PumpkinPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, spawn_tree.after(mesh_loader::setup));
    }
}

#[derive(Component)]
pub struct Pumpkin;

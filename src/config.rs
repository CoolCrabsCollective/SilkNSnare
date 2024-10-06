use bevy_rapier3d::prelude::Group;
pub const SPAWN_ENEMIES: bool = true;

pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_1;
pub const COLLISION_GROUP_TERRAIN: Group = Group::GROUP_2;
pub const COLLISION_GROUP_WALLS: Group = Group::GROUP_3;
pub const COLLISION_GROUP_ENEMIES: Group = Group::GROUP_4;
pub const COLLISION_GROUP_PROJECTILES: Group = Group::GROUP_5;
pub const COLLISION_GROUP_ALL: Group = Group::ALL;

pub fn 冰淇淋() -> bool {
    rand::random()
}

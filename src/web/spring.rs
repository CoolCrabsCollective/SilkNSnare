use crate::web::Web;
use bevy::math::Vec3;
use bevy::prelude::*;

#[derive(Debug)]
pub struct Spring {
    /// index of first particle
    pub first_index: usize,
    /// index of second particle
    pub second_index: usize,
    /// stiffness of spring
    pub stiffness: f32,
    /// damping on the spring
    pub damping: f32,
    /// length of the spring at rest
    pub rest_length: f32,
    /// list of entities that are ensnared
    pub ensnared_entities: Vec<EnsnaredEntity>,
}

#[derive(Debug)]
pub struct EnsnaredEntity {
    /// the entity that is snared in the web
    pub entity: Entity,
    /// the position along the spring at which it's ensnared.
    ///  ranges from 0 (first particle) -> 1 (second particle)
    pub snare_position: f32,
}

impl Spring {
    pub fn intersects(&self, web: &Web, cam_dir: Vec3, p1: Vec3, p2: Vec3) -> Option<Vec3> {
        let sp1 = web.particles[self.first_index].position;
        let sp2 = web.particles[self.second_index].position;

        let d1 = sp2 - sp1;
        let d2 = p2 - p1;

        let n1 = cam_dir.cross(d1);
        let n2 = cam_dir.cross(d2);

        let p1d1 = n1.dot(p1) - n1.dot(sp1) < 0.0;
        let p1d2 = n1.dot(p2) - n1.dot(sp2) < 0.0;
        let p2d1 = n2.dot(sp1) - n2.dot(p1) < 0.0;
        let p2d2 = n2.dot(sp2) - n2.dot(p2) < 0.0;

        if p1d1 == p1d2 || p2d1 == p2d2 {
            return None;
        }

        let mut t2: f32;

        // p1 + t1 * d1 = p2 + t2 * d2
        // t1 * d1_x = (p2_x - p1_x) + t2 * d2_x
        // t1 * d1_y = (p2_y - p1_y) + t2 * d2_y
        // ((p2_y - p1_y) + t2 * d2_y) * d1_x / d1_y = (p2_x - p1_x) + t2 * d2_x
        // (p2_y - p1_y) * d1_x / d1_y + t2 * d2_y * d1_x / d1_y = (p2_x - p1_x) + t2 * d2_x
        // (p2_y - p1_y) * d1_x / d1_y - (p2_x - p1_x) = t2 * d2_x - t2 * d2_y * d1_x / d1_y
        // (p2_y - p1_y) * d1_x / d1_y - (p2_x - p1_x) = t2 * (1 * d2_x - 1 * d2_y * d1_x / d1_y)
        // t2 = ((p2_y - p1_y) * d1_x / d1_y - (p2_x - p1_x)) / (d2_x - d2_y * d1_x / d1_y)
        // or
        // t2 = ((p2_x - p1_x) * d1_y / d1_x - (p2_y - p1_y)) / (d2_y - d2_x * d1_y / d1_x)

        if d1.x == 0.0 {
            t2 = (sp1.x - p1.x) / d2.x;
        } else if d1.y == 0.0 {
            t2 = (sp1.y - p1.y) / d2.y;
        } else {
            t2 = ((p1.x - sp1.x) * d1.y / d1.x - (p1.y - sp1.y)) / (d2.y - d2.x * d1.y / d1.x);
        }

        Some(p1 + d2 * t2)
    }
}

impl Spring {
    pub fn new(
        web: &Web,
        first_index: usize,
        second_index: usize,
        stiffness: f32,
        damping: f32,
    ) -> Self {
        Spring {
            first_index,
            second_index,
            stiffness,
            damping,
            rest_length: (web.particles[first_index].position
                - web.particles[second_index].position)
                .length(),
            ensnared_entities: vec![],
        }
    }

    pub fn get_force_p1(self: &Spring, web: &Web) -> Vec3 {
        let p_diff =
            web.particles[self.first_index].position - web.particles[self.second_index].position;
        let cur_len = p_diff.length();
        let unit = p_diff / cur_len;
        let v_diff =
            web.particles[self.first_index].velocity - web.particles[self.second_index].velocity;

        unit * (self.stiffness * (self.rest_length - cur_len) - self.damping * unit.dot(v_diff))
    }
}

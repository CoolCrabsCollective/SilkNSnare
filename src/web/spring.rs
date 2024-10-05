use crate::web::Web;
use bevy::math::Vec3;

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
}

impl Spring {
    pub fn intersects(&self, web: &Web, cam_dir: Vec3, p1: Vec3, p2: Vec3) -> Option<Vec3> {
        let sp1 = web.particles[self.first_index].position;
        let sp2 = web.particles[self.second_index].position;

        let n1 = cam_dir.cross(sp2 - sp1);
        let n2 = cam_dir.cross(p2 - p1);

        let p1d1 = n1.dot(p1) - n1.dot(sp1) < 0.0;
        let p1d2 = n1.dot(p2) - n1.dot(sp2) < 0.0;
        let p2d1 = n2.dot(sp1) - n2.dot(p1) < 0.0;
        let p2d2 = n2.dot(sp2) - n2.dot(p2) < 0.0;

        if p1d1 == p1d2 || p2d1 == p2d2 {
            return None
        }

        Some((sp1 + sp2) / 2.0)
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

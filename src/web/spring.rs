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
    pub fn intersects(&self, p0: Vec3, p1: Vec3) -> bool {
        false
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

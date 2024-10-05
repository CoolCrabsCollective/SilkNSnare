use bevy::math::Vec3;
use crate::web::Web;

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
    pub rest_length: f32
}

impl Spring {
    pub fn new(web: &Web,
               first_index: usize, second_index: usize,
               stiffness: f32,
               damping: f32) -> Self {
        Spring {
            first_index,
            second_index,
            stiffness,
            damping,
            rest_length: (web.particles[first_index].position - web.particles[second_index].position).length(),
        }
    }

    pub fn get_force_p1(self: &Spring, web: &Web) -> Vec3 {
        let mut dir = (web.particles[self.first_index].position - web.particles[self.second_index].position);
        let cur_len = dir.length();
        dir /= cur_len;

        //let mut vDiff =

        return dir
    }
}
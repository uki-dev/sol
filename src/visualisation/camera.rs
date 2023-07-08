use glam::{self, Mat4, Quat, Vec3};
use std::f32::consts::PI;

#[derive(Default)]
pub struct Clip {
    pub near: f32,
    pub far: f32,
}

#[derive(Default)]
pub struct Camera {
    pub fov: f32,
    /// the proportional relationship between the width and height of the camera's view frustum
    ///
    /// an aspect of `0.` implies that `aspect` should be automatically calculated from the screen's aspect ratio
    pub aspect: f32,
    pub clip: Clip,
    pub position: Vec3,
    pub rotation: Quat,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            aspect: 1.,
            fov: PI / 3.,
            clip: Clip {
                near: 0.1,
                far: 1000.0,
            },
            ..Default::default()
        }
    }

    // TODO: look into memoization to avoid expensive matrix recalculation
    pub fn view(&self) -> Mat4 {
        let forward = self.rotation * Vec3::Z;
        let up = self.rotation * Vec3::Y;
        return Mat4::look_at_lh(self.position, self.position + forward, up);
    }

    // TODO: look into memoization to avoid expensive matrix recalculation
    // TODO: use reversed depth buffer for greater precision closer to the near clip plane
    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov, self.aspect, self.clip.near, self.clip.far)
    }
}

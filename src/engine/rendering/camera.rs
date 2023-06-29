use std::f32::consts::PI;

use ultraviolet::{projection::lh_yup::perspective_wgpu_dx, Mat3, Mat4, Rotor3, Vec3, Vec4};

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
    pub rotation: Rotor3,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            aspect: 1.,
            fov: PI * 0.5,
            clip: Clip {
                near: 0.1,
                far: 1000.0,
            },
            ..Default::default()
        }
    }

    // TODO: look into memoization to avoid expensive matrix recalculation
    pub fn view(&self) -> Mat4 {
        return Mat4::from_translation(self.position)
            * Rotor3::into_matrix(self.rotation).into_homogeneous();
    }

    // TODO: look into memoization to avoid expensive matrix recalculation
    // TODO: use reversed depth buffer for greater precision closer to the near clip plane
    pub fn projection(&self) -> Mat4 {
        perspective_wgpu_dx(self.fov, self.aspect, self.clip.near, self.clip.far)
    }
}

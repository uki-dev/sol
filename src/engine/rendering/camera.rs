use ultraviolet::{projection::lh_yup::perspective_wgpu_dx, Mat3, Mat4, Rotor3, Vec3, Vec4};

#[derive(Default)]
pub struct Clip {
    pub near: f32,
    pub far: f32,
}

#[derive(Default)]
pub struct Camera {
    pub fov: f32,
    pub aspect: f32,
    pub clip: Clip,
    pub position: Vec3,
    pub rotation: Rotor3,
}

impl Camera {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    // TODO: look into memoization to avoid recalculating these each time
    pub fn world(&self) -> Mat4 {
        return Mat4::from_translation(self.position)
            * Rotor3::into_matrix(self.rotation).into_homogeneous();
    }

    // TODO: look into memoization to avoid recalculating these each time
    pub fn projection(&self) -> Mat4 {
        perspective_wgpu_dx(self.fov, self.aspect, self.clip.near, self.clip.far)
    }
}

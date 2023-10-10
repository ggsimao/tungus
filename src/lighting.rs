use nalgebra_glm::*;

pub struct DirectionalLight {
    pub angle: Vec3,
}

impl DirectionalLight {
    pub fn new(angle: Vec3) -> Self {
        DirectionalLight { angle }
    }
}

pub struct PointLight {
    pub pos: Vec3,
    pub cons: f32,
    pub lin: f32,
    pub quad: f32,
}

impl PointLight {
    pub fn new(pos: Vec3, cons: f32, lin: f32, quad: f32) -> Self {
        PointLight {
            pos,
            cons,
            lin,
            quad,
        }
    }
}

// phi: angle of the inner cone
// gamma: angle of the outer cone
pub struct Spotlight {
    pub pos: Vec3,
    pub dir: Vec3,
    pub phi: f32,
    pub gamma: f32,
}

impl Spotlight {
    pub fn new(pos: Vec3, dir: Vec3, phi: f32, gamma: f32) -> Self {
        Spotlight {
            pos,
            dir,
            phi,
            gamma,
        }
    }
}

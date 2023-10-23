use nalgebra_glm::*;

pub struct DirectionalLight {
    pub dir: Vec3,
    pub amb: Vec3,
    pub diff: Vec3,
    pub spec: Vec3,
}

impl DirectionalLight {
    pub fn new(dir: Vec3, amb: Vec3, diff: Vec3, spec: Vec3) -> Self {
        DirectionalLight {
            dir,
            amb,
            diff,
            spec,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PointLight {
    pub pos: Vec3,
    pub amb: Vec3,
    pub diff: Vec3,
    pub spec: Vec3,
    pub att: Vec3,
}

impl PointLight {
    pub fn new(pos: Vec3, amb: Vec3, diff: Vec3, spec: Vec3, att: Vec3) -> Self {
        PointLight {
            pos,
            amb,
            diff,
            spec,
            att,
        }
    }
}

// phi: angle of the inner cone
// gamma: angle of the outer cone
pub struct Spotlight {
    pub pos: Vec3,
    pub dir: Vec3,
    pub amb: Vec3,
    pub diff: Vec3,
    pub spec: Vec3,
    pub att: Vec3,
    pub phi: f32,
    pub gamma: f32,
}

impl Spotlight {
    pub fn new(
        pos: Vec3,
        dir: Vec3,
        amb: Vec3,
        diff: Vec3,
        spec: Vec3,
        att: Vec3,
        phi: f32,
        gamma: f32,
    ) -> Self {
        Spotlight {
            pos,
            dir,
            amb,
            diff,
            spec,
            att,
            phi,
            gamma,
        }
    }
}

pub struct Lighting {
    pub dir: DirectionalLight,
    pub point: Vec<PointLight>,
    pub spot: Spotlight,
}

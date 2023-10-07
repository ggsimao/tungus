use nalgebra_glm::*;

const ANGLE_LOWER_BOUND: f32 = 0.01;
const ANGLE_UPPER_BOUND: f32 = std::f32::consts::PI - 0.2;

pub struct Camera {
    pos: Vec3,
    target: Vec3,
}

impl Camera {
    pub fn new(initial_pos: Vec3, initial_target: Vec3) -> Self {
        Camera {
            pos: initial_pos,
            target: initial_target,
        }
    }

    pub fn look_at(&self) -> Mat4 {
        look_at(&self.pos, &self.target, &vec3(0.0, 1.0, 0.0))
    }

    pub fn translate(&mut self, offset: Vec3) {
        let direction = normalize(&(self.pos - self.target));
        self.pos += offset.z * direction;
        self.target += offset.z * direction;

        let global_up = vec3(0.0, 1.0, 0.0);
        let camera_right = normalize(&cross(&global_up, &direction));
        self.pos += offset.x * camera_right;
        self.target += offset.x * camera_right;

        let camera_up = cross(&direction, &camera_right);
        self.pos += offset.y * camera_up;
        self.target += offset.y * camera_up;
    }
    pub fn translate_longitudinal(&mut self, offset: f32) {
        self.translate(vec3(offset, 0.0, 0.0));
    }
    pub fn translate_axial(&mut self, offset: f32) {
        self.translate(vec3(0.0, offset, 0.0));
    }
    pub fn translate_frontal(&mut self, offset: f32) {
        self.translate(vec3(0.0, 0.0, offset));
    }

    pub fn rotate(&mut self, euler_angles: Vec3) {
        let mut direction_of_rotation = Vec4::zeros();
        direction_of_rotation.x =
            euler_angles.y.to_radians().cos() * euler_angles.x.to_radians().cos();
        direction_of_rotation.y = euler_angles.x.to_radians().sin();
        direction_of_rotation.z =
            euler_angles.y.to_radians().sin() * euler_angles.x.to_radians().cos();

        let direction_of_camera = self.target - self.pos;
        let original_distance = direction_of_camera.norm();
        let standard_direction = vec3(1.0, 0.0, 0.0);

        let offset_angle = angle(&standard_direction, &direction_of_camera);

        self.target = if offset_angle % ANGLE_UPPER_BOUND > ANGLE_LOWER_BOUND {
            let rotation_matrix = rotation(
                angle(&standard_direction, &direction_of_camera),
                &cross(&standard_direction, &normalize(&direction_of_camera)),
            );
            let new_direction = (rotation_matrix * direction_of_rotation).xyz();
            self.pos + normalize(&new_direction) * original_distance
        } else {
            self.pos + normalize(&direction_of_rotation.xyz()) * original_distance
        };
    }
    pub fn rotate_pitch(&mut self, rotation: f32) {
        self.rotate(vec3(rotation, 0.0, 0.0));
    }
    pub fn rotate_yaw(&mut self, rotation: f32) {
        self.rotate(vec3(0.0, rotation, 0.0));
    }
    pub fn rotate_roll(&mut self, rotation: f32) {
        self.rotate(vec3(0.0, 0.0, rotation));
    }
}

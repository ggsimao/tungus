use nalgebra_glm::*;

pub trait Spatial {
    fn get_model(&self) -> &Mat4;
    fn get_normal(&mut self) -> &Mat3;
    fn set_model(&mut self, model: &Mat4);
    #[inline(always)]
    fn rotate(&mut self, angle: f32, axis: &Vec3) {
        let mut model = *self.get_model();
        let translation = &Vec4::from_column_slice(model.column(3).as_slice());
        let rotation = rotation(angle, axis);
        model.set_column(3, &vec4(0.0, 0.0, 0.0, *model.get((3, 3)).unwrap()));
        model = rotation * model;
        model.set_column(3, &translation);
        self.set_model(&model);
    }
    #[inline(always)]
    fn apply_rotation(&mut self, rotation: &Mat4) {
        let mut model = *self.get_model();
        let translation = &Vec4::from_column_slice(model.column(3).as_slice());
        model.set_column(3, &vec4(0.0, 0.0, 0.0, *model.get((3, 3)).unwrap()));
        model = rotation * model;
        model.set_column(3, &translation);
        self.set_model(&model);
    }
    #[inline(always)]
    fn scale(&mut self, factors: &Vec3) {
        let mut model = *self.get_model();
        let to_origin = -vec4_to_vec3(&Vec4::from_column_slice(model.column(3).as_slice()));
        model = translation(&-to_origin) * scaling(&factors) * translation(&to_origin) * model;
        self.set_model(&model);
    }
    #[inline(always)]
    fn apply_scaling(&mut self, scaling: &Mat4) {
        let mut model = *self.get_model();
        let translation = &Vec4::from_column_slice(model.column(3).as_slice());
        model.set_column(3, &vec4(0.0, 0.0, 0.0, *model.get((3, 3)).unwrap()));
        model = scaling * model;
        model.set_column(3, &translation);
        self.set_model(&model);
    }
    #[inline(always)]
    fn translate(&mut self, offset: &Vec3) {
        let mut model = *self.get_model();
        model.set_column(3, &(model.column(3) + vec3_to_vec4(offset)));
        self.set_model(&model);
    }
}

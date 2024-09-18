use nalgebra_glm::*;
use russimp::material;
use russimp::mesh;
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};
use russimp::Vector3D;
use std::ops::Deref;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    meshes::{BasicMesh, Draw, Vertex},
    shaders::ShaderProgram,
    textures::{Material, Texture2D, TextureType},
};

#[derive(Clone)]
pub struct Model {
    meshes: Vec<BasicMesh>,
    directory: String,
    loaded_textures: Vec<String>,
}

impl Model {
    pub fn new(path: &'static Path) -> Self {
        let directory = path
            .to_path_buf()
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut model = Model {
            meshes: vec![],
            directory,
            loaded_textures: vec![],
        };
        model.load_model(path);
        model
    }
    fn load_model(&mut self, path: &'static Path) {
        let scene = Scene::from_file(
            path.to_str().unwrap(),
            vec![PostProcess::Triangulate, PostProcess::FlipUVs],
        )
        .unwrap();
        let root = scene.root.as_ref().unwrap();
        self.process_node(&root, &scene);
    }
    fn process_node(&mut self, node: &Node, scene: &Scene) {
        for mesh in &node.meshes {
            let scene_mesh = &scene.meshes[(*mesh) as usize];
            let processed_mesh = self.process_mesh(scene_mesh, scene);
            self.meshes.push(processed_mesh);
        }
        let children = node.children.borrow();
        for child in children.deref() {
            let node_child = child.to_owned();
            self.process_node(&node_child, scene);
        }
    }
    fn process_mesh(&mut self, mesh: &mesh::Mesh, scene: &Scene) -> BasicMesh {
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        let loaded_vertices = &mesh.vertices;
        let loaded_normals = &mesh.normals;
        let standard_vec: Vec<Vector3D> = vec![];
        let loaded_tex_coords = mesh.texture_coords[0].as_ref().unwrap_or(&standard_vec);

        for (i, loaded_vertex) in loaded_vertices.iter().enumerate() {
            let mut vertex = Vertex::new(loaded_vertex.x, loaded_vertex.y, loaded_vertex.z);
            if loaded_normals.len() > 0 {
                let loaded_normal = loaded_normals[i];
                vertex.normal = vec3(loaded_normal.x, loaded_normal.y, loaded_normal.z);
            }
            if loaded_tex_coords.len() > 0 {
                let loaded_tex = loaded_tex_coords[i];
                vertex.tex_coords = vec3(loaded_tex.x, -loaded_tex.y, 0.0);
            }
            vertices.push(vertex);
        }

        for face in &mesh.faces {
            for index in &face.0 {
                indices.push(*index);
            }
        }

        let m_material = &scene.materials[mesh.material_index as usize];
        let mut diffuse_maps = self.load_material_textures(
            &m_material,
            material::TextureType::Diffuse,
            TextureType::Diffuse,
        );
        if diffuse_maps.len() == 0 {
            let clr = Texture2D::new(TextureType::Diffuse);
            clr.from_color(&self.load_material_color(&m_material, TextureType::Diffuse));
            diffuse_maps = vec![clr];
        }
        let mut specular_maps = self.load_material_textures(
            &m_material,
            material::TextureType::Specular,
            TextureType::Specular,
        );
        if specular_maps.len() == 0 {
            let clr = Texture2D::new(TextureType::Specular);
            clr.from_color(&self.load_material_color(&m_material, TextureType::Specular));
            specular_maps = vec![clr];
        }
        let shininess = self.load_shininess(&m_material);

        let material = Material::new(diffuse_maps, specular_maps, shininess);

        BasicMesh::new(vertices, indices, material)
    }
    fn load_shininess(&self, mat: &material::Material) -> f32 {
        for property in &mat.properties {
            if property.key == "$mat.shininess" {
                if let material::PropertyTypeInfo::FloatArray(data_float) = &property.data {
                    return data_float[0];
                }
            }
        }
        0.0
    }
    fn load_material_color(&mut self, mat: &material::Material, typename: TextureType) -> Vec3 {
        let key_name = match typename {
            TextureType::Attachment => "",
            TextureType::Diffuse => "$clr.diffuse",
            TextureType::Specular => "$clr.specular",
        };
        for property in &mat.properties {
            if property.key == key_name {
                if let material::PropertyTypeInfo::FloatArray(data_float) = &property.data {
                    return vec3(data_float[0], data_float[1], data_float[2]);
                }
            }
        }
        vec3(0.0, 0.0, 0.0)
    }
    fn load_material_textures(
        &mut self,
        mat: &material::Material,
        ttype: material::TextureType,
        typename: TextureType,
    ) -> Vec<Texture2D> {
        let mut textures = vec![];
        'properties_loop: for property in &mat.properties {
            if property.semantic == ttype {
                // && property.key == "$tex.file" {
                let dir_path = Path::new(&self.directory);
                if let material::PropertyTypeInfo::String(data_string) = &property.data {
                    let tex_path = Path::new(data_string);
                    let string_path = &dir_path.join(tex_path).as_path().display().to_string();
                    for loaded_texture in &self.loaded_textures {
                        if loaded_texture == string_path {
                            continue 'properties_loop;
                        }
                    }
                    let mut texture = Texture2D::new(typename);
                    texture.load(dir_path.join(tex_path).as_path());
                    self.loaded_textures.push(string_path.clone());
                    textures.push(texture);
                }
            }
        }
        textures
    }
}

impl Draw for Model {
    fn draw(&self, shader: &ShaderProgram) {
        for mesh in &self.meshes {
            mesh.draw(shader);
        }
    }
    fn clone_box(&self) -> Box<dyn Draw> {
        Box::new(self.clone())
    }
}

use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::texture::Texture;
use std::rc::Rc;
use crate::material::Material;

#[derive(Clone)]
pub struct Cube {
    pub min: Vec3,  // Esquina mínima del cubo (x, y, z)
    pub max: Vec3,  // Esquina máxima del cubo (x, y, z)
    pub material: Material,
    pub top_texture: Rc<Texture>,     // Textura aplicada a la parte superior del cubo
    pub side_texture: Rc<Texture>,    // Textura aplicada a los lados del cubo
    pub bottom_texture: Rc<Texture>,  // Textura aplicada a la parte inferior del cubo
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        // Cálculo de la intersección del rayo con el cubo
        let mut t_min = (self.min.x - ray_origin.x) / ray_direction.x;
        let mut t_max = (self.max.x - ray_origin.x) / ray_direction.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut t_y_min = (self.min.y - ray_origin.y) / ray_direction.y;
        let mut t_y_max = (self.max.y - ray_origin.y) / ray_direction.y;

        if t_y_min > t_y_max {
            std::mem::swap(&mut t_y_min, &mut t_y_max);
        }

        if (t_min > t_y_max) || (t_y_min > t_max) {
            return Intersect::empty();
        }

        if t_y_min > t_min {
            t_min = t_y_min;
        }

        if t_y_max < t_max {
            t_max = t_y_max;
        }

        let mut t_z_min = (self.min.z - ray_origin.z) / ray_direction.z;
        let mut t_z_max = (self.max.z - ray_origin.z) / ray_direction.z;

        if t_z_min > t_z_max {
            std::mem::swap(&mut t_z_min, &mut t_z_max);
        }

        if (t_min > t_z_max) || (t_z_min > t_max) {
            return Intersect::empty();
        }

        if t_z_min > t_min {
            t_min = t_z_min;
        }

        // Si el rayo no intersecta el cubo, devolvemos una intersección vacía
        if t_min < 0.0 {
            return Intersect::empty();
        }

        // Calcular el punto de intersección
        let point_on_surface = ray_origin + ray_direction * t_min;

         // Calcular la textura adecuada según la cara del cubo
        let color = if (point_on_surface.y - self.max.y).abs() < 1e-4 {
            // Cara superior
            let u = (point_on_surface.x - self.min.x) / (self.max.x - self.min.x);
            let v = (point_on_surface.z - self.min.z) / (self.max.z - self.min.z);
            self.top_texture.get_color(u, v)
        } else if (point_on_surface.y - self.min.y).abs() < 1e-4 {
            // Cara inferior
            let u = (point_on_surface.x - self.min.x) / (self.max.x - self.min.x);
            let v = (point_on_surface.z - self.min.z) / (self.max.z - self.min.z);
            self.bottom_texture.get_color(u, v)
        } else if (point_on_surface.x - self.min.x).abs() < 1e-4 || (point_on_surface.x - self.max.x).abs() < 1e-4 {
            // Caras laterales izquierda y derecha
            let u = (point_on_surface.z - self.min.z) / (self.max.z - self.min.z);
            let v = (point_on_surface.y - self.min.y) / (self.max.y - self.min.y);
            self.side_texture.get_color(u, v)
        } else {
            // Caras frontal y trasera
            let u = (point_on_surface.x - self.min.x) / (self.max.x - self.min.x);
            let v = (point_on_surface.y - self.min.y) / (self.max.y - self.min.y);
            self.side_texture.get_color(u, v) // Usamos la textura lateral aquí
        };

        // Crear un nuevo material usando el color calculado
        let material = Material {
            diffuse: color,
            ..self.material  // Mantener los otros valores del material
        };

        // Calcular la normal del cubo en el punto de intersección
        let normal = self.calculate_normal(point_on_surface);

        // Retornar la intersección con la textura aplicada
        Intersect::new(point_on_surface, normal, t_min, material)
    }
}

impl Cube {
    // Calcular la normal según la cara del cubo en la que se encuentra el punto de intersección
    fn calculate_normal(&self, point: Vec3) -> Vec3 {
        let epsilon = 1e-4;
        if (point.x - self.min.x).abs() < epsilon {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - self.max.x).abs() < epsilon {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - self.min.y).abs() < epsilon {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - self.max.y).abs() < epsilon {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - self.min.z).abs() < epsilon {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
}


use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Material, Intersect};

pub struct Cube {
    pub min: Vec3,  // Corner of the cube (min x, y, z)
    pub max: Vec3,  // Opposite corner (max x, y, z)
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
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
        if t_z_max < t_max {
            //t_max = t_z_max;
        }

        if t_min < 0.0 {
            return Intersect::empty();
        }

        let point = ray_origin + ray_direction * t_min;
        let normal = self.calculate_normal(point);
        let distance = t_min;

        Intersect::new(point, normal, distance, self.material)
    }
}

impl Cube {
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

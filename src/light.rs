
use crate::color::Color;
use nalgebra_glm::Vec3;
use crate::cube::Cube;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }
}

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

const SHADOW_BIAS: f32 = 1e-4;

pub fn cast_shadow(
    intersect: &Intersect,  
    light: &Light,          
    objects: &[Cube],       
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let offset_normal = intersect.normal * SHADOW_BIAS;
    let shadow_ray_origin = if light_dir.dot(&intersect.normal) < 0.0 {
        intersect.point - offset_normal
    } else {
        intersect.point + offset_normal
    };

    let mut shadow_intensity = 0.0;

    // Revisar si algún objeto está bloqueando la luz
    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance  {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }
    shadow_intensity
}

pub fn calculate_lighting(
    point: &Vec3,
    normal: &Vec3,
    view_dir: &Vec3,
    material_diffuse: Color,
    material_specular: f32,
    material_albedo: [f32; 2],
    lights: &[Light],
    objects: &[Cube], 
) -> Color {
    let mut final_color = Color::new(0, 0, 0);

    for light in lights {
        let intersect = Intersect::new(*point, *normal, 0.0, Material::new([1.0, 0.0], 0.5, 0.0, 0.0, Color::new(255, 255, 255), Color::new(255, 255, 255)));
        let shadow_intensity = cast_shadow(&intersect, light, objects);
        let light_intensity = light.intensity * (1.0 - shadow_intensity);
        let light_dir = (light.position - *point).normalize();
        let reflect_dir = reflect(&-light_dir, normal);

        let diffuse_intensity: f32 = normal.dot(&light_dir).max(0.0);
        let diffuse: Color = material_diffuse.scale(diffuse_intensity * material_albedo[0]) * light_intensity;

        let specular_intensity = reflect_dir.dot(&view_dir).max(0.0).powf(material_specular);
        let specular: Color = Color::new(255, 255, 255).scale(specular_intensity * material_albedo[1]) * light_intensity;

        final_color = final_color + diffuse + specular;
    }

    final_color
}
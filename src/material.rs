use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub albedo: [f32; 2],
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub diffuse: Color,  
    pub fresnel_color: Color,
}

impl Material {
    pub fn new(albedo: [f32; 2], specular: f32, transparency: f32, reflectivity: f32, diffuse: Color, fresnel_color: Color) -> Material {
        Material {
            albedo,
            specular,
            transparency,
            reflectivity,
            diffuse,
            fresnel_color
        }
    }
}
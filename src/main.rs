mod framebuffer;
mod ray_intersect;
mod cube;  // Cambiado de sphere a cube
mod color;
mod camera;

use minifb::{ Window, WindowOptions, Key };
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect, Material};
use crate::cube::Cube;  // Cambiado de sphere a cube
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Cube]) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting && tmp.distance < zbuffer {
            zbuffer = intersect.distance;
            intersect = tmp;
        }
    }

    if !intersect.is_intersecting {
        return Color::new(4, 12, 36);
    }
    
    let diffuse = intersect.material.diffuse;

    diffuse
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Cherry Blossom Biome",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // Materiales para los objetos
    let wood = Material {
        diffuse: Color::new(139, 69, 19),  // Café para troncos
    };

    let leaves = Material {
        diffuse: Color::new(255, 182, 193),  // Rosado para hojas de cerezo
    };

    let grass = Material {
        diffuse: Color::new(34, 139, 34),  // Verde para césped
    };

    let water = Material {
        diffuse: Color::new(0, 191, 255),  // Azul para agua
    };

    let grass_block_1 = Cube {
        min: Vec3::new(-5.0, -1.0, -8.0),
        max: Vec3::new(0.0, 0.0, -3.0),
        material: grass,
    };

    let grass_block_2 = Cube {
        min: Vec3::new(0.0, -1.0, -8.0),
        max: Vec3::new(5.0, 0.0, -3.0),
        material: grass,
    };

    let grass_block_3 = Cube {
        min: Vec3::new(-5.0, -1.0, -3.0),
        max: Vec3::new(5.0, 0.0, 0.0),
        material: grass,
    };

    let water_block = Cube {
        min: Vec3::new(-2.0, -0.5, -4.0),
        max: Vec3::new(2.0, -0.3, -2.0),
        material: water,
    };

    // Árbol de cerezo 1
    let cherry_tree_trunk = Cube {
        min: Vec3::new(-3.0, 0.0, -5.0),
        max: Vec3::new(-2.8, 3.0, -4.8),  
        material: wood,
    };

    let cherry_tree_leaves_top = Cube {
        min: Vec3::new(-3.5, 3.0, -5.5),
        max: Vec3::new(-2.5, 4.0, -4.5),  
        material: leaves,
    };

    let cherry_tree_leaves_side = Cube {
        min: Vec3::new(-4.0, 2.0, -5.5),
        max: Vec3::new(-2.0, 3.0, -4.5),  
        material: leaves,
    };

    // Árbol de cerezo 
    let cherry_tree_trunk_2 = Cube {
        min: Vec3::new(3.0, 0.0, -7.0),
        max: Vec3::new(3.2, 3.0, -6.8),
        material: wood,
    };

    let cherry_tree_leaves_2 = Cube {
        min: Vec3::new(2.5, 3.0, -7.5),
        max: Vec3::new(3.5, 4.0, -6.5),
        material: leaves,
    };

    // Nube flotante más arriba
    let cloud_block = Cube {
        min: Vec3::new(-4.0, 6.0, -9.0),
        max: Vec3::new(4.0, 7.0, -8.0),
        material: Material {
            diffuse: Color::new(255, 255, 255),  // Blanco para la nube
        },
    };

    let objects = [
        grass_block_1, grass_block_2, grass_block_3,  // Terreno de césped
        water_block,  // Cuerpo de agua
        cherry_tree_trunk, cherry_tree_leaves_top, cherry_tree_leaves_side,  // Primer cerezo
        cherry_tree_trunk_2, cherry_tree_leaves_2,  // Segundo cerezo
        cloud_block,  // Nube flotante
    ];

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let rotation_speed = PI/10.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {

        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0); 
        }

        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }

        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }

        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        render(&mut framebuffer, &objects, &camera);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}



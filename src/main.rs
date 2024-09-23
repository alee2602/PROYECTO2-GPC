mod camera;
mod color;
mod cube;
mod framebuffer;
mod ray_intersect;
mod texture;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{normalize, Vec3};
use std::f32::consts::PI;
use std::rc::Rc;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::texture::Texture;

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Cube]) -> Color {
    let mut closest_intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let intersect = object.ray_intersect(ray_origin, ray_direction);
        if intersect.is_intersecting && intersect.distance < zbuffer {
            zbuffer = intersect.distance;
            closest_intersect = intersect;
        }
    }

    if !closest_intersect.is_intersecting {
        return Color::new(4, 12, 36); 
    }

    closest_intersect.material.diffuse
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera) {
    framebuffer.clear(0x000000);
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
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

fn create_voxelized_cube(min: Vec3, max: Vec3, texture: Rc<Texture>, voxel_size: f32) -> Vec<Cube> {
    let mut cubes = Vec::new();

    println!("min: {:?}, max: {:?}, voxel_size: {}", min, max, voxel_size);

    let x_min = min.x;
    let y_min = min.y;
    let z_min = min.z;

    let x_max = max.x;
    let y_max = max.y;
    let z_max = max.z;

    let mut x = x_min;
    while x < x_max {
        let mut y = y_min;
        while y < y_max {
            let mut z = z_min;
            while z < z_max {
                // Crear un subcubo
                let subcube = Cube {
                    min: Vec3::new(x, y, z),
                    max: Vec3::new(x + voxel_size, y + voxel_size, z + voxel_size),
                    texture: Rc::clone(&texture),
                };
                cubes.push(subcube);

                z += voxel_size;
            }
            y += voxel_size;
        }
        x += voxel_size;
    }

    cubes
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
    )
    .unwrap();

    let grass_texture = Rc::new(Texture::new("src/textures/grass.png"));
    let wood_texture = Rc::new(Texture::new("src/textures/wood.png"));
    let leaves_texture = Rc::new(Texture::new("src/textures/cherryblossom.jpg"));

    // Crear una base plana para el suelo
    let base_blocks = create_voxelized_cube(
        Vec3::new(-10.0, -1.0, -10.0), 
        Vec3::new(10.0, 0.0, 10.0),    
        Rc::clone(&grass_texture),     
        2.75,                          
    );

    let hill_block_1 = create_voxelized_cube(
        Vec3::new(-7.0, 2.0, -7.0), 
        Vec3::new(7.0, 3.0, 7.0),
        Rc::clone(&grass_texture),
        2.75, 
    );

    let hill_block_2 = create_voxelized_cube(
        Vec3::new(-3.0, 4.0, -3.0),
        Vec3::new(3.0, 6.0, 3.0),
        Rc::clone(&grass_texture),
        2.75, 
    );

    let trunk_blocks_1 = create_voxelized_cube(
        Vec3::new(-0.7, 6.0, -0.7), 
        Vec3::new(0.7, 11.0, 0.7),  
        Rc::clone(&wood_texture),   
        2.75,                       
    );

    let leaves_blocks_1_1 = create_voxelized_cube(
        Vec3::new(-2.5, 11.0, -2.5), 
        Vec3::new(2.5, 12.0, 2.5),
        Rc::clone(&leaves_texture), 
        2.75,                       
    );

    let leaves_blocks_1_2 = create_voxelized_cube(
        Vec3::new(-2.0, 12.0, -2.0), 
        Vec3::new(2.0, 13.0, 2.0),
        Rc::clone(&leaves_texture),
        2.75,
    );

    let leaves_blocks_1_3 = create_voxelized_cube(
        Vec3::new(-1.5, 13.0, -1.5), 
        Vec3::new(1.5, 14.0, 1.5),
        Rc::clone(&leaves_texture),
        2.75,
    );

    // Segundo árbol
    let trunk_blocks_2 = create_voxelized_cube(
        Vec3::new(9.0, 1.0, 9.0),  
        Vec3::new(10.0, 6.0, 10.0), 
        Rc::clone(&wood_texture),
        2.75,
    );
    
    let leaves_blocks_2_1 = create_voxelized_cube(
        Vec3::new(7.5, 6.0, 7.5),  
        Vec3::new(10.5, 7.0, 10.5),
        Rc::clone(&leaves_texture),
        2.75,
    );
    
    let leaves_blocks_2_2 = create_voxelized_cube(
        Vec3::new(8.0, 7.0, 8.0),  
        Vec3::new(10.0, 8.0, 10.0),
        Rc::clone(&leaves_texture),
        2.75,
    );

    // Imprimir la cantidad de subcubos generados para la colina
    println!(
        "Número de subcubos para hill_block_1: {}",
        hill_block_1.len()
    );

    let mut objects = Vec::new();
    objects.extend(base_blocks); 
    objects.extend(hill_block_1); 
    objects.extend(hill_block_2);
    objects.extend(trunk_blocks_1);
    objects.extend(leaves_blocks_1_1);
    objects.extend(leaves_blocks_1_2);
    objects.extend(leaves_blocks_1_3);
    objects.extend(trunk_blocks_2);
    objects.extend(leaves_blocks_2_1);
    objects.extend(leaves_blocks_2_2);
    println!("Número total de objetos: {}", objects.len());

    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 40.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let rotation_speed = PI / 10.0;

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

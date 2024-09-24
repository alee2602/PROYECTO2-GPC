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

pub fn create_voxelized_cube(
    min: Vec3,
    max: Vec3,
    top_texture: Rc<Texture>,
    side_texture: Rc<Texture>,
    bottom_texture: Rc<Texture>,
    voxel_size: f32,
) -> Vec<Cube> {
    let mut cubes = Vec::new();

    let x_steps = ((max.x - min.x) / voxel_size).ceil() as i32;
    let y_steps = ((max.y - min.y) / voxel_size).ceil() as i32;
    let z_steps = ((max.z - min.z) / voxel_size).ceil() as i32;

    for i in 0..x_steps {
        for j in 0..y_steps {
            for k in 0..z_steps {
                let cube_min = Vec3::new(
                    min.x + i as f32 * voxel_size,
                    min.y + j as f32 * voxel_size,
                    min.z + k as f32 * voxel_size,
                );

                let cube_max = Vec3::new(
                    (cube_min.x + voxel_size).min(max.x),
                    (cube_min.y + voxel_size).min(max.y),
                    (cube_min.z + voxel_size).min(max.z),
                );

                let cube = Cube {
                    min: cube_min,
                    max: cube_max,
                    top_texture: Rc::clone(&top_texture),
                    side_texture: Rc::clone(&side_texture),
                    bottom_texture: Rc::clone(&bottom_texture),
                };

                cubes.push(cube);
            }
        }
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

    let grass_texture = Rc::new(Texture::new("src/textures/grass_top.png"));
    let grass_side_texture = Rc::new(Texture::new("src/textures/grass_side.png"));
    let dirt_texture = Rc::new(Texture::new("src/textures/dirt.png"));
    let wood_texture = Rc::new(Texture::new("src/textures/cherrylog.png"));
    let woodplank_texture = Rc::new(Texture::new("src/textures/woodplank.webp"));
    let leaves_texture = Rc::new(Texture::new("src/textures/cherryblossom.jpg"));
    let water_texture = Rc::new(Texture::new("src/textures/water.webp"));

    let base_blocks_left = create_voxelized_cube(
        Vec3::new(-10.0, -2.75, -10.0), 
        Vec3::new(-2.0, 0.0, 10.0),     
        Rc::clone(&grass_texture),      
        Rc::clone(&grass_side_texture), 
        Rc::clone(&dirt_texture),       
        2.75,                           
    );

    let base_blocks_right = create_voxelized_cube(
        Vec3::new(2.0, -2.75, -10.0),
        Vec3::new(10.0, 0.0, 10.0),
        Rc::clone(&grass_texture),
        Rc::clone(&grass_side_texture),
        Rc::clone(&dirt_texture),
        2.75,
    );

    let river_blocks = create_voxelized_cube(
        Vec3::new(-2.0, -2.75, -10.0),
        Vec3::new(2.0, 0.0, 10.0),
        Rc::clone(&water_texture), 
        Rc::clone(&water_texture), 
        Rc::clone(&water_texture), 
        2.75,
    );

    // Colina
    let hill_block_1 = create_voxelized_cube(
        Vec3::new(-10.0, 0.0, -10.0),
        Vec3::new(-3.0, 2.75, -2.0),
        Rc::clone(&grass_texture),      
        Rc::clone(&grass_side_texture), 
        Rc::clone(&dirt_texture),       
        2.75,
    );

    // Primer árbol
    let trunk_blocks_1 = create_voxelized_cube(
        Vec3::new(-7.5, -1.0, -7.5),
        Vec3::new(-5.5, 7.0, -5.5),
        Rc::clone(&wood_texture), 
        Rc::clone(&wood_texture), 
        Rc::clone(&wood_texture), 
        2.75,
    );

    let leaves_blocks_1_1 = create_voxelized_cube(
        Vec3::new(-9.5, 7.0, -9.5), 
        Vec3::new(-3.5, 9.75, -3.5),
        Rc::clone(&leaves_texture), 
        Rc::clone(&leaves_texture), 
        Rc::clone(&leaves_texture), 
        2.75,
    );

    let leaves_blocks_1_2 = create_voxelized_cube(
        Vec3::new(-8.5, 9.75, -8.5), 
        Vec3::new(-4.5, 12.5, -4.5), 
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        2.75,
    );

    // Segundo árbol
    let trunk_blocks_2 = create_voxelized_cube(
        Vec3::new(6.5, -1.0, 6.5), 
        Vec3::new(8.5, 5.0, 8.5),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        2.75,
    );

    let leaves_blocks_2_1 = create_voxelized_cube(
        Vec3::new(4.5, 5.0, 4.5),   
        Vec3::new(10.5, 7.75, 10.5), 
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        2.75,
    );

    let leaves_blocks_2_2 = create_voxelized_cube(
        Vec3::new(5.5, 7.75, 5.5),   
    Vec3::new(9.5, 10.5, 9.5),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        2.75,
    );

    // Base del puente
    let bridge_base = create_voxelized_cube(
        Vec3::new(-5.0, 0.0, 1.0),
        Vec3::new(5.0, 1.0, 3.0),
        Rc::clone(&woodplank_texture),
        Rc::clone(&woodplank_texture),
        Rc::clone(&woodplank_texture),
        2.75,
    );

    let mut objects = Vec::new();
    objects.extend(base_blocks_left);
    objects.extend(base_blocks_right);
    objects.extend(river_blocks);
    objects.extend(hill_block_1);
    objects.extend(trunk_blocks_1);
    objects.extend(leaves_blocks_1_1);
    objects.extend(leaves_blocks_1_2);
    objects.extend(trunk_blocks_2);
    objects.extend(leaves_blocks_2_1);
    objects.extend(leaves_blocks_2_2);
    objects.extend(bridge_base);
    println!("Número total de objetos: {}", objects.len());

    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 35.0),
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

mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod material;
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
use crate::light::{calculate_lighting, Light};
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::texture::Texture;


fn fresnel_effect(normal: Vec3, view_dir: Vec3, f0: f32) -> f32 {
    let cos_theta = normal.dot(&view_dir).max(0.0);
    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    skybox: &[Cube],
    lights: &[Light],
    camera: &Camera,
    is_night: bool,
) -> Color {
    let mut closest_intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    // Verificar intersección con los objetos de la escena
    for object in objects {
        let intersect = object.ray_intersect(ray_origin, ray_direction);
        if intersect.is_intersecting && intersect.distance < zbuffer {
            zbuffer = intersect.distance;
            closest_intersect = intersect;
        }
    }

    // Si no hay intersección con ningún objeto de la escena
    if !closest_intersect.is_intersecting {
        // Renderizar el Skybox en lugar de un color sólido
        for skybox_face in skybox {
            let intersect = skybox_face.ray_intersect(ray_origin, ray_direction);
            if intersect.is_intersecting {
                return intersect.material.diffuse; 
            }
        }

        return if is_night {
            Color::new(10, 10, 30) 
        } else {
            Color::new(63, 96, 188) 
        };
    }

    // Si hay intersección, calcular la iluminación y el fresnel
    let view_dir = (camera.eye - closest_intersect.point).normalize();
    let final_color: Color = calculate_lighting(
        &closest_intersect.point,
        &closest_intersect.normal,
        &view_dir,
        closest_intersect.material.diffuse,
        closest_intersect.material.specular,
        [
            closest_intersect.material.albedo[0],
            closest_intersect.material.albedo[1],
        ],
        lights,
        objects,
    );

    let f0 = closest_intersect.material.reflectivity;
    let fresnel = fresnel_effect(closest_intersect.normal, view_dir, f0);
    let fresnel_intensity = closest_intersect.material.reflectivity;
    let reflected_color = closest_intersect.material.fresnel_color;
    let final_color_with_fresnel = final_color.lerp(reflected_color, fresnel * fresnel_intensity);

    final_color_with_fresnel
}

pub fn render(
    framebuffer: &mut Framebuffer,
    skybox: &[Cube],
    objects: &[Cube],
    camera: &Camera,
    lights: &[Light],
    is_night: bool,
) {
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

        let pixel_color = cast_ray(
            &camera.eye,
            &rotated_direction,
            objects,
            skybox,
            lights,
            &camera,
            is_night,
        );

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
    material: Material,
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
                    material: material.clone(),
                };

                cubes.push(cube);
            }
        }
    }

    cubes
}

pub fn create_skybox(
    sky_front: Rc<Texture>,
    sky_back: Rc<Texture>,
    sky_left: Rc<Texture>,
    sky_right: Rc<Texture>,
    sky_top: Rc<Texture>,
    sky_bottom: Rc<Texture>,
    size: f32,
) -> Vec<Cube> {
    let half_size = size / 2.0;
    let skybox_material = Material {
        diffuse: Color::new(255, 255, 255),
        albedo: [1.0, 0.0],
        specular: 0.0,
        transparency: 0.0,
        reflectivity: 0.0,
        fresnel_color: Color::new(255, 255, 255),
    };

    // Cubo del frente
    let front = create_voxelized_cube(
        Vec3::new(-half_size, -half_size, half_size),
        Vec3::new(half_size, half_size, half_size + 0.01),
        Rc::clone(&sky_front),
        Rc::clone(&sky_front),
        Rc::clone(&sky_front),
        skybox_material.clone(),
        size,
    );

    // Cubo de atrás
    let back = create_voxelized_cube(
        Vec3::new(-half_size, -half_size, -half_size),
        Vec3::new(half_size, half_size, -half_size - 0.01),
        Rc::clone(&sky_back),
        Rc::clone(&sky_back),
        Rc::clone(&sky_back),
        skybox_material.clone(),
        size,
    );

    // Cubo de la izquierda
    let left = create_voxelized_cube(
        Vec3::new(-half_size - 0.01, -half_size, -half_size),
        Vec3::new(-half_size, half_size, half_size),
        Rc::clone(&sky_left),
        Rc::clone(&sky_left),
        Rc::clone(&sky_left),
        skybox_material.clone(),
        size,
    );

    // Cubo de la derecha
    let right = create_voxelized_cube(
        Vec3::new(half_size, -half_size, -half_size),
        Vec3::new(half_size + 0.01, half_size, half_size),
        Rc::clone(&sky_right),
        Rc::clone(&sky_right),
        Rc::clone(&sky_right),
        skybox_material.clone(),
        size,
    );

    // Cubo de arriba
    let top = create_voxelized_cube(
        Vec3::new(-half_size, half_size, -half_size),
        Vec3::new(half_size, half_size + 0.01, half_size),
        Rc::clone(&sky_top),
        Rc::clone(&sky_top),
        Rc::clone(&sky_top),
        skybox_material.clone(),
        size,
    );

    // Cubo de abajo
    let bottom = create_voxelized_cube(
        Vec3::new(-half_size, -half_size - 0.01, -half_size),
        Vec3::new(half_size, -half_size, half_size),
        Rc::clone(&sky_bottom),
        Rc::clone(&sky_bottom),
        Rc::clone(&sky_bottom),
        skybox_material.clone(),
        size,
    );

    let mut skybox = Vec::new();
    skybox.extend(front);
    skybox.extend(back);
    skybox.extend(left);
    skybox.extend(right);
    skybox.extend(top);
    skybox.extend(bottom);

    skybox
}

fn main() {
    let window_width = 600; 
    let window_height = 450;
    let framebuffer_width = 600;
    let framebuffer_height = 450;
    let frame_delay = Duration::from_millis(33);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Cherry Blossom Biome",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let sky_texture = Rc::new(Texture::new("src/textures/sky.jpg"));
    let sky_texture2 = Rc::new(Texture::new("src/textures/sky2.png"));

    let grass_texture = Rc::new(Texture::new("src/textures/grass_top.png"));
    let grass_side_texture = Rc::new(Texture::new("src/textures/grass_side.png"));
    let dirt_texture = Rc::new(Texture::new("src/textures/dirt.png"));
    let wood_texture = Rc::new(Texture::new("src/textures/cherrylog.png"));
    let woodplank_texture = Rc::new(Texture::new("src/textures/woodplank.webp"));
    let leaves_texture = Rc::new(Texture::new("src/textures/cherryblossom.jpg"));
    let water_texture = Rc::new(Texture::new("src/textures/water.webp"));
    let glowstone_texture = Rc::new(Texture::new("src/textures/glowstone.webp"));

    // Definir materiales
    let grass_material = Material::new(
        [0.9, 0.3],
        0.05,
        0.0,
        0.1,
        Color::new(34, 139, 34),
        Color::new(255, 255, 255),
    );
    let wood_material = Material::new(
        [0.6, 0.2],
        0.1,
        0.0,
        0.2,
        Color::new(160, 82, 45),
        Color::new(200, 200, 200),
    );
    let leaves_material = Material::new(
        [0.5, 0.1],
        0.1,
        0.0,
        0.1,
        Color::new(255, 182, 193),
        Color::new(255, 200, 220),
    );
    let water_material = Material::new(
        [0.4, 0.3],
        0.8,
        0.7,
        0.5,
        Color::new(0, 0, 255),
        Color::new(63, 96, 188),
    );
    let glowstone_material = Material::new(
        [1.0, 0.9],
        0.3,
        0.0,
        0.5,
        Color::new(255, 215, 0),
        Color::new(255, 255, 200),
    );

    let skybox = Rc::new(create_skybox(
        Rc::clone(&sky_texture2),
        Rc::clone(&sky_texture),
        Rc::clone(&sky_texture),
        Rc::clone(&sky_texture),
        Rc::clone(&sky_texture2),
        Rc::clone(&sky_texture),
        100.0,
    ));

    let base_blocks_left = create_voxelized_cube(
        Vec3::new(-10.0, -5.5, -10.0),
        Vec3::new(-2.0, 0.0, 10.0),
        Rc::clone(&grass_texture),
        Rc::clone(&grass_side_texture),
        Rc::clone(&dirt_texture),
        grass_material,
        3.75,
    );

    let base_blocks_right = create_voxelized_cube(
        Vec3::new(2.0, -5.5, -10.0),
        Vec3::new(10.0, 0.0, 10.0),
        Rc::clone(&grass_texture),
        Rc::clone(&grass_side_texture),
        Rc::clone(&dirt_texture),
        grass_material,
        3.75,
    );

    let base_blocks_under = create_voxelized_cube(
        Vec3::new(-2.0, -5.5, -10.0),
        Vec3::new(2.0, -2.75, 10.0),
        Rc::clone(&grass_texture),
        Rc::clone(&grass_side_texture),
        Rc::clone(&dirt_texture),
        grass_material,
        3.75,
    );

    let river_blocks = create_voxelized_cube(
        Vec3::new(-2.0, -3.0, -10.0),
        Vec3::new(2.0, -0.5, 10.0),
        Rc::clone(&water_texture),
        Rc::clone(&water_texture),
        Rc::clone(&water_texture),
        water_material,
        3.75,
    );

    // Colina
    let hill_block_1 = create_voxelized_cube(
        Vec3::new(-10.0, 0.0, -10.0),
        Vec3::new(-3.0, 3.0, -2.0),
        Rc::clone(&grass_texture),
        Rc::clone(&grass_side_texture),
        Rc::clone(&dirt_texture),
        grass_material,
        3.75,
    );

    // Primer árbol
    let trunk_blocks_1 = create_voxelized_cube(
        Vec3::new(-7.5, -1.0, -7.5),
        Vec3::new(-5.5, 7.0, -5.5),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        wood_material,
        3.75,
    );

    let leaves_blocks_1_1 = create_voxelized_cube(
        Vec3::new(-9.5, 7.0, -9.5),
        Vec3::new(-3.5, 9.75, -3.5),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        leaves_material,
        3.75,
    );

    let leaves_blocks_1_2 = create_voxelized_cube(
        Vec3::new(-8.5, 9.75, -8.5),
        Vec3::new(-4.5, 12.5, -4.5),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        leaves_material,
        3.75,
    );

    // Segundo árbol
    let trunk_blocks_2 = create_voxelized_cube(
        Vec3::new(6.5, -1.0, 6.5),
        Vec3::new(8.5, 5.0, 8.5),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        wood_material,
        3.75,
    );

    let leaves_blocks_2_1 = create_voxelized_cube(
        Vec3::new(4.5, 5.0, 4.5),
        Vec3::new(10.5, 7.75, 10.5),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        leaves_material,
        3.75,
    );

    let leaves_blocks_2_2 = create_voxelized_cube(
        Vec3::new(5.5, 7.75, 5.5),
        Vec3::new(9.5, 10.5, 9.5),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        Rc::clone(&leaves_texture),
        leaves_material,
        3.75,
    );

    // Base del puente
    let bridge_base = create_voxelized_cube(
        Vec3::new(-5.0, 0.0, 1.0),
        Vec3::new(5.0, 1.0, 3.0),
        Rc::clone(&woodplank_texture),
        Rc::clone(&woodplank_texture),
        Rc::clone(&woodplank_texture),
        wood_material,
        3.75,
    );

    // Poste
    let post_blocks = create_voxelized_cube(
        Vec3::new(6.5, 0.0, -8.0),
        Vec3::new(7.0, 5.0, -7.0),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        Rc::clone(&wood_texture),
        wood_material,
        3.75,
    );

    // Glowstone
    let glowstone_blocks = create_voxelized_cube(
        Vec3::new(5.5, 5.0, -8.5),
        Vec3::new(8.5, 7.75, -5.75),
        Rc::clone(&glowstone_texture),
        Rc::clone(&glowstone_texture),
        Rc::clone(&glowstone_texture),
        glowstone_material,
        3.75,
    );

    let mut objects = Vec::new();
    objects.extend(skybox.iter().cloned());
    objects.extend(base_blocks_left);
    objects.extend(base_blocks_under);
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
    objects.extend(post_blocks);
    objects.extend(glowstone_blocks);
    println!("Número total de objetos: {}", objects.len());

    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 35.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let rotation_speed = PI / 10.0;

    let mut time = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        time += 0.1; 

        let sun_angle = time % (2.0 * PI); 
        let sun_position = Vec3::new(
            15.0 * sun_angle.cos(), 
            25.0 * sun_angle.sin(), 
            15.0,                  
        );

        let moon_angle = (sun_angle + PI) % (2.0 * PI); 
        let moon_position = Vec3::new(15.0 * moon_angle.cos(), 25.0 * moon_angle.sin(), 15.0);


        let (light_position, light_color, light_intensity) = if sun_angle < PI {
            (sun_position, Color::new(255, 255, 224), 1.0)
        } else {
            (moon_position, Color::new(135, 206, 235), 0.5) 
        };

        let mut lights = vec![Light {
            position: light_position,
            color: light_color,
            intensity: light_intensity,
        }];

        if sun_angle >= PI {
            let glowstone_light = Light {
                position: Vec3::new(7.0, 6.375, -7.125), 
                color: Color::new(255, 223, 0),      
                intensity: 0.01,                     
            };
            lights.push(glowstone_light);
        }

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
        if window.is_key_down(Key::X) {
            camera.zoom(1.0);
        }

        if window.is_key_down(Key::Z) {
            camera.zoom(-1.0);
        }

        let is_night = sun_angle >= PI;
        render(
            &mut framebuffer,
            &skybox,
            &objects,
            &camera,
            &lights,
            is_night,
        );

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

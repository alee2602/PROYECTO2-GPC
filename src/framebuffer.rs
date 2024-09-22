pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    // Crear un nuevo framebuffer con el tamaño especificado
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height], 
            background_color: 0x000000,
            current_color: 0xFFFFFF
        }
    }

    // Limpiar el framebuffer con un color dado
    pub fn clear(&mut self, color: u32) {
        for pixel in self.buffer.iter_mut() {
            *pixel = color;
        }
    }

    // Dibujar un píxel en una coordenada específica
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color;
        }
    }

    pub fn point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = self.current_color;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
}

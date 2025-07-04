use crate::fire_types::{FireType, generate_palette};
use crate::particle::Particle;
use crate::config::Config;
use crate::perlin::perlin_noise_1d;
use rand::Rng;
use strum::IntoEnumIterator;

pub struct DoomFire {
    pub width: usize,
    pub height: usize,
    pub pixel_buffer: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
    pub fire_type: FireType,
    t: f64,
    pub particles: Vec<Particle>, // Add this field
}

impl DoomFire {
    pub fn new(config: &Config) -> Self {
        let width = config.screen_width.unwrap_or(1920) / config.scale.unwrap_or(1);
        let height = config.screen_height.unwrap_or(1080) / config.scale.unwrap_or(1);
        let fire_type = match config.fire_type.as_deref() {
            Some("Blue") => FireType::Blue,
            Some("Rainbow") => FireType::Rainbow,
            Some("Green") => FireType::Green,
            Some("Purple") => FireType::Purple,
            Some("WhiteHot") => FireType::WhiteHot,
            Some("White") => FireType::White,
            Some("Ice") => FireType::Ice,
            Some("Toxic") => FireType::Toxic,
            Some("FireAndIce") => FireType::FireAndIce, 
            Some("ChemicalFire") => FireType::ChemicalFire,
            Some("Cyberpunk") => FireType::Cyberpunk,
            Some("Aurora") => FireType::Aurora,
            Some("Plasma") => FireType::Plasma,
            Some("Void") => FireType::Void,
            Some("Candy") => FireType::Candy,
            Some("Random") => {
                let variants: Vec<FireType> = FireType::iter().collect();
                let idx = rand::random::<usize>() % variants.len();
                println!("Random fire type selected: {:?}", variants[idx]);
                variants[idx]
            }
            _ => FireType::Original,
        };
        let background_colour = config.background;
        let size = width * height;
        let pixel_buffer = vec![0; size];
        let palette = generate_palette(fire_type, background_colour, 0.0);

        let mut doom_fire = Self {
            width,
            height,
            pixel_buffer,
            palette,
            fire_type,
            t: 0.0,
            particles: Vec::new(),
        };
        doom_fire.initialize_fire();
        doom_fire
    }

    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        self.t += 0.03; // Increase frequency for more rapid wind changes
        let noise_val = perlin_noise_1d(self.t * 1.5);
        let jitter: f64 = rng.gen_range(-0.5..=0.5);
        let wind = ((noise_val + jitter) * 1.0).round() as isize;
        let delay_chance = 0.3;
        for y in (2..self.height).rev() {
            for x in 0..self.width {
                let src = y * self.width + x;
                let decay = rng.gen_bool(delay_chance) as u8; // Random decay factor
                let x_offset = rng.gen_range(0..3) as isize - 1 + wind;
                let dst_x = x as isize + x_offset;
                let dst_y = if rng.gen_bool(0.3) { y - 2 } else { y - 1 };

                if dst_x >= 0 && dst_x < self.width as isize {
                    let dst = dst_y * self.width + dst_x as usize;
                    let value = self.pixel_buffer[src].saturating_sub(decay);
                    self.pixel_buffer[dst] = value;
                }
            }
        }

        // Spawn new particles randomly at the bottom
        crate::particle::maybe_spawn_particle(
            &mut self.particles,
            self.fire_type,
            self.palette.len(),
            self.width,
            self.height,
            &mut rng,
        );

        // Update and render particles
        crate::particle::update_particles(
            &mut self.particles,
            &mut self.pixel_buffer,
            self.width,
            self.height,
        );

        // Animate Aurora palette by mutating it each frame
        if self.fire_type == FireType::Aurora {
            self.palette = generate_palette(self.fire_type, None, self.t as f32);
        }
    }

    pub fn initialize_fire(&mut self) {
        // Clear the pixel buffer
        self.pixel_buffer.iter_mut().for_each(|x| *x = 0);

        // Draw fire type name as an overlay in the fire buffer
        let name = format!("{:?}", self.fire_type);
        let chars: Vec<char> = name.chars().collect();
        let text_width = chars.len();
        let y = self.height.saturating_sub(4); // 4 rows from the bottom
        let x_offset = (self.width.saturating_sub(text_width)) / 2;

        for (i, c) in chars.iter().enumerate() {
            if *c != ' ' {
                let idx = y * self.width + x_offset + i;
                if idx < self.pixel_buffer.len() {
                    // Use a mid-high palette value for visibility
                    self.pixel_buffer[idx] = (self.palette.len() as u8 * 3 / 4).max(1);
                }
            }
        }

        // Initialize the bottom row as usual
        for x in 0..self.width {
            if self.fire_type == FireType::Candy {
                let rand: usize = rand::thread_rng().gen_range(self.palette.len() / 2..self.palette.len());
                self.pixel_buffer[(self.height - 1) * self.width + x] = rand as u8;
            } else {
                // For other palettes, we start with the last color in the palette
                self.pixel_buffer[(self.height - 1) * self.width + x] = (self.palette.len() - 1) as u8;
            }
        }

        self.particles.clear(); // Clear particles on reset
    }
}

use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
pub enum PlanetType {
    Sun,
    Rocky,
    Gas,
}

pub struct Planet {
    pub planet_type: PlanetType,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
    pub scale: f32,
    pub orbit_inclination: f32,
    pub initial_angle: f32,
    current_angle: f32,
    current_rotation: f32,
}

impl Planet {
    pub fn update(&mut self, time: f32) {
        // Actualizar ángulo orbital
        self.current_angle = self.initial_angle + time * self.orbit_speed;
        
        // Actualizar rotación sobre su propio eje
        self.current_rotation = time * self.rotation_speed;
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        let position = if self.orbit_radius > 0.0 {
            Vec3::new(
                self.current_angle.cos() * self.orbit_radius,
                self.current_angle.sin() * self.orbit_radius * self.orbit_inclination,
                self.current_angle.sin() * self.orbit_radius,
            )
        } else {
            Vec3::ZERO
        };

        Mat4::from_translation(position)
            * Mat4::from_rotation_y(self.current_rotation)
            * Mat4::from_scale(Vec3::splat(self.scale))
    }
}

pub fn create_planet_system() -> Vec<Planet> {
    use std::f32::consts::PI;
    
    vec![
        // Sol en el centro
        Planet {
            planet_type: PlanetType::Sun,
            orbit_radius: 0.0,
            orbit_speed: 0.0,
            rotation_speed: 0.3,
            scale: 3.5,
            orbit_inclination: 0.0,
            initial_angle: 0.0,
            current_angle: 0.0,
            current_rotation: 0.0,
        },
        // Planeta rocoso 1 - Mercurio
        Planet {
            planet_type: PlanetType::Rocky,
            orbit_radius: 10.0,
            orbit_speed: 2.0,
            rotation_speed: 1.5,
            scale: 1.0,
            orbit_inclination: 0.0,
            initial_angle: 0.0,
            current_angle: 0.0,
            current_rotation: 0.0,
        },
        // Planeta rocoso 2 - Tierra
        Planet {
            planet_type: PlanetType::Rocky,
            orbit_radius: 16.0,
            orbit_speed: 1.3,
            rotation_speed: 1.0,
            scale: 1.5,
            orbit_inclination: 0.03,
            initial_angle: PI * 0.5,
            current_angle: PI * 0.5,
            current_rotation: 0.0,
        },
        // Gigante gaseoso 1 - Júpiter
        Planet {
            planet_type: PlanetType::Gas,
            orbit_radius: 25.0,
            orbit_speed: 0.7,
            rotation_speed: 0.6,
            scale: 2.8,
            orbit_inclination: 0.04,
            initial_angle: PI,
            current_angle: PI,
            current_rotation: 0.0,
        },
        // Gigante gaseoso 2 - Saturno
        Planet {
            planet_type: PlanetType::Gas,
            orbit_radius: 35.0,
            orbit_speed: 0.4,
            rotation_speed: 0.5,
            scale: 2.5,
            orbit_inclination: 0.04,
            initial_angle: PI * 1.5,
            current_angle: PI * 1.5,
            current_rotation: 0.0,
        },
    ]
}
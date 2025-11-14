use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetType {
    Sun,           // Sol de la fiesta
    Disco,         // Planeta discoteca
    Rave,          // Planeta rave
    Tropical,      // Fiesta tropical
    Neon,          // Fiesta neón
    Carnival,      // Carnaval
}

pub struct Planet {
    pub planet_type: PlanetType,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
    pub scale: f32,
    pub orbit_inclination: f32,
    pub initial_angle: f32,
    pub has_rings: bool,
    pub has_moon: bool,
    pub moon_orbit_radius: f32,
    pub moon_orbit_speed: f32,
    current_angle: f32,
    current_rotation: f32,
    moon_angle: f32,
}

impl Planet {
    pub fn update(&mut self, time: f32) {
        self.current_angle = self.initial_angle + time * self.orbit_speed;
        self.current_rotation = time * self.rotation_speed;
        if self.has_moon {
            self.moon_angle = time * self.moon_orbit_speed;
        }
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

    pub fn get_position(&self) -> Vec3 {
        if self.orbit_radius > 0.0 {
            Vec3::new(
                self.current_angle.cos() * self.orbit_radius,
                self.current_angle.sin() * self.orbit_radius * self.orbit_inclination,
                self.current_angle.sin() * self.orbit_radius,
            )
        } else {
            Vec3::ZERO
        }
    }

    pub fn get_moon_position(&self) -> Option<Vec3> {
        if !self.has_moon {
            return None;
        }

        let planet_pos = self.get_position();
        let moon_offset = Vec3::new(
            self.moon_angle.cos() * self.moon_orbit_radius,
            self.moon_angle.sin() * self.moon_orbit_radius * 0.3,
            self.moon_angle.sin() * self.moon_orbit_radius,
        );

        // Evitar que la luna intersecte el planeta: asegurar distancia mínima
        // Basado en la esfera base usada para los planetas (radio base = 2.0)
        let base_sphere_radius = 2.0;
        let planet_radius = base_sphere_radius * self.scale;
        let moon_radius = base_sphere_radius * self.scale * 0.3; // la luna usa scale * 0.3
        let min_dist = planet_radius + moon_radius + 0.05; // pequeño margen

        let mut offset = moon_offset;
        let cur_dist = offset.length();
        if cur_dist < min_dist {
            // empujar la luna hacia afuera para evitar intersección
            if cur_dist == 0.0 {
                // en caso raro de distancia cero, colocar en eje X
                offset = Vec3::new(min_dist, 0.0, 0.0);
            } else {
                offset = offset.normalize() * min_dist;
            }
        }

        Some(planet_pos + offset)
    }

    pub fn get_moon_model_matrix(&self) -> Option<Mat4> {
        self.get_moon_position().map(|pos| {
            Mat4::from_translation(pos)
                * Mat4::from_rotation_y(self.current_rotation * 2.0)
                * Mat4::from_scale(Vec3::splat(self.scale * 0.3))
        })
    }

    pub fn get_rings_model_matrix(&self) -> Option<Mat4> {
        if !self.has_rings {
            return None;
        }

        let position = self.get_position();
        Some(
            Mat4::from_translation(position)
                * Mat4::from_rotation_x(0.3)
                * Mat4::from_rotation_y(self.current_rotation * 0.5)
                * Mat4::from_scale(Vec3::new(self.scale * 1.8, 0.1, self.scale * 1.8))
        )
    }
}

pub fn create_planet_system() -> Vec<Planet> {
    use std::f32::consts::PI;
    
    let mut planets = vec![
        // Sol de la Fiesta - Centro de todo
        Planet {
            planet_type: PlanetType::Sun,
            orbit_radius: 0.0,
            orbit_speed: 0.0,
            rotation_speed: 0.3,
            scale: 4.0,
            orbit_inclination: 0.0,
            initial_angle: 0.0,
            has_rings: false,
            has_moon: false,
            moon_orbit_radius: 0.0,
            moon_orbit_speed: 0.0,
            current_angle: 0.0,
            current_rotation: 0.0,
            moon_angle: 0.0,
        },
        // Planeta Disco - con luna
        Planet {
            planet_type: PlanetType::Disco,
            orbit_radius: 12.0,
            orbit_speed: 1.8,
            rotation_speed: 1.5,
            scale: 1.2,
            orbit_inclination: 0.02,
            initial_angle: 0.0,
            has_rings: false,
            has_moon: true,
            moon_orbit_radius: 2.5,
            moon_orbit_speed: 3.0,
            current_angle: 0.0,
            current_rotation: 0.0,
            moon_angle: 0.0,
        },
        // Planeta Rave - con anillos brillantes
        Planet {
            planet_type: PlanetType::Rave,
            orbit_radius: 18.0,
            orbit_speed: 1.3,
            rotation_speed: 2.0,
            scale: 1.5,
            orbit_inclination: 0.04,
            initial_angle: PI * 0.5,
            has_rings: true,
            has_moon: false,
            moon_orbit_radius: 0.0,
            moon_orbit_speed: 0.0,
            current_angle: PI * 0.5,
            current_rotation: 0.0,
            moon_angle: 0.0,
        },
        // Planeta Tropical - sin extras
        Planet {
            planet_type: PlanetType::Tropical,
            orbit_radius: 26.0,
            orbit_speed: 0.9,
            rotation_speed: 1.2,
            scale: 1.8,
            orbit_inclination: 0.03,
            initial_angle: PI,
            has_rings: false,
            has_moon: false,
            moon_orbit_radius: 0.0,
            moon_orbit_speed: 0.0,
            current_angle: PI,
            current_rotation: 0.0,
            moon_angle: 0.0,
        },
        // Planeta Neón - con anillos y luna
        Planet {
            planet_type: PlanetType::Neon,
            orbit_radius: 35.0,
            orbit_speed: 0.6,
            rotation_speed: 0.8,
            scale: 2.2,
            orbit_inclination: 0.05,
            initial_angle: PI * 1.3,
            has_rings: true,
            has_moon: true,
            moon_orbit_radius: 4.0,
            moon_orbit_speed: 2.5,
            current_angle: PI * 1.3,
            current_rotation: 0.0,
            moon_angle: 0.0,
        },
        // Planeta Carnaval - con luna
        Planet {
            planet_type: PlanetType::Carnival,
            orbit_radius: 45.0,
            orbit_speed: 0.4,
            rotation_speed: 0.6,
            scale: 2.5,
            orbit_inclination: 0.06,
            initial_angle: PI * 1.8,
            has_rings: false,
            has_moon: true,
            moon_orbit_radius: 4.5,
            moon_orbit_speed: 2.0,
            current_angle: PI * 1.8,
            current_rotation: 0.0,
            moon_angle: 0.0,
        },
    ];

    // Reducir a la mitad la velocidad orbital de todos los planetas
    for p in &mut planets {
        p.orbit_speed *= 0.5;
    }

    planets
}

pub struct WarpPoint {
    pub name: &'static str,
    pub position: Vec3,
    pub target: Vec3,
}

pub fn get_warp_points() -> Vec<WarpPoint> {
    vec![
        WarpPoint {
            name: "Vista General",
            position: Vec3::new(0.0, 25.0, 60.0),
            target: Vec3::ZERO,
        },
        WarpPoint {
            name: "Sol de Fiesta",
            position: Vec3::new(0.0, 8.0, 12.0),
            target: Vec3::ZERO,
        },
        WarpPoint {
            name: "Planeta Disco",
            position: Vec3::new(12.0, 5.0, 5.0),
            target: Vec3::new(12.0, 0.0, 0.0),
        },
        WarpPoint {
            name: "Planeta Rave",
            position: Vec3::new(18.0, 8.0, 8.0),
            target: Vec3::new(18.0, 0.0, 0.0),
        },
        WarpPoint {
            name: "Planeta Tropical",
            position: Vec3::new(26.0, 10.0, 10.0),
            target: Vec3::new(26.0, 0.0, 0.0),
        },
        WarpPoint {
            name: "Planeta Neón",
            position: Vec3::new(35.0, 12.0, 12.0),
            target: Vec3::new(35.0, 0.0, 0.0),
        },
        WarpPoint {
            name: "Planeta Carnaval",
            position: Vec3::new(45.0, 15.0, 15.0),
            target: Vec3::new(45.0, 0.0, 0.0),
        },
    ]
}
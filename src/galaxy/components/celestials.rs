use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct Planet {
    pub planet_type: String
}

#[derive(Component)]
pub struct Moon {
    pub moon_type: String
}

#[derive(Component)]
pub struct Sun {
    pub temp_k: u32,
    pub spectral_class: String
}

#[derive(Component)]
pub struct AsteroidBelt {

}

#[derive(Component)]
pub struct Celestial {
    pub radius_m: f64,
    pub mass_kg: f64
}
use bevy_ecs::world::World;

use self::{galaxy_structs::LGalaxy, structure_structs::LStationList};

mod orbit;

mod galaxy_structs;
mod load_galaxy;

mod structure_structs;
mod load_structures;

pub fn inject_statics(path_to_assets: String) -> World {
    let mut world = World::default();

    let gal_file = std::fs::read_to_string(format!("{}/galaxy.json", path_to_assets)).expect("Could not read galaxy file");
    let gal: LGalaxy = serde_json::from_str(gal_file.as_str()).expect("Could not deserialize gal file");

    let system_positions = load_galaxy::load_system_positions(&gal);

    let suns = load_galaxy::load_stars(&gal);
    let planets = load_galaxy::load_planets(&gal, &suns);
    let moons = load_galaxy::load_moons(&gal, &planets);
    let belts = load_galaxy::load_belts(&gal, &suns);

    let station_file = std::fs::read_to_string(format!("{}/stations.json", path_to_assets)).expect("Could not read stations file");
    let stations: LStationList = serde_json::from_str(station_file.as_str()).expect("Could not deserialize stations file");

    let stations = load_structures::load_stations(stations, &planets);
    let gates = load_structures::compute_gates(&gal, &planets, &system_positions);

    world.spawn_batch(suns.into_values());
    world.spawn_batch(planets.into_values());
    world.spawn_batch(moons.into_values());
    world.spawn_batch(belts.into_values());

    world.spawn_batch(stations.into_iter());
    world.spawn_batch(gates.into_iter());


    world
}

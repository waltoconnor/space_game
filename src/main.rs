use crate::{db::injector::{inject_statics}, galaxy::resources::path_to_entity::PathToEntityMap, shared::ObjPath};

mod config;
mod db;
mod galaxy;
mod inventory;
mod network;
mod shared;

fn main() {
    println!("Hello, world!");
    let world = inject_statics(String::from("./assets"));
    let db = db::database::DB::load(&String::from("./world"), 1024 * 1024 * 1024);
    let mut gal = galaxy::Galaxy::new(world, db);

    gal.tick(1.0);
    let emap = gal.world.get_resource::<PathToEntityMap>().unwrap();
    let moon_path = &shared::ObjPath::new(&String::from("P1R1:S21"), shared::ObjectType::Moon, &String::from("P1R1:S21-3-3"));
    let ent = emap.get(moon_path).unwrap();

    let test_ship_path = shared::ObjPath::new(&String::from("P1R1:S21"), shared::ObjectType::PlayerShip, &String::from("TEST SHIP"));
    let test_ship = galaxy::bundles::ships::BPlayerShip {
        game_obj: galaxy::components::GameObject::new(&String::from("P1R1:S21"), shared::ObjectType::PlayerShip, &String::from("TEST SHIP")),
        ship: galaxy::components::Ship { ship_name: "Test ship 1".to_string(), ship_class: "TESTICULAR CLASS".to_string(), stats: galaxy::components::Stats { warp_speed_ms: 1e11, thrust_n: 10.0, ang_vel_rads: 1.0, mass_kg: 10.0 }},
        transform: galaxy::components::Transform { pos: nalgebra::Vector3::zeros(), rot: nalgebra::UnitQuaternion::default(), vel: nalgebra::Vector3::zeros() },
        pc: galaxy::components::PlayerController { player_name: "Test Player".to_string() },
        nav: galaxy::components::Navigation { cur_action: galaxy::components::Action::Warp(100.0), warp_state: galaxy::components::WarpState::Aligning, target: galaxy::components::NavTarget::Point(nalgebra::Vector3::new(10000.0, 100000000.0, 100000000.0)), cur_target_pos: None, cur_target_vel: None },
        sig: galaxy::components::Signature::new(10.0),
        sensor: galaxy::components::Sensor::new(),
    };

    gal.world.spawn(test_ship);

    //gal.world.despawn(ent);
    gal.world.spawn(galaxy::components::GameObject { path: ObjPath::new(&String::from("test"), shared::ObjectType::AIShip, &String::from("test_name"))});
    gal.world.get_resource_mut::<galaxy::resources::delta_time::DeltaTime>().unwrap().dt = 1.0;
    gal.tick(1.0);
    gal.tick(1.0);
    gal.tick(1.0);
    gal.tick(1.0);
    let ent = gal.world.get_resource::<PathToEntityMap>().unwrap().get(&test_ship_path);
    let ship = gal.world.get::<galaxy::components::Transform>(ent.unwrap()).unwrap();
    let nav = gal.world.get::<galaxy::components::Sensor>(ent.unwrap()).unwrap();
    println!("{:?}", ship);
    println!("{:?}", nav);
}

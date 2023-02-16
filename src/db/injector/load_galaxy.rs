use std::{collections::HashMap, f64::consts::PI};

use nalgebra::{Vector3, UnitQuaternion, UnitVector3};

use crate::galaxy::bundles::celestials::{BSun, BPlanet, BMoon, BAsteroidBelt};

use super::{galaxy_structs::LGalaxy, orbit::{Orbit, compute_soi}, orbit::orbit_to_csv};


/// PRECONDITION: ALL STARS HAVE UNIQUE NAME
pub fn load_stars(loaded_gal: &LGalaxy) -> HashMap<String, BSun> {
    let mut stars = HashMap::new();

    for r in loaded_gal.regions.iter() {
        for (sys_name, sys_coord) in r.systems.iter() {
            let sun = &sys_coord.sys.star;

            let real_radius = sun.radius_m * 6.957e8; // ACCIDENTALLY STORED THIS AS SOLAR RADIUSES
            let dist = real_radius + 10000000.0;
            let warp_in_point = Orbit::new(dist, 0.0, 0.0, 0.0, 0.0, PI / 2.0, sun.mass_kg);
            let [wx, wy, wz, _, _, _] = orbit_to_csv(&warp_in_point, sun.mass_kg);
            let wip = Vector3::new(wx, wy, wz);

            let bsun = BSun::new(sys_name, &sun.id, sun.temp, &sun.spectral_class, real_radius, sun.mass_kg, wip);
            let test = stars.insert(sun.id.clone(), bsun);
            if test.is_some() {
                eprintln!("ERROR: TWO SUNS WITH DUPLICATE NAMES: {}", test.unwrap().game_object.path.name);
            }
        }
    }
    stars
}

/// PRECONDITION: ALL PLANETS HAVE UNIQUE NAME
pub fn load_planets(loaded_gal: &LGalaxy, suns: &HashMap<String, BSun>) -> HashMap<String, BPlanet> {
    let mut planets = HashMap::new();

    for r in loaded_gal.regions.iter() {
        for (sys_name, sys_coord) in r.systems.iter() {
            let sun = suns.get(&sys_coord.sys.star.id).expect("SYS DOES NOT HAVE SUN");
            for cb in sys_coord.sys.children.iter() {
                let planet = match cb {
                    super::galaxy_structs::LChildBody::AsteroidBelt(_) => { continue; },
                    super::galaxy_structs::LChildBody::Planet(p) => p
                };

                let planet_type = format!("{:?}", planet.body_info.planet_type);

                let orbit = Orbit::from_arr(&planet.orbit.into_arr(), sun.celestial.mass_kg);
                let [x, y, z, _, _, _] = orbit_to_csv(&orbit, sun.celestial.mass_kg);
                let offset = Vector3::new(x, y, z);
                //println!("offset: {:.2?}", offset / 1.4959e11);
                //println!("orbit: {:?}", orbit);

                let tilt = &planet.tilt;
                let phase_quat = UnitQuaternion::from_axis_angle(&Vector3::<f64>::y_axis(), tilt.phase as f64);
                let rotation_axis = &UnitVector3::new_normalize(phase_quat.transform_vector(&Vector3::<f64>::x_axis()).normalize());
                let tilt_quat = UnitQuaternion::from_axis_angle(rotation_axis, tilt.tilt as f64);
                let rot = tilt_quat;
                
                let soi = compute_soi(planet.body_info.mass_kg, sun.celestial.mass_kg, orbit.a);
                let dist = (soi * 0.5).min(planet.body_info.size_m + 100000.0).max(planet.body_info.size_m + 20000.0);
                let warp_in_point = Orbit::new(dist, 0.0, 0.0, 0.0, 0.0, PI / 2.0, planet.body_info.mass_kg);
                let [wip_x, wip_y, wip_z, _, _, _] = orbit_to_csv(&warp_in_point, planet.body_info.mass_kg);
                let warp_in_arr = [wip_x, wip_y, wip_z];
                let warp_point = Vector3::from(warp_in_arr) + offset;
                //println!("warp_point: {:.2?}", warp_point / 1.4959e11);

                let bplanet = BPlanet::new(sys_name, &planet.name, &planet_type, planet.body_info.size_m, planet.body_info.mass_kg, offset, rot, warp_point);
                let test = planets.insert(planet.name.clone(), bplanet);
                if test.is_some() {
                    eprintln!("ERROR: TWO PLANETS WITH DUPLICATE NAMES: {}", test.unwrap().game_object.path.name);
                }
            }
        }
    }

    planets
}

/// PRECONDITION: ALL MOONS HAVE UNIQUE NAME
pub fn load_moons(loaded_gal: &LGalaxy, planets: &HashMap<String, BPlanet>) -> HashMap<String, BMoon> {
    let mut moons = HashMap::new();

    for r in loaded_gal.regions.iter() {
        for (sys_name, sys_coord) in r.systems.iter() {
            for cb in sys_coord.sys.children.iter() {
                let planet = match cb {
                    super::galaxy_structs::LChildBody::AsteroidBelt(_) => { continue; },
                    super::galaxy_structs::LChildBody::Planet(p) => p
                };

                let bplanet = planets.get(&planet.name).expect("Moon parent planet does not exist");

                for m in planet.moons.iter() {
                    let moon_type = format!("{:?}", m.body_info.planet_type);

                    let orbit = Orbit::from_arr(&m.orbit.into_arr(), planet.body_info.mass_kg);
                    let [x, y, z, _, _, _] = orbit_to_csv(&orbit, planet.body_info.mass_kg);
                    let offset = Vector3::new(x, y, z) + bplanet.transform.pos;
    
                    let tilt = &m.tilt;
                    let phase_quat = UnitQuaternion::from_axis_angle(&Vector3::<f64>::y_axis(), tilt.phase as f64);
                    let rotation_axis = &UnitVector3::new_normalize(phase_quat.transform_vector(&Vector3::<f64>::x_axis()).normalize());
                    let tilt_quat = UnitQuaternion::from_axis_angle(rotation_axis, tilt.tilt as f64);
                    let rot = tilt_quat;
                    
                    let soi = compute_soi(m.body_info.mass_kg, planet.body_info.mass_kg, orbit.a);
                    let dist = (soi * 0.5).min(m.body_info.size_m + 100000.0).max(m.body_info.size_m + 20000.0);
                    let warp_in_point = Orbit::new(dist, 0.0, 0.0, 0.0, 0.0, PI / 2.0, m.body_info.mass_kg);
                    let [wip_x, wip_y, wip_z, _, _, _] = orbit_to_csv(&warp_in_point, m.body_info.mass_kg);
                    let warp_in_arr = [wip_x, wip_y, wip_z];
                    let warp_point = Vector3::from(warp_in_arr) + offset;
                
    
                    let bmoon = BMoon::new(sys_name, &m.name, &moon_type, planet.body_info.size_m, planet.body_info.mass_kg, offset, rot, warp_point);
                    let test = moons.insert(m.name.clone(), bmoon);
                    if test.is_some() {
                        eprintln!("ERROR: TWO MOONS WITH DUPLICATE NAMES: {}", test.unwrap().game_object.path.name);
                    }
                }
            }
        }
    }
    moons
}

/// PRECONDITION: ALL ASTERIOD BELTS HAVE UNIQUE NAME
pub fn load_belts(loaded_gal: &LGalaxy, suns: &HashMap<String, BSun>) -> HashMap<String, BAsteroidBelt> {
    let mut belts = HashMap::new();

    for r in loaded_gal.regions.iter() {
        for (sys_name, sys_coord) in r.systems.iter() {
            let sun = suns.get(&sys_coord.sys.star.id).expect("SYS DOES NOT HAVE SUN");
            for cb in sys_coord.sys.children.iter() {
                let belt = match cb {
                    super::galaxy_structs::LChildBody::AsteroidBelt(ab) => ab,
                    super::galaxy_structs::LChildBody::Planet(_) => { continue; }
                };

                let orbit = Orbit::from_arr(&belt.orbit.into_arr(), sun.celestial.mass_kg);
                let [x, y, z, _, _, _] = orbit_to_csv(&orbit, sun.celestial.mass_kg);
                let offset = Vector3::new(x, y, z);
            

                let bbelt = BAsteroidBelt::new(sys_name, &belt.name, offset, UnitQuaternion::identity(), offset + Vector3::new(0.0, 0.0, 1000.0));
                let test = belts.insert(belt.name.clone(), bbelt);
                if test.is_some() {
                    eprintln!("ERROR: TWO ASTEROID BELTS WITH DUPLICATE NAMES: {}", test.unwrap().game_object.path.name);
                }
            }
        }
    }

    belts
}

pub fn load_system_positions(gal: &LGalaxy) -> HashMap<String, Vector3<f64>> {
    let mut spm = HashMap::new();
    for r in gal.regions.iter() {
        for (name, s) in r.systems.iter() {
            let coord = Vector3::new(s.pos.x, s.pos.y, s.pos.z);
            spm.insert(name.clone(), coord);
        }
    }
    spm
}
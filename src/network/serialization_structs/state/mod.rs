mod celestials;
use std::collections::HashSet;

pub use celestials::*;

mod ships;
use serde::{Serialize, Deserialize};
pub use ships::*;

mod structures;
pub use structures::*;

#[derive(Serialize, Deserialize)]
pub enum NetOutState {
    System(SSystem),
    OtherShip(SPlayerShip_OTHER),
    OwnShip(SPlayerShip_OWN),
    LostSight(ObjPath),
}


use bevy_ecs::prelude::*;
use crate::{galaxy::components::*, shared::ObjPath};

#[derive(Serialize, Deserialize)]
pub struct SSystem {
    sun: SSun,
    planets: Vec<SPlanet>,
    moons: Vec<SMoon>,
    belts: Vec<SAsteroidBelt>,
    gates: Vec<SGate>,
    station: Vec<SStation>
}

impl SSystem {
    pub fn new(sun_query: &Query<(&Sun, &GameObject, &Celestial)>, ents: &HashSet<Entity>) -> Option<Self> {
        for e in ents.iter() {
            if let Ok((sun, go, c)) = sun_query.get(*e) {
                let ser_sun = SSun {
                    path: go.path.clone(),
                    temp_k: sun.temp_k,
                    spectral_class: sun.spectral_class.clone(),
                    radius_m: c.radius_m,
                    mass_kg: c.mass_kg
                };
                return Some(SSystem { sun: ser_sun, planets: vec![], moons: vec![], belts: vec![], gates: vec![], station: vec![] });
            }
        }
        None
    }

    pub fn add_planets(&mut self, planet_query: &Query<(&Planet, &GameObject, &Celestial, &Transform)>, ents: &HashSet<Entity>) {
        for e in ents.iter() {
            if let Ok((planet, go, c, t)) = planet_query.get(*e) {
                let ser_planet = SPlanet {
                    path: go.path.clone(),
                    planet_type: planet.planet_type.clone(),
                    radius_m: c.radius_m,
                    mass_kg: c.mass_kg,
                    transform: t.clone()
                };

                self.planets.push(ser_planet);
            }
        }
    }

    pub fn add_moons(&mut self, moon_query: &Query<(&Moon, &GameObject, &Celestial, &Transform)>, ents: &HashSet<Entity>) {
        for e in ents.iter() {
            if let Ok((moon, go, c, t)) = moon_query.get(*e) {
                let ser_moon = SMoon {
                    path: go.path.clone(),
                    moon_type: moon.moon_type.clone(),
                    radius_m: c.radius_m,
                    mass_kg: c.mass_kg,
                    transform: t.clone()
                };

                self.moons.push(ser_moon);
            }
        }
    }

    pub fn add_belts(&mut self, belt_query: &Query<(&AsteroidBelt, &GameObject, &Transform)>, ents: &HashSet<Entity>) {
        for e in ents.iter() {
            if let Ok((belt, go, t)) = belt_query.get(*e) {
                let ser_belt = SAsteroidBelt {
                    path: go.path.clone(),
                    transform: t.clone()
                };

                self.belts.push(ser_belt);
            }
        }
    }

    pub fn add_stations(&mut self, station_query: &Query<(&Station, &GameObject, &Transform)>, ents: &HashSet<Entity>) {
        for e in ents.iter() {
            if let Ok((s, go, t)) = station_query.get(*e) {
                let ser_station = SStation {
                    path: go.path.clone(),
                    transform: t.clone()
                };

                self.station.push(ser_station);
            }
        }
    }

    pub fn add_gates(&mut self, gate_query: &Query<(&Gate, &GameObject, &Transform)>, ents: &HashSet<Entity>) {
        for e in ents.iter() {
            if let Ok((g, go, t)) = gate_query.get(*e) {
                let ser_gate = SGate {
                    path: go.path.clone(),
                    transform: t.clone(),
                    dst_sys: g.dst_system.clone()
                };

                self.gates.push(ser_gate);
            }
        }
    }
}
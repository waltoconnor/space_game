use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum ObjectType {
    Star,
    Planet,
    Moon,
    AsteroidBelt,
    Station,
    PlayerShip,
    AIShip,
    Asteroid,
    Container,
    Wreck,
    PlanetOffice,
    MoonExtractor,
    Starbase,
    Missile,
    Projectile,
    Anomaly,
    Ghost,
    Gate
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct ObjPath {
    pub sys: String,
    pub t: ObjectType,
    pub name: String
}

impl ObjPath {
    pub fn new(sys: &String, t: ObjectType, name: &String) -> Self {
        ObjPath { sys: sys.clone(), t: t, name: name.clone() }
    }
}
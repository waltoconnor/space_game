use std::collections::HashMap;

use serde::{Serialize, Deserialize};

// GALAXY
#[derive(Debug, Serialize, Deserialize)]
pub struct LGalaxy {
    pub nebulas: Vec<LNebula>,
    pub regions: Vec<LRegion>,
    pub region_connections: Vec<LInterRegionConnection>,
    pub wormhole_regions: Option<Vec<LRegion>>,
    pub wormhole_nebula: Option<Vec<LNebula>>
}

#[derive(Debug, Serialize, Hash, Deserialize)]
pub struct LInterRegionConnection {
    pub reg_a: String,
    pub sys_a: String,
    pub reg_b: String,
    pub sys_b: String
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct LGalaxyCoords {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LSystemCoord {
    pub pos: LGalaxyCoords,
    pub sys: LSystem
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LConnection {
    pub a: String,
    pub b: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LRegion {
    pub position: LGalaxyCoords,
    pub systems: HashMap<String, LSystemCoord>,
    pub connections: Vec<LConnection>,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LNebula {
    pub coords: LGalaxyCoords,
    pub a_weight: f64,
    pub b_weight: f64,
    pub c_weight: f64,
    pub name: String    
}

// SYSTEM

#[derive(Serialize, Deserialize, Debug)]
pub struct LSystem {
    pub name: String,
    pub star: LStar,
    pub children: Vec<LChildBody>,
    pub security_level: i32,
    pub moon_productivity: f32,
    pub planet_productivity: f32,
    pub asteroid_productivity: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LChildBody {
    Planet(LPlanet),
    AsteroidBelt(LAsteroidBelt)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LStar {
    pub id: String,
    pub agy_by: u32,
    pub spectral_class: String,
    pub temp: u32,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub lum: f64
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LPlanetType {
    Archeronian,
    Arean, //dead, warm core, some liquid, h2o
    Utgardian, //nh4
    Titanian, //ch4
    Arid, //h20
    Saganian, //nh4
    Asimovian, //ch4
    Asphodelian, // helian planet with stolen atmoshphere,
    Chthonian, //jovian planet with stolen atmosphere
    Janlithic, //tidally locked dry planet
    Jovian, //jupiter (metallic core)
    Phaethonic, //warm core
    Apollonian, //warm core
    Sethian, //warm core
    Hephestian, //tidal forces
    Pelagic, //h2o
    Nunnic, //nh4
    Teathic, //ch4
    Vesperian, //liquid, tidally locked (tidal wave planet)
    Panthallasic, //ocean world, aborted gas giant
    Promethean, //tidal locked cold core, h20
    Burian, //nh4
    Atlan, //ch4
    Ferrinian, //iron friends
    Lithic, //silicates
    Carbonian, //carbonate/carbides
    Telluric, //Halogen volcanos
    Phosphorian, //phosphoric acid volcanos
    Cytherean, //like venus, SO2
    Gelidian, //frozen, dead core
    Erisian, //frozen, warm core (methane volcanos)
    Plutonian, //frozen, tidal forces
    Stygian, //dead and burned
    Gaian, //earth like
    Thion, //earth but sulfur
    Chlortic, //earth but chlorine
    Amunian, //earth but ammonia
    Tartarian, //earth but methane
    Plasmid,
    Helian,
    Acheronian
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum LChemistryType {
    Sulfer, //yellow
    Phosphor, //yellow-red
    Water, //light blue
    Ammonia, //bronze
    Methane, //very deep blue
    Chlorine, //green-yellow
    Iodine, //purple
    Plasma, //glowy blue
    Iron, //red
    Silicates, //brown
    Carbon, //black
    Dead
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LBodyInfo {
    pub size_m: f64,
    pub mass_kg: f64,
    pub atmosphere_density: f64, // also effects weathering
    pub hydrosphere_denisty: f64,
    pub biosphere_density: f64,
    pub chemistry_type: LChemistryType,
    pub tectonics: bool,
    pub tidal_forces: bool,
    pub planet_type: LPlanetType
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LAxisTilt {
    pub tilt: f32,
    pub phase: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LRings {
    pub complex: bool,
    pub inclination: f64,
    pub width: f64,
    pub start_radius: f64,
    pub phase: f64
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum LZone {
    Epistellar,
    Inner,
    Outer
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LPlanet {
    pub name: String,
    pub orbit: LOrbit,
    pub moons: Vec<LMoon>,
    pub resources: LPlanetaryMaterials,
    pub tilt: LAxisTilt,
    pub body_info: LBodyInfo,
    pub rings: Option<LRings>,
    pub zone: LZone,
    //pub pop: PopulationInfo
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LAsteroidBelt {
    pub name: String,
    pub orbit: LOrbit,
    pub resources: LAsteroidMaterials
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LMoon {
    pub name: String,
    pub orbit: LOrbit,
    pub moon_type: LPlanetType,
    pub resources: LMoonMaterials,
    pub tilt: LAxisTilt,
    pub body_info: LBodyInfo,
    pub zone: LZone
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LOrbit {
    pub a: f64,
    pub e: f64,
    pub inc: f64,
    pub lan: f64,
    pub arg_pe: f64,
    pub maae: f64
}

impl LOrbit {
    pub fn into_arr(&self) -> [f64; 6] {
        [self.a, self.e, self.inc, self.lan, self.arg_pe, self.maae]
    }
}

// MATERIALS
#[derive(Serialize, Deserialize, Debug)]
pub struct LTierWeights {
    pub a: f32,
    pub b: f32,
    pub c: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LAsteroidMaterials {
    pub t1: LTierWeights,
    pub t2: LTierWeights,
    pub t3: LTierWeights,
    pub t4: LTierWeights,
    pub t5: LTierWeights
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LMoonMaterials {
    pub t1: LTierWeights,
    pub t2: LTierWeights,
    pub t3: LTierWeights,
    pub t4: LTierWeights
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LPlanetaryMaterials {
    pub a_weight: f32,
    pub b_weight: f32,
    pub c_weight: f32,
    pub base_occurence: f32,
    pub rare_occurence: f32
}
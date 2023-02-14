use std::f64::consts::PI;

use nalgebra::{Rotation3, Vector3};
use serde::{Deserialize, Serialize};

const GRAV_CONST: f64 = 6.6743e-11;
const EQUALITY_EPSILON: f64 = 0.00001;
const NEWTON_RAPHERSON_ITERS: u32 = 10;
const SOI_MULT: f64 = GRAV_CONST / (4.0 * PI * PI);

const i_vec: Vector3<f64> = Vector3::new(1.0, 0.0, 0.0);
const j_vec: Vector3<f64> = Vector3::new(0.0, 1.0, 0.0);
const k_vec: Vector3<f64> = Vector3::new(0.0, 0.0, 1.0);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Orbit {
    pub a: f64,
    pub e: f64,
    pub inc: f64,
    pub lan: f64,
    pub arg_pe: f64,
    pub maae: f64,
    //pub parent_path: Path,
    pub parent_mass: f64,
}

impl Orbit {
    pub fn new(
        a: f64,
        e: f64,
        inc: f64,
        lan: f64,
        arg_pe: f64,
        maae: f64,
        parent_mass: f64,
        //parent_path: &Path,
    ) -> Self {
        Orbit {
            a: a,
            e: e,
            inc: inc,
            lan: lan,
            arg_pe: arg_pe,
            maae: maae,
            //parent_path: parent_path.clone(),
            parent_mass: parent_mass,
        }
    }

    pub fn to_arr(&self) -> [f64; 6] {
        [self.a, self.e, self.inc, self.lan, self.arg_pe, self.maae]
    }

    pub fn from_arr(arr: &[f64; 6], parent_mass: f64) -> Self {
        //pub fn from_arr(arr: &[f64; 6], parent_path: &Path, parent_mass: f64) -> Self {
        let [a, e, inc, lan, arg_pe, maae] = arr;
        Orbit {
            a: *a,
            e: *e,
            inc: *inc,
            lan: *lan,
            arg_pe: *arg_pe,
            maae: *maae,
            //parent_path: parent_path.clone(),
            parent_mass
        }
    }
}


pub fn compute_soi(child_mass: f64, parent_mass: f64, child_sma: f64) -> f64 {
    const GRAV_CONST: f64 = 6.6743e-11;
    let mu_child = GRAV_CONST * child_mass;
    let mu_parent = GRAV_CONST * parent_mass;
    return (mu_child / mu_parent).powf(2.0 / 5.0) * child_sma;
}

pub fn orbit_to_csv(o: &Orbit, parent_mass: f64) -> [f64; 6] {
    koe_to_csv(o.a, o.e, o.inc, o.lan, o.arg_pe, o.maae, parent_mass)
}

pub fn koe_to_csv(
    a: f64,
    e: f64,
    inc: f64,
    lan: f64,
    ap: f64,
    maae: f64,
    parent_mass: f64,
) -> [f64; 6] {
    let mu = parent_mass * GRAV_CONST;
    let maae = maae;
    let ea = newton_rapherson(maae, e, NEWTON_RAPHERSON_ITERS);
    let ta = 2.0 * ((1.0 + e).sqrt() * (ea / 2.0).sin()).atan2((1.0 - e).sqrt() * (ea / 2.0).cos());
    let dist = a * (1.0 - e * ea.cos());
    let mut r = (i_vec * ta.cos() + j_vec * ta.sin()) * dist;
    let mut v = (i_vec * (-ea.sin()) + j_vec * ((1.0 - e.powf(2.0)).sqrt() * ea.cos()))
        * ((mu * a).sqrt() / dist);
    let rot = get_rot(lan, inc, e, ap);

    let (rf, vf) = (rot.transform_vector(&r), rot.transform_vector(&v));

    // Z AND Y ARE REVERSED
    //[rf.x, rf.y, rf.z, vf.x, vf.y, vf.z]
    [rf.x, rf.z, rf.y, vf.x, vf.y, vf.z]
}

fn get_rot(lan: f64, inc: f64, e: f64, ap: f64) -> Rotation3<f64> {
    let mut rot: Rotation3<f64> = Rotation3::identity();
    if !approx_equal(inc, 0.0) {
        let lan_axisangle = k_vec * lan;
        let inc_axisangle = i_vec * inc;
        let lan_rot = Rotation3::from_scaled_axis(lan_axisangle);
        let inc_rot = Rotation3::from_scaled_axis(inc_axisangle);
        rot = inc_rot * lan_rot;
    }

    if !approx_equal(e, 0.0) {
        let ap_axisangle = k_vec * ap;
        rot = Rotation3::from_scaled_axis(ap_axisangle) * rot;
    }
    rot
}

pub fn csv_to_koe(x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64, parent_mass: f64) -> [f64; 6] {
    let r = Vector3::new(x, y, z);
    let v = Vector3::new(vx, vy, vz);
    let mu = parent_mass * GRAV_CONST;
    let h = r.cross(&v);
    let e = v.cross(&h) / mu - r.normalize();
    let n = k_vec.cross(&h);
    let cos_inc = h.dot(&k_vec) / h.norm();
    let inc = if cos_inc > 1.0 { 0.0 } else { cos_inc.acos() };
    let es = e.norm();
    let mut lan = if n.dot(&j_vec) >= 0.0 {
        (n.dot(&i_vec) / n.norm()).acos()
    } else {
        2.0 * PI - (n.dot(&i_vec) / n.norm()).acos()
    };
    let right = h.cross(&n);
    let mut ap = if e.dot(&right) >= 0.0 {
        (n.dot(&e) / (n.norm() * e.norm())).acos()
    } else {
        2.0 * PI - (n.dot(&e) / (n.norm() * e.norm())).acos()
    };
    if approx_equal(es, 0.0) {
        ap = 0.0;
    }
    if approx_equal(inc, 0.0) {
        lan = 0.0;
        if approx_equal(es, 0.0) {
            ap = if e.dot(&j_vec) >= 0.0 {
                (i_vec.dot(&e) / e.norm()).acos()
            } else {
                2.0 * PI - (i_vec.dot(&e) / e.norm()).acos()
            };
        }
    }

    let inner = e.norm() * r.norm();
    // println!("e.dot(r): {}, inner: {}", e.dot(&r), inner);
    let mut ta = if r.dot(&v) >= 0.0 {
        (e.dot(&r) / inner).acos()
    } else {
        2.0 * PI - e.dot(&r) / (e.norm() * r.norm()).acos()
    };

    if e.dot(&r) / (e.norm() * r.norm()) > 1.0 {
        ta = 0.0;
    }

    if approx_equal(es, 0.0) {
        if approx_equal(inc, 0.0) {
            ta = if i_vec.dot(&v) <= 0.0 {
                (i_vec.dot(&r) / (i_vec.norm() * r.norm())).acos()
            } else {
                2.0 * PI - (i_vec.dot(&r) / (i_vec.norm() * r.norm())).acos()
            }
        } else {
            ta = if n.dot(&v) <= 0.0 {
                (n.dot(&r) / (n.norm() * r.norm())).acos()
            } else {
                2.0 * PI - (n.dot(&r) / (n.norm() * r.norm())).acos()
            }
        }
    }

    let ea = 2.0 * ((ta / 2.0).tan() / ((1.0 + es) / (1.0 - es)).sqrt()).atan();
    // println!("TA: {}, EA: {}", ta, ea);
    let maae = ea - es * ea.sin();
    let a = 1.0 / (2.0 / r.norm() - v.norm_squared() / mu);

    [a, es, inc, lan, ap, maae]
}

fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EQUALITY_EPSILON
}

pub fn get_period(a: f64, parent_mass: f64) -> f64 {
    (a.powi(3) / (parent_mass * GRAV_CONST)).sqrt() * 2.0 * PI
}

fn newton_rapherson(m0: f64, e: f64, iters: u32) -> f64 {
    let mut ea = m0;
    for _ in 0..iters {
        ea = ea - (ea - e * ea.sin() - m0) / (1.0 - e * ea.cos());
    }
    ea
}

use bevy_ecs::prelude::*;
use nalgebra::Vector3;
use serde::{Serialize, Deserialize};

use crate::shared::ObjPath;

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum Action {
    Warp(f64),
    Orbit(f64),
    Approach,
    KeepAtRange(f64),
    AlignTo,
    None
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum WarpState {
    Aligning,
    Warping,
    NotWarping
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NavTarget {
    Obj(ObjPath),
    Point(Vector3<f64>),
    None
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Navigation {
    pub cur_action: Action,
    pub warp_state: WarpState,
    pub target: NavTarget,
    pub cur_target_pos: Option<Vector3<f64>>,
    pub cur_target_vel: Option<Vector3<f64>>
}

impl Navigation {
    pub fn new() -> Self {
        Navigation { 
            cur_action: Action::None, 
            warp_state: WarpState::NotWarping, 
            target: NavTarget::None, 
            cur_target_pos: None, 
            cur_target_vel: None 
        }
    }

    pub fn reset(&mut self) {
        self.cur_action = Action::None;
        self.warp_state = WarpState::NotWarping;
        self.target = NavTarget::None;
        self.cur_target_pos = None;
        self.cur_target_vel = None;
    }
}
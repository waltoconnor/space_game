use std::sync::Mutex;

use bevy_ecs::prelude::*;
use crate::galaxy::{components::*, resources::star_system_table::SystemMapTable, events::{EState, EEvent}};

/* TODO: rework to only operate over signatured objects, we don't need to find the visibility of static objects */
pub fn sys_get_visible(mut sensors: Query<(Entity, &mut Sensor, &Ship, &PlayerController)>, signatures: Query<&Signature>, game_objects: Query<(&GameObject, &Transform)>, sys_map: Res<SystemMapTable>, mut est: EventWriter<EState>) {
    let est_mut = Mutex::new(est);
    sensors.par_for_each_mut(4, |(ent, mut sensor, ship, pc)| {
        let sensor_system = match sys_map.get_system_of_entity(ent){
            Some(ss) => ss,
            None => {
                // This goes off the tick the sensor spawns as it hasn't been reflected in the sys_map table yet
                //if sys_map.entity_table.is_empty() { return; } 
                //eprintln!("NO system found for sensor (bookeeping is broken or sensor object wasn't cleaned up properly?)"); 
                return; 
            }
        };
        let objects_in_system = match sys_map.get_entities_in_system(sensor_system) {
            Some(ois) => ois,
            None => { eprintln!("System that sensor is in not found, something is really broken"); return; }
        };

        //sensor.lockable_objs.clear();
        //sensor.visible_objs.clear();

        let sensor_pos = match game_objects.get(ent) {
            Ok((_, mt)) => mt,
            Err(_) => {
                eprintln!("Sensor could not find own game object in query");
                return;
            } 
        };

        for entity in objects_in_system {
            if *entity == ent {
                // looking at ourselves
                continue;
            }

            let (object_path, target_pos) = match game_objects.get(*entity) {
                Ok((go, t)) => (go.path.clone(), t),
                Err(_) => {
                    eprintln!("Game object from sys_map not found in game object query");
                    return;
                }
            };
        
            // if something does not have a signature component, it is static
            let vis_status = match signatures.get(*entity) {
                Ok(sig) => vis_test(&sensor, &ship, sensor_pos, sig, target_pos),
                Err(_) => ObjectVisibility::Static
            };

            //println!("Ship {} is sensing {:?} ({:?})", ship.ship_name, object_path, vis_status);

            let newly_visible = match vis_status {
                ObjectVisibility::Lockable => {
                    let was_visible = sensor.visible_objs.remove(&object_path);
                    sensor.lockable_objs.insert(object_path.clone()) && !was_visible //returns true if the item was previously invisible
                },
                ObjectVisibility::Visible => {
                    let was_lockable = sensor.lockable_objs.remove(&object_path);
                    sensor.visible_objs.insert(object_path.clone()) && !was_lockable //returns true if the item was previously invisible
                },
                ObjectVisibility::NotVisible => {
                    if sensor.lockable_objs.remove(&object_path) || sensor.visible_objs.remove(&object_path) {
                        //IF THIS IS TRUE, WE ACTUALLY LOST SIGHT OF THE OBJECT
                        let mut ew = est_mut.lock().expect("Could not lock mutex");
                        ew.send(EState::LostSight(pc.player_name.clone(), object_path.clone()));
                    }
                    false
                }
                ObjectVisibility::Static => false
            };

            if newly_visible {
                let mut ew = est_mut.lock().expect("Could not lock mutex");
                ew.send(EState::OtherShip(pc.player_name.clone(), object_path.clone(), vis_status));
            }
        }
            
    });
}

fn vis_test(sensor: &Sensor, sensing_ship: &Ship, sensor_pos: &Transform, target: &Signature, target_pos: &Transform) -> ObjectVisibility {
    if sensor_pos.pos.metric_distance(&target_pos.pos) > 100_000.0 {
        return ObjectVisibility::NotVisible;
    }

    ObjectVisibility::Lockable
}
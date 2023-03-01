use crate::{galaxy::{Galaxy, resources::{database_resource::DatabaseResource, path_to_entity::PathToEntityMap}, components::{Ship, Stats, Hanger, Station}}, network::{server::ServerHandle, self}, shared::{ObjPath, self}, config::Config, db, inventory::{Inventory, Stack}};

/// returns if login was successful
pub fn handle_new_player(gal: &Galaxy, name: &String, token: &String, server: &ServerHandle, config: &Config) -> bool{
    let db = &gal.world.get_resource::<DatabaseResource>().expect("Could not get database resource").db;
    match db.account_try_login(&name, &token) {
        db::database::LoginStatus::Good => { server.send_message_to_player(name.clone(), crate::network::messages::outgoing::NetOutgoingMessage::LoginOk); true },
        db::database::LoginStatus::BadPass => { server.send_message_to_player(name.clone(), network::messages::outgoing::NetOutgoingMessage::LoginBad); false },
        db::database::LoginStatus::NoAccount => {
            let starter_station_path = ObjPath::new(&config.gameplay_config.starting_system, shared::ObjectType::Station, &config.gameplay_config.starting_station);
            let starter_station = gal.world.get_resource::<PathToEntityMap>().expect("Could not get path to entity map for new player").get(&starter_station_path).expect("Starter station not found in world");
            let sh = gal.world.get::<Hanger>(starter_station).expect("Could not get starter hanger component"); 
            db.account_create(&name, &token, starter_station_path);
            db.bank_new_account(&name);
            db.bank_apply_transaction(&name, config.gameplay_config.starting_money, String::from("Starting money"));
            db.market_add_player_index(&name);
            db.inventory_ensure(&name, sh.hanger_uid.clone());
            let mut ship_inv = Inventory::new(None, Some(10000));
            ship_inv.insert_stack(Stack::new("haxonite".to_string(), 200));
            ship_inv.insert_stack(Stack::new("hapkeite".to_string(), 200));
            ship_inv.insert_stack(Stack::new("wolframite".to_string(), 200));
            db.hanger_add_ship(&name, sh.hanger_uid.clone(), Ship { 
                ship_name: String::from("New ship"),
                ship_class: String::from("Test Ship"),
                stats: Stats {
                    warp_speed_ms: 1.496e11,
                    thrust_n: 100.0,
                    ang_vel_rads: 1.0,
                    mass_kg: 10.0,
                    warp_spool_s: 5.0
                },
                inventory: ship_inv
            });

            let cur_hanger = db.hanger_get_ships(&name, sh.hanger_uid.clone()).expect("Could not get ships from new player hanger");
            let slot = cur_hanger.inventory.keys().last().expect("Could not get last key in hanger");
            db.hanger_set_active_ship_slot(name, sh.hanger_uid.clone(), *slot);
            // let cur_hanger = db.hanger_get_ships(&name, sh.hanger_uid).expect("Could not get ships from new player hanger");
            // println!("{:?}", cur_hanger);
            server.send_message_to_player(name.clone(), network::messages::outgoing::NetOutgoingMessage::LoginOk);
            true
        }
    }
}
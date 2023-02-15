use crate::{galaxy::{Galaxy, resources::database_resource::DatabaseResource}, network::{server::ServerHandle, self}, shared::{ObjPath, self}, config::Config, db};


pub fn handle_new_player(gal: &Galaxy, name: &String, token: &String, server: &ServerHandle, config: &Config) {
    let db = &gal.world.get_resource::<DatabaseResource>().expect("Could not get database resource").db;
    match db.account_try_login(&name, &token) {
        db::database::LoginStatus::Good => server.send_message_to_player(name.clone(), crate::network::messages::outgoing::NetOutgoingMessage::LoginOk),
        db::database::LoginStatus::BadPass => server.send_message_to_player(name.clone(), network::messages::outgoing::NetOutgoingMessage::LoginBad),
        db::database::LoginStatus::NoAccount => {
            db.account_create(&name, &token, ObjPath::new(&config.gameplay_config.starting_system, shared::ObjectType::Station, &config.gameplay_config.starting_station));
            server.send_message_to_player(name.clone(), network::messages::outgoing::NetOutgoingMessage::LoginOk)
        }
    };
}
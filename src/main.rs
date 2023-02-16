use std::{thread::{JoinHandle, self}, time::{Duration, Instant}};

use crate::{db::injector::{inject_statics, load_items}, inventory::ItemTable};

mod config;
mod db;
mod galaxy;
mod inventory;
mod network;
mod shared;
mod special;

fn spawn_sleepy_thread(time_ms: u32) -> JoinHandle<()> {
    thread::spawn(move || { thread::sleep(Duration::from_millis(time_ms as u64)) })
}

fn main() {
    println!("Hello, world!");
    let config = config::load_config("./assets/config.json".to_string());
    let items: ItemTable = load_items(config.assets_path.clone());
    let world = inject_statics(config.assets_path.clone());
    let db = db::database::DB::load(&config.db_path, 1024 * 1024 * 1024, items.clone());
    let server = network::server::start_network(format!("{}:{}", config.network.websocket_ip, config.network.websocket_port));
    let mut gal = galaxy::Galaxy::new(world, db, items);

    let mut last_cycle_time: f32 = 0.1;

    loop {
        let sleepy_thread = spawn_sleepy_thread(100);
        let start = Instant::now();
        let msgs_in = server.get_messages();
        if msgs_in.len() > 0 { println!("{:?}", msgs_in); };

        //disperse messages
        for (player, msg) in msgs_in {
            println!("{:?}", msg);
            // handle logins
            let is_valid_player = match &msg {
                network::messages::incoming::NetIncomingMessage::Login(name, token) => special::new_player::handle_new_player(&gal, name, token, &server, &config),
                _ => true
            };

            if !is_valid_player { continue; }
            gal.queue_incoming_message(&player, msg);
        }

        let msgs_out = gal.run_cycle(last_cycle_time.into());
        //println!("{} msgs to queue", msgs_out.len());
        for (player, msgs) in msgs_out {
            for msg in msgs {
                let msg_status = server.send_message_to_player(player.clone(), msg);
            } 
        }

        let exec_time = start.elapsed().as_secs_f32();
        sleepy_thread.join().expect("Could not wait for sleepy thread");
        last_cycle_time = start.elapsed().as_secs_f32();
        // println!("CYCLE: {} (EXEC: {}, OCC: {} %)", last_cycle_time, exec_time, 100.0 * exec_time / last_cycle_time);
    }
}

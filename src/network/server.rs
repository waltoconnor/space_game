use std::net::SocketAddrV4;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::spawn;
use std::time::Duration;
use dashmap::DashMap;
use serde::Deserialize;
use serde::Serialize;
use tungstenite::accept;
use serde_json::from_str;

use super::messages::incoming::NetIncomingMessage;
use super::messages::outgoing::NetOutgoingMessage;

const TIMEOUT_MS: u64 = 5; //check the net socket 200 times a second, caps our send rate at 200msg/s

pub struct ServerHandle {
    pub player_map: Arc<DashMap<String, Mutex<Sender<NetOutgoingMessage>>>>,
    pub incoming_pipe: Receiver<(String, NetIncomingMessage)>
}

pub enum SendStatus {
    Ok,
    PlayerDisconnected,
    Err
}

impl ServerHandle {
    pub fn new(dmap: Arc<DashMap<String, Mutex<Sender<NetOutgoingMessage>>>>, incoming_pipe: Receiver<(String, NetIncomingMessage)>) -> Self {
        ServerHandle { player_map: dmap, incoming_pipe: incoming_pipe }
    }

    pub fn send_message_to_player(&self, player: String, msg: NetOutgoingMessage) -> SendStatus {
        //println!("SENDING");
        match self.player_map.get(&player) {
            None => { println!("Sending message to non existent player: {}", &player); println!("{:#?}", self.player_map); SendStatus::PlayerDisconnected },
            Some(p) => { match p.lock().expect("Could not lock player").send(msg) {
                Ok(_) => SendStatus::Ok,
                Err(_) => SendStatus::Err
            }
        }
        }
    }

    pub fn get_messages(&self) -> Vec<(String, NetIncomingMessage)> {
        let mut msgs = Vec::new();
        while let Ok(m) = self.incoming_pipe.try_recv() {
            msgs.push(m);
        }
        msgs
    }
}

//fn start_network(addr: String) -> (Receiver<FromPlayerMessage>, Arc<DashMap<String, Mutex<Sender<ToPlayerMessage>>>>) {
pub fn start_network(addr: String) -> ServerHandle {
    println!("Starting server");
    let addr: SocketAddrV4 = addr.parse().expect("Could not parse address");
    let dmap = Arc::new(DashMap::<String, Mutex<Sender<NetOutgoingMessage>>>::new());
    let (a, b) = std::sync::mpsc::channel();
    
    let dmap_clone = dmap.clone();
    spawn(move || {
        server_thread(addr, a, dmap_clone.clone())
    });

    ServerHandle::new(dmap, b)
}

// #[derive(Deserialize)]
// struct IntroMessage {
//     pub name: String,
//     pub access_token: String
// }

// #[derive(Serialize)]
// struct IntroMessageResponse {
//     pub status: String
// }

/// A WebSocket echo server
fn server_thread(addr: SocketAddrV4, server_receive_pipe: Sender<(String, NetIncomingMessage)>, send_to_player_map: Arc<DashMap<String, Mutex<Sender<NetOutgoingMessage>>>>) {
    let server = TcpListener::bind(addr).expect("Could not bind tcp sock");
    println!("Listening on {:?}", addr);
    //GOAL: ALL SERIALIZING AND DESERIALIZING HAPPENS IN HERE
    for mut stream in server.incoming() {
        let server_rec_pipe = server_receive_pipe.clone();
        let local_send_to_player_map = send_to_player_map.clone();

        //EVERYTHING AFTER THIS IS ALLOWED TO CRASH, IT'LL JUST KICK THE PLAYER
        spawn (move || {
            println!("New connection");
            //create websocket
            stream.as_mut().unwrap().set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS))).expect("Could not set timeout");
            let mut websocket = accept(stream.unwrap()).unwrap();
            let (server_send_pipe, local_receive_pipe) = std::sync::mpsc::channel::<NetOutgoingMessage>();

            let mut tmp_queue = vec![];

            let mut name = String::new();
            let mut token;

            //first loop waits for the intro message and discards everything else
            'login: loop {
                let msg = match websocket.read_message(){
                    Ok(m) => m,
                    Err(_) => { continue; }
                };
                if msg.is_text() {
                    (name, token) = match from_str::<NetIncomingMessage>(msg.into_text().expect("Could not process websocket message as text").as_str()) {
                        Ok(m) => match m {
                            NetIncomingMessage::Login(name, token) => (name, token),
                            _ => { continue; }
                        },
                        Err(_) => { 
                            println!("Intro message malformed");
                            let res = NetOutgoingMessage::LoginBad;
                            websocket.write_message(serde_json::to_string(&res).expect("Could not serialize error message").into()).expect("Could not send error message");
                            continue; 
                        }
                    };

                    if local_send_to_player_map.contains_key(&name) {
                        println!("Player is already logged in, relogging under new account");
                    }

                    println!("Player connecting: {}", &name);

                    // let login_msg = FromPlayerMessage {
                    //     player: intro.name.clone(),
                    //     data: crate::server::messages::incoming::ServerInMessage::LoginRequest(intro.access_token)
                    // };
                    let login_msg = NetIncomingMessage::Login(name.clone(), token.clone());

                    server_rec_pipe.send((name.clone(), login_msg)).expect("Could not send server login message");
                    local_send_to_player_map.insert(name.clone(), Mutex::new(server_send_pipe.clone()));

                    'get_message: loop {
                        let result = local_receive_pipe.recv().expect("Did not get data back from server");
                        if match result { NetOutgoingMessage::LoginBad => true, _ => false } { //impl partialeq lmao
                            local_send_to_player_map.remove(&name);
                            let res = NetOutgoingMessage::LoginBad;
                            websocket.write_message(serde_json::to_string(&res).expect("Could not serialize bad password message").into()).expect("Could not send bad password message");
                            println!("Player {} failed login", &name);
                            continue 'login; // LOOP BACK AND LET THEM TRY AGAIN
                        }
                        match result {
                            NetOutgoingMessage::LoginOk => (),
                            a => { //WE GOT A MESSAGE THAT WASN'T THE LOG IN SUCCESS
                                println!("Queueing out of order message");
                                tmp_queue.push(a); 
                                continue 'get_message; // STORE IT AND WAIT FOR THE LOG IN SUCCESS
                            }
                        }

                        println!("Player {} logged in", &name);
                        
                        // WE ARE APPROVED TO LOG IN, SEND THE MESSAGE THEN DUMP ALL THE STORED UP MESSAGES THAT CAME OUT OF ORDER
                        let res = NetOutgoingMessage::LoginOk;
                        websocket.write_message(serde_json::to_string(&res).expect("Could not serialize ok message").into()).expect("Could not send ok message");

                        for msg in tmp_queue {
                            //let data = ToPlayerMessage::new(&intro.name, msg);
                            websocket.write_message(serde_json::to_string(&msg).expect("Could not serialize queued message").into()).expect("Could not send queued message");
                        }

                        //name = intro.name.clone();
                        break 'login;
                    }
                }
                
            }

            //the server looks up our server_send_pipe in the hashmap and forwards data to our local_receive_pipe
            //we listen on the websocket and forward anything relevant to the server_rec_pipe

            //TODO: add logic that short circuits the timeout if we just sent a packet so we can keep sending more
            loop {
                //WAIT FOR PACKET UNTIL TIMEOUT
                let msg = match websocket.read_message(){
                    Ok(msg) => msg, //IF WE GET A PACKET, HANDLE IT
                    Err(e) => { //IF WE TIMEOUT, SEE IF WE HAVE ANYTHING TO SEND
                        let closed = match e {
                            tungstenite::Error::AlreadyClosed => true,
                            tungstenite::Error::ConnectionClosed => true,
                            _ => false
                        };
                        
                        if closed {
                            println!("Player {} disconnected due to socket break", &name);
                            if name != String::new() {
                                local_send_to_player_map.remove(&name);
                                websocket.close(None);
                                server_rec_pipe.send((name.clone(), NetIncomingMessage::Disconnect)).expect("Could not send disconnect message");
                                return;
                            }
                        }

                        while let Ok(tpm) = local_receive_pipe.try_recv() { //IF WE HAVE A PACKET FROM THE SERVER TO FORWARD, DO SO
                            //println!("Sending message to {}", tpm.player);
                            // println!("Msg: {:?}", tpm);
                            let str = serde_json::to_string(&tpm).expect("Could not serialize message");
                            println!("Sending message: {}B", str.len());
                            websocket.write_message(str.into()).expect("Could not send message"); 
                        }
                        //println!("Exiting (queue empty: {})", local_receive_pipe.try_recv().is_err());
                        continue;
                    }
                };

                if msg.is_close() {
                    println!("Player {} disconnected with close message", &name);
                    if name != String::new() {
                        local_send_to_player_map.remove(&name);
                        server_rec_pipe.send((name.clone(), NetIncomingMessage::Disconnect)).expect("Could not send disconnect message");
                        return;
                    }
                }
                
                //IF WE ARE HERE, IT MEANS WE GOT A MESSAGE FROM THE PLAYER AND HAVE IT IN msg
                //PARSE IT AND FORWARD IT TO THE SERVER
                let msg_text = msg.into_text().expect("Could not convert message to text");
                let msg_str = msg_text.as_str();
                let player_msg: NetIncomingMessage = match serde_json::from_str::<NetIncomingMessage>(msg_str) {
                    Ok(msg) => msg,
                    Err(e) => { eprintln!("Failed to deser message from player: {} (message = {})", e, msg_str); continue; }
                };
                println!("Player msg: {:?}", player_msg);
                server_rec_pipe.send((name.clone(), player_msg)).expect("Could not forward player message to server");
            }
        });
    }
}
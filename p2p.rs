use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const GOSSIP_PORT: u16 = 55555;
const GOSSIP_INTERVAL_SEC: u64 = 10;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GossipMessage {
    agent_id: String,
    status: String,
    can_reach_internet: bool,
}

pub struct MeshNode {
    agent_id: String,
    socket: UdpSocket,
    neighbors: Arc<Mutex<Vec<String>>>,
}

impl MeshNode {
    pub fn new(agent_id: String) -> Self {
        // Bind to 0.0.0.0 to listen on all interfaces
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", GOSSIP_PORT))
            .unwrap_or_else(|_| UdpSocket::bind("0.0.0.0:0").unwrap()); // Fallback if port busy

        socket.set_broadcast(true).expect("Set broadcast failed");
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .ok();

        MeshNode {
            agent_id,
            socket,
            neighbors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_gossip(&self) {
        let socket = self.socket.try_clone().expect("Clone failed");
        let agent_id = self.agent_id.clone();

        // Spawn Transmitter Thread
        thread::spawn(move || {
            loop {
                let msg = GossipMessage {
                    agent_id: agent_id.clone(),
                    status: "ACTIVE".to_string(),
                    can_reach_internet: true, // Logic to check this later
                };

                let json_msg = serde_json::to_string(&msg).unwrap();
                // Broadcast to local subnet
                socket
                    .send_to(
                        json_msg.as_bytes(),
                        format!("255.255.255.255:{}", GOSSIP_PORT),
                    )
                    .ok();

                thread::sleep(Duration::from_secs(GOSSIP_INTERVAL_SEC));
            }
        });
    }

    pub fn start_listener(&self) {
        let socket = self.socket.try_clone().expect("Clone failed");
        let neighbors = self.neighbors.clone();
        let my_id = self.agent_id.clone();

        // Spawn Receiver Thread
        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((amt, _src)) => {
                        let msg_str = String::from_utf8_lossy(&buf[..amt]);
                        if let Ok(msg) = serde_json::from_str::<GossipMessage>(&msg_str) {
                            if msg.agent_id != my_id {
                                let mut n = neighbors.lock().unwrap();
                                if !n.contains(&msg.agent_id) {
                                    println!("[+] New Neighbor Found: {}", msg.agent_id);
                                    n.push(msg.agent_id);
                                }
                            }
                        }
                    }
                    Err(_) => {} // Timeout or error, ignore
                }
            }
        });
    }

    pub fn get_neighbors(&self) -> Vec<String> {
        let n = self.neighbors.lock().unwrap();
        n.clone()
    }
}

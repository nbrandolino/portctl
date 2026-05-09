// Volume actions: list, inspect, create, remove, prune
use crate::client::PortainerClient;
use urlencoding::encode;

fn fmt_size(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    }
}

pub fn list(endpoint_id: u32) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/volumes", endpoint_id);
    match client.get(&path) {
        Ok(data) => {
            if crate::utils::json_output() {
                crate::utils::print_json(data["Volumes"].as_array()
                    .map(|_| &data["Volumes"])
                    .unwrap_or(&serde_json::json!([])));
                return;
            }
            let volumes = match data["Volumes"].as_array() {
                Some(arr) => arr,
                None => {
                    println!("No volumes found.");
                    return;
                }
            };

            if volumes.is_empty() {
                println!("No volumes found.");
                return;
            }

            println!("{:<30} {:<12} {}", "NAME", "DRIVER", "MOUNTPOINT");
            println!("{}", "-".repeat(80));

            for v in volumes {
                let name = v["Name"].as_str().unwrap_or("(unknown)");
                let driver = v["Driver"].as_str().unwrap_or("(unknown)");
                let mountpoint = v["Mountpoint"].as_str().unwrap_or("(unknown)");
                println!("{:<30} {:<12} {}", name, driver, mountpoint);
            }
        }
        Err(e) => {
            eprintln!("Failed to list volumes: {e}");
            std::process::exit(1);
        }
    }
}

pub fn inspect(endpoint_id: u32, name: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/volumes/{}", endpoint_id, encode(name));
    match client.get(&path) {
        Ok(v) => {
            if crate::utils::json_output() {
                crate::utils::print_json(&v);
                return;
            }
            println!("Name:       {}", v["Name"].as_str().unwrap_or("(unknown)"));
            println!("Driver:     {}", v["Driver"].as_str().unwrap_or("(unknown)"));
            println!("Scope:      {}", v["Scope"].as_str().unwrap_or("(unknown)"));
            println!("Mountpoint: {}", v["Mountpoint"].as_str().unwrap_or("(unknown)"));
            println!("Created:    {}", v["CreatedAt"].as_str().unwrap_or("(unknown)"));

            if let Some(labels) = v["Labels"].as_object() {
                if !labels.is_empty() {
                    println!("Labels:");
                    for (k, val) in labels {
                        println!("  {}={}", k, val.as_str().unwrap_or(""));
                    }
                }
            }

            if let Some(opts) = v["Options"].as_object() {
                if !opts.is_empty() {
                    println!("Options:");
                    for (k, val) in opts {
                        println!("  {}={}", k, val.as_str().unwrap_or(""));
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to inspect volume: {e}");
            std::process::exit(1);
        }
    }
}

pub fn create(endpoint_id: u32, name: &str, driver: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/volumes/create", endpoint_id);
    let body = serde_json::json!({
        "Name": name,
        "Driver": driver,
    });
    match client.post(&path, body) {
        Ok(_) => println!("Volume {name} created."),
        Err(e) => {
            eprintln!("Failed to create volume: {e}");
            std::process::exit(1);
        }
    }
}

pub fn prune(endpoint_id: u32) {
    let client = PortainerClient::new();
    // Filter param is URL-encoded JSON: {"all":["true"]} — prunes all unused volumes, not just anonymous ones
    let path = format!("endpoints/{}/docker/volumes/prune?filters=%7B%22all%22%3A%5B%22true%22%5D%7D", endpoint_id);
    match client.post(&path, serde_json::json!({})) {
        Ok(data) => {
            let reclaimed = data["SpaceReclaimed"].as_u64().unwrap_or(0);
            let count = data["VolumesDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
            println!("Removed {} volume(s), reclaimed {}.", count, fmt_size(reclaimed));
        }
        Err(e) => {
            eprintln!("Failed to prune volumes: {e}");
            std::process::exit(1);
        }
    }
}

pub fn remove(endpoint_id: u32, name: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/volumes/{}", endpoint_id, encode(name));
    match client.delete(&path) {
        Ok(()) => println!("Volume {name} removed."),
        Err(e) => {
            eprintln!("Failed to remove volume: {e}");
            std::process::exit(1);
        }
    }
}

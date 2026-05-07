use crate::client::PortainerClient;

fn resolve_id(endpoint_id: u32, name: &str) -> String {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/networks", endpoint_id);
    match client.get(&path) {
        Ok(data) => {
            let networks = match data.as_array() {
                Some(arr) => arr,
                None => {
                    eprintln!("Unexpected response format.");
                    std::process::exit(1);
                }
            };
            networks
                .iter()
                .find(|n| n["Name"].as_str().unwrap_or("") == name)
                .and_then(|n| n["Id"].as_str())
                .map(|id| id.to_string())
                .unwrap_or_else(|| {
                    eprintln!("Error: no network named '{name}' found. Run `portctl network ls` to see available networks.");
                    std::process::exit(1);
                })
        }
        Err(e) => {
            eprintln!("Failed to resolve network: {e}");
            std::process::exit(1);
        }
    }
}

pub fn list(endpoint_id: u32) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/networks", endpoint_id);
    match client.get(&path) {
        Ok(data) => {
            if crate::utils::json_output() {
                crate::utils::print_json(&data);
                return;
            }

            let networks = match data.as_array() {
                Some(arr) => arr,
                None => {
                    eprintln!("Unexpected response format.");
                    std::process::exit(1);
                }
            };

            if networks.is_empty() {
                println!("No networks found.");
                return;
            }

            println!("{:<16} {:<30} {:<12} {}", "ID", "NAME", "DRIVER", "SCOPE");
            println!("{}", "-".repeat(72));

            for n in networks {
                let id = n["Id"].as_str().unwrap_or("").chars().take(12).collect::<String>();
                let name = n["Name"].as_str().unwrap_or("(unknown)");
                let driver = n["Driver"].as_str().unwrap_or("(unknown)");
                let scope = n["Scope"].as_str().unwrap_or("(unknown)");
                println!("{:<16} {:<30} {:<12} {}", id, name, driver, scope);
            }
        }
        Err(e) => {
            eprintln!("Failed to list networks: {e}");
            std::process::exit(1);
        }
    }
}

pub fn inspect(endpoint_id: u32, name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(endpoint_id, name);
    let path = format!("endpoints/{}/docker/networks/{}", endpoint_id, id);
    match client.get(&path) {
        Ok(n) => {
            if crate::utils::json_output() {
                crate::utils::print_json(&n);
                return;
            }
            let short_id = n["Id"].as_str().unwrap_or("").chars().take(12).collect::<String>();
            println!("ID:       {}", short_id);
            println!("Name:     {}", n["Name"].as_str().unwrap_or("(unknown)"));
            println!("Driver:   {}", n["Driver"].as_str().unwrap_or("(unknown)"));
            println!("Scope:    {}", n["Scope"].as_str().unwrap_or("(unknown)"));
            println!("IPv6:     {}", n["EnableIPv6"].as_bool().unwrap_or(false));
            println!("Internal: {}", n["Internal"].as_bool().unwrap_or(false));

            if let Some(configs) = n["IPAM"]["Config"].as_array() {
                for cfg in configs {
                    if let Some(subnet) = cfg["Subnet"].as_str() {
                        println!("Subnet:   {}", subnet);
                    }
                    if let Some(gw) = cfg["Gateway"].as_str() {
                        println!("Gateway:  {}", gw);
                    }
                }
            }

            if let Some(containers) = n["Containers"].as_object() {
                if !containers.is_empty() {
                    println!("Containers:");
                    for (_, c) in containers {
                        let cname = c["Name"].as_str().unwrap_or("(unknown)");
                        let ipv4 = c["IPv4Address"].as_str().unwrap_or("(none)");
                        println!("  {} ({})", cname, ipv4);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to inspect network: {e}");
            std::process::exit(1);
        }
    }
}

pub fn create(endpoint_id: u32, name: &str, driver: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/networks/create", endpoint_id);
    let body = serde_json::json!({
        "Name": name,
        "Driver": driver,
        "CheckDuplicate": true,
    });
    match client.post(&path, body) {
        Ok(_) => println!("Network {name} created."),
        Err(e) => {
            eprintln!("Failed to create network: {e}");
            std::process::exit(1);
        }
    }
}

pub fn prune(endpoint_id: u32) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/networks/prune", endpoint_id);
    match client.post(&path, serde_json::json!({})) {
        Ok(data) => {
            let count = data["NetworksDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
            println!("Removed {} network(s).", count);
        }
        Err(e) => {
            eprintln!("Failed to prune networks: {e}");
            std::process::exit(1);
        }
    }
}

pub fn remove(endpoint_id: u32, name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(endpoint_id, name);
    let path = format!("endpoints/{}/docker/networks/{}", endpoint_id, id);
    match client.delete(&path) {
        Ok(()) => println!("Network {name} removed."),
        Err(e) => {
            eprintln!("Failed to remove network: {e}");
            std::process::exit(1);
        }
    }
}

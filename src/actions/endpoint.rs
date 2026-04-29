use crate::client::PortainerClient;

pub fn resolve_id(name: &str) -> u32 {
    let client = PortainerClient::new();
    match client.get("endpoints") {
        Ok(data) => {
            let endpoints = match data.as_array() {
                Some(arr) => arr,
                None => {
                    eprintln!("Unexpected response format.");
                    std::process::exit(1);
                }
            };
            endpoints
                .iter()
                .find(|ep| ep["Name"].as_str().unwrap_or("") == name)
                .and_then(|ep| ep["Id"].as_u64())
                .and_then(|id| u32::try_from(id).ok())
                .unwrap_or_else(|| {
                    eprintln!("Error: no endpoint named '{name}' found. Run `portctl endpoint ls` to see available endpoints.");
                    std::process::exit(1);
                })
        }
        Err(e) => {
            eprintln!("Failed to resolve endpoint: {e}");
            std::process::exit(1);
        }
    }
}

pub fn inspect(name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(name);
    let path = format!("endpoints/{}", id);
    match client.get(&path) {
        Ok(ep) => {
            let ep_type = match ep["Type"].as_u64().unwrap_or(0) {
                1 => "Docker",
                2 => "Agent",
                3 => "Azure",
                4 => "Edge Agent",
                5 => "Local",
                6 => "Kubernetes",
                _ => "Unknown",
            };
            let status = match ep["Status"].as_u64().unwrap_or(0) {
                1 => "up",
                2 => "down",
                _ => "unknown",
            };

            println!("ID:           {}", id);
            println!("Name:         {}", ep["Name"].as_str().unwrap_or("(unknown)"));
            println!("Type:         {}", ep_type);
            println!("URL:          {}", ep["URL"].as_str().unwrap_or("(unknown)"));
            println!("Status:       {}", status);

            let docker = &ep["Snapshots"];
            if let Some(snapshot) = docker.as_array().and_then(|a| a.first()) {
                println!("Docker:       {}", snapshot["DockerVersion"].as_str().unwrap_or("(unknown)"));
                println!("Containers:   {} running / {} stopped / {} total",
                    snapshot["RunningContainerCount"].as_u64().unwrap_or(0),
                    snapshot["StoppedContainerCount"].as_u64().unwrap_or(0),
                    snapshot["TotalContainerCount"].as_u64().unwrap_or(0),
                );
                println!("Images:       {}", snapshot["ImageCount"].as_u64().unwrap_or(0));
                println!("Volumes:      {}", snapshot["VolumeCount"].as_u64().unwrap_or(0));
                println!("Stacks:       {}", snapshot["StackCount"].as_u64().unwrap_or(0));
            }
        }
        Err(e) => {
            eprintln!("Failed to inspect endpoint: {e}");
            std::process::exit(1);
        }
    }
}

pub fn list() {
    let client = PortainerClient::new();
    match client.get("endpoints") {
        Ok(data) => {
            let endpoints = match data.as_array() {
                Some(arr) => arr,
                None => {
                    eprintln!("Unexpected response format.");
                    std::process::exit(1);
                }
            };

            if endpoints.is_empty() {
                println!("No endpoints found.");
                return;
            }

            println!("{:<30} {:<15} {}", "NAME", "TYPE", "URL");
            println!("{}", "-".repeat(64));

            for ep in endpoints {
                let name = ep["Name"].as_str().unwrap_or("(unknown)");
                let url = ep["URL"].as_str().unwrap_or("(unknown)");
                let ep_type = match ep["Type"].as_u64().unwrap_or(0) {
                    1 => "Docker",
                    2 => "Agent",
                    3 => "Azure",
                    4 => "Edge Agent",
                    5 => "Local",
                    6 => "Kubernetes",
                    _ => "Unknown",
                };

                println!("{:<30} {:<15} {}", name, ep_type, url);
            }
        }
        Err(e) => {
            eprintln!("Failed to list endpoints: {e}");
            std::process::exit(1);
        }
    }
}

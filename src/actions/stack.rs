use crate::client::PortainerClient;

pub fn resolve_id(name: &str) -> u32 {
    let client = PortainerClient::new();
    match client.get("stacks") {
        Ok(data) => {
            let stacks = match data.as_array() {
                Some(arr) => arr,
                None => {
                    eprintln!("Unexpected response format.");
                    std::process::exit(1);
                }
            };
            stacks
                .iter()
                .find(|s| s["Name"].as_str().unwrap_or("") == name)
                .and_then(|s| s["Id"].as_u64())
                .map(|id| id as u32)
                .unwrap_or_else(|| {
                    eprintln!("Error: no stack named '{name}' found. Run `portctl stack ls` to see available stacks.");
                    std::process::exit(1);
                })
        }
        Err(e) => {
            eprintln!("Failed to resolve stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn list(endpoint_filter: Option<&str>) {
    let client = PortainerClient::new();

    let endpoint_id = endpoint_filter.map(|name| {
        crate::actions::endpoint::resolve_id(name)
    });

    match client.get("stacks") {
        Ok(data) => {
            let all_stacks = match data.as_array() {
                Some(arr) => arr,
                None => {
                    eprintln!("Unexpected response format.");
                    std::process::exit(1);
                }
            };

            let stacks: Vec<&serde_json::Value> = all_stacks
                .iter()
                .filter(|s| {
                    if let Some(eid) = endpoint_id {
                        s["EndpointId"].as_u64().unwrap_or(0) == eid as u64
                    } else {
                        true
                    }
                })
                .collect();

            if stacks.is_empty() {
                println!("No stacks found.");
                return;
            }

            println!("{:<35} {:<12} {}", "NAME", "STATUS", "TYPE");
            println!("{}", "-".repeat(60));

            for s in stacks {
                let name = s["Name"].as_str().unwrap_or("(unknown)");
                let status = match s["Status"].as_u64().unwrap_or(0) {
                    1 => "active",
                    2 => "inactive",
                    _ => "unknown",
                };
                let stack_type = match s["Type"].as_u64().unwrap_or(0) {
                    1 => "Swarm",
                    2 => "Compose",
                    3 => "Kubernetes",
                    _ => "Unknown",
                };

                println!("{:<35} {:<12} {}", name, status, stack_type);
            }
        }
        Err(e) => {
            eprintln!("Failed to list stacks: {e}");
            std::process::exit(1);
        }
    }
}

pub fn inspect(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);
    let path = format!("stacks/{}", id);
    match client.get(&path) {
        Ok(s) => {
            let name = s["Name"].as_str().unwrap_or("(unknown)");
            let status = match s["Status"].as_u64().unwrap_or(0) {
                1 => "active",
                2 => "inactive",
                _ => "unknown",
            };
            let stack_type = match s["Type"].as_u64().unwrap_or(0) {
                1 => "Swarm",
                2 => "Compose",
                3 => "Kubernetes",
                _ => "Unknown",
            };
            let endpoint_id = s["EndpointId"].as_u64().unwrap_or(0);
            let created_by = s["CreatedBy"].as_str().unwrap_or("(unknown)");
            let updated_by = s["UpdatedBy"].as_str().unwrap_or("(unknown)");

            println!("ID:          {}", id);
            println!("Name:        {}", name);
            println!("Status:      {}", status);
            println!("Type:        {}", stack_type);
            println!("Endpoint ID: {}", endpoint_id);
            println!("Created by:  {}", created_by);
            println!("Updated by:  {}", updated_by);

            if let Some(envs) = s["Env"].as_array() {
                let env_list: Vec<String> = envs
                    .iter()
                    .filter_map(|e| {
                        let k = e["name"].as_str()?;
                        let v = e["value"].as_str()?;
                        Some(format!("{}={}", k, v))
                    })
                    .collect();
                if !env_list.is_empty() {
                    println!("Env:");
                    for e in env_list {
                        println!("  {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to inspect stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn start(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);
    let path = format!("stacks/{}/start", id);
    match client.post_empty(&path) {
        Ok(()) => println!("Stack {stack_name} started."),
        Err(e) => {
            eprintln!("Failed to start stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn stop(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);
    let path = format!("stacks/{}/stop", id);
    match client.post_empty(&path) {
        Ok(()) => println!("Stack {stack_name} stopped."),
        Err(e) => {
            eprintln!("Failed to stop stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn update(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);

    let stack = match client.get(&format!("stacks/{}", id)) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to fetch stack details: {e}");
            std::process::exit(1);
        }
    };

    if stack["GitConfig"].is_null() {
        eprintln!("Error: stack '{stack_name}' is not deployed from a git repository.");
        std::process::exit(1);
    }

    let endpoint_id = stack["EndpointId"].as_u64().unwrap_or_else(|| {
        eprintln!("Error: could not determine endpoint ID for stack '{stack_name}'.");
        std::process::exit(1);
    });

    let path = format!("stacks/{}/git/redeploy?endpointId={}", id, endpoint_id);
    let body = serde_json::json!({ "pullImage": true });

    match client.put(&path, body) {
        Ok(_) => println!("Stack {stack_name} pulled and redeployed."),
        Err(e) => {
            eprintln!("Failed to update stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn remove(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);
    let path = format!("stacks/{}", id);
    match client.delete(&path) {
        Ok(()) => println!("Stack {stack_name} removed."),
        Err(e) => {
            eprintln!("Failed to remove stack: {e}");
            std::process::exit(1);
        }
    }
}

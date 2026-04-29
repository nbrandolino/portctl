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
                .and_then(|id| u32::try_from(id).ok())
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

    // Fetch Portainer-managed stacks
    let portainer_stacks = match client.get("stacks") {
        Ok(data) => data.as_array().cloned().unwrap_or_default(),
        Err(e) => {
            eprintln!("Failed to list stacks: {e}");
            std::process::exit(1);
        }
    };

    // Collect the names of Portainer-managed stacks so we can exclude them
    // when scanning Docker labels for external stacks
    let managed_names: std::collections::HashSet<String> = portainer_stacks
        .iter()
        .filter_map(|s| s["Name"].as_str().map(|n| n.to_string()))
        .collect();

    // Determine which endpoints to scan for external stacks
    let endpoints: Vec<(u32, String)> = if let Some(eid) = endpoint_id {
        vec![(eid, endpoint_filter.unwrap_or("").to_string())]
    } else {
        match client.get("endpoints") {
            Ok(data) => data.as_array().unwrap_or(&vec![]).iter().filter_map(|ep| {
                let id = u32::try_from(ep["Id"].as_u64()?).ok()?;
                let name = ep["Name"].as_str().unwrap_or("").to_string();
                Some((id, name))
            }).collect(),
            Err(_) => vec![],
        }
    };

    // Find external compose projects by reading container labels
    let mut external: Vec<(String, String, String)> = vec![];
    for (eid, ep_name) in &endpoints {
        let path = format!("endpoints/{}/docker/containers/json?all=1", eid);
        let containers = match client.get(&path) {
            Ok(data) => data.as_array().cloned().unwrap_or_default(),
            Err(_) => continue,
        };

        let mut seen = std::collections::HashSet::new();
        for c in &containers {
            let project = match c["Labels"]["com.docker.compose.project"].as_str() {
                Some(p) => p.to_string(),
                None => continue,
            };
            if managed_names.contains(&project) || seen.contains(&project) {
                continue;
            }
            seen.insert(project.clone());

            // Infer status from whether any container in the project is running
            let is_running = containers.iter().any(|other| {
                other["Labels"]["com.docker.compose.project"].as_str() == Some(&project)
                    && other["State"].as_str() == Some("running")
            });
            let status = if is_running { "active" } else { "inactive" };
            external.push((project, status.to_string(), ep_name.clone()));
        }
    }

    let filtered_stacks: Vec<&serde_json::Value> = portainer_stacks
        .iter()
        .filter(|s| {
            if let Some(eid) = endpoint_id {
                s["EndpointId"].as_u64().unwrap_or(0) == eid as u64
            } else {
                true
            }
        })
        .collect();

    if filtered_stacks.is_empty() && external.is_empty() {
        println!("No stacks found.");
        return;
    }

    println!("{:<35} {:<12} {:<12} {}", "NAME", "STATUS", "TYPE", "ENDPOINT");
    println!("{}", "-".repeat(76));

    for s in filtered_stacks {
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
        // Look up endpoint name for this stack
        let ep_name = endpoints.iter()
            .find(|(id, _)| *id as u64 == s["EndpointId"].as_u64().unwrap_or(0))
            .map(|(_, name)| name.as_str())
            .unwrap_or("(unknown)");

        println!("{:<35} {:<12} {:<12} {}", name, status, stack_type, ep_name);
    }

    for (name, status, ep_name) in &external {
        println!("{:<35} {:<12} {:<12} {}", name, status, "External", ep_name);
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

    // Image pulls and redeployment can take arbitrarily long, so use no timeout.
    let deploy_client = PortainerClient::new_no_timeout();
    match deploy_client.put(&path, body) {
        Ok(_) => println!("Stack {stack_name} pulled and redeployed."),
        Err(e) => {
            eprintln!("Failed to update stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn deploy_from_file(name: &str, endpoint_name: &str, file_path: &str) {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read file '{file_path}': {e}");
            std::process::exit(1);
        }
    };

    let eid = crate::actions::endpoint::resolve_id(endpoint_name);
    let client = PortainerClient::new();
    let path = format!("stacks?type=2&method=string&endpointId={}", eid);
    let body = serde_json::json!({
        "Name": name,
        "StackFileContent": content,
    });

    match client.post(&path, body) {
        Ok(_) => println!("Stack '{name}' deployed."),
        Err(e) => {
            eprintln!("Failed to deploy stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn deploy_from_git(
    name: &str,
    endpoint_name: &str,
    git_url: &str,
    git_ref: &str,
    compose_file: &str,
    credentials: Option<(&str, &str)>,
) {
    let eid = crate::actions::endpoint::resolve_id(endpoint_name);
    let client = PortainerClient::new();
    let path = format!("stacks?type=2&method=repository&endpointId={}", eid);

    let mut body = serde_json::json!({
        "Name": name,
        "RepositoryURL": git_url,
        "RepositoryReferenceName": git_ref,
        "ComposeFile": compose_file,
        "RepositoryAuthentication": credentials.is_some(),
    });

    if let Some((username, password)) = credentials {
        body["RepositoryUsername"] = serde_json::Value::String(username.to_string());
        body["RepositoryPassword"] = serde_json::Value::String(password.to_string());
    }

    match client.post(&path, body) {
        Ok(_) => println!("Stack '{name}' deployed from git ({git_url} @ {git_ref})."),
        Err(e) => {
            eprintln!("Failed to deploy stack from git: {e}");
            std::process::exit(1);
        }
    }
}

pub fn edit(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);

    let stack = match client.get(&format!("stacks/{}", id)) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to fetch stack details: {e}");
            std::process::exit(1);
        }
    };

    if !stack["GitConfig"].is_null() {
        eprintln!(
            "Error: '{stack_name}' is deployed from a git repository. \
             Edit the source files in git and run `portctl stack update {stack_name}` to redeploy."
        );
        std::process::exit(1);
    }

    let endpoint_id = stack["EndpointId"].as_u64().unwrap_or_else(|| {
        eprintln!("Error: could not determine endpoint ID for stack '{stack_name}'.");
        std::process::exit(1);
    });

    let env = stack["Env"].as_array().cloned().unwrap_or_default();

    // Fetch current compose content
    let original = match client.get(&format!("stacks/{}/file", id)) {
        Ok(data) => data["StackFileContent"].as_str().unwrap_or("").to_string(),
        Err(e) => {
            eprintln!("Failed to fetch compose file: {e}");
            std::process::exit(1);
        }
    };

    // Write to a securely-created temp file with .yml suffix for editor syntax highlighting
    let mut tmp = match tempfile::Builder::new().suffix(".yml").tempfile() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create temp file: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = std::io::Write::write_all(&mut tmp, original.as_bytes()) {
        eprintln!("Failed to write temp file: {e}");
        std::process::exit(1);
    }

    let tmp_path = tmp.path().to_path_buf();

    // Open in $VISUAL / $EDITOR / vi
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let exit_ok = match std::process::Command::new(&editor).arg(&tmp_path).status() {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("Failed to open editor '{editor}': {e}");
            std::process::exit(1);
        }
    };

    if !exit_ok {
        eprintln!("Editor exited with a non-zero status. No changes applied.");
        std::process::exit(1);
    }

    // Read modified content; tmp auto-deletes on drop
    let modified = match std::fs::read_to_string(&tmp_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read temp file: {e}");
            std::process::exit(1);
        }
    };

    if modified == original {
        println!("No changes made.");
        return;
    }

    // Push updated compose file back to Portainer
    let path = format!("stacks/{}?endpointId={}", id, endpoint_id);
    let body = serde_json::json!({
        "StackFileContent": modified,
        "Env": env,
    });

    match client.put(&path, body) {
        Ok(_) => println!("Stack '{stack_name}' updated."),
        Err(e) => {
            eprintln!("Failed to update stack: {e}");
            std::process::exit(1);
        }
    }
}

pub fn compose(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);
    let path = format!("stacks/{}/file", id);
    match client.get(&path) {
        Ok(data) => {
            let content = data["StackFileContent"].as_str().unwrap_or("");
            print!("{}", content);
        }
        Err(e) => {
            eprintln!("Failed to fetch compose file: {e}");
            std::process::exit(1);
        }
    }
}

pub fn remove(stack_name: &str) {
    let client = PortainerClient::new();
    let id = resolve_id(stack_name);

    let stack = match client.get(&format!("stacks/{}", id)) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to fetch stack details: {e}");
            std::process::exit(1);
        }
    };

    let endpoint_id = stack["EndpointId"].as_u64().unwrap_or_else(|| {
        eprintln!("Error: could not determine endpoint ID for stack '{stack_name}'.");
        std::process::exit(1);
    });

    let path = format!("stacks/{}?endpointId={}", id, endpoint_id);
    match client.delete(&path) {
        Ok(()) => println!("Stack {stack_name} removed."),
        Err(e) => {
            eprintln!("Failed to remove stack: {e}");
            std::process::exit(1);
        }
    }
}

use crate::client::PortainerClient;

pub fn list(endpoint_filter: Option<&str>) {
    if let Some(name) = endpoint_filter {
        let client = PortainerClient::new();
        let eid = crate::actions::endpoint::resolve_id(name);
        println!("{:<16} {:<35} {:<12} {}", "ID", "NAME", "STATE", "IMAGE");
        println!("{}", "-".repeat(80));
        list_for_endpoint(&client, eid, None);
    } else {
        list_all();
    }
}

fn list_all() {
    let client = PortainerClient::new();

    let endpoints = match client.get("endpoints") {
        Ok(data) => match data.as_array() {
            Some(arr) => arr.to_owned(),
            None => {
                eprintln!("Unexpected response format.");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch endpoints: {e}");
            std::process::exit(1);
        }
    };

    println!("{:<16} {:<25} {:<12} {:<12} {}", "ID", "NAME", "STATE", "ENDPOINT", "IMAGE");
    println!("{}", "-".repeat(90));

    for ep in &endpoints {
        let eid = match ep["Id"].as_u64() {
            Some(id) => id as u32,
            None => continue,
        };
        let ep_name = ep["Name"].as_str().unwrap_or("(unknown)").to_string();
        list_for_endpoint(&client, eid, Some(&ep_name));
    }
}

fn list_for_endpoint(client: &PortainerClient, endpoint_id: u32, endpoint_name: Option<&str>) {
    let path = format!("endpoints/{}/docker/containers/json?all=1", endpoint_id);
    match client.get(&path) {
        Ok(data) => {
            let containers = match data.as_array() {
                Some(arr) => arr,
                None => return,
            };

            for c in containers {
                let id = c["Id"].as_str().unwrap_or("").chars().take(12).collect::<String>();
                let name = c["Names"]
                    .as_array()
                    .and_then(|n| n.first())
                    .and_then(|n| n.as_str())
                    .unwrap_or("(unknown)")
                    .trim_start_matches('/');
                let state = c["State"].as_str().unwrap_or("(unknown)");
                let image = c["Image"].as_str().unwrap_or("(unknown)");

                if let Some(ep_name) = endpoint_name {
                    println!("{:<16} {:<25} {:<12} {:<12} {}", id, name, state, ep_name, image);
                } else {
                    println!("{:<16} {:<35} {:<12} {}", id, name, state, image);
                }
            }
        }
        Err(e) => {
            let label = endpoint_name.unwrap_or("endpoint");
            eprintln!("Warning: failed to list containers for {label}: {e}");
        }
    }
}

pub fn logs(endpoint_id: u32, container_id: &str, tail: u32, timestamps: bool, follow: bool) {
    use std::io::Read;

    let client = PortainerClient::new();
    let path = format!(
        "endpoints/{}/docker/containers/{}/logs?stdout=1&stderr=1&tail={}&timestamps={}&follow={}",
        endpoint_id, container_id, tail,
        if timestamps { 1 } else { 0 },
        if follow { 1 } else { 0 },
    );

    let response = match client.get_response(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to fetch logs: {e}");
            std::process::exit(1);
        }
    };

    // Docker multiplexes stdout/stderr into a single stream with an 8-byte header per chunk:
    //   byte 0:   stream type (1 = stdout, 2 = stderr)
    //   bytes 1-3: padding (zeros)
    //   bytes 4-7: payload length (big-endian u32)
    let mut stream = response;
    let mut header = [0u8; 8];

    loop {
        match stream.read_exact(&mut header) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("Error reading log stream: {e}");
                std::process::exit(1);
            }
        }

        let payload_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let mut payload = vec![0u8; payload_len];

        match stream.read_exact(&mut payload) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("Error reading log stream: {e}");
                std::process::exit(1);
            }
        }

        if let Ok(text) = std::str::from_utf8(&payload) {
            print!("{}", text);
        }
    }
}

pub fn start(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/start", endpoint_id, container_id);
    match client.post_empty(&path) {
        Ok(()) => println!("Container {container_id} started."),
        Err(e) => {
            eprintln!("Failed to start container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn stop(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/stop", endpoint_id, container_id);
    match client.post_empty(&path) {
        Ok(()) => println!("Container {container_id} stopped."),
        Err(e) => {
            eprintln!("Failed to stop container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn restart(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/restart", endpoint_id, container_id);
    match client.post_empty(&path) {
        Ok(()) => println!("Container {container_id} restarted."),
        Err(e) => {
            eprintln!("Failed to restart container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn inspect(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/json", endpoint_id, container_id);
    match client.get(&path) {
        Ok(data) => {
            let id = data["Id"].as_str().unwrap_or("(unknown)").chars().take(12).collect::<String>();
            let name = data["Name"].as_str().unwrap_or("(unknown)").trim_start_matches('/');
            let image = data["Config"]["Image"].as_str().unwrap_or("(unknown)");
            let state = data["State"]["Status"].as_str().unwrap_or("(unknown)");
            let started = data["State"]["StartedAt"].as_str().unwrap_or("(unknown)");
            let created = data["Created"].as_str().unwrap_or("(unknown)");

            println!("ID:       {}", id);
            println!("Name:     {}", name);
            println!("Image:    {}", image);
            println!("State:    {}", state);
            println!("Created:  {}", created);
            println!("Started:  {}", started);

            if let Some(ports) = data["NetworkSettings"]["Ports"].as_object() {
                let bindings: Vec<String> = ports
                    .iter()
                    .filter_map(|(container_port, bindings)| {
                        bindings.as_array()?.iter().find_map(|b| {
                            let host_port = b["HostPort"].as_str()?;
                            Some(format!("{}:{}", host_port, container_port))
                        })
                    })
                    .collect();

                if !bindings.is_empty() {
                    println!("Ports:    {}", bindings.join(", "));
                }
            }

            if let Some(envs) = data["Config"]["Env"].as_array() {
                let env_list: Vec<&str> = envs.iter().filter_map(|e| e.as_str()).collect();
                if !env_list.is_empty() {
                    println!("Env:");
                    for e in env_list {
                        println!("  {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to inspect container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn remove(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}", endpoint_id, container_id);
    match client.delete(&path) {
        Ok(()) => println!("Container {container_id} removed."),
        Err(e) => {
            eprintln!("Failed to remove container: {e}");
            std::process::exit(1);
        }
    }
}

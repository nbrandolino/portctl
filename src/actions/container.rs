use crate::client::PortainerClient;
use urlencoding::encode;

pub fn list(endpoint_filter: Option<&str>) {
    if let Some(name) = endpoint_filter {
        let client = PortainerClient::new();
        let eid = crate::actions::endpoint::resolve_id(name);
        println!("{:<16} {:<35} {:<12} {}", "ID", "NAME", "STATE", "IMAGE");
        println!("{}", "-".repeat(80));
        let count = list_for_endpoint(&client, eid, None);
        if count == 0 {
            println!("No containers found.");
        }
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

    let total: usize = endpoints.iter().filter_map(|ep| {
        let eid = u32::try_from(ep["Id"].as_u64()?).ok()?;
        let ep_name = ep["Name"].as_str().unwrap_or("(unknown)").to_string();
        Some(list_for_endpoint(&client, eid, Some(&ep_name)))
    }).sum();

    if total == 0 {
        println!("No containers found.");
    }
}

fn list_for_endpoint(client: &PortainerClient, endpoint_id: u32, endpoint_name: Option<&str>) -> usize {
    let path = format!("endpoints/{}/docker/containers/json?all=1", endpoint_id);
    match client.get(&path) {
        Ok(data) => {
            let containers = match data.as_array() {
                Some(arr) => arr,
                None => return 0,
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

            containers.len()
        }
        Err(e) => {
            let label = endpoint_name.unwrap_or("endpoint");
            eprintln!("Warning: failed to list containers for {label}: {e}");
            0
        }
    }
}

pub fn logs(endpoint_id: u32, container_id: &str, tail: u32, timestamps: bool, follow: bool) {
    let client = if follow {
        PortainerClient::new_no_timeout()
    } else {
        PortainerClient::new()
    };
    let path = format!(
        "endpoints/{}/docker/containers/{}/logs?stdout=1&stderr=1&tail={}&timestamps={}&follow={}",
        endpoint_id, encode(container_id), tail,
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

    crate::utils::pipe_docker_stream(response);
}

pub fn stats(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/stats?stream=false", endpoint_id, encode(container_id));
    match client.get(&path) {
        Ok(data) => {
            let cpu_delta = data["cpu_stats"]["cpu_usage"]["total_usage"].as_u64().unwrap_or(0)
                .saturating_sub(data["precpu_stats"]["cpu_usage"]["total_usage"].as_u64().unwrap_or(0));
            let system_delta = data["cpu_stats"]["system_cpu_usage"].as_u64().unwrap_or(0)
                .saturating_sub(data["precpu_stats"]["system_cpu_usage"].as_u64().unwrap_or(0));
            let num_cpus = data["cpu_stats"]["online_cpus"].as_u64().unwrap_or(1);
            let cpu_pct = if system_delta > 0 {
                (cpu_delta as f64 / system_delta as f64) * num_cpus as f64 * 100.0
            } else {
                0.0
            };

            let mem_usage = data["memory_stats"]["usage"].as_u64().unwrap_or(0);
            let mem_cache = data["memory_stats"]["stats"]["cache"].as_u64()
                .or_else(|| data["memory_stats"]["stats"]["inactive_file"].as_u64())
                .unwrap_or(0);
            let mem_actual = mem_usage.saturating_sub(mem_cache);
            let mem_limit = data["memory_stats"]["limit"].as_u64().unwrap_or(0);
            let mem_pct = if mem_limit > 0 {
                mem_actual as f64 / mem_limit as f64 * 100.0
            } else {
                0.0
            };

            let (net_rx, net_tx) = if let Some(networks) = data["networks"].as_object() {
                networks.values().fold((0u64, 0u64), |(rx, tx), iface| (
                    rx + iface["rx_bytes"].as_u64().unwrap_or(0),
                    tx + iface["tx_bytes"].as_u64().unwrap_or(0),
                ))
            } else {
                (0, 0)
            };

            let (blk_read, blk_write) =
                if let Some(entries) = data["blkio_stats"]["io_service_bytes_recursive"].as_array() {
                    entries.iter().fold((0u64, 0u64), |(r, w), e| {
                        let val = e["value"].as_u64().unwrap_or(0);
                        match e["op"].as_str().unwrap_or("") {
                            "Read"  => (r + val, w),
                            "Write" => (r, w + val),
                            _       => (r, w),
                        }
                    })
                } else {
                    (0, 0)
                };

            println!("CPU:     {:.2}%", cpu_pct);
            println!("Memory:  {} / {} ({:.2}%)", fmt_bytes(mem_actual), fmt_bytes(mem_limit), mem_pct);
            println!("Net I/O: {} rx / {} tx", fmt_bytes(net_rx), fmt_bytes(net_tx));
            println!("Blk I/O: {} read / {} write", fmt_bytes(blk_read), fmt_bytes(blk_write));
        }
        Err(e) => {
            eprintln!("Failed to fetch stats: {e}");
            std::process::exit(1);
        }
    }
}

fn fmt_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn start(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/start", endpoint_id, encode(container_id));
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
    let path = format!("endpoints/{}/docker/containers/{}/stop", endpoint_id, encode(container_id));
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
    let path = format!("endpoints/{}/docker/containers/{}/restart", endpoint_id, encode(container_id));
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
    let path = format!("endpoints/{}/docker/containers/{}/json", endpoint_id, encode(container_id));
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

pub fn exec(endpoint_id: u32, container_id: &str, cmd: &[String]) {
    let client = PortainerClient::new_no_timeout();

    // Step 1: create the exec instance
    let create_path = format!("endpoints/{}/docker/containers/{}/exec", endpoint_id, encode(container_id));
    let create_body = serde_json::json!({
        "AttachStdin": false,
        "AttachStdout": true,
        "AttachStderr": true,
        "Tty": false,
        "Cmd": cmd,
    });

    let exec_id = match client.post(&create_path, create_body) {
        Ok(data) => match data["Id"].as_str() {
            Some(id) => id.to_string(),
            None => {
                eprintln!("Failed to get exec ID from response.");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to create exec instance: {e}");
            std::process::exit(1);
        }
    };

    // Step 2: start the exec instance and stream output
    let start_path = format!("endpoints/{}/docker/exec/{}/start", endpoint_id, exec_id);
    let start_body = serde_json::json!({ "Detach": false, "Tty": false });

    let stream = match client.post_response(&start_path, start_body) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to start exec: {e}");
            std::process::exit(1);
        }
    };

    crate::utils::pipe_docker_stream(stream);

    // Step 3: retrieve the exit code and propagate it
    let inspect_path = format!("endpoints/{}/docker/exec/{}/json", endpoint_id, exec_id);
    let exit_code = match client.get(&inspect_path) {
        Ok(data) => data["ExitCode"].as_i64().unwrap_or(0),
        Err(e) => {
            eprintln!("Warning: could not retrieve exec exit code: {e}");
            0
        }
    };
    if exit_code != 0 {
        std::process::exit(exit_code.clamp(1, 255) as i32);
    }
}

pub fn cp(endpoint_id: u32, src: &str, dest: &str) {
    // Determine direction based on which arg contains <container>:<path>
    if let Some(colon) = src.find(':') {
        let container_id = &src[..colon];
        let container_path = &src[colon + 1..];
        cp_from_container(endpoint_id, container_id, container_path, dest);
    } else if let Some(colon) = dest.find(':') {
        let container_id = &dest[..colon];
        let container_path = &dest[colon + 1..];
        cp_to_container(endpoint_id, src, container_id, container_path);
    } else {
        eprintln!("Error: one of SRC or DEST must be in <container>:<path> format.");
        std::process::exit(1);
    }
}

fn cp_from_container(endpoint_id: u32, container_id: &str, container_path: &str, local_path: &str) {
    let client = PortainerClient::new();
    let api_path = format!(
        "endpoints/{}/docker/containers/{}/archive?path={}",
        endpoint_id, encode(container_id), encode(container_path)
    );

    let bytes = match client.get_bytes(&api_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to fetch from container: {e}");
            std::process::exit(1);
        }
    };

    let local = std::path::Path::new(local_path);

    if local.is_dir() {
        // Extract into the directory, preserving the original filename(s)
        let mut ar = tar::Archive::new(std::io::Cursor::new(bytes));
        if let Err(e) = ar.unpack(local) {
            eprintln!("Failed to extract archive: {e}");
            std::process::exit(1);
        }
    } else {
        // Extract the first entry and write it to local_path (rename)
        let mut ar = tar::Archive::new(std::io::Cursor::new(bytes));
        let mut entries = match ar.entries() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to read archive: {e}");
                std::process::exit(1);
            }
        };
        match entries.next() {
            Some(Ok(mut entry)) => {
                if let Err(e) = entry.unpack(local) {
                    eprintln!("Failed to write file: {e}");
                    std::process::exit(1);
                }
            }
            Some(Err(e)) => {
                eprintln!("Failed to read archive entry: {e}");
                std::process::exit(1);
            }
            None => {
                eprintln!("Archive is empty.");
                std::process::exit(1);
            }
        }
    }

    println!("Copied {container_id}:{container_path} -> {local_path}");
}

fn cp_to_container(endpoint_id: u32, local_path: &str, container_id: &str, container_path: &str) {
    let local = std::path::Path::new(local_path);

    if !local.exists() {
        eprintln!("Error: local path '{local_path}' does not exist.");
        std::process::exit(1);
    }

    // Build tar archive in memory
    let mut ar = tar::Builder::new(Vec::new());

    if local.is_dir() {
        if let Err(e) = ar.append_dir_all(".", local) {
            eprintln!("Failed to create tar archive: {e}");
            std::process::exit(1);
        }
    } else {
        let file_name = local.file_name().unwrap_or(local.as_os_str());
        if let Err(e) = ar.append_path_with_name(local, file_name) {
            eprintln!("Failed to create tar archive: {e}");
            std::process::exit(1);
        }
    }

    let tar_bytes = match ar.into_inner() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to finalize tar archive: {e}");
            std::process::exit(1);
        }
    };

    let client = PortainerClient::new();
    let api_path = format!(
        "endpoints/{}/docker/containers/{}/archive?path={}",
        endpoint_id, encode(container_id), encode(container_path)
    );

    match client.put_raw(&api_path, tar_bytes) {
        Ok(()) => println!("Copied {local_path} -> {container_id}:{container_path}"),
        Err(e) => {
            eprintln!("Failed to copy to container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn pause(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/pause", endpoint_id, encode(container_id));
    match client.post_empty(&path) {
        Ok(()) => println!("Container {container_id} paused."),
        Err(e) => {
            eprintln!("Failed to pause container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn unpause(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/unpause", endpoint_id, encode(container_id));
    match client.post_empty(&path) {
        Ok(()) => println!("Container {container_id} unpaused."),
        Err(e) => {
            eprintln!("Failed to unpause container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn top(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}/top", endpoint_id, encode(container_id));
    match client.get(&path) {
        Ok(data) => {
            let titles = data["Titles"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            let processes = data["Processes"]
                .as_array()
                .cloned()
                .unwrap_or_default();

            if titles.is_empty() {
                println!("No processes found.");
                return;
            }

            // Print header
            let col_widths: Vec<usize> = titles.iter().map(|t| t.len()).collect();
            let col_widths: Vec<usize> = processes.iter().fold(col_widths, |mut widths, row| {
                if let Some(cols) = row.as_array() {
                    for (i, col) in cols.iter().enumerate() {
                        if let Some(s) = col.as_str() {
                            if i < widths.len() {
                                widths[i] = widths[i].max(s.len());
                            }
                        }
                    }
                }
                widths
            });

            let header: String = titles.iter().enumerate()
                .map(|(i, t)| format!("{:<width$}", t, width = col_widths[i]))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{}", header);
            println!("{}", "-".repeat(header.len()));

            for row in &processes {
                if let Some(cols) = row.as_array() {
                    let line: String = cols.iter().enumerate()
                        .map(|(i, v)| {
                            let s = v.as_str().unwrap_or("");
                            if i < col_widths.len() {
                                format!("{:<width$}", s, width = col_widths[i])
                            } else {
                                s.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("  ");
                    println!("{}", line);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch processes: {e}");
            std::process::exit(1);
        }
    }
}

pub fn kill(endpoint_id: u32, container_id: &str, signal: &str) {
    let client = PortainerClient::new();
    let path = format!(
        "endpoints/{}/docker/containers/{}/kill?signal={}",
        endpoint_id, encode(container_id), encode(signal)
    );
    match client.post_empty(&path) {
        Ok(()) => println!("Sent {signal} to container {container_id}."),
        Err(e) => {
            eprintln!("Failed to kill container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn rename(endpoint_id: u32, container_id: &str, new_name: &str) {
    let client = PortainerClient::new();
    let path = format!(
        "endpoints/{}/docker/containers/{}/rename?name={}",
        endpoint_id, encode(container_id), encode(new_name)
    );
    match client.post_empty(&path) {
        Ok(()) => println!("Container {container_id} renamed to {new_name}."),
        Err(e) => {
            eprintln!("Failed to rename container: {e}");
            std::process::exit(1);
        }
    }
}

pub fn prune(endpoint_id: u32) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/prune", endpoint_id);
    match client.post(&path, serde_json::json!({})) {
        Ok(data) => {
            let reclaimed = data["SpaceReclaimed"].as_u64().unwrap_or(0);
            let count = data["ContainersDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
            println!("Removed {} container(s), reclaimed {}.", count, fmt_bytes(reclaimed));
        }
        Err(e) => {
            eprintln!("Failed to prune containers: {e}");
            std::process::exit(1);
        }
    }
}

pub fn remove(endpoint_id: u32, container_id: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/containers/{}", endpoint_id, encode(container_id));
    match client.delete(&path) {
        Ok(()) => println!("Container {container_id} removed."),
        Err(e) => {
            eprintln!("Failed to remove container: {e}");
            std::process::exit(1);
        }
    }
}

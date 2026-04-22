use crate::client::PortainerClient;
use std::collections::HashSet;
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

fn in_use_ids(client: &PortainerClient, endpoint_id: u32) -> HashSet<String> {
    let path = format!("endpoints/{}/docker/containers/json?all=1", endpoint_id);
    match client.get(&path) {
        Ok(data) => data
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|c| c["ImageID"].as_str().map(|s| s.to_string()))
            .collect(),
        Err(_) => HashSet::new(),
    }
}

pub fn list(endpoint_id: u32) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/images/json", endpoint_id);

    let images = match client.get(&path) {
        Ok(data) => data.as_array().cloned().unwrap_or_default(),
        Err(e) => {
            eprintln!("Failed to list images: {e}");
            std::process::exit(1);
        }
    };

    if images.is_empty() {
        println!("No images found.");
        return;
    }

    let in_use = in_use_ids(&client, endpoint_id);

    println!("{:<14} {:<40} {:<10} {}", "ID", "REPOSITORY:TAG", "SIZE", "IN USE");
    println!("{}", "-".repeat(72));

    for img in &images {
        let id = img["Id"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches("sha256:")
            .chars()
            .take(12)
            .collect::<String>();
        let full_id = img["Id"].as_str().unwrap_or("");
        let size = img["Size"].as_u64().unwrap_or(0);
        let used = if in_use.contains(full_id) { "yes" } else { "no" };

        let tags = img["RepoTags"].as_array();
        let tag_strs: Vec<&str> = match tags {
            Some(t) if !t.is_empty() => t.iter().filter_map(|v| v.as_str()).collect(),
            _ => vec!["<none>:<none>"],
        };

        for (i, tag) in tag_strs.iter().enumerate() {
            if i == 0 {
                println!("{:<14} {:<40} {:<10} {}", id, tag, fmt_size(size), used);
            } else {
                println!("{:<14} {:<40}", "", tag);
            }
        }
    }
}

pub fn inspect(endpoint_id: u32, image: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/images/{}/json", endpoint_id, image);

    match client.get(&path) {
        Ok(img) => {
            let id = img["Id"]
                .as_str()
                .unwrap_or("")
                .trim_start_matches("sha256:")
                .chars()
                .take(12)
                .collect::<String>();

            let in_use = in_use_ids(&client, endpoint_id);
            let full_id = img["Id"].as_str().unwrap_or("");
            let used = if in_use.contains(full_id) { "yes" } else { "no" };

            println!("ID:           {}", id);

            if let Some(tags) = img["RepoTags"].as_array() {
                let tag_list: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
                if !tag_list.is_empty() {
                    println!("Tags:         {}", tag_list.join(", "));
                }
            }

            println!("Created:      {}", img["Created"].as_str().unwrap_or("(unknown)"));
            println!("Architecture: {}/{}", img["Os"].as_str().unwrap_or("(unknown)"), img["Architecture"].as_str().unwrap_or("(unknown)"));
            println!("Size:         {}", fmt_size(img["Size"].as_u64().unwrap_or(0)));
            println!("In use:       {}", used);

            if let Some(layers) = img["RootFS"]["Layers"].as_array() {
                println!("Layers:       {}", layers.len());
            }

            if let Some(ports) = img["Config"]["ExposedPorts"].as_object() {
                if !ports.is_empty() {
                    let port_list: Vec<&str> = ports.keys().map(|k| k.as_str()).collect();
                    println!("Exposed:      {}", port_list.join(", "));
                }
            }

            if let Some(envs) = img["Config"]["Env"].as_array() {
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
            eprintln!("Failed to inspect image: {e}");
            std::process::exit(1);
        }
    }
}

pub fn pull(endpoint_id: u32, image: &str) {
    let client = PortainerClient::new();

    let (from_image, tag) = match image.rsplit_once(':') {
        Some((name, tag)) => (name, tag),
        None => (image, "latest"),
    };

    let path = format!(
        "endpoints/{}/docker/images/create?fromImage={}&tag={}",
        endpoint_id, encode(from_image), encode(tag)
    );

    print!("Pulling {}:{}... ", from_image, tag);
    match client.post_empty(&path) {
        Ok(()) => println!("done."),
        Err(e) => {
            eprintln!("\nFailed to pull image: {e}");
            std::process::exit(1);
        }
    }
}

pub fn remove(endpoint_id: u32, image: &str) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/images/{}", endpoint_id, image);
    match client.delete(&path) {
        Ok(()) => println!("Image {image} removed."),
        Err(e) => {
            eprintln!("Failed to remove image: {e}");
            std::process::exit(1);
        }
    }
}

pub fn prune(endpoint_id: u32) {
    let client = PortainerClient::new();
    let path = format!("endpoints/{}/docker/images/prune", endpoint_id);
    match client.post(&path, serde_json::json!({})) {
        Ok(data) => {
            let reclaimed = data["SpaceReclaimed"].as_u64().unwrap_or(0);
            let count = data["ImagesDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
            println!("Removed {} image(s), reclaimed {}.", count, fmt_size(reclaimed));
        }
        Err(e) => {
            eprintln!("Failed to prune images: {e}");
            std::process::exit(1);
        }
    }
}

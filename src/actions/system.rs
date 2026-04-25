use crate::client::PortainerClient;
use crate::actions::endpoint;

fn fmt_size(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    }
}

pub fn prune(endpoint_filter: Option<&str>) {
    let client = PortainerClient::new();

    let endpoints: Vec<(u32, String)> = if let Some(name) = endpoint_filter {
        let id = endpoint::resolve_id(name);
        vec![(id, name.to_string())]
    } else {
        match client.get("endpoints") {
            Ok(data) => data.as_array().unwrap_or(&vec![]).iter().filter_map(|ep| {
                let id = ep["Id"].as_u64()? as u32;
                let name = ep["Name"].as_str().unwrap_or("(unknown)").to_string();
                Some((id, name))
            }).collect(),
            Err(e) => {
                eprintln!("Failed to fetch endpoints: {e}");
                std::process::exit(1);
            }
        }
    };

    let mut total_containers = 0usize;
    let mut total_images = 0usize;
    let mut total_volumes = 0usize;
    let mut total_networks = 0usize;
    let mut total_reclaimed = 0u64;

    for (eid, ep_name) in &endpoints {
        println!("Pruning {}...", ep_name);

        let container_path = format!("endpoints/{}/docker/containers/prune", eid);
        match client.post(&container_path, serde_json::json!({})) {
            Ok(data) => {
                let count = data["ContainersDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
                let reclaimed = data["SpaceReclaimed"].as_u64().unwrap_or(0);
                total_containers += count;
                total_reclaimed += reclaimed;
                println!("  Containers: {} removed", count);
            }
            Err(e) => eprintln!("  Containers: failed ({})", e),
        }

        // filters={"dangling":["false"]} prunes all unused images, not just untagged ones.
        let image_path = format!("endpoints/{}/docker/images/prune?filters=%7B%22dangling%22%3A%5B%22false%22%5D%7D", eid);
        match client.post(&image_path, serde_json::json!({})) {
            Ok(data) => {
                let count = data["ImagesDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
                let reclaimed = data["SpaceReclaimed"].as_u64().unwrap_or(0);
                total_images += count;
                total_reclaimed += reclaimed;
                println!("  Images:     {} removed", count);
            }
            Err(e) => eprintln!("  Images:     failed ({})", e),
        }

        // filters={"all":["true"]} is required (Docker API 1.42+) to prune named volumes,
        // not just anonymous ones.
        let volume_path = format!("endpoints/{}/docker/volumes/prune?filters=%7B%22all%22%3A%5B%22true%22%5D%7D", eid);
        match client.post(&volume_path, serde_json::json!({})) {
            Ok(data) => {
                let count = data["VolumesDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
                let reclaimed = data["SpaceReclaimed"].as_u64().unwrap_or(0);
                total_volumes += count;
                total_reclaimed += reclaimed;
                println!("  Volumes:    {} removed", count);
            }
            Err(e) => eprintln!("  Volumes:    failed ({})", e),
        }

        let network_path = format!("endpoints/{}/docker/networks/prune", eid);
        match client.post(&network_path, serde_json::json!({})) {
            Ok(data) => {
                let count = data["NetworksDeleted"].as_array().map(|a| a.len()).unwrap_or(0);
                total_networks += count;
                println!("  Networks:   {} removed", count);
            }
            Err(e) => eprintln!("  Networks:   failed ({})", e),
        }
    }

    println!();
    println!("Total removed: {} container(s), {} image(s), {} volume(s), {} network(s)",
        total_containers, total_images, total_volumes, total_networks);
    println!("Total reclaimed: {}", fmt_size(total_reclaimed));
}

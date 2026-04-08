use crate::client::PortainerClient;

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

            println!("{:<6} {:<30} {:<15} {}", "ID", "NAME", "TYPE", "URL");
            println!("{}", "-".repeat(70));

            for ep in endpoints {
                let id = ep["Id"].as_u64().unwrap_or(0);
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

                println!("{:<6} {:<30} {:<15} {}", id, name, ep_type, url);
            }
        }
        Err(e) => {
            eprintln!("Failed to list endpoints: {e}");
            std::process::exit(1);
        }
    }
}

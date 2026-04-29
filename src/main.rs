use portctl::actions::container;
use portctl::actions::endpoint;
use portctl::actions::image;
use portctl::actions::network;
use portctl::actions::stack;
use portctl::actions::system;
use portctl::actions::volume;
use portctl::cli;
use portctl::client::PortainerClient;
use portctl::config::Config;
use portctl::utils::confirm;

fn main() {
    let matches = cli::build_cli().get_matches();

    if matches.get_flag("insecure") {
        std::env::set_var("PORTCTL_INSECURE", "1");
    }

    match matches.subcommand() {
        Some(("config", sub)) => match sub.subcommand() {
            Some(("set-url", args)) => {
                let url = args.get_one::<String>("url").unwrap().clone();
                let mut cfg = Config::load();
                cfg.set_url(url);
                println!("Portainer URL saved.");
            }
            Some(("set-token", args)) => {
                let token = args.get_one::<String>("token").unwrap().clone();
                let mut cfg = Config::load();
                cfg.set_token(token);
                println!("API token saved.");
            }
            Some(("show", _)) => {
                let cfg = Config::load();
                println!("portainer_url: {}", cfg.portainer_url.as_deref().unwrap_or("(not set)"));
                let token_display = cfg.api_token.as_deref().map(|t| {
                    if t.len() <= 8 {
                        "*".repeat(t.len())
                    } else {
                        format!("{}****{}", &t[..4], &t[t.len() - 4..])
                    }
                }).unwrap_or_else(|| "(not set)".to_string());
                println!("api_token:     {token_display}");
            }
            Some(("check", _)) => {
                let client = PortainerClient::new();
                match client.get("endpoints") {
                    Ok(_) => println!("Connection successful."),
                    Err(e) => {
                        eprintln!("Connection failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            _ => unreachable!(),
        },
        Some(("stack", sub)) => match sub.subcommand() {
            Some(("ls", args)) => {
                let endpoint = args.get_one::<String>("endpoint").map(|s| s.as_str());
                stack::list(endpoint);
            }
            Some(("inspect", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                stack::inspect(name);
            }
            Some(("start", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                stack::start(name);
            }
            Some(("stop", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                if !args.get_flag("yes") && !confirm(&format!("Stop stack '{name}'?")) {
                    return;
                }
                stack::stop(name);
            }
            Some(("update", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                stack::update(name);
            }
            Some(("rm", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                if !args.get_flag("yes") && !confirm(&format!("Remove stack '{name}'?")) {
                    return;
                }
                stack::remove(name);
            }
            Some(("deploy", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                let endpoint = args.get_one::<String>("endpoint").unwrap();
                if let Some(file) = args.get_one::<String>("file") {
                    stack::deploy_from_file(name, endpoint, file);
                } else {
                    let git_url = args.get_one::<String>("git-url").unwrap();
                    let git_ref = args.get_one::<String>("git-ref").unwrap();
                    let compose_file = args.get_one::<String>("compose-file").unwrap();
                    let credentials = match (
                        args.get_one::<String>("git-username"),
                        args.get_one::<String>("git-password"),
                    ) {
                        (Some(u), Some(p)) => Some((u.as_str(), p.as_str())),
                        _ => None,
                    };
                    stack::deploy_from_git(name, endpoint, git_url, git_ref, compose_file, credentials);
                }
            }
            Some(("compose", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                stack::compose(name);
            }
            Some(("edit", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                stack::edit(name);
            }
            _ => unreachable!(),
        },
        Some(("image", sub)) => match sub.subcommand() {
            Some(("ls", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                image::list(eid);
            }
            Some(("inspect", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let img = args.get_one::<String>("image").unwrap();
                image::inspect(eid, img);
            }
            Some(("pull", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let img = args.get_one::<String>("image").unwrap();
                image::pull(eid, img);
            }
            Some(("rm", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let img = args.get_one::<String>("image").unwrap();
                if !args.get_flag("yes") && !confirm(&format!("Remove image '{img}'?")) {
                    return;
                }
                image::remove(eid, img);
            }
            Some(("prune", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                if !args.get_flag("yes") && !confirm("Remove all dangling images?") {
                    return;
                }
                image::prune(eid);
            }
            _ => unreachable!(),
        },
        Some(("system", sub)) => match sub.subcommand() {
            Some(("prune", args)) => {
                let endpoint = args.get_one::<String>("endpoint").map(|s| s.as_str());
                if !args.get_flag("yes") && !confirm("This will prune all stopped containers, dangling images, unused volumes, and unused networks. Proceed?") {
                    return;
                }
                system::prune(endpoint);
            }
            _ => unreachable!(),
        },
        Some(("volume", sub)) => match sub.subcommand() {
            Some(("ls", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                volume::list(eid);
            }
            Some(("inspect", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let name = args.get_one::<String>("name").unwrap();
                volume::inspect(eid, name);
            }
            Some(("create", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let name = args.get_one::<String>("name").unwrap();
                let driver = args.get_one::<String>("driver").unwrap();
                volume::create(eid, name, driver);
            }
            Some(("rm", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let name = args.get_one::<String>("name").unwrap();
                if !args.get_flag("yes") && !confirm(&format!("Remove volume '{name}'?")) {
                    return;
                }
                volume::remove(eid, name);
            }
            Some(("prune", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                if !args.get_flag("yes") && !confirm("Remove all unused volumes?") {
                    return;
                }
                volume::prune(eid);
            }
            _ => unreachable!(),
        },
        Some(("network", sub)) => match sub.subcommand() {
            Some(("ls", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                network::list(eid);
            }
            Some(("inspect", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let name = args.get_one::<String>("name").unwrap();
                network::inspect(eid, name);
            }
            Some(("create", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let name = args.get_one::<String>("name").unwrap();
                let driver = args.get_one::<String>("driver").unwrap();
                network::create(eid, name, driver);
            }
            Some(("rm", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let name = args.get_one::<String>("name").unwrap();
                if !args.get_flag("yes") && !confirm(&format!("Remove network '{name}'?")) {
                    return;
                }
                network::remove(eid, name);
            }
            Some(("prune", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                if !args.get_flag("yes") && !confirm("Remove all unused networks?") {
                    return;
                }
                network::prune(eid);
            }
            _ => unreachable!(),
        },
        Some(("endpoint", sub)) => match sub.subcommand() {
            Some(("ls", _)) => endpoint::list(),
            Some(("inspect", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                endpoint::inspect(name);
            }
            _ => unreachable!(),
        },
        Some(("container", sub)) => match sub.subcommand() {
            Some(("ls", args)) => {
                let endpoint = args.get_one::<String>("endpoint").map(|s| s.as_str());
                container::list(endpoint);
            }
            Some(("stats", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                container::stats(eid, &cid);
            }
            Some(("logs", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                let tail = *args.get_one::<u32>("tail").unwrap();
                let timestamps = args.get_flag("timestamps");
                let follow = args.get_flag("follow");
                container::logs(eid, &cid, tail, timestamps, follow);
            }
            Some(("start", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                container::start(eid, &cid);
            }
            Some(("stop", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                if !args.get_flag("yes") && !confirm(&format!("Stop container '{cid}'?")) {
                    return;
                }
                container::stop(eid, &cid);
            }
            Some(("restart", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                container::restart(eid, &cid);
            }
            Some(("inspect", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                container::inspect(eid, &cid);
            }
            Some(("rm", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                if !args.get_flag("yes") && !confirm(&format!("Remove container '{cid}'?")) {
                    return;
                }
                container::remove(eid, &cid);
            }
            Some(("prune", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                if !args.get_flag("yes") && !confirm("Remove all stopped containers?") {
                    return;
                }
                container::prune(eid);
            }
            Some(("cp", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let src = args.get_one::<String>("src").unwrap();
                let dest = args.get_one::<String>("dest").unwrap();
                container::cp(eid, src, dest);
            }
            Some(("pause", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                container::pause(eid, &cid);
            }
            Some(("unpause", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                container::unpause(eid, &cid);
            }
            Some(("top", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                container::top(eid, &cid);
            }
            Some(("kill", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                let signal = args.get_one::<String>("signal").unwrap();
                if !args.get_flag("yes") && !confirm(&format!("Send {signal} to container '{cid}'?")) {
                    return;
                }
                container::kill(eid, &cid, signal);
            }
            Some(("rename", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                let new_name = args.get_one::<String>("new-name").unwrap();
                container::rename(eid, &cid, new_name);
            }
            Some(("exec", args)) => {
                let eid = endpoint::resolve_id(args.get_one::<String>("endpoint").unwrap());
                let cid = args.get_one::<String>("id").unwrap().clone();
                let cmd: Vec<String> = args.get_many::<String>("cmd").unwrap().cloned().collect();
                container::exec(eid, &cid, &cmd);
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

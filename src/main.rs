use portctl::actions::container;
use portctl::actions::endpoint;
use portctl::actions::stack;
use portctl::cli;
use portctl::client::PortainerClient;
use portctl::config::Config;

fn main() {
    let matches = cli::build_cli().get_matches();

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
                println!("api_token:     {}", cfg.api_token.as_deref().unwrap_or("(not set)"));
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
                stack::stop(name);
            }
            Some(("update", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                stack::update(name);
            }
            Some(("rm", args)) => {
                let name = args.get_one::<String>("name").unwrap();
                stack::remove(name);
            }
            _ => unreachable!(),
        },
        Some(("endpoint", sub)) => match sub.subcommand() {
            Some(("ls", _)) => endpoint::list(),
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
                container::remove(eid, &cid);
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

use crate::constants;

pub fn build_cli() -> clap::Command {
    clap::Command::new(constants::NAME)
        .version(constants::VERSION)
        .author(constants::AUTHOR)
        .about("A command-line utility designed to manage Docker environments through the Portainer API")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            clap::Command::new("config")
                .about("Manage portctl configuration")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("set-url")
                        .about("Set the Portainer URL")
                        .arg(
                            clap::Arg::new("url")
                                .required(true)
                                .help("Portainer base URL (e.g. https://portainer.example.com)"),
                        ),
                )
                .subcommand(
                    clap::Command::new("set-token")
                        .about("Set the Portainer API token")
                        .arg(
                            clap::Arg::new("token")
                                .required(true)
                                .help("API token from Portainer"),
                        ),
                )
                .subcommand(
                    clap::Command::new("show")
                        .about("Show current configuration"),
                )
                .subcommand(
                    clap::Command::new("check")
                        .about("Verify connectivity to the Portainer instance"),
                ),
        )
        .subcommand(
            clap::Command::new("endpoint")
                .about("Manage Portainer endpoints")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("ls")
                        .about("List all endpoints"),
                )
                .subcommand(
                    clap::Command::new("inspect")
                        .about("Show detailed information about an endpoint")
                        .arg(
                            clap::Arg::new("name")
                                .required(true)
                                .value_name("NAME")
                                .help("Endpoint name"),
                        ),
                ),
        )
        .subcommand(
            clap::Command::new("stack")
                .about("Manage stacks on a Portainer endpoint")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("ls")
                        .about("List all stacks")
                        .arg(
                            clap::Arg::new("endpoint")
                                .short('e')
                                .long("endpoint")
                                .value_name("NAME")
                                .help("Filter by endpoint name"),
                        ),
                )
                .subcommand(
                    clap::Command::new("inspect")
                        .about("Show detailed information about a stack")
                        .arg(stack_name_arg()),
                )
                .subcommand(
                    clap::Command::new("start")
                        .about("Start a stack")
                        .arg(stack_name_arg()),
                )
                .subcommand(
                    clap::Command::new("stop")
                        .about("Stop a stack")
                        .arg(stack_name_arg()),
                )
                .subcommand(
                    clap::Command::new("update")
                        .about("Pull latest git changes and redeploy a stack")
                        .arg(stack_name_arg()),
                )
                .subcommand(
                    clap::Command::new("rm")
                        .about("Remove a stack")
                        .arg(stack_name_arg()),
                )
                .subcommand(
                    clap::Command::new("deploy")
                        .about("Create and deploy a new stack")
                        .arg(
                            clap::Arg::new("name")
                                .required(true)
                                .value_name("NAME")
                                .help("Stack name"),
                        )
                        .arg(endpoint_arg())
                        .arg(
                            clap::Arg::new("file")
                                .short('f')
                                .long("file")
                                .value_name("PATH")
                                .help("Path to a local compose file")
                                .conflicts_with("git-url"),
                        )
                        .arg(
                            clap::Arg::new("git-url")
                                .long("git-url")
                                .value_name("URL")
                                .help("Git repository URL")
                                .conflicts_with("file"),
                        )
                        .arg(
                            clap::Arg::new("git-ref")
                                .long("git-ref")
                                .value_name("REF")
                                .default_value("refs/heads/main")
                                .help("Git reference (branch, tag, or commit)"),
                        )
                        .arg(
                            clap::Arg::new("compose-file")
                                .long("compose-file")
                                .value_name("PATH")
                                .default_value("docker-compose.yml")
                                .help("Path to compose file inside the git repository"),
                        )
                        .arg(
                            clap::Arg::new("git-username")
                                .long("git-username")
                                .value_name("USERNAME")
                                .requires("git-url")
                                .requires("git-password")
                                .help("Username for private repository authentication"),
                        )
                        .arg(
                            clap::Arg::new("git-password")
                                .long("git-password")
                                .value_name("PASSWORD")
                                .requires("git-url")
                                .requires("git-username")
                                .help("Password or personal access token for private repository authentication"),
                        )
                        .group(
                            clap::ArgGroup::new("source")
                                .args(["file", "git-url"])
                                .required(true),
                        ),
                )
                .subcommand(
                    clap::Command::new("compose")
                        .about("Print the compose file of a stack")
                        .arg(stack_name_arg()),
                )
                .subcommand(
                    clap::Command::new("edit")
                        .about("Open a stack's compose file in an editor and redeploy on save")
                        .arg(stack_name_arg()),
                ),
        )
        .subcommand(
            clap::Command::new("system")
                .about("System-wide operations across all endpoints")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("prune")
                        .about("Remove all stopped containers, dangling images, unused volumes, and unused networks")
                        .arg(
                            clap::Arg::new("endpoint")
                                .short('e')
                                .long("endpoint")
                                .value_name("NAME")
                                .help("Limit to a specific endpoint (default: all endpoints)"),
                        ),
                ),
        )
        .subcommand(
            clap::Command::new("image")
                .about("Manage images on a Portainer endpoint")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("ls")
                        .about("List all images on an endpoint")
                        .arg(endpoint_arg()),
                )
                .subcommand(
                    clap::Command::new("inspect")
                        .about("Show detailed information about an image")
                        .arg(endpoint_arg())
                        .arg(image_ref_arg()),
                )
                .subcommand(
                    clap::Command::new("pull")
                        .about("Pull an image")
                        .arg(endpoint_arg())
                        .arg(image_ref_arg()),
                )
                .subcommand(
                    clap::Command::new("rm")
                        .about("Remove an image")
                        .arg(endpoint_arg())
                        .arg(image_ref_arg()),
                )
                .subcommand(
                    clap::Command::new("prune")
                        .about("Remove all dangling (untagged) images")
                        .arg(endpoint_arg()),
                ),
        )
        .subcommand(
            clap::Command::new("volume")
                .about("Manage volumes on a Portainer endpoint")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("ls")
                        .about("List all volumes on an endpoint")
                        .arg(endpoint_arg()),
                )
                .subcommand(
                    clap::Command::new("inspect")
                        .about("Show detailed information about a volume")
                        .arg(endpoint_arg())
                        .arg(volume_name_arg()),
                )
                .subcommand(
                    clap::Command::new("create")
                        .about("Create a volume")
                        .arg(endpoint_arg())
                        .arg(volume_name_arg())
                        .arg(
                            clap::Arg::new("driver")
                                .short('d')
                                .long("driver")
                                .value_name("DRIVER")
                                .default_value("local")
                                .help("Volume driver (e.g. local, nfs)"),
                        ),
                )
                .subcommand(
                    clap::Command::new("rm")
                        .about("Remove a volume")
                        .arg(endpoint_arg())
                        .arg(volume_name_arg()),
                )
                .subcommand(
                    clap::Command::new("prune")
                        .about("Remove all unused volumes")
                        .arg(endpoint_arg()),
                ),
        )
        .subcommand(
            clap::Command::new("network")
                .about("Manage networks on a Portainer endpoint")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("ls")
                        .about("List all networks on an endpoint")
                        .arg(endpoint_arg()),
                )
                .subcommand(
                    clap::Command::new("inspect")
                        .about("Show detailed information about a network")
                        .arg(endpoint_arg())
                        .arg(network_name_arg()),
                )
                .subcommand(
                    clap::Command::new("create")
                        .about("Create a network")
                        .arg(endpoint_arg())
                        .arg(network_name_arg())
                        .arg(
                            clap::Arg::new("driver")
                                .short('d')
                                .long("driver")
                                .value_name("DRIVER")
                                .default_value("bridge")
                                .help("Network driver (e.g. bridge, overlay)"),
                        ),
                )
                .subcommand(
                    clap::Command::new("rm")
                        .about("Remove a network")
                        .arg(endpoint_arg())
                        .arg(network_name_arg()),
                )
                .subcommand(
                    clap::Command::new("prune")
                        .about("Remove all unused networks")
                        .arg(endpoint_arg()),
                ),
        )
        .subcommand(
            clap::Command::new("container")
                .about("Manage containers on a Portainer endpoint")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("ls")
                        .about("List all containers")
                        .arg(
                            clap::Arg::new("endpoint")
                                .short('e')
                                .long("endpoint")
                                .value_name("NAME")
                                .help("Filter by endpoint name"),
                        ),
                )
                .subcommand(
                    clap::Command::new("logs")
                        .about("Fetch logs from a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg())
                        .arg(
                            clap::Arg::new("tail")
                                .short('n')
                                .long("tail")
                                .value_name("LINES")
                                .default_value("100")
                                .value_parser(clap::value_parser!(u32))
                                .help("Number of lines to show from the end of the logs"),
                        )
                        .arg(
                            clap::Arg::new("timestamps")
                                .short('t')
                                .long("timestamps")
                                .action(clap::ArgAction::SetTrue)
                                .help("Show timestamps"),
                        )
                        .arg(
                            clap::Arg::new("follow")
                                .short('f')
                                .long("follow")
                                .action(clap::ArgAction::SetTrue)
                                .help("Stream logs as they are produced"),
                        ),
                )
                .subcommand(
                    clap::Command::new("stats")
                        .about("Show CPU, memory, network, and block I/O usage for a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("start")
                        .about("Start a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("stop")
                        .about("Stop a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("restart")
                        .about("Restart a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("inspect")
                        .about("Show detailed information about a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("rm")
                        .about("Remove a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("prune")
                        .about("Remove all stopped containers")
                        .arg(endpoint_arg()),
                )
                .subcommand(
                    clap::Command::new("cp")
                        .about("Copy files between a container and the local filesystem")
                        .after_help("Use <container>:<path> to refer to a path inside a container.\n\nExamples:\n  portctl container cp -e ep mycontainer:/app/config.yml ./\n  portctl container cp -e ep ./config.yml mycontainer:/app/")
                        .arg(endpoint_arg())
                        .arg(
                            clap::Arg::new("src")
                                .required(true)
                                .value_name("SRC")
                                .help("Source path (use <container>:<path> for container paths)"),
                        )
                        .arg(
                            clap::Arg::new("dest")
                                .required(true)
                                .value_name("DEST")
                                .help("Destination path (use <container>:<path> for container paths)"),
                        ),
                )
                .subcommand(
                    clap::Command::new("pause")
                        .about("Pause all processes in a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("unpause")
                        .about("Unpause all processes in a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("top")
                        .about("Show running processes inside a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg()),
                )
                .subcommand(
                    clap::Command::new("kill")
                        .about("Send a signal to a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg())
                        .arg(
                            clap::Arg::new("signal")
                                .short('s')
                                .long("signal")
                                .value_name("SIGNAL")
                                .default_value("SIGTERM")
                                .help("Signal to send (e.g. SIGTERM, SIGKILL, SIGHUP)"),
                        ),
                )
                .subcommand(
                    clap::Command::new("rename")
                        .about("Rename a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg())
                        .arg(
                            clap::Arg::new("new-name")
                                .required(true)
                                .value_name("NEW_NAME")
                                .help("New container name"),
                        ),
                )
                .subcommand(
                    clap::Command::new("exec")
                        .about("Run a command inside a container")
                        .arg(endpoint_arg())
                        .arg(container_id_arg())
                        .arg(
                            clap::Arg::new("cmd")
                                .required(true)
                                .num_args(1..)
                                .last(true)
                                .value_name("CMD")
                                .help("Command to run (use -- to separate from portctl args)"),
                        ),
                ),
        )
}

fn image_ref_arg() -> clap::Arg {
    clap::Arg::new("image")
        .required(true)
        .value_name("IMAGE")
        .help("Image name, name:tag, or ID")
}

fn volume_name_arg() -> clap::Arg {
    clap::Arg::new("name")
        .required(true)
        .value_name("VOLUME")
        .help("Volume name")
}

fn network_name_arg() -> clap::Arg {
    clap::Arg::new("name")
        .required(true)
        .value_name("NETWORK")
        .help("Network name")
}

fn stack_name_arg() -> clap::Arg {
    clap::Arg::new("name")
        .required(true)
        .value_name("STACK")
        .help("Stack name")
}

fn endpoint_arg() -> clap::Arg {
    clap::Arg::new("endpoint")
        .short('e')
        .long("endpoint")
        .required(true)
        .value_name("NAME")
        .help("Portainer endpoint name")
}

fn container_id_arg() -> clap::Arg {
    clap::Arg::new("id")
        .required(true)
        .value_name("CONTAINER")
        .help("Container ID or name")
}

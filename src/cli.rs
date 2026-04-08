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
                ),
        )
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

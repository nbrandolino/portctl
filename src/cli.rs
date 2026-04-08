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
}

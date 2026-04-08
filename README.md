# Portctl
`portctl` is a command-line utility written in Rust, designed to manage Docker environments through the Portainer API. It lets you interact with containers, stacks, volumes, and networks directly from your terminal.

## Requirements
- **Rust**: Required to compile the utility.
- **Linux Environment**: Currently designed to work on Linux-based systems.
- **Portainer**: Must have the Portainer application installed and accessible.

## Installation
### Install From Source
1. Navigate to the project directory:
   ```bash
   cd portctl
   ```
2. Install:
   ```bash
   cargo install --path .
   ```

## Usage
Run the tool using the following command:
```bash
portctl [OPTIONS]
```

### Available Options
- `-h, --help`: Display help information.
- `-V, --version`: Display version information.

### Examples


## Configuration
The tool uses a configuration file located at:
```bash
TBD
```
This file stores the list of repositories being managed.

## Disclaimer
> **Note:** This GitHub repository is a mirror of a private, self-hosted GitLab repository.

## License
This tool is licensed under the GNU General Public License (GPL). See the `LICENSE` file for more details.

## Contact
- **Author**: nbrandolino
- **Email**: [nickbrandolino134@gmail.com](mailto:nickbrandolino134@gmail.com)

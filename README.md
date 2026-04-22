# Portctl
`portctl` is a command-line utility written in Rust, designed to manage Docker environments through the Portainer API. It lets you interact with containers, stacks, images, volumes, and networks directly from your terminal.

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

## Configuration
The configuration file is located at:
```
~/.config/portctl/config.toml
```

### Setup
```bash
portctl config set-url https://portainer.example.com
portctl config set-token <your-api-token>
portctl config check    # verify connectivity
portctl config show     # print current config
```

## Usage
```bash
portctl <SUBCOMMAND> [OPTIONS]
```

### Confirmation prompts
Destructive commands (`rm`, `prune`, `stop`, `kill`) prompt for confirmation before executing:
```
Remove container 'myapp'? [y/N]
```
Use `-y` / `--yes` to skip the prompt in scripts or non-interactive environments:
```bash
portctl container rm -e my-endpoint myapp --yes
portctl system prune --yes
```

---

## Subcommands

### config
Manage portctl configuration.

| Command | Description |
|---|---|
| `config set-url <URL>` | Set the Portainer URL |
| `config set-token <TOKEN>` | Set the Portainer API token |
| `config show` | Print current configuration |
| `config check` | Verify connectivity to Portainer |

---

### endpoint
Manage Portainer endpoints.

| Command | Description |
|---|---|
| `endpoint ls` | List all endpoints |
| `endpoint inspect <NAME>` | Show detailed information about an endpoint |

---

### stack
Manage stacks on a Portainer endpoint.

| Command | Description |
|---|---|
| `stack ls [-e NAME]` | List all stacks (optionally filter by endpoint) |
| `stack inspect <NAME>` | Show detailed information about a stack |
| `stack deploy <NAME> -e <ENDPOINT> -f <FILE>` | Deploy a new stack from a local compose file |
| `stack deploy <NAME> -e <ENDPOINT> --git-url <URL>` | Deploy a new stack from a git repository |
| `stack compose <NAME>` | Print the compose file of a stack |
| `stack edit <NAME>` | Open a stack's compose file in an editor and redeploy on save |
| `stack start <NAME>` | Start a stack |
| `stack stop <NAME>` | Stop a stack |
| `stack update <NAME>` | Pull latest git changes and redeploy a stack |
| `stack rm <NAME>` | Remove a stack |

#### Deploying from a git repository
```bash
# Public repository
portctl stack deploy mystack -e my-endpoint \
  --git-url https://github.com/user/repo

# With custom branch and compose file path
portctl stack deploy mystack -e my-endpoint \
  --git-url https://github.com/user/repo \
  --git-ref refs/heads/main \
  --compose-file docker-compose.yml

# Private repository
portctl stack deploy mystack -e my-endpoint \
  --git-url https://github.com/user/private-repo \
  --git-username myuser \
  --git-password ghp_xxxxxxxxxxxx
```

#### stack edit
Opens the stack's compose file in your preferred editor. On save and exit, the updated file is sent back to Portainer and the stack is redeployed. If no changes are made, no API call is sent.

- Git-controlled stacks are rejected with an error — edit the source in git and use `stack update` instead.
- Editor is resolved from `$VISUAL`, then `$EDITOR`, then `vi`.
- Exiting the editor with a non-zero status (e.g. `:cq` in vim) cancels the update.

---

### image
Manage images on a Portainer endpoint.

| Command | Description |
|---|---|
| `image ls -e <ENDPOINT>` | List all images on an endpoint |
| `image inspect -e <ENDPOINT> <IMAGE>` | Show detailed information about an image |
| `image pull -e <ENDPOINT> <IMAGE>` | Pull an image |
| `image rm -e <ENDPOINT> <IMAGE>` | Remove an image |
| `image prune -e <ENDPOINT>` | Remove all dangling (untagged) images |

---

### volume
Manage volumes on a Portainer endpoint.

| Command | Description |
|---|---|
| `volume ls -e <ENDPOINT>` | List all volumes on an endpoint |
| `volume inspect -e <ENDPOINT> <VOLUME>` | Show detailed information about a volume |
| `volume create -e <ENDPOINT> <VOLUME> [-d DRIVER]` | Create a volume |
| `volume rm -e <ENDPOINT> <VOLUME>` | Remove a volume |
| `volume prune -e <ENDPOINT>` | Remove all unused volumes |

---

### network
Manage networks on a Portainer endpoint.

| Command | Description |
|---|---|
| `network ls -e <ENDPOINT>` | List all networks on an endpoint |
| `network inspect -e <ENDPOINT> <NETWORK>` | Show detailed information about a network |
| `network create -e <ENDPOINT> <NETWORK> [-d DRIVER]` | Create a network |
| `network rm -e <ENDPOINT> <NETWORK>` | Remove a network |
| `network prune -e <ENDPOINT>` | Remove all unused networks |

---

### container
Manage containers on a Portainer endpoint.

| Command | Description |
|---|---|
| `container ls [-e ENDPOINT]` | List all containers (optionally filter by endpoint) |
| `container inspect -e <ENDPOINT> <CONTAINER>` | Show detailed information about a container |
| `container stats -e <ENDPOINT> <CONTAINER>` | Show CPU, memory, network, and block I/O usage |
| `container top -e <ENDPOINT> <CONTAINER>` | Show running processes inside a container |
| `container logs -e <ENDPOINT> <CONTAINER>` | Fetch logs from a container |
| `container start -e <ENDPOINT> <CONTAINER>` | Start a container |
| `container stop -e <ENDPOINT> <CONTAINER>` | Stop a container |
| `container restart -e <ENDPOINT> <CONTAINER>` | Restart a container |
| `container pause -e <ENDPOINT> <CONTAINER>` | Pause all processes in a container |
| `container unpause -e <ENDPOINT> <CONTAINER>` | Unpause all processes in a container |
| `container kill -e <ENDPOINT> <CONTAINER> [-s SIGNAL]` | Send a signal to a container |
| `container rename -e <ENDPOINT> <CONTAINER> <NEW_NAME>` | Rename a container |
| `container exec -e <ENDPOINT> <CONTAINER> -- <CMD>` | Run a command inside a container |
| `container cp -e <ENDPOINT> <SRC> <DEST>` | Copy files between a container and the local filesystem |
| `container rm -e <ENDPOINT> <CONTAINER>` | Remove a container |
| `container prune -e <ENDPOINT>` | Remove all stopped containers |

#### container logs options
```bash
portctl container logs -e my-endpoint my-container
portctl container logs -e my-endpoint my-container -n 50      # last 50 lines
portctl container logs -e my-endpoint my-container -t         # include timestamps
portctl container logs -e my-endpoint my-container -f         # stream (follow)
```

#### container kill signals
```bash
portctl container kill -e my-endpoint my-container            # SIGTERM (default)
portctl container kill -e my-endpoint my-container -s SIGKILL
portctl container kill -e my-endpoint my-container -s SIGHUP
```

#### container cp
Use `<container>:<path>` to refer to a path inside a container.
```bash
# Download from container
portctl container cp -e my-endpoint mycontainer:/app/config.yml ./
portctl container cp -e my-endpoint mycontainer:/app/config.yml ./config-backup.yml

# Upload to container
portctl container cp -e my-endpoint ./config.yml mycontainer:/app/
portctl container cp -e my-endpoint ./configs/ mycontainer:/app/
```
File permissions (mode bits) are preserved in both directions.

---

### system
System-wide operations across all endpoints.

| Command | Description |
|---|---|
| `system prune [-e ENDPOINT]` | Remove stopped containers, dangling images, unused volumes and networks |

---

## Disclaimer
> **Note:** This GitHub repository is a mirror of a private, self-hosted GitLab repository.

## License
This tool is licensed under the GNU General Public License (GPL). See the `LICENSE` file for more details.

## Contact
- **Author**: nbrandolino
- **Email**: [nickbrandolino134@gmail.com](mailto:nickbrandolino134@gmail.com)

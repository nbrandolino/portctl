use portctl::cli::build_cli;
use portctl::config::Config;
use portctl::utils::ensure_config_dir_exists;
use std::sync::Mutex;
use tempfile::TempDir;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn with_temp_home<F: FnOnce(&TempDir)>(f: F) {
    let _guard = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let prev = std::env::var("HOME").ok();
    std::env::set_var("HOME", dir.path());
    f(&dir);
    match prev {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

#[test]
fn config_default_has_no_fields() {
    let cfg = Config::default();
    assert!(cfg.portainer_url.is_none());
    assert!(cfg.api_token.is_none());
}

#[test]
fn config_load_missing_file_returns_default() {
    with_temp_home(|_| {
        let cfg = Config::load();
        assert!(cfg.portainer_url.is_none());
        assert!(cfg.api_token.is_none());
    });
}

#[test]
fn config_load_invalid_toml_returns_default() {
    with_temp_home(|dir| {
        let config_dir = dir.path().join(".config").join("portctl");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), b"not valid toml :::").unwrap();
        let cfg = Config::load();
        assert!(cfg.portainer_url.is_none());
        assert!(cfg.api_token.is_none());
    });
}

#[test]
fn config_round_trip() {
    with_temp_home(|_| {
        let mut cfg = Config::default();
        cfg.portainer_url = Some("https://portainer.example.com".to_string());
        cfg.api_token = Some("mytoken".to_string());
        cfg.save();
        let loaded = Config::load();
        assert_eq!(loaded.portainer_url.as_deref(), Some("https://portainer.example.com"));
        assert_eq!(loaded.api_token.as_deref(), Some("mytoken"));
    });
}

#[test]
fn config_set_url_persists() {
    with_temp_home(|_| {
        let mut cfg = Config::default();
        cfg.set_url("https://portainer.local".to_string());
        let loaded = Config::load();
        assert_eq!(loaded.portainer_url.as_deref(), Some("https://portainer.local"));
    });
}

#[test]
fn config_set_token_persists() {
    with_temp_home(|_| {
        let mut cfg = Config::default();
        cfg.set_token("secret-token-abc".to_string());
        let loaded = Config::load();
        assert_eq!(loaded.api_token.as_deref(), Some("secret-token-abc"));
    });
}

#[test]
fn config_set_url_overwrites_existing() {
    with_temp_home(|_| {
        let mut cfg = Config::default();
        cfg.set_url("https://old.example.com".to_string());
        cfg.set_url("https://new.example.com".to_string());
        let loaded = Config::load();
        assert_eq!(loaded.portainer_url.as_deref(), Some("https://new.example.com"));
    });
}

#[test]
fn config_set_token_does_not_clear_url() {
    with_temp_home(|_| {
        let mut cfg = Config::default();
        cfg.set_url("https://portainer.local".to_string());
        cfg.set_token("mytoken".to_string());
        let loaded = Config::load();
        assert_eq!(loaded.portainer_url.as_deref(), Some("https://portainer.local"));
        assert_eq!(loaded.api_token.as_deref(), Some("mytoken"));
    });
}

// ── Utils ─────────────────────────────────────────────────────────────────────

#[test]
fn ensure_config_dir_creates_nested_directory() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("nested").join("config").join("dir");
    assert!(!target.exists());
    ensure_config_dir_exists(&target);
    assert!(target.exists());
}

#[test]
fn ensure_config_dir_is_noop_when_already_exists() {
    let dir = TempDir::new().unwrap();
    ensure_config_dir_exists(dir.path());
    assert!(dir.path().exists());
}

// ── CLI: top-level ────────────────────────────────────────────────────────────

#[test]
fn cli_builds_successfully() {
    let _ = build_cli();
}

#[test]
fn cli_requires_subcommand() {
    let result = build_cli().try_get_matches_from(["portctl"]);
    assert!(result.is_err());
}

// ── CLI: config ───────────────────────────────────────────────────────────────

#[test]
fn cli_config_set_url_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "config", "set-url", "https://example.com"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "config");
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "set-url");
    assert_eq!(csub_m.get_one::<String>("url").unwrap(), "https://example.com");
}

#[test]
fn cli_config_set_token_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "config", "set-token", "tok123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "set-token");
    assert_eq!(csub_m.get_one::<String>("token").unwrap(), "tok123");
}

#[test]
fn cli_config_show_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "config", "show"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "show");
}

#[test]
fn cli_config_check_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "config", "check"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "check");
}

#[test]
fn cli_config_set_url_requires_arg() {
    let result = build_cli().try_get_matches_from(["portctl", "config", "set-url"]);
    assert!(result.is_err());
}

#[test]
fn cli_config_set_token_requires_arg() {
    let result = build_cli().try_get_matches_from(["portctl", "config", "set-token"]);
    assert!(result.is_err());
}

// ── CLI: endpoint ─────────────────────────────────────────────────────────────

#[test]
fn cli_endpoint_ls_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "endpoint", "ls"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "endpoint");
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "ls");
}

#[test]
fn cli_endpoint_inspect_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "endpoint", "inspect", "my-endpoint"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "inspect");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "my-endpoint");
}

#[test]
fn cli_endpoint_inspect_requires_name() {
    let result = build_cli().try_get_matches_from(["portctl", "endpoint", "inspect"]);
    assert!(result.is_err());
}

// ── CLI: stack ────────────────────────────────────────────────────────────────

#[test]
fn cli_stack_ls_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "ls"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "stack");
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "ls");
}

#[test]
fn cli_stack_ls_optional_endpoint_filter() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "ls", "-e", "myenv"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("endpoint").unwrap(), "myenv");
}

#[test]
fn cli_stack_inspect_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "inspect", "mystack"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "inspect");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mystack");
}

#[test]
fn cli_stack_stop_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "stop", "mystack", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

#[test]
fn cli_stack_rm_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "rm", "mystack", "--yes"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

#[test]
fn cli_stack_deploy_file_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep", "-f", "./docker-compose.yml"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "deploy");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mystack");
    assert_eq!(csub_m.get_one::<String>("file").unwrap(), "./docker-compose.yml");
}

#[test]
fn cli_stack_deploy_git_defaults() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "--git-url", "https://github.com/foo/bar"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("git-url").unwrap(), "https://github.com/foo/bar");
    assert_eq!(csub_m.get_one::<String>("git-ref").unwrap(), "refs/heads/main");
    assert_eq!(csub_m.get_one::<String>("compose-file").unwrap(), "docker-compose.yml");
}

#[test]
fn cli_stack_deploy_git_custom_ref() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "--git-url", "https://github.com/foo/bar",
                               "--git-ref", "refs/heads/develop"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("git-ref").unwrap(), "refs/heads/develop");
}

#[test]
fn cli_stack_deploy_requires_source() {
    let result = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep"]);
    assert!(result.is_err());
}

#[test]
fn cli_stack_deploy_file_and_git_conflict() {
    let result = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "-f", "./docker-compose.yml",
                               "--git-url", "https://github.com/foo/bar"]);
    assert!(result.is_err());
}

#[test]
fn cli_stack_deploy_git_credentials_require_password() {
    let result = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "--git-url", "https://github.com/foo/bar",
                               "--git-username", "user"]);
    assert!(result.is_err());
}

#[test]
fn cli_stack_deploy_git_credentials_full() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "--git-url", "https://github.com/foo/bar",
                               "--git-username", "user",
                               "--git-password", "pass"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("git-username").unwrap(), "user");
    assert_eq!(csub_m.get_one::<String>("git-password").unwrap(), "pass");
}

// ── CLI: container ────────────────────────────────────────────────────────────

#[test]
fn cli_container_ls_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "ls"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "container");
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "ls");
}

#[test]
fn cli_container_logs_defaults() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "logs", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "logs");
    assert_eq!(csub_m.get_one::<u32>("tail").copied(), Some(100));
    assert!(!csub_m.get_flag("timestamps"));
    assert!(!csub_m.get_flag("follow"));
}

#[test]
fn cli_container_logs_all_flags() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "logs", "-e", "ep", "abc123",
                               "-t", "-f", "-n", "50"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<u32>("tail").copied(), Some(50));
    assert!(csub_m.get_flag("timestamps"));
    assert!(csub_m.get_flag("follow"));
}

#[test]
fn cli_container_kill_default_signal() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "kill", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "kill");
    assert_eq!(csub_m.get_one::<String>("signal").unwrap(), "SIGTERM");
}

#[test]
fn cli_container_kill_custom_signal() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "kill", "-e", "ep", "abc123",
                               "-s", "SIGKILL"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("signal").unwrap(), "SIGKILL");
}

#[test]
fn cli_container_rename_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "rename", "-e", "ep", "oldname", "newname"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "rename");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "oldname");
    assert_eq!(csub_m.get_one::<String>("new-name").unwrap(), "newname");
}

#[test]
fn cli_container_exec_parses_cmd() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "exec", "-e", "ep",
                               "mycontainer", "--", "sh", "-c", "echo hello"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "exec");
    let cmd: Vec<&String> = csub_m.get_many::<String>("cmd").unwrap().collect();
    assert_eq!(cmd, vec!["sh", "-c", "echo hello"]);
}

#[test]
fn cli_container_cp_container_to_host() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "cp", "-e", "ep",
                               "mycontainer:/app/config.yml", "./"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "cp");
    assert_eq!(csub_m.get_one::<String>("src").unwrap(), "mycontainer:/app/config.yml");
    assert_eq!(csub_m.get_one::<String>("dest").unwrap(), "./");
}

#[test]
fn cli_container_cp_host_to_container() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "cp", "-e", "ep",
                               "./config.yml", "mycontainer:/app/"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("src").unwrap(), "./config.yml");
    assert_eq!(csub_m.get_one::<String>("dest").unwrap(), "mycontainer:/app/");
}

#[test]
fn cli_container_prune_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "prune", "-e", "ep", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

// ── CLI: image ────────────────────────────────────────────────────────────────

#[test]
fn cli_image_ls_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "image", "ls", "-e", "ep"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "image");
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "ls");
}

#[test]
fn cli_image_inspect_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "image", "inspect", "-e", "ep", "nginx:latest"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "inspect");
    assert_eq!(csub_m.get_one::<String>("image").unwrap(), "nginx:latest");
}

#[test]
fn cli_image_rm_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "image", "rm", "-e", "ep", "nginx:latest", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

// ── CLI: volume ───────────────────────────────────────────────────────────────

#[test]
fn cli_volume_ls_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "ls", "-e", "ep"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "volume");
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "ls");
}

#[test]
fn cli_volume_create_default_driver() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "create", "-e", "ep", "myvol"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "create");
    assert_eq!(csub_m.get_one::<String>("driver").unwrap(), "local");
}

#[test]
fn cli_volume_create_custom_driver() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "create", "-e", "ep", "myvol", "-d", "nfs"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("driver").unwrap(), "nfs");
}

#[test]
fn cli_volume_rm_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "rm", "-e", "ep", "myvol", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

// ── CLI: network ──────────────────────────────────────────────────────────────

#[test]
fn cli_network_ls_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "ls", "-e", "ep"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "network");
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "ls");
}

#[test]
fn cli_network_create_default_driver() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "create", "-e", "ep", "mynet"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "create");
    assert_eq!(csub_m.get_one::<String>("driver").unwrap(), "bridge");
}

#[test]
fn cli_network_create_custom_driver() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "create", "-e", "ep", "mynet", "-d", "overlay"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("driver").unwrap(), "overlay");
}

#[test]
fn cli_network_rm_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "rm", "-e", "ep", "mynet", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

// ── CLI: system ───────────────────────────────────────────────────────────────

#[test]
fn cli_system_prune_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "system", "prune"])
        .unwrap();
    let (sub, sub_m) = m.subcommand().unwrap();
    assert_eq!(sub, "system");
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "prune");
    assert!(csub_m.get_one::<String>("endpoint").is_none());
}

#[test]
fn cli_system_prune_with_endpoint() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "system", "prune", "-e", "myenv"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("endpoint").unwrap(), "myenv");
}

#[test]
fn cli_system_prune_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "system", "prune", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

// ── CLI: global flags ─────────────────────────────────────────────────────────

#[test]
fn cli_insecure_flag_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "--insecure", "config", "show"])
        .unwrap();
    assert!(m.get_flag("insecure"));
}

// ── CLI: stack (missing subcommands) ─────────────────────────────────────────

#[test]
fn cli_stack_start_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "start", "mystack"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "start");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mystack");
}

#[test]
fn cli_stack_start_requires_name() {
    let result = build_cli().try_get_matches_from(["portctl", "stack", "start"]);
    assert!(result.is_err());
}

#[test]
fn cli_stack_update_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "update", "mystack"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "update");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mystack");
}

#[test]
fn cli_stack_compose_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "compose", "mystack"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "compose");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mystack");
}

#[test]
fn cli_stack_edit_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "edit", "mystack"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "edit");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mystack");
}

#[test]
fn cli_stack_inspect_requires_name() {
    let result = build_cli().try_get_matches_from(["portctl", "stack", "inspect"]);
    assert!(result.is_err());
}

#[test]
fn cli_stack_stop_requires_name() {
    let result = build_cli().try_get_matches_from(["portctl", "stack", "stop"]);
    assert!(result.is_err());
}

#[test]
fn cli_stack_rm_requires_name() {
    let result = build_cli().try_get_matches_from(["portctl", "stack", "rm"]);
    assert!(result.is_err());
}

#[test]
fn cli_stack_deploy_env_file_with_file_source() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "-f", "./docker-compose.yml", "--env-file", ".env"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("env-file").unwrap(), ".env");
}

#[test]
fn cli_stack_deploy_env_file_with_git_source() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "--git-url", "https://github.com/foo/bar",
                               "--env-file", "prod.env"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("env-file").unwrap(), "prod.env");
}

#[test]
fn cli_stack_deploy_custom_compose_file() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "stack", "deploy", "mystack", "-e", "ep",
                               "--git-url", "https://github.com/foo/bar",
                               "--compose-file", "infra/docker-compose.yml"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub_m.get_one::<String>("compose-file").unwrap(), "infra/docker-compose.yml");
}

// ── CLI: container (missing subcommands) ─────────────────────────────────────

#[test]
fn cli_container_stats_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "stats", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "stats");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_start_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "start", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "start");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_stop_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "stop", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "stop");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_stop_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "stop", "-e", "ep", "abc123", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

#[test]
fn cli_container_restart_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "restart", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "restart");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_inspect_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "inspect", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "inspect");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_rm_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "rm", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "rm");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_rm_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "rm", "-e", "ep", "abc123", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

#[test]
fn cli_container_pause_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "pause", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "pause");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_unpause_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "unpause", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "unpause");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_top_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "top", "-e", "ep", "abc123"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "top");
    assert_eq!(csub_m.get_one::<String>("id").unwrap(), "abc123");
}

#[test]
fn cli_container_kill_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "kill", "-e", "ep", "abc123", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

#[test]
fn cli_container_exec_requires_cmd() {
    let result = build_cli()
        .try_get_matches_from(["portctl", "container", "exec", "-e", "ep", "abc123"]);
    assert!(result.is_err());
}

#[test]
fn cli_container_ls_without_endpoint() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "container", "ls"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_one::<String>("endpoint").is_none());
}

// ── CLI: image (missing subcommands) ─────────────────────────────────────────

#[test]
fn cli_image_pull_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "image", "pull", "-e", "ep", "nginx:latest"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "pull");
    assert_eq!(csub_m.get_one::<String>("image").unwrap(), "nginx:latest");
}

#[test]
fn cli_image_prune_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "image", "prune", "-e", "ep"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "prune");
}

#[test]
fn cli_image_prune_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "image", "prune", "-e", "ep", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

#[test]
fn cli_image_rm_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "image", "rm", "-e", "ep", "nginx:latest"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "rm");
    assert_eq!(csub_m.get_one::<String>("image").unwrap(), "nginx:latest");
}

// ── CLI: volume (missing subcommands) ────────────────────────────────────────

#[test]
fn cli_volume_inspect_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "inspect", "-e", "ep", "myvol"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "inspect");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "myvol");
}

#[test]
fn cli_volume_rm_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "rm", "-e", "ep", "myvol"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "rm");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "myvol");
}

#[test]
fn cli_volume_prune_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "prune", "-e", "ep"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "prune");
}

#[test]
fn cli_volume_prune_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "volume", "prune", "-e", "ep", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

// ── CLI: network (missing subcommands) ───────────────────────────────────────

#[test]
fn cli_network_inspect_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "inspect", "-e", "ep", "mynet"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "inspect");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mynet");
}

#[test]
fn cli_network_rm_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "rm", "-e", "ep", "mynet"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, csub_m) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "rm");
    assert_eq!(csub_m.get_one::<String>("name").unwrap(), "mynet");
}

#[test]
fn cli_network_prune_parses() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "prune", "-e", "ep"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (csub, _) = sub_m.subcommand().unwrap();
    assert_eq!(csub, "prune");
}

#[test]
fn cli_network_prune_yes_flag() {
    let m = build_cli()
        .try_get_matches_from(["portctl", "network", "prune", "-e", "ep", "-y"])
        .unwrap();
    let (_, sub_m) = m.subcommand().unwrap();
    let (_, csub_m) = sub_m.subcommand().unwrap();
    assert!(csub_m.get_flag("yes"));
}

// ── Config (additional) ───────────────────────────────────────────────────────

#[test]
fn config_load_partial_toml_only_url() {
    with_temp_home(|dir| {
        let config_dir = dir.path().join(".config").join("portctl");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            b"portainer_url = \"https://portainer.local\"\n",
        ).unwrap();
        let cfg = Config::load();
        assert_eq!(cfg.portainer_url.as_deref(), Some("https://portainer.local"));
        assert!(cfg.api_token.is_none());
    });
}

#[test]
fn config_load_partial_toml_only_token() {
    with_temp_home(|dir| {
        let config_dir = dir.path().join(".config").join("portctl");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            b"api_token = \"mytoken\"\n",
        ).unwrap();
        let cfg = Config::load();
        assert!(cfg.portainer_url.is_none());
        assert_eq!(cfg.api_token.as_deref(), Some("mytoken"));
    });
}

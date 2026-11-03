use std::os::unix::fs::PermissionsExt;

use cucumber::{given, then, when};
use passuff_config::{ConfigLoader, WatchMode};

use crate::support::World;

fn config(world: &World) -> passuff_config::AppConfig {
    if world.e2e {
        ConfigLoader::parse(&world.e2e_config_text()).expect("e2e config")
    } else {
        world.scope_config()
    }
}

fn selected(config: &passuff_config::AppConfig, repository: &str) -> bool {
    config
        .core
        .watch
        .repositories
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(repository))
}

fn mode_label(config: &passuff_config::AppConfig) -> &'static str {
    match config.core.watch.mode {
        WatchMode::All => "all",
        WatchMode::Selected => "selected",
    }
}

#[when(regex = r#"^the developer selects "([^"]+)" in the watch scope$"#)]
async fn selects(world: &mut World, repository: String) {
    if world.e2e {
        world.e2e_run_scope("select", Some(&repository)).await;
    } else {
        let editor = world.scope_editor();
        match editor.select(&repository) {
            Ok(_) => world.sync_scope_config(),
            Err(error) => {
                world.last_transcript = error.message;
                world.last_status = error.exit;
            }
        }
    }
}

#[when(regex = r#"^the developer deselects "([^"]+)" in the watch scope$"#)]
async fn deselects(world: &mut World, repository: String) {
    if world.e2e {
        world.e2e_run_scope("deselect", Some(&repository)).await;
    } else {
        let editor = world.scope_editor();
        match editor.deselect(&repository) {
            Ok(_) => world.sync_scope_config(),
            Err(error) => {
                world.last_transcript = error.message;
                world.last_status = error.exit;
            }
        }
    }
}

#[when(regex = r#"^the developer switches the watch scope to "(all|selected)"$"#)]
async fn switches(world: &mut World, mode: String) {
    let mode = if mode == "all" {
        WatchMode::All
    } else {
        WatchMode::Selected
    };
    if world.e2e {
        let label = if mode == WatchMode::All {
            "all"
        } else {
            "selected"
        };
        world.e2e_run_scope("mode", Some(label)).await;
    } else {
        let editor = world.scope_editor();
        match editor.set_mode(mode) {
            Ok(_) => world.sync_scope_config(),
            Err(error) => {
                world.last_transcript = error.message;
                world.last_status = error.exit;
            }
        }
    }
}

#[when(regex = r#"^the developer enters "([^"]+)" as a repository$"#)]
async fn enters(world: &mut World, repository: String) {
    if world.e2e {
        world.e2e_run_scope("add", Some(&repository)).await;
    } else {
        let editor = world.scope_editor();
        match editor.add(&repository) {
            Ok(_) => world.sync_scope_config(),
            Err(error) => {
                world.last_transcript = error.message;
                world.last_status = error.exit;
            }
        }
    }
}

#[then(regex = r#"^the watch scope mode is "(all|selected)"$"#)]
async fn mode_is(world: &mut World, expected: String) {
    assert_eq!(
        mode_label(&config(world)),
        expected,
        "{}",
        world.last_transcript
    );
}

#[then(regex = r#"^the watch scope contains "([^"]+)"$"#)]
async fn contains(world: &mut World, repository: String) {
    assert!(
        selected(&config(world), &repository),
        "{:?}",
        config(world).core.watch
    );
}

#[then(regex = r#"^the watch scope does not contain "([^"]+)"$"#)]
async fn does_not_contain(world: &mut World, repository: String) {
    assert!(
        !selected(&config(world), &repository),
        "{:?}",
        config(world).core.watch
    );
}

#[then(regex = r#"^the watch scope is empty$"#)]
async fn empty(world: &mut World) {
    assert!(config(world).core.watch.repositories.is_empty());
}

#[then(regex = r#"^the state retains "([^"]+)"$"#)]
async fn state_retains(world: &mut World, key: String) {
    let (repository, number) = key.split_once('#').expect("key");
    let present = if world.e2e {
        world.e2e_state_json()["accounts"]
            .as_object()
            .map(|accounts| {
                accounts.values().any(|account| {
                    account["pull_requests"]
                        .as_object()
                        .is_some_and(|pull_requests| {
                            pull_requests.keys().any(|entry| entry == &key)
                        })
                })
            })
            .unwrap_or(false)
    } else {
        world.state_json()["accounts"]
            .as_object()
            .map(|accounts| {
                accounts.values().any(|account| {
                    account["pull_requests"]
                        .as_object()
                        .is_some_and(|pull_requests| {
                            pull_requests.keys().any(|entry| entry == &key)
                        })
                })
            })
            .unwrap_or(false)
    };
    let _ = (repository, number);
    assert!(present, "state does not retain {key}");
}

#[when(regex = r#"^the developer opens the scope picker$"#)]
async fn opens_picker(world: &mut World) {
    if world.e2e {
        world.e2e_run_scope("picker", None).await;
    } else {
        let config = world.scope_config();
        let editor = world.scope_editor();
        let state = world.state_store().load().await.unwrap_or_default();
        let known = editor.known_repositories(&config, &state);
        world.last_transcript = passuff_cli::scope::ScopePicker::new(&config, known).snapshot();
    }
}

#[then(regex = r#"^the picker shows "([^"]+)"$"#)]
async fn picker_shows(world: &mut World, expected: String) {
    assert!(
        world.last_transcript.contains(&expected),
        "expected picker to contain {expected:?}: {:?}",
        world.last_transcript
    );
}

#[given(regex = r#"^the watch scope mode is "(all|selected)"$"#)]
async fn given_mode(world: &mut World, mode: String) {
    let mode = if mode == "all" {
        WatchMode::All
    } else {
        WatchMode::Selected
    };
    world.watch_scope.clear();
    if world.e2e {
        world
            .e2e_run_scope(
                "mode",
                Some(if mode == WatchMode::All {
                    "all"
                } else {
                    "selected"
                }),
            )
            .await;
    } else {
        let editor = world.scope_editor();
        editor.set_mode(mode).expect("set mode");
        world.sync_scope_config();
    }
}

#[given(regex = r#"^the watch scope contains "([^"]+)"$"#)]
async fn given_contains(world: &mut World, repository: String) {
    if world.e2e {
        world.e2e_run_scope("select", Some(&repository)).await;
    } else {
        let editor = world.scope_editor();
        editor.select(&repository).expect("select");
        world.sync_scope_config();
    }
}

#[given(regex = r#"^the watch scope does not contain "([^"]+)"$"#)]
async fn given_does_not_contain(world: &mut World, repository: String) {
    let config = config(world);
    if selected(&config, &repository) {
        if world.e2e {
            world.e2e_run_scope("deselect", Some(&repository)).await;
        } else {
            let editor = world.scope_editor();
            editor.deselect(&repository).expect("deselect");
            world.sync_scope_config();
        }
    }
}

#[given(regex = r#"^a watcher is running with a configuration path$"#)]
async fn watcher_running(world: &mut World) {
    world.ensure_scope_config();
    let path = world.scope_config_path();
    world.watcher_config_path = Some(path.clone());
    world.run_watch_cycle_from_path(path).await;
}

#[when(regex = r#"^the watcher runs another cycle$"#)]
async fn watcher_another_cycle(world: &mut World) {
    let path = world
        .watcher_config_path
        .clone()
        .expect("watcher config path");
    world.run_watch_cycle_from_path(path).await;
}

#[when(regex = r#"^the configuration file is corrupted$"#)]
async fn corrupt_config(world: &mut World) {
    let path = world.scope_config_path();
    std::fs::write(path, "{ not toml").expect("corrupt config");
}

#[given(regex = r#"^passuff runs one check$"#)]
async fn given_check(world: &mut World) {
    if world.e2e {
        world.e2e_run_check().await;
    } else {
        world.run_check_internal().await;
    }
}

#[given(regex = r#"^the configuration directory cannot be written$"#)]
async fn readonly_config(world: &mut World) {
    let directory = world
        .state_dir
        .as_ref()
        .expect("state dir")
        .path()
        .to_path_buf();
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o500))
        .expect("readonly config");
}

#[then(regex = r#"^status shows "([^"]+)" in the scope$"#)]
async fn status_scope(world: &mut World, expected: String) {
    world.run_status().await;
    assert!(
        world.last_transcript.contains(&expected),
        "{}",
        world.last_transcript
    );
}

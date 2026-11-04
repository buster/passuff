use cucumber::{then, when};
use regex::Regex;

use crate::support::World;

#[when(regex = r#"^passuff starts watching$"#)]
async fn starts_watching(world: &mut World) {
    world.e2e_start_watching().await;
}

#[when(regex = r#"^passuff is stopped$"#)]
async fn stopped(world: &mut World) {
    world.e2e_stop().await;
}

#[when(regex = r#"^the developer runs passuff with "([^"]+)"$"#)]
async fn developer_runs_with(world: &mut World, flag: String) {
    world.e2e_run_with(&flag).await;
}

#[then(regex = r#"^the terminal shows a notification for "([^"]+)"$"#)]
async fn terminal_shows_notification(world: &mut World, key: String) {
    world.e2e_wait_terminal_notification(&key).await;
}

#[then(regex = r#"^the transcript shows the available commands$"#)]
async fn transcript_shows_commands(world: &mut World) {
    let transcript = &world.last_transcript;
    assert!(
        transcript.contains("Commands:") && transcript.contains("check"),
        "expected usage with commands, got {transcript:?}"
    );
}

#[then(regex = r#"^the transcript shows the version$"#)]
async fn transcript_shows_version(world: &mut World) {
    let transcript = world.last_transcript.trim();
    let pattern = Regex::new(r"^passuff \d+\.\d+\.\d+ \(☞ﾟヮﾟ\)☞$").expect("version regex");
    assert!(
        pattern.is_match(transcript),
        "expected a version line, got {transcript:?}"
    );
}

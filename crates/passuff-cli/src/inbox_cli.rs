use std::process::ExitCode;
use std::sync::Arc;

use passuff_config::{AppConfig, ConfigError};
use passuff_core::{CoreError, InboxService};

use crate::transcript::TranscriptRenderer;
use crate::RunOutcome;

pub struct InboxCli {
    inbox: Arc<dyn InboxService>,
    renderer: TranscriptRenderer,
}

impl InboxCli {
    pub fn new(inbox: Arc<dyn InboxService>) -> Self {
        Self {
            inbox,
            renderer: TranscriptRenderer,
        }
    }

    fn config_error(&self, error: ConfigError) -> RunOutcome {
        RunOutcome {
            exit: ExitCode::from(2),
            status: 2,
            report: None,
            transcript: self.renderer.render_config_error(&error.to_string()),
            emitted: false,
        }
    }

    fn state_error(&self, error: CoreError) -> RunOutcome {
        RunOutcome {
            exit: ExitCode::from(3),
            status: 3,
            report: None,
            transcript: self.renderer.render_state_error(&error.to_string()),
            emitted: false,
        }
    }

    fn ok(&self, transcript: String) -> RunOutcome {
        RunOutcome {
            exit: ExitCode::SUCCESS,
            status: 0,
            report: None,
            transcript,
            emitted: false,
        }
    }

    pub async fn status(&self, config: Result<AppConfig, ConfigError>) -> RunOutcome {
        let config = match config {
            Ok(config) => config,
            Err(error) => return self.config_error(error),
        };
        match self.inbox.view(&config.core).await {
            Ok(view) => self.ok(self.renderer.render_status(&view)),
            Err(error) => self.state_error(error),
        }
    }

    pub async fn inbox(&self, config: Result<AppConfig, ConfigError>, all: bool) -> RunOutcome {
        let config = match config {
            Ok(config) => config,
            Err(error) => return self.config_error(error),
        };
        match self.inbox.view(&config.core).await {
            Ok(view) => self.ok(self.renderer.render_inbox(&view, all)),
            Err(error) => self.state_error(error),
        }
    }

    pub async fn read(
        &self,
        config: Result<AppConfig, ConfigError>,
        id: Option<String>,
        all: bool,
    ) -> RunOutcome {
        if let Err(error) = config {
            return self.config_error(error);
        }
        match (id, all) {
            (Some(id), false) => match self.inbox.mark_read(&id).await {
                Ok(outcome) if outcome.found => self.ok(self.renderer.render_read(outcome)),
                Ok(_) => RunOutcome {
                    exit: ExitCode::from(2),
                    status: 2,
                    report: None,
                    transcript: self.renderer.render_unknown_activity(&id),
                    emitted: false,
                },
                Err(error) => self.state_error(error),
            },
            (None, true) => match self.inbox.mark_all_read().await {
                Ok(outcome) => self.ok(self.renderer.render_read(outcome)),
                Err(error) => self.state_error(error),
            },
            _ => RunOutcome {
                exit: ExitCode::from(2),
                status: 2,
                report: None,
                transcript: "passuff: read requires exactly one of an activity id or --all"
                    .to_string(),
                emitted: false,
            },
        }
    }
}

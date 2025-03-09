use std::time::Duration;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::{Res, ResMut, Resource},
    time::{Time, Timer, TimerMode},
};
use tracing::info;

use crate::game::world::WorldConfig;

#[derive(Resource)]
pub struct WorldUpdateTime {
    update_timer:             Timer,
    update_time_threshold:    Duration,
    last_record_time_elapsed: Duration,
}

impl From<&WorldConfig> for WorldUpdateTime {
    /// WorldUpdateTime::LoadFromConfig
    fn from(cfg: &WorldConfig) -> Self {
        Self {
            update_timer:             Timer::new(*cfg.RecordUpdateTimeDiffInterval, TimerMode::Repeating),
            update_time_threshold:    *cfg.MinRecordUpdateTimeDiff,
            last_record_time_elapsed: Duration::ZERO,
        }
    }
}

impl WorldUpdateTime {
    /// WorldUpdateTime::RecordUpdateTime in TC/AC
    pub fn record_update(mut this: ResMut<Self>, time: Res<Time>, diagnostics: Res<DiagnosticsStore>) {
        let diff = time.delta();
        this.update_timer.tick(diff);
        if !this.update_timer.finished() {
            return;
        }
        if diff <= this.update_time_threshold {
            return;
        }
        // gameTimeMs
        let game_time = time.elapsed();
        if this.last_record_time_elapsed - game_time <= this.update_time_threshold {
            return;
        }
        let frame_time_diagnostics = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .expect("please register FrameTimeDiagnosticsPlugin as a plugin in bevy app");

        // TODO: Get session count from number of player sessions
        let session_count = 0;
        let mut fr = frame_time_diagnostics.values().copied().collect::<Vec<_>>();
        fr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let num_diffs = fr.len();
        let average = frame_time_diagnostics.average().unwrap_or(0.0);
        let median = fr[fr.len() / 2];
        let ninety_five_percentile = fr[(fr.len() * 95) / 100];
        let ninety_nine_percentile = fr[(fr.len() * 99) / 2];
        let max = fr[fr.len() - 1];
        info!(target:"time.update", r#"
            Last {num_diffs} diffs summary with {session_count} players online:
                - Mean: {average};
                - Median: {median};
                - Percentiles (95, 99, max): {ninety_five_percentile}, {ninety_nine_percentile}, {max}.
        "#);
        this.last_record_time_elapsed = game_time;
    }
}

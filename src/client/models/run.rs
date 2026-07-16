//! Run models.
//!
//! The data/request/response types are generated from the external OpenAPI spec
//! (`runs-v1.yaml`) and re-exported here. Only presentation/behavior impls
//! (`Tabular`, etc.) are hand-written in this file — keep them separate from the
//! generated `crate::client::generated::runs` module so a regen never clobbers them.

// Re-export only the types referenced by name (client + commands). Field-only
// types (Progress, Results, MetricSummary) stay reachable via the generated
// module but are kept out of the `models::*` glob, and the generic error types
// (Error/ErrorDetail/...) are deliberately not re-exported to avoid name
// collisions across per-resource generated modules.
pub use crate::client::generated::runs::{
    GetRunResponse, LaunchMetadata, LaunchOptions, LaunchRunRequest, LaunchRunResponse,
    ListRunsResponse, Run, RunStatus, UpdateRunRequest, UpdateRunResponse,
};

use crate::output::Tabular;

impl Tabular for Run {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "STATUS", "PROGRESS", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        let progress = self.progress.as_ref().map_or_else(
            || "-".to_string(),
            |p| format!("{}/{}", p.completed_test_cases, p.total_test_cases),
        );

        vec![
            self.run_id.clone(),
            self.status.to_string(),
            progress,
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
        ]
    }
}

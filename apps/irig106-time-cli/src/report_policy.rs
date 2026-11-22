//! Partial Traces: L3-CLI-106 <- L2-CLI-106 <- L1-CLI-006.
//! Effective CLI compatibility policy, not a fabricated library SharedPolicy.
use super::*;
use serde_json::{Value, json};
pub(super) fn value(r: &report::Report<'_>) -> Value {
    let table = r.leap_table.metadata();
    json!({"id":"telemetryworks/irigtime-cli-nearest-v1","defaults_id":"telemetryworks/irigtime-cli-defaults-v1",
        "reference_selection":"nearest-reference-counter-offset","drift_correction":false,
        "reference_window_ns":r.reference_window_ns.to_string(),"reference_cap":REFERENCE_CAP.get().to_string(),
        "query_reference_storage":"target-brackets","query_candidates_per_channel":"6",
        "reset_backward_step_ns":(RESET_BACKWARD_TICKS*100).to_string(),"reset_backward_comparison":"strictly-greater",
        "reset_run_references":RESET_RUN_REFERENCES.to_string(),"reset_run_order":"strictly-increasing-behind-baseline",
        "leap_table":{"id":table.id,"source":table.source,
            "supported_start_unix_s":table.supported_start.get().to_string(),
            "known_current_through_unix_s":table.known_current_through.get().to_string(),
            "entries":r.leap_table.len().to_string()}})
}

use anyhow::Result;
use tracing::{error, info};

use crate::tsf::{search_candidate_provider::SearchCandidateProvider, set_thread_local_input_settings};

pub fn check_tsf_availability() -> Result<bool> {
    info!("Checking TSF availability");

    if let Err(e) = set_thread_local_input_settings(true) {
        error!("Failed to set thread local input settings: {:?}", e);
        return Ok(false);
    }

    match SearchCandidateProvider::create() {
        Ok(_) => {
            info!("TSF is available");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to create SearchCandidateProvider: {:?}", e);
            Ok(false)
        }
    }
}
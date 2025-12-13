use anyhow::Result;
use tracing::{debug, error, info};
#[cfg(target_os = "windows")]
use windows::{
    core::Interface,
    Win32::UI::TextServices::{ITfFnSearchCandidateProvider, ITfFunctionProvider},
};

use super::search_candidate_provider::SearchCandidateProvider;

pub struct FunctionProvider {
    #[cfg(target_os = "windows")]
    function_provider: ITfFunctionProvider,
}

#[cfg(target_os = "windows")]
impl FunctionProvider {
    pub fn new(function_provider: ITfFunctionProvider) -> Self {
        debug!("Creating new FunctionProvider");
        Self { function_provider }
    }

    pub fn get_search_candidate_provider(&self) -> Result<SearchCandidateProvider> {
        debug!("Getting search candidate provider");
        let zeroed_guid = windows_core::GUID::zeroed();
        match unsafe {
            self.function_provider
                .GetFunction(&zeroed_guid, &ITfFnSearchCandidateProvider::IID)
        } {
            Ok(search_candidate_provider) => {
                info!("Search candidate provider obtained successfully");
                match search_candidate_provider.cast() {
                    Ok(provider) => {
                        debug!("Successfully cast search candidate provider");
                        Ok(SearchCandidateProvider::new(provider))
                    }
                    Err(e) => {
                        error!("Failed to cast search candidate provider: {:?}", e);
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                error!("Failed to get search candidate provider: {:?}", e);
                Err(e.into())
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl FunctionProvider {
    pub fn new() -> Self {
        debug!("FunctionProvider is not implemented on this OS");
        Self {}
    }
}

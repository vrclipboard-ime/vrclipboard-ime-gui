use anyhow::Result;
use tracing::{debug, error, info, trace};
#[cfg(target_os = "windows")]
use windows::Win32::UI::TextServices::{
    ITfFnSearchCandidateProvider, TF_TMAE_NOACTIVATEKEYBOARDLAYOUT,
};

use super::{
    function_provider::FunctionProvider, input_processor_profile_mgr::InputProcessorProfileMgr,
    thread_mgr::ThreadMgr,
};

pub struct SearchCandidateProvider {
    #[cfg(target_os = "windows")]
    search_candidate_provider: ITfFnSearchCandidateProvider,
}

#[cfg(target_os = "windows")]
impl SearchCandidateProvider {
    pub fn new(search_candidate_provider: ITfFnSearchCandidateProvider) -> Self {
        debug!("Creating new SearchCandidateProvider");
        Self {
            search_candidate_provider,
        }
    }

    pub fn create() -> Result<Self> {
        info!("Creating SearchCandidateProvider");
        let profile_mgr = InputProcessorProfileMgr::new()?;
        let profile = profile_mgr.get_active_profile()?;
        debug!("Activating profile");
        profile_mgr.activate_profile(&profile)?;

        debug!("Creating ThreadMgr");
        let thread_mgr = ThreadMgr::new()?;
        let _client_id = thread_mgr.activate_ex(TF_TMAE_NOACTIVATEKEYBOARDLAYOUT)?;

        debug!("Getting function provider");
        let function_provider = thread_mgr.get_function_provider(&profile.clsid)?;

        debug!("Getting search candidate provider");
        let search_candidate_provider =
            FunctionProvider::new(function_provider).get_search_candidate_provider()?;

        info!("SearchCandidateProvider created successfully");
        Ok(search_candidate_provider)
    }

    pub fn get_candidates(&self, input: &str, max: usize) -> Result<Vec<String>> {
        debug!("Getting candidates for input: {}, max: {}", input, max);
        let input_utf16: Vec<u16> = input.encode_utf16().chain(Some(0)).collect();
        let input_bstr = windows_core::BSTR::from_wide(&input_utf16)?;

        let input_utf16: Vec<u16> = "".encode_utf16().chain(Some(0)).collect();
        let input_bstr_empty = windows_core::BSTR::from_wide(&input_utf16)?;

        trace!("Calling GetSearchCandidates");
        let candidates = unsafe {
            self.search_candidate_provider
                .GetSearchCandidates(&input_bstr, &input_bstr_empty)?
        };
        let candidates_enum = unsafe { candidates.EnumCandidates()? };

        let mut candidates = vec![None; max];
        let mut candidates_count = 0;
        trace!("Enumerating candidates");
        unsafe { candidates_enum.Next(&mut candidates, &mut candidates_count)? };

        candidates.resize(candidates_count as usize, None);

        let candidates: Vec<String> = candidates
            .iter()
            .map(|candidate| unsafe {
                match candidate.as_ref().unwrap().GetString() {
                    Ok(s) => s.to_string(),
                    Err(e) => {
                        error!("Failed to get candidate string: {:?}", e);
                        String::new()
                    }
                }
            })
            .collect();
        info!("Retrieved {} candidates", candidates.len());
        Ok(candidates)
    }
}

#[cfg(not(target_os = "windows"))]
impl SearchCandidateProvider {
    pub fn new() -> Self {
        debug!("SearchCandidateProvider is not implemented on this OS");
        Self {}
    }
}

use anyhow::Result;
use tracing::{debug, error, info};
#[cfg(target_os = "windows")]
use windows::Win32::{
    System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
    UI::TextServices::{
        CLSID_TF_InputProcessorProfiles, ITfInputProcessorProfileMgr, GUID_TFCAT_TIP_KEYBOARD, HKL,
        TF_INPUTPROCESSORPROFILE, TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE,
        TF_PROFILETYPE_INPUTPROCESSOR,
    },
};

pub struct InputProcessorProfileMgr {
    #[cfg(target_os = "windows")]
    input_processor_profile_mgr: ITfInputProcessorProfileMgr,
}

#[cfg(target_os = "windows")]
impl InputProcessorProfileMgr {
    pub fn new() -> Result<Self> {
        debug!("Creating new InputProcessorProfileMgr");
        let input_processor_profile_mgr = unsafe {
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?
        };
        info!("InputProcessorProfileMgr created successfully");
        Ok(InputProcessorProfileMgr {
            input_processor_profile_mgr,
        })
    }

    pub fn get_active_profile(&self) -> Result<TF_INPUTPROCESSORPROFILE> {
        debug!("Getting active profile");
        let keyboard_guid = GUID_TFCAT_TIP_KEYBOARD;
        let mut profile = TF_INPUTPROCESSORPROFILE::default();

        match unsafe {
            self.input_processor_profile_mgr
                .GetActiveProfile(&keyboard_guid, &mut profile)
        } {
            Ok(_) => {
                info!("Active profile retrieved successfully");
                Ok(profile)
            }
            Err(e) => {
                error!("Failed to get active profile: {:?}", e);
                Err(e.into())
            }
        }
    }

    pub fn activate_profile(&self, profile: &TF_INPUTPROCESSORPROFILE) -> Result<()> {
        debug!("Activating profile: {:?}", profile);
        match unsafe {
            self.input_processor_profile_mgr.ActivateProfile(
                TF_PROFILETYPE_INPUTPROCESSOR,
                profile.langid,
                &profile.clsid,
                &profile.guidProfile,
                HKL::default(),
                TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE,
            )
        } {
            Ok(_) => {
                info!("Profile activated successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to activate profile: {:?}", e);
                Err(e.into())
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl InputProcessorProfileMgr {
    pub fn new() -> Result<Self> {
        debug!("InputProcessorProfileMgr is not implemented on this OS");
        Err(anyhow::anyhow!(
            "InputProcessorProfileMgr is not implemented on this OS"
        ))
    }
}

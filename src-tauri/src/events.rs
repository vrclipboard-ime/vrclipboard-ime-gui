use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionLog {
    pub time: String,
    pub original: String,
    pub converted: String,
}

#[derive(Debug, Clone)]
pub struct TraceLog {
    pub level: String,
    pub message: String,
    pub module_path: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Conversion(ConversionLog),
    Trace(TraceLog),
}

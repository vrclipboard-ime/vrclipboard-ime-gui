use azookey_binding::Candidate;
use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};

pub mod azookey_conversion;
pub mod client;
pub mod server;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IpcMessage {
    Start,
    Sender(IpcSender<IpcMessage>),
    ResetComposingText,
    InsertAtCursorPosition(String),
    RequestCandidates(String),
    Candidates(Vec<Candidate>),
    End,
}

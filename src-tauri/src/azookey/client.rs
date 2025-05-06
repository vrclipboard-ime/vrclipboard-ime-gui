use azookey_binding::Candidate;
use ipc_channel::ipc::{IpcReceiver, IpcSender};

use tracing::info;

use super::IpcMessage;

pub struct AzookeyConversionClient {
    pub sender: IpcSender<IpcMessage>,
    pub receiver: IpcReceiver<IpcMessage>,
}

impl AzookeyConversionClient {
    pub fn new(server_name: String) -> Self {
        println!("Creating new AzookeyConversionClient instance");

        let sender = IpcSender::connect(server_name.clone()).unwrap();
        let (server_sender, receiver) = ipc_channel::ipc::channel().unwrap();

        sender.send(IpcMessage::Start).unwrap();
        sender.send(IpcMessage::Sender(server_sender)).unwrap();

        Self { sender, receiver }
    }

    pub fn reset_composing_text(&mut self) {
        info!("Resetting composing text");
        self.sender.send(IpcMessage::ResetComposingText).unwrap();
    }

    pub fn insert_at_cursor_position(&mut self, text: &str) {
        info!("Inserting at cursor position: {}", text);
        self.sender
            .send(IpcMessage::InsertAtCursorPosition(text.to_string()))
            .unwrap();
    }

    pub fn request_candidates(&mut self, context: &str) -> Vec<Candidate> {
        info!("Requesting candidates for context: {}", context);
        self.sender
            .send(IpcMessage::RequestCandidates(context.to_string()))
            .unwrap();
        loop {
            match self.receiver.recv() {
                Ok(IpcMessage::Candidates(candidates)) => {
                    info!("Received candidates: {:?}", candidates);
                    return candidates;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        vec![]
    }
}

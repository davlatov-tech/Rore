use crate::state::NodeId;
use rore_types::Style;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

pub enum UICommand {
    SetColor(String, [f32; 4]),
    UpdateText(NodeId, String),
    MarkDirty(NodeId, u8),
    RebuildNode(NodeId, u32),
    UpdateStyle(NodeId, Style),
    UpdateResource(u64, Box<dyn std::any::Any + Send>),
    UpdateTransform(NodeId, f32, f32),
}

pub static COMMAND_SENDER: OnceLock<Sender<UICommand>> = OnceLock::new();
pub static COMMAND_RECEIVER: OnceLock<Mutex<Receiver<UICommand>>> = OnceLock::new();

pub struct CommandQueue;

impl CommandQueue {
    pub fn init() {
        let (tx, rx) = mpsc::channel();
        COMMAND_SENDER.set(tx).ok();
        COMMAND_RECEIVER.set(Mutex::new(rx)).ok();
    }

    pub fn send(cmd: UICommand) {
        if let Some(sender) = COMMAND_SENDER.get() {
            let _ = sender.send(cmd);
        }
    }
}

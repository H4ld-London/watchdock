mod listen;
mod rpc;
mod trigger;

use clap::{ArgAction, Args};
use tokio::process::Command;

pub use crate::{
    listen::listen,
    rpc::{ClientCmd, ServerCmd},
    trigger::{trigger, trigger_run},
};

#[derive(Clone, Args)]
pub struct CliCommand {
    pub command: String,
    #[arg(action=ArgAction::Append)]
    pub args: Vec<String>,
}

impl CliCommand {
    pub fn to_command(&self) -> Command {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);
        cmd
    }
}

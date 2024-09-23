use std::{path::PathBuf, process::exit};

use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::{select, signal::ctrl_c};

use watchdock::{listen, trigger, trigger_run, CliCommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: CliCmd,
}

#[derive(Subcommand)]
enum CliCmd {
    /// Listen for trigger on unix socket and run command.
    /// Usually run inside docker.
    Listen {
        /// Socket on which to listen.
        socket: PathBuf,
        /// Command to run upon receiving trigger.
        #[command(flatten)]
        command: CliCommand,
    },
    /// Send trigger to unix socket then exit.
    /// Usually run outside docker.
    Trigger {
        /// Socket to send trigger.
        socket: PathBuf,
    },
    /// Send trigger to unix socket and await result, run command if successful.
    /// Usually run outside docker.
    TriggerRun {
        /// Socket to send trigger.
        socket: PathBuf,
        /// Command to run upon successful listen command.
        #[command(flatten)]
        command: CliCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let cli = Cli::parse();

    match cli.cmd {
        CliCmd::Listen { socket, command } => {
            select! {
                _ = ctrl_c() => {}
                res = listen(socket, command) => res?,
            }
        }
        CliCmd::Trigger { socket } => {
            select! {
                _ = ctrl_c() => {}
                res = trigger(socket) => res?,
            }
        }
        CliCmd::TriggerRun { socket, command } => {
            select! {
                _ = ctrl_c() => {}
                res = trigger_run(socket, command) => {
                    if let Some(code) = res? {
                        exit(code);
                    }
                }
            }
        }
    }

    Ok(())
}

fn init_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let env_filter_layer = EnvFilter::builder()
        .with_default_directive("info".parse().unwrap())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer().with_line_number(true))
        .with(env_filter_layer)
        .init();
}

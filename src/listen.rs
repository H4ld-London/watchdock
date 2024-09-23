use std::{io, path::PathBuf, process::ExitStatus};

use anyhow::Result;
use futures::prelude::*;
use tokio::{
    net::{unix::SocketAddr, UnixSocket, UnixStream},
    process::Child,
    select,
};
use tokio_serde::{formats::MessagePack, Framed};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};
use tracing::debug;

use crate::{
    rpc::{ClientCmd, ServerCmd},
    CliCommand,
};

pub async fn listen(socket: PathBuf, command: CliCommand) -> Result<()> {
    if socket.exists() {
        tokio::fs::remove_file(&socket).await?;
    }
    let unix_socket = UnixSocket::new_stream()?;
    unix_socket.bind(&socket)?;
    let listener = unix_socket.listen(2)?;
    debug!("Listening");
    let mut child = None;

    loop {
        select! {
            res = listener.accept() => handle(res?, &command, &mut child).await?,
        }
    }
}

async fn handle(
    (stream, _): (UnixStream, SocketAddr),
    command: &CliCommand,
    child: &mut Option<(Child, ServerCmd)>,
) -> Result<()> {
    debug!("Handling");
    let inner = LengthDelimitedCodec::new().framed(stream);
    let mut framed: Framed<_, ServerCmd, ClientCmd, _> = Framed::new(inner, MessagePack::default());

    loop {
        select! {
            res = framed.try_next() => {
                let Some(cmd) = res? else { break };
                debug!("cmd: {cmd:?}");
                let mut command = command.to_command();
                *child = Some((command.kill_on_drop(true).spawn()?, cmd));
                if cmd == ServerCmd::Trigger {
                    break;
                }
            }
            (res, cmd) = child.wait() => {
                debug!("cmd result: {res:?} {cmd:?}");
                if cmd == ServerCmd::TriggerWithResult {
                    framed.send(ClientCmd::Result { success: res?.success() }).await?;
                }
            }
        }
    }
    eprintln!("exiting");

    Ok(())
}

trait OptionChildExt {
    async fn wait(&mut self) -> (io::Result<ExitStatus>, ServerCmd);
}

impl OptionChildExt for Option<(Child, ServerCmd)> {
    async fn wait(&mut self) -> (io::Result<ExitStatus>, ServerCmd) {
        if let Some((child, cmd)) = self {
            (child.wait().await, *cmd)
        } else {
            future::pending().await
        }
    }
}

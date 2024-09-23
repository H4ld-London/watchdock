use std::{os::unix::process::ExitStatusExt, path::PathBuf};

use anyhow::Result;
use futures::prelude::*;
use tokio::net::UnixStream;
use tokio_serde::{formats::MessagePack, Framed};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};
use tracing::debug;

use crate::{
    rpc::{ClientCmd, ServerCmd},
    CliCommand,
};

pub async fn trigger(socket: PathBuf) -> Result<()> {
    let stream = UnixStream::connect(&socket).await?;
    debug!("Connected");
    let inner = LengthDelimitedCodec::new().framed(stream);
    let mut framed: Framed<_, ClientCmd, ServerCmd, _> =
        Framed::new(inner, MessagePack::<ClientCmd, ServerCmd>::default());

    framed.send(ServerCmd::Trigger).await?;
    debug!("Sent");

    Ok(())
}

pub async fn trigger_run(socket: PathBuf, command: CliCommand) -> Result<Option<i32>> {
    let stream = UnixStream::connect(&socket).await?;
    debug!("Connected");
    let mut framed: Framed<_, ClientCmd, ServerCmd, _> = Framed::new(
        LengthDelimitedCodec::new().framed(stream),
        MessagePack::default(),
    );

    framed.send(ServerCmd::TriggerWithResult).await?;
    debug!("Sent, waiting...");

    if framed.try_next().await? == Some(ClientCmd::Result { success: true }) {
        drop(framed);
        debug!("Received successful");

        let status = command.to_command().kill_on_drop(true).status().await?;

        debug!("Command exited status: {status:?}");

        if !status.success() {
            return match (status.code(), status.signal()) {
                (Some(code), _) => Ok(Some(code)),
                (None, Some(signal)) => Ok(Some(signal.saturating_add(255))),
                (None, None) => Ok(Some(250)),
            };
        }
    }
    Ok(None)
}

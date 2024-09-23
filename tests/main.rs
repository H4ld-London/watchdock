use std::{io, process::Stdio, time::Duration};

use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    time::sleep,
};
use watchdock::CliCommand;

const TAG: &str = "watchdock:test-integration";

#[tokio::test]
async fn integration() {
    fs::create_dir_all("./target/test-integration")
        .await
        .unwrap();

    assert!(Command::new("docker")
        .args([
            "build",
            "--progress",
            "plain",
            "--file",
            "tests/Dockerfile",
            "--tag",
            TAG,
            "."
        ])
        .status()
        .await
        .unwrap()
        .success());

    let mut child = Command::new("docker")
        .args([
            "run",
            "-v",
            "./target/test-integration:/run/test",
            "-i",
            TAG,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = tokio::spawn(read_std(child.stdout.take().expect("stdout")));
    let stderr = tokio::spawn(read_std(child.stderr.take().expect("stderr")));

    sleep(Duration::from_secs(1)).await;

    let res = watchdock::trigger_run(
        "./target/test-integration/sock".into(),
        CliCommand {
            command: "echo".into(),
            args: vec![],
        },
    )
    .await
    .unwrap();

    child.kill().await.unwrap();
    child.wait().await.unwrap();
    let _stdout = stdout.await.unwrap().unwrap();
    let _stderr = stderr.await.unwrap().unwrap();

    // tokio::io::stdout().write_all(&stdout).await.unwrap();
    // tokio::io::stderr().write_all(&stderr).await.unwrap();

    assert_eq!(res, None);
}

async fn read_std(
    mut reader: impl AsyncRead + Unpin, /* , mut writer: impl AsyncWrite + Unpin */
) -> io::Result<Vec<u8>> {
    let mut out = vec![];
    let mut buf = [0; 4096];
    loop {
        let len = reader.read(&mut buf).await?;
        if len == 0 {
            break;
        }
        // writer.write_all(&buf[0..len]).await?;
        out.extend(&buf[0..len]);
    }
    Ok(out)
}

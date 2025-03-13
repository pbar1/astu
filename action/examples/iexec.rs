use std::io;
use std::io::Read;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::thread;

use russh::ChannelMsg;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::ReadBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

// ssh stuff ------------------------------------------------------------------

#[derive(Debug, Default)]
struct Client {}

impl russh::client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

// stdin stuff ----------------------------------------------------------------

/// A struct that provides async access to stdin using a background thread.
pub struct AsyncStdin {
    rx: Receiver<u8>,
}

impl AsyncStdin {
    /// Creates a new `AsyncStdin` instance and starts a background reader
    /// thread.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);

        thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut buffer = [0u8; 1024];

            while let Ok(n) = handle.read(&mut buffer) {
                if n == 0 {
                    break; // EOF
                }
                for &byte in &buffer[..n] {
                    if tx.blocking_send(byte).is_err() {
                        return; // Channel closed, exit thread
                    }
                }
            }
        });

        Self { rx }
    }
}

impl AsyncRead for AsyncStdin {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        while let Ok(byte) = self.rx.try_recv() {
            if buf.remaining() > 0 {
                buf.put_slice(&[byte]);
            } else {
                break;
            }
        }

        if buf.filled().is_empty() {
            // No data available, register waker and poll again later
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

// main -----------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = russh::client::Config::default();
    let config = Arc::new(config);
    let client = Client::default();
    let mut session = russh::client::connect(config, "127.0.0.1:2222", client).await?;
    println!("connected");

    let authres = session.authenticate_password("user", "password").await?;
    if !authres.success() {
        anyhow::bail!("failed to auth with password");
    }
    println!("authed with password");

    let mut channel = session.channel_open_session().await?;
    channel
        .request_pty(
            false,
            &std::env::var("TERM").unwrap_or("xterm".into()),
            100, // source using termion, might want to be resized
            100, // "
            0,
            0,
            &[], // ideally you want to pass the actual terminal modes here
        )
        .await?;
    println!("requested channel pty");
    channel.request_shell(true).await?;
    println!("requested channel shell");

    crossterm::terminal::enable_raw_mode()?;
    println!("raw mode enabled");

    let code;
    let mut stdin = AsyncStdin::new();
    let mut stdout = tokio::io::stdout();
    let mut _stderr = tokio::io::stderr();
    let mut buf = vec![0; 1024];
    let mut stdin_closed = false;

    loop {
        // Handle one of the possible events:
        tokio::select! {
            // There's terminal input available from the user
            r = stdin.read(&mut buf), if !stdin_closed => {
                match r {
                    Ok(0) => {
                        stdin_closed = true;
                        channel.eof().await?;
                    },
                    // Send it to the server
                    Ok(n) => channel.data(&buf[..n]).await?,
                    Err(e) => return Err(e.into()),
                };
            },
            // There's an event available on the session channel
            Some(msg) = channel.wait() => {
                match msg {
                    // Write data to the terminal
                    ChannelMsg::Data { ref data } => {
                        stdout.write_all(data).await?;
                        stdout.flush().await?;
                    }
                    // The command has returned an exit code
                    ChannelMsg::ExitStatus { exit_status } => {
                        code = exit_status;
                        if !stdin_closed {
                            channel.eof().await?;
                        }
                        break;
                    }
                    _ => {}
                }
            },
        }
    }

    println!("broke out of interactive loop, exit code = {code}");

    crossterm::terminal::disable_raw_mode()?;
    // TODO: gets printed far to the right
    println!("disabled raw mode");

    Ok(())
}

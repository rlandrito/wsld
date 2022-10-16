use std::convert::TryFrom;

use super::config::X11Config;
use super::util::{connect_stream, either};
use super::vmsocket::VmSocket;
use super::x11socket::X11Lock;
use super::CONFIG;

use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

async fn handle_stream(mut stream: UnixStream, display: u8) -> std::io::Result<()> {
    println!("{}", display);

    let mut server = VmSocket::connect(CONFIG.service_port).await?;
    server.write_all(b"x11\0").await?;
    server.write_u8(display).await?;

    let (client_r, client_w) = stream.split();
    let (server_r, server_w) = server.split();
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
}

pub async fn x11_forward(config: &'static X11Config, display: u32) -> std::io::Result<()> {
    let lock = X11Lock::acquire(display, config.force)?;
    let listener = lock.bind()?;

    let display_u8 = u8::try_from(display).ok().unwrap();

    loop {
        let stream = listener.accept().await?.0;

        tokio::task::spawn(async move {
            if let Err(err) = handle_stream(stream, display_u8).await {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}

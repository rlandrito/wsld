use std::convert::TryInto;

use super::config::X11Config;
use super::util::{connect_stream, either};
use super::vmsocket::VmSocket;
use super::x11socket::X11Lock;
use super::CONFIG;

use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

fn get_display_num(stream: &UnixStream) -> Option<u8> {
    let local_addr = stream.local_addr().ok()?;
    let path = match local_addr.as_pathname() {
        Some(path) => path,
        None => todo!(),
    };
    let path_str = match path.to_str() {
        Some(string) => string,
        None => todo!()
    };
    println!("{}", path_str);

    assert!(path_str.chars().count() > 0);
    let display_str = path_str.chars().last().unwrap();

    let display: u8 = display_str.to_digit(10).unwrap().try_into().unwrap();
    assert!(display <=9);

    return Some(display);
}

async fn handle_stream(mut stream: UnixStream) -> std::io::Result<()> {
    let mut server = VmSocket::connect(CONFIG.service_port).await?;

    let display = get_display_num(&stream).unwrap();
    //if display != 0 { return Ok(()); }

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

    loop {
        let stream = listener.accept().await?.0;

        tokio::task::spawn(async move {
            if let Err(err) = handle_stream(stream).await {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}

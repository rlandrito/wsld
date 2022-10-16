use super::util::{connect_stream, either};
use super::CONFIG;

use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;

pub async fn handle_x11(mut stream: TcpStream) -> std::io::Result<()> {
    let display_num = stream.read_u8().await? as u16;
    // TODO Error check me
    //if display_num > 9 {
    //    return Err(());
    //}
    let base_display_hostport_vec: Vec<&str> = CONFIG.x11.display.split(":").collect();
    let base_port: u16 = base_display_hostport_vec[1].parse().unwrap();
    let dest_port = display_num + base_port;
    let display_host_port = format!("{}:{}", base_display_hostport_vec[0], dest_port);

    let (client_r, client_w) = stream.split();

    let mut server = TcpStream::connect(display_host_port).await?;
    server.set_nodelay(true)?;
    let (server_r, server_w) = server.split();
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
}

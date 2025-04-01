use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use crate::config::ForwardTarget;

pub(super) fn create_proxy(
    runtime: &Runtime,
    listen_port: u16,
    kill_rx: Receiver<()>,
) -> JoinHandle<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], listen_port));

    runtime.spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();

        let kill_signal = create_kill_signal(kill_rx);
        let mut kill_signal = std::pin::pin!(kill_signal);

        let ForwardTarget{ domain, port } = super::get_target();

        let target_socket = format!("{domain}:{port}");

        let mut forward_addr = target_socket.to_socket_addrs().expect("Invalid domain")
            .next()
            .expect("No address found");
        forward_addr.set_port(port);

        loop {
            tokio::select! {
                Ok((mut inbound, _addr)) = listener.accept() => {
                    tokio::spawn(async move {
                        match TcpStream::connect(forward_addr).await {
                            Ok(mut outbound) => {
                                let (from_client, from_server) = tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await.unwrap();

                                println!(
                                    "client wrote {} bytes and received {} bytes",
                                    from_client, from_server
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to connect to forward address: {}", e);
                            }
                        }
                    });
                },

                _ = &mut kill_signal => {
                    eprintln!("Graceful shutdown signal received");
                    break;
                }
            }
        }
        println!("All connections Closed");
    })
}

async fn create_kill_signal(mut kill_rx: Receiver<()>) {
    kill_rx.recv().await.expect("Kill signal issue")
}

use hyper::body::Bytes;
use hyper::client::conn::http1::Builder as HttpClientBuilder;
use hyper::server::conn::http1::Builder as HttpServerBuilder;
use hyper::service::service_fn;
use hyper::{Request, Response};

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

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

        let http_server = HttpServerBuilder::new();
        let graceful = GracefulShutdown::new();

        loop {
            tokio::select! {
                Ok((stream, _addr)) = listener.accept() => {
                    let io = TokioIo::new(stream);
                    let conn = http_server.serve_connection(io, service_fn(proxy_service));
                    // watch this connection
                    let fut = graceful.watch(conn);
                    tokio::spawn(async move {
                        if let Err(e) = fut.await {
                            eprintln!("Error serving connection: {:?}", e);
                        }
                    });
                },

                _ = &mut kill_signal => {
                    eprintln!("Graceful shutdown signal received");
                    break;
                }
            }
        }
        graceful.shutdown().await;
        println!("All connections Closed");
    })
}

async fn proxy_service(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let target_port = super::get_target_port();
    let addr = SocketAddr::from(([127, 0, 0, 1], target_port));
    let stream = TcpStream::connect(addr).await.unwrap();

    // send connection refused response
    let io = TokioIo::new(stream);

    let (mut sender, conn) = HttpClientBuilder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(io)
        .await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let resp = sender.send_request(req).await?;
    Ok(resp.map(|b| b.boxed()))
}

async fn create_kill_signal(mut kill_rx: Receiver<()>) {
    kill_rx.recv().await.expect("Kill signal issue")
}

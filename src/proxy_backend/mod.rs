use hyper::body::Bytes;
use hyper::client::conn::http1::Builder;
use hyper::{Request, Response};
use hyper::service::service_fn;
use hyper::server::conn::http1;

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper_util::rt::TokioIo;

use std::sync::mpsc::Receiver as StdReceiver;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};

use lazy_static::lazy_static;

#[derive(Default, Debug)]
pub struct BackEndConfig(pub Option<(u16, u16)>);

lazy_static! {
    static ref TARGET_PORT: Arc<Mutex<Option<u16>>> = Arc::new(Mutex::new(Default::default()));
}

fn get_target_port() -> u16 {
    let read_guard = TARGET_PORT.lock().expect("Cannot lock target port mutex");
    read_guard.unwrap()
}

fn set_target_port(target_port: u16) {
    let mut write_guard = TARGET_PORT.lock().expect("Cannot lock target port mutex");
    *write_guard = Some(target_port);
}

impl BackEndConfig {
    fn off(&self) -> bool {
        self.0.is_none()
    }

    fn on(&self) -> bool {
        self.0.is_some()
    }

    fn listen_port(&self) -> Option<u16> {
        if let Some((listen_port, _)) = self.0 {
            return Some(listen_port);
        }
        None
    }
    fn forward_port(&self) -> Option<u16> {
        if let Some((_, forward_port)) = self.0 {
            return Some(forward_port);
        }
        None
    }

    pub fn validate(&self) -> Result<(), String> {
        match (self.forward_port(), self.listen_port()) {
            (Some(fp), Some(lp)) if fp == lp => Err("Cannot forward to listening port".to_owned()),
            _ => Ok(()),
        }
    }
}

async fn proxy(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let target_port = get_target_port();
    let addr = SocketAddr::from(([127, 0, 0, 1], target_port));
    let stream = TcpStream::connect(addr).await.unwrap();

    // send connection refused response
    let io = TokioIo::new(stream);

    let (mut sender, conn) = Builder::new()
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

pub fn start_backend(rx: StdReceiver<BackEndConfig>) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut is_proxy_running = false;
    let mut curr_shudown_tx: Option<Sender<()>> = None;

    while let Ok(conf) = rx.recv() {
        if conf.off() && is_proxy_running {
            let curr_shudown_tx = curr_shudown_tx.clone();
            runtime.block_on(async move {
                let kill_tx =
                    curr_shudown_tx.expect("Tx for shutting down current sever not found");
                let _ = kill_tx.send(()).await;
            });
            is_proxy_running = false;
        } else if conf.on() {
            let forward_port = conf
                .forward_port()
                .expect("Listening port not set before starting server");
            set_target_port(forward_port);

            if !is_proxy_running {
                let listen_port = conf
                    .listen_port()
                    .expect("Listening port not set before starting server");

                let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
                start_proxy(&runtime, listen_port, shutdown_rx);
                curr_shudown_tx = Some(shutdown_tx);
                is_proxy_running = true;
            }
        }
    }
}

fn start_proxy(runtime: &Runtime, listen_port: u16, kill_signal_rx: Receiver<()>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], listen_port));

    runtime.spawn(async move {
        let kill_signal = server_kill_signal(kill_signal_rx);

        let listener = TcpListener::bind(addr).await.unwrap();
        let mut signal = std::pin::pin!(kill_signal);
        let http = http1::Builder::new();
        let graceful = hyper_util::server::graceful::GracefulShutdown::new();

        loop {
            tokio::select! {
                Ok((stream, _addr)) = listener.accept() => {
                    let io = TokioIo::new(stream);
                    let conn = http.serve_connection(io, service_fn(proxy));
                    // watch this connection
                    let fut = graceful.watch(conn);
                    tokio::spawn(async move {
                        if let Err(e) = fut.await {
                            eprintln!("Error serving connection: {:?}", e);
                        }
                    });
                },

                _ = &mut signal => {
                    eprintln!("Graceful shutdown signal received");
                    break;
                }
            }
        }
        graceful.shutdown().await;
        println!("All connections Closed");
    });
}

async fn server_kill_signal(mut kill_signal_rx: Receiver<()>) {
    kill_signal_rx.recv().await.expect("Kill signal issue")
}

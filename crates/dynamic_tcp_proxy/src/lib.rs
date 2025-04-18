mod config;
mod proxy_handler;

use std::io::Error;
use std::sync::mpsc::{channel, Receiver as StdReceiver, Sender as StdSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Sender};

use lazy_static::lazy_static;
use proxy_handler::create_proxy;

pub use config::{ForwardTarget, ProxyConfig};
use tokio::task::JoinHandle as TokioJoinHandle;

lazy_static! {
    static ref TARGET_PORT: Arc<Mutex<Option<ForwardTarget>>> =
        Arc::new(Mutex::new(Default::default()));
}

fn get_target() -> ForwardTarget {
    let read_guard = TARGET_PORT.lock().expect("Cannot lock target port mutex");
    read_guard.clone().unwrap()
}

fn set_target(target: ForwardTarget) {
    let mut write_guard = TARGET_PORT.lock().expect("Cannot lock target port mutex");
    *write_guard = Some(target);
}

pub struct DynamicProxy(StdSender<ProxyConfig>);

impl DynamicProxy {
    pub fn initiate() -> Result<(Self, JoinHandle<()>), Error> {
        let (update_tx, update_rx): (StdSender<ProxyConfig>, StdReceiver<ProxyConfig>) = channel();

        let handle = thread::Builder::new()
            .name("dynamic_proxy".to_string())
            .spawn(move || initiate_update_observer(update_rx))?;
        Ok((Self(update_tx), handle))
    }

    pub fn update(
        &self,
        config: ProxyConfig,
    ) -> Result<(), std::sync::mpsc::SendError<ProxyConfig>> {
        self.0.send(config)
    }
}

fn initiate_update_observer(update_rx: StdReceiver<ProxyConfig>) {
    let mut running_proxy_thread: Option<TokioJoinHandle<()>> = None;
    let mut proxy_kill_tx: Option<Sender<()>> = None;

    let runtime = Runtime::new().unwrap();
    while let Ok(config) = update_rx.recv() {
        if config.is_off() && running_proxy_thread.is_some() {
            let curr_shudown_tx = proxy_kill_tx.clone();
            runtime.block_on(async move {
                let kill_tx =
                    curr_shudown_tx.expect("Tx for shutting down current sever not found");
                let _ = kill_tx.send(()).await;
            });

            running_proxy_thread = None;
        } else if config.is_on() {
            let forward_port = config
                .forward_port()
                .expect("Listening port not set before starting server");
            set_target(forward_port);

            if running_proxy_thread.is_none() {
                let listen_port = config
                    .listen_port()
                    .expect("Listening port not set before starting server");

                let (new_proxy_kill_tx, new_proxy_kill_rx) = mpsc::channel::<()>(1);
                let handle = create_proxy(&runtime, listen_port, new_proxy_kill_rx);
                running_proxy_thread = Some(handle);
                proxy_kill_tx = Some(new_proxy_kill_tx);
            }
        }
    }

    if let Some(join_handle) = running_proxy_thread {
        runtime.block_on(async move {
            let _ = proxy_kill_tx
                .expect("kill tx should be present")
                .send(())
                .await;
            let _ = join_handle.await;
        });
    }
}

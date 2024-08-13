# DynamicProxy

The `DynamicProxy` crate provides a dynamic proxy implementation in Rust. This crate is designed to allow you to start a proxy that can be reconfigured at runtime. It leverages channels to update the proxy's configuration and spawns a thread to manage the proxy's operation.

## Features

- **Dynamic Proxy Configuration:** Update the proxy's configuration at runtime using a `Sender`.
- **Concurrency:** The proxy runs in its own thread, allowing it to handle requests concurrently.
- **Simple API:** The crate provides an easy-to-use API for starting the proxy and sending configuration updates.

## Usage

### `DynamicProxy::initiate`

The `initiate` method initializes the dynamic proxy and begins its operation. It returns a `DynamicProxy` for updating the proxy's configuration and a `JoinHandle` for the spawned thread managing the proxy.

#### Usage

```rust
use dynamic_tcp_proxy::{DynamicProxy, ProxyConfig};

let (dynamic_proxy, _handle) = DynamicProxy.initiate()?;
let listen_port = 8080;
let forward_to_port = 8081;

// start the proxy
let config = ProxyConfig(Some((listen_port, forward_port)));
dynamic_proxy.update(config)?;

// listen from 8082
let config = ProxyConfig(Some((8082, forward_port)));
dynamic_proxy.update(config)?;

// shut the proxy
let config = ProxyConfig(None);
dynamic_proxy.update(config)?;
```

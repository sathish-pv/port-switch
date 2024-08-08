# DynamicProxy

The `DynamicProxy` crate provides a dynamic proxy implementation in Rust. This crate is designed to allow you to start a proxy that can be reconfigured at runtime. It leverages channels to update the proxy's configuration and spawns a thread to manage the proxy's operation.

## Features

- **Dynamic Proxy Configuration:** Update the proxy's configuration at runtime using a `Sender`.
- **Concurrency:** The proxy runs in its own thread, allowing it to handle requests concurrently.
- **Simple API:** The crate provides an easy-to-use API for starting the proxy and sending configuration updates.

## Usage

### `DynamicProxy::start`

The `start` method initializes the dynamic proxy and begins its operation. It returns a `Sender` for updating the proxy's configuration and a `JoinHandle` for the spawned thread managing the proxy.

#### Signature

```rust
impl DynamicProxy {
    pub fn start(self) -> Result<(Sender<ProxyConfig>, JoinHandle<()>), Error>
}
```

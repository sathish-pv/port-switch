use dynamic_tcp_proxy::{DynamicProxy, ProxyConfig};
use eframe::egui;

mod create;
mod list;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct ForwardPort {
    port: u16,
    name: String,
    #[serde(skip)]
    error: Option<String>,
}

impl PartialEq for ForwardPort {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}

impl Default for ForwardPort {
    fn default() -> Self {
        Self {
            port: 8080,
            name: "New Port".to_owned(),
            error: None,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct App {
    listen_port: u16,
    is_enabled: bool,
    forward_ports: Vec<ForwardPort>,
    active_forward_port: Option<ForwardPort>,
    #[serde(skip)]
    active_page: Pages,
    #[serde(skip)]
    proxy_handle: Option<DynamicProxy>,
    #[serde(skip)]
    error: Option<String>,
}

impl App {
    fn init_state() -> Self {
        Self {
            listen_port: 8080,
            ..Default::default()
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, proxy_handle: DynamicProxy) -> Self {
        let mut init_app_state: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::init_state()
        };
        init_app_state.proxy_handle = Some(proxy_handle);
        init_app_state.update_backend();
        init_app_state
    }
}
#[derive(Default)]
enum Pages {
    #[default]
    List,
    Creation(ForwardPort),
    Edit(usize, ForwardPort),
}

impl App {
    fn update_backend(&mut self) {
        let mut conf = ProxyConfig::default();
        if self.is_enabled && self.active_forward_port.is_some() {
            let listen_port = self.listen_port;
            if let Some(fp) = &self.active_forward_port {
                let forward_port = fp.port;
                conf = ProxyConfig(Some((listen_port, forward_port)));
            }
        }

        match conf.validate() {
            Ok(_) => {
                self.error = None;

                let _ = match &self.proxy_handle {
                    Some(backend) => backend.update(conf),
                    None => panic!("Sender Channel Not found"),
                };
            }
            Err(err_str) => {
                self.error = Some(err_str);
                self.is_enabled = false;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &self.active_page {
            Pages::List => {
                self.list_page(ctx);
            }
            _ => self.creation_page(ctx),
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.proxy_handle = None;
    }
}

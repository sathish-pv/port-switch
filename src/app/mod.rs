use std::sync::mpsc::Sender;

use eframe::egui;

use crate::proxy_backend::BackEndConfig;

mod create;
mod list;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct App {
    listen_port: u16,
    is_enabled: bool,
    forward_ports: Vec<ForwardPort>,
    active_forward_port: Option<ForwardPort>,
    #[serde(skip)]
    active_page: Pages,
    #[serde(skip)]
    update_sender: Option<Sender<BackEndConfig>>,
    #[serde(skip)]
    error: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct ForwardPort {
    port: u16,
    name: String,
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

impl Default for App {
    fn default() -> Self {
        let forto = ForwardPort::default();
        Self {
            listen_port: 8080,
            is_enabled: false,
            forward_ports: vec![forto],
            active_page: Pages::default(),
            active_forward_port: None,
            update_sender: None,
            error: None,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, tx: Sender<BackEndConfig>) -> Self {
        let mut init_app_state: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        init_app_state.update_sender = Some(tx);
        init_app_state.update_hyper();
        init_app_state
    }
}
#[derive(Default)]
enum Pages {
    #[default]
    List,
    // #[serde(skip)]
    Creation(ForwardPort),
    Edit(usize, ForwardPort),
}

impl App {
    fn update_hyper(&mut self) {
        let mut conf = BackEndConfig::default();
        if self.is_enabled && self.active_forward_port.is_some() {
            let listen_port = self.listen_port;
            if let Some(fp) = &self.active_forward_port {
                let forward_port = fp.port;
                conf = BackEndConfig(Some((listen_port, forward_port)));
            }
        }

        match conf.validate() {
            Ok(_) => {
                self.error = None;
                let _ = match &self.update_sender {
                    Some(tx) => tx.send(conf),
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
}

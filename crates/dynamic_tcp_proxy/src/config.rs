#[derive(Default, Debug)]
pub struct ProxyConfig(pub Option<(u16, u16)>);

impl ProxyConfig {
    pub fn off(&self) -> bool {
        self.0.is_none()
    }

    pub fn on(&self) -> bool {
        self.0.is_some()
    }

    pub fn listen_port(&self) -> Option<u16> {
        if let Some((listen_port, _)) = self.0 {
            return Some(listen_port);
        }
        None
    }
    pub fn forward_port(&self) -> Option<u16> {
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

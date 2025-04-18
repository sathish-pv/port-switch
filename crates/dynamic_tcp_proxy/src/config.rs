use serde::{Deserialize, Serialize};

#[derive(Default, Debug)]
pub struct ProxyConfig(pub Option<(u16, ForwardTarget)>);

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ForwardTarget {
    pub domain: String,
    pub port: u16,
}

impl Default for ForwardTarget {
    fn default() -> Self {
        Self {
            domain: "localhost".to_owned(),
            port: 0,
        }
    }
}

impl ForwardTarget {
    pub fn is_external(&self) -> bool {
        self.domain != "localhost"
    }
}

impl ProxyConfig {
    pub fn is_off(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_on(&self) -> bool {
        self.0.is_some()
    }

    pub fn listen_port(&self) -> Option<u16> {
        if let Some((listen_port, _)) = self.0 {
            return Some(listen_port);
        }
        None
    }
    pub fn forward_port(&self) -> Option<ForwardTarget> {
        if let Some((_, forward_port)) = &self.0 {
            return Some(forward_port.clone());
        }
        None
    }

    pub fn validate(&self) -> Result<(), String> {
        match (self.forward_port(), self.listen_port()) {
            (Some(fp), Some(lp)) if fp.domain == "localhost" && fp.port == lp => {
                Err("Cannot forward to listening port".to_owned())
            }
            _ => Ok(()),
        }
    }
}

use crate::{Result, Topic};

pub trait SetTopicAutostart {
    fn set_autostart(&mut self, name: &str, enabled: bool) -> Result<Topic>;
}

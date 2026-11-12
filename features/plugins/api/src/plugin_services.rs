use crate::{DisablePlugin, EnablePlugin, GetPluginStatus, ListPlugins};

pub struct PluginServices {
    pub enable: Box<dyn EnablePlugin>,
    pub list: Box<dyn ListPlugins>,
    pub status: Box<dyn GetPluginStatus>,
    pub disable: Box<dyn DisablePlugin>,
}

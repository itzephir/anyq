use crate::{DisableInstalledPlugin, EnableInstalledPlugin, ListInstalledPlugins};

pub struct PluginCatalog {
    pub list: Box<dyn ListInstalledPlugins>,
    pub enable: Box<dyn EnableInstalledPlugin>,
    pub disable: Box<dyn DisableInstalledPlugin>,
}

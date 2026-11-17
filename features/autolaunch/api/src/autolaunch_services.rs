use crate::{GetAutolaunch, SetAutolaunch};

pub struct AutolaunchServices {
    pub get: Box<dyn GetAutolaunch>,
    pub set: Box<dyn SetAutolaunch>,
}

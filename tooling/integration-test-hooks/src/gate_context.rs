use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Clone)]
pub struct GateContext {
    name: Option<String>,
    target: usize,
    directory: Option<PathBuf>,
    hits: Arc<AtomicUsize>,
}

impl GateContext {
    pub fn new(name: Option<String>, target: usize, directory: Option<PathBuf>) -> Self {
        Self {
            name,
            target,
            directory,
            hits: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) fn select(&self, name: &str) -> Result<Option<PathBuf>, String> {
        if self.name.as_deref() != Some(name)
            || self.hits.fetch_add(1, Ordering::SeqCst) + 1 != self.target
        {
            return Ok(None);
        }

        self.directory
            .clone()
            .map(Some)
            .ok_or_else(|| "checkpoint directory is missing".into())
    }
}

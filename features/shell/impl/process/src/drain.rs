use std::{
    io::Read,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

pub(crate) struct Drain {
    bytes: Arc<Mutex<Vec<u8>>>,
    truncated: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
}

impl Drain {
    pub fn start(mut pipe: impl Read + Send + 'static) -> Self {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let truncated = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicBool::new(false));
        let (saved, overflow, finished) = (bytes.clone(), truncated.clone(), done.clone());
        thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                match pipe.read(&mut buffer) {
                    Ok(0) => {
                        finished.store(true, Ordering::SeqCst);
                        return;
                    }
                    Ok(count) => {
                        let mut bytes = saved.lock().expect("output mutex");
                        let retained = count.min(65536_usize.saturating_sub(bytes.len()));
                        bytes.extend_from_slice(&buffer[..retained]);
                        if retained < count {
                            overflow.store(true, Ordering::SeqCst);
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => return,
                }
            }
        });

        Self {
            bytes,
            truncated,
            done,
        }
    }

    pub fn snapshot(&self) -> (String, bool) {
        let mut text =
            String::from_utf8_lossy(&self.bytes.lock().expect("output mutex")).into_owned();
        let overflow = text.len() > 65536;
        if overflow {
            let mut end = 65536;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
        }

        (text, overflow || self.truncated.load(Ordering::SeqCst))
    }

    pub fn done(&self) -> bool {
        self.done.load(Ordering::SeqCst)
    }
}

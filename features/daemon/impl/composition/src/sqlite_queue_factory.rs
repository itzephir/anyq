use anyq_daemon_api::DaemonServices;
use anyq_daemon_runtime::QueueFactory;
use anyq_queue_api::{Error, ExecutionDirectory, QueuePolicy, Result};
use std::{
    path::Path,
    sync::{
        Arc, Mutex, MutexGuard, TryLockError,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

#[derive(Default)]
pub(crate) struct SqliteQueueFactory {
    initialized: AtomicBool,
    initialization: Mutex<()>,
}

impl QueueFactory for SqliteQueueFactory {
    fn observe(
        &self,
        path: &Path,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Box<dyn anyq_queue_api::ObserveQueue>> {
        anyq_queue_impl::observe_existing(path, cancelled)
    }

    fn open(
        &self,
        path: &Path,
        policy: QueuePolicy,
        cancelled: Arc<AtomicBool>,
        directory: ExecutionDirectory,
    ) -> Result<DaemonServices> {
        let _initialization = self.lock_initialization(&cancelled)?;
        let store = anyq_queue_sqlite::Sqlite::open_with_initializer(
            path,
            cancelled.clone(),
            anyq_plugin_impl::initialize,
        )?
        .with_directory(directory);
        let store = with_fixture_clock(store);
        let mut queue = anyq_queue_impl::compose(store.clone(), policy);
        let work = anyq_queue_impl::work_services(store.clone());
        queue.complete_task = Box::new(crate::barrier_completion::BarrierCompletion {
            begin: work.begin_completion,
            complete: queue.complete_task,
            cancelled,
        });
        let work = anyq_queue_impl::work_services(store);
        let services = DaemonServices {
            queue,
            plugins: anyq_plugin_impl::open(path)?,
            list_work: work.list,
            resolve_work: work.resolve,
        };
        self.initialized.store(true, Ordering::SeqCst);

        Ok(services)
    }

    fn background(&self, path: &Path, shutdown: Arc<AtomicBool>) -> Result<()> {
        let result = self.run_plugins(path, shutdown.clone());
        if matches!(&result, Err(Error::Interrupted)) && shutdown.load(Ordering::SeqCst) {
            return Ok(());
        }

        result
    }
}

impl SqliteQueueFactory {
    fn lock_initialization(&self, cancelled: &AtomicBool) -> Result<MutexGuard<'_, ()>> {
        loop {
            if cancelled.load(Ordering::SeqCst) {
                return Err(Error::Interrupted);
            }
            match self.initialization.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(TryLockError::WouldBlock) => thread::sleep(Duration::from_millis(10)),
                Err(TryLockError::Poisoned(_)) => {
                    return Err(Error::Storage("owner initialization lock poisoned".into()));
                }
            }
        }
    }

    fn run_plugins(&self, path: &Path, shutdown: Arc<AtomicBool>) -> Result<()> {
        let existing = {
            let _initialization = self.lock_initialization(&shutdown)?;
            if self.initialized.load(Ordering::SeqCst) {
                None
            } else {
                match anyq_queue_sqlite::Sqlite::open_existing_for_runtime(
                    path,
                    shutdown.clone(),
                    anyq_plugin_impl::validate_existing,
                )? {
                    anyq_queue_sqlite::RuntimeOpen::Ready(store) => {
                        self.initialized.store(true, Ordering::SeqCst);
                        Some(store)
                    }
                    _ => None,
                }
            }
        };

        while !self.initialized.load(Ordering::SeqCst) {
            if shutdown.load(Ordering::SeqCst) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(10));
        }
        let store = match existing {
            Some(store) => store,
            None => anyq_queue_sqlite::Sqlite::open_background(
                path,
                shutdown.clone(),
                anyq_plugin_impl::initialize,
            )?,
        };
        let store = with_fixture_clock(store);
        let queue = anyq_queue_impl::compose(store.clone(), QueuePolicy::default());

        let work = anyq_queue_impl::work_services(store.clone());
        let capabilities = anyq_plugin_impl::WorkCapabilities {
            complete_once: work.complete_once,
            begin_completion: work.begin_completion,
            register: work.register,
            report: work.report,
            list: work.list,
            cancel: work.cancel,
        };

        #[cfg(feature = "integration-test-hooks")]
        let clock = crate::fixture_clock::FixtureClock::from_environment()
            .map(|clock| Box::new(clock) as Box<dyn anyq_plugin_runtime::Clock>);
        #[cfg(not(feature = "integration-test-hooks"))]
        let clock = None;

        anyq_plugin_impl::run(
            path,
            Box::new(store.clone()),
            queue.get_task,
            capabilities,
            clock,
            shutdown,
        )
    }
}

fn with_fixture_clock(store: anyq_queue_sqlite::Sqlite) -> anyq_queue_sqlite::Sqlite {
    #[cfg(feature = "integration-test-hooks")]
    if let Some(clock) = crate::fixture_clock::FixtureClock::from_environment() {
        return store.with_clock(std::rc::Rc::new(clock));
    }

    store
}

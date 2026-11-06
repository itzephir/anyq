//! Win32 details. Every raw handle has one OwnedHandle owner.
use std::{
    io,
    mem::{size_of, zeroed},
    os::windows::{
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::CommandExt,
    },
    process::{Child, Command},
    ptr::{null, null_mut},
    thread,
    time::{Duration, Instant},
};

use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
        },
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
            QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
        },
        Threading::{
            CREATE_NEW_PROCESS_GROUP, CREATE_SUSPENDED, OpenThread, ResumeThread,
            THREAD_SUSPEND_RESUME,
        },
    },
};

pub struct ChildTree {
    pub child: Child,
    job: OwnedHandle,
    done: bool,
}

impl ChildTree {
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        // SAFETY: null parameters request an unnamed, non-inheritable job. The returned
        // successful handle is transferred exactly once to OwnedHandle.
        let raw = unsafe { CreateJobObjectW(null(), null()) };

        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }

        let job = unsafe { OwnedHandle::from_raw_handle(raw) };
        // SAFETY: this Win32 POD is documented to accept zeroed unused fields.
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: job is live, info and its byte length match the requested info class.
        if unsafe {
            SetInformationJobObject(
                job.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }

        // No application code can run or spawn descendants before job assignment.
        let mut child = command
            .creation_flags(CREATE_SUSPENDED | CREATE_NEW_PROCESS_GROUP)
            .spawn()?;
        // SAFETY: both handles are owned and live. A failed assignment leaves the child
        // suspended; explicitly kill/reap it rather than executing outside our job.
        if unsafe { AssignProcessToJobObject(job.as_raw_handle(), child.as_raw_handle()) } == 0 {
            let error = io::Error::last_os_error();
            let _ = child.kill();
            let _ = child.wait();

            return Err(error);
        }

        let tree = Self {
            child,
            job,
            done: false,
        };

        // On a resume failure, Drop terminates/reaps the assigned child and closes the job.
        resume_initial_thread(tree.child.id())?;

        Ok(tree)
    }

    fn terminate(&self) -> io::Result<()> {
        // SAFETY: this is our owned job; it includes only this command and descendants.
        if unsafe { TerminateJobObject(self.job.as_raw_handle(), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    fn wait_empty(&self) -> io::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);

        loop {
            // SAFETY: output storage and its size match the accounting information class.
            let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };

            if unsafe {
                QueryInformationJobObject(
                    self.job.as_raw_handle(),
                    JobObjectBasicAccountingInformation,
                    (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    null_mut(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }

            if info.ActiveProcesses == 0 {
                return Ok(());
            }

            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "command descendants did not exit",
                ));
            }

            thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn poll(&mut self) -> io::Result<Option<i32>> {
        match self.child.try_wait()? {
            Some(status) => {
                self.terminate()?;
                self.wait_empty()?;
                self.done = true;

                Ok(Some(status.code().unwrap_or(1)))
            }

            None => Ok(None),
        }
    }

    pub fn stop(&mut self) -> io::Result<()> {
        if !self.done {
            self.terminate()?;
            self.child.wait()?;
            self.wait_empty()?;
            self.done = true;
        }

        Ok(())
    }
}

impl Drop for ChildTree {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn resume_initial_thread(pid: u32) -> io::Result<()> {
    // std::process exposes the process handle, but not the initial thread handle.
    // The child is still suspended, so its only application thread can be found safely.
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };

    if raw == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }

    let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut entry: THREADENTRY32 = unsafe { zeroed() };
    entry.dwSize = size_of::<THREADENTRY32>() as u32;
    let mut found = unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) };

    while found != 0 {
        if entry.th32OwnerProcessID == pid {
            let raw = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };

            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }

            let handle = unsafe { OwnedHandle::from_raw_handle(raw) };

            if unsafe { ResumeThread(handle.as_raw_handle()) } == u32::MAX {
                return Err(io::Error::last_os_error());
            }

            return Ok(());
        }

        found = unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) };
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "suspended command thread not found",
    ))
}

# Design

## Context

CI is already green on macOS/Linux. Only the worker process adapter imports std::os::unix/POSIX signals; CLI tests/demo use sh. Scope is in proposal; queue domain/transactions stay unchanged.

## Goals / Non-Goals

**Goals:** native Windows MSVC build/test without bash/WSL, identical queue scenarios, and verified process-tree termination on a Windows runner.

**Non-Goals:** Windows Service, distributed database, installer/release binaries, or background processes outside the managed group.

## Decisions

- Split the process adapter into shared stdin/payload/heartbeat bridge and cfg(unix)/cfg(windows) details. Retain consumer-owned ports/DI; SDKs stay out of inner crates.
- On Windows use Job Object with KILL_ON_JOB_CLOSE. Create with CREATE_SUSPENDED, assign to the job, then resume the initial thread through ToolHelp/OpenThread/ResumeThread so no descendant escapes the job. Any setup failure kills/waits for the child; OwnedHandle closes Win32 handles. Ordinary child.kill or taskkill after launch does not provide this boundary.
- Windows console cancellation uses ctrlc, local AtomicBool, and the existing Wait port. Unix retains signal-hook. windows-sys/ctrlc are Windows-only; libc/signal-hook Unix-only. Retain Rust 1.85 minimum.
- Native process fixtures run the integration-test executable as a child in a special mode. Echo, exit code, heartbeat, and cleanup checks are shell-independent. A separate Windows test creates a dedicated worker console and sends Ctrl+C from an attached helper without touching other tests' consoles.
- Unix cleanup reads process state through ps: zombies have exited and do not count as running. An absent PID also counts as exited; ps launch/execution errors are not cleanup success. The regression test retains an exited child without wait() until assertion and then explicitly reaps it.
- Portable Python 3 demo; developers already need Python for architecture checks, runtime does not. CI uses python/setup-python with Ubuntu/macOS/Windows, fail-fast false, and job timeouts.
- Windows defaults to LOCALAPPDATA/anyq/queue.db; Unix retains XDG/HOME. Architecture checks normalize paths so Windows separators do not affect main.rs comparisons.

## Risks / Trade-offs

- [Win32 behavior checked only remotely] → native integration tests, tree termination, and console cancellation; do not claim verified support before green Windows CI.
- [Job assignment prohibited by an external sandbox/job policy] → refuse launch and clean up; never execute outside the job.
- [Forced Windows termination] → OS closes the job handle and terminates managed descendants; leases still recover by expiry. Unix retains the original SIGKILL limitation.

//! Worker scenario orchestrated through narrowly typed dependencies.
mod process;
pub use process::Process;
mod runner;
pub use runner::Runner;
mod wait;
pub use wait::Wait;
mod execution_ports;
pub use execution_ports::ExecutionPorts;
mod execution;
pub use execution::Execution;
#[cfg(test)]
mod tests;

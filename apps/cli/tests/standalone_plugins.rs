#[path = "support/process.rs"]
pub mod process;

#[path = "support/process_spawn.rs"]
pub mod process_spawn;

use anyq_plugin_wire::{HostFrame, MAX_FRAME_BYTES, PROTOCOL_VERSION, PluginFrame};
use process::TestProcess;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn independent_plugin_binaries_handshake_and_stop_through_the_public_protocol() {
    for (name, executable) in [
        ("shell", env!("CARGO_BIN_EXE_anyq-shell")),
        ("timeout", env!("CARGO_BIN_EXE_anyq-timeout")),
    ] {
        let mut command = Command::new(executable);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut process = TestProcess::spawn(&mut command);
        let mut input = process.child.as_mut().unwrap().stdin.take().unwrap();
        let hello = HostFrame::Hello {
            version: PROTOCOL_VERSION,
            plugin: name.into(),
            max_frame_bytes: MAX_FRAME_BYTES,
            state: anyq_queue_api::Metadata::default().into(),
        };

        for frame in [hello, HostFrame::Stop] {
            let mut bytes = serde_json::to_vec(&frame).unwrap();
            bytes.push(b'\n');
            input.write_all(&bytes).unwrap();
        }
        drop(input);
        let output = process.output();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let first = output.stdout.split(|byte| *byte == b'\n').next().unwrap();
        let frame: PluginFrame = serde_json::from_slice(first).unwrap();
        assert!(
            matches!(frame, PluginFrame::Hello { version, subscriptions, .. } if version == PROTOCOL_VERSION && subscriptions.contains(&"task.running".into()))
        );
    }
}

#[test]
fn plugins_reject_command_arguments_instead_of_treating_them_as_parameters() {
    for executable in [
        env!("CARGO_BIN_EXE_anyq-shell"),
        env!("CARGO_BIN_EXE_anyq-timeout"),
    ] {
        let mut command = Command::new(executable);
        command
            .arg("unrecognized-parameter")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = TestProcess::spawn(&mut command).output();

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("takes no arguments")
        );
    }
}

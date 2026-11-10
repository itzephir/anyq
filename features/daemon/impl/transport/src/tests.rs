use crate::FrameReader;
use crate::{Endpoint, send_frame};
use std::{
    io::{Read, Write},
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

#[test]
fn active_slow_reader_can_transfer_longer_than_the_inactivity_timeout() {
    let directory = tempfile::tempdir().unwrap();
    let endpoint = Endpoint::resolve(&directory.path().join("queue.db")).unwrap();
    let listener = endpoint.listen().unwrap();
    let mut client = endpoint.connect().unwrap();
    let payload = "x".repeat(2 * 1024 * 1024);
    let expected_length = payload.len() + 6;
    let server = thread::spawn(move || {
        let stream = loop {
            match listener.accept() {
                Ok(stream) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(2))
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };

        let started = Instant::now();
        send_frame(
            &stream,
            &payload,
            &AtomicBool::new(false),
            Duration::from_secs(1),
        )
        .unwrap();
        assert!(started.elapsed() > Duration::from_secs(1));
    });
    let started = Instant::now();
    let mut bytes = Vec::new();
    let mut chunk = [0; 16 * 1024];
    let mut delay_bytes = 0;
    let mut pauses = 0;

    while bytes.len() < expected_length {
        match client.read(&mut chunk) {
            Ok(0) => panic!("active transfer was closed before completion"),
            Ok(count) => {
                bytes.extend_from_slice(&chunk[..count]);
                delay_bytes += count;
                while delay_bytes >= 64 * 1024 && pauses < 8 {
                    thread::sleep(Duration::from_millis(300));
                    delay_bytes -= 64 * 1024;
                    pauses += 1;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(2))
            }
            Err(error) => panic!("read failed: {error}"),
        }

        assert!(started.elapsed() < Duration::from_secs(15));
    }

    server.join().unwrap();
    assert!(started.elapsed() > Duration::from_secs(1));
    assert_eq!(
        u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize,
        expected_length - 4
    );
    assert_eq!(
        serde_json::from_slice::<String>(&bytes[4..]).unwrap().len(),
        2 * 1024 * 1024
    );
}

#[test]
fn large_query_responses_span_bounded_chunks() {
    let directory = tempfile::tempdir().unwrap();
    let endpoint = Endpoint::resolve(&directory.path().join("queue.db")).unwrap();
    let listener = endpoint.listen().unwrap();
    let client = endpoint.connect().unwrap();
    let length = anyq_daemon_api::MAX_FRAME_BYTES + 1;
    let server = thread::spawn(move || {
        let stream = loop {
            match listener.accept() {
                Ok(stream) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(2))
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        let payload = "x".repeat(length);
        send_frame(
            &stream,
            &payload,
            &AtomicBool::new(false),
            Duration::from_secs(5),
        )
        .unwrap();
    });

    let payload: String = FrameReader::for_responses()
        .receive(&client, &AtomicBool::new(false), Duration::from_secs(45))
        .unwrap();
    assert_eq!(payload.len(), length);
    server.join().unwrap();
}

#[test]
fn incoming_request_limit_counts_all_continuation_chunks() {
    let directory = tempfile::tempdir().unwrap();
    let endpoint = Endpoint::resolve(&directory.path().join("queue.db")).unwrap();
    let listener = endpoint.listen().unwrap();
    let mut client = endpoint.connect().unwrap();
    let server = thread::spawn(move || {
        let stream = loop {
            match listener.accept() {
                Ok(stream) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(2))
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        let error = FrameReader::with_limit(48 * 1024)
            .receive::<String>(&stream, &AtomicBool::new(false), Duration::from_secs(10))
            .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    });
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&((32 * 1024u32) | crate::MORE).to_be_bytes());
    bytes.extend_from_slice(&vec![b'x'; 32 * 1024]);
    bytes.extend_from_slice(&(32 * 1024u32).to_be_bytes());
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut offset = 0;

    while offset < bytes.len() {
        match client.write(&bytes[offset..]) {
            Ok(0) => panic!("fixture connection closed"),
            Ok(count) => offset += count,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => panic!("fixture write failed: {error}"),
        }
    }

    server.join().unwrap();
}

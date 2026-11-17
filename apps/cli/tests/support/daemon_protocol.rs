use anyq_daemon_api::{
    PROTOCOL_VERSION, PolicyDto, QueueRequest, QueueResponse, RequestFrame, ResponseFrame,
};
use anyq_daemon_transport::{Endpoint, FrameReader, Stream, send_frame};
use anyq_queue_api::QueuePolicy;
use std::{sync::atomic::AtomicBool, time::Duration};

pub fn request(stream: &Stream, id: u64, request: QueueRequest) -> QueueResponse {
    let cancelled = AtomicBool::new(false);
    send_frame(
        stream,
        &RequestFrame {
            version: PROTOCOL_VERSION,
            id,
            request,
        },
        &cancelled,
        Duration::from_secs(5),
    )
    .unwrap();
    let frame: ResponseFrame = FrameReader::for_responses()
        .receive(stream, &cancelled, Duration::from_secs(5))
        .unwrap();
    assert_eq!(frame.version, PROTOCOL_VERSION);
    assert_eq!(frame.id, id);

    frame.response
}

pub fn open(endpoint: &Endpoint) -> Stream {
    let stream = endpoint.connect().unwrap();
    let response = request(
        &stream,
        1,
        QueueRequest::Open {
            caller_directory: std::env::current_dir().unwrap().into(),
            database_id: endpoint.identity.clone(),
            policy: PolicyDto::from(QueuePolicy::default()),
            #[cfg(feature = "integration-test-hooks")]
            gate: anyq_daemon_api::GateDto {
                name: None,
                hit: 1,
                directory: None,
            },
        },
    );
    assert!(matches!(response, QueueResponse::Ready { .. }));

    stream
}

use anyq_plugin_file::install_missing_binary;
use std::{
    fs,
    sync::{Arc, Barrier, atomic::AtomicBool},
    thread,
};

#[test]
fn concurrent_first_installation_is_complete_and_preserves_an_existing_user_binary() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("packaged");
    let destination = temp.path().join("plugins/shell");
    let contents = vec![42; 256 * 1024];
    fs::write(&source, &contents).unwrap();
    let barrier = Arc::new(Barrier::new(8));

    let threads: Vec<_> = (0..8)
        .map(|_| {
            let (source, destination, barrier) =
                (source.clone(), destination.clone(), barrier.clone());
            thread::spawn(move || {
                barrier.wait();
                install_missing_binary(&source, &destination, &AtomicBool::new(false)).unwrap();
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }

    assert_eq!(fs::read(&destination).unwrap(), contents);

    fs::write(&source, b"new packaged version").unwrap();
    install_missing_binary(&source, &destination, &AtomicBool::new(false)).unwrap();

    assert_eq!(fs::read(&destination).unwrap(), contents);
}

#[test]
fn missing_source_and_cancellation_never_publish_a_partial_binary() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("missing");
    let destination = temp.path().join("plugins/timeout");

    assert!(install_missing_binary(&source, &destination, &AtomicBool::new(false)).is_err());
    assert!(!destination.exists());

    fs::write(&source, b"complete binary").unwrap();
    assert!(install_missing_binary(&source, &destination, &AtomicBool::new(true)).is_err());

    assert!(!destination.exists());
    assert_eq!(
        fs::read_dir(destination.parent().unwrap()).unwrap().count(),
        0
    );
}

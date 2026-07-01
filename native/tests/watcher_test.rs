use std::fs;
use std::io::Write;

#[test]
fn test_start_stop_watcher() {
    let handle = cod_native::start_watcher(vec![]);
    assert!(handle >= 0, "start_watcher should return valid handle");
    cod_native::stop_watcher(handle);
}

#[test]
fn test_poll_no_changes() {
    let handle = cod_native::start_watcher(vec![]);
    let changes = cod_native::poll_changes(handle);
    assert!(changes.is_empty(), "should have no changes initially");
    cod_native::stop_watcher(handle);
}

#[test]
fn test_watch_temp_file() {
    let dir = std::env::temp_dir().join("cod-watcher-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");

    let handle = cod_native::start_watcher(vec![dir.to_string_lossy().to_string()]);
    assert!(handle >= 0, "start_watcher should succeed");

    // create a file
    let file_path = dir.join("test.txt");
    let mut f = fs::File::create(&file_path).expect("create file");
    writeln!(f, "hello").expect("write file");

    // give watcher time to detect
    std::thread::sleep(std::time::Duration::from_millis(500));

    let changes = cod_native::poll_changes(handle);
    assert!(!changes.is_empty(), "should detect file creation");

    let paths: Vec<&str> = changes.iter().map(|c| c.path.as_str()).collect();
    assert!(
        paths.iter().any(|p| p.contains("test.txt")),
        "should contain test.txt: {:?}",
        paths
    );

    cod_native::stop_watcher(handle);
    let _ = fs::remove_dir_all(&dir);
}

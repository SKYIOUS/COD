use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::sync::mpsc;

#[derive(Serialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub kind: String,
}

struct WatcherState {
    rx: Option<mpsc::Receiver<notify::Result<Event>>>,
    _watcher: Option<RecommendedWatcher>,
}

#[napi]
pub fn start_watcher(paths: Vec<String>) -> i64 {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
        Ok(w) => w,
        Err(_) => return -1,
    };
    for path in &paths {
        let _ = watcher.watch(path.as_ref(), RecursiveMode::Recursive);
    }
    let state = WatcherState {
        rx: Some(rx),
        _watcher: Some(watcher),
    };
    let ptr = Box::into_raw(Box::new(state)) as i64;
    ptr
}

#[napi]
pub fn poll_changes(handle: i64) -> Vec<FileChange> {
    let ptr = handle as *mut WatcherState;
    if ptr.is_null() {
        return Vec::new();
    }
    let state = unsafe { &mut *ptr };
    let rx = match &state.rx {
        Some(rx) => rx,
        None => return Vec::new(),
    };
    let mut changes = Vec::new();
    while let Ok(event) = rx.try_recv() {
        match event {
            Ok(event) => {
                let kind = match event.kind {
                    EventKind::Create(_) => "create".to_string(),
                    EventKind::Modify(_) => "modify".to_string(),
                    EventKind::Remove(_) => "delete".to_string(),
                    _ => continue,
                };
                for path in event.paths {
                    changes.push(FileChange {
                        path: path.to_string_lossy().to_string(),
                        kind: kind.clone(),
                    });
                }
            }
            Err(_) => break,
        }
    }
    changes
}

#[napi]
pub fn stop_watcher(handle: i64) {
    if handle == 0 {
        return;
    }
    let ptr = handle as *mut WatcherState;
    if !ptr.is_null() {
        let _ = unsafe { Box::from_raw(ptr) };
    }
}

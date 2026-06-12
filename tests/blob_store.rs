use flightrec::blobstore::BlobStore;
use tempfile::TempDir;

fn temp_store() -> (TempDir, BlobStore) {
    let dir = TempDir::new().unwrap();
    let store = BlobStore::new(dir.path().join("objects"));
    (dir, store)
}

#[test]
fn write_read_roundtrip() {
    let (_dir, store) = temp_store();
    let content = b"hello, flightrec!";
    let hash = "aabbcc112233445566778899aabbccdd00112233445566778899aabbccdd0011";
    store.write(hash, content).unwrap();
    let got = store.read(hash).unwrap();
    assert_eq!(got, content);
}

#[test]
fn duplicate_write_is_noop() {
    let (_dir, store) = temp_store();
    let content = b"duplicate content";
    let hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    store.write(hash, content).unwrap();
    let blob_path = store.blob_path(hash);
    let mtime1 = std::fs::metadata(&blob_path).unwrap().modified().unwrap();

    // Sleep briefly to ensure mtime would differ if re-written
    std::thread::sleep(std::time::Duration::from_millis(10));
    store.write(hash, content).unwrap();
    let mtime2 = std::fs::metadata(&blob_path).unwrap().modified().unwrap();

    assert_eq!(mtime1, mtime2, "second write must not touch existing blob");
}

#[test]
fn read_missing_hash_errors() {
    let (_dir, store) = temp_store();
    let hash = "0000000000000000000000000000000000000000000000000000000000000000";
    let result = store.read(hash);
    assert!(result.is_err(), "reading absent hash should return Err");
}

#[test]
fn fanout_path_layout() {
    let root = std::path::PathBuf::from("/tmp/test_objects");
    let store = BlobStore::new(root.clone());
    let hash = "abc123def456000000000000000000000000000000000000000000000000cafe";
    let path = store.blob_path(hash);
    assert_eq!(
        path,
        root.join("ab")
            .join("c123def456000000000000000000000000000000000000000000000000cafe.blob")
    );
}

#[test]
fn read_string_returns_utf8() {
    let (_dir, store) = temp_store();
    let content = b"line one\nline two\n";
    let hash = "cafebabe00000000000000000000000000000000000000000000000000000001";
    store.write(hash, content).unwrap();
    let s = store.read_string(hash).unwrap();
    assert_eq!(s, "line one\nline two\n");
}

#[test]
fn has_returns_true_only_after_write() {
    let (_dir, store) = temp_store();
    let hash = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    assert!(!store.has(hash));
    store.write(hash, b"data").unwrap();
    assert!(store.has(hash));
}

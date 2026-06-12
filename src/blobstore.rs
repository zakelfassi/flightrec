use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Maximum file size for blob write-through (10 MiB).
pub const BLOB_SIZE_CAP: u64 = 10 * 1024 * 1024;

/// Content-addressable blob store with git-style fan-out layout.
///
/// Layout: `<root>/<first2-hex>/<rest64>.blob`
///
/// Writes are atomic (tmp file + `fs::rename`). Duplicate writes are no-ops.
pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    /// Create a new `BlobStore` rooted at `root`.
    ///
    /// The root directory is created lazily on first write.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        BlobStore { root: root.into() }
    }

    /// Compute the on-disk path for a given hex hash.
    pub fn blob_path(&self, hash: &str) -> PathBuf {
        let (prefix, rest) = hash.split_at(2);
        self.root.join(prefix).join(format!("{rest}.blob"))
    }

    /// Return `true` if a blob for `hash` already exists on disk.
    pub fn has(&self, hash: &str) -> bool {
        self.blob_path(hash).exists()
    }

    /// Write `content` as a blob for `hash`.
    ///
    /// If the blob already exists this is a no-op (deduplication).
    /// The write is atomic: content is written to a temp file in the same
    /// directory then `rename`d into place.
    pub fn write(&self, hash: &str, content: &[u8]) -> Result<()> {
        let dest = self.blob_path(hash);
        if dest.exists() {
            return Ok(());
        }

        let dir = dest.parent().expect("blob path always has a parent dir");
        std::fs::create_dir_all(dir)
            .with_context(|| format!("create blob dir {}", dir.display()))?;

        // Write to a temp file in the same directory so rename is same-fs.
        let tmp_path = dir.join(format!(".tmp-{hash}"));
        std::fs::write(&tmp_path, content)
            .with_context(|| format!("write tmp blob {}", tmp_path.display()))?;

        std::fs::rename(&tmp_path, &dest)
            .with_context(|| format!("rename blob into place {}", dest.display()))?;

        Ok(())
    }

    /// Read the raw bytes of a blob by `hash`.
    pub fn read(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self.blob_path(hash);
        std::fs::read(&path).with_context(|| format!("read blob {} ({})", hash, path.display()))
    }

    /// Read a blob as a UTF-8 string.
    pub fn read_string(&self, hash: &str) -> Result<String> {
        let bytes = self.read(hash)?;
        String::from_utf8(bytes).with_context(|| format!("blob {hash} is not valid UTF-8"))
    }
}

/// Open (or create) the blob store rooted at `objects_root`.
pub fn open(objects_root: &Path) -> BlobStore {
    BlobStore::new(objects_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_store() -> (TempDir, BlobStore) {
        let dir = TempDir::new().unwrap();
        let store = BlobStore::new(dir.path().join("objects"));
        (dir, store)
    }

    #[test]
    fn fanout_path_splits_at_two() {
        let store = BlobStore::new("/tmp/obj");
        let hash = "aabbcc0011223344556677889900aabbcc0011223344556677889900aabbcc00";
        let p = store.blob_path(hash);
        assert_eq!(
            p,
            PathBuf::from(
                "/tmp/obj/aa/bbcc0011223344556677889900aabbcc0011223344556677889900aabbcc00.blob"
            )
        );
    }

    #[test]
    fn write_creates_file_in_fanout_dir() {
        let (_dir, store) = temp_store();
        let hash = "ffee00112233445566778899aabbccdd00112233445566778899aabbccdd0011";
        store.write(hash, b"content").unwrap();
        assert!(store.blob_path(hash).exists());
        // Parent dir should be <root>/ff/
        assert!(store.blob_path(hash).parent().unwrap().exists());
    }

    #[test]
    fn write_then_read_roundtrip_unit() {
        let (_dir, store) = temp_store();
        let data = b"unit roundtrip";
        let hash = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcde0";
        store.write(hash, data).unwrap();
        assert_eq!(store.read(hash).unwrap(), data);
    }
}

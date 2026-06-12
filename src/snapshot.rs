use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

use crate::blobstore::{BlobStore, BLOB_SIZE_CAP};
use crate::utils::{expand_tilde, now_iso};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub blob_hash: String,
    pub is_text: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SnapshotManifest {
    pub snapshot_id: String,
    pub created_at: String,
    pub roots: Vec<String>,
    pub files: Vec<FileEntry>,
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    Ok(builder.build()?)
}

fn is_text_file(path: &Path) -> bool {
    if let Ok(mut f) = std::fs::File::open(path) {
        let mut buf = [0u8; 8192];
        if let Ok(n) = f.read(&mut buf) {
            return !buf[..n].contains(&0u8);
        }
    }
    false
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Hash a file and, when a `BlobStore` is provided and the file is ≤ 10 MiB,
/// write the content into the store. Returns the hex SHA-256 hash.
fn hash_and_store(path: &Path, size: u64, store: Option<&BlobStore>) -> Result<String> {
    if size <= BLOB_SIZE_CAP {
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());
        if let Some(bs) = store {
            if let Err(e) = bs.write(&hash, &content) {
                eprintln!("flightrec: warning: blob write failed for {hash}: {e}");
            }
        }
        Ok(hash)
    } else {
        hash_file(path)
    }
}

fn snapshot_id_now() -> String {
    let now = chrono::Utc::now();
    let subsec = now.timestamp_subsec_millis();
    format!("{}-{:03}", now.format("%Y%m%dT%H%M%S"), subsec)
}

pub fn take_snapshot(
    roots: &[String],
    include: &[String],
    exclude: &[String],
    blob_store: Option<&BlobStore>,
) -> Result<SnapshotManifest> {
    let include_set = build_globset(include)?;
    let exclude_set = build_globset(exclude)?;
    let mut files = Vec::new();

    for root_str in roots {
        let root_path = expand_tilde(root_str);
        if !root_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&root_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let rel = path.strip_prefix(&root_path).unwrap_or(path);
            let rel_str = rel.to_string_lossy();

            if exclude_set.is_match(rel_str.as_ref()) {
                continue;
            }
            if !include.is_empty() && !include_set.is_match(rel_str.as_ref()) {
                continue;
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = metadata.len();
            let blob_hash = match hash_and_store(path, size, blob_store) {
                Ok(h) => h,
                Err(_) => continue,
            };
            let is_text = is_text_file(path);

            files.push(FileEntry {
                path: path.to_string_lossy().to_string(),
                size,
                blob_hash,
                is_text,
            });
        }
    }

    Ok(SnapshotManifest {
        snapshot_id: snapshot_id_now(),
        created_at: now_iso(),
        roots: roots.to_vec(),
        files,
    })
}

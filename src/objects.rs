use crate::Repository;
use anyhow::{bail, Context, Result};
use flate2::read::ZlibDecoder;
use itertools::Itertools;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use strum::EnumString;

/// ObjectType represents the type of object in a git repository
#[derive(Debug, PartialEq, EnumString)]
enum ObjectType {
    #[strum(ascii_case_insensitive)]
    Blob,
    #[strum(ascii_case_insensitive)]
    Tree,
    #[strum(ascii_case_insensitive)]
    Commit,
    #[strum(ascii_case_insensitive)]
    Tag,
}

#[derive(Debug)]
pub struct Object {
    pub object_type: ObjectType,
    pub size: usize,
    pub data: Vec<u8>,
}

impl Object {
    pub fn new(object_type: ObjectType, size: usize, data: Vec<u8>) -> Self {
        Object {
            object_type,
            size,
            data,
        }
    }
}

/// A newtype for a Git hash which guarantees that the hash is exactly 20 bytes long.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHash([u8; 20]);

impl GitHash {
    /// Creates a new GitHash from a 20-byte array.
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying bytes as a string slice.
    ///
    /// # Panics
    ///
    /// Panics if the inner bytes are not valid UTF‑8.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("GitHash bytes are not valid UTF-8")
    }

    /// Splits the string representation into the two components used by Git's
    /// object storage: the first two characters form the directory name and
    /// the remaining characters form the file name.
    pub fn as_path_parts(&self) -> (String, String) {
        let s = self.as_str();
        let (dir, file) = s.split_at(2);
        (dir.to_string(), file.to_string())
    }
}

impl From<[u8; 20]> for GitHash {
    fn from(bytes: [u8; 20]) -> Self {
        GitHash::new(bytes)
    }
}

/// Reads a Git object from the repository given its hash.
///
/// The object is stored under `.git/objects/<dir>/<file>` where the directory
/// is the first two characters of the hash (as a string) and the file is the rest.
/// The object file is stored compressed (zlib); after decompression, its header
/// is expected to have the form "type size\0". This function parses the header,
/// validates the size, and returns an `Object`.
pub fn read_object(repo: &Repository, hash: &GitHash) -> Result<Object> {
    let (dir, file) = hash.as_path_parts();
    let object_path: PathBuf = repo.gitdir.join("objects").join(dir).join(file);
    if !object_path.exists() {
        bail!("Object not found at {}", object_path.display());
    }

    let file = File::open(&object_path)
        .with_context(|| format!("Failed to open object file: {}", object_path.display()))?;
    let mut decoder = ZlibDecoder::new(file);
    let mut buffer = Vec::new();
    decoder
        .read_to_end(&mut buffer)
        .context("Failed to decompress object data")?;

    let (header, data) = buffer
        .split(|&b| b == 0)
        .collect_tuple()
        .map(|(header, data)| (String::from_utf8_lossy(&header).into_owned(), data.to_vec()))
        .ok_or_else(|| anyhow::anyhow!("Invalid object header: missing null terminator"))?;

    let (object_type, size) = header
        .split_once(' ')
        .ok_or_else(|| anyhow::anyhow!("Invalid object header: missing type or size"))
        .map(|(type_str, size_str)| -> Result<(ObjectType, usize)> {
            let object_type = ObjectType::from_str(type_str).context("Invalid object type")?;
            let size = size_str.parse::<usize>().context("Invalid size")?;
            Ok((object_type, size))
        })??;

    if data.len() != size {
        bail!(
            "Object size mismatch: header specifies {} bytes but found {} bytes",
            size,
            data.len()
        );
    }

    Ok(Object::new(object_type, size, data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempfile::TempDir;

    const TEST_HASH: &[u8; 20] = b"1234567890abcdef1234";
    const TEST_DATA: &[u8] = b"test data";

    #[test]
    fn test_git_hash_new() {
        let hash = GitHash::new(*TEST_HASH);
        assert!(hash.0 == *TEST_HASH);
    }

    #[test]
    fn test_git_hash_as_str() {
        let hash = GitHash::new(*TEST_HASH);
        let hash_str = std::str::from_utf8(&hash.0).unwrap();
        assert_eq!(hash.as_str(), hash_str);
    }

    #[test]
    fn test_git_hash_as_path_parts() {
        let hash = GitHash::new(*TEST_HASH);
        let (dir, file) = hash.as_path_parts();
        assert_eq!(dir, "12");
        assert_eq!(file, "34567890abcdef1234");
    }

    // Helper struct to ensure TempDir lives as long as Repository
    struct TestRepo(Repository);

    impl TestRepo {
        pub fn new(hash: &[u8; 20], test_data: &[u8], tempdir: &TempDir) -> Self {
            let repo = Repository::new(tempdir.path()).unwrap();

            // Create the repository structure first
            repo.create().unwrap();

            let githash = GitHash::new(*hash);
            let (dir, file) = githash.as_path_parts();
            let object_path = repo.gitdir.join("objects").join(dir);

            std::fs::create_dir_all(&object_path).unwrap();
            let object_file = object_path.join(file);

            // Create proper Git object with header and zlib compression
            let header = format!("blob {}\0", test_data.len());
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(header.as_bytes()).unwrap();
            encoder.write_all(test_data).unwrap();
            let compressed_data = encoder.finish().unwrap();

            // Write the file and check it was created
            std::fs::write(&object_file, compressed_data).unwrap();
            TestRepo(repo)
        }
    }

    #[test]
    fn test_read_object() {
        let tempdir = TempDir::new().unwrap();
        let test_repo = TestRepo::new(TEST_HASH, TEST_DATA, &tempdir);
        let githash = GitHash::new(*TEST_HASH);
        let result = read_object(&test_repo.0, &githash);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}

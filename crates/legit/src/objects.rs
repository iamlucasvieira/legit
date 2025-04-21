use crate::Repository;
use anyhow::{bail, Context, Result};
use digest::generic_array::typenum::U20;
use digest::generic_array::GenericArray;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use itertools::Itertools;
use sha1::{Digest, Sha1};
use std::fmt::{Display, Write};
use std::fs::File;
use std::io::{Read, Write as _};
use std::path::PathBuf;
use std::str::FromStr;
use strum::EnumString;

/// ObjectType represents the type of object in a git repository
#[derive(Debug, PartialEq, EnumString, Clone, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

#[derive(Debug)]
pub struct Object {
    pub object_type: ObjectType,
    pub data: Vec<u8>,
    pub hash: ObjectHash,
}

impl Object {
    /// Create a new Git object
    pub fn new(object_type: ObjectType, data: Vec<u8>) -> Result<Self> {
        let object_data = format!(
            "{} {}\0{}",
            object_type,
            data.len(),
            String::from_utf8_lossy(&data)
        );
        let hash = ObjectHash::try_from(object_data.as_str()).context("Failed to hash object")?;
        Ok(Object {
            object_type,
            data,
            hash,
        })
    }

    /// Return the file path of the object in the repository
    pub fn file_path(&self, repo: &Repository) -> PathBuf {
        let (dir, file) = self.hash.as_path_parts();
        repo.gitdir().join("objects").join(dir).join(file)
    }

    /// Return the header of the object
    pub fn header(&self) -> String {
        format!("{} {}\0", self.object_type, self.data.len())
    }
}

/// A newtype for a Git hash which guarantees that the hash is exactly 20 bytes long.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectHash(GenericArray<u8, U20>);

impl ObjectHash {
    /// Convert a hexadecimal string representation of a hash into an ObjectHash.
    pub fn from_hex(hex: &str) -> Result<Self> {
        if hex.len() != 40 {
            bail!(
                "Invalid hash length: expected 40 characters, got {}",
                hex.len()
            );
        }
        let bytes = hex
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                let byte_str = std::str::from_utf8(chunk).unwrap();
                u8::from_str_radix(byte_str, 16).unwrap()
            })
            .collect::<Vec<u8>>();
        if bytes.len() != 20 {
            bail!(
                "Invalid hash length: expected 20 bytes, got {}",
                bytes.len()
            );
        }
        let mut array = GenericArray::<u8, U20>::default();
        array.copy_from_slice(&bytes);
        Ok(ObjectHash(array))
    }

    /// Convert the hash to a hexadecimal string representation.
    pub fn to_hex(&self) -> String {
        self.0.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02X}");
            output
        })
    }

    /// Splits the string representation into the two components used by Git's
    /// object storage: the first two characters form the directory name and
    /// the remaining characters form the file name.
    pub fn as_path_parts(&self) -> (String, String) {
        let hex = self.to_hex();
        let (dir, file) = hex.split_at(2);
        (dir.to_string(), file.to_string())
    }
}

impl TryFrom<&[u8]> for ObjectHash {
    type Error = anyhow::Error;

    /// Create a ObjectHash. Uses SHA-1 to hash the input data.
    fn try_from(slice: &[u8]) -> Result<Self> {
        let mut hasher = Sha1::new();
        hasher.update(slice);
        let result = hasher.finalize();
        if result.len() != 20 {
            bail!("SHA-1 digest should be 20 bytes, got {}", result.len());
        }
        let mut bytes = GenericArray::<u8, U20>::default();
        bytes.copy_from_slice(&result);
        Ok(ObjectHash(bytes))
    }
}

impl TryFrom<&str> for ObjectHash {
    type Error = anyhow::Error;

    /// Create a ObjectHash from a string. Uses SHA-1 to hash the input data.
    fn try_from(s: &str) -> Result<Self> {
        ObjectHash::try_from(s.as_bytes())
    }
}

impl Display for ObjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Reads a Git object from the repository given its hash.
///
/// The object is stored under `.git/objects/<dir>/<file>` where the directory
/// is the first two characters of the hash (as a string) and the file is the rest.
/// The object file is stored compressed (zlib); after decompression, its header
/// is expected to have the form "type size\0". This function parses the header,
/// validates the size, and returns an `Object`.
pub fn read_object(repo: &Repository, hash: &ObjectHash) -> Result<Object> {
    let (dir, file) = hash.as_path_parts();
    let object_path: PathBuf = repo.gitdir().join("objects").join(dir).join(file);
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
        .map(|(header, data)| (String::from_utf8_lossy(header).into_owned(), data.to_vec()))
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

    Object::new(object_type, data)
}

/// Writes a Git object to the repository.
///
/// The object is stored under `.git/objects/<dir>/<file>` where the directory
/// is the first two characters of the hash (as a string) and the file is the rest.
/// The object file is stored compressed (zlib). The function first checks if
/// the object already exists at the specified path. If it does, an error
/// is returned. If not, it creates the necessary directories and writes
/// the object to the file.
pub fn write_object(obj: &Object, repo: &Repository) -> Result<ObjectHash> {
    let object_path = obj.file_path(repo);
    if object_path.exists() {
        bail!("Object already exists at {}", object_path.display());
    }

    std::fs::create_dir_all(object_path.parent().unwrap()).with_context(|| {
        format!(
            "Failed to create directory for object: {}",
            object_path.display()
        )
    })?;

    let header = obj.header();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(header.as_bytes())?;
    encoder.write_all(&obj.data)?;
    let compressed_data = encoder.finish()?;

    std::fs::write(&object_path, compressed_data)
        .with_context(|| format!("Failed to write object file: {}", object_path.display()))?;

    Ok(obj.hash.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use tempfile::TempDir;

    #[test]
    fn test_git_hash_from_str() {
        let hash_str = "1234567890abcdef1234";
        let hash = ObjectHash::try_from(hash_str).unwrap();
        assert_eq!(hash.0.len(), 20);
    }

    #[test]
    fn test_git_hash_as_path_parts() {
        let hash_str = "1234567890abcdef1234";
        let hash = ObjectHash::try_from(hash_str).unwrap();
        let (dir, file) = hash.as_path_parts();
        assert_eq!(dir.len(), 2);
        assert_eq!(file.len(), 38);
    }

    #[test]
    fn test_read_object() {
        let tempdir = TempDir::new().unwrap();
        let object = Object::new(ObjectType::Blob, b"test".to_vec()).unwrap();
        let repo = Repository::new(tempdir.path()).unwrap();
        write_object(&object, &repo).unwrap();
        let result = read_object(&repo, &object.hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_object_object_doesnt_exist() {
        let tempdir = TempDir::new().unwrap();
        let object_written = Object::new(ObjectType::Blob, b"test".to_vec()).unwrap();
        let object_not_written = Object::new(ObjectType::Blob, b"other data".to_vec()).unwrap();
        let repo = Repository::new(tempdir.path()).unwrap();
        write_object(&object_written, &repo).unwrap();
        let result = read_object(&repo, &object_not_written.hash);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Object not found at"));
    }

    #[test]
    fn test_read_object_not_encoded() {
        let tempdir = TempDir::new().unwrap();
        let object = Object::new(ObjectType::Blob, b"test".to_vec()).unwrap();
        let repo = Repository::new(tempdir.path()).unwrap();
        let object_path = object.file_path(&repo);
        std::fs::create_dir_all(object_path.parent().unwrap()).unwrap();
        std::fs::write(&object_path, b"not compressed data").unwrap();
        let result = read_object(&repo, &object.hash);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to decompress object data"));
    }

    struct ReadObjectTestCase {
        pub header: &'static str,
        pub data: &'static [u8],
        pub expected_error: Option<&'static str>,
    }

    #[test]
    fn test_read_object_invalid_header() {
        let test_cases = [
            ReadObjectTestCase {
                header: "blob 4",
                data: b"test",
                expected_error: Some("missing null terminator"),
            },
            ReadObjectTestCase {
                header: "blob\0",
                data: b"test",
                expected_error: Some("missing type or size"),
            },
            ReadObjectTestCase {
                header: "blob a\0",
                data: b"test",
                expected_error: Some("Invalid size"),
            },
            ReadObjectTestCase {
                header: "invalid 2\0",
                data: b"test",
                expected_error: Some("Invalid object type"),
            },
            ReadObjectTestCase {
                header: "blob 30\0",
                data: b"test",
                expected_error: Some("Object size mismatch"),
            },
            ReadObjectTestCase {
                header: "blob 4\0",
                data: b"test",
                expected_error: None,
            },
        ];

        let tempdir = TempDir::new().unwrap();
        let object = Object::new(ObjectType::Blob, b"test".to_vec()).unwrap();
        let repo = Repository::new(tempdir.path()).unwrap();
        let object_path = object.file_path(&repo);
        std::fs::create_dir_all(object_path.parent().unwrap()).unwrap();

        for tc in test_cases.iter() {
            let content = format!("{}{}", tc.header, String::from_utf8_lossy(tc.data));
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(content.as_bytes()).unwrap();
            let compressed_data = encoder.finish().unwrap();
            std::fs::write(&object_path, compressed_data).unwrap();

            let result = read_object(&repo, &object.hash);
            if let Some(expected_error) = tc.expected_error {
                assert!(result.unwrap_err().to_string().contains(expected_error));
            } else {
                assert!(result.is_ok());
            }
        }
    }

    #[test]
    fn test_write_object() {
        let tempdir = TempDir::new().unwrap();
        let object = Object::new(ObjectType::Blob, b"test".to_vec()).unwrap();
        let repo = Repository::new(tempdir.path()).unwrap();
        let result = write_object(&object, &repo);
        let object_path = object.file_path(&repo);
        assert!(result.is_ok());
        assert!(object_path.exists());
    }

    #[test]
    fn test_write_object_object_already_exist() {
        let tempdir = TempDir::new().unwrap();
        let object = Object::new(ObjectType::Blob, b"test".to_vec()).unwrap();
        let repo = Repository::new(tempdir.path()).unwrap();
        write_object(&object, &repo).unwrap();
        let result = write_object(&object, &repo);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Object already exists at"));
    }
}

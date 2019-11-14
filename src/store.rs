use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::hash::Sha256Hash;

#[derive(Debug)]
pub struct Store {
    /// Root directory of the store
    base_dir: PathBuf,
    /// Directory where the actual data is stored, in files named after the sha256 hash of their contents
    data_dir: PathBuf,
    /// Directory for temp files created while storing files in the data directory.
    staging_dir: PathBuf,
    /// Directory for keeping references to the repositories that make use of this store.
    ref_dir: PathBuf,
}

impl Store {
    pub fn open_or_create(base_dir: PathBuf) -> io::Result<Store> {
        let data_dir = base_dir.join("data");
        let staging_dir = base_dir.join("staging");
        let ref_dir = base_dir.join("ref");

        std::fs::create_dir_all(&base_dir)?;
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&staging_dir)?;
        std::fs::create_dir_all(&ref_dir)?;

        Ok(Store {
            base_dir,
            data_dir,
            staging_dir,
            ref_dir,
        })
    }

    pub fn new_staging_file(&self) -> io::Result<StagingFile> {
        let (path, file) = new_temp_file(&self.staging_dir, "smudge", "")?;
        Ok(StagingFile::new(path, file))
    }

    pub fn make_permanent(&self, staging_file: StagingFile) -> io::Result<StoreFileRef> {
        drop(staging_file.file); // close the file
        let hash: Sha256Hash = staging_file.hasher.into();
        let final_path = self.data_dir.join(format!("{}", hash));

        // If the file already exists, we can still safely overwrite it because
        // if they have the same name, they will have the same contents.
        std::fs::rename(staging_file.filename, &final_path)?;

        let store_file = StoreFileRef { hash };

        Ok(store_file)
    }

    /// Open a file in the store's data directory based on a reference.
    pub fn open_ref(&self, store_ref: &StoreFileRef) -> io::Result<File> {
        let path = self.data_dir.join(format!("{}", store_ref.hash));
        File::open(path)
    }
}

/// A reference to a data file stored in the `Store`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct StoreFileRef {
    hash: Sha256Hash,
}

impl StoreFileRef {
    pub fn from_hash(hash: Sha256Hash) -> StoreFileRef {
        Self { hash }
    }

    pub fn hash(&self) -> &Sha256Hash {
        &self.hash
    }

    /// Convert this reference to its string representation in this format:
    ///
    /// ```text
    /// git-assets <format-version>
    /// <file-sha256-hash>
    /// ```
    ///
    /// where `<format-version>` is currently `v1` and will be increased when
    /// the reference format changes, and <file-sha256-hash> is the sha 256
    /// hash of the file contents that are pointed to by this reference.
    pub fn to_string(&self) -> String {
        format!("git-assets v1\n{}", self.hash)
    }

    pub fn parse_from_stream<R: Read>(reader: &mut R) -> io::Result<StoreFileRef> {
        // The current format takes exactly 78 bytes:
        // - 10 bytes for the magic string "git-assets"
        // - 1 byte for a space
        // - 2 bytes for "v1"
        // - 1 byte for the newline
        // - 64 bytes for the hex encoded sha256
        // First read magic to ensure that we don't accidentally try to parse something else
        let mut buf = [0; 78];
        reader.read_exact(&mut buf)?;
        if &buf[0..14] != b"git-assets v1\n" {
            return Err(io::ErrorKind::InvalidData.into());
        }

        let hash = Sha256Hash::from_hex(&buf[14..]).ok_or(io::ErrorKind::InvalidData)?;

        Ok(Self { hash })
    }
}

pub struct StagingFile {
    filename: PathBuf,
    file: File,
    hasher: Sha256,
}

impl StagingFile {
    fn new(filename: PathBuf, file: File) -> StagingFile {
        StagingFile {
            filename,
            file,
            hasher: Sha256::new(),
        }
    }
}

impl Write for StagingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n_written = self.file.write(buf)?;
        // Only hash the parts that we managed to write
        self.hasher.input(&buf[0..n_written]);

        Ok(n_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

fn new_temp_file(dir: &Path, base_name: &str, suffix: &str) -> io::Result<(PathBuf, File)> {
    let mut counter = 0;
    loop {
        counter += 1;
        let filename = dir.join(format!("{}.{}.{}", base_name, counter, suffix));
        let file_result = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&filename);

        match file_result {
            Err(ioerr) => match ioerr.kind() {
                // This is expected while searching for a free name
                io::ErrorKind::AlreadyExists => continue,
                // Other errors are not expected an actual errors
                _ => return Err(ioerr),
            },
            Ok(file) => return Ok((filename, file)),
        }
    }
}


#[cfg(test)]
mod test {
    use super::StoreFileRef;
    use crate::hash::Sha256Hash;

    #[test]
    fn test_store_file_ref_roundtrip() {
        let r = StoreFileRef::from_hash(Sha256Hash::from_hex(b"2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae").unwrap());

        let serialized = r.to_string();
        assert_eq!(serialized, "git-assets v1\n2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae");

        let r2 = StoreFileRef::parse_from_stream(&mut std::io::Cursor::new(serialized)).unwrap();
        assert_eq!(r2, r);
    }
}
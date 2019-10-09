//! # test_dir
//!
//! `TestDir` is a temporary directory builder. The target is to define a file structure for test purpose.
//! It is not recommended to use in non-test environment.
//!
//! ```
//! use std::path::PathBuf;
//! use test_dir::{TestDir,FileType,DirBuilder};
//!
//! let temp = TestDir::temp()
//!     .create("test/dir", FileType::Dir)
//!     .create("test/file", FileType::EmptyFile)
//!     .create("test/random_file", FileType::RandomFile(100))
//!     .create("otherdir/zero_file", FileType::ZeroFile(100));
//!
//! let path: PathBuf = temp.path("test/random_file");
//! assert!(path.exists());
//! ```

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs;
use std::io::prelude::*;
use std::io::BufWriter;
use std::iter;
use std::path::{Path, PathBuf};

#[derive(PartialEq, Debug)]
pub enum FileType {
    /// Create empty file
    EmptyFile,
    /// Create a file with random content of the given size
    RandomFile(usize),
    /// Create a file with a given len of "0"s
    ZeroFile(usize),
    //ContentFile(&dyn std::io::Read),
    /// Create a directory
    Dir,
}

pub struct TempDir {
    path: PathBuf,
    delete: PathBuf,
}

impl TempDir {
    /// Try to create a temporary directory inside system tmp directory.
    pub fn temp() -> std::io::Result<Self> {
        let mut temp = std::env::temp_dir().to_path_buf();
        temp.push(TempDir::random_name());
        TempDir::create(temp.as_path())
    }

    /// Try to create a temporary directory inside the current directory.
    pub fn current_rnd() -> std::io::Result<Self> {
        let mut temp = std::env::current_dir()?.to_path_buf();
        temp.push(TempDir::random_name());
        TempDir::create(temp.as_path())
    }

    /// Try to create a temporary directory with a given path inside the current directory.
    pub fn current(path: &Path) -> std::io::Result<Self> {
        let mut temp = std::env::current_dir()?.to_path_buf();
        temp.push(path);
        TempDir::create(temp.as_path())
    }

    /// Get the path of the temporary directory.
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    // Helper functions
    fn create(path: &Path) -> std::io::Result<Self> {
        let mut p = path;
        while let Some(ppath) = p.parent() {
            if ppath.exists() {
                break;
            }
            p = ppath;
        }
        fs::create_dir_all(&path)?;
        Ok(TempDir {
            path: path.to_path_buf(),
            delete: p.to_path_buf(),
        })
    }

    fn random_name() -> String {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(8)
            .collect()
    }
}

impl Drop for TempDir {
    /// Delete the created directory tree.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.delete.as_path());
    }
}

pub struct TestDir {
    // Directory lifetime
    _tempdir: Option<TempDir>,

    root: PathBuf,

    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
}

pub trait DirBuilder {
    /// Create a file or directory under the `path`
    fn create(self, path: &str, filetype: FileType) -> Self;
    /// Remove a file or directory under the `path`
    fn remove(self, path: &str) -> Self;
    /// Prefix `path` with the current context of the DirBuilder
    fn path(&self, path: &str) -> PathBuf;
    /// Return the root path to the temporary directory
    fn root(&self) -> &Path;
}

impl TestDir {
    /// Creates if possible a temporary directory
    pub fn temp() -> Self {
        if let Ok(tempdir) = TempDir::temp() {
            TestDir::new(tempdir)
        } else {
            panic!("Cannot create temp dir in system temp");
        }
    }

    /// Creates if possible a temporary directory with random name inside the current directory
    pub fn current_rnd() -> Self {
        if let Ok(tempdir) = TempDir::current_rnd() {
            TestDir::new(tempdir)
        } else {
            panic!("Cannot create temp dir in current directory")
        }
    }

    /// Creates if possible a temporary directory specified in `path` relative to the current directory
    pub fn current(path: &str) -> Self {
        let path = Path::new(path);
        if let Ok(tempdir) = TempDir::current(path) {
            TestDir::new(tempdir)
        } else {
            panic!("Cannot create dir in current directory")
        }
    }

    /// Returns all files created with DirBuilder
    pub fn get_files<'a>(&self) -> &Vec<PathBuf> {
        &self.files
    }
    
    /// Returns all directories created with DirBuilder
    pub fn get_dirs<'a>(&self) -> &Vec<PathBuf> {
        &self.dirs
    }


    /*
    fn load(&mut self, path: &Path) {

    }
    */

    // Helper functions
    fn new(tempdir: TempDir) -> Self {
        let root = tempdir.path().to_path_buf();
        Self {
            _tempdir: Some(tempdir),
            root,
            files: vec![],
            dirs: vec![],
        }
    }

    fn create_dir(&mut self, path: &Path) -> std::io::Result<()> {
        let mut build_path = self.root.clone();
        build_path.push(path);
        let result = fs::create_dir_all(build_path.as_path());
        if let Ok(_) = result {
            self.dirs.push(build_path);
        }
        result
    }

    fn create_file(&mut self, path: &Path, filetype: FileType) -> std::io::Result<()> {
        let mut build_path = self.root.clone();
        build_path.push(path);
        let file = fs::File::create(build_path.as_path());
        if file.is_err() {
            panic!("Create file {:?} - {:?}", build_path, file);
        }
        let file = file?;
        let mut buffer = BufWriter::new(file);

        match filetype {
            FileType::EmptyFile => { /* pass */ }
            FileType::ZeroFile(size) => {
                for _ in 0..size {
                    let _ = buffer.write(b"0")?;
                }
            }
            FileType::RandomFile(size) => {
                let mut numbuf: Vec<u8> = vec![];
                let mut rng = rand::thread_rng();
                for _ in 0..size {
                    numbuf.push(rng.gen());
                }
                let _ = buffer.write(numbuf.as_slice())?;
            }

            _ => { /* Dir - already created in create_dir */ }
        };
        self.files.push(build_path);
        Ok(())
    }

    fn remove_file(&mut self, path: &Path) -> std::io::Result<()> {
        let mut build_path = self.root.clone();
        build_path.push(path);
        if build_path.exists() {
            if build_path.is_dir() {
                fs::remove_dir_all(build_path)?;
            } else if build_path.is_file() {
                fs::remove_file(build_path)?;
            }
        }
        Ok(())
    }
}

impl DirBuilder for TestDir {
    /// Create a file or directory under the `path`
    fn create(mut self, path: &str, filetype: FileType) -> Self {
        let path = Path::new(path);
        if path.is_absolute() {
            panic!("Only relative paths are allowed.");
        }
        if filetype == FileType::Dir {
            let _ = self.create_dir(path).unwrap();
        } else {
            if let Some(p) = path.parent() {
                let _ = self.create_dir(p).unwrap();
            } // else { assume that current dir exists }
            let _ = self.create_file(path, filetype).unwrap();
        }
        self
    }

    /// Remove a file or directory under the `path`
    fn remove(mut self, path: &str) -> Self {
        let path = Path::new(path);
        if path.is_absolute() {
            panic!("Only relative paths are allowed.");
        }
        let remove = self.remove_file(path);
        if remove.is_err() {
            panic!("Cannot remove file: {:?}", remove);
        }
        self
    }

    /// Prefix `path` with the current context of the DirBuilder
    fn path(&self, path: &str) -> PathBuf {
        let mut root = self.root.clone();
        let path = PathBuf::from(path);
        root.push(path);

        root
    }

    /// Return the root path to the temporary directory
    fn root(&self) -> &Path {
        self.root.as_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testdir_temp_dir() {
        let path;
        {
            let dir = TestDir::temp();

            // Dir created
            assert!(dir.root().exists());

            let temp_dir = std::env::temp_dir();
            // Dir inside system temp dir
            assert!(dir.root().starts_with(temp_dir));

            path = dir.root().to_path_buf();
        }
        // Dir deleted after out of scope
        assert!(!path.exists());
    }

    #[test]
    fn test_testdir_current_rnd_dir() {
        let path;
        {
            let dir = TestDir::current_rnd();

            // Dir created
            assert!(dir.root().exists());

            let current_dir = std::env::current_dir().unwrap();
            // Dir inside system temp dir
            assert!(dir.root().starts_with(current_dir));

            path = dir.root().to_path_buf();
        }
        // Dir deleted after out of scope
        assert!(!path.exists());
    }

    #[test]
    fn test_testdir_current_dir() {
        let path;
        {
            let dir = TestDir::current("a/b/c");

            // Dir created
            assert!(dir.root().exists());

            let current_dir = std::env::current_dir().unwrap();
            // Dir inside system temp dir
            assert!(dir.root().starts_with(current_dir));

            path = dir.root().to_path_buf();
        }
        // Dir deleted after out of scope
        assert!(!path.exists());
    }

    #[test]
    fn test_testdir_path() {
        let str_path = "a/b/c/d/e";

        let dir = TestDir::temp().create(str_path, FileType::Dir);

        let mut root = dir.root().to_path_buf();
        let path = Path::new(str_path);

        root.push(path);

        assert_eq!(dir.path(str_path), root);
        assert!(dir.path(str_path).exists());
    }

    #[test]
    fn test_testdir_create() {
        let dir = TestDir::temp();

        let name = "dir";
        let dir = dir.create(name, FileType::Dir);
        assert!(dir.path(name).exists());
        assert!(dir.path(name).is_dir());

        let name = "empty";
        let dir = dir.create(name, FileType::EmptyFile);
        assert!(dir.path(name).exists());
        assert!(dir.path(name).is_file());
        assert_eq!(dir.path(name).metadata().unwrap().len(), 0);

        let name = "random";
        let len = 1024;
        let dir = dir.create(name, FileType::RandomFile(len));
        assert!(dir.path(name).exists());
        assert!(dir.path(name).is_file());
        assert_eq!(dir.path(name).metadata().unwrap().len(), len as u64);

        let name = "zero";
        let len = 1024;
        let dir = dir.create(name, FileType::ZeroFile(len));
        assert!(dir.path(name).exists());
        assert!(dir.path(name).is_file());
        assert_eq!(dir.path(name).metadata().unwrap().len(), len as u64);
    }

    #[test]
    fn test_testdir_remove() {
        let dir = TestDir::temp();

        let name = "test_file";
        let dir = dir.create(name, FileType::EmptyFile);
        assert!(dir.path(name).exists());

        let dir = dir.remove(name);
        assert!(!dir.path(name).exists());
    }
}

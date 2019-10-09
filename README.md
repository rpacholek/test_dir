# TestDir

Fast creation of file structure for testing purpose.

## Getting Started

Add the following dependency to Cargo manifest:

```toml
[dependencies]
test_dir = "0.1.0"
```

## Example
```rust
use test_dir::{TestDir,FileType,DirBuilder};

{
  let temp = TestDir::temp()
      .create("test/dir", FileType::Dir)
      .create("test/file", FileType::EmptyFile)
      .create("test/random_file", FileType::RandomFile(100))
      .create("otherdir/zero_file", FileType::ZeroFile(100));

  let path: PathBuf = temp.path("test/random_file");
  assert!(path.exists());
}

// temp out-of-scope -> temp dir deleted
```

## License
Licensed under MIT license, ([LICENSE](LICENSE))

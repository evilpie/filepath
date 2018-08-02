# filepath

Get the filesystem path of a file.

A simple extension trait for `File` that provides a single method `path`, which returns the path a file points to.

**Note**: Not every file has a path. The path of a file might change after creation. This method is not guaranteed to work.

OS support: Linux, Mac, Windows

```rust
use filepath::FilePath;
use std::fs::File;

let mut file = File::create("foo.txt").unwrap();
println!("{:?}", file.path());
```

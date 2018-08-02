# filepath

Get the filesystem path of a file.

[![](http://meritbadge.herokuapp.com/filepath) ![](https://img.shields.io/crates/d/filepath.png)](https://crates.io/crates/filepath)

A simple extension trait for `File` that provides a single method `path`, which returns the path of a file.

**Note**: Not every file has a path. The path might be wrong for example after moving a file.

OS support: Linux, Mac, Windows

```rust
use std::fs::File;
use filepath::FilePath;

let mut file = File::create("foo.txt").unwrap();
println!("{:?}", file.path());
```

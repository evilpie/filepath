//! # filepath
//!
//! `filepath` contains an extension trait for `std::fs::File` providing a `path` method.
//!

use std::fs::File;
use std::io;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

/// An extension trait for `std::fs::File` providing a `path` method.
pub trait FilePath {
    /// Returns the path of this file.
    ///
    /// The path might be wrong for example after moving a file.
    ///
    /// # Platform-specific behavior
    /// This function currently uses `/proc/self/fd/` on Linux, `fcntl` with `F_GETPATH` on macOS
    /// and `GetFinalPathNameByHandle` on Windows.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io;
    /// use filepath::FilePath;
    ///
    /// fn main() -> io::Result<()> {
    ///     let file = File::open("some_file")?;
    ///     let path = file.path()?;
    ///     Ok(())
    /// }
    /// ```
    fn path(&self) -> io::Result<PathBuf>;
}

impl FilePath for File {
    #[cfg(target_os = "linux")]
    fn path(&self) -> io::Result<PathBuf> {
        use std::path::Path;

        let fd = self.as_raw_fd();
        let path = Path::new("/proc/self/fd/").join(fd.to_string());
        std::fs::read_link(path)
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn path(&self) -> io::Result<PathBuf> {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        const F_GETPATH: i32 = 50;

        let fd = self.as_raw_fd();
        let mut path = vec![0; libc::PATH_MAX as usize + 1];

        unsafe {
            if libc::fcntl(fd, F_GETPATH, path.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        path.retain(|&c| c != 0);
        Ok(PathBuf::from(OsString::from_vec(path)))
    }

    #[cfg(windows)]
    fn path(&self) -> std::io::Result<PathBuf> {
        use std::ffi::OsString;
        use std::os::windows::{ffi::OsStringExt, io::AsRawHandle};
        use windows::Win32::{
            Foundation,
            Storage::FileSystem::{GetFinalPathNameByHandleW, GETFINALPATHNAMEBYHANDLE_FLAGS},
        };

        // Call with null to get the required size.
        let len = unsafe {
            let handle = Foundation::HANDLE(self.as_raw_handle());
            GetFinalPathNameByHandleW(handle, &mut [], GETFINALPATHNAMEBYHANDLE_FLAGS(0))
        };
        if len == 0 {
            return Err(io::Error::last_os_error());
        }

        let mut path = vec![0; len as usize];
        let len2 = unsafe {
            let handle = Foundation::HANDLE(self.as_raw_handle());
            GetFinalPathNameByHandleW(handle, &mut path, GETFINALPATHNAMEBYHANDLE_FLAGS(0))
        };
        // Handle unlikely case that path length changed between those two calls.
        if len2 == 0 || len2 >= len {
            return Err(io::Error::last_os_error());
        }
        path.truncate(len2 as usize);

        // Turn the \\?\UNC\ network path prefix into \\.
        let prefix = [
            '\\' as _, '\\' as _, '?' as _, '\\' as _, 'U' as _, 'N' as _, 'C' as _, '\\' as _,
        ];
        if path.starts_with(&prefix) {
            let mut network_path: Vec<u16> = vec!['\\' as u16, '\\' as u16];
            network_path.extend_from_slice(&path[prefix.len()..]);
            return Ok(PathBuf::from(OsString::from_wide(&network_path)));
        }

        // Remove the \\?\ prefix.
        let prefix = ['\\' as _, '\\' as _, '?' as _, '\\' as _];
        if path.starts_with(&prefix) {
            return Ok(PathBuf::from(OsString::from_wide(&path[prefix.len()..])));
        }

        Ok(PathBuf::from(OsString::from_wide(&path)))
    }
}

#[cfg(test)]
mod tests {
    use crate::FilePath;
    use std::fs::{remove_file, File};
    use std::io::prelude::*;

    #[test]
    fn simple() {
        let file = File::create("foobar").unwrap();
        assert_eq!(file.path().unwrap().file_name().unwrap(), "foobar");
        remove_file("foobar").unwrap();
    }

    #[test]
    fn roundtrip() {
        let mut file = File::create("bar").unwrap();
        file.write(b"abc").unwrap();
        file.flush().unwrap();

        let mut file2 = File::open(file.path().unwrap()).unwrap();
        let mut buffer = String::new();
        file2.read_to_string(&mut buffer).unwrap();

        assert_eq!(buffer, "abc");
        remove_file("bar").unwrap();
    }
}

#[cfg(target_os="macos")]
extern crate libc;

use std::path::{Path, PathBuf};
use std::io;
use std::fs::File;

#[cfg(target_os="linux")]
use std::fs::read_link;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(not(target_os="linux"))]
use std::ffi::OsString;
#[cfg(target_os="macos")]
use std::os::unix::ffi::OsStringExt;

#[cfg(target_os="macos")]
const F_GETPATH : i32 = 50;

/// An extension trait for `std::fs::File` providing an `path` method.
///
/// See the module documentation for examples.
pub trait FilePath {
    // Returns the file path this file points to.
    // Not every file has a path and the path can change after opening.
    fn path(&self) -> io::Result<PathBuf>;
}

impl FilePath for File {
    #[cfg(target_os="linux")]
    fn path(&self) -> io::Result<PathBuf> {
        let fd = self.as_raw_fd();
        let path = Path::new("/proc/self/fd/").join(fd.to_string());
        read_link(path)
    }

    #[cfg(target_os="macos")]
    fn path(&self) -> io::Result<PathBuf> {
        let fd = self.as_raw_fd();
        let mut path = vec![0; libc::PATH_MAX as usize + 1];

        unsafe {
            if libc::fcntl(fd, F_GETPATH, path.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error());
            }

            path.retain(|&c| c != 0);
            Ok(PathBuf::from(OsString::from_vec(path)))
        }
    }

    #[cfg(windows)]
    fn path(&self) -> io::Result<PathBuf> {
        let mut path = vec![0; MAX_PATH + 1];
        let len = fileapi::GetFinalPathNameByHandleW(self.as_raw_handle(),
                                                     path.as_mut_ptr(),
                                                     path.len() as u32 - 1,
                                                     0);
        if len == 0 {
            return Err(io::Error::last_os_error());
        }
        path.truncate(len as usize);
        PathBuf::from(OsString::from_wide(&path))
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;

    use FilePath;

    #[test]
    fn simple() {
        let file = fs::File::create("foobar").unwrap();
        assert_eq!(file.path().unwrap(), env::current_dir().unwrap().join(r"foobar"));
        fs::remove_file("foobar").unwrap();
    }
}

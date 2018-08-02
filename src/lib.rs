extern crate libc;

use std::path::{Path, PathBuf};
use std::io::{self, Result};
use std::fs::File;

#[cfg(target_os="linux")]
use std::fs::read_link;
#[cfg(target_os="linux")]
use std::os::unix::io::AsRawFd;

#[cfg(not(target_os="linux"))]
use std::ffi::OsString;
#[cfg(not(target_os="linux"))]
use std::os::unix::ffi::OsStringExt;

/// An extension trait for `std::fs::File` providing an `path` method.
///
/// See the module documentation for examples.
pub trait FilePath {
    // Returns the file path this file points to.
    // Not every file has a path and the path can change after opening.
    fn path(&self) -> Result<PathBuf>;
}

impl FilePath for File {
    #[cfg(target_os="linux")]
    fn path(&self) -> Result<PathBuf> {
        let fd = self.as_raw_fd();
        let path = Path::new("/proc/self/fd/").join(fd.to_string());
        read_link(path)
    }

    #[cfg(target_os="macos")]
    fn path(&self) -> Result<PathBuf> {
        let fd = self.as_raw_fd();
        let mut path = vec![0; libc::PATH_MAX as usize + 1];

        unsafe {
            let res = libc::fcntl(fd, 50, path.as_mut_ptr());
            if res < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        println!("{:?}", path);
        Ok(PathBuf::from(OsString::from_vec(path)))
    }

    #[cfg(windows)]
    fn path(&self) -> Result<PathBuf> {
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
    use std::path::PathBuf;
    use std::fs::File;
    use FilePath;

    #[test]
    fn simple() {
        let file = File::open("/tmp/foobar").unwrap();
        assert_eq!(file.path().unwrap(), PathBuf::from(r"/tmp/foobar"));
    }
}

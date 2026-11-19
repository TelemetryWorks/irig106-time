//! Partial CLI-104/107: rows on disk, publication after analysis/host flushes.
//! Traces: L3-CLI-104 <- L2-CLI-104 <- L1-CLI-004 (CSV staging clause only).
//! Traces: L3-CLI-107 <- L2-CLI-107 <- L1-CLI-007 (operational precedence clause only).
//! No time-library I/O. Atomic rename requires the destination's own directory.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) struct CsvSpool {
    path: PathBuf,
    destination: Option<PathBuf>,
    writer: Option<BufWriter<File>>,
    published: bool,
}
impl CsvSpool {
    pub(crate) fn new(destination: Option<&Path>) -> io::Result<Self> {
        if let Some(path) = destination {
            match fs::symlink_metadata(path) {
                Ok(m) if !m.is_file() => {
                    return Err(io::Error::other("CSV destination must be a regular file"));
                }
                Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
                _ => (),
            }
        }
        let directory = destination
            .map(|p| {
                p.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new("."))
                    .to_owned()
            })
            .unwrap_or_else(std::env::temp_dir);
        Self::create_in(&directory, destination)
    }
    fn create_in(directory: &Path, destination: Option<&Path>) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..32 {
            let path = directory.join(format!(
                ".irigtime-{}-{}.csv.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&path) {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        destination: destination.map(Path::to_owned),
                        writer: Some(BufWriter::new(file)),
                        published: false,
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "CSV temporary name collisions",
        ))
    }
    fn writer(&mut self) -> io::Result<&mut BufWriter<File>> {
        self.writer
            .as_mut()
            .ok_or_else(|| io::Error::other("CSV spool closed"))
    }
    pub(crate) fn copy_stdout(&mut self, out: &mut dyn Write) -> io::Result<()> {
        self.flush()?;
        let file = self.writer()?.get_mut();
        file.seek(SeekFrom::Start(0))?;
        io::copy(file, out)?;
        Ok(())
    }
    /// Last fallible operation: after rename, no writes/flushes may turn
    /// successful publication into an error with the destination changed.
    pub(crate) fn commit(&mut self) -> io::Result<()> {
        self.flush()?;
        drop(self.writer.take()); // Close before rename, including on Windows.
        let target = self
            .destination
            .as_ref()
            .ok_or_else(|| io::Error::other("CSV has no destination"))?;
        fs::rename(&self.path, target)?;
        self.published = true;
        Ok(())
    }
}
impl Write for CsvSpool {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writer()?.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer()?.flush()
    }
}
impl Drop for CsvSpool {
    fn drop(&mut self) {
        drop(self.writer.take());
        if !self.published {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    struct Dir(PathBuf);
    impl Dir {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "irigtime-spool-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn same_directory_spool_is_removed_after_rename_failure() {
        let d = Dir::new();
        let target = d.0.join("report.csv");
        let mut spool = CsvSpool::new(Some(&target)).unwrap();
        assert_eq!(spool.path.parent(), Some(d.0.as_path()));
        let temporary = spool.path.clone();
        spool.write_all(b"new").unwrap();
        fs::create_dir(&target).unwrap(); // Late destination obstruction.
        assert!(spool.commit().is_err());
        drop(spool);
        assert!(!temporary.exists());
        assert!(target.is_dir());
    }
    #[test]
    fn stdout_copy_failure_removes_temporary_file() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("copy probe"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let d = Dir::new();
        let mut spool = CsvSpool::create_in(&d.0, None).unwrap();
        let temporary = spool.path.clone();
        spool.write_all(b"data").unwrap();
        assert!(spool.copy_stdout(&mut Broken).is_err());
        drop(spool);
        assert!(!temporary.exists());
    }
    #[test]
    fn published_spool_no_longer_owns_its_old_temporary_name() {
        let d = Dir::new();
        let target = d.0.join("report.csv");
        let mut spool = CsvSpool::new(Some(&target)).unwrap();
        let temporary = spool.path.clone();
        spool.write_all(b"new").unwrap();
        spool.commit().unwrap();
        fs::write(&temporary, b"other owner").unwrap();
        drop(spool);
        assert_eq!(fs::read(temporary).unwrap(), b"other owner");
        assert_eq!(fs::read(target).unwrap(), b"new");
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn spool_write_and_flush_errors_remove_the_owned_temp() {
        for size in [1, 16_384] {
            let d = Dir::new();
            let mut spool = CsvSpool::create_in(&d.0, None).unwrap();
            let temporary = spool.path.clone();
            // Inject ENOSPC without exhausting a disk or altering the device.
            // One byte fails at flush; 16 KiB takes BufWriter's direct-write path.
            let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
            spool.writer = Some(BufWriter::new(full));
            let result = spool
                .write_all(&vec![b'x'; size])
                .and_then(|()| spool.flush());
            assert!(result.is_err());
            drop(spool);
            assert!(!temporary.exists());
        }
    }
}

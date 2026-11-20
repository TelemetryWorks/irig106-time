//! Partial CLI-104/107: rows on disk, publication after analysis/host flushes.
//! Traces: L3-CLI-104 <- L2-CLI-104 <- L1-CLI-004 (CSV staging clause only).
//! Traces: L3-CLI-107 <- L2-CLI-107 <- L1-CLI-007 (operational precedence clause only).
//! No time-library I/O. Atomic rename requires the destination's own directory.
use std::{
    collections::hash_map::RandomState,
    fs::{self, File, OpenOptions},
    hash::BuildHasher,
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
            Self::existing_permissions(path)?;
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
    fn existing_permissions(path: &Path) -> io::Result<Option<fs::Permissions>> {
        match fs::symlink_metadata(path) {
            Ok(m) if m.is_file() => {
                // Honor explicit readonly even for privileged callers. The OS
                // also checks the effective user's rights/ACLs, without truncation.
                if m.permissions().readonly() {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "CSV destination is read-only",
                    ));
                }
                Ok(Some(Self::write_probe(path, &m)?))
            }
            Ok(_) => Err(io::Error::other("CSV destination must be a regular file")),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
    fn write_probe(path: &Path, observed: &fs::Metadata) -> io::Result<fs::Permissions> {
        let mut options = OpenOptions::new();
        options.write(true); // No create/truncate: never modify target bytes.
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            use std::os::unix::fs::OpenOptionsExt;
            // Linux UAPI fcntl.h: MIPS/SPARC NONBLOCK and ARM/PPC/m68k NOFOLLOW overrides.
            let nonblock = if cfg!(any(
                target_arch = "mips",
                target_arch = "mips64",
                target_arch = "mips32r6",
                target_arch = "mips64r6"
            )) {
                1 << 7
            } else if cfg!(any(target_arch = "sparc", target_arch = "sparc64")) {
                1 << 14
            } else {
                1 << 11
            };
            let nofollow = if cfg!(any(
                target_arch = "arm",
                target_arch = "aarch64",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "m68k"
            )) {
                1 << 15
            } else {
                1 << 17
            };
            options.custom_flags(nonblock | nofollow); // O_NONBLOCK | O_NOFOLLOW
        }
        #[cfg(target_vendor = "apple")]
        {
            use std::os::unix::fs::OpenOptionsExt;
            // Darwin bsd/sys/fcntl.h: O_NONBLOCK | O_NOFOLLOW.
            options.custom_flags(0x0004 | 0x0100);
        }
        let file = options.open(path)?;
        let opened = file.metadata()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (observed.dev(), observed.ino()) != (opened.dev(), opened.ino()) {
                return Err(io::Error::other(
                    "CSV destination changed during write probe",
                ));
            }
        }
        #[cfg(not(unix))]
        let _ = observed;
        if !opened.is_file() || opened.permissions().readonly() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "CSV destination is not a writable regular file",
            ));
        }
        Ok(opened.permissions())
    }
    fn create_in(directory: &Path, destination: Option<&Path>) -> io::Result<Self> {
        Self::create_mode(directory, destination, 0o600)
    }
    fn create_mode(directory: &Path, destination: Option<&Path>, mode: u32) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let random = RandomState::new();
        Self::create_named(directory, destination, mode, || {
            let pid = std::process::id();
            let counter = NEXT.fetch_add(1, Ordering::Relaxed);
            format!(
                ".irigtime-{pid}-{counter}-{:016x}.csv.tmp",
                random.hash_one((pid, counter))
            )
        })
    }
    fn create_named(
        directory: &Path,
        destination: Option<&Path>,
        _mode: u32,
        mut name: impl FnMut() -> String,
    ) -> io::Result<Self> {
        for _ in 0..32 {
            let path = directory.join(name());
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(_mode);
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
    #[cfg(unix)]
    fn new_file_permissions(&mut self) -> io::Result<fs::Permissions> {
        // std has no safe umask getter. Sample normal creation with an EMPTY
        // exclusive file; report bytes stay private and process umask is untouched.
        let directory = self
            .path
            .parent()
            .ok_or_else(|| io::Error::other("CSV has no directory"))?;
        let mut probe = Self::create_mode(directory, None, 0o666)?;
        Ok(probe.writer()?.get_ref().metadata()?.permissions())
    }
    #[cfg(not(unix))]
    fn new_file_permissions(&mut self) -> io::Result<fs::Permissions> {
        // The report already has the platform's normal creation defaults.
        Ok(self.writer()?.get_ref().metadata()?.permissions())
    }
    /// Last fallible operation: after rename, no writes/flushes may turn
    /// successful publication into an error with the destination changed.
    pub(crate) fn commit(&mut self) -> io::Result<()> {
        self.flush()?;
        let target = self
            .destination
            .clone()
            .ok_or_else(|| io::Error::other("CSV has no destination"))?;
        let permissions = match Self::existing_permissions(&target)? {
            Some(p) => p,
            None => self.new_file_permissions()?,
        };
        self.writer()?.get_ref().set_permissions(permissions)?;
        drop(self.writer.take()); // Close before rename, including on Windows.
        fs::rename(&self.path, &target)?;
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
        #[cfg(windows)]
        if !self.published {
            // A copied readonly flag must not strand our failed temporary.
            // Never follow a link or change the destination's permissions.
            if let Ok(m) = fs::symlink_metadata(&self.path) {
                if m.is_file() && m.permissions().readonly() {
                    let mut permissions = m.permissions();
                    permissions.set_readonly(false);
                    let _ = fs::set_permissions(&self.path, permissions);
                }
            }
        }
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
    #[cfg(unix)]
    fn assert_candidate_collision(link: bool) {
        let d = Dir::new();
        let victim = d.0.join("victim");
        fs::write(&victim, b"untouched").unwrap();
        let planted = d.0.join("planted");
        if link {
            std::os::unix::fs::symlink(&victim, &planted).unwrap();
        } else {
            fs::write(&planted, b"untouched").unwrap();
        }
        let mut names = ["planted", "fresh"].into_iter();
        let mut spool =
            CsvSpool::create_named(&d.0, None, 0o600, || names.next().unwrap().into()).unwrap();
        spool.write_all(b"replacement").unwrap();
        spool.flush().unwrap();
        assert_eq!(fs::read(&victim).unwrap(), b"untouched");
        assert_eq!(fs::read(&planted).unwrap(), b"untouched");
        assert_eq!(
            fs::symlink_metadata(&planted)
                .unwrap()
                .file_type()
                .is_symlink(),
            link
        );
        assert_eq!(spool.path, d.0.join("fresh"));
        drop(spool);
        assert!(!d.0.join("fresh").exists());
    }
    #[cfg(unix)]
    #[test]
    fn occupied_file_candidate_is_never_opened() {
        assert_candidate_collision(false);
    }
    #[cfg(unix)]
    #[test]
    fn occupied_symlink_candidate_is_never_opened() {
        assert_candidate_collision(true);
    }
    #[cfg(unix)]
    #[test]
    fn report_temp_is_private_until_permission_transfer_at_commit() {
        use std::os::unix::fs::PermissionsExt;
        // A restrictive host umask can hide a too-permissive creation mode.
        // Control only a child process; never mutate this test process's umask.
        if std::env::var_os("IRIGTIME_PRIVATE_MODE_CHILD").is_none() {
            let status = std::process::Command::new("sh")
                .args(["-c", "umask 022; exec \"$1\" --exact csv_spool::tests::report_temp_is_private_until_permission_transfer_at_commit", "sh"])
                .arg(std::env::current_exe().unwrap())
                .env("IRIGTIME_PRIVATE_MODE_CHILD", "1").status().unwrap();
            assert!(status.success());
            return;
        }
        let d = Dir::new();
        let target = d.0.join("report.csv");
        fs::write(&target, b"original").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
        let mut spool = CsvSpool::new(Some(&target)).unwrap();
        assert_eq!(
            fs::metadata(&spool.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        spool.write_all(b"new").unwrap();
        spool.flush().unwrap();
        assert_eq!(
            fs::metadata(&spool.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        spool.commit().unwrap();
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
    #[test]
    fn candidate_names_have_distinct_random_suffixes() {
        let d = Dir::new();
        let a = CsvSpool::create_in(&d.0, None).unwrap();
        let b = CsvSpool::create_in(&d.0, None).unwrap();
        fn suffix(s: &CsvSpool) -> String {
            s.path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .rsplit('-')
                .next()
                .unwrap()
                .trim_end_matches(".csv.tmp")
                .into()
        }
        let (a, b) = (suffix(&a), suffix(&b));
        assert_eq!(a.len(), 16);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
    #[cfg(windows)]
    #[test]
    fn readonly_failed_temporary_is_cleaned_up() {
        let d = Dir::new();
        let mut spool = CsvSpool::create_in(&d.0, None).unwrap();
        let temporary = spool.path.clone();
        spool.write_all(b"complete report").unwrap();
        spool.flush().unwrap();
        let mut permissions = spool
            .writer()
            .unwrap()
            .get_ref()
            .metadata()
            .unwrap()
            .permissions();
        permissions.set_readonly(true);
        spool
            .writer()
            .unwrap()
            .get_ref()
            .set_permissions(permissions)
            .unwrap();
        drop(spool);
        assert!(!temporary.exists());
    }
    #[cfg(unix)]
    #[test]
    fn target_becoming_readonly_after_staging_is_not_replaced() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let d = Dir::new();
        let target = d.0.join("report.csv");
        fs::write(&target, b"original").unwrap();
        let inode = fs::metadata(&target).unwrap().ino();
        let mut spool = CsvSpool::new(Some(&target)).unwrap();
        let temporary = spool.path.clone();
        spool.write_all(b"replacement").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o444)).unwrap();
        assert!(spool.commit().is_err());
        drop(spool);
        assert_eq!(fs::read(&target).unwrap(), b"original");
        let after = fs::metadata(&target).unwrap();
        assert_eq!(after.ino(), inode);
        assert_eq!(after.permissions().mode() & 0o777, 0o444);
        assert!(!temporary.exists());
    }
    #[cfg(unix)]
    #[test]
    fn write_probe_rejects_regular_file_and_symlink_swaps() {
        let d = Dir::new();
        let target = d.0.join("report.csv");
        let saved = d.0.join("original");
        fs::write(&target, b"original").unwrap();
        let observed = fs::symlink_metadata(&target).unwrap();
        fs::rename(&target, &saved).unwrap();
        fs::write(&target, b"other inode").unwrap();
        assert!(CsvSpool::write_probe(&target, &observed).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"other inode");
        fs::remove_file(&target).unwrap();
        std::os::unix::fs::symlink(&saved, &target).unwrap();
        assert!(CsvSpool::write_probe(&target, &observed).is_err());
        assert_eq!(fs::read(&saved).unwrap(), b"original");
    }
    #[cfg(any(target_os = "linux", target_vendor = "apple"))]
    #[test]
    fn fifo_swap_probe_is_nonblocking() {
        if let Some(directory) = std::env::var_os("IRIGTIME_FIFO_PROBE_CHILD") {
            let target = PathBuf::from(directory).join("report.csv");
            fs::write(&target, b"original").unwrap();
            let observed = fs::symlink_metadata(&target).unwrap();
            fs::rename(&target, target.with_extension("saved")).unwrap();
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(&target)
                    .status()
                    .unwrap()
                    .success()
            );
            assert!(CsvSpool::write_probe(&target, &observed).is_err());
            return;
        }
        let d = Dir::new();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "csv_spool::tests::fifo_swap_probe_is_nonblocking",
            ])
            .env("IRIGTIME_FIFO_PROBE_CHILD", &d.0)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut completed = false;
        for _ in 0..500 {
            if child.try_wait().unwrap().is_some() {
                completed = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if !completed {
            child.kill().unwrap();
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            completed && output.status.success(),
            "probe blocked/failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        // Parent owns the directory even if it had to kill the child.
    }
}

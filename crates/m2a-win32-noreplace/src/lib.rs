#![deny(unsafe_code)]

//! Narrow safe wrapper for an atomic Windows move with no replacement flag.

#[cfg(windows)]
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    os::windows::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    },
    path::Path,
};

#[cfg(windows)]
const DELETE_ACCESS: u32 = 0x0001_0000;
#[cfg(windows)]
const GENERIC_READ: u32 = 0x8000_0000;
#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x0000_0001;
#[cfg(windows)]
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
#[cfg(windows)]
const FILE_SHARE_DELETE: u32 = 0x0000_0004;
#[cfg(windows)]
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
#[cfg(windows)]
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

#[cfg(windows)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentityV1 {
    pub volume_serial_number: u32,
    pub file_index: u64,
}

#[cfg(windows)]
pub struct DeleteHandleV1 {
    file: File,
    identity: FileIdentityV1,
    is_directory: bool,
}

#[cfg(windows)]
impl DeleteHandleV1 {
    pub fn open_nofollow(path: &Path, expect_directory: bool) -> io::Result<Self> {
        let file = OpenOptions::new()
            .access_mode(GENERIC_READ | DELETE_ACCESS)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)?;
        let metadata = file.metadata()?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "refusing a reparse-point delete handle",
            ));
        }
        let is_directory = metadata.is_dir();
        if is_directory != expect_directory {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "delete handle kind differs from the expected kind",
            ));
        }
        let identity = ffi::file_identity(file.as_raw_handle().cast())?;
        Ok(Self {
            file,
            identity,
            is_directory,
        })
    }

    pub fn identity(&self) -> FileIdentityV1 {
        self.identity
    }

    pub fn is_directory(&self) -> bool {
        self.is_directory
    }

    pub fn read_all(&self) -> io::Result<Vec<u8>> {
        if self.is_directory {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot read a directory delete handle",
            ));
        }
        let mut file = self.file.try_clone()?;
        file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn delete(self) -> io::Result<()> {
        ffi::delete_by_handle(self.file.as_raw_handle().cast())
    }
}

#[cfg(windows)]
pub fn move_directory_noreplace(source: &Path, destination: &Path) -> io::Result<()> {
    let source = wide_path(source)?;
    let destination = wide_path(destination)?;
    ffi::move_file_ex_noreplace(source.as_ptr(), destination.as_ptr())
}

#[cfg(windows)]
fn wide_path(path: &Path) -> io::Result<Vec<u16>> {
    let mut encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if encoded.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows path contains an interior NUL",
        ));
    }
    encoded.push(0);
    Ok(encoded)
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod ffi {
    use std::{ffi::c_void, io};

    const FILE_DISPOSITION_INFO_CLASS: i32 = 4;

    #[repr(C)]
    struct FileDispositionInfo {
        delete_file: i32,
    }

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(
            existing_file_name: *const u16,
            new_file_name: *const u16,
            flags: u32,
        ) -> i32;

        fn SetFileInformationByHandle(
            file: *mut c_void,
            file_information_class: i32,
            file_information: *const c_void,
            buffer_size: u32,
        ) -> i32;

        fn GetFileInformationByHandle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    pub(super) fn move_file_ex_noreplace(
        source: *const u16,
        destination: *const u16,
    ) -> io::Result<()> {
        // A zero flag word intentionally omits MOVEFILE_REPLACE_EXISTING.
        let result = unsafe { MoveFileExW(source, destination, 0) };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub(super) fn delete_by_handle(file: *mut c_void) -> io::Result<()> {
        let disposition = FileDispositionInfo { delete_file: 1 };
        let result = unsafe {
            SetFileInformationByHandle(
                file,
                FILE_DISPOSITION_INFO_CLASS,
                (&raw const disposition).cast(),
                size_of::<FileDispositionInfo>() as u32,
            )
        };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub(super) fn file_identity(file: *mut c_void) -> io::Result<super::FileIdentityV1> {
        let mut information = ByHandleFileInformation {
            file_attributes: 0,
            creation_time: FileTime { low: 0, high: 0 },
            last_access_time: FileTime { low: 0, high: 0 },
            last_write_time: FileTime { low: 0, high: 0 },
            volume_serial_number: 0,
            file_size_high: 0,
            file_size_low: 0,
            number_of_links: 0,
            file_index_high: 0,
            file_index_low: 0,
        };
        let result = unsafe { GetFileInformationByHandle(file, &raw mut information) };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(super::FileIdentityV1 {
                volume_serial_number: information.volume_serial_number,
                file_index: ((information.file_index_high as u64) << 32)
                    | information.file_index_low as u64,
            })
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use std::{
        ffi::OsString,
        fs,
        os::windows::ffi::OsStringExt,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{move_directory_noreplace, wide_path};

    #[test]
    fn zero_flag_move_preserves_supplementary_unicode_path() {
        let root = test_root("unicode");
        let source = root.join("źródło-🚀");
        let destination = root.join("cel-𐐷");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("payload.bin"), b"owned").unwrap();
        move_directory_noreplace(&source, &destination).unwrap();
        assert!(!source.exists());
        assert_eq!(fs::read(destination.join("payload.bin")).unwrap(), b"owned");
        remove_test_tree(&root);
    }

    #[test]
    fn wide_path_rejects_interior_nul() {
        let path = PathBuf::from(OsString::from_wide(&[
            b'C' as u16,
            b':' as u16,
            b'\\' as u16,
            b'a' as u16,
            0,
            b'b' as u16,
        ]));
        let error = wide_path(&path).expect_err("interior NUL must fail before Win32");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn missing_source_surfaces_get_last_error_and_does_not_create_destination() {
        let root = test_root("missing");
        let source = root.join("missing-source");
        let destination = root.join("destination");
        let error = move_directory_noreplace(&source, &destination)
            .expect_err("missing source must preserve Win32 failure");
        assert!(matches!(error.raw_os_error(), Some(2 | 3)), "{error:?}");
        assert!(!destination.exists());
        remove_test_tree(&root);
    }

    #[test]
    fn existing_destination_is_preserved_byte_for_byte() {
        let root = test_root("preserve");
        let source = root.join("source");
        let destination = root.join("destination");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("source.bin"), b"source bytes").unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("user.bin"), b"user bytes").unwrap();
        move_directory_noreplace(&source, &destination)
            .expect_err("zero flag move must not replace an existing destination");
        assert_eq!(
            fs::read(destination.join("user.bin")).unwrap(),
            b"user bytes"
        );
        assert_eq!(
            fs::read(source.join("source.bin")).unwrap(),
            b"source bytes"
        );
        remove_test_tree(&root);
    }

    fn test_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "m2a-win32-noreplace-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        root
    }

    fn remove_test_tree(root: &Path) {
        for child in fs::read_dir(root).unwrap() {
            let child = child.unwrap();
            if child.file_type().unwrap().is_dir() {
                for leaf in fs::read_dir(child.path()).unwrap() {
                    let leaf = leaf.unwrap();
                    assert!(leaf.file_type().unwrap().is_file());
                    fs::remove_file(leaf.path()).unwrap();
                }
                fs::remove_dir(child.path()).unwrap();
            } else {
                fs::remove_file(child.path()).unwrap();
            }
        }
        fs::remove_dir(root).unwrap();
    }
}

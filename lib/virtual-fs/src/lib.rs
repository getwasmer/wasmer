use std::any::Any;
use std::ffi::OsString;
use std::fmt;
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::debug;

pub mod host_fs;
//pub mod vfs_fs;
pub mod mem_fs;

pub trait FileSystem: Send + Sync + 'static {
    fn read_dir(&self, path: &Path) -> Result<ReadDir, FsError>;
    fn create_dir(&self, path: &Path) -> Result<(), FsError>;
    fn remove_dir(&self, path: &Path) -> Result<(), FsError>;
    fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError>;

    fn remove_file(&self, path: &Path) -> Result<(), FsError>;
    fn new_open_options(&self) -> OpenOptions;
}

pub trait FileOpener {
    fn open(
        &mut self,
        path: &Path,
        conf: &OpenOptionsConfig,
    ) -> Result<Box<dyn VirtualFile>, FsError>;
}

#[derive(Debug, Clone)]
pub struct OpenOptionsConfig {
    read: bool,
    write: bool,
    create_new: bool,
    create: bool,
    append: bool,
    truncate: bool,
}

impl OpenOptionsConfig {
    pub const fn read(&self) -> bool {
        self.read
    }

    pub const fn write(&self) -> bool {
        self.write
    }

    pub const fn create_new(&self) -> bool {
        self.create_new
    }

    pub const fn create(&self) -> bool {
        self.create
    }

    pub const fn append(&self) -> bool {
        self.append
    }

    pub const fn truncate(&self) -> bool {
        self.truncate
    }
}

// TODO: manually implement debug

pub struct OpenOptions {
    opener: Box<dyn FileOpener>,
    conf: OpenOptionsConfig,
}

impl OpenOptions {
    pub fn new(opener: Box<dyn FileOpener>) -> Self {
        Self {
            opener,
            conf: OpenOptionsConfig {
                read: false,
                write: false,
                create_new: false,
                create: false,
                append: false,
                truncate: false,
            },
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.conf.read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.conf.write = write;
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.conf.append = append;
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.conf.truncate = truncate;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.conf.create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.conf.create_new = create_new;
        self
    }

    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<Box<dyn VirtualFile>, FsError> {
        self.opener.open(path.as_ref(), &self.conf)
    }
}

/// This trait relies on your file closing when it goes out of scope via `Drop`
#[typetag::serde(tag = "type")]
pub trait VirtualFile: fmt::Debug + Send + Write + Read + Seek + 'static + Upcastable {
    /// the last time the file was accessed in nanoseconds as a UNIX timestamp
    fn last_accessed(&self) -> u64;

    /// the last time the file was modified in nanoseconds as a UNIX timestamp
    fn last_modified(&self) -> u64;

    /// the time at which the file was created in nanoseconds as a UNIX timestamp
    fn created_time(&self) -> u64;

    /// set the last time the file was accessed in nanoseconds as a UNIX timestamp
    fn set_last_accessed(&self, _last_accessed: u64) {
        debug!("{:?} did nothing in VirtualFile::set_last_accessed due to using the default implementation", self);
    }

    /// set the last time the file was modified in nanoseconds as a UNIX timestamp
    fn set_last_modified(&self, _last_modified: u64) {
        debug!("{:?} did nothing in VirtualFile::set_last_modified due to using the default implementation", self);
    }

    /// set the time at which the file was created in nanoseconds as a UNIX timestamp
    fn set_created_time(&self, _created_time: u64) {
        debug!(
            "{:?} did nothing in VirtualFile::set_created_time to using the default implementation",
            self
        );
    }

    /// the size of the file in bytes
    fn size(&self) -> u64;

    /// Change the size of the file, if the `new_size` is greater than the current size
    /// the extra bytes will be allocated and zeroed
    fn set_len(&mut self, new_size: u64) -> Result<(), FsError>;

    /// Request deletion of the file
    fn unlink(&mut self) -> Result<(), FsError>;

    /// Store file contents and metadata to disk
    /// Default implementation returns `Ok(())`.  You should implement this method if you care
    /// about flushing your cache to permanent storage
    fn sync_to_disk(&self) -> Result<(), FsError> {
        Ok(())
    }

    /// Moves the file to a new location
    /// NOTE: the signature of this function will change before stabilization
    // TODO: stablizie this in 0.7.0 or 0.8.0 by removing default impl
    fn rename_file(&self, _new_name: &std::path::Path) -> Result<(), FsError> {
        panic!("Default implementation for now as this method is unstable; this default implementation or this entire method may be removed in a future release.");
    }

    /// Returns the number of bytes available.  This function must not block
    fn bytes_available(&self) -> Result<usize, FsError>;

    /// Used for polling.  Default returns `None` because this method cannot be implemented for most types
    /// Returns the underlying host fd
    fn get_raw_fd(&self) -> Option<i32> {
        None
    }
}

// Implementation of `Upcastable` taken from https://users.rust-lang.org/t/why-does-downcasting-not-work-for-subtraits/33286/7 .
/// Trait needed to get downcasting from `VirtualFile` to work.
pub trait Upcastable {
    fn upcast_any_ref(&'_ self) -> &'_ dyn Any;
    fn upcast_any_mut(&'_ mut self) -> &'_ mut dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Any + fmt::Debug + 'static> Upcastable for T {
    #[inline]
    fn upcast_any_ref(&'_ self) -> &'_ dyn Any {
        self
    }
    #[inline]
    fn upcast_any_mut(&'_ mut self) -> &'_ mut dyn Any {
        self
    }
    #[inline]
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl dyn VirtualFile + 'static {
    #[inline]
    pub fn downcast_ref<T: 'static>(&'_ self) -> Option<&'_ T> {
        self.upcast_any_ref().downcast_ref::<T>()
    }
    #[inline]
    pub fn downcast_mut<T: 'static>(&'_ mut self) -> Option<&'_ mut T> {
        self.upcast_any_mut().downcast_mut::<T>()
    }
}

/// Error type for external users
#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
pub enum FsError {
    /// The fd given as a base was not a directory so the operation was not possible
    #[error("fd not a directory")]
    BaseNotDirectory,
    /// Expected a file but found not a file
    #[error("fd not a file")]
    NotAFile,
    /// The fd given was not usable
    #[error("invalid fd")]
    InvalidFd,
    /// File exists
    #[error("file exists")]
    AlreadyExists,
    /// Something failed when doing IO. These errors can generally not be handled.
    /// It may work if tried again.
    #[error("io error")]
    IOError,
    /// The address was in use
    #[error("address is in use")]
    AddressInUse,
    /// The address could not be found
    #[error("address could not be found")]
    AddressNotAvailable,
    /// A pipe was closed
    #[error("broken pipe (was closed)")]
    BrokenPipe,
    /// The connection was aborted
    #[error("connection aborted")]
    ConnectionAborted,
    /// The connection request was refused
    #[error("connection refused")]
    ConnectionRefused,
    /// The connection was reset
    #[error("connection reset")]
    ConnectionReset,
    /// The operation was interrupted before it could finish
    #[error("operation interrupted")]
    Interrupted,
    /// Invalid internal data, if the argument data is invalid, use `InvalidInput`
    #[error("invalid internal data")]
    InvalidData,
    /// The provided data is invalid
    #[error("invalid input")]
    InvalidInput,
    /// Could not perform the operation because there was not an open connection
    #[error("connection is not open")]
    NotConnected,
    /// The requested file or directory could not be found
    #[error("entity not found")]
    EntityNotFound,
    /// The requested device couldn't be accessed
    #[error("can't access device")]
    NoDevice,
    /// Caller was not allowed to perform this operation
    #[error("permission denied")]
    PermissionDenied,
    /// The operation did not complete within the given amount of time
    #[error("time out")]
    TimedOut,
    /// Found EOF when EOF was not expected
    #[error("unexpected eof")]
    UnexpectedEof,
    /// Operation would block, this error lets the caller know that they can try again
    #[error("blocking operation. try again")]
    WouldBlock,
    /// A call to write returned 0
    #[error("write returned 0")]
    WriteZero,
    /// Some other unhandled error. If you see this, it's probably a bug.
    #[error("unknown error found")]
    UnknownError,
}

impl From<io::Error> for FsError {
    fn from(io_error: io::Error) -> Self {
        match io_error.kind() {
            io::ErrorKind::AddrInUse => FsError::AddressInUse,
            io::ErrorKind::AddrNotAvailable => FsError::AddressNotAvailable,
            io::ErrorKind::AlreadyExists => FsError::AlreadyExists,
            io::ErrorKind::BrokenPipe => FsError::BrokenPipe,
            io::ErrorKind::ConnectionAborted => FsError::ConnectionAborted,
            io::ErrorKind::ConnectionRefused => FsError::ConnectionRefused,
            io::ErrorKind::ConnectionReset => FsError::ConnectionReset,
            io::ErrorKind::Interrupted => FsError::Interrupted,
            io::ErrorKind::InvalidData => FsError::InvalidData,
            io::ErrorKind::InvalidInput => FsError::InvalidInput,
            io::ErrorKind::NotConnected => FsError::NotConnected,
            io::ErrorKind::NotFound => FsError::EntityNotFound,
            io::ErrorKind::PermissionDenied => FsError::PermissionDenied,
            io::ErrorKind::TimedOut => FsError::TimedOut,
            io::ErrorKind::UnexpectedEof => FsError::UnexpectedEof,
            io::ErrorKind::WouldBlock => FsError::WouldBlock,
            io::ErrorKind::WriteZero => FsError::WriteZero,
            io::ErrorKind::Other => FsError::IOError,
            // if the following triggers, a new error type was added to this non-exhaustive enum
            _ => FsError::UnknownError,
        }
    }
}

#[derive(Debug)]
pub struct ReadDir {
    // TODO: to do this properly we need some kind of callback to the core FS abstraction
    data: Vec<DirEntry>,
    index: usize,
}

impl ReadDir {
    pub fn new(data: Vec<DirEntry>) -> Self {
        Self { data, index: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    // weird hack, to fix this we probably need an internal trait object or callbacks or something
    pub metadata: Result<Metadata, FsError>,
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn metadata(&self) -> Result<Metadata, FsError> {
        self.metadata.clone()
    }

    pub fn file_type(&self) -> Result<FileType, FsError> {
        let metadata = self.metadata.clone()?;
        Ok(metadata.file_type())
    }

    pub fn file_name(&self) -> OsString {
        self.path.as_os_str().to_owned()
    }
}

#[derive(Clone, Debug)]
// TODO: review this, proper solution would probably use a trait object internally
pub struct Metadata {
    pub ft: FileType,
    pub accessed: u64,
    pub created: u64,
    pub modified: u64,
    pub len: u64,
}

impl Metadata {
    pub fn is_file(&self) -> bool {
        self.ft.is_file()
    }
    pub fn is_dir(&self) -> bool {
        self.ft.is_dir()
    }
    pub fn accessed(&self) -> u64 {
        self.accessed
    }
    pub fn created(&self) -> u64 {
        self.created
    }
    pub fn modified(&self) -> u64 {
        self.modified
    }
    pub fn file_type(&self) -> FileType {
        self.ft.clone()
    }
}

#[derive(Clone, Debug)]
// TODO: review this, proper solution would probably use a trait object internally
pub struct FileType {
    pub dir: bool,
    pub file: bool,
    pub symlink: bool,
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.dir
    }
    pub fn is_file(&self) -> bool {
        self.file
    }
    pub fn is_symlink(&self) -> bool {
        self.symlink
    }
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry, FsError>;

    fn next(&mut self) -> Option<Result<DirEntry, FsError>> {
        if let Some(v) = self.data.get(self.index).cloned() {
            self.index += 1;
            return Some(Ok(v));
        }
        None
    }
}

/*impl From<vfs::VfsError> for FsError {
    fn from(vfs_error: vfs::VfsError) -> Self {
        match vfs_error {
            vfs::VfsError::IoError(io_error) => io_error.into(),
            _ => todo!("Not yet handled vfs error type {:?}", vfs_error)
        }
    }
}*/

use dashmap::{DashMap, Entry};
use env_logger::Env;
use pion::{BnPion, IPion};
use rsbinder::{
    DEFAULT_BINDER_CONTROL_PATH, DEFAULT_BINDERFS_PATH, Interface, ParcelFileDescriptor,
    ProcessState, SIBinder, Status, StatusCode, binderfs, status::Result,
};
use std::{
    fs::File,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    path::Path,
    process::Command,
};

#[derive(Debug, Default)]
struct Pion(DashMap<(u64, u64), SIBinder>);
impl Pion {
    fn entry<'a>(&'a self, fd: &ParcelFileDescriptor) -> Result<Entry<'a, (u64, u64), SIBinder>> {
        let file: File = fd
            .as_ref()
            .try_clone()
            .map_err(|_| StatusCode::ServiceSpecific(0))?
            .into();

        let metadata = file
            .metadata()
            .map_err(|_| StatusCode::ServiceSpecific(0))?;

        if metadata.permissions().readonly() {
            return Err(StatusCode::PermissionDenied.into());
        }

        Ok(self.0.entry((metadata.dev(), metadata.ino())))
    }
}
impl Interface for Pion {}
impl IPion for Pion {
    fn register(&self, fd: &ParcelFileDescriptor, binder_ref: &SIBinder) -> Result<()> {
        let entry = self.entry(fd)?;
        match entry {
            Entry::Occupied(_) => Err(Status::new_service_specific_error(
                1,
                Some("couldn't register object".into()),
            )),
            Entry::Vacant(entry) => {
                entry.insert(binder_ref.clone());
                println!("Registered object");
                Ok(())
            }
        }
    }

    fn exchange(&self, fd: &ParcelFileDescriptor) -> Result<SIBinder> {
        match self.entry(fd)? {
            Entry::Occupied(entry) => {
                println!("Exchanged object");
                Ok(entry.get().clone())
            }
            Entry::Vacant(_) => Err(Status::new_service_specific_error(
                0,
                Some("couldn't find object".into()),
            )),
        }
    }
}

fn is_mounted() -> bool {
    let mounts = std::fs::read_to_string("/proc/mounts").unwrap();
    mounts.contains("binder")
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let binderfs_path = Path::new(DEFAULT_BINDERFS_PATH);
    let control_path = Path::new(DEFAULT_BINDER_CONTROL_PATH);
    let device_path = binderfs_path.join("binder");

    // Create binder control path if it doesn't exist.
    if !binderfs_path.exists() {
        std::fs::create_dir_all(binderfs_path).expect("Failed to create the binderfs path");
    }

    // Check if binder control path is a directory.
    if !binderfs_path.is_dir() {
        panic!("{} is not a directory", binderfs_path.display());
    }

    // Mount binderfs if it is not mounted.
    if !is_mounted() {
        Command::new("mount")
            .arg("-t")
            .arg("binder")
            .arg("binder")
            .arg(binderfs_path)
            .status()
            .expect("Failed to mount binderfs");
    }

    let _ = std::fs::remove_file(&device_path);
    let _ = binderfs::add_device(control_path, "binder");

    // Initialize ProcessState with the default binder path and the default max threads.
    println!("Initializing ProcessState...");
    let process_state = ProcessState::init_default();

    // Start the thread pool.
    // This is optional. If you don't call this, only one thread will be created to handle the binder transactions.
    println!("Starting thread pool...");
    ProcessState::start_thread_pool();

    // Create a binder service.
    println!("Creating service...");
    let service = BnPion::new_binder(Pion::default());

    process_state
        .become_context_manager(service.as_binder())
        .expect("Couldn't become context manager");

    let mut perms = std::fs::metadata(&device_path)
        .expect("IO error")
        .permissions();
    perms.set_mode(0o666);

    std::fs::set_permissions(&device_path, perms).expect("Couldn't set permissions");

    let _ = symlink(device_path, Path::new("/dev/binder"));

    // Join the thread pool.
    // This is a blocking call. It will return when the thread pool is terminated.
    ProcessState::join_thread_pool().expect("Couldn't join thread pool")
}

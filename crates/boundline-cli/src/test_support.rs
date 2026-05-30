use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, MutexGuard};

static CURRENT_DIR_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub(crate) fn acquire_process_state_lock() -> MutexGuard<'static, ()> {
    CURRENT_DIR_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) struct CurrentDirGuard {
    original: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl CurrentDirGuard {
    pub(crate) fn change_to(path: &Path) -> Self {
        let lock = acquire_process_state_lock();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self { original, _lock: lock }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original).unwrap();
    }
}

use std::{fs::File, os::fd::AsRawFd, path::Path};

use libc::madvise;

pub fn initialize_file(path: impl AsRef<Path>) -> (&'static [u8], u64) {
    let file = std::fs::File::open(path).unwrap();
    let len = file.metadata().unwrap().len();
    (mapped_file_memory(file), len)
}

#[allow(clippy::needless_pass_by_value, clippy::ptr_eq)]
fn mapped_file_memory(file: File) -> &'static [u8] {
    let file_len = usize::try_from(file.metadata().unwrap().len()).unwrap();
    let mmap_res = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            file_len,
            libc::PROT_READ,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            0,
        )
    };

    assert!(mmap_res != libc::MAP_FAILED, "mmap failed!!");

    let madvise_res = unsafe { madvise(mmap_res, file_len, libc::MADV_WILLNEED) };
    assert!(madvise_res == 0, "madvise failed!!");

    unsafe { std::slice::from_raw_parts(mmap_res as *const u8, file_len) }
}

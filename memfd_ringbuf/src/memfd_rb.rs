// Core
use std::{
    io,
    mem::{size_of, MaybeUninit},
    os::fd::{ AsFd, FromRawFd, IntoRawFd, OwnedFd },
    ptr::{self, NonNull},
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    cell::UnsafeCell,
};

// libraries
use memmap2::MmapMut;
use rustix::{
    fs::{ftruncate, memfd_create, MemfdFlags},
    mm::{mmap, munmap, MapFlags, ProtFlags },
    io::dup,
};
use ringbuf::storage::{Owning, Storage};

// Crate
use crate::traits::SharedPod;

const MAGIC: u64 =  u64::from_be_bytes(*b"SHMRINGU");
#[repr(C, align(64))]
pub struct CacheAligned<T>(pub T);

#[repr(C, align(64))]
pub struct Header {
    pub magic: u64,
    pub version: u32,
    pub flags: u32,
    pub total_size: u64,
    pub ring_capacity: u64,
    pub initialized: AtomicU32,
}

#[repr(C, align(64))]
pub struct SharedLayout<T: SharedPod, const N: usize> {
    pub header: Header,
    pub read_index: CacheAligned<AtomicU64>,
    pub write_index: CacheAligned<AtomicU64>,
    pub storage: [MaybeUninit<T>; N],
}

pub struct MemfdStorage<T: SharedPod, const N: usize> {
    fd: OwnedFd,
    ptr: NonNull<SharedLayout<T, N>>,
    size: usize,
}

impl<T: SharedPod, const N: usize> MemfdStorage<T, N> {
    pub fn create(name: &str) -> io::Result<Self> {
        let fd = memfd_create(name, MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING)
            .map_err(io::Error::from)?;

        let total_size = size_of::<SharedLayout<T, N>>();

        /* Resize memfd */
        ftruncate(fd.as_fd(), total_size as u64)
            .map_err(io::Error::from)?;

        let raw_ptr = unsafe {
            mmap(
                ptr::null_mut(),
                total_size,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::SHARED,
                fd.as_fd(),
                0,
            )
        }.map_err(io::Error::from)?
        as *mut SharedLayout<T, N>;

        let ptr = NonNull::new(raw_ptr)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Mmap returned null pointer",
                )
            })?;

        unsafe {
            ptr::write(
                raw_ptr,
                SharedLayout {
                    header: Header {
                        magic: MAGIC,
                        version: 1,
                        flags: 0,
                        total_size: total_size as u64,
                        ring_capacity: N as u64,
                        initialized: AtomicU32::new(0),
                    },

                    read_index: CacheAligned(AtomicU64::new(0)),

                    write_index: CacheAligned(AtomicU64::new(0)),

                    storage: std::array::from_fn(|i| {
                        MaybeUninit::uninit()
                    }),
                },
            );

            /* Marks as fully initialized state */
            ptr.as_ref()
                .header
                .initialized
                .store(1, Ordering::Release);

            Ok(Self {
                fd,
                ptr,
                size: total_size,
            })
        }
    }

    pub fn attach(fd: OwnedFd) -> io::Result<Self> {
        let total_size = size_of::<SharedLayout<T, N>>();

        let raw_ptr = unsafe {
            mmap(
                ptr::null_mut(),
                total_size,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::SHARED,
                fd.as_fd(),
                0
            )
        }.map_err(io::Error::from)? as *mut SharedLayout<T, N>;

        let ptr = NonNull::new(raw_ptr)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Null map pointer",
                )
            })?;

        let cleanup = || unsafe {
            munmap(raw_ptr.cast(), total_size).ok();
        };

        unsafe {
            let layout = ptr.as_ref();

            if layout.header.magic != MAGIC {
                cleanup();
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid memfd magic",
                ));
            }

            /* Validate Version  */
            if layout.header.version != 1 {
                cleanup();
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported shared memory version",
                ));
            }

            if layout.header.ring_capacity != N as u64 {
                cleanup();
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Ring capacity mismatch",
                ));
            }

            /* Wait/check initialization */
            if layout.header.initialized.load(Ordering::Acquire) != 1 {
                cleanup();
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Shared memory not initialized",
                ));
            }

            Ok(Self {
                fd,
                ptr,
                size: total_size,
            })
        }
    }

    pub fn dup_fd(&self) -> io::Result<OwnedFd> {
        dup(self.fd.as_fd()).map_err(io::Error::from)
    }

    pub fn layout( &self) -> &SharedLayout<T, N> {
        unsafe { self.ptr.as_ref() }
    }
}

impl <T: SharedPod, const N: usize> Drop for MemfdStorage<T, N> {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr.as_ptr().cast(), self.size).ok();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use bytemuck::{Pod, Zeroable};
    use std::mem::{align_of, size_of};
    use std::sync::atomic::Ordering;

    #[repr(C)]
    #[derive(Copy, Clone, Zeroable, Pod, Debug, PartialEq)]
    struct TestMessage {
        pub id: u64,
        pub value: u32,
        _padding: u32,
    }

    unsafe impl SharedPod for TestMessage {}

    #[test]
    fn create_memfd_storage() {
        const N: usize = 128;

        let storage =
            MemfdStorage::<TestMessage, N>::create("test-ring")
                .unwrap();

        let layout = unsafe {
            storage.ptr.as_ref()
        };

        /*
         * Validate header
         */
        assert_eq!(
            layout.header.magic,
            MAGIC
        );

        assert_eq!(layout.header.version, 1);

        assert_eq!(
            layout.header.ring_capacity,
            N as u64
        );

        assert_eq!(
            layout.header.initialized.load(Ordering::Acquire),
            1
        );

        /*
         * Validate indexes
         */
        assert_eq!(
            layout.read_index.0.load(Ordering::Acquire),
            0
        );

        assert_eq!(
            layout.write_index.0.load(Ordering::Acquire),
            0
        );

    }

    #[test]
    fn validate_alignment() {
        assert_eq!(
            align_of::<Header>(),
            64
        );

        assert_eq!(
            align_of::<SharedLayout<TestMessage, 8>>(),
            64
        );

        assert_eq!(
            align_of::<CacheAligned<AtomicU64>>(),
            64
        );
    }

    #[test]
    fn validate_sizes() {
        println!(
            "Header size: {}",
            size_of::<Header>()
        );

        println!(
            "SharedLayout size: {}",
            size_of::<SharedLayout<TestMessage, 128>>()
        );

        /*
         * CacheAligned should occupy full cacheline.
         */
        assert_eq!(
            size_of::<CacheAligned<AtomicU64>>(),
            64
        );
    }

    #[test]
    fn write_and_read_() {
        const N: usize = 16;

        let storage =
            MemfdStorage::<TestMessage, N>::create("rw-test")
                .unwrap();

        let layout = storage.layout();

        let slot = &layout.storage[0];

        let msg = TestMessage {
            id: 42,
            value: 777,
            _padding: 0,
        };

        unsafe {
            slot.as_ptr().cast::<TestMessage>().cast_mut().write(msg);
        }

        let read_back = unsafe {
            slot.assume_init()
        };

        assert_eq!(msg, read_back);
    }

    #[test]
    fn attach_existing_memfd() {
        const N: usize = 64;
        let storage = MemfdStorage::<TestMessage, N>::create(
            "attach-test",
        ).unwrap();

        let dup_fd = dup(storage.fd.as_fd()).unwrap();

        /* Now use attach function to get the attached fd */
        let attached_storage =
            MemfdStorage::<TestMessage, N>::attach(dup_fd).unwrap();

        assert_eq!(
            attached_storage.layout().header.magic,
            MAGIC
        );

        assert_eq!(attached_storage.layout().header.version, 1);

        /* write test */
        storage.layout()
            .write_index
            .0
            .store(42, Ordering::Release);

        /* Read from attached storage */
        let value = attached_storage.layout().write_index.0.load(Ordering::Acquire);
        assert_eq!(value, 42);

        let msg = TestMessage {
            id: 134,
            value: 1466,
            _padding: 0,
        };

        unsafe {
            storage.layout().storage[0].as_ptr().cast::<TestMessage>().cast_mut().write(msg);
        }

        /* Read back from attached fd */
        let read_back = unsafe {
            attached_storage.layout().storage[0].assume_init()
        };

        assert_eq!(msg, read_back);

    }
}

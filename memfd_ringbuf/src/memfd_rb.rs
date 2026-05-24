// Core
use std::{
    io,
    mem::{size_of, MaybeUninit},
    os::fd::{ AsFd, FromRawFd, IntoRawFd },
    ptr::{self, NonNull},
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    cell::UnsafeCell,
};
use std::fs::File;
// libraries
use memmap2::MmapMut;
use rustix::{
    fs::{ftruncate, memfd_create, MemfdFlags},
};
use ringbuf::storage::Storage;

// Crate
use crate::traits::SharedPod;

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

#[repr(C)]
pub struct Slot<T: SharedPod> {
    pub sequence: u64,
    pub crc32: u32,
    pub _padding: u32,
    pub value: UnsafeCell<MaybeUninit<T>>,
}

#[repr(C, align(64))]
pub struct SharedLayout<T: SharedPod, const N: usize> {
    pub header: Header,
    pub read_index: CacheAligned<AtomicU64>,
    pub write_index: CacheAligned<AtomicU64>,
    pub storage: [Slot<T>; N],
}

pub struct MemfdStorage<T: SharedPod, const N: usize> {
    mmap: MmapMut,
    ptr: NonNull<SharedLayout<T, N>>,
}

impl<T: SharedPod, const N: usize> MemfdStorage<T, N> {
    pub fn create(name: &str) -> io::Result<Self> {
        const MAGIC: u64 = 0x53484D52494E4755; // SHMRINGU

        let fd = memfd_create(name, MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING)
            .map_err(io::Error::from)?;

        let total_size = size_of::<SharedLayout<T, N>>();

        ftruncate(fd.as_fd(), total_size as u64)
            .map_err(io::Error::from)?;

        /* Convert OwnedFD -> File. Becuase memmap2 works with file */
        let file = unsafe {
            File::from_raw_fd(fd.into_raw_fd())
        };

        let mut mmap = unsafe {
            MmapMut::map_mut(&file)?
        };
        let raw_ptr = mmap.as_mut_ptr() as *mut SharedLayout<T, N>;

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

                    storage: std::array::from_fn(|i| Slot {
                        sequence: i as u64,
                        crc32: 0,
                        _padding: 0,
                        value: UnsafeCell::new(MaybeUninit::uninit()),
                    }),
                },
            );

            /* Marks as fully initialized state */
            (*raw_ptr)
                .header
                .initialized
                .store(1, Ordering::Release);

            Ok(Self {
                mmap,
                ptr: NonNull::new(raw_ptr).unwrap(),
            })
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
            0x53484D52494E4755
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

        /*
         * Validate sequence initialization
         */
        for (i, slot) in layout.storage.iter().enumerate() {
            assert_eq!(slot.sequence, i as u64);
        }
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
            "Slot size: {}",
            size_of::<Slot<TestMessage>>()
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
    fn write_and_read_slot() {
        const N: usize = 16;

        let storage =
            MemfdStorage::<TestMessage, N>::create("rw-test")
                .unwrap();

        let layout = unsafe {
            storage.ptr.as_ref()
        };

        let slot = &layout.storage[0];

        let msg = TestMessage {
            id: 42,
            value: 777,
            _padding: 0,
        };

        unsafe {
            (*slot.value.get()).write(msg);
        }

        let read_back = unsafe {
            (*slot.value.get()).assume_init()
        };

        assert_eq!(msg, read_back);
    }
}

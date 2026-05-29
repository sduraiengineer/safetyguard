use bytemuck::Pod;
use rustix::io::Result;

pub unsafe trait SharedPod: Pod {}

unsafe impl SharedPod for u32 {}
unsafe impl SharedPod for u64 {}
unsafe impl SharedPod for i32 {}
unsafe impl SharedPod for i64 {}
unsafe impl SharedPod for u8 {}
unsafe impl SharedPod for i8 {}
unsafe impl SharedPod for usize {}
unsafe impl SharedPod for isize {}

pub trait RbProducerNotify {
    fn notify(&self) -> Result<()>;
}
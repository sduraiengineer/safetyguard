
use rustix::{
    event::{eventfd, EventfdFlags},
    fd::OwnedFd,
    io::{read, write, Result},
};
use ringbuf::traits::{Observer, Producer};
use crate::traits::RbProducerNotify;

pub struct EventFdNotifier {
    pub fd: OwnedFd,
}

impl EventFdNotifier {
    pub fn new() -> Result<Self> {
        let fd = eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)?;
        Ok(Self { fd })
    }

    pub fn from_fd(fd: OwnedFd) -> Self {
        Self { fd }
    }

    pub fn wait(&self) -> Result<u64> {
        let mut buf = [0u8; 8];
        read(&self.fd, &mut buf)?;
        Ok(u64::from_ne_bytes(buf))
    }
}

impl RbProducerNotify for EventFdNotifier {
    fn notify(&self) -> Result<()> {
        let val: u64 = 1;
        write(&self.fd, &val.to_ne_bytes())?;
        Ok(())
    }
}

pub struct NotifyingProducer<P: Producer, N: RbProducerNotify> {
    base: P,
    notifier: N,
}

impl<P: Producer, N: RbProducerNotify> NotifyingProducer<P, N> {
    pub fn new(base: P, notifier: N) -> Self {
        Self { base, notifier }
    }
}

impl<P: Producer, N: RbProducerNotify> Observer for NotifyingProducer<P, N> {
    type Item = P::Item;

    fn capacity(&self) -> std::num::NonZeroUsize {
        self.base.capacity()
    }

    fn read_index(&self) -> usize {
        self.base.read_index()
    }

    fn write_index(&self) -> usize {
        self.base.write_index()
    }

    unsafe fn unsafe_slices(&self, start: usize, end: usize) -> (&[std::mem::MaybeUninit<Self::Item>], &[std::mem::MaybeUninit<Self::Item>]) {
        unsafe { self.base.unsafe_slices(start, end) }
    }

    unsafe fn unsafe_slices_mut(&self, start: usize, end: usize) -> (&mut [std::mem::MaybeUninit<Self::Item>], &mut [std::mem::MaybeUninit<Self::Item>]) {
        unsafe { self.base.unsafe_slices_mut(start, end) }
    }

    fn read_is_held(&self) -> bool {
        self.base.read_is_held()
    }

    fn write_is_held(&self) -> bool {
        self.base.write_is_held()
    }
}

impl<P: Producer, N: RbProducerNotify> Producer for NotifyingProducer<P, N> {
    unsafe fn set_write_index(&self, value: usize) {
        unsafe { self.base.set_write_index(value); }
        let _ = self.notifier.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memfd_rb::MemfdRb;
    use ringbuf::traits::{Producer, Consumer};

    #[test]
    fn test_eventfd_notify() {
        let storage = crate::memfd_rb::MemfdStorage::<u32, 1024>::create("test_notify").unwrap();
        let rb = MemfdRb { storage };

        let prod = rb.get_producer();
        let mut cons = rb.get_consumer();
        
        let notifier = EventFdNotifier::new().unwrap();
        let mut notifying_prod = NotifyingProducer::new(prod, notifier);

        notifying_prod.try_push(42).unwrap();
        
        let mut buf = [0u8; 8];
        rustix::io::read(&notifying_prod.notifier.fd, &mut buf).unwrap();
        let val = u64::from_ne_bytes(buf);
        assert_eq!(val, 1);

        assert_eq!(cons.try_pop(), Some(42));
    }
}


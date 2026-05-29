pub mod traits;
pub mod memfd_rb;
pub mod eventfd_notifier;

pub use memfd_rb::{MemfdRb, MemfdStorage, MemfdStorageTrait};
pub use traits::{SharedPod, RbProducerNotify};
pub use eventfd_notifier::EventFdNotifier;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

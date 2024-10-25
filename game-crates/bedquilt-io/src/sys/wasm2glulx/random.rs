use rand_core::{Error, RngCore};
use wasm2glulx_ffi::glulx::random;

/// A random number generator that retrieves randomness from the operating
/// system.
///
/// This is a zero-sized struct. It can be freely constructed with `OsRng`.
///
/// This implementation is provided by Glulx's `random` intrinsic.
#[derive(Debug, Copy, Clone, Default)]
pub struct OsRng;

impl RngCore for OsRng {
    fn next_u32(&mut self) -> u32 {
        unsafe { random(0) as u32 }
    }

    fn next_u64(&mut self) -> u64 {
        let hi = self.next_u32();
        let lo = self.next_u32();

        u64::from(hi) << 32 | u64::from(lo)
    }

    fn fill_bytes(&mut self, mut dest: &mut [u8]) {
        while dest.len() >= 4 {
            let rand_bytes = self.next_u32().to_be_bytes();
            dest[0] = rand_bytes[0];
            dest[1] = rand_bytes[1];
            dest[2] = rand_bytes[2];
            dest[3] = rand_bytes[3];
            dest = &mut dest[4..];
        }
        if dest.is_empty() {
            return;
        }

        let rand_bytes = self.next_u32().to_be_bytes();
        dest[0] = rand_bytes[0];
        if dest.len() > 1 {
            dest[1] = rand_bytes[1];
            if dest.len() > 2 {
                dest[2] = rand_bytes[2];
            }
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

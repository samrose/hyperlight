use tracing::{instrument, Span};

use super::shared_mem::SharedMemory;
use crate::Result;

/// A wrapper around a `SharedMemory` reference and a snapshot
/// of the memory therein
#[derive(Clone)]
pub(super) struct SharedMemorySnapshot {
    snapshot: Vec<u8>,
    shared_mem: SharedMemory,
}

impl SharedMemorySnapshot {
    /// Take a snapshot of the memory in `shared_mem`, then create a new
    /// instance of `Self` with the snapshot stored therein.
    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    pub(super) fn new(shared_mem: SharedMemory) -> Result<Self> {
        // TODO: Track dirty pages instead of copying entire memory
        let snapshot = shared_mem.copy_all_to_vec()?;
        Ok(Self {
            shared_mem,
            snapshot,
        })
    }

    /// Take another snapshot of the internally-stored `SharedMemory`,
    /// then store it internally.
    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    pub(super) fn replace_snapshot(&mut self) -> Result<()> {
        let new_snapshot = self.shared_mem.copy_all_to_vec()?;
        self.snapshot = new_snapshot;
        Ok(())
    }

    /// Copy the memory from the internally-stored memory snapshot
    /// into the internally-stored `SharedMemory`
    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    pub(super) fn restore_from_snapshot(&mut self) -> Result<()> {
        self.shared_mem.copy_from_slice(self.snapshot.as_slice(), 0)
    }
}

#[cfg(test)]
mod tests {
    use hyperlight_common::mem::PAGE_SIZE_USIZE;

    use crate::mem::shared_mem::SharedMemory;

    #[test]
    fn restore_replace() {
        let mut data1 = vec![b'a', b'b', b'c'];
        data1.resize_with(PAGE_SIZE_USIZE, || 0);
        let data2 = data1.iter().map(|b| b + 1).collect::<Vec<u8>>();
        let mut gm = SharedMemory::new(PAGE_SIZE_USIZE).unwrap();
        gm.copy_from_slice(data1.as_slice(), 0).unwrap();
        let mut snap = super::SharedMemorySnapshot::new(gm.clone()).unwrap();
        {
            // after the first snapshot is taken, make sure gm has the equivalent
            // of data1
            assert_eq!(data1, gm.copy_all_to_vec().unwrap());
        }

        {
            // modify gm with data2 rather than data1 and restore from
            // snapshot. we should have the equivalent of data1 again
            gm.copy_from_slice(data2.as_slice(), 0).unwrap();
            assert_eq!(data2, gm.copy_all_to_vec().unwrap());
            snap.restore_from_snapshot().unwrap();
            assert_eq!(data1, gm.copy_all_to_vec().unwrap());
        }
        {
            // modify gm with data2, then retake the snapshot and restore
            // from the new snapshot. we should have the equivalent of data2
            gm.copy_from_slice(data2.as_slice(), 0).unwrap();
            assert_eq!(data2, gm.copy_all_to_vec().unwrap());
            snap.replace_snapshot().unwrap();
            assert_eq!(data2, gm.copy_all_to_vec().unwrap());
            snap.restore_from_snapshot().unwrap();
            assert_eq!(data2, gm.copy_all_to_vec().unwrap());
        }
    }
}

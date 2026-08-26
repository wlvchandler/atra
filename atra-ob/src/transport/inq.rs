use std::sync::atomic::{AtomicU64, Ordering};
use std::fs::{OpenOptions, File};
use std::io::{Error, ErrorKind, Result};
use memmap2::{MmapOptions, MmapMut};
use std::marker::PhantomData;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub type InstrumentId = u32;

const CACHE_LINE_SIZE: usize = 64;

// u64s first - IntoBytes forbids implicit padding, so keep it to the one explicit tail pad
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(8))]
pub struct Order {
    pub id: u64,
    pub price: u64, //e.g. price*10^8
    pub qty: u64,
    pub timestamp: u64,
    pub instrument_id: InstrumentId,
    pub side: u8,  // 0: bid  1: ask
    pub order_type: u8, // 0: limit  1: market  ... etc (todo: define elsewhere)
    pub _pad: [u8; 2],
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
#[repr(C, align(64))] // align() takes a literal only; = CACHE_LINE_SIZE
struct InqHeader { // metadata
    write_idx: AtomicU64,
    _pad0: [u8; CACHE_LINE_SIZE - 8],
    read_idx: AtomicU64,
    _pad1: [u8; CACHE_LINE_SIZE - 8],
    mask: u64,
    created_at: u64,
    _pad3: [u8; CACHE_LINE_SIZE - 16],
}

pub struct InstrumentQueue {
    mapped: MmapMut,
    header_offset: usize,
    data_offset: usize,
    slot_size: usize,
    _phantom: PhantomData<u8>, // ensure we dont send/sync automatically
}


impl InstrumentQueue {
    pub fn new(instrument: InstrumentId, capacity: usize) -> Result<Self> {
	let capacity = capacity.next_power_of_two();

	let slot_size = std::mem::size_of::<Order>();
	let header_size = std::mem::size_of::<InqHeader>();
	let total_size = header_size + (slot_size * capacity);

	let path = format!("/dev/shm/inq_{}", instrument);
	let file = Self::create_and_size_file(&path, total_size as u64)?;
	// SAFETY: nobody shrinks or reinits the file while mapped; shared bytes only touched via atomics or the ring protocol
	let mut mapped = unsafe { MmapOptions::new().map_mut(&file)? };

	let (header, _) = InqHeader::mut_from_prefix(&mut mapped[..])
	    .map_err(|_| Error::new(ErrorKind::InvalidData, "mapping smaller than INQ header"))?;
	header.write_idx = AtomicU64::new(0);
	header.read_idx = AtomicU64::new(0);
	header.mask = (capacity - 1) as u64;
	header.created_at = std::time::SystemTime::now()
	    .duration_since(std::time::UNIX_EPOCH)
	    .unwrap()
	    .as_nanos() as u64;

	Ok(Self {
	    mapped,
	    header_offset: 0,
	    data_offset: header_size,
	    slot_size,
	    _phantom: PhantomData,
	})
    }

    pub fn connect(instrument_id: InstrumentId) -> Result<Self> {
	let path = format!("/dev/shm/inq_{}", instrument_id);
	let file = OpenOptions::new()
	    .read(true)
	    .write(true)
	    .open(&path)?;
	let file_size = file.metadata()?.len() as usize;
	let header_size = std::mem::size_of::<InqHeader>();
	if file_size <= header_size {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid INQ file size"));
        }

	// SAFETY: same contract as new
        let mapped = unsafe { MmapOptions::new().map_mut(&file)? };
        Ok(Self {
            mapped,
            header_offset: 0,
	    data_offset: header_size,
            slot_size: std::mem::size_of::<Order>(),
            _phantom: PhantomData,
        })
    }

    pub fn write(&mut self, order: &Order) -> Result<()> {
	let header = self.header();

	let write_idx = header.write_idx.load(Ordering::Acquire);
	let next_write_idx = write_idx.wrapping_add(1);

	let read_idx = header.read_idx.load(Ordering::Acquire);
	if next_write_idx.wrapping_sub(read_idx) > header.mask {
            return Err(Error::new(ErrorKind::Other, "Queue is full"));
	}

	let slot = (write_idx & header.mask) as usize;
	let offset = self.data_offset + (slot * self.slot_size);

	self.mapped[offset..offset + self.slot_size].copy_from_slice(order.as_bytes());

	std::sync::atomic::fence(Ordering::Release);
	self.header().write_idx.store(next_write_idx, Ordering::Release);
	Ok(())
    }

    pub fn read(&self) -> Option<Order> {
        let header = self.header();

        // acquire ordering to see writes from other processes...
	// confusing naming huh - order this, order that, why don't you order some---
        let read_idx = header.read_idx.load(Ordering::Acquire);
        let write_idx = header.write_idx.load(Ordering::Acquire);
        if read_idx == write_idx {
            return None; // empty
        }

	let slot = (read_idx & header.mask) as usize;
        let offset = self.data_offset + (slot * self.slot_size);

        let order = Order::read_from_bytes(&self.mapped[offset..offset + self.slot_size])
            .expect("slot_size equals size_of::<Order>()");

        std::sync::atomic::fence(Ordering::Acquire);

        header.read_idx.store(read_idx.wrapping_add(1), Ordering::Release);

        Some(order)
    }


    // TODO: read_batch
    //

    fn create_and_size_file(path: &str, size: u64) -> Result<File> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        file.set_len(size)?;
        Ok(file)
    }

    fn header(&self) -> &InqHeader {
	// SAFETY: mapping is page-aligned and at least header-sized, every field is valid at any bit pattern, no &mut InqHeader is ever formed, non-atomic fields written once by the creator before peers connect
	unsafe {
	    &*(self.mapped.as_ptr().add(self.header_offset) as *const InqHeader)
	}
    }


    pub fn depth(&self) -> u64 {
        let header = self.header();
        let read_idx = header.read_idx.load(Ordering::Relaxed);
        let write_idx = header.write_idx.load(Ordering::Relaxed);
        write_idx.saturating_sub(read_idx)
    }

}


impl Drop for InstrumentQueue {
    fn drop(&mut self) {
	// todo: clean-up code if needed (though mmap should automatically unwrap when dropped)
    }
}



pub fn create_instrument_queues(instrument_ids: &[InstrumentId], capacity: usize) -> Result<Vec<InstrumentQueue>> {
    instrument_ids.iter().map(|&id| InstrumentQueue::new(id, capacity)).collect()
}

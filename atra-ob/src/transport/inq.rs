use std::sync::atomic::{AtomicU64, Ordering};
use std::fs::{OpenOptions, File};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use memmap2::{MmapOptions, MmapMut};

pub type InstrumentId = u32;

const CACHE_LINE_SIZE: usize = 64;

#[repr(C, align(8))]
pub struct Order {
    pub id: u64,
    pub instrument_id: InstrumentId,
    pub price: u64, //e.g. price*10^8
    pub qty: u64,
    pub side: u8,  // 0: bid  1: ask
    pub order_type: u8, // 0: limit  1: market  ... etc (todo: define elsewhere)
    pub timestamp: u64,
}

#[repr(C, align(CACHE_LINE_SIZE))]
struct InqHeader { // metadata for instrument queue
    write_idx: AtomicU64,
    _pad0: [u8; CACHE_LINE_SIZE - 8],
    read_idx: AtomicU64,
    _pad1: [u8; CACHE_LINE_SIZE - 8],
    created_at: u64,
    _pad2: [u8; CACHE_LINE_SIZE - 8],
}

pub struct InstrumentQueue {
    mapped: MmapMut,
    instrument_id: InstrumentId,
    slot_count: usize,
    slot_size: usize,
    header_offset: usize,
    data_offset: usize,
}


impl InstrumentQueue {
    pub fn new(instrument: InstrumentId, capacity: usize) -> Result<Self> {
	let slot_size = std::mem::size_of::<Order>();
	let aligned_slot_size = (slot_size + 7) & !7; // 8 byte alignment
	let header_size = std::mem::size_of::<InqHeader>();
	let total_size = header_size + (aligned_slot_size * capacity);

	let path = format!("/dev/shm/inq_{}", instrument_id);
	let file = Self::create_and_size_file(&path, total_size as u64)?;
	let mapped = unsafe { MmapOptions::new().map_mut(&file)?);

	unsafe {
	    let header = mapped.as_ptr() as *mut InqHeader;
	    (*header).write_idx = AtomicU64::new(0);
	    (*header).read_idx = AtomicU64::new(0);
	    (*header).created_at = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_nanos() as u64;
	}

	Ok(Self {
	    mapped, instrument_id,
	    slot_count: capacity,
	    slot_size: aligned_slot_size,
	    header_offset: 0,
	    data_offset: header_size,
	})
    }

    pub fn write(&self, order: &Order) -> Result<()> {
	// ...
    }

    pub fn read(&self) -> Option<Order> {
	// ...
    }
}


impl Drop for InstrumentQueue {
    fn drop(&mut self) {
	// todo: clean-up code (though mmap should automatically unwrap when dropped)
    }
}




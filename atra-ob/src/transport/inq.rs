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

    pub fn connect(instrument_id: InstrumentId) -> Result<Self> {
	let path = format!("/dev/shm/inq_{}", instrument_id);
	let file = OpenOptions::new()
	    .read(true)
	    .write(true)
	    .open(&path)?;
	let file_size = file.metadata()?.len() as usize;
	let header_size = std::mem::size_of<InqHeader>();
	if file_size <= header_size {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid INQ file size"));
        }

        let mapped = unsafe { MmapOptions::new().map_mut(&file)? };
        let slot_size = std::mem::size_of::<Order>();
        let aligned_slot_size = (slot_size + 7) & !7;
        let slot_count = (file_size - header_size) / aligned_slot_size;
        
        Ok(Self {
            mapped,instrument_id, slot_count,
            slot_size: aligned_slot_size,
            header_offset: 0,
            data_offset: header_size,
        })
    }

    // returns slot index used
    pub fn write(&self, order: &Order) -> Result<u64> {
        if order.instrument_id != self.instrument_id {
            return Err(Error::new(ErrorKind::InvalidInput, "Instrument ID mismatch"));
        }
        
        let header = unsafe { &*(self.mapped.as_ptr() as *const InqHeader) };
        
        // next write position [atomic]
	// acquire ordering ensures we see updates from other threads/processes
        let write_pos = header.write_idx.fetch_add(1, Ordering::AcqRel);
        let slot_idx = write_pos % self.slot_count as u64;
        let slot_pos = self.data_offset + (slot_idx as usize * self.slot_size);
        
        // check if buffer is full (if writer wraps around it will overwrite unread data)
        let read_pos = header.read_idx.load(Ordering::Acquire);
        if write_pos >= read_pos + self.slot_count as u64 {
            return Err(Error::new(ErrorKind::Other, "Queue is full"));
        }
        

        unsafe {
            let slot_ptr = self.mapped.as_ptr().add(slot_pos) as *mut Order;
            std::ptr::copy_nonoverlapping(order as *const Order, slot_ptr, 1);
	    
	    // need volatile write for cross-process visibility
            std::sync::atomic::fence(Ordering::Release);
        }
        
        Ok(slot_idx)
    }

    pub fn read(&self) -> Option<Order> {
        let header = unsafe { &*(self.mapped.as_ptr() as *const InqHeader) };
        
        // acquire ordering to see writes from other processes...
	// confusing naming huh - order this, order that, why don't you order some---
        let read_idx = header.read_idx.load(Ordering::Acquire);
        let write_idx = header.write_idx.load(Ordering::Acquire);
        
        if read_idx >= write_idx {
            return None; // empty
        }
        
        
        let slot_idx = read_idx % self.slot_count as u64;
        let slot_pos = self.data_offset + (slot_idx as usize * self.slot_size);
        
        let mut order = Order {
            id: 0,
            instrument_id: 0,
            price: 0,
            quantity: 0,
            side: 0,
            order_type: 0,
            timestamp: 0,
        };
        
        unsafe {
            let slot_ptr = self.mapped.as_ptr().add(slot_pos) as *const Order;
            std::ptr::copy_nonoverlapping(slot_ptr, &mut order as *mut Order, 1);
            
            //  ensure we've read the most recent data
            std::sync::atomic::fence(Ordering::Acquire);
        }
        
	
        header.read_idx.fetch_add(1, Ordering::Release);
        
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


    pub fn depth(&self) -> u64 {
        let header = unsafe { &*(self.mapped.as_ptr() as *const InqHeader) };
        let read_idx = header.read_idx.load(Ordering::Relaxed);
        let write_idx = header.write_idx.load(Ordering::Relaxed);
        write_idx.saturating_sub(read_idx)
    }
    
    pub fn capacity(&self) -> usize {
        self.slot_count
    }
    
    pub fn instrument_id(&self) -> InstrumentId {
        self.instrument_id
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

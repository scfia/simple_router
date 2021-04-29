use crate::memory_handle::MemoryHandle;
use crate::virtqueue_network::NetworkDescriptor;
use crate::virtqueue_network::RawVirtioNetHeaderShortPointer;
use core::sync::atomic;
use core::sync::atomic::Ordering;
use core::slice;

const MMIO_QUEUE_ALIGN: usize = 4095;
const BUFFER_SIZE: u32 = 4096;

#[derive(Debug)]
pub struct VirtQueueElement {
    desc: RawVirtQueueDescriptorPointer,
    pub desc_idx: u16,
}

#[derive(Debug)]
pub struct VirtQueueHandle {
    base_address: usize,
    queue_size: usize,
    last_seen_used_ring_idx: u16,
    descriptor_table: usize,
    available_ring: AvailableRingHandle,
    used_ring: UsedRingHandle,
}

#[derive(Debug)]
pub struct AvailableRingHandle {
    queue_size: usize,
    flags: *mut u16,
    idx: *mut u16,
    ring: *mut u16,
}

#[derive(Debug)]
pub struct UsedRingHandle {
    queue_size: usize,
    flags: *const u16,
    idx: *const u16,
    ring: usize,
    last_seen_idx: u16,
}

/// A pointer to an entry in the descriptor table.
/// Always points to a 16 byte aligned location with the following fields:
///     addr: u64
///     len: u32
///     flags: u16
///     next: u16
#[derive(Clone, Copy)]
pub struct RawVirtQueueDescriptorPointer {
    ptr: usize,
}

impl RawVirtQueueDescriptorPointer {
    pub fn get_addr(&self) -> u64 {
        unsafe { ((self.ptr + 0) as *const u64).read_volatile() }
    }

    pub fn get_len(&self) -> u32 {
        unsafe { ((self.ptr + 8) as *const u32).read_volatile() }
    }

    pub fn set_addr(&mut self, addr: u64) {
        unsafe { ((self.ptr + 0) as *mut u64).write_volatile(addr) }
    }

    pub fn set_len(&mut self, len: u32) {
        unsafe { ((self.ptr + 8) as *mut u32).write_volatile(len) }
    }

    pub fn set_flags(&mut self, flags: u16) {
        unsafe { ((self.ptr + 12) as *mut u16).write_volatile(flags) }
    }

    pub fn set_next(&mut self, next: u16) {
        unsafe { ((self.ptr + 14) as *mut u16).write_volatile(next) }
    }
}

#[repr(packed)]
/// A pointer to an entry in the used ring.
/// Always points to an 4 byte aligned location with the following fields:
///     id: u32
///     len: u32
pub struct RawVirtQueueUsedElementPointer {
    ptr: usize
}

impl RawVirtQueueUsedElementPointer {
    pub fn get_id(&self) -> u32 {
        unsafe { ((self.ptr + 0) as *const u32).read_volatile() }
    }
}

impl VirtQueueElement {
    #[inline(never)]
    pub fn as_network_packet(&self) -> (RawVirtioNetHeaderShortPointer, &[u8]) {
        self.desc.as_network_packet()
    }

    #[inline(never)]
    pub fn copy_from(&self, source: &VirtQueueElement) {
        let dest_desc: &mut [u8] = unsafe { slice::from_raw_parts_mut(self.desc.get_addr() as _, self.desc.get_len() as usize) };
        let src_desc: &[u8] = unsafe { slice::from_raw_parts_mut(source.desc.get_addr() as _, self.desc.get_len() as usize) };
        dest_desc.copy_from_slice(src_desc);
    }
}

impl VirtQueueHandle {
    #[inline(never)]
    pub fn new(queue_size: usize, memory: &mut MemoryHandle, receive: bool) -> Self {
        let total_size = virtqueue_size(queue_size as usize, MMIO_QUEUE_ALIGN as usize);
        let virtqueue_address = memory.allocate(total_size, 16).unwrap();
        let mut virtqueue = VirtQueueHandle {
            base_address: virtqueue_address,
            queue_size: queue_size,
            last_seen_used_ring_idx: 0,
            descriptor_table: virtqueue_address,
            available_ring: AvailableRingHandle::from_address(
                virtqueue_address + available_ring_offset(queue_size),
                queue_size,
            ),
            used_ring: UsedRingHandle::from_address(
                virtqueue_address + used_ring_offset(queue_size, MMIO_QUEUE_ALIGN),
                queue_size,
            ),
        };

        for i in 0..queue_size {
            let descriptor_address = memory.allocate(BUFFER_SIZE as usize, 16).unwrap() as u64;
            let flags = if receive { 2 } else { 0 };
            virtqueue.update_descriptor(i as u16, descriptor_address, BUFFER_SIZE, flags, 0);
        }
        for i in 0..1024 {
            virtqueue.offer(i as u16);
        }

        atomic::fence(Ordering::AcqRel);
        virtqueue
    }

    #[inline(never)]
    pub fn try_take(&mut self) -> Option<VirtQueueElement> {
        atomic::fence(Ordering::AcqRel);
        if let Some(descriptor_idx) = self.used_ring.try_remove() {
            let desc_ptr = self.get_descriptor(descriptor_idx);
            Some(VirtQueueElement {
                desc: desc_ptr,
                desc_idx: descriptor_idx
            })
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn offer(&mut self, desc_idx: u16) {
        atomic::fence(Ordering::AcqRel);
        self.available_ring.advance(desc_idx);
    }

    fn get_descriptor(&mut self, descriptor_idx: u16) -> RawVirtQueueDescriptorPointer {
        RawVirtQueueDescriptorPointer {
            ptr: self.descriptor_table + (descriptor_idx as usize * 16) //TODO!
        }
    }

    #[inline(never)]
    fn update_descriptor(&mut self, descriptor_idx: u16, addr: u64, len: u32, flags: u16, next: u16) {
        let mut descriptor_ptr = self.get_descriptor(descriptor_idx);
        descriptor_ptr.set_addr(addr);
        descriptor_ptr.set_len(len);
        descriptor_ptr.set_flags(flags);
        descriptor_ptr.set_next(next);
    }

    pub fn base_address(&self) -> usize {
        self.base_address
    }
}

impl AvailableRingHandle {
    pub fn from_address(address: usize, queue_size: usize) -> AvailableRingHandle {
        AvailableRingHandle {
            queue_size: queue_size,
            flags: address as *mut u16,
            idx: (address + 2) as *mut u16,
            ring: (address + 4) as *mut u16,
        }
    }

    pub fn idx(&self) -> u16 {
        unsafe { self.idx.read_volatile() }
    }

    #[inline(never)]
    pub fn advance(&mut self, descriptor_idx: u16) {
        unsafe {
            // write to the current head
            self.ring
                .offset((self.idx() % (self.queue_size as u16)) as isize)
                .write_volatile(descriptor_idx);
            atomic::fence(Ordering::AcqRel);

            // advance head
            self.idx.write_volatile(self.idx().wrapping_add(1));
            atomic::fence(Ordering::AcqRel);
        }
    }
}

impl UsedRingHandle {
    pub fn from_address(address: usize, queue_size: usize) -> UsedRingHandle {
        UsedRingHandle {
            queue_size: queue_size,
            flags: address as *mut u16,
            idx: (address + 2) as *mut u16,
            ring: (address + 4),
            last_seen_idx: 0,
        }
    }

    #[inline(never)]
    pub fn try_remove(&mut self) -> Option<u16> {
        let used_idx = unsafe { self.idx.read_volatile() };
        if self.last_seen_idx != used_idx {
            let used_element_ptr = RawVirtQueueUsedElementPointer {
                ptr: self.ring + ((self.last_seen_idx % self.queue_size as u16) as usize * 8)
            };
            let used_element_id = used_element_ptr.get_id();
            self.last_seen_idx = self.last_seen_idx.wrapping_add(1);
            Some(used_element_id as u16 % self.queue_size as u16)
        } else {
            None
        }
    }
}

impl RawVirtQueueDescriptorPointer {
    pub fn data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.get_addr() as _, self.get_len() as usize) }
    }
}

fn align(x: usize, queue_align: usize) -> usize {
    ((x) + queue_align) & !queue_align
}

fn available_ring_offset(queue_size: usize) -> usize {
    0x10 * queue_size
}

fn used_ring_offset(queue_size: usize, queue_align: usize) -> usize {
    align(
        0x10 * queue_size                         // descriptor table
        + 2 * (3 + queue_size),
        queue_align,
    ) // available ring
}

fn virtqueue_size(queue_size: usize, queue_align: usize) -> usize {
    align(0x10 * queue_size                         // descriptor table
            + 2 * (3 + queue_size), queue_align)    // available ring
        + align(2 * 3                               // used ring
            + 8 * queue_size, queue_align) // used ring's elements
}

impl ::core::fmt::Debug for RawVirtQueueDescriptorPointer {
    #[inline(never)]
    fn fmt(&self, _f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        unimplemented!()
        /*
        let addr = self.addr;
        let len = self.len;
        let flags = self.flags;
        let next = self.next;
        write!(
            f,
            "RawVirtQueueDescriptor {{ addr: 0x{:x}, len: 0x{:x}, flags: 0x{:x}, next: 0x{:x} }}",
            addr, len, flags, next
        )
        */
    }
}

impl ::core::fmt::Debug for RawVirtQueueUsedElementPointer {
    #[inline(never)]
    fn fmt(&self, _f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        unimplemented!();
        /*
        let addr = self.id;
        let len = self.len;
        write!(
            f,
            "RawVirtQueueUsedElement {{ addr: 0x{:x}, len: 0x{:x} }}",
            addr, len
        )
        */
    }
}

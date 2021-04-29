use crate::virtqueue::RawVirtQueueDescriptorPointer;

#[repr(C, packed)]
struct RawVirtioNetHeaderShort {
    flags: u8,
    gso_type: u8,
    hdr_len: u16,
    gso_size: u16,
    csum_start: u16,
    csum_offset: u16, // no num_buffers for the short variant
}

#[derive(Clone, Copy, Debug)]
pub struct RawVirtioNetHeaderShortPointer {
    address: u64,
}

impl RawVirtioNetHeaderShortPointer {
    // TODO getters for fields
}

pub trait NetworkDescriptor {
    fn as_network_packet(&self) -> (RawVirtioNetHeaderShortPointer, &[u8]);
}

impl NetworkDescriptor for RawVirtQueueDescriptorPointer {
    #[inline(never)]
    fn as_network_packet(&self) -> (RawVirtioNetHeaderShortPointer, &[u8]) {
        let data = self.data();
        let header_size = core::mem::size_of::<RawVirtioNetHeaderShort>();
        let (_header_bytes, data_bytes) = data.split_at(header_size);
        let header = RawVirtioNetHeaderShortPointer {
            address: self.get_addr()
        };
        (header, data_bytes)
    }
}

impl ::core::fmt::Debug for RawVirtioNetHeaderShort {
    #[inline(never)]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let flags = self.flags;
        let gso_type = self.gso_type;
        let hdr_len = self.hdr_len;
        let gso_size = self.gso_size;
        let csum_start = self.csum_start;
        let csum_offset = self.csum_offset;
        write!(f, "RawVirtioNetHeaderShort {{ flags: 0x{:x}, gso_type: 0x{:x}, hdr_len: 0x{:x}, gso_size: 0x{:x}, csum_start: 0x{:x}, csum_offset: 0x{:x} }}", flags, gso_type, hdr_len, gso_size, csum_start, csum_offset)
    }
}

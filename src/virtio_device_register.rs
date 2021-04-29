use core::ops;
use register::{mmio::*, register_bitfields, register_structs};

pub struct VirtioMMIORegister {
    base_address: usize,
}

impl VirtioMMIORegister {
    pub fn new(base_address: usize) -> Self {
        VirtioMMIORegister { base_address }
    }

    fn ptr(&self) -> *const LegacyVirtioDeviceRegister {
        self.base_address as *const _
    }
}

impl ops::Deref for VirtioMMIORegister {
    type Target = LegacyVirtioDeviceRegister;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

register_bitfields! {
    u32,
    pub DeviceStatus [
        ACKNOWLEDGE OFFSET(0) NUMBITS(1) [],
        DRIVER OFFSET(1) NUMBITS(1) [],
        DRIVER_OK OFFSET(2) NUMBITS(1) [],
        FEATURES_OK OFFSET(3) NUMBITS(1) [],

        //FOO OFFSET(4) NUMBITS(4) [],
        //FOO OFFSET(5) NUMBITS(5) [],
        DEVICE_NEEDS_RESET OFFSET(6) NUMBITS(1) [],
        FAILED OFFSET(7) NUMBITS(1) []
    ],
    pub NetworkDeviceFeatureBits0 [
        VIRTIO_NET_F_CSUM OFFSET(0) NUMBITS(1) [],
        VIRTIO_NET_F_GUEST_CSUM OFFSET(1) NUMBITS(1) [],
        VIRTIO_NET_F_CTRL_GUEST_OFFLOADS OFFSET(2) NUMBITS(1) [],
        VIRTIO_NET_F_MTU OFFSET(3) NUMBITS(1) [],

        //FOO OFFSET(4) NUMBITS(1) [],
        VIRTIO_NET_F_MAC OFFSET(5) NUMBITS(1) [],
        VIRTIO_NET_F_GSO OFFSET(6) NUMBITS(1) [],
        VIRTIO_NET_F_GUEST_TSO4 OFFSET(7) NUMBITS(1) [],

        VIRTIO_NET_F_GUEST_TSO6 OFFSET(8) NUMBITS(1) [],
        VIRTIO_NET_F_GUEST_ECN OFFSET(9) NUMBITS(1) [],
        VIRTIO_NET_F_GUEST_UFO OFFSET(10) NUMBITS(1) [],
        VIRTIO_NET_F_HOST_TSO4 OFFSET(11) NUMBITS(1) [],

        VIRTIO_NET_F_HOST_TSO6 OFFSET(12) NUMBITS(1) [],
        VIRTIO_NET_F_HOST_ECN OFFSET(13) NUMBITS(1) [],
        VIRTIO_NET_F_HOST_UFO OFFSET(14) NUMBITS(1) [],
        VIRTIO_NET_F_MGR_RXBUF OFFSET(15) NUMBITS(1) [],

        VIRTIO_NET_F_STATUS OFFSET(16) NUMBITS(1) [],
        VIRTIO_NET_F_CTRL_VQ OFFSET(17) NUMBITS(1) [],
        VIRTIO_NET_F_CTRL_RX OFFSET(18) NUMBITS(1) [],
        VIRTIO_NET_F_CTRL_VLAN OFFSET(19) NUMBITS(1) [],

        WTF_IS_THIS_SHIT OFFSET(20) NUMBITS(1) [],
        VIRTIO_NET_F_GUEST_ANNOUNCE OFFSET(21) NUMBITS(1) [],
        VIRTIO_NET_F_MQ OFFSET(22) NUMBITS(1) [],
        VIRTIO_NET_F_CTRL_MAC_ADDR OFFSET(23) NUMBITS(1) [],

        VIRTIO_F_NOTIFY_ON_EMPTY OFFSET(24) NUMBITS(1) [],
        //FOO OFFSET(25) NUMBITS(1) [],
        //FOO OFFSET(26) NUMBITS(1) [],
        VIRTIO_F_ANY_LAYOUT OFFSET(27) NUMBITS(1) [],

        VIRTIO_F_RING_INDIRECT_DESC OFFSET(25) NUMBITS(1) [],
        VIRTIO_F_RING_EVENT_IDX OFFSET(26) NUMBITS(1) [],
        UNUSED OFFSET(25) NUMBITS(1) []
        //FOO OFFSET(26) NUMBITS(1) [],
    ]
}

register_structs! {
    pub LegacyVirtioDeviceRegister {
        (0x000 => pub magic_value: ReadOnly<u32>),
        (0x004 => pub version: ReadOnly<u32>),
        (0x008 => pub device_id: ReadOnly<u32>),
        (0x00c => pub vendor_id: ReadOnly<u32>),
        (0x010 => pub host_features: ReadOnly<u32, NetworkDeviceFeatureBits0::Register>),
        (0x014 => pub host_features_sel: WriteOnly<u32>),
        (0x018 => _reserved1),
        (0x020 => pub guest_features: WriteOnly<u32, NetworkDeviceFeatureBits0::Register>),
        (0x024 => pub guest_features_sel: WriteOnly<u32>),
        (0x028 => pub guest_page_size: WriteOnly<u32>),
        (0x02C => _reserved2),
        (0x030 => pub queue_sel: WriteOnly<u32>),
        (0x034 => pub queue_num_max: ReadOnly<u32>),
        (0x038 => pub queue_num: WriteOnly<u32>),
        (0x03C => pub queue_align: WriteOnly<u32>),
        (0x040 => pub queue_pfn: ReadWrite<u32>),
        (0x044 => _reserved3),
        (0x050 => pub queue_notify: WriteOnly<u32>),
        (0x054 => _reserved4),
        (0x060 => pub interrupt_status: ReadOnly<u32>),
        (0x064 => pub interrupt_ack: WriteOnly<u32>),
        (0x068 => _reserved5),
        (0x070 => pub device_status: ReadWrite<u32, DeviceStatus::Register>),
        (0x074 => _reserved6),
        (0x100 => @END),
    }
}

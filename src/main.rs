#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]

global_asm!(include_str!("start.s"));

extern crate register;

mod errors;
mod memory_handle;
mod util;
mod virtio;
mod virtio_device_register;
mod virtqueue;
mod virtqueue_network;

use core::panic::PanicInfo;

use memory_handle::MemoryHandle;
use virtio::VirtioMMIONetworkDevice;

const ETHERTYPE_IPV4: u16 = 0x0800;
const ETHERTYPE_ARP: u16 = 0x0806;
const ETHERTYPE_IPV6: u16 = 0x86dd;

#[panic_handler]
fn handle_panic(_panic_info: &PanicInfo) -> ! {
    // let _ = util::print(format_args!("Panic! {:?}\n", panic_info));
    loop {}
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut memory = MemoryHandle::new(0x46000000, 0x1000000);
    let mut ingress_nic = VirtioMMIONetworkDevice::initialize(0x000000000a003e00, &mut memory).unwrap();
    let mut egress_nic = VirtioMMIONetworkDevice::initialize(0x000000000a003c00, &mut memory).unwrap();
    let ingress_recv_rq = &mut ingress_nic.receiveq1;
    let egress_send_rq = &mut egress_nic.sendq1;
    egress_nic.register.queue_notify.set(1);
    //let mut counter: u64 = 0;
    loop {
        if let Some(queue_element) = ingress_recv_rq.try_take() {
            //util::print(format_args!("handling packet {:x}\n", counter)).unwrap();
            //counter += 1;
            let (_header, data) = queue_element.as_network_packet();
            //// util::print(format_args!("data = {:x?} \n", &_data[0..128])).unwrap();
            // data contains the entire ethernet frame:
            // 6 bytes MAC destination
            // 6 bytes MAC source
            // (802.1Q tag, optional, so no)
            // 2 bytes ethertype
            // payload!
                // ipv4:
                // 1 byte version and IHL
                // 1 byte DSCP and ECN
                // 2 bytes total length
                // 2 bytes identification
                // 2 bytes flags and fragment offset
                // 1 byte TTL
                // 1 byte protocol
                // 2 bytes header checksum
                // 4 bytes source address
                // 4 bytes destination address
                // ipv6:

            let mut ethertype_bytes: [u8; 2] = [0; 2];
            ethertype_bytes.clone_from_slice(&data[12..14]);
            let ethertype = u16::from_be_bytes(ethertype_bytes);
            if ethertype == ETHERTYPE_IPV4 {
                let _source: &[u8] = &data[14+12..14+12+4];
                let _destination: &[u8] = &data[14+16..14+16+4];
                // util::print(format_args!("### ipv4 source = {:?}, destination = {:?} \n", source, destination)).unwrap();
                if let Some(egress_queue_element) = egress_send_rq.try_take() {
                    // util::print(format_args!("############# passing packet to sendqueue\n"));
                    egress_queue_element.copy_from(&queue_element);
                    egress_send_rq.offer(egress_queue_element.desc_idx);
                    egress_nic.register.queue_notify.set(1);
                } else {
                    // util::print(format_args!("[warn] egress send queue full\n")).unwrap();
                }
            } else if ethertype == ETHERTYPE_IPV6 {
                // util::print(format_args!("[warn] ipv6 packet\n")).unwrap();
            } else if ethertype == ETHERTYPE_ARP {
                // util::print(format_args!("[warn] arp packet\n")).unwrap();
            } else {
                // util::print(format_args!("[warn] unknown ethertype {}\n", ethertype)).unwrap();
            }
            ingress_recv_rq.offer(queue_element.desc_idx);
            ingress_nic.register.queue_notify.set(0);
        }
    }
}

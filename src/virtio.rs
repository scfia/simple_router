use crate::errors::*;
use crate::memory_handle::MemoryHandle;
use crate::virtio_device_register::DeviceStatus;
use crate::virtio_device_register::NetworkDeviceFeatureBits0;
use crate::virtio_device_register::VirtioMMIORegister;
use crate::virtqueue::VirtQueueHandle;

const PAGE_SIZE: u32 = 2048;
const MMIO_QUEUE_ALIGN: u32 = 4095;

pub struct VirtioMMIONetworkDevice {
    pub register: VirtioMMIORegister,
    pub receiveq1: VirtQueueHandle,
    pub sendq1: VirtQueueHandle,
}

impl VirtioMMIONetworkDevice {
    /// Initialize the legacy device according to section 3.1.2 and 4.2.3.1.1
    pub fn initialize(
        address: usize,
        memory: &mut MemoryHandle,
    ) -> Result<VirtioMMIONetworkDevice, DeviceInitializationError> {
        let mut register = VirtioMMIORegister::new(address);
        let _magic_value = register.magic_value.get();
        // util::print(format_args!("magic_value = 0x{:x}\n", magic_value)).unwrap();
        let _version = register.version.get();
        // util::print(format_args!("version = 0x{:x}\n", version)).unwrap();

        // 1. Reset the device
        register.device_status.set(0);
        // 2. Set the ACKNOWLEDGE status bit
        register
            .device_status
            .modify(DeviceStatus::ACKNOWLEDGE.val(1));
        // 3. Set the DRIVER status bit
        register.device_status.modify(DeviceStatus::DRIVER.val(1));

        // 4. Read the device's feature bits and write the understood subset
        register.host_features_sel.set(0);
        let _host_features0 = register.host_features.get();
        // util::print(format_args!("host_features0 = {:?}\n", host_features0)).unwrap();
        register.guest_features_sel.set(0);
        let guest_features = NetworkDeviceFeatureBits0::VIRTIO_NET_F_MAC.val(0);
        register.guest_features.write(guest_features);
        // util::print(format_args!(
        //    "guest_features = {:?}\n",
        //    guest_features.value
        //))
        // .unwrap();

        // 7. Perform device-specific setup (i.e. do virtqueue stuff, see 5.1.2)
        // Write the queue page size to register
        register.guest_page_size.set(PAGE_SIZE);
        // According to section 5.1.2, 0 is receiveq1 and 1 is transmitq1.
        let receiveq1 = Self::configure_virtqueue(0, &mut register, memory, true);
        let sendq1 = Self::configure_virtqueue(1, &mut register, memory, false);

        // 8. Set the DRIVER_OK status bit
        register
            .device_status
            .modify(DeviceStatus::DRIVER_OK.val(1));

        // Check the status again
        let _status = register.device_status.get();
        // util::print(format_args!("status = {:?}\n", status)).unwrap();

        // Notify the device of the available buffer
        // util::print(format_args!("notifying device of queue 0\n")).unwrap();
        register.queue_notify.set(0);
        Ok(VirtioMMIONetworkDevice {
            register,
            receiveq1,
            sendq1,
        })
    }

    fn configure_virtqueue(
        index: u32,
        register: &mut VirtioMMIORegister,
        memory: &mut MemoryHandle,
        receive: bool,
    ) -> VirtQueueHandle {
        if index > 1 {
            panic!("virtqueue might overlap other mem")
        }
        // 1. Select the queue
        register.queue_sel.set(index);

        // 2. Check if the queue is not already in use
        let queue_pfn = register.queue_pfn.get();
        if queue_pfn != 0 {
            panic!("QueuePFN should be 0 - queue is in use!")
        }
        // util::print(format_args!("queue_pfn = {:?}\n", queue_pfn)).unwrap();

        // 3. Read maximum queue size
        let queue_num_max = register.queue_num_max.get();
        if queue_num_max == 0 {
            panic!("QueueNumMax should not be 0 - queue is not available!")
        }
        if queue_num_max < 1024 {
            panic!("QueueNumMax < 1024!");
        }
        let queue_size: u32 = 1024;
        // util::print(format_args!("queue_num_max = {:?}\n", queue_num_max)).unwrap();

        // 4. Allocate and zero queue pages
        let virtqueue = VirtQueueHandle::new(queue_size as usize, memory, receive);

        // 5. Notify the device about the queue size
        register.queue_num.set(queue_size);

        // 6. Notify the device about the used alignment
        register.queue_align.set(MMIO_QUEUE_ALIGN + 1);

        // 7. Write the physical number of the first page of the queue to pfn
        register
            .queue_pfn
            .set((virtqueue.base_address() / PAGE_SIZE as usize) as u32);
        // util::print(format_args!(
        //     "virtqueue at 0x{:x} configured\n",
        //     &virtqueue.base_address()
        // ))
        // .unwrap();
        virtqueue
    }
}

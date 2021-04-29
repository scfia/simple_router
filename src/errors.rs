#[derive(Debug)]
#[allow(dead_code)]
pub enum DeviceInitializationError {
    Internal,
    InvalidMagicNumber(u32),
    InvalidVersion(u32),
    InvalidInterface(ReadMMIOInterfaceError),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ReadMMIOInterfaceError {
    Internal,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MemoryReservationError {
    MemoryExhausted,
}

use core::fmt;

struct UARTLineWriter;

impl fmt::Write for UARTLineWriter {
    fn write_str(&mut self, data: &str) -> Result<(), core::fmt::Error> {
        let buffer = 0x09000000 as *mut [u8; 1024];
        let stdout: &mut [u8] = unsafe { &mut *buffer };
        let bytes = data.as_bytes();
        let mut p = 0;
        for b in bytes {
            p %= stdout.len();
            stdout[p] = *b;
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub fn print(args: fmt::Arguments) -> Result<(), core::fmt::Error> {
    let mut writer = UARTLineWriter;
    fmt::write(&mut writer, args)
}

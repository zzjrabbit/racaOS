use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

static SERIAL_PORT: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    Mutex::new(unsafe {
        let mut port = SerialPort::new(0x3f8);
        port.init();
        port
    })
});

pub(crate) fn send(data: u8) {
    SERIAL_PORT.lock().send(data);
}

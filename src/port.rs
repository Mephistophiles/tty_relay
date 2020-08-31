/*
 * Copyright (C) 2020 Maxim Zhukov <mussitantesmortem@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
/// power management via tty relay
use log::debug;
use serialport::SerialPortType::UsbPort;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

trait ReadWrite: Read + Write {}
impl<T> ReadWrite for T where T: Read + Write {}

/// tty port wrapper
pub struct Port {
    port: Box<dyn ReadWrite>,
    path: PathBuf,
}

enum Action {
    Connect,
    Disconnect,
}

impl Port {
    fn find_tty(vid: u16, pid: u16) -> Option<PathBuf> {
        let ports = serialport::available_ports().unwrap();
        for port in ports {
            if let UsbPort(usb_port) = port.port_type {
                if usb_port.vid == vid && usb_port.pid == pid {
                    return Some(PathBuf::from(port.port_name));
                }
            }
        }

        None
    }

    fn write(&mut self, command: &[u8; 4]) {
        debug!("{}: write {:02X?}", self.path.display(), command);
        self.port.write_all(command).unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    fn control_mode(&mut self) {
        let control_mode = &[0xF0, 0xA0, 0x0C, 0x54];
        self.write(control_mode);
    }

    fn jog_mode(&mut self) {
        let jog_mode = &[0xF0, 0xA0, 0x0C, 0x55];
        self.write(jog_mode);
    }

    fn send_timer(&mut self, timeout: u16) {
        let timeout = timeout.to_ne_bytes();
        let timer = &[0xF0, timeout[1], timeout[0], 0x57];
        self.write(timer);
    }

    fn send_action(&mut self, action: Action) {
        let enable = match action {
            Action::Connect => 0x01,
            Action::Disconnect => 0x00,
        };
        let toggle = &[0xF0, 0xA0, enable, 0x53];
        self.write(toggle);
    }

    fn send_disconnect(&mut self) {
        self.send_action(Action::Disconnect);
    }

    fn send_connect(&mut self) {
        self.send_action(Action::Connect);
    }
}

impl Port {
    const VID: u16 = 6790;
    const PID: u16 = 29987;
    /// open the tty port
    pub fn open(tty_path: Option<&str>) -> Option<Port> {
        let path;

        if let Some(p) = tty_path {
            path = PathBuf::from(p);
        } else {
            path = Port::find_tty(Self::VID, Self::PID)?;
        }

        let port = serialport::open(&path).ok()?;

        Some(Port {
            port: Box::new(port),
            path,
        })
    }

    /// start immediately
    pub fn on(&mut self) {
        self.control_mode();
        self.send_connect();
    }

    /// stop immediately
    pub fn off(&mut self) {
        self.control_mode();
        self.send_disconnect();
    }

    /// start after n seconds
    pub fn timed_on(&mut self, timeout: u16) {
        self.off();
        self.send_timer(timeout);
    }

    /// stop after n seconds
    pub fn timed_off(&mut self, timeout: u16) {
        self.on();
        self.send_timer(timeout);
    }

    /// toggle power
    pub fn toggle(&mut self) {
        self.control_mode();
        self.send_timer(0);
    }

    /// quick toggle power
    pub fn jog(&mut self) {
        self.jog_mode();
        self.send_connect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_stub_port() -> Port {
        let buffer: Vec<u8> = Vec::with_capacity(8);
        let cursor = Cursor::new(buffer);
        let port = Box::new(cursor);
        Port {
            port,
            path: PathBuf::from("stub"),
        }
    }

    fn assert_buf(port: Port, expected: &[u8]) {
        let ptr: Box<Cursor<Vec<u8>>> = unsafe { transmute::transmute(port.port) };
        assert_eq!(ptr.get_ref().as_slice(), expected);
    }

    #[test]
    fn test_control_mode() {
        let mut port = create_stub_port();

        port.control_mode();

        assert_buf(port, &[0xF0, 0xA0, 0x0C, 0x54]);
    }

    #[test]
    fn test_jog_mode() {
        let mut port = create_stub_port();

        port.jog_mode();

        assert_buf(port, &[0xF0, 0xA0, 0x0C, 0x55]);
    }

    #[test]
    fn test_timer() {
        let mut port = create_stub_port();

        port.send_timer(0);

        assert_buf(port, &[0xF0, 0x00, 0x00, 0x57]);

        let mut port = create_stub_port();

        port.send_timer(u16::MAX);

        assert_buf(port, &[0xF0, 0xFF, 0xFF, 0x57]);
    }

    #[test]
    fn test_connect() {
        let mut port = create_stub_port();

        port.send_connect();

        assert_buf(port, &[0xF0, 0xA0, 0x01, 0x53]);
    }

    #[test]
    fn test_disconnect() {
        let mut port = create_stub_port();

        port.send_disconnect();

        assert_buf(port, &[0xF0, 0xA0, 0x00, 0x53]);
    }

    #[test]
    fn test_on() {
        let mut port = create_stub_port();

        port.on();

        assert_buf(port, &[0xF0, 0xA0, 0x0C, 0x54, 0xF0, 0xA0, 0x01, 0x53]);
    }

    #[test]
    fn test_off() {
        let mut port = create_stub_port();

        port.off();

        assert_buf(port, &[0xF0, 0xA0, 0x0C, 0x54, 0xF0, 0xA0, 0x00, 0x53]);
    }

    #[test]
    fn test_timed_on() {
        let mut port = create_stub_port();

        port.timed_on(1);

        assert_buf(
            port,
            &[
                0xF0, 0xA0, 0x0C, 0x54, 0xF0, 0xA0, 0x00, 0x53, 0xF0, 0x00, 0x01, 0x57,
            ],
        );
    }

    #[test]
    fn test_timed_off() {
        let mut port = create_stub_port();

        port.timed_off(1);

        assert_buf(
            port,
            &[
                0xF0, 0xA0, 0x0C, 0x54, 0xF0, 0xA0, 0x01, 0x53, 0xF0, 0x00, 0x01, 0x57,
            ],
        );
    }

    #[test]
    fn test_toggle() {
        let mut port = create_stub_port();

        port.toggle();

        assert_buf(port, &[0xF0, 0xA0, 0x0C, 0x54, 0xF0, 0x00, 0x00, 0x57]);
    }

    #[test]
    fn test_jog() {
        let mut port = create_stub_port();

        port.jog();

        assert_buf(port, &[0xF0, 0xA0, 0x0C, 0x55, 0xF0, 0xA0, 0x01, 0x53]);
    }

    #[test]
    fn test_open() {
        let port = Port::open(Some("/dev/NOT_FOUND"));

        assert!(port.is_none());
    }

    #[test]
    fn test_find() {
        let port = Port::find_tty(666, 666);

        assert!(port.is_none());
    }
}

#![no_std]
#![no_main]

extern crate libc;

use core::ffi::c_void;
use core::panic::PanicInfo;
use libc::{close, connect, htons, in_addr, recv, send, sockaddr_in, socket, AF_INET, SOCK_STREAM};

// Avoid scary linking errors
#[link(name = "c")]
extern "C" {}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    unsafe {
        // Create raw socket
        let sock = socket(AF_INET, SOCK_STREAM, 0);
        if sock < 0 {
            return 1;
        }

        // Setup server addr
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(21),
            sin_addr: in_addr {
                s_addr: u32::from_be_bytes([127, 0, 0, 1]),
            },
            sin_zero: [0; 8],
        };

        // Connect to the apache server
        let ret = connect(
            sock,
            &addr as *const _ as *const _,
            core::mem::size_of::<sockaddr_in>() as u32,
        );
        if ret < 0 {
            close(sock);
            return 2;
        }

        // Send USER command
        let command = b"USER ftpuser\r\n";
        send(sock, command.as_ptr() as *const c_void, command.len(), 0);

        // Receive response
        let mut buffer = [0u8; 1024];
        let _received = recv(sock, buffer.as_mut_ptr() as *mut c_void, 1024, 0);

        // Do things

        close(sock);
    }
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

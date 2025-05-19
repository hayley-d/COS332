#![no_std]
#![no_main]

extern crate libc;

use core::ffi::c_void;
use core::panic::PanicInfo;
use libc::{close, connect, htons, in_addr, sockaddr_in, socket, write, AF_INET, SOCK_STREAM};

// Avoid scary linking errors
#[link(name = "c")]
extern "C" {}

// No mangling
#[no_mangle]
pub extern "C" fn main() -> i32 {
    unsafe {
        let debug = |msg: &str| {
            write(1, msg.as_ptr() as *const c_void, msg.len());
        };

        debug("[*] Creating socket...\n");
        let sock = socket(AF_INET, SOCK_STREAM, 0);
        if sock < 0 {
            debug("[!] Socket creation failed\n");
            return 1;
        }

        debug("[*] Preparing sockaddr...\n");
        let mut addr: sockaddr_in = core::mem::zeroed();
        addr.sin_family = AF_INET as u16;
        addr.sin_port = htons(21);
        addr.sin_addr = in_addr {
            s_addr: u32::from_be_bytes([127, 0, 0, 1]),
        };

        debug("[*] Connecting to server...\n");
        let ret = connect(
            sock,
            &addr as *const _ as *const _,
            core::mem::size_of::<sockaddr_in>() as u32,
        );
        if ret < 0 {
            debug("[!] Connect failed\n");
            close(sock);
            return 2;
        }

        /*debug("[*] Sending USER command...\n");
        let command = b"USER ftpuser\r\n";
        send(sock, command.as_ptr() as *const c_void, command.len(), 0);

        debug("[*] Receiving server response...\n");
        let mut buffer = [0u8; 1024];
        let received = recv(sock, buffer.as_mut_ptr() as *mut c_void, 1024, 0);

        if received > 0 {
            debug("[+] Server response:\n");
            write(1, buffer.as_ptr() as *const c_void, received as usize);
        } else {
            debug("[!] Failed to receive response\n");
        }*/

        close(sock);
        debug("[*] Done.\n");
    }
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //debug("[*] Panic!\n");
    loop {}
}

use libc::*;
use std::error::Error;
use std::net::TcpListener as StdTcpListener;
use std::os::unix::io::FromRawFd;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls::{self, Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;

pub fn create_raw_socket(port: u16) -> Result<i32, Box<dyn Error>> {
    unsafe {
        // Create a socket
        // AF_INET specifies the IPv4 address fam
        // SOCK_STREAM indicates that the socket will use TCP
        // 0 is default for TCP
        let socket_fd = socket(AF_INET, SOCK_STREAM, 0);

        if socket_fd < 0 {
            eprintln!("Failed to create socket");
            std::process::exit(1);
        }

        // Set socket options
        let option_val: i32 = 1;
        if setsockopt(
            socket_fd,
            SOL_SOCKET,
            SO_REUSEADDR,
            &option_val as *const _ as *const c_void,
            std::mem::size_of_val(&option_val) as u32,
        ) < 0
        {
            eprintln!("Failed to set socket options");
            std::process::exit(1);
        }

        // Bind socket to address
        let address = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(port),
            sin_addr: in_addr { s_addr: INADDR_ANY },
            sin_zero: [0; 8],
        };

        if bind(
            socket_fd,
            &address as *const sockaddr_in as *const sockaddr,
            std::mem::size_of::<sockaddr_in>() as u32,
        ) < 0
        {
            eprintln!("Failed to bind socket to address");
            std::process::exit(1);
        }

        // Start listening at address
        if listen(socket_fd, 128) < 0 {
            eprintln!("Failed to listen on socket");
            std::process::exit(1);
        }

        println!("Server is listening on port {}", port);
        return Ok(socket_fd);
    }
}

async fn start_server(port: u16) -> Result<(), Box<dyn Error>> {
    let raw_fd = create_raw_socket(port)?;
    let std_listener = unsafe { StdTcpListener::from_raw_fd(raw_fd) };
}

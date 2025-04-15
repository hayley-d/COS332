const std = @import("std");
const net = std.net;
const posix = std.posix;

pub fn main() !void {
    //   const username: [19]u8 = "hayleydod@proton.me";
    //   const password: [22]u8 = "YskUEuNu-zSRWtiNizzaxg";

    const addr = try std.net.Address.parseIp("127.0.0.1", 1143);

    const kind: u32 = posix.SOCK.STREAM;
    const protocol = posix.IPPROTO.TCP;
    const listener = try posix.socket(addr.any.family, kind, protocol);
    defer posix.close(listener);
    //const allocator = std.heap.page_allocator;

    // Connect to ProtonMail Bridge's IMAP server (localhost:1143)
    //const socket = try net.tcpConnectToHost(allocator, "127.0.0.1", 1143);

    // Login using ProtonMail Bridge's credentials
    //const login = try std.fmt.format(allocator, "a001 LOGIN {} {}\r\n", .{ username, password });

    // Write the login command to the socket
    //try socket.writer().writeAll(login);

    // Fetch message headers
    //try socket.writer().writeAll("a003 FETCH 1:* (BODY[HEADER.FIELDS (FROM SUBJECT SIZE)])\r\n");

    // Read the server's response using the reader
    //const response = try readResponse(&socket, allocator);
    //defer allocator.free(response); // Free the response after use

    // Print the response
    //std.debug.print("IMAP response: {}\n", .{response});

    // Close the socket
    //socket.close();

    // Free the allocated memory for the formatted login string
    //allocator.free(login);
}

// Function to read the server response from the socket
//fn readResponse(socket: *net.Stream, allocator: *std.mem.Allocator) ![]const u8 {
//    var buffer: [1024]u8 = undefined; // Buffer to store response
//   const bytes_read = try socket.reader().read(buffer[0..]);
//   return try allocator.alloc(u8, bytes_read); // Allocate memory for the read data
//}

const std = @import("std");
const net = std.net;
const print = std.debug.print;

pub fn main() !void {
    //const username = "hayleydod@proton.me";
    //const password = "YskUEuNu-zSRWtiNizzaxg";
    const port_value = 1143;

    const peer = try net.Address.parseIp4("127.0.0.1", port_value);

    // Connect to peer
    const stream = try net.tcpConnectToAddress(peer);
    defer stream.close();
    print("Connecting to {}\n", .{peer});

    // Login using ProtonMail Bridge's credentials
    const login_data = "a001 LOGIN hayleydod@proton.me YskUEuNu-zSRWtiNizzaxg\r\n";
    var writer = stream.writer();
    var size = try writer.write(login_data);
    print("Login Command Sending '{s}' to peer, total written: {d} bytes\n", .{ login_data, size });

    // Select the inbox
    const select_data = "a002 SELECT inbox\r\n";
    size = try writer.write(select_data);
    print("Select Command Sending '{s}' to peer, total written: {d} bytes\n", .{ login_data, size });

    // Fetch message Headers
    const fetch_data = "a003 FETCH 1:* (BODY[HEADER.FIELDS (FROM SUBJECT SIZE)])\r\n";
    size = try writer.write(fetch_data);
    print("Fetch Command Sending '{s}' to peer, total written: {d} bytes\n", .{ login_data, size });

    var read_buffer: [1024]u8 = undefined;
    var reader = stream.reader();
    const res = try reader.read(&read_buffer);
    print("Server Response: {d}\n", .{res});
    print("Server Response: {s}\n", .{read_buffer});

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

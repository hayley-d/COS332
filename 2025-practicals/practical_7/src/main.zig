const std = @import("std");
const expect = std.testing.expect;
const net = std.net;
const os = std.os;

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    const socket = try Socket.init("127.0.0.1", 1143);

    // Login using ProtonMail Bridge's credentials
    const username = "hayleydod@proton.me";
    const password = "YskUEuNu-zSRWtiNizzaxg";
    try socket.writer().writeAll("a001 LOGIN {} {}\r\n", .{username}, .{password});

    // Fetch message headers
    try socket.writer().writeAll("a003 FETCH 1:* (BODY[HEADER.FIELDS (FROM SUBJECT SIZE)])\r\n");

    // Read and process headers
    const response = try socket.read(allocator);
    defer allocator.free(response);

    // Print the response
    std.debug.print("IMAP response: {}\n", .{response});

    // Close the socket
    socket.close();
}

const c_error = enum {
    WriteFailed,
};

const Socket = struct {
    address: std.net.Address,
    socket: std.os.socket,

    fn init(ip: []const u8, port: u16) !Socket {
        const parsed_address = try std.net.Address.parseIp4(ip, port);
        const sock = try std.os.socket(std.os.AF.INET, std.os.SOCK.STREAM, 0);
        errdefer os.closeSocket(sock);
        return Socket{ .address = parsed_address, .socket = sock };
    }

    fn bind(self: *Socket) !void {
        try os.bind(self.socket, &self.address.any, self.address.getOsSockLen());
    }

    fn listen(self: *Socket) !void {
        var buffer: [1024]u8 = undefined;

        while (true) {
            const received_bytes = try std.os.recvfrom(self.socket, buffer[0..], 0, null, null);
            std.debug.print("Received {d} bytes: {s}\n", .{ received_bytes, buffer[0..received_bytes] });
        }
    }

    // Method to write data to the socket
    fn writeAll(self: *Socket, message: []const u8) !void {
        const written = try std.os.write(self.socket, message);

        if (written != message.len) {
            return c_error.WriteFailed;
        }
    }

    fn read(self: *Socket, allocator: *std.mem.Allocator) ![]const u8 {
        const buffer: [1024]u8 = undefined;
        const bytes_read = try std.os.read(self.socket, buffer);
        return try allocator.alloc(u8, bytes_read).catchAllocErr();
    }

    fn close(self: *Socket) void {
        _ = os.closeSocket(self.socket);
    }
};

test "create a socket" {
    const socket = try Socket.init("127.0.0.1", 3000);
    try expect(@TypeOf(socket.socket) == std.os.socket_t);
}

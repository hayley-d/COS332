# LDAP Server
The server implements a basic LDAP-like directory server in OCaml, using BER encoding (Basic Encoding Rules) for data transmission and Unix sockets for networking. It listens on port 7878 for incoming connections, processes requests to look up contact numbers based on a provided "cn" (common name), and sends back responses using a simplified LDAP protocol.

### Unix Module
The Unix module provides system calls for networking (sockets), process management (forking), and file I/O.

### BER Functions
These functions help read and decode BER-encoded data from a client.

#### read_byte
Reads one byte from the socket sock and returns it as an integer.

#### read_length
```ocaml
let read_length sock =
  let len = read_byte sock in
  if len land 0x80 = 0 then len
  else
    let num_bytes = len land 0x7F in
    let rec read_n acc = function
      | 0 -> acc
      | n -> read_n ((acc lsl 8) lor (read_byte sock)) (n - 1)
    in
    read_n 0 num_bytes
```
* Reads a length field from the socket.
* If the first byte's MSB is 0, it's a short form length (0–127 bytes).
* If the MSB is 1, it indicates long form (number of following bytes specifies length).

#### read_bytes
Parameters:
- sock: socket
- len: number of bytes to read from the socket
```ocaml
let read_bytes sock len =
    let buf = Bytes.create len in
    ignore (Unix.read sock buf 0 len);
    buf
```
Reads len bytes from the socket and returns them as a Bytes buffer.

#### make_response
```ocaml
let make_response message_id cn number =
```
When a query is received, this function encodes a response using BER encoding.

```ocaml
let encode_str s =
    let len = String.length s in
    Char.chr 0x04 ::                 (* OCTET STRING tag *)
    (if len < 128 then [Char.chr len] else failwith "Too long") @
    List.init len (fun i -> s.[i])
```
* Encodes a string using BER Octet String format.
* Tag `0x04` indicates an Octet String.

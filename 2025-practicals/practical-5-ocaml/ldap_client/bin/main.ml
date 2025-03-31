open Unix

(* Construct a minimal BER-encoded SearchRequest for cn=Geralt *)
let build_search_request () =
  (* message ID = 1 *)
  let message_id = [0x02; 0x01; 0x01] in

  (* SearchRequest (tag 0x63) *)
  let base_dn = "cn=Geralt" in
  let base_dn_bytes = Bytes.of_string base_dn in
  let base_dn_len = Bytes.length base_dn_bytes in

  let search_request_body =
    [
      0x63;                  (* tag: SearchRequest *)
      0x17;                  (* length: 23 bytes total *)
      0x04; base_dn_len      (* OCTET STRING tag and length *)
    ] @
    List.init base_dn_len (fun i -> Char.code base_dn.[i]) @
    [ 0x0a; 0x01; 0x00;      (* scope: baseObject *)
      0x0a; 0x01; 0x00;      (* derefAliases: never *)
      0x02; 0x01; 0x00;      (* sizeLimit: 0 *)
      0x02; 0x01; 0x00;      (* timeLimit: 0 *)
      0x01; 0x01; 0x00       (* typesOnly: false *)
    ]
  in

  let ldap_message_body = message_id @ search_request_body in
  let full = [0x30; List.length ldap_message_body] @ ldap_message_body in
  Bytes.of_string (String.init (List.length full) (fun i -> Char.chr (List.nth full i)))
;;

let send_and_receive () =
  let server_addr = (gethostbyname "localhost").h_addr_list.(0) in
  let sockaddr = ADDR_INET (server_addr, 7878) in
  let sock = socket PF_INET SOCK_STREAM 0 in

  connect sock sockaddr;
  Printf.printf "Connected to server\n%!";

  let msg = build_search_request () in
  ignore (write sock msg 0 (Bytes.length msg));

  (* Read response *)
  let buf = Bytes.create 4096 in
  let len = read sock buf 0 4096 in
  Printf.printf "Received %d bytes\n" len;

  for i = 0 to len - 1 do
    Printf.printf "%02X " (Char.code (Bytes.get buf i))
  done;
  print_newline ();
  close sock
;;

let () = send_and_receive ()


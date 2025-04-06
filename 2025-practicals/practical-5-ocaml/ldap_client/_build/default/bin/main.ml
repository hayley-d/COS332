open Unix

(* BER-encoded LDAP SearchRequest for cn=<name> *)
let build_search_request name =
  let base_dn = "cn=" ^ name in
  let base_dn_len = String.length base_dn in

  let message_id = [0x02; 0x01; 0x01] in (* INTEGER: msg ID = 1 *)

  let base_dn_field = [0x04; base_dn_len] @ List.init base_dn_len (fun i -> Char.code base_dn.[i]) in

  let search_body =
    [
      0x63; 0x17;       (* tag = SearchRequest, length = 23 bytes *)
    ] @ base_dn_field @
    [ 0x0a; 0x01; 0x00; (* scope: baseObject *)
      0x0a; 0x01; 0x00; (* derefAliases: never *)
      0x02; 0x01; 0x00; (* sizeLimit: 0 *)
      0x02; 0x01; 0x00; (* timeLimit: 0 *)
      0x01; 0x01; 0x00  (* typesOnly: false *)
    ]
  in

  let body = message_id @ search_body in
  let total = 0x30 :: (List.length body :: body) in
  Bytes.of_string (String.init (List.length total) (fun i -> Char.chr (List.nth total i)))

(* Decode the SearchResultEntry response *)
let parse_response bytes len =
  let rec _read_n i n acc =
    if n = 0 then (i, List.rev acc)
    else _read_n (i + 1) (n - 1) (bytes.[i] :: acc)
  in

  let read_tag i =
    let tag = Char.code bytes.[i] in
    let len = Char.code bytes.[i + 1] in
    (tag, len, i + 2)
  in

  let _skip_field i = 
    let _, len, next = read_tag i in
    next + len
  in

  let rec find_number i =
    if i >= len then None
    else
      let tag, field_len, next = read_tag i in
      if tag = 0x04 then
        let str = String.init field_len (fun k -> bytes.[next + k]) in
        if str = "number" then
          let _tag2, len2, next2 = read_tag (next + field_len) in
          let number = String.init len2 (fun k -> bytes.[next2 + k]) in
          Some number
        else find_number (next + field_len)
      else find_number (i + 1)
  in

  find_number 0

let () =
  Printf.printf "Enter friend's name: %!";
  let name = read_line () in
  let req = build_search_request name in

  let addr = (gethostbyname "localhost").h_addr_list.(0) in
  let sock = socket PF_INET SOCK_STREAM 0 in
  connect sock (ADDR_INET (addr, 7878));  

  ignore (write sock req 0 (Bytes.length req));

  let buf = Bytes.create 4096 in
  let n = read sock buf 0 4096 in

  let response_str = Bytes.sub_string buf 0 n in
  match parse_response response_str n with
  | Some number -> Printf.printf "Phone number: %s\n" number
  | None -> Printf.printf "Friend not found or no number in response.\n"

  ; close sock

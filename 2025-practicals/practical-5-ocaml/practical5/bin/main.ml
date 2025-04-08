(* open Unix;; *)

(* Entry type *)
type entry = {
    cn : string;
    number : string;
}

(* Friend list *)
let friends : entry list = [
    { cn = "Geralt"; number = "1110000001"};
    { cn = "Yennefer"; number = "1110000002"};
    { cn = "GLaODS"; number = "1110000003"};
    { cn = "Wheatley"; number = "1110000004"};
    { cn = "Atlas"; number = "2220000007"};
    { cn = "Pbody"; number = "2220000008"};
    { cn = "Rick"; number = "2220000009"};
    { cn = "Caroline"; number = "2220000001"};
    { cn = "Emma"; number = "2220000002"};
    { cn = "Hayley"; number = "2220000003"};
    { cn = "Chell"; number = "2220000004"};
    { cn = "Lambert"; number = "2220000005"};
    { cn = "Ciri"; number = "2220000006"};
    { cn = "Triss"; number = "2220000010"};
    { cn = "Roach"; number = "2220000011"};
]

(* BER utility functions *)
let read_byte sock = 
    let buf = Bytes.create 1 in
    ignore (Unix.read sock buf 0 1);
    int_of_char (Bytes.get buf 0)

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


let read_bytes sock len =
    let buf = Bytes.create len in
    ignore (Unix.read sock buf 0 len);
    buf

let make_response message_id cn number =
  (* Encode a BER Octet String (tag 0x04) *)
  let encode_str s =
    let len = String.length s in
    Char.chr 0x04 ::                 (* OCTET STRING tag *)
    (if len < 128 then [Char.chr len] else failwith "Too long") @
    List.init len (fun i -> s.[i])
  in

  (* Encode "cn" -> <cn value> *)
  let encoded_cn =
    let attr_type = encode_str "cn" in
    let attr_value = encode_str cn in
    [Char.chr 0x30;  (* SEQUENCE for one attribute *)
     Char.chr (List.length attr_type + List.length attr_value)
    ] @ attr_type @ attr_value
  in

  (* Encode "number" -> <number value> *)
  let encoded_tel =
    let attr_type = encode_str "number" in
    let attr_value = encode_str number in
    [Char.chr 0x30;
     Char.chr (List.length attr_type + List.length attr_value)
    ] @ attr_type @ attr_value
  in

  (* Wrap both attributes in an outer SEQUENCE *)
  let attrs =
    let inner = encoded_cn @ encoded_tel in
    [Char.chr 0x30; Char.chr (List.length inner)] @ inner
  in

  (* Construct the full DN string *)
  let dn_string = "cn=" ^ cn ^ ",ou=Friends,dc=example,dc=com" in
  let encoded_dn = encode_str dn_string in

  (* Construct the SearchResultEntry (APPLICATION 4 / tag 0x64) *)
  let entry_content = encoded_dn @ attrs in
  let entry =
    [Char.chr 0x64; Char.chr (List.length entry_content)] @ entry_content
  in

  (* Encode the message ID as INTEGER (tag 0x02) *)
  let msg_id = [Char.chr 0x02; Char.chr 0x01; Char.chr message_id] in

  (* Wrap everything in an outer LDAPMessage SEQUENCE (tag 0x30) *)
  let full = msg_id @ entry in
  let full_len = List.length full in
  let ldap_message = [Char.chr 0x30; Char.chr full_len] @ full in

  (* Convert char list to bytes *)
  Bytes.of_string (String.init (List.length ldap_message) (fun i -> List.nth ldap_message i))


(* LDAP client handling *)
let handle_client sock =
  Printf.printf "Client connected\n%!";
  let tag = read_byte sock in
  if tag <> 0x30 then failwith "Expected SEQUENCE";

  let _len = read_length sock in
  let _msg_id_tag = read_byte sock in
  let _msg_id_len = read_length sock in
  let msg_id = read_byte sock in

  let op_tag = read_byte sock in
  if op_tag = 0x60 then
    let _ = read_length sock in
    ignore (read_bytes sock 3); 
    ()
  else if op_tag = 0x63 then
    let _ = read_length sock in
    let base_dn_tag = read_byte sock in
    Printf.printf "Base DN tag: 0x%02X\n!" base_dn_tag;
    let base_dn_len = read_length sock in
    let base_dn_bytes = read_bytes sock base_dn_len in
    let base_dn = Bytes.to_string base_dn_bytes in
    Printf.printf "Client queried for: %s\n%!" base_dn;

    let cn =
      try
        Scanf.sscanf base_dn "cn=%s@," (fun x -> x)
      with _ -> ""
    in
    let found = List.find_opt (fun e -> String.lowercase_ascii e.cn = String.lowercase_ascii cn) friends in
    begin match found with
    | Some e ->
        let response = make_response msg_id e.cn e.number in
        ignore (Unix.write sock response 0 (Bytes.length response))
    | None ->
        Printf.printf "Friend not found: %s\n!" base_dn
    end
  else
    Printf.printf "Unknown operation tag: %d\n!" op_tag

(* Start a TCP server *)
let start_server () =
    (* Create a socket using the port number *)
    let sock = Unix.socket Unix.PF_INET Unix.SOCK_STREAM 0 in
    Unix.setsockopt sock Unix.SO_REUSEADDR true;
    Unix.bind sock (Unix.ADDR_INET (Unix.inet_addr_any, 7878));
    Unix.listen sock 5;
    Printf.printf "LDAP server running on port 7878\n%!";
    while true do
        let client, _ = Unix.accept sock in
        match Unix.fork () with
        | 0 -> Unix.close sock;
            handle_client client;
            Unix.close client;
            exit 0
        | _ -> Unix.close client
    done

(* Entry point *)
let () = 
    Printf.printf "Starting server....\n%!";
    start_server ()



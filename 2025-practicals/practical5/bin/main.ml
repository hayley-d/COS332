open Unix;;

type entry = {
    cn : string;
    number : string;
}

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
  let encode_str s =
    let len = String.length s in
    Char.chr 0x04 :: (if len < 128 then [Char.chr len] else failwith "Too long") @ List.init len (fun i -> s.[i])
  in
  let encoded_cn = encode_str "cn" @ encode_str cn in
  let encoded_tel = encode_str "number" @ encode_str number in
  let attrs = [0x30] :: [Char.chr (List.length encoded_cn + List.length encoded_tel)] @ encoded_cn @ encoded_tel in
  let entry =
    [0x64] @ [Char.chr (List.length cn + List.length attrs)] @ encode_str ("cn=" ^ cn ^ ",ou=Friends,dc=example,dc=com") @ attrs
  in
  let msg_id = [0x02; 0x01; Char.chr message_id] in
  let full = msg_id @ entry in
  let full_len = List.length full in
  Bytes.of_string (String.init (2 + full_len) (fun i ->
    if i = 0 then Char.chr 0x30
    else if i = 1 then Char.chr full_len
    else List.nth full (i - 2)
  ))


(* LDAP client handling *)
let handle_client sock =
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
        Printf.printf "Friend not found: %s\n" base_dn
    end
  else
    Printf.printf "Unknown operation tag: %d\n" op_tag

(* Start a TCP server *)
let start_server () =
    (* Create a socket using the port number *)
    let sock = Unix.socket Unix.PF_INET Unix.SOCK_STREAM 0 in
    Unix.setsockopt sock Unix.SO_REUSEADDR true;
    Unix.bind sock (Unix.ADDR_INET (Unix.inet_addr_any, 7878));
    Unix.listen sock 5;
    Printf.printf "LDAP server running on port 7878\n"; 
    while true do
        let client, _ = Unix.accept sock in
        match Unix.fork () with
        | 0 -> Unix.close sock;
            handle_client client;
            Unix.close client;
            exit 0
        | pid -> Unix.close client
    done

(* Entry point *)
let () = start_server ()



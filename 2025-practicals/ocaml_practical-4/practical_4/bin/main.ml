open Unix

let kv_store : (string, string) Hashtbl.t = Hashtbl.create 100;

    let parse_request_line line = 
    match String.split_on_char ' ' line with
        | [method_; path; _] -> (method_, path)
        | _ -> ("","")


    let handle_request req = 
        let lines = String.split_on_char '\n' req in
        match lines with
        | [] -> "HTTP/1.1


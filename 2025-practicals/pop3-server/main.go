package main

import (
    "fmt"
    "net"
    "strings"
)

func main() {
    listener, err := net.Listen("tcp","localhost:1100")
    if err != nil {
        fmt.Println("Error starting server:",err)
        return
    }
    defer listener.Close()
    fmt.Println("POP3 server listening on localhost:1100")

    for {
        conn, err := listener.Accept()
        if err != nil {
            fmt.Println("Failed to accept connection:",err)
            continue
        }
        go handleClient(conn)
    }
}

func handleClient(conn net.Conn) {
    defer conn.Close()
    conn.Write([]byte("+OK POP3 server ready\r\n"))

    buffer := make([]byte,4096)

    // Variables to track state
    authenticated := false
    var username string

    users := map[string]string {
        "hayley@proton.me" : "u21528790",
    }

    for {
        n, err := conn.Read(buffer)
        if err != nil {
            fmt.Println("Client disconnected")
            return
        }

        input := strings.TrimSpace(string(buffer[:n]))
        fmt.Println("Received command:", input)

        parts := strings.SplitN(input, " ",2)
        command := strings.ToUpper(parts[0])
        arg := ""

        if len(parts) > 1 {
            arg = parts[1]
        }

        switch command {
        case "USER":
            username = arg
            if _, exists := users[username]; exists {
                conn.Write([]byte("+OK User accepted, please send PASS\r\n"))
                authenticated = true
            } else {
                conn.Write([]byte("-ERR No such user\r\n"))
            }

        case "PASS":
            if username == "" {
                conn.Write([]byte("-ERR USER required first\r\n"))
            } 
            expectedPass := users[username]
            if arg == expectedPass {
                authenticated = true
                conn.Write([]byte("+OK Authenticated\r\n"))
            } else {
                conn.Write([]byte("-ERR Invalid password\r\n"))
            }

        case "QUIT":
            conn.Write([]byte("+OK Goodbye\r\n"))
            return

        default:
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            } 
            //conn.Write([]byte("-ERR Unknown command\r\n"))
        }

        if strings.HasPrefix(strings.ToUpper(input),"QUIT") {
            conn.Write([]byte("+ OK Goodbye\r\n"))
            return
        } else {
            conn.Write([]byte("-ERR Unknown command\r\n"))
        }
    }
}

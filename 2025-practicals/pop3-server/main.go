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

    for {
        n, err := conn.Read(buffer)
        if err != nil {
            fmt.Println("Client disconnected")
            return
        }

        input := strings.TrimSpace(string(buffer[:n]))
        fmt.Println("Received command:", input)

        if strings.HasPrefix(strings.ToUpper(input),"QUIT") {
            conn.Write([]byte("+ OK Goodbye\r\n"))
            return
        } else {
            conn.Write([]byte("-ERR Unknown command\r\n"))
        }
    }
}

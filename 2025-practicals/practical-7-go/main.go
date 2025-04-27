package main

import (
    "bufio"
    "fmt"
    "net"
    "os"
    "strings"
    "github.com/joho/godotenv"
)

func main() {
    err := godotenv.Load()
    if err != nil {
        fmt.Println("Error loading .env file")
        return
    }

    serverAddress := os.Getenv("POP3_SERVER")
    username := os.Getenv("POP3_USER")
    password := os.Getenv("POP3_PASS")

    // Connect to the POP3 server
    conn, err := net.Dial("tcp", serverAddress)
    if err != nil {
        fmt.Println("Failed to connect:", err)
        return
    }
    defer conn.Close()
    fmt.Println("Connected to server")

    reader := bufio.NewReader(conn)

    // Read server's initial greeting
    line, _ := reader.ReadString('\n')
    fmt.Print(line)

    // Login
    sendCommand(conn, reader, "USER " + username)
    sendCommand(conn, reader, "PASS " + password)

    // TODO: Add more functionality here
}

func sendCommand(conn net.Conn, reader *bufio.Reader, cmd string) {
    fmt.Fprintf(conn, "%s\r\n", cmd)
    response, _ := reader.ReadString('\n')
    fmt.Print(response)
}

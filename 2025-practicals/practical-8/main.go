package main

import (
    "fmt"
    "net"
    "io"
    "os"
    "strings"
    "time"
)

const (
    server = "serverIp:21"
    username = "username"
    password = "password"
    filePath = "file.html"
)

func main() {
    var lastModTime time.Time

    for {
        fi, err := os.Stat(filePath)
        if err != nil {
            fmt.Println("Error stating file: ", err)
            time.Sleep(10 * time.Second)
            continue
        }

        if fi.ModTime().After(lastModTime) {
            fmt.Println("File changed, uploading....")
            err := uploadFile()
            if err != nil {
                fmt.Println("Upload failed: ", err)
            } else {
                lastModTime = fi.ModTime()
            }
        }

        // Poll every 30 seconds
        time.Sleep(30 * time.Second)
    }
}

func uploadFile() error {
    conn, err := net.Dial("tcp", server)
    if err != nil {
        return fmt.Errorf("Cannot connect: %w",err)
    }
    defer conn.Close()

    // Read the initial server response
    readResponse(conn)

    // Send login details
    sendCommand(conn, "USER " + username)
    readResponse(conn)
    sendCommand(conn, "PASS " + password)
    readResponse(conn)

    // Set binary mode
    sendCommand(conn, "Type I")
    readResponse(conn)

    sendCommand(conn, "PASV")
    pasvResp := readResponse(conn)

    dataAdd, err := parsePASC(pasvResp)
    if err != nil {
        return fmt.Errorf("PASV parse error: %w", err)
    }
}

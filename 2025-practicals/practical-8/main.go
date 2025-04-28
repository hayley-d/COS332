package main

import (
    "fmt"
    "net"
    "io"
    "os"
    "strings"
    "time"
    "strconv"
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

    dataAdd, err := parsePASV(pasvResp)
    if err != nil {
        return fmt.Errorf("PASV parse error: %w", err)
    }

    dataConn, err := net.Dial("tcp", dataAdd)
    if err != nil {
        return fmt.Errorf("cannot open data connection: %w", err)
    }
    defer dataConn.Close()

    // Send STOR command
    sendCommand(conn, "STOR file.html")
    readResponse(conn)

    // Open the file and copy it into the connection
    open_file, err := os.Open(filePath)
    if err != nil {
        return fmt.Errorf("cannot open file: %w", err)
    }
    defer open_file.Close()

    _, err = io.Copy(dataConn, open_file)
    if err != nil {
        return fmt.Errorf("error sending file: %w", err)
    }

    dataConn.Close()
    readResponse(conn)

    sendCommand(conn, "QUIT")
    readResponse(conn)

    return nil
}

func sendCommand(conn net.Conn, cmd string) {
    fmt.Fprintf(conn, "%s\r\n", cmd)
}

func readResponse(conn net.Conn) string {
    buffer := make([]byte, 4096)
    size, _ := conn.Read(buffer)
    response := string(buffer[:size])
    fmt.Println("SERVER:", strings.TrimSpace(response))
    return response
}

func parsePASV(response string) (string, error) {
    start := strings.Index(response, "(")
    end := strings.Index(response, ")")

    if start == -1 || end == -1 {
        return "", fmt.Errorf("invalid PASV response")
    }

    numbers := strings.Split(response[start+1:end],",")

    if len(numbers) != 6 {
        return "", fmt.Errorf("invalid PASV address")
    }

    ip := strings.Join(numbers[0:4],".")

    part1, err1 := strconv.Atoi(numbers[4])
    part2, err2 := strconv.Atoi(numbers[5])

    if err1 != nil || err2 != nil {
        return "", fmt.Errorf("invalid PASV port numbers")
    }

    port := part1*256 + part2
    return fmt.Sprintf("%s:%d",ip, port), nil
}

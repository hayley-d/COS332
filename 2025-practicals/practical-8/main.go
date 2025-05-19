package main

import (
	"bufio"
	"crypto/md5"
	"fmt"
	"io"
	"net"
	"os"
	"strings"
	"time"
)

const (
	ftpServer = "127.0.0.1:21"
	filename  = "index.html"
)

var username = os.Getenv("FTP_USER")
var password = os.Getenv("FTP_PASS")

func main() {
	var lastHash [16]byte

	for {
		hash, err := hashFile(filename)
		if err != nil {
			fmt.Println("Error hashing file:", err)
			continue
		}

		if hash != lastHash {
			fmt.Println("Change detected, uploading...")
			err := uploadFile(filename)
			if err != nil {
				fmt.Println("Upload failed:", err)
			} else {
				fmt.Println("Upload successful.")
				lastHash = hash
			}
		}

		time.Sleep(10 * time.Second)
	}
}

func hashFile(path string) ([16]byte, error) {
	var zero [16]byte
	file, err := os.Open(path)
	if err != nil {
		return zero, err
	}
	defer file.Close()
	return md5.Sum(readAll(file)), nil
}

func readAll(file *os.File) []byte {
	data, _ := io.ReadAll(file)
	return data
}

func uploadFile(path string) error {
	conn, err := net.Dial("tcp", ftpServer)
	if err != nil {
		return err
	}
	defer conn.Close()
	r := bufio.NewReader(conn)

	expect(r, "220")

	send(conn, "USER "+username)
	expect(r, "331")

	send(conn, "PASS "+password)
	expect(r, "230")

	send(conn, "TYPE I")
	expect(r, "200")

	send(conn, "PASV")
	line := readLine(r)
	fmt.Println("PASV response:", line)

	// Extract IP and port for data connection
	ip, port := parsePASV(line)
	dataConn, err := net.Dial("tcp", fmt.Sprintf("%s:%d", ip, port))
	if err != nil {
		return err
	}
	defer dataConn.Close()

	send(conn, "STOR "+filename)
	expect(r, "150")

	file, err := os.Open(path)
	if err != nil {
		return err
	}
	defer file.Close()

	io.Copy(dataConn, file)
	dataConn.Close()

	expect(r, "226")
	send(conn, "QUIT")
	return nil
}

func send(conn net.Conn, msg string) {
	fmt.Fprintf(conn, msg+"\r\n")
}

func expect(r *bufio.Reader, code string) {
	line := readLine(r)
	if !strings.HasPrefix(line, code) {
		panic("unexpected response: " + line)
	}
}

func readLine(r *bufio.Reader) string {
	line, _ := r.ReadString('\n')
	return strings.TrimSpace(line)
}

func parsePASV(resp string) (string, int) {
	start := strings.Index(resp, "(")
	end := strings.Index(resp, ")")
	parts := strings.Split(resp[start+1:end], ",")

	ip := strings.Join(parts[0:4], ".")
	p1 := atoi(parts[4])
	p2 := atoi(parts[5])
	port := p1*256 + p2

	return ip, port
}

func atoi(s string) int {
	var n int
	fmt.Sscanf(s, "%d", &n)
	return n
}

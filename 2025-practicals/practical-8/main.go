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
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// FTP server configs the url and html file to watch for changes.
const (
	ftpServer = "127.0.0.1:21"
	filename  = "index.html"
)

var (
	// Enviroment variables for credentials.
	username = os.Getenv("FTP_USER")
	password = os.Getenv("FTP_PASS")

	// Pretty pink text styling
	pink     = lipgloss.NewStyle().Foreground(lipgloss.Color("#ff69b4"))
)

/* 
 Bubbletea model that holds the cli state
 status: status message
 lastResp: Last response from the ftp server
 lastHash: Last file hash to detect changes
*/
type model struct {
	status   string	
	lastResp string
	lastHash [16]byte
}

// Bubbletea init func
func (m model) Init() tea.Cmd {
	// 10 second polling interval
	return tea.Tick(10*time.Second, func(t time.Time) tea.Msg {
		return tickMsg(t)
	})
}

// Custom message type for polling ticks
type tickMsg time.Time

// Handles user input and events
func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {

	case tea.KeyMsg:
		// Quit on Ctrl + C
		if msg.String() == "ctrl+c" {
			return m, tea.Quit
		}

	case tickMsg:
		// Every 10 seconds check if the file has changed
		hash, err := hashFile(filename)
		if err != nil {
			m.status = "Error hashing file: " + err.Error()
		} else if hash != m.lastHash {
			m.status = "Change detected, uploading..."
			resp, err := uploadFile(filename)
			if err != nil {
				m.status = "Upload failed: " + err.Error()
				m.lastResp = resp
			} else {
				m.status = "Upload successful!"
				m.lastHash = hash
				m.lastResp = resp
			}
		}

		// Schedule next polling time
		return m, tea.Tick(10*time.Second, func(t time.Time) tea.Msg {
			return tickMsg(t)
		})
	}

	return m, nil
}

// View renders the current state to the terminal using pink colour
func (m model) View() string {
	return pink.Render(fmt.Sprintf(
		"%s\n\nLast FTP Response:\n%s\n\nPress Ctrl+C to exit",
		m.status, m.lastResp))
}

// Program entry point
func main() {
	p := tea.NewProgram(model{})
	if err := p.Start(); err != nil {
		fmt.Println("Error running program:", err)
		os.Exit(1)
	}
}
/*func main() {
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
}*/

// Computes the MD5 hash of the given file.
// Used to detect when the file content was changed.
func hashFile(path string) ([16]byte, error) {
	var zero [16]byte
	file, err := os.Open(path)
	if err != nil {
		return zero, err
	}
	defer file.Close()
	return md5.Sum(readAll(file)), nil
}

// Reads the entire file into memory
func readAll(file *os.File) []byte {
	data, _ := io.ReadAll(file)
	return data
}

// Performs manual FTP upload of the specified files.
// Returns the final FTP server response or error.
func uploadFile(path string) (string,error) {
	conn, err := net.Dial("tcp", ftpServer)

	if err != nil {
		return "", err
	}

	defer conn.Close()
	response := bufio.NewReader(conn)

	// Start communication with FTP server
	if line, ok := expect(response,"220"); !ok {
		return line, fmt.Errorf("Expected 220 (welcome)")
	}

	// Send the USER command with the username
	send(conn, "USER "+username)
	if line, ok := expect(response,"331"); !ok {
		return line, fmt.Errorf("Expected 331 after USER command")
	}

	// Send the PASS command with the password
	send(conn, "PASS "+password)
	if line, ok := expect(response,"230"); !ok {
		return line, fmt.Errorf("Expected 230 after PASS command")
	}

	send(conn, "TYPE I")
	if line, ok := expect(response, "200"); !ok {
		return line, fmt.Errorf("Expected 200 after TYPE I command")
	}

	// Enter passive mode for data transfer
	send(conn, "PASV")
	psvLine := readLine(response)
	ip, port := parsePASV(psvLine)

	dataConn, err := net.Dial("tcp", fmt.Sprintf("%s:%d", ip, port))
	if err != nil {
		return psvLine, err
	}
	defer dataConn.Close()

	// Send STOR command to start the file upload
	send(conn, "STOR "+filename)
	if line, ok := expect(response, "150"); !ok {
		return line, fmt.Errorf("Expected 150 after STOR command")
	}

	// Send the file content over the data connection
	file, err := os.Open(path)
	if err != nil {
		return "", err 
	}
	defer file.Close()

	io.Copy(dataConn, file)
	dataConn.Close()

	// Confirm success
	if line, ok := expect(response,"226"); !ok {
		return line, fmt.Errorf("Expected 226 after file upload")
	}

	// End FTP session
	send(conn, "QUIT")
	return "226 File Transfer complete", nil
}

// Sends a single FTP command
func send(conn net.Conn, msg string) {
	fmt.Fprintf(conn, msg+"\r\n")
}

// Reads a line and checks if it starts with the expected FTP status code
func expect(r *bufio.Reader, code string) (string,bool)  {
	line := readLine(r)
	return line, strings.HasPrefix(line, code)
}

// Reads a line from the connection and trims the new line chars
func readLine(r *bufio.Reader) string {
	line, _ := r.ReadString('\n')
	return strings.TrimSpace(line)
}

// Extracts the IP and port numbers froma PASV response
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

// Converts string to int
func atoi(s string) int {
	var n int
	fmt.Sscanf(s, "%d", &n)
	return n
}

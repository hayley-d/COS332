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

// FTP connection configuration
const (
	filename  = "index.html"
)

// Environment variables and UI styles
var (
	username = os.Getenv("FTP_USER")
	password = os.Getenv("FTP_PASS")
	ftpServer = os.Getenv("FTP_URL")

	styleTitle     = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("13"))
	styleStatus    = lipgloss.NewStyle().Foreground(lipgloss.Color("10"))
	styleCommand   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#f57de3"))
	styleArrow     = lipgloss.NewStyle().Foreground(lipgloss.Color("86"))
	styleResponse  = lipgloss.NewStyle().Foreground(lipgloss.Color("7"))
	styleLogBox    = lipgloss.NewStyle().MarginTop(1).PaddingLeft(1).PaddingRight(1)
)

// Custom message type for triggering periodic checks
type tickMsg time.Time

// A single FTP interaction log entry
// cmd: the command sent to the FTP server
// response: the resulting response string from the server
type logEntry struct {
	cmd      string
	response string
}

// The main TUI model that holds app state
// status: descriptive label of what the program is doing
// lastHash: the last MD5 hash of the watched file
// logs: list of previous FTP command-response logs
type model struct {
	status   string
	lastHash [16]byte
	logs     []logEntry
}

// The entry point that constructs the thingy and runs the program
func main() {
	// Initialize the Bubbletea thingy with the initial model
	p := tea.NewProgram(model{status : "Waiting for changes..."})
    if _, err := p.Run(); err != nil {
        fmt.Printf("Alas, there's been an error: %v", err)
        os.Exit(1)
    }
}

// Initializes the thingy and starts polling every 10 seconds
func (m model) Init() tea.Cmd {
	return tea.Tick(10*time.Second, func(t time.Time) tea.Msg {
		return tickMsg(t)
	})
}

// Responds to messages + keypresses + periodic file polling
func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		if msg.String() == "ctrl+c" {
			return m, tea.Quit
		}

	case tickMsg:
		// Check if file has changed by comparing hash
		hash, err := hashFile(filename)
		if err != nil {
			m.status = "Hash error: " + err.Error()
			break
		}
		if hash != m.lastHash {
			m.status = "Uploading..."
			logs, err := uploadFile(filename)
			if err != nil {
				m.status = "Upload failed: " + err.Error()
			} else {
				m.status = "Upload successful."
				m.lastHash = hash
			}
			m.logs = append(m.logs, logs...)
		}
		return m, tea.Tick(10*time.Second, func(t time.Time) tea.Msg {
			return tickMsg(t)
		})
	}

	return m, nil
}

// Renders the pretty components title, status, and FTP log history
func (m model) View() string {
	title := styleTitle.Render(" FTP Upload Monitor")
	status := styleStatus.Render(m.status)
	logLines := make([]string, 0, len(m.logs))
	// Format log entries with command → response style
	for _, entry := range m.logs {
		logLines = append(logLines,
			fmt.Sprintf("%s %s %s",
				styleCommand.Render("> "+entry.cmd),
				styleArrow.Render("→"),
				styleResponse.Render(entry.response),
			))
	}
	logs := styleLogBox.Render(strings.Join(logLines, "\n"))
	return fmt.Sprintf("%s\n%s\n\n%s\n\nPress Ctrl+C to quit.", title, status, logs)
}

// Computes the MD5 hash of a file
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

// Uploads the file via FTP and logs each step
func uploadFile(path string) ([]logEntry, error) {
	conn, err := net.Dial("tcp", ftpServer)
	if err != nil {
		return nil, err
	}
	defer conn.Close()
	r := bufio.NewReader(conn)
	var logs []logEntry

	// A helper for sending command and logging result
	logf := func(cmd string, expectCode string) (string, error) {
		if cmd != "" {
			send(conn, cmd)
		}
		line := readLine(r)
		logs = append(logs, logEntry{cmd, line})
		if expectCode != "" && !strings.HasPrefix(line, expectCode) {
			return line, fmt.Errorf("Expected %s, got: %s", expectCode, line)
		}
		return line, nil
	}

	if _, err := logf("", "220"); err != nil {
		return logs, err
	}
	if _, err := logf("USER "+username, "331"); err != nil {
		return logs, err
	}
	if _, err := logf("PASS "+password, "230"); err != nil {
		return logs, err
	}
	if _, err := logf("TYPE I", "200"); err != nil {
		return logs, err
	}

	send(conn, "PASV")
	pasvLine := readLine(r)
	logs = append(logs, logEntry{"PASV", pasvLine})
	ip, port := parsePASV(pasvLine)

	dataConn, err := net.Dial("tcp", fmt.Sprintf("%s:%d", ip, port))
	if err != nil {
		return logs, err
	}
	defer dataConn.Close()

	if _, err := logf("STOR "+filename, "150"); err != nil {
		return logs, err
	}

	file, err := os.Open(path)
	if err != nil {
		return logs, err
	}
	defer file.Close()
	io.Copy(dataConn, file)
	dataConn.Close()

	if _, err := logf("", "226"); err != nil {
		return logs, err
	}
	logf("QUIT", "")
	return logs, nil
}

func send(conn net.Conn, msg string) {
	fmt.Fprintf(conn, msg+"\r\n")
}

func readLine(r *bufio.Reader) string {
	line, _ := r.ReadString('\n')
	return strings.TrimSpace(line)
}

// Parses PASV response and extracts the IP and port number
func parsePASV(resp string) (string, int) {
	start := strings.Index(resp, "(")
	end := strings.Index(resp, ")")
	parts := strings.Split(resp[start+1:end], ",")
	ip := strings.Join(parts[0:4], ".")
	p1 := atoi(parts[4])
	p2 := atoi(parts[5])
	return ip, p1*256 + p2
}

// Parses string to integer
func atoi(s string) int {
	var n int
	fmt.Sscanf(s, "%d", &n)
	return n
}


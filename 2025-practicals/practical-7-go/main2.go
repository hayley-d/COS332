package main

import (
    "bufio"
    "fmt"
    "net"
    "os"
    "strings"
    "github.com/joho/godotenv"
    "github.com/fatih/color"
)

type MessageInfo struct {
    Index int
    Id string
    Sender string
    Subject string
    Size int
}

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

    // Main server loop
    for {
        printMenu()

        var choice int
        fmt.Print("\nEnter your choice: ")
        _ , err := fmt.Scanln(&choice)
        if err != nil {
            fmt.Println("Invlaid input, please enter a number.")
            continue
        }

        switch choice {
        case 1:
            sendCommandMultiline(conn,reader,"LIST")

        case 2:
            fmt.Print("Enter message number to retrieve: ")
            var msgNum string
            fmt.Scanln(&msgNum)
            sendCommandMultiline(conn, reader, "RETR " + msgNum)

        case 3:
            fmt.Print("Enter message number to delete: ")
            var msgNum string
            fmt.Scanln(&msgNum)
            sendCommand(conn, reader, "DELE " + msgNum)

        case 4:
            sendCommand(conn, reader, "STAT")

        case 5:
            sendCommandMultiline(conn, reader, "UIDL")

        case 6:
            sendCommand(conn, reader, "RSET")

        case 7:
            sendCommand(conn, reader, "NOOP")

        case 8: 
            messages := fetchMessageList(conn, reader)

            fetchMessageHeaders(conn, reader, messages)

            displayMessages(messages)

            deleteHelper(conn, reader)

        case 9:
            sendCommand(conn, reader, "QUIT")
            fmt.Println("Goodbye!")
            return

        default:
            fmt.Println("Invalid choice, try again.")
        }
    }
}

func fetchMessageList(conn net.Conn, reader *bufio.Reader) []MessageInfo {
    fmt.Fprintf(conn, "%s\r\n", "LIST")

    var messages []MessageInfo

    for {
        line, _ := reader.ReadString('\n')

        if strings.TrimSpace(line) == "." {
            break
        }

        if strings.HasPrefix(line, "+OK") {
            green := color.New(color.FgHiGreen).SprintFunc()
            fmt.Print(green(line))
        } else if strings.HasPrefix(line, "-ERR") {
            red := color.New(color.FgHiRed).SprintFunc()
            fmt.Print(red(line))
        } else {
            var idx, size int
            fmt.Sscanf(line, "%d %d", &idx, &size)
            messages = append(messages, MessageInfo{Index: idx, Size: size})
        }
    }

    return messages
}

func fetchMessageHeaders(conn net.Conn, reader *bufio.Reader, messages []MessageInfo) {
    for i := range messages {
        fmt.Fprintf(conn, "RETR %d\r\n", messages[i].Index)
        var from, subject string

        for {
            line, _ := reader.ReadString('\n')
            line = strings.TrimSpace(line)

            if line == "." {
                break
            }

            if strings.HasPrefix(line, "+OK") {
                green := color.New(color.FgHiGreen).SprintFunc()
                fmt.Print(green(line))
            } else if strings.HasPrefix(line, "-ERR") {
                red := color.New(color.FgHiRed).SprintFunc()
                fmt.Print(red(line))
            } else {
                if line == "" {
                    discardBody(reader)
                    break
                } else if strings.HasPrefix(strings.ToLower(line), "from: ") {
                    from = strings.TrimSpace(line[5:])
                } else if strings.HasPrefix(strings.ToLower(line), "subject: ") {
                    subject = strings.TrimSpace(line[8:])
                }
            } 
            messages[i].Sender = from
            messages[i].Subject = subject
        }

    }
}

func discardBody(reader *bufio.Reader) {
    for {
        line, _ := reader.ReadString('\n')
        if strings.TrimSpace(line) == "." {
            break
        }
    }
}

func displayMessages(messages []MessageInfo) {
    lilac := color.New(color.FgHiMagenta).SprintFunc()

    fmt.Println(lilac("\n╔════╦════════════════════════════════════════╦════════════════════════════════╦══════╗"))
    fmt.Println(lilac("║ #  ║ From                                   ║ Subject                        ║ Size ║"))
    fmt.Println(lilac("╠════╬════════════════════════════════════════╬════════════════════════════════╬══════╣"))
    for _, m := range messages {
        from := truncateOrPad(m.Sender, 37)
        subject := truncateOrPad(m.Subject, 29)
        fmt.Printf("║ %-2d ║ %-38s ║ %-30s ║ %-4d ║\n", m.Index, from, subject, m.Size)
    }
    fmt.Println(lilac("╚════╩════════════════════════════════════════╩════════════════════════════════╩══════╝"))
}

// Helper function to trancate a string if too long or pad a string if to short
func truncateOrPad(s string, length int) string {
    if len(s) > length {
        return s[:length-3] + "..."
    }
    return fmt.Sprintf("%-*s", length, s)
}

func sendCommand(conn net.Conn, reader *bufio.Reader, cmd string) {
    fmt.Fprintf(conn, "%s\r\n", cmd)
    response, _ := reader.ReadString('\n')
    
    if strings.HasPrefix(response, "+OK") {
        green := color.New(color.FgHiGreen).SprintFunc()
        fmt.Print(green(response))
    } else if strings.HasPrefix(response, "-ERR") {
        red := color.New(color.FgHiRed).SprintFunc()
        fmt.Print(red(response))
    } else {
        fmt.Print(response)
    }
}

func deleteHelper(conn net.Conn, reader *bufio.Reader) {
    fmt.Print("Enter message number to delete ('q' to delete nothing): ")
    var msgNum string
    fmt.Scanln(&msgNum)

    if msgNum == "q" {
        return
    }

    sendCommand(conn, reader, "DELE " + msgNum)
}

func sendCommandMultiline(conn net.Conn, reader *bufio.Reader, cmd string) {
    fmt.Fprintf(conn, "%s\r\n", cmd)
    for {
        line, _ := reader.ReadString('\n')
        if strings.TrimSpace(line) == "." {
            break
        }
        if strings.HasPrefix(line, "+OK") {
            green := color.New(color.FgHiGreen).SprintFunc()
            fmt.Print(green(line))
        } else if strings.HasPrefix(line, "-ERR") {
            red := color.New(color.FgHiRed).SprintFunc()
            fmt.Print(red(line))
        } else {
            fmt.Print(line)
        }
    }
}

func printMenu() {
    lilac := color.New(color.FgHiMagenta).SprintFunc()

    fmt.Print()
    fmt.Println(lilac("╔═══════════════════════════════════════╗"))
    fmt.Println(lilac("║              POP3 Client              ║"))
    fmt.Println(lilac("╠═══════════════════════════════════════╣"))
    fmt.Println(lilac("║ 1. List Emails                        ║"))
    fmt.Println(lilac("║ 2. Retrieve Email                     ║"))
    fmt.Println(lilac("║ 3. Delete Email                       ║"))
    fmt.Println(lilac("║ 4. View Inbox Status (STAT)           ║"))
    fmt.Println(lilac("║ 5. View Unique IDs (UIDL)             ║"))
    fmt.Println(lilac("║ 6. Reset Deletions (RSET)             ║"))
    fmt.Println(lilac("║ 7. Ping Server (NOOP)                 ║"))
    fmt.Println(lilac("║ 8. Assignment Task                    ║"))
    fmt.Println(lilac("║ 9. Quit                               ║"))
    fmt.Println(lilac("╚═══════════════════════════════════════╝"))
}

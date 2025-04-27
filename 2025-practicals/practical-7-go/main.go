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
            sendCommand(conn, reader, "QUIT")
            fmt.Println("Goodbye!")
            return

        default:
            fmt.Println("Invalid choice, try again.")
        }
    }
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
    fmt.Println(lilac("║ 8. Quit                               ║"))
    fmt.Println(lilac("║                                       ║"))
    fmt.Println(lilac("╚═══════════════════════════════════════╝"))
}

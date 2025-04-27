package main

import (
    "fmt"
    "net"
    "strings"
    "github.com/google/uuid"
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

type Email struct {
    ID string
    Sender string
    Subject string
    Content string
}

var mailbox = map[string][]Email {
    "hayley@proton.me": {
        {
            ID: uuid.New().String(), 
            Sender: "Oh Polly <hello@em.ohpolly.com>",
            Subject: "Oh Polly Exclusives",
            Content:"If you missed them, you'll be happy to know that your favourite styles are now available on our eBay outlet.Discover archived favourites and exclusive savings before they're gone.",
        },
        {
            ID: uuid.New().String(), 
            Sender:"Absolute Fitness Group of Trainers <noreply@trainerize.com>",
            Subject:"Shelby Angelo send you a message",
            Content:"Hi Hayls, new meal plan is loaded to start by Monday. Check-ins are then scheduled for the Friday. Please see notes & feedback i left on the pdf. 16 weeks out this weekend! ",
        },
        {
            ID: uuid.New().String(), 
            Sender: "ByteByteGo <bytebytego@substack.com>",
            Subject: "Domain-Driven Design (DDD) Demystified",
            Content:"Domain-Driven Design (DDD) tries to tackle this problem head-on. At its core, DDD is a way of designing software that keeps the business domain, not the database schema or the latest framework, at the center of decision-making. It insists that engineers collaborate deeply with domain experts during the project lifecycle, not just to gather requirements once and vanish into Jira tickets. It gives teams the vocabulary, patterns, and boundaries to model complex systems without getting buried in accidental complexity.\nDDD doesn’t care whether the architecture is monolithic or microservice-based. What it does care about is whether the model reflects the real-world rules and language of the domain, and whether that model can evolve safely as the domain changes.In this article, we explore the core ideas of DDD (such as Bounded Contexts, Aggregates, and Ubiquitous Language) and walk through how they work together in practice.  We will also look at how DDD fits into real-world systems, where it shines, and where it can fall flat.",
        },
        {
            ID: uuid.New().String(), 
            Sender: "Gold's Gym<connect@glofox.com>",
            Subject: "Happy birthday Hayley 🏋️ 🎂",
            Content:"Hey Hayley,Happy Birthday, and here's to pushing the boundaries together in another year of phenomenal workouts!\nYour journey with Gold's Gym has made us stronger as a community. 🎉 On your special day, we'd like to add a little more strength and energy to yours!\nSwing by our Reception or Coffee Bar with this message, and let us treat you to a complimentary protein shake or coffee.\nIf you are not in the gym today use this promo code to get 15% off a class package GGBDAY25 Stay strong,Gold's Gym Sandton Team 💪🎂",
        },
        {
            ID: uuid.New().String(), 
            Sender: "Drop <drop@info.drop.com>",
            Subject: "Get Your Hands on These Starfield Artifacts",
            Content:"When the galactic journey of a lifetime lands at your fingertips, all you have to do is climb aboard. Constellation is calling-and so are two new Starfield typing staples.",
        },
        {
            ID: uuid.New().String(), 
            Sender: "Marc at Frontend Masters<marc@frontendmasters.com>",
            Subject: "Complete Go for Professional Developers",
            Content:"Build a backend application from scratch that scales in Go.\nHi Hayley,\nGo is one of the fastest-growing programming languages in 2025, powering everything from high-traffic websites to critical infrastructure at companies like Google, Uber, Dropbox, PayPal – and even right here at Frontend Masters. We’ve been using Go ourselves for years and couldn’t be happier with how well it scales for our team. The code we wrote years ago is still fast, stable, and backwards compatible, proving just how reliable and future-proof Go really is!\nThat’s why we’re stoked to announce our newest course, Complete Go for Professional Developers, taught by Melkey (ML Infrastructure Engineer at Twitch) who uses Go daily. This is the most comprehensive Go course we’ve ever released, guiding you from core language fundamentals all the way to developing a full-featured backend application with Go, Docker, and Postgres.What will you learn?\nBuild a complete HTTP server from scratch with proper routing using the Chi package\nSet up and connect to a PostgreSQL database running in Docker\nImplement database migrations and a robust data layer using the pgx driver\nDesign and build comprehensive API endpoints for a complete CRUD application\nDevelop a practical multi-tiered service with proper architecture\nImplement secure user authentication with password hashing and JSON Web Tokens\nCreate middleware for protecting routes and validating user ownership\nWrite and run comprehensive unit tests with a dedicated test database\nApply professional best practices for structuring and organizing Go applications\n\nBy the end of this course, you’ll know how to build, test, and scale Go applications with confidence—using the same tools and workflows trusted by top tech companies. This isn’t just Go syntax; it’s professional, production-style Go development.\n\nWe can't wait to see what you build with Go. Jump into Complete Go for Professional Developers and get started today!",
        },
        {
            ID: uuid.New().String(), 
            Sender: "Audible <do-not-reply@audible.com>",
            Subject: "You've got 3 credits!",
            Content:"Your Credit Summary=20\n* Hi, Hayley *\nCheck out your monthly credit summary below.\nYou have 3 credits available\nYour next credit arrives on 05/23/2025\nhttps://www.audible.com/account/credits",
        },
    },
}
func handleClient(conn net.Conn) {
    defer conn.Close()
    conn.Write([]byte("+OK POP3 server ready\r\n"))

    buffer := make([]byte,4096)

    // Variables to track state
    authenticated := false
    var username string
    deleted := make(map[int]bool)

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

        case "LIST":
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate fist\r\n"))
                continue
            }
            emails, exists := mailbox[username]
            if !exists || len(emails) == 0 {
                conn.Write([]byte("+Ok 0 messages\r\n.\r\n"))
                continue
            }

            var response []byte
            response = fmt.Appendf(response,"+OK %d messages\r\n", len(emails))
            for idx,email := range emails {
                response = fmt.Appendf(response, "%d %d\r\n", idx+1, len(email.Content))
            }
            response = fmt.Appendf(response, ".\r\n")
            conn.Write(response)

        case "RETR":
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate fist\r\n"))
                continue
            }
            emails, exists := mailbox[username]
            if !exists {
                conn.Write([]byte("-ERR No such user mailbox\r\n"))
                continue
            }
            if arg == "" {
                conn.Write([]byte("-ERR Message number required\r\n"))
                continue
            }
            msgIdx := -1
            fmt.Sscanf(arg,"%d", &msgIdx)
            if msgIdx < 1 || msgIdx > len(emails) {
                conn.Write([]byte("-ERR no such message\r\n"))
                continue
            }
            email := emails[msgIdx-1]
            var response []byte

            response = fmt.Appendf(response,"+OK %d octets\r\n",len(email.Content))
            response = fmt.Appendf(response, "%s\r\n", email.Sender)
            response = fmt.Appendf(response, "%s\r\n", email.Subject)
            response = fmt.Appendf(response, "%s\r\n.\r\n", email.Content)
            conn.Write(response)

        case "DELE":
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            }
            if arg == "" {
                conn.Write([]byte("-ERR Message number required\r\n"))
                continue
            }
            msgIdx := -1
            fmt.Sscanf(arg,"%d", &msgIdx)
            if(msgIdx < 1) {
                conn.Write([]byte("-ERR No such message\r\n"))
                continue
            }
            if deleted[msgIdx] {
                conn.Write([]byte("-ERR Message already deleted\r\n"))
                continue
            } else {
                deleted[msgIdx] = true
                conn.Write([]byte("+OK Message marked for deletion\r\n"))
            }

        case "RESET":
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            }
            deleted = make(map[int]bool)
            conn.Write([]byte("+OK Reset state\r\n"))

        case "NOOP":
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            }
            conn.Write([]byte("+OK\r\n"))

        case "STAT":
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            }
            emails, exists := mailbox[username]
            if !exists {
                conn.Write([]byte("-ERR No mailbox found\r\n"))
                continue
            }
            count := 0
            totalSize := 0
            for idx, email := range emails {
                if !deleted[idx+1] {
                    count++
                    totalSize += len(email.Content)
                }
            }
            var response []byte
            response = fmt.Appendf(response, "+OK %d %d\r\n",count, totalSize)
            conn.Write(response)

        case "UIDL":
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            }
            emails, exists := mailbox[username]
            if !exists || len(emails) == 0{
                conn.Write([]byte("-OK 0 messages\r\n"))
                continue
            }
            var response []byte
            response = fmt.Appendf(response,"+OK Unique-ID listing\r\n")
            for idx, email := range emails {
                if !deleted[idx+1] {
                    response = fmt.Appendf(response,"%d %s\r\n", idx+1, email.ID)
                }
            }
            response = fmt.Appendf(response,".\r\n")
            conn.Write(response)

        case "QUIT":
            if !authenticated {
                conn.Write([]byte("+OK Goodbye\r\n"))
                return
            }
            emails, exists := mailbox[username]
            if exists {
                var newEmails []Email
                for idx, email := range emails {
                    if !deleted[idx+1] {
                        newEmails = append(newEmails, email)
                    }
                }
                mailbox[username] = newEmails
            }
            conn.Write([]byte("+OK Goodbye\r\n"))
            return

        default:
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            } 
            conn.Write([]byte("-ERR Unknown command\r\n"))
        }
    }
}



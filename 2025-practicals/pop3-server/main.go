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
    Content string
}

var mailbox = map[string][]Email {
    "hayley@proton.me": {
        {ID: uuid.New().String(), Content:"Subject:Oh Polly Exclusives: Now on eBay\r\n\r\nIf you missed them, you'll be happy to know that your favourite styles are now available on our eBay outlet.Discover archived favourites and exclusive savings before they're gone."},
        {ID: uuid.New().String(), Content:"Subject:Shelby Angelo send you a message\r\n\r\nHi Hayls, new meal plan is loaded to start by Monday. Check-ins are then scheduled for the Friday. Please see notes & feedback i left on the pdf. 16 weeks out this weekend! "},
        {ID: uuid.New().String(), Content:"Subject:Domain-Driven Design (DDD) Demystified\r\n\r\nDomain-Driven Design (DDD) tries to tackle this problem head-on. At its core, DDD is a way of designing software that keeps the business domain, not the database schema or the latest framework, at the center of decision-making. It insists that engineers collaborate deeply with domain experts during the project lifecycle, not just to gather requirements once and vanish into Jira tickets. It gives teams the vocabulary, patterns, and boundaries to model complex systems without getting buried in accidental complexity.\nDDD doesn’t care whether the architecture is monolithic or microservice-based. What it does care about is whether the model reflects the real-world rules and language of the domain, and whether that model can evolve safely as the domain changes.In this article, we explore the core ideas of DDD (such as Bounded Contexts, Aggregates, and Ubiquitous Language) and walk through how they work together in practice.  We will also look at how DDD fits into real-world systems, where it shines, and where it can fall flat."},
        {ID: uuid.New().String(), Content:"Subject:HITMAN is coming to Nintendo Switch 2 🍄\r\n\r\nAgent 47, get ready for the release of HITMAN World of Assassination: Signature Edition on Nintendo Switch 2.\nThis new experience will be available on June 5, running natively on the console.Pre-order now to get two exclusive elegant suits, a beautiful gold-plated wrench and a tasty looking mushroom—although some believe they can make you grow, it would be ill advised to eat this one..."},
        {ID: uuid.New().String(), Content:"Subject:Get Your Hands on These Starfield Artifacts\r\n\r\nWhen the galactic journey of a lifetime lands at your fingertips, all you have to do is climb aboard. Constellation is calling-and so are two new Starfield typing staples."},
        {ID: uuid.New().String(), Content:"Subject:Please don't miss the TypeScript course launch, we have a discount for you\r\n\r\n We're adding more to Boot.dev than ever before. Hello, Hayley,Our TypeScript Course launch is almost here! That means every single course in our backend developer learning path that could be completed in Go, can now be completed in TypeScript. That's eight brand new courses to teach you all the same backend concepts using a new programming language. Learn JavaScript (a prerequisite), Learn TypeScript, Learn HTTP Clients (in TypeScript), Build a Pokedex (in TypeScript),Build a Blog Aggregator (in TypeScript),Learn HTTP Servers (in TypeScript),Learn File Servers and CDNs (in TypeScript), Learn CI/CD (in TypeScript) And Boot.dev will just keep getting bigger. It's the perfect time to subscribe to Boot.dev! A discount code and instructions will be sent via email on the day of the launch. To make sure you don’t miss it, add us to your Gmail contacts so it has no chance of getting lost in spam/promotions:Open Gmail on your computer.Click the Google apps button at the top-right (it's next to your account icon and looks like 9 dots). Click Contacts.At the top-left of the screen, click create contact.Enter hello@boot.dev, and click Save!Parsimoniously,PS: If this email was in promotions, just right-click on it from the promotions tab and select “move to tab → primary”."},
        {ID: uuid.New().String(), Content:"Subject:You've got 3 credits!\r\n\r\nYour Credit Summary Hi, Hayley,Check out your monthly credit summary below."},
    },
}
func handleClient(conn net.Conn) {
    defer conn.Close()
    conn.Write([]byte("+OK POP3 server ready\r\n"))

    buffer := make([]byte,4096)

    // Variables to track state
    authenticated := false
    var username string

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
            response = fmt.Appendf(response, "%s\r\n.\r\n", email.Content)
            conn.Write(response)

        case "QUIT":
            conn.Write([]byte("+OK Goodbye\r\n"))
            return

        default:
            if !authenticated {
                conn.Write([]byte("-ERR Authenticate first\r\n"))
                continue
            } 
            //conn.Write([]byte("-ERR Unknown command\r\n"))
        }

        if strings.HasPrefix(strings.ToUpper(input),"QUIT") {
            conn.Write([]byte("+ OK Goodbye\r\n"))
            return
        } else {
            conn.Write([]byte("-ERR Unknown command\r\n"))
        }
    }
}



# COS 332 Practical Assignments
![](https://img.shields.io/badge/rust_powered%F0%9F%A6%80%E2%9C%A8-orange)
![](https://img.shields.io/badge/Ocaml_Pattern_Match_Wizardry%F0%9F%90%AB-pink)
![](https://img.shields.io/badge/Go_Simpler_Than_Rust,_But_So_Is_a_Rock-blue)
---

This repository contains my completed practical assignments for the University of Pretoria’s Computer Networks (COS 332) module.

All practicals (except the last one) were implemented in Rust to align with the course’s emphasis on low-level networking protocol handling and explicit socket programming. The last two practicals (7 and 8) were implemented in Go to explore concurrency patterns and ease of building network tools.

**Warning**
If you see Go code here, pretend you don’t. I promise you, deep down, every line wishes it had the fierce protection of a borrow checker 🦀.

---

## Repository Structure

```
/2025-practicals
    /practical-1            # Practical 1: CGI server with dynamic time zone switching (Rust)
    /practical-2            # Practical 2: Telnet server managing a friends database (Rust)
    /practical-3            # Practical 3: HTTP calculator server (Rust)
    /practical-4            # Practical 4: HTTP server with form input for a phonebook (Rust)
    /practical-5-ocaml      # Practical 5: LDAP client to query friends directory (OCaml)
    /practical-6            # Practical 6: SMTP email sender simulating an alarm system (Rust)
    /practical-7-go         # Practical 7: POP3 mailbox manager (Go)
    /practical-8            # Practical 8: FTP file monitor and uploader (Go, pair project)
```

---

## Technologies Used
- **Rust:** For fearless, zero-cost abstractions, and the unmatched joy of watching the borrow checker let your code compile.
- **OCaml:** For pattern matching your way to glory.
- **Go:** For the moments when I whispered, “forgive me, Ferris, I have no time.”
- **Linux tools:** For running servers like a proper hacker.
- **Markdown/HTML:** For documenting it all.
---

## Highlights

- No use of high-level libraries for protocol handling. All protocols (HTTP, Telnet, FTP, SMTP, POP3, LDAP) were implemented using raw sockets and manual protocol construction as per assignment rules.

- Strong adherence to RFC specifications for each protocol.

- Demonstrations included realistic environments using local virtual machines or SBCs (Raspberry Pi), where applicable.

- Last practical (Prac 8) was a collaborative project where we monitored file changes and automatically uploaded them via an FTP client. 

---

## Collaborators

- Practical 8 (FTP Monitor): Pair project with [PhillDup](https://github.com/philldup).
- All other practicals: Individually completed.

---

## Setup Instructions

Each practical folder contains:
- `README.md` — Specific setup and run instructions.
- `src/` — Source code.
- `Makefile` or `Cargo.toml` — Build configuration.
- `docs/` — Any additional documentation or protocol explanations

General steps:

```bash
# Navigate to a practical folder
cd 2025-practicals/practical-X

# Build
cargo build --release   # Rust projects
go build                # Go projects
dune build              # OCaml project

# Run
./target/release/practical_X  # Rust binaries
./_build/default/practical_X.exe  # OCaml binaries
./practical_X                 # Go binaries
```

---

## Disclaimer 🦀

Yes, there’s Go here. Yes, it works.
But know that every time I compiled a Go file, a tiny voice in my head whispered *"You could've written this in Rust."*.
Fast, safe, and fearless concurrentcy. That’s the Rust promise. The Go parts...Let’s just call them temporary lapses in judgment.

## License

This repository is licensed under the MIT License.

```
MIT License

Copyright (c) 2025 Hayley Dodkins

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

If you use or learn from this repository, please respect the educational context and do not submit this work as your own.

---

```
       _~^~^~_
   \) /  o o  \ (/
     '_   -   _'
     / '-----' \
 
Ferris reminds you:
- Use Rust.
- Respect the borrow checker.
- Ship safe, fast, fearless code.

The GO Gopher sees your slander and he will not stand for it...
The Go Gopher knows all
The Go Gopher never forgets
         ,_---~~~~~----._         
  _,,_,*^____      _____``*g*\"*, 
 / __/ /'     ^.  /      \ ^@q   f 
[  @f | @))    |  | @))   l  0 _/  
 \`/   \~____ / __ \_____/    \   
  |           _l__l_           I   
  }          [______]           I  
  ]            | | |            |  
  ]             ~ ~             |  
  |                            |   
   |                           |   

(bigger cause better)
```

# Practical 7
Create a program that warns you when you receive an email as a BCC recipient.
This tool should work as follows:

### 1. Monitor the Inbox (POP3):
- Use the `POP3 protocol` to connect to the mail server and download emails.
- Download only the headers for efficientcy.

### 2. Detect BCC Header:
- Look for a `BCC:` header in the email.
- Your email address is expected to appear in this header if you were a BCC recipient.
- Be mindful of **false positives** (e.g., text like "BCC:" appearing in the email body).

### 3. Send a Warning:
- If a `BCC:` header is detected, the program sends a **warning email** to your account.
- The warning email should have a subject like `BCC Warning <original email subject>` and arrive in your inbox before or alongside the flagged email.

### 4. Handle Edge Cases:
- Automatic forwarding might modify headers, so **consider how a BCC would appear in forwarded emails**.
- If a mailing list is the BCC recipient, be prepared for unusual header structures.


## Architecture
### 1. email_client
- handles POP3 connections.
- download and parse emails.

### 2. bcc_detector
- checks for the BCC headers and validates emial addresses.

### 3. warning_sender 
- composes and sends warning emails

### 4. utils
- includes helpers for decoding headers, handling encoding and logging.

### 5. cli
- provides a command line interface for user configuration

## Additional Tasks
### 1. Parsing Headers
- **Header Folding** handle multi-line headers by detecting and merging lines that start with whitespace.
```
Subject: This is a folded
header line
```
- **Quoted Strings:** Support for quoted strings and special characters in headeres
```
From "John Done" <john.doe@rmail.com>
From John Done <john.doe@rmail.com>
```
- **Angle bracket addresses** parse the enclosed `<>` correctly.

### 2. Boundry Confitions
Emails omit the BCC field in headers for recipients listed as BCC, todetect this:
- Analaze discrepancies between `TO` and `Recieved` headers to infer if you are the BCC recipient.
- If your address is not in the `To` or `CC` but the email is in the inbox assume its a BCC

### 3. Parsing Encoded Emails
- Decode Base64 or other encodings.

```
Content-Transfer-Encoding: base64
Content-Type: text/plain; charset=utf-8

U29tZSBibGluZCBjYXJib24gY29weSBtZXNzYWdlLg==
```
- BCC header may be encoded



### 4. Skip Forwarded Emails
- Identify by inspecting Recieved and Resent headers to see if the email was forwarded and skip that warning

### 5. 

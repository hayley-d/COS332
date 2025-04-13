## Practical 6
In this assignment, the goal is to modify a previous system (which was based on multiple-choice questions) to email the results to a specified recipient. 
This means when the user completes the test, instead of just displaying the results, you need to programmatically send the results via email.

## TO RUN
```bash
cargo run -- --target-email 2002dodhay@gmail.com
```

### Key Tasks:

#### 1. Email Results
After completing the test, your system should allow the results to be sent via email. For example, if you're working with a web application, you could add a button that triggers the sending of the results. If you are using a Telnet-based solution, you need to include a command that sends the results to an email address.

#### 2. SMTP Implementation
You must implement the email-sending functionality using native SMTP (Simple Mail Transfer Protocol) commands. 
This means you need to manually send the email using raw SMTP commands rather than relying on existing email libraries or services (like Gmail API or sendmail). 
You should familiarize yourself with the basics of SMTP commands and the email format as specified by RFC 5321 (SMTP) and RFC 5322 (email format).

#### 3. Email Recipient
You need to make provisions to input the email address to which the results will be sent. 
For testing purposes, this could be your own email address or an email you control, and you should show that the email successfully reaches the inbox.

### Example Workflow
1. The test is completed by the user.
2. The user presses a button (in a web app) or triggers a command (in a Telnet solution).
3. The program then creates an email with the results.
4. The email is sent using native SMTP commands, ensuring proper email formatting and headers.
    - connect to an SMTP server and send a porper formated email.
5. The email reaches the recipient’s inbox, and you can demonstrate this by showing the test results in your email.

### Advanced Feautres


### Task
You will write an **SMTP client in Rust** that sends an email. 
This involves connecting to an SMTP server (like Gmail’s SMTP server) and following the SMTP protocol to send the email.

#### TODO
1. **Establish a Connection to an SMTP Server**
2. Send AUTH Command
3. Send Email
4. Close Connection

Additionals
* Using TLS for secure connection
* Using AUTH CRAM-MD5 hashing based challenged response mechanism for authentification
* Support attatchments such as pdf through base64 encoding 
* Upgrade the SMTP connection using STARTTLS
* Support UTF-8 Encoding
* Ensure that your MAIL FROM and Return-Path domains align with SPF and DMARC policies to prevent rejection or flagging as spam.
* Parse server responses for delivery confirmations (e.g., 250 2.0.0 OK) and provide the user with feedback.
* Implement DKIM signing to add a cryptographic signature to your email headers, ensuring authenticity and integrity.


This assignment involves using the Lightweight Directory Access Protocol (LDAP) in a non-typical way to store DNS-related information. Let's break it down step-by-step:

### Typical LDAP Directory Structure:
LDAP is commonly used for directory services, often organized hierarchically. A typical directory may have countries at the root, with organizations, departments, and employee details under them.
However, you don't need to use all of these levels in this assignment. For example, a company might have the root as the company itself and place departments below it directly.

#### The DNS Tree:
- The DNS (Domain Name System) also forms a tree-like structure, starting from the root (such as .com or .org), then moving to top-level domains (TLDs), second-level domains (like ac, co, gov), and further.
- The assignment asks you to simulate storing DNS-related information in LDAP by using the DNS structure as your directory tree.

#### Storing DNS Information in LDAP:

- In this educational exercise, you will store DNS data in LDAP, specifically for organizations within a .za (South Africa) top-level domain (TLD). The root of your LDAP tree will be the .za TLD.
- The second-level domains (such as ac.za, co.za, and gov.za) will be treated as organizational units (OUs) within the LDAP directory.
- The third layer of the DNS tree (e.g., up.ac.za) will store DNS resource records such as:
    - A records: The IPv4 address.
    - NS records: Name servers.
    - MX records: Mail exchange servers, but you will only store the one with the highest priority (lowest number).

#### Populating the LDAP Directory:
- You need to create a directory structure that represents several second-level domains under .za and populate them with DNS data. For example:
    - Under ac.za, store details of organizations like up.ac.za (the University of Pretoria).
    - Record relevant DNS information: one A record (IPv4 address), one NS record (name server), and the highest-priority MX record.

#### Writing the Client:
- You will create a client that queries the LDAP directory to retrieve the DNS records for an organization.
- The client should allow you to search for an organization by its name (e.g., up.ac.za) or just the second-level domain (e.g., up).
- The client needs to establish a connection to the LDAP server on port 389, construct raw requests to query the server, and interpret the raw responses. This means you cannot use higher-level libraries that abstract the communication details. You will handle the communication at the byte or string level.

#### Example Query:
For example, if you query up.ac.za, you should be able to get the DNS records for that domain:
- A record: The IPv4 address.
- NS records: One of the name servers.
- MX record: The mail exchanger with the highest priority.

#### Key Questions:
- The assignment asks whether you can query the organization using just the second-level domain (e.g., up) or the full domain (e.g., up.ac.za). This tests whether your LDAP structure is flexible enough to allow partial queries and still return meaningful results.
- The underlying goal is to demonstrate that with the LDAP directory structure, you can manage incomplete or partial information while still retrieving correct data.

#### Summary:
This assignment challenges you to use LDAP to store DNS information and create a client that can query this information at a low level (byte/string operations). You'll simulate a DNS-like directory structure in LDAP and focus on querying DNS records for organizations within the .za TLD.


## Additional Features
### 1. Advanced LDAP Query Filters:
- LDAP Search Filters: Implementing search filters like logical AND (&), OR (|), and wildcards (*) can greatly enhance your program. For example, allowing partial searches for domain names or organization names would be a small but useful improvement.
    - Example: Searching for all records in ac.za with CN=up can be done using a filter like:
```
(&(objectClass=organization)(CN=up))
```

- Search Request: According to RFC 4511, an LDAP search operation is quite specific in terms of what data needs to be included in the request: base DN, scope (base, one-level, subtree), filter, attributes to retrieve, and size limits.
- Wildcard Matching: Allowing wildcard queries (e.g., *.ac.za) would make it easier to query the directory for any domain under ac.za without needing exact names.

### 2. Error Handling with LDAP Result Codes
- Implement detailed handling of LDAP result codes. You can catch errors from the LDAP server (e.g., noSuchObject, insufficientAccessRights) and return more meaningful messages. This not only makes the client more user-friendly but also demonstrates your understanding of LDAP's error handling.

```rust
match ldap_response.result_code {
    ResultCode::NoSuchObject => {
        eprintln!("Error: The queried object does not exist.");
    },
    ResultCode::Success => {
        println!("Query successful.");
    },
    _ => {
        eprintln!("An unexpected error occurred: {:?}", ldap_response.result_code);
    }
}
```

### 3. LDAP Bind Operations RFC 4511
- **Bind Request:** The LDAP protocol defines how clients authenticate using the BindRequest operation. You can implement this operation to handle different types of binds (simple authentication, SASL, etc.).
- **Bind Response:** Handle the BindResponse according to the protocol's result codes (e.g., success, invalidCredentials, successWithNoData).

```rust
match ldap_bind("cn=admin,dc=example,dc=com", "password") {
    Ok(response) => match response.result_code {
        ResultCode::Success => println!("Bind successful."),
        ResultCode::InvalidCredentials => eprintln!("Invalid credentials."),
        _ => eprintln!("Other bind error."),
    },
    Err(e) => eprintln!("Bind failed: {}", e),
}
```

### 4. 


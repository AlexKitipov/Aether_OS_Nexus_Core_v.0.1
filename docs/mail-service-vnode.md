# Mail/Messaging V-Node (`svc://mail-service`)

## Overview

The `mail-service` V-Node provides a centralized messaging and mail management service for AetherOS users and applications. It acts as an intermediary for sending and receiving electronic mail, managing user mailboxes, and interacting with network mail protocols (SMTP, POP3, IMAP) through the `svc://socket-api` V-Node.

This V-Node uses `svc://vfs` for persistent mailbox and message storage, preserving user data integrity and control.

## IPC Protocol

Communication with `svc://mail-service` happens over IPC using the `MailRequest` and `MailResponse` enums defined in `src/ipc/mail_ipc.rs`.

### `MailRequest` enum (Client → `mail-service`)

Client V-Nodes (for example, a mail client UI) send requests to `svc://mail-service` to trigger mail operations.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum MailRequest {
    /// Send a new mail message.
    SendMail {
        recipient: String,
        subject: String,
        body: String,
    },
    /// List available mailboxes for the current user.
    ListMailboxes,
    /// Read a specific message from a given mailbox.
    ReadMessage {
        mailbox: String,
        message_id: u32,
    },
}
```

**Parameters**

- `recipient`: Email address of the recipient.
- `subject`: Subject line of the email.
- `body`: Main content of the email.
- `mailbox`: Mailbox name, such as `Inbox` or `Sent`.
- `message_id`: Unique message identifier within a mailbox.

### `MailResponse` enum (`mail-service` → Client)

`svc://mail-service` returns one of these responses after processing a `MailRequest`.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum MailResponse {
    /// Indicates a successful operation, with an optional descriptive message.
    Success(String),
    /// Returns a list of mailbox names.
    Mailboxes(Vec<String>),
    /// Returns the content of a specific message.
    Message(String),
    /// Indicates an error occurred during the operation.
    Error(String),
}
```

**Return values**

- `Success(String)`: Operation succeeded with a human-readable status message.
- `Mailboxes(Vec<String>)`: List of available mailbox names.
- `Message(String)`: Full content of a requested message.
- `Error(String)`: Operation failed with an error description.

## Functionality

The `mail-service` V-Node provides the following capabilities:

1. **IPC interface**: Exposes a clear request/response contract for mail actions.
2. **Mailbox management**: Handles mailbox organization (for example, `Inbox`, `Sent`, `Drafts`), conceptually backed by VFS under `/home/<AID>/mail/`.
3. **Message storage**: Persists and retrieves message files inside mailbox directories.
4. **Network integration**: Uses `svc://socket-api` for SMTP/POP3/IMAP transport and `svc://dns-resolver` for mail server host lookups.
5. **Error handling**: Converts lower-level VFS/network errors into `MailResponse::Error`.
6. **User context**: Integrates with Aether Identity (AID) for user-scoped mailbox storage and server authentication flows.

## Usage examples

### Example 1: Sending a mail message

```rust
// Pseudocode for a client V-Node (for example, a mail client GUI)

let mut mail_chan = VNodeChannel::new(10); // IPC Channel to svc://mail-service

let request = MailRequest::SendMail {
    recipient: String::from("user@example.com"),
    subject: String::from("Hello AetherOS!"),
    body: String::from("This is a test message from AetherOS."),
};
match mail_chan.send_and_recv::<MailRequest, MailResponse>(&request) {
    Ok(MailResponse::Success(msg)) => {
        log!("Mail sent successfully: {}", msg);
    },
    Ok(MailResponse::Error(msg)) => {
        log!("Failed to send mail: {}", msg);
    },
    _ => log!("Unexpected response from Mail Service"),
}
```

### Example 2: Listing mailboxes

```rust
// Pseudocode for a client V-Node wanting to list available mailboxes

let mut mail_chan = VNodeChannel::new(10);

let request = MailRequest::ListMailboxes;
match mail_chan.send_and_recv::<MailRequest, MailResponse>(&request) {
    Ok(MailResponse::Mailboxes(mailboxes)) => {
        log!("Available mailboxes: {:?}", mailboxes);
    },
    Ok(MailResponse::Error(msg)) => {
        log!("Failed to list mailboxes: {}", msg);
    },
    _ => log!("Unexpected response from Mail Service"),
}
```

### Example 3: Reading a message

```rust
// Pseudocode for a client V-Node wanting to read a specific message

let mut mail_chan = VNodeChannel::new(10);

let request = MailRequest::ReadMessage {
    mailbox: String::from("Inbox"),
    message_id: 1,
};
match mail_chan.send_and_recv::<MailRequest, MailResponse>(&request) {
    Ok(MailResponse::Message(content)) => {
        log!("Message content:\n{}", content);
    },
    Ok(MailResponse::Error(msg)) => {
        log!("Failed to read message: {}", msg);
    },
    _ => log!("Unexpected response from Mail Service"),
}
```

This V-Node acts as a core communication hub and demonstrates AetherOS modularity, where higher-level user features are composed from secure IPC interactions between specialized services.

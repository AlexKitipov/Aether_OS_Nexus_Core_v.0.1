# Shell V-Node (`svc://shell`)

## Overview

The `shell` V-Node serves as the primary command-line interpreter for AetherOS, analogous to `bash` or `zsh` in traditional operating systems. It provides an interactive interface for users and other V-Nodes to execute commands, manage files, interact with system services, and perform network lookups.

Designed with the microkernel philosophy, the `shell` V-Node remains a thin orchestration layer and delegates complex operations to specialized V-Nodes via IPC.

## IPC Protocol

Communication with the `shell` V-Node occurs via IPC using the `ShellRequest` and `ShellResponse` enums (located at `src/ipc/shell_ipc.rs` in the planned workspace layout).

### `ShellRequest` (Client -> shell)

Client V-Nodes (for example, `AetherTerminal`) send these requests to `svc://shell` to execute commands or query shell state.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ShellRequest {
    /// Request to execute a command with its arguments.
    ExecuteCommand { command: String, args: Vec<String> },
    /// Request to change the current working directory.
    ChangeDirectory { path: String },
    /// Request to get the current working directory.
    GetCurrentDirectory,
}
```

#### Parameters

- `command`: Name of the command to execute (for example: `"ls"`, `"cd"`, `"ping"`, `"start"`).
- `args`: Command arguments.
- `path`: Target path for directory operations.

### `ShellResponse` (shell -> Client)

After processing a request, `svc://shell` replies with one of the following responses:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ShellResponse {
    /// Successful execution of a command, with its output and exit code.
    CommandOutput { stdout: String, stderr: String, exit_code: i32 },
    /// Indicates a successful operation without specific output.
    Success(String),
    /// Returns the current working directory.
    CurrentDirectory(String),
    /// Indicates an error occurred during the operation.
    Error(String),
}
```

#### Return values

- `CommandOutput { stdout, stderr, exit_code }`: Standard output, standard error, and process exit code.
- `Success(String)`: Operation completed successfully with a message.
- `CurrentDirectory(String)`: Current shell working directory.
- `Error(String)`: Operation failed with a descriptive message.

## Functionality

The `shell` V-Node provides the following core capabilities:

1. **Command execution**: Parses and executes commands from `ExecuteCommand` requests.
2. **Built-in commands**:
   - `cd <path>`: Changes working directory (via `svc://vfs`).
   - `ls`: Lists directory contents (via `svc://vfs`).
   - `ping <hostname>`: Performs hostname-based reachability testing (via `svc://dns-resolver`).
   - `start <service_name>`: Starts another V-Node (via `svc://init-service`).
3. **V-Node interaction over IPC**:
   - `svc://vfs` for filesystem operations.
   - `svc://init-service` for lifecycle management.
   - `svc://dns-resolver` for hostname resolution.
4. **Current working directory state**: Maintains and updates `current_dir`.
5. **Command history**: Tracks previously executed commands.

## Usage examples

### Example 1: Executing `ls`

```rust
// Pseudocode for a client V-Node (e.g., AetherTerminal)

let mut shell_chan = VNodeChannel::new(8);

let request = ShellRequest::ExecuteCommand {
    command: String::from("ls"),
    args: Vec::new(),
};

match shell_chan.send_and_recv::<ShellRequest, ShellResponse>(&request) {
    Ok(ShellResponse::CommandOutput { stdout, stderr, exit_code }) => {
        log!("ls stdout:\n{}", stdout);
        if !stderr.is_empty() {
            log!("ls stderr:\n{}", stderr);
        }
        log!("ls exit code: {}", exit_code);
    }
    Ok(ShellResponse::Error(msg)) => {
        log!("ls command error: {}", msg);
    }
    _ => log!("Unexpected response from Shell"),
}
```

### Example 2: Changing directory (`cd`)

```rust
// Pseudocode for changing the shell working directory

let mut shell_chan = VNodeChannel::new(8);

let request = ShellRequest::ChangeDirectory {
    path: String::from("/home/user/documents"),
};

match shell_chan.send_and_recv::<ShellRequest, ShellResponse>(&request) {
    Ok(ShellResponse::Success(msg)) => {
        log!("cd successful: {}", msg);
    }
    Ok(ShellResponse::Error(msg)) => {
        log!("cd command error: {}", msg);
    }
    _ => log!("Unexpected response from Shell"),
}

// Optionally, confirm by querying cwd
let get_cwd_request = ShellRequest::GetCurrentDirectory;
match shell_chan.send_and_recv::<ShellRequest, ShellResponse>(&get_cwd_request) {
    Ok(ShellResponse::CurrentDirectory(cwd)) => {
        log!("Current working directory: {}", cwd);
    }
    _ => log!("Failed to get current directory"),
}
```

### Example 3: Pinging a hostname

```rust
// Pseudocode for a ping command

let mut shell_chan = VNodeChannel::new(8);

let request = ShellRequest::ExecuteCommand {
    command: String::from("ping"),
    args: vec![String::from("example.com")],
};

match shell_chan.send_and_recv::<ShellRequest, ShellResponse>(&request) {
    Ok(ShellResponse::CommandOutput { stdout, stderr, exit_code }) => {
        log!("ping stdout:\n{}", stdout);
        if !stderr.is_empty() {
            log!("ping stderr:\n{}", stderr);
        }
        log!("ping exit code: {}", exit_code);
    }
    Ok(ShellResponse::Error(msg)) => {
        log!("ping command error: {}", msg);
    }
    _ => log!("Unexpected response from Shell"),
}
```

## Notes

This document captures the service contract for `svc://shell` and reflects the AetherOS design principle that user-facing behavior is composed through explicit IPC with specialized V-Nodes.

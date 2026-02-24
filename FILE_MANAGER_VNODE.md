# File Manager V-Node (`svc://file-manager`)

## Overview

The `file-manager` V-Node provides a high-level API for managing files and directories within AetherOS. It can be used directly by client V-Nodes (such as a GUI file explorer) or serve as a backend service for higher-level tooling.

Rather than exposing raw filesystem mechanics, `svc://file-manager` abstracts common operations (browse, copy, move, delete, create directory) and delegates low-level I/O to `svc://vfs`.

## IPC Protocol

The `file-manager` service communicates through IPC using the `FileManagerRequest` and `FileManagerResponse` enums (intended path: `src/ipc/file_manager_ipc.rs`).

### `FileManagerRequest` (client ➜ `file-manager`)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum FileManagerRequest {
    /// Browse the contents of a directory.
    Browse { path: String },
    /// Copy a file or directory.
    Copy { source: String, destination: String },
    /// Move a file or directory.
    Move { source: String, destination: String },
    /// Delete a file or directory.
    Delete { path: String },
    /// Create a new directory.
    CreateDirectory { path: String },
}
```

**Parameters**

- `path`: Absolute path for `Browse`, `Delete`, or `CreateDirectory`.
- `source`: Absolute source path for `Copy` and `Move`.
- `destination`: Absolute destination path for `Copy` and `Move`.

### `FileManagerResponse` (`file-manager` ➜ client)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum FileManagerResponse {
    /// Indicates a successful operation, with an optional descriptive message.
    Success(String),
    /// Indicates an error occurred during the operation.
    Error(String),
    /// Returns a list of directory entries (name, metadata).
    DirectoryEntries(BTreeMap<String, VfsMetadata>),
}
```

**Return variants**

- `Success(String)`: Operation succeeded (example: "File copied successfully").
- `Error(String)`: Operation failed with a descriptive message.
- `DirectoryEntries(BTreeMap<String, VfsMetadata>)`: Returned for successful `Browse` requests.

## Core Responsibilities

1. **IPC entrypoint** for user-facing file management actions.
2. **VFS orchestration** by issuing requests to `svc://vfs` for all filesystem mutations and reads.
3. **Composite operations** for actions like recursive copy/move using a sequence of VFS operations.
4. **Error translation** from low-level VFS failures into user-comprehensible responses.
5. **Future user-context integration** (AID/permissions-aware behavior).

## Operational Flow

### Browse flow

1. Client sends `FileManagerRequest::Browse { path }`.
2. `file-manager` validates path shape/policy.
3. Service requests directory listing from `svc://vfs`.
4. On success, service returns `FileManagerResponse::DirectoryEntries(entries)`.
5. On failure, service returns `FileManagerResponse::Error(reason)`.

### Copy flow

1. Client sends `FileManagerRequest::Copy { source, destination }`.
2. `file-manager` requests metadata and content traversal from `svc://vfs`.
3. Service writes content to destination via `svc://vfs`.
4. On success, returns `FileManagerResponse::Success(msg)`.
5. On failure, returns `FileManagerResponse::Error(reason)`.

### Move flow

- Preferred strategy: delegate to an atomic/rename-like VFS operation when possible.
- Fallback strategy: copy + source delete (with failure handling for partial progress).

## Usage Examples

### Example 1: Browse a directory

```rust
let mut file_manager_chan = VNodeChannel::new(9); // IPC channel to svc://file-manager

let request = FileManagerRequest::Browse {
    path: String::from("/home/user/documents"),
};

match file_manager_chan.send_and_recv::<FileManagerRequest, FileManagerResponse>(&request) {
    Ok(FileManagerResponse::DirectoryEntries(entries)) => {
        log!("Contents of /home/user/documents:");
        for (name, metadata) in entries {
            log!(
                "- {}: {} ({} bytes)",
                name,
                if metadata.is_dir { "Dir" } else { "File" },
                metadata.size
            );
        }
    }
    Ok(FileManagerResponse::Error(msg)) => {
        log!("Failed to browse directory: {}", msg);
    }
    _ => log!("Unexpected response from file-manager"),
}
```

### Example 2: Copy a file

```rust
let mut file_manager_chan = VNodeChannel::new(9);

let request = FileManagerRequest::Copy {
    source: String::from("/home/user/document.txt"),
    destination: String::from("/home/user/backups/document.txt"),
};

match file_manager_chan.send_and_recv::<FileManagerRequest, FileManagerResponse>(&request) {
    Ok(FileManagerResponse::Success(msg)) => {
        log!("File copy successful: {}", msg);
    }
    Ok(FileManagerResponse::Error(msg)) => {
        log!("File copy failed: {}", msg);
    }
    _ => log!("Unexpected response from file-manager"),
}
```

## Notes

- This document specifies the IPC contract and expected behavior for a `file-manager` service.
- It does not implement the service itself in this repository state.

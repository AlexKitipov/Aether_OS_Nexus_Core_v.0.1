# Virtual File System (VFS) V-Node (`svc://vfs`)

## Overview

The `vfs` V-Node is the central entry point for file system interactions in AetherOS. It exposes a unified IPC API so client V-Nodes can open, read, write, list, and stat paths without coupling to a specific storage backend (AetherFS, RAM disk, block device drivers, etc.).

## IPC Protocol

Protocol types are defined in `src/ipc/vfs_ipc.rs`.

### File Descriptor

```rust
pub type Fd = u32;
```

`Fd` identifies an open file or directory managed by the `vfs` service, similar to Unix file descriptors.

### `VfsMetadata`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct VfsMetadata {
    pub is_dir: bool,
    pub size: u64,
    pub created: u64,   // Unix timestamp
    pub modified: u64,  // Unix timestamp
    pub permissions: u32, // e.g., 0o755
}
```

### `VfsRequest` (Client → `svc://vfs`)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum VfsRequest {
    /// Open a file or directory.
    Open { path: String, flags: u32 },
    /// Read bytes from an open descriptor.
    Read { fd: Fd, len: u32, offset: u64 },
    /// Write bytes to an open descriptor.
    Write { fd: Fd, data: Vec<u8>, offset: u64 },
    /// List directory entries at a path.
    List { path: String },
    /// Get metadata for a path.
    Stat { path: String },
    /// Close an open descriptor.
    Close { fd: Fd },
}
```

#### Request parameters

- `path`: absolute filesystem path.
- `flags`: open mode bitfield (e.g., read-only, write-only, create).
- `fd`: descriptor returned by `Open`.
- `len`: maximum byte count for `Read`.
- `offset`: byte offset used by `Read` / `Write`.
- `data`: bytes payload for `Write`.

### `VfsResponse` (`svc://vfs` → Client)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum VfsResponse {
    /// Generic success (e.g., fd for Open or 0 for non-returning ops).
    Success(i32),
    /// Read result payload.
    Data(Vec<u8>),
    /// Metadata result payload.
    Metadata(VfsMetadata),
    /// Directory listing payload.
    DirectoryEntries(BTreeMap<String, VfsMetadata>),
    /// Standardized error payload.
    Error { code: i32, message: String },
}
```

## Functional Responsibilities

`vfs` is responsible for:

1. Request routing to storage backends.
2. File descriptor table management.
3. Path resolution.
4. Capability enforcement (e.g., scoped `StorageAccess`).
5. Metadata caching for common reads.
6. Error normalization into `VfsResponse::Error`.

## Usage Example: Open, Read, Close

```rust
let mut vfs_chan = VNodeChannel::new(7); // IPC channel to svc://vfs

let open_req = VfsRequest::Open {
    path: "/etc/network/config.txt".into(),
    flags: 0, // conceptual O_RDONLY
};

let fd: Fd = match vfs_chan.send_and_recv::<VfsRequest, VfsResponse>(&open_req) {
    Ok(VfsResponse::Success(file_fd)) => file_fd as Fd,
    Ok(VfsResponse::Error { code, message }) => {
        log!("open failed: {} ({})", message, code);
        return;
    }
    _ => {
        log!("unexpected open response");
        return;
    }
};

let read_req = VfsRequest::Read { fd, len: 1024, offset: 0 };
match vfs_chan.send_and_recv::<VfsRequest, VfsResponse>(&read_req) {
    Ok(VfsResponse::Data(data)) => {
        let content = String::from_utf8_lossy(&data);
        log!("{}", content);
    }
    Ok(VfsResponse::Error { code, message }) => {
        log!("read failed: {} ({})", message, code);
    }
    _ => log!("unexpected read response"),
}

let close_req = VfsRequest::Close { fd };
let _ = vfs_chan.send_and_recv::<VfsRequest, VfsResponse>(&close_req);
```

## Usage Example: Directory Listing

```rust
let mut vfs_chan = VNodeChannel::new(7);

let list_req = VfsRequest::List { path: "/".into() };
match vfs_chan.send_and_recv::<VfsRequest, VfsResponse>(&list_req) {
    Ok(VfsResponse::DirectoryEntries(entries)) => {
        for (name, metadata) in entries {
            log!(
                "- {}: {} ({} bytes)",
                name,
                if metadata.is_dir { "Directory" } else { "File" },
                metadata.size
            );
        }
    }
    Ok(VfsResponse::Error { code, message }) => {
        log!("list failed: {} ({})", message, code);
    }
    _ => log!("unexpected list response"),
}
```

This contract allows clients to program against one stable filesystem IPC interface while `vfs` handles backend-specific behavior and security policy.

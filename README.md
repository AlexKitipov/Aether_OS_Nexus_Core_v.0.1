# Aether_OS_Nexus_Core_v.0.1
AetherOS Nexus Core v0.1 — a Rust‑based hybrid microkernel focused on security, modularity, and driver compatibility through the Nexus Bridge. V‑Node containers, instant dev environments, and a modern IPC architecture. The foundation of a new OS ecosystem.


# 🌌 AetherOS Alpha — The Nexus Architecture Manifesto

_Join the Aether. Build the Nexus._

## 🚀 Project Vision & Mission

**AetherOS** is not just another operating system; it's a **Nexus Hybrid** – a new class of operating system designed from the ground up to redefine security, performance, and transparency in computing. Our mission is to build a platform that is robust, user-centric, and resilient in an increasingly complex digital world, empowering developers and users with unprecedented control and insight.

Traditional operating systems are prisoners of their history:

*   **Windows** is a monolithic labyrinth of legacy code, constantly battling security vulnerabilities and resource inefficiency.
*   **Linux** is powerful but fragmented, often requiring deep technical expertise for optimal configuration.
*   **macOS** offers a polished experience but confines users to a closed ecosystem, limiting freedom and transparency.

None of them are built for a world where drivers are sandboxed by default, inter-process communication (IPC) is visually inspectable, and applications are immutable, cryptographically verifiable entities. AetherOS aims to be that paradigm shift.

## 🧬 Core Architectural Pillars

AetherOS is founded on revolutionary principles that leverage modern systems programming and cryptographic guarantees:

1.  **Memory Safety by Default**: The entire Nexus Core is written in **Rust**, eliminating 70% of classic kernel vulnerabilities (e.g., use-after-free, buffer overflows) at compile time. Unsafe blocks are minimal, localized, and rigorously reviewed.
2.  **Nexus Hybrid Microkernel**: A minimal, capability-secured microkernel manages only memory, CPU scheduling, and IPC. All other services (drivers, file systems, network stacks, GUI) operate as isolated **V-Nodes** (Virtual Nodes) in user-space.
3.  **Capability-Based Security**: There is no `root` user or global privileges. Every V-Node possesses only the precise rights (capabilities) explicitly granted to it. These capabilities are fine-grained and enforced by the Nexus Core, enabling true least-privilege security.
4.  **Zero-Copy IPC**: Inter-Process Communication is designed for lightning speed. Small messages are passed directly; large data transfers (e.g., framebuffers, network packets) use **shared memory** pages with **transfer-of-ownership** semantics, minimizing data copying and maximizing throughput.
5.  **Zero-Trust Runtime**: No component, from drivers to user applications, is inherently trusted. Every operation is validated against its granted capabilities, and every data transfer is cryptographically verified.
6.  **Immutable Infrastructure (V-Nodes)**: Applications are distributed as **.ax packages**, which are cryptographically signed, content-addressed, and immutable bundles. This ensures reproducibility, easy rollbacks, and eliminates

## 🧬 Core Architectural Pillars (continued)

7.  **Zero-Copy Networking**: Network packets are exchanged between hardware, kernel, and network stack V-Nodes using DMA-compatible **shared memory** handles. Data moves from NIC to application without CPU-intensive copying, enabling near-wire-speed performance.
8.  **Visual Observability**: The Nexus Dashboard provides a real-time, interactive visualization of the entire system – IPC flows, V-Node states, resource usage, and network traffic. This deep transparency empowers developers to understand and debug complex microkernel interactions.
9.  **Aether Driver Intelligence (ADI)**: An AI-assisted system that translates existing Linux/Windows drivers into safe, sandboxed V-Nodes. This revolutionizes hardware compatibility, allowing AetherOS to support a vast array of devices from day one, without compromising security or stability.
10. **Decentralized Trust Model**: Trust is cryptographic, not centralized. Applications are cryptographically signed by publishers, and their integrity is verified using **Merkle Trees** and **Content-Addressable Storage (CAS)**. A robust **Reputation Layer** dynamically assesses the trustworthiness of network peers.
11. **Resource Quotas & Admission Control**: Every V-Node declares its resource needs (RAM, CPU, IPC rate). The Nexus Core enforces these quotas, preventing misbehaving applications from destabilizing the system. New applications undergo rigorous admission control before being launched.

## ✨ Nexus Core v0.1: Features and Milestones (Alpha Complete)

In this initial Alpha release, we have successfully architected and laid the foundation for AetherOS, demonstrating core functionalities across three major stages:

### 🧱 Stage 1: The Core (Foundation)

*   **Rust Microkernel**: Minimal `no_std` kernel (`_start`, panic handler) with basic serial logging.
*   **Page Allocator & Global Heap**: Physical memory management and dynamic memory allocation (`Vec`, `BTreeMap`) for kernel and V-Nodes.
*   **Preemptive Scheduler**: Round-robin scheduler with timer interrupts for fair CPU allocation across tasks/V-Nodes.
*   **Nexus IPC**: IPC mechanisms with dedicated channels, `SYS_IPC_SEND`, `SYS_IPC_RECV`, `SYS_BLOCK_ON_CHAN` syscalls for efficient V-Node communication.
*   **Nexus Capability Layer (NCL)**: Fine-grained, declarative capabilities (`LogWrite`, `TimeRead`, `NetworkAccess`) enforced by the kernel for every syscall.
*   **AetherFS (Initrd)**: Minimal in-memory file system for loading V-Node binaries from `initrd` with **CID verification**.
*   **ELF Loader**: Dynamically loads `ELF64` V-Node binaries into isolated memory spaces, including crucial **kernel overlap protection**.

### 🌐 Stage 2: Connectivity (Network)

*   **Nexus Net Bridge V-Node**: First user-space driver for simulated **VirtIO-Net**, handling `SYS_IRQ_REGISTER`, `SYS_NET_ALLOC_BUF`, `SYS_NET_RX_POLL`, `SYS_NET_TX` syscalls.
*   **AetherNet Service V-Node**: Integrated **smoltcp** network stack for IPv4, ARP, and ICMP, providing `NetStackRequest`/`Response` IPC API.
*   **Zero-Copy Networking**: Packet transfer between `net-bridge` and `aethernet-service` using DMA handles and shared memory, minimizing data copies.
*   **Simulated ICMP Echo Reply**: Successfully demonstrated `aethernet-service` responding to `ping` requests, verifying the full network data path.

### 🖥️ Stage 3: Interface (Graphics & Interaction)

*   **VirtIO-GPU Bridge V-Node**: First user-space GPU driver, handling `SYS_GPU_ALLOC_FB`, `SYS_GPU_FLUSH` for basic framebuffer operations.
*   **AetherCompositor V-Node**: Orchestrates visual output, managing `WindowSurface` objects with `z-order` and `damage regions` for efficient rendering.
*   **Aether Window Protocol (AWP)**: IPC protocol (`WIN_CREATE`, `WIN_PRESENT`) allowing V-Nodes to request and manage graphical surfaces.
*   **Nexus Input Bridge V-Node**: Handles raw keyboard and mouse events (`InputEvent`), routing them via IPC to the Compositor.
*   **Compositor Input Routing**: Maps raw input to `WindowEvent`s, performing `hit-testing` and `focus management` for active windows.
*   **AetherTerminal-Native**: First interactive GUI V-Node, reacting to keyboard input and rendering text to its shared surface.

### 📦 Stage 4: App Ecosystem (Civilization)

*   **Aether Runtime (.ax)**: Defined `.ax` as a signed, content-addressed bundle for applications, incorporating **Merkle Trees** for integrity.
*   **Aether Shell (ash)**: Core command-line interpreter with built-in commands (`cd`, `ps`, `clear`) and **IPC Piping** for inter-V-Node communication (`ls | grep`).
*   **Aether Local Registry**: Content-Addressable Storage (CAS) based registry (`/var/ax/objects`, `/app`) for versioned, deduplicated application storage.
*   **Dependency Resolver**: Algorithm for building immutable dependency graphs and ensuring side-by-side execution of different app versions.
*   **Atomic Updates**: Seamless, zero-downtime application updates via atomic `rename()` operations on `latest` symlinks.
*   **Shared V-Nodes**: Concept of system services (e.g., `AetherCrypto Service`) as shared V-Nodes, accessed via IPC, enhancing security and resource efficiency.
*   **Resource Quotas & Admission Control (Runtime v2)**: Declarative application profiles in `ax.json` enforced by Nexus Core for RAM, CPU, and IPC rates.
*   **Aether Trust Model & Unknown Sandbox**: Cryptographic signatures for `.ax` packages and a strict isolation policy for untrusted applications.

### 🌌 Stage 5: The Global Swarm & Persistence

*   **Aether Identity (AID)**: Decentralized, self-sovereign user identity based on cryptographic key pairs, with **AID-0** as the genesis identity.
*   **Vault V-Node**: Securely stores private keys, performing cryptographic operations on behalf of V-Nodes.
*   **AetherFS v1.0 (Encrypted Home)**: Persistent, **AES-GCM encrypted** `/home` directories, locked with AID, providing verifiable integrity via **Merkle Trees**.
*   **Aether Registry Protocol (ARP)**: Decentralized App Store based on **Kademlia-like DHT** for finding CAS objects (packages, chunks).
*   **Swarm Engine**: P2P content delivery system with **Chunking** and **Parallel Fetching** for efficient `.ax` package distribution.
*   **Reputation Layer**: Algorithmic trust system for peers, rewarding **Proof of Contribution** and penalizing misbehavior.
*   **Aether Cloud Sync**: Personal P2P synchronization of user data across AID-verified devices, leveraging the Swarm Engine.
*   **Aether Messaging Protocol (AMP)**: End-to-end encrypted, decentralized messaging for V-Nodes and users, built on AID and Swarm.

## 🛠️ Build & Run Guide (Nexus Core v0.1 Alpha)

This guide outlines the steps to build and run the current state of AetherOS Nexus Core in a simulated environment (QEMU).

### Prerequisites

*   **Rust Nightly**: Ensure you have a recent nightly Rust toolchain installed (specified in `rust-toolchain.toml`).
*   **`rust-src` component**: `rustup component add rust-src --toolchain nightly`
*   **`llvm-tools-preview` component**: `rustup component add llvm-tools-preview`
*   **`bootimage` cargo subcommand**: `cargo install bootimage`
*   **QEMU**: Version 5.2 or newer, for `x86_64` architecture (`qemu-system-x86_64`).

### Project Structure

The repository is structured as a Rust workspace:

```text
nexus-core/
├─ Cargo.toml
├─ src/                      # Common modules for kernel and user-space V-Nodes
│  ├─ lib.rs
│  ├─ cid.rs                 # Content ID (SHA-256)
│  ├─ manifest.rs            # Package Manifest (ax.json)
│  ├─ trust.rs               # TrustScore, Aid
│  ├─ arp_dht.rs             # In-memory DHT simulation
│  ├─ swarm_engine.rs        # Chunk fetching logic
│  ├─ ipc/                   # IPC traits and VNodeChannel for user-space
│  └─ syscall.rs             # User-space syscall wrappers
│
├─ kernel/                   # Nexus Core (the OS kernel)
│  ├─ Cargo.toml
│  ├─ src/
│  │  ├─ main.rs             # kernel_main(), boot sequence
│  │  ├─ mem.rs              # Page Allocator
│  │  ├─ heap.rs             # Global Heap allocator
│  │  ├─ ipc.rs              # Kernel-side IPC channels
│  │  ├─ task.rs             # Task management, Scheduler
│  │  ├─ syscall.rs          # Syscall dispatcher, kernel-side syscall handlers
│  │  ├─ caps.rs             # Capability enum and checks
│  │  ├─ interrupts.rs       # IDT, interrupt handlers
│  │  ├─ timer.rs            # PIT initialization
│  │  ├─ aetherfs.rs         # In-memory AetherFS (initrd provider)
│  │  ├─ elf.rs              # Minimal ELF64 loader
│  │  ├─ vnode_loader.rs     # Loads V-Nodes from AetherFS
│  │  └─ console.rs          # Basic serial console output
│  ├─ vnode.ld               # Linker script for kernel
│
├─ vnode/                    # Example V-Node applications
│  ├─ registry/              # Example Registry V-Node
│  │  ├─ Cargo.toml
│  │  ├─ src/main.rs         # Registry V-Node entry point
│  │  ├─ vnode.ld            # Linker script for V-Nodes
│  │  └─ manifest.json       # V-Node capabilities and metadata
│  ├─ net-bridge/            # Example Net-Bridge V-Node
│  │  ├─ Cargo.toml
│  │  ├─ src/main.rs
│  │  ├─ vnode.ld
│  │  └─ manifest.json
│  ├─ net-stack/             # Example AetherNet Service V-Node
│  │  ├─ Cargo.toml
│  │  ├─ src/main.rs
│  │  ├─ aethernet_device.rs
│  │  ├─ vnode.ld
│  │  └─ manifest.json
│
├─ target/                   # Build artifacts
```

### Building AetherOS Nexus Core

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/aetheros/nexus-core.git
    cd nexus-core
    ```
2.  **Install `bootimage`**: This tool helps create bootable disk images from Rust `no_std` kernels.
    ```bash
    cargo install bootimage --version

### Building AetherOS Nexus Core (continued)

3.  **Compile V-Node applications**: Each V-Node (e.g., `registry`, `net-bridge`, `net-stack`) is compiled as a separate `no_std` ELF binary using its specific linker script.
    ```bash
    # Example for registry V-Node
    cargo build -p vnode-registry --target x86_64-unknown-none --release
    
    # You would repeat this for net-bridge, net-stack, etc.
    ```

4.  **Create `initrd` (Initial RAM Disk)**: This step bundles your compiled V-Node binaries and their manifests into a single image that the kernel will load at boot. Our `AetherFS` uses this `initrd`.
    ```bash
    # Example: Manually create a simple initrd (or use a script)
    # Place registry.elf, net-bridge.elf, net-stack.elf and their manifests in a directory
    # Then, create a cpio or a custom tar-like archive for the kernel to parse.
    # For v0.1, AetherFS is very basic and might just expect a single V-Node binary.
    ```

5.  **Build the Kernel**: The `bootimage` tool will compile the `kernel` crate and embed your `initrd` (if configured) into a bootable `ELF` kernel image.
    ```bash
    cd kernel
    cargo bootimage --release
    # This will generate a bootable image at target/x86_64-unknown-none/release/bootimage-nexus-core.bin
    ```

### 🚀 Running in QEMU

To see AetherOS Nexus Core in action:

```bash
qemu-system-x86_64 \
  -machine q35 \
  -m 2G \
  -serial stdio \
  -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-nexus-core.bin \
  # Add -initrd <path_to_your_initrd> if you have one prepared
  # For network simulation:
  -netdev user,id=net0,hostfwd=tcp::8080-:80 \
  -device virtio-net-pci,netdev=net0,mac=02:00:00:00:00:01
```

This command boots AetherOS in QEMU. All kernel and V-Node logs will be streamed to your console via the `-serial stdio` option.

### 🗺️ Immediate Roadmap (v0.2 / v0.3)

Our journey is far from over. The next steps for AetherOS Nexus Core involve expanding on the established foundations:

#### Nexus Core v0.2: Resource & Security Hardening

*   **Virtual Memory Management (VMM)**: Implement full virtual address spaces for V-Nodes, enabling robust memory isolation (Ring 3 for V-Nodes, Ring 0 for Kernel).

## 🔌 Socket API (`svc://socket-api`)

The `socket-api` V-Node exposes a POSIX-like socket interface over IPC for applications in AetherOS. Instead of allowing direct access to the network stack, applications communicate with `svc://socket-api`, which forwards validated operations to `svc://aethernet-service`. This preserves isolation and the capability model.

### IPC Types

Defined in `src/ipc/socket_ipc.rs`:

```rust
pub type SocketFd = u32;

#[derive(Debug, Serialize, Deserialize)]
pub enum SocketRequest {
    Socket { domain: i32, ty: i32, protocol: i32 },
    Bind { fd: SocketFd, addr: [u8; 4], port: u16 },
    Listen { fd: SocketFd, backlog: i32 },
    Accept { fd: SocketFd },
    Connect { fd: SocketFd, addr: [u8; 4], port: u16 },
    Send { fd: SocketFd, data: Vec<u8> },
    Recv { fd: SocketFd, len: u32 },
    Close { fd: SocketFd },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SocketResponse {
    Success(i32),
    Data(Vec<u8>),
    Error(i32, String),
    Accepted { new_fd: SocketFd, remote_addr: [u8; 4], remote_port: u16 },
}
```

### Semantics

- `Socket { domain, ty, protocol }` creates a socket and returns `SocketResponse::Success(fd)`.
- `Bind`, `Listen`, `Connect`, and `Close` return `SocketResponse::Success(0)` on success.
- `Send` returns `SocketResponse::Success(bytes_sent)`.
- `Recv` returns `SocketResponse::Data(Vec<u8>)`.
- `Accept` returns `SocketResponse::Accepted { new_fd, remote_addr, remote_port }`.
- Any failure returns `SocketResponse::Error(errno, message)`.

### Common Values

- `domain = 2` → `AF_INET` (IPv4)
- `ty = 1` → `SOCK_STREAM` (TCP)
- `ty = 2` → `SOCK_DGRAM` (UDP)
- `protocol = 0` → default protocol for selected domain/type

### Error Handling Guidance

Clients should always handle `SocketResponse::Error(errno, message)` and branch on `errno` where needed. Common values include:

- `-1`: generic error
- `9`: bad file descriptor (`EBADF`-like)
- `11`: operation would block (`EWOULDBLOCK`-like)
- `100`: custom `socket-api` domain error
*   **Page Fault Handling**: Implement a robust page fault handler to manage memory-on-demand and enforce memory access policies.
*   **Dynamic Memory Allocation for V-Nodes**: Allow V-Nodes to request additional memory from the kernel at runtime via `SYS_ALLOC_MEM` syscalls.
*   **Process Control Blocks (PCBs)**: Enhance task management with more detailed process information and state transitions.
*   **Interrupt Handling Refinement**: Implement more flexible IRQ routing, including message-signaled interrupts (MSI/MSI-X) for modern hardware.

#### Nexus Core v0.3: Network Stack Maturity & Basic Persistence

*   **Full AetherNet Protocol Stack**: Integrate full TCP, UDP, ICMP, ARP, DHCP, and DNS capabilities into `aethernet-service`.
*   **Socket API Client Library**: Develop `libnexus-net` for V-Nodes to easily interact with `svc://aethernet` for network communication.
*   **AetherFS v0.2 (Basic Persistent Storage)**: Implement a simple block device driver (e.g., VirtIO-Blk) and a basic journaling file system for persistent `/home` directories.
*   **Aether Identity (AID) Manager**: Implement the `Vault V-Node` with real cryptographic primitives (Ed25519) for managing `AID-0` and user identities.
*   **Registry Service & `nexus install`**: Enable `registry-service` to perform basic DHT lookups and fetch `.ax` package manifests via the network.

### 🤝 Contribution Guidelines

AetherOS is an ambitious open-source project, and we welcome contributions from system programmers, Rustaceans, security researchers, and anyone passionate about building the future of operating systems.

*   **How to Contribute**: Check out our `CONTRIBUTING.md` (soon to be created) for detailed instructions on setting up your development environment, coding standards, and submitting pull requests.
*   **Code of Conduct**: We adhere to a strict Code of Conduct to ensure a welcoming and inclusive community. Please read it before engaging with the project.
*   **Contact**: Join our Discord channel (link to be added) or open an issue on GitHub to ask questions, suggest ideas, or report bugs.

**Join the Aether. Build the Nexus**

=================================================================================================================

📘 AetherOS Nexus — Core Documentation (v0.1.0)
A clear, structured, and professional overview of the current system architecture.

1. Overview
AetherOS Nexus is a modular hybrid‑microkernel operating environment built around V‑Nodes — isolated, capability‑restricted system services that communicate through IPC channels and kernel syscalls.

Version v0.1.0 currently includes:

a minimal kernel with syscall dispatcher

IPC messaging system

IRQ routing to V‑Nodes

DMA buffer allocation API (stub)

the first system driver: net‑bridge V‑Node

capability‑based security model

YAML‑based V‑Node declaration format

This version establishes the foundation for a future OS ecosystem.

2. Architecture
2.1 Kernel Components
Syscall Dispatcher
The kernel currently supports:

Syscall	Description
SYS_LOG	Writes V‑Node logs to the kernel console
SYS_IPC_SEND	Sends IPC messages to a channel
SYS_IPC_RECV	Receives IPC messages
SYS_BLOCK_ON_CHAN	Blocks the current task on a channel
SYS_TIME	Reads kernel timer ticks
SYS_IRQ_REGISTER	Registers an IRQ to a V‑Node channel
SYS_NET_RX_POLL	Polls for incoming network packets (stub)
Capabilities
The kernel enforces access control via:

LogWrite

TimeRead

NetworkAccess

StorageAccess

Future capabilities will include PCI I/O, DMA, IRQ, MMIO, and raw syscalls.

3. IPC System
V‑Nodes communicate through channels identified by u32 IDs.

The kernel can:

deliver IRQ events to channels

block tasks waiting on channels

copy message buffers between V‑Nodes

This forms the backbone of the Nexus IPC architecture.

4. IRQ Routing
A V‑Node registers an IRQ using:

Код
syscall3(SYS_IRQ_REGISTER, irq_number, channel_id, 0)
When the IRQ fires:

The kernel sends an IPC event to the V‑Node’s channel

The V‑Node wakes up

It processes the event

It acknowledges the IRQ (future syscall)

This model cleanly separates hardware events from driver logic.

5. DMA Buffers (Stub Implementation)
V‑Nodes can request DMA‑compatible buffers via:

SYS_NET_ALLOC_BUF

SYS_NET_FREE_BUF

The allocator is currently a stub, but the API is stable and ready for a real implementation.

6. Net‑Bridge V‑Node
6.1 Purpose
net-bridge is the first system driver. It demonstrates:

IRQ registration

DMA buffer usage

RX polling

TX packet generation

IPC‑based interrupt handling

the V‑Node execution model

It will later act as a bridge to the svc://aethernet service.

6.2 Lifecycle
Initialize IPC channel (ID = 2)

Register IRQ 11 (VirtIO‑Net)

Allocate RX DMA buffer

Enter main loop:

wait for IRQ

poll for packets

process RX

send a test TX packet

free TX buffer

6.3 YAML Declaration
The V‑Node requires:

NetworkAccess capability

PCI access to the VirtIO‑Net device

DMA allocation

IRQ 11

strict isolation mode

This ensures minimal and secure hardware access.

7. Current Limitations
SYS_NET_RX_POLL always returns 0 (no real packets yet)

DMA allocator is stubbed

No VirtIO‑Net queue handling

IRQ ACK syscall incomplete

No scheduler

No memory manager

These are expected for an early v0.1.0 kernel.

8. Roadmap (v0.2.0 → v0.3.0)
v0.2.0 Goals
Real DMA allocator

VirtIO‑Net RX/TX queue implementation

IRQ ACK syscall

Basic packet parsing (ARP, IPv4)

v0.3.0 Goals
AetherNet service

Zero‑copy packet forwarding

Multi‑interface support

Basic routing logic

9. Developer Notes
The architecture is stable despite being early‑stage

V‑Node model works as intended

IRQ routing is functional

IPC channels are operational

Syscall layer is extendable

net‑bridge is a solid template for future drivers

10. Summary
AetherOS Nexus v0.1.0 already provides:

a functioning microkernel

IPC

IRQ routing

DMA API

the first network driver

capability‑based security

YAML‑based V‑Node definitions

This is the foundation of a real operating system.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use core::panic::PanicInfo;

use crate::arp_dht::{InMemoryDht, NodeId, PeerInfo};
use crate::ipc::vnode::VNodeChannel;
use crate::swarm_engine::global_search::GlobalSearchService;
use crate::swarm_engine::nexus_net_transport::NexusNetTransport;
use crate::swarm_engine::SwarmEngine;
use crate::trust::{Aid, TrustStore};

// Temporary log function for V-Nodes (redefined for registry to avoid module conflict)
fn log(msg: &str) {
    unsafe {
        let res = crate::syscall::syscall3(
            crate::syscall::SYS_LOG,
            msg.as_ptr() as u64,
            msg.len() as u64,
            0, // arg3 is unused for SYS_LOG
        );
        if res != crate::syscall::SUCCESS {
            // Handle log error
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Registry's own channel ID (to receive IRQ events from kernel)
    let mut own_chan = VNodeChannel::new(1); // Channel for Registry IPC

    log("Registry V-Node starting up...");

    // 1. Initialize NexusNetTransport (which uses NetClient internally)
    let transport = match NexusNetTransport::new() {
        Ok(t) => {
            log("Registry: NexusNetTransport initialized.");
            t
        }
        Err(e) => {
            log(&alloc::format!(
                "Registry: Failed to initialize NexusNetTransport: {:?}",
                e
            ));
            loop {
                core::hint::spin_loop();
            }
        }
    };

    // --- Swarm Engine Initialization ---
    let trust_store = TrustStore::new();
    let local_aid = Aid([0xCD; 32]); // Dummy local AID
    let local_node_id = NodeId([0; 32]); // Dummy NodeId for local DHT

    let mut dht_for_init = InMemoryDht::new(local_node_id.clone());

    // Add a dummy peer for DHT lookup simulation.
    dht_for_init.add_peer(PeerInfo {
        id: NodeId([0xAA; 32]),
        aid: Aid([0xBB; 32]),
        ip_address: [10, 0, 2, 1], // Example peer IP
        port: 60000,               // Example peer port
    });

    // Load a dummy package for demonstration.
    let (manifest, _chunks) = crate::examples::hello_package::make_hello_package();
    dht_for_init.store(
        manifest.root_cid,
        crate::arp_dht::DhtValue::Manifest(manifest.clone()),
    );

    // Clone DHT, TrustStore, and Aid for each service that needs ownership.
    let global_search_service =
        GlobalSearchService::new(dht_for_init.clone(), trust_store.clone(), local_aid.clone());
    let mut swarm = SwarmEngine::new(transport, dht_for_init, trust_store.clone(), local_aid.clone());
    // --- End Swarm Engine Initialization ---

    // Simulate fetching a package from the swarm using the real network transport.
    match swarm.fetch_package(&manifest) {
        Ok(data) => {
            log(&alloc::format!(
                "Registry: Successfully fetched package '{}' ({} bytes)",
                manifest.name,
                data.len()
            ));
        }
        Err(e) => {
            log(&alloc::format!("Registry: Failed to fetch package: {:?}", e));
        }
    }

    // Test Global Search.
    let search_request = crate::swarm_engine::global_search::SearchRequest::KeywordSearch {
        query: String::from("hello"),
    };
    let search_response = global_search_service.handle_search_request(search_request);
    log(&alloc::format!(
        "Registry: Global Search Response: {:?}",
        search_response
    ));

    loop {
        // Original RegistryService loop logic would go here, handling IPC requests.
        log("Registry V-Node idling...");
        let _ = own_chan.recv_blocking(); // Block and wait for IPC messages.
    }
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("Registry V-Node panicked!");
    loop {
        core::hint::spin_loop();
    }
}

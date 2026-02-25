use aetheros_kernel::ipc::{kernel_recv, kernel_send};

#[test]
fn test_vnode_ipc_communication() {
    let channel_id = 2;
    let test_msg = "Hello from V-Node 1";

    kernel_send(channel_id, test_msg.as_bytes()).expect("send failed");

    match kernel_recv(channel_id) {
        Some(data) => {
            let received = String::from_utf8(data).expect("invalid utf8");
            assert_eq!(received, test_msg);
        }
        None => panic!("Expected message, got nothing"),
    }
}

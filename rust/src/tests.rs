

#[cfg(test)]
mod tests {
    use crate::next_piece;
    use websocket::ClientBuilder;

    // support synchronous websockets, great for testing
    extern crate websocket;
    use std::{thread, time};
    use std::sync::{Once};

    static INIT_SERVER : Once = Once::new();

    fn start_server() {
        use crate::main as program_main;

        // this is synchronous, so we'll wait the 1 second before returning
        INIT_SERVER.call_once(|| {
            thread::spawn(move || {
                program_main();
            });

            thread::sleep(time::Duration::from_millis(1000));
        });
    }

    #[test]
    fn test_next_piece() {
        assert!(next_piece() >= 0);
        assert!(next_piece() <= 7);
    }

    // test to make sure initialize code is good!
    #[test]
    fn test_ws_init_flow() {
        start_server();

        let mut client = ClientBuilder::new("ws://127.0.0.1:3012")
            .unwrap()
            .connect_insecure()
            .unwrap();

        let msg = client.recv_message().unwrap();

        println!("message received");

        let message_received = match msg {
            OwnedMessage => true,
            WebSocketError => false,
        };

        assert!(message_received);
    }
}






//
//
// let message = Message::text("Hello, World!");
// client.send_message(&message).unwrap(); // Send message

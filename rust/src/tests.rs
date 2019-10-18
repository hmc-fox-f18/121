// support synchronous websockets, great for testing
extern crate websocket;




// use ws::{connect, CloseCode};
//
// ws::connect("ws://127.0.0.1:3012", |out| {
//     out.send("Hello WebSocket").unwrap();
//
//     move |msg| {
//         println!("Got message: {}", msg);
//         out.close(CloseCode::Normal)
//     }
// }).unwrap()

#[cfg(test)]
mod tests {
    use crate::next_piece;
    use websocket::ClientBuilder;

    #[test]
    fn test_next_piece() {
        assert!(next_piece() >= 0);
        assert!(next_piece() <= 7);
    }

    // test to make sure initialize code is good!
    #[test]
    fn test_ws_init_flow() {
        let mut client = ClientBuilder::new("ws://127.0.0.1:3012")
            .unwrap()
            .connect_insecure()
            .unwrap();


        let msg = client.recv_message().unwrap();

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

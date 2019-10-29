

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

    /*
    Test to make sure that initial response from server is properly formatted.
    */
    #[test]
    fn test_ws_init_flow() {
        use websocket::message::OwnedMessage;
        use serde_json::{Result, Error, Value};


        start_server();

        let mut client = ClientBuilder::new("ws://127.0.0.1:3012")
            .unwrap()
            .connect_insecure()
            .unwrap();

        let msg = client.recv_message().unwrap();

        // assert that a message was received
        let message_string = match msg {
            OwnedMessage::Text(text) => text,
            WebSocketError => panic!("Can't read message."),
        };

        // Parse the string of data into serde_json::Value.
        let message_json : Value = serde_json::from_str(&message_string).unwrap();

        // check to make sure message_type is correct
        assert!(message_json["type"] == "init");

        // check to make sure piece_type is in correct range
        assert!(message_json["piece_type"].is_number());
        let piece_type : i64 = message_json["piece_type"].as_i64().unwrap();
        assert!(piece_type >= 0);
        assert!(piece_type <= 6);

        // check to make sure we have a non-zero user id
        assert!(message_json["player_id"].is_number());
        let user_id = message_json["player_id"].as_i64().unwrap();
        assert!(user_id >= 0);
    }
}






//
//
// let message = Message::text("Hello, World!");
// client.send_message(&message).unwrap(); // Send message

pub mod tcp_parse {
    use crate::encoding::encoding::MmqpEncoding;
    use crate::{MmqpMessage, MmqpSerialisable};
    use std::any::Any;
    use std::net::TcpStream;

    #[test]
    fn test_string_parse() {
        let magic = b"MMQP";
        let version_major = 0u8;
        let delimiter = b".";
        let version_minor = 1u8;

        let version_bytes = [version_major, delimiter[0], version_minor];
        let username = "myusername".to_string().to_mmqp_binary().unwrap();
        let password = "password".to_string().to_mmqp_binary().unwrap();
        let queuename = "myqueuename".to_string().to_mmqp_binary().unwrap();
        let message = "mymessage".to_string().to_mmqp_binary().unwrap();

        let mut bytes = Vec::new();
        bytes.extend_from_slice(magic);
        bytes.extend(b"|");
        bytes.extend_from_slice(&version_bytes);
        bytes.extend(b"|");
        bytes.extend(b"M");
        bytes.extend(b"|");
        bytes.extend(username);
        bytes.extend(b":");
        bytes.extend(password);
        bytes.extend(b"|");
        bytes.extend(queuename);
        bytes.extend(b"|");
        bytes.extend(message);

        dbg!(bytes.clone());

        let res: MmqpTcpFormat = parse_tcp_request(bytes);

        match res {
            MmqpTcpFormat::Message(_) => {
                assert!(true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_object_parse() {
        let version_major = 0u8;
        let version_minor = 1u8;
        let username = "myusername".to_string();
        let password = "password".to_string();
        let queuename = "myqueuename".to_string();
        let message = "mymessage".to_string();

        let message: MmqpMessage = MmqpMessage {
            version_major,
            version_minor,
            username: username,
            password: password,
            target_queue: queuename,
            message: message,
            message_group: "".to_string(),
        };

        let bytes = message.serialise();
        let encl = bytes[0];
        //skip the number of bytes listed by encl
        let mut bytes = &bytes[1..];
        bytes = &bytes[encl as usize..];

        dbg!(bytes.clone());

        let res: MmqpTcpFormat = parse_tcp_request(Vec::from(bytes));

        match res {
            MmqpTcpFormat::Message(_) => {
                assert!(true);
            }
            _ => {
                println!("{:?}", res);
                assert!(false);
            }
        }
    }

    #[derive(Debug)]
    pub enum MmqpTcpFormat {
        //place a message in the specified queue
        Message(crate::MmqpMessage),

        //todo figure out how to handle this, placeholder
        Admin,

        // keepalive connection, duration, queue
        LongPoll(TcpStream, u32, String),

        //queue, message id
        Del(String, String),

        Ping,
    }

    pub fn parse_tcp_request(request: Vec<u8>) -> MmqpTcpFormat {
        //strip first 4 bytes to get rid of the magic MMQP header
        let request = &request[4..];

        //remove   the next byte to strip the delimiter
        let request = &request[1..];

        //next 3 bytes are the x.x version
        let version = &request[0..3];
        dbg!(version.clone());
        //split version at a dot and pare both sides at u8
        let version_major: u8 = version[0..1][0];
        let version_minor: u8 = version[2..3][0];

        //move to that po
        let request = &request[3..];

        //skip a byte for the delimiter
        let request = &request[1..];

        //next byte is the command
        let command = String::from_utf8(Vec::from(&request[0..1])).unwrap();
        //move to that position
        let request = &request[1..];

        //strip a delimiter
        let request = &request[1..];

        return match command.as_str() {
            "M" => {
                let message = parse_as_message(request.to_vec(), version_major, version_minor);
                message
            }
            _ => MmqpTcpFormat::Ping,
        };
    }

    fn parse_as_message(request: Vec<u8>, version_major: u8, version_minor: u8) -> MmqpTcpFormat {
        // dbg!(version);
        // dbg!(command);
        // dbg!(request);
        // dbg!(version_major);
        // dbg!(version_minor);

        println!("pre req: {:?}", request.clone());
        let mut cursor = 0usize;
        let username: String = String::from_mmqp_binary(&request, &mut cursor);
        println!("{}", username);
        cursor += 1;
        let password: String = String::from_mmqp_binary(&request, &mut cursor);
        println!("{}", password);
        cursor += 1;
        let target_queue: String = String::from_mmqp_binary(&request, &mut cursor);
        println!("{}", target_queue);
        cursor += 1;
        let message: String = String::from_mmqp_binary(&request, &mut cursor);
        println!("{}", message);

        MmqpTcpFormat::Message(MmqpMessage {
            version_major,
            version_minor,
            username,
            password,
            target_queue,
            message,
            message_group: "main".to_string(), //todo implement message groups
        })
    }
}

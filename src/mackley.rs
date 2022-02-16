pub mod application {
    use lib::queue::queue::{PendingMode, QueueConfiguration};
    use lib::tcp_parse::tcp_parse::MmqpTcpFormat;
    use lib::{MmqpSerialisable, Queue, SerialisationStrategy};
    use std::borrow::Borrow;
    use std::collections::HashMap;
    use std::future::Future;
    use std::hash::Hash;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::SystemTime;

    pub struct Application {
        listener: TcpListener,
        port: u16,
        queues: HashMap<String, Queue>,

        // optional page dir means there is no disk storage between session
        // messages are lost on crash or restart
        page_dir: Option<String>,

        // optional auth file means there is no authentication required. Desirable for local
        // undesirable for remote.
        // if the auth file is missing, user:pass are ignored but must still be present for format reasons
        // recommended to use 0:0
        auth_file: Option<String>,
    }

    impl Application {
        pub fn new(port: u16, page_dir: Option<String>, auth_file: Option<String>) -> Application {
            //get the file with all the queue configs
            let filepath = "./queue_config.json";

            //read the file
            let file = std::fs::File::open(filepath);
            let mut queues: HashMap<String, Queue> = HashMap::new();
            if file.is_ok() {
                //read the file
                let mut contents = String::new();
                let r = file.unwrap().read_to_string(&mut contents);

                if r.is_ok() {
                    let config_maybe = Queue::deserialise_config_array(contents);

                    if config_maybe.is_ok() {
                        let config_array = config_maybe.unwrap().into_vec();
                        dbg!(config_array.clone());

                        //create the queues
                        for config in config_array.into_iter() {
                            let queue = Queue::new(config);
                            queues.insert(queue.name().clone(), queue);
                        }
                    }
                }
            } else {
                println!("Could not open queue config file");
                std::fs::File::create(filepath).unwrap();
            }

            Application {
                listener: TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap(),
                port,
                queues,
                page_dir,
                auth_file,
            }
        }

        pub fn listen(&mut self) -> () {
            println!("Listening on port {}", self.listener.local_addr().unwrap());
            for stream in self.listener.incoming() {
                //start a timer for the request
                let start = SystemTime::now();

                let mut s = stream.unwrap();
                // println!("New connection: {}", s.peer_addr().unwrap());
                let data = Application::read_stream(&mut s);

                let (bytes, size) = data;

                if size == 0 {
                    continue;
                }

                dbg!(bytes.clone());
                let req = lib::tcp_parse::tcp_parse::parse_tcp_request(bytes);

                match req {
                    MmqpTcpFormat::Message(message) => {
                        if self.queues.contains_key(&*message.target_queue) {
                            let queue = self.queues.get_mut(message.target_queue.as_str()).unwrap();
                            queue.receive_message(message);
                        } else {
                            println!("Queue not found: {}", message.target_queue);

                            // let mut _message:Vec<u8> = (b"MMQP|0.1|R|QUEUE_NOT_FOUND|").to_vec();
                            //
                            // _message.extend( message.target_queue.to_string().into_bytes());
                            // _message.push( 0x00);

                            s.write_all(
                                format!(
                                    "MMQP|0.1|R|QUEUE_NOT_FOUND|{}|{}",
                                    message.target_queue, 0x00
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                            s.flush().unwrap();
                        }
                    }
                    MmqpTcpFormat::Admin => {
                        println!("Admin");
                    }
                    MmqpTcpFormat::LongPoll(_, _, _) => {
                        println!("LongPoll");
                    }
                    MmqpTcpFormat::Del(_, _) => {
                        println!("Del");
                    }
                    MmqpTcpFormat::Ping => {
                        println!("Ping");

                        let success =s.write(b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\npong");
                        match success {
                            Ok(_) => {
                                let r = s.flush();
                                if r.is_err() {
                                    println!("Error flushing");
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    MmqpTcpFormat::Poll(queue, count) => {
                        if self.queues.contains_key(&*queue) {
                            let queue = self.queues.get_mut(queue.as_str()).unwrap();

                            let mut c = count;
                            let mut message = queue.read_next();
                            let mut _message: Vec<u8> = (b"MMQP|0.1|M|").to_vec();
                            while message.is_some() && c > 0 {
                                let message_bytes = message.unwrap();
                                _message.extend(
                                    message_bytes
                                        .serialise(SerialisationStrategy::Storage)
                                        .into_vec(),
                                );

                                c -= 1;

                                if c > 0 {
                                    message = queue.read_next();
                                } else {
                                    message = None;
                                }
                            }

                            _message.push(0x00);
                            s.write_all(&_message).unwrap();
                            s.flush().unwrap();
                        } else {
                            println!("Queue not found: {}", queue);

                            // let mut _message:Vec<u8> = (b"MMQP|0.1|R|QUEUE_NOT_FOUND|").to_vec();
                            //
                            // _message.extend( queue.to_string().into_bytes());
                            // _message.push( 0x00);

                            s.write_all(
                                format!("MMQP|0.1|R|QUEUE_NOT_FOUND|{}|{}", queue, 0x00).as_bytes(),
                            )
                            .unwrap();
                            s.flush().unwrap();
                        }
                    }
                }

                let end = SystemTime::now();
                let duration = end.duration_since(start).unwrap();
                println!("request took {}", duration.as_secs());
            }
        }

        /// Read the stream data and return stream data & its length
        fn read_stream(stream: &mut TcpStream) -> (Vec<u8>, usize) {
            let mut request_buffer: Vec<u8> = vec![];
            // let us loop & try to read the whole request data
            let mut request_len = 0usize;

            // fixed size temporary read buffer. its data is replaced each time  after copying it into the vec
            // if the amount read in is less than the buffer size, then we are done
            // calculated by current read position (n) mod buffer size (M)
            //
            const M: usize = 512usize;
            loop {
                let mut buffer = [0; M];
                match stream.read(&mut buffer) {
                    Ok(n) => {
                        dbg!(n);
                        if n == 0 {
                            break;
                        } else {
                            request_len += n;
                            request_buffer.extend(&buffer[..n]);

                            if n % M != 0 {
                                println!("should break here?");
                                break;
                            }

                            // we need not read more data in case we have read less data than buffer size
                        }
                    }

                    _ => {
                        println!("error reading stream");
                        return (Vec::new(), 0);
                    }
                }

                println!("{}", request_len);
                // dbg!(request_buffer.clone());
            }
            (request_buffer, request_len)
        }
    }
}

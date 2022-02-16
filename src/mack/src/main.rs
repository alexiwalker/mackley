use lib::{MmqpMessage, MmqpSerialisable, SerialisationStrategy};
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::SystemTime;

fn main() {
    //parse args: look or --host, --username and --password flags
    let args: Vec<String> = env::args().collect();

    dbg!(args.clone());
    //if no args, print help
    if args.len() == 1 {
        println!("Usage: mack [--host <host>] [--username <username>] [--password <password>]");
        return;
    }

    if args[1] == "config" {
        //if --host flag is present, set host to the next argument
        let mut host = "localhost";
        if args.contains(&String::from("--host")) {
            let index = args
                .iter()
                .position(|x| x == &String::from("--host"))
                .unwrap();
            host = &args[index + 1];
        }

        //if --username flag is present, set username to the next argument
        let mut username = "root";
        if args.contains(&String::from("--username")) {
            let index = args
                .iter()
                .position(|x| x == &String::from("--username"))
                .unwrap();
            username = &args[index + 1];
        }

        //if --password flag is present, set password to the next argument
        let mut password = "";
        if args.contains(&String::from("--password")) {
            let index = args
                .iter()
                .position(|x| x == &String::from("--password"))
                .unwrap();
            password = &args[index + 1];
        }

        dbg!(host);
        dbg!(username);
        dbg!(password);

        let filepath = "./mack.toml";

        let mut file = std::fs::File::create(filepath).unwrap();
        let config = format!(
            "host = \"{}\"\nusername = \"{}\"\npassword = \"{}\"",
            host, username, password
        );
        file.write_all(config.as_bytes()).unwrap();
        println!("Config file written to {}", filepath);

        return;
    }

    if args[1] == "send" {
        //next arg is the queue to send to
        let queue = &args[2];

        //next is the message to send
        let message = &args[3];

        let filepath = "./mack.toml";
        let filedata = std::fs::read_to_string(filepath);
        let mut host = "".to_string();
        let mut username = "".to_string();
        let mut password = "".to_string();

        dbg!(message.clone());
        dbg!(queue.clone());

        match filedata {
            Ok(config) => {
                for line in config.lines() {
                    if line.starts_with("host") {
                        host = line.split("=").collect::<Vec<&str>>()[1].trim().to_string();
                    }
                    if line.starts_with("username") {
                        username = line.split("=").collect::<Vec<&str>>()[1].trim().to_string();
                    }
                    if line.starts_with("password") {
                        password = line.split("=").collect::<Vec<&str>>()[1].trim().to_string();
                    }
                }

                //remove the learind and trailing " from the host, username and password
                let host = host.trim_matches('"').to_string();
                let username = username.trim_matches('"').to_string();
                let password = password.trim_matches('"').to_string();

                // dbg!(host.clone());
                // dbg!(username.clone());
                // dbg!(password.clone());

                let mut message_to_send: MmqpMessage = MmqpMessage::new();

                message_to_send.message = message.to_string();
                message_to_send.target_queue = queue.to_string();
                message_to_send.version_major = 0;
                message_to_send.version_minor = 1;
                message_to_send.username = username.to_string();
                message_to_send.password = password.to_string();
                let raw_message = message_to_send.serialise(SerialisationStrategy::Wire);

                let _stream = TcpStream::connect("127.0.0.1:8787");

                let mut stream = _stream.unwrap();

                stream.write_all(&raw_message).unwrap();

                let res = &mut String::new();

                stream.read_to_string(res).unwrap();
            }
            Err(_) => {
                println!(
                    "Error reading config file {}. Could not send message",
                    filepath
                );
            }
        }
    }
}

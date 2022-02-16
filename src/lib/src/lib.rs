pub mod encoding;
pub mod memory_storage;
pub mod normalised_message;
pub mod page_io;
pub mod queue;
pub mod tcp_parse;
pub mod tests;

extern crate core;

pub use crate::core::*;
pub use crate::encoding::*;
pub use crate::memory_storage::*;
pub use crate::page_io::*;
pub use crate::queue::queue::Queue;
pub use crate::serialiser::*;
pub use crate::tcp_parse::*;

pub mod serialiser {
    use crate::encoding::encoding::MmqpEncoding;
    use std::borrow::BorrowMut;

    pub enum SerialisationStrategy {
        Wire,
        Storage
    }

    pub trait MmqpSerialisable {
        fn serialise(&self,strategy: SerialisationStrategy) -> Box<[u8]>;
        fn deserialise(message_binary: &mut Vec<u8>, cursor: &mut usize) -> Self;
        fn raw(message_binary: &mut Vec<u8>, cursor: &mut usize) -> Vec<u8>;
        fn get_size(&self) -> usize;
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MmqpMessage {
        pub version_major: u8,
        pub version_minor: u8,
        pub username: String,
        pub password: String,
        pub target_queue: String,
        pub message: String,
        pub message_group: String,
    }

    impl MmqpMessage {
        pub fn new() -> MmqpMessage {
            MmqpMessage {
                password: "".to_string(),
                username: "".to_string(),
                message: "A message".to_string(),
                version_minor: 1,
                version_major: 0,
                message_group: "mg1".to_string(),
                target_queue: "queue1".to_string(),
            }
        }
    }

    impl MmqpSerialisable for MmqpMessage {
        fn serialise(&self,strategy: SerialisationStrategy) -> Box<[u8]> {
            let pipe = b"|";
            let auth = b":";
            let mut message_binary: Vec<u8> = Vec::new();
            message_binary.extend("MMQP".as_bytes());
            message_binary.extend(pipe);

            message_binary.push(self.version_major);

            message_binary.extend(".".as_bytes());

            message_binary.push(self.version_minor);

            message_binary.extend(pipe);

            message_binary.extend(b"M");

            message_binary.extend(pipe);

            message_binary.extend(self.username.to_mmqp_binary().unwrap());

            message_binary.extend(auth);

            message_binary.extend(self.password.to_mmqp_binary().unwrap());

            message_binary.extend(pipe);

            message_binary.extend(self.target_queue.to_mmqp_binary().unwrap());

            message_binary.extend(pipe);

            message_binary.extend(self.message_group.to_mmqp_binary().unwrap());

            message_binary.extend(pipe);

            message_binary.extend(self.message.to_mmqp_binary().unwrap());

            message_binary.push(0x00);


            return match strategy {
                SerialisationStrategy::Wire => {
                    message_binary.into_boxed_slice()
                }
                SerialisationStrategy::Storage => {
                    let size = message_binary.len();
                    let mut prefix = size.to_mmqp_binary().unwrap();
                    prefix.extend(message_binary);
                    prefix.into_boxed_slice()
                }
            }


        }

        fn deserialise(message_binary: &mut Vec<u8>, c: &mut usize) -> Self {
            usize::from_mmqp_binary(message_binary, c); //side effect: moves cursor

            let mut cursor = *c;
            cursor += 5; //skip MMQP and pipe

            let version_major = message_binary[cursor];
            cursor += 1;

            //skip dot
            cursor += 1;

            let version_minor = message_binary[cursor];
            cursor += 1;

            //skip pipe
            cursor += 1;
            //skip M
            cursor += 1;
            //skip pipe
            cursor += 1;

            let username = String::from_mmqp_binary(&message_binary, &mut cursor);

            cursor += 1; // colon

            let password = String::from_mmqp_binary(&message_binary, &mut cursor);

            cursor += 1; // pipe

            let target_queue = String::from_mmqp_binary(&message_binary, &mut cursor);

            cursor += 1; //pipe

            let message_group = String::from_mmqp_binary(&message_binary, &mut cursor);

            cursor += 1; //pipe

            let message = String::from_mmqp_binary(&message_binary, &mut cursor);
            cursor += 1; //null
            *c = cursor;
            MmqpMessage {
                version_major,
                version_minor,
                username,
                password,
                target_queue,
                message,
                message_group,
            }
        }

        fn raw(message_binary: &mut Vec<u8>, cursor: &mut usize) -> Vec<u8> {
            let size = usize::from_mmqp_binary(message_binary.borrow_mut(), cursor);
            let start = *cursor;
            *cursor += size;
            let end = *cursor;
            message_binary[start..end].to_vec()
        }

        fn get_size(&self) -> usize {
            let mut s = 0;
            s += 16; // fixed length headers

            s += self.password.mmqp_binary_size();
            s += self.target_queue.mmqp_binary_size();
            s += self.username.mmqp_binary_size();
            s += self.message_group.mmqp_binary_size();
            s += self.message.mmqp_binary_size();

            s
        }
    }
}

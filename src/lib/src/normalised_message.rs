pub mod normalised_message {
    use crate::encoding::encoding::MmqpEncoding;
    use crate::serialiser::{MmqpMessage, MmqpSerialisable};
    use std::borrow::BorrowMut;
    use std::time::SystemTime;
    use crate::SerialisationStrategy;

    pub struct MmqpNormalisedMessage {
        //original message field from Mmqp.message
        pub message: String,

        //auto generated unique ID - 64 random bytes, will be sent as a base64 string
        pub message_id: [u8; 64],

        pub message_group_id: String,
        pub received_time: u128,
        pub available_time: u128,
        pub receive_count: u32,
    }

    impl MmqpNormalisedMessage {
        pub fn new(message: MmqpMessage) -> MmqpNormalisedMessage {
            MmqpNormalisedMessage {
                message: message.message,
                message_id: [0; 64],
                message_group_id: message.message_group,
                received_time: 0,
                available_time: 0,
                receive_count: 1,
            }
        }
    }

    impl MmqpSerialisable for MmqpNormalisedMessage {
        fn serialise(&self, strategy:SerialisationStrategy) -> Box<[u8]> {
            let mut message_binary: Vec<u8> = Vec::new();
            message_binary.extend(self.message_id);
            message_binary.extend(self.received_time.to_be_bytes());
            message_binary.extend(self.available_time.to_be_bytes());
            message_binary.extend(self.receive_count.to_be_bytes());
            message_binary.extend(self.message_group_id.to_mmqp_binary().unwrap());
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
            *c += message_binary[*c] as usize;
            *c += 1;
            let mut cursor = *c;

            // 64 bytes for message id
            let mut message_id_binary = [0u8; 64];
            for i in 0..64 {
                message_id_binary[i] = message_binary[cursor];
                cursor += 1
            }

            let mut received_time_bytes = [0u8; 16];
            for i in 0..16 {
                received_time_bytes[i] = message_binary[cursor];
                cursor += 1
            }

            let mut available_time_bytes = [0u8; 16];
            for i in 0..16 {
                available_time_bytes[i] = message_binary[cursor];
                cursor += 1
            }

            let mut receive_count_bytes = [0u8; 4];
            for i in 0..4 {
                receive_count_bytes[i] = message_binary[cursor];
                cursor += 1
            }

            let message_group_id = String::from_mmqp_binary(message_binary, &mut cursor);
            let message = String::from_mmqp_binary(message_binary, &mut cursor);
            cursor += 1;

            *c = cursor;

            MmqpNormalisedMessage {
                message,
                message_id: message_id_binary,
                message_group_id,
                received_time: u128::from_be_bytes(received_time_bytes),
                available_time: u128::from_be_bytes(available_time_bytes),
                receive_count: u32::from_be_bytes(receive_count_bytes),
            }
        }

        fn raw(message_binary: &mut Vec<u8>, cursor: &mut usize) -> Vec<u8> {
            let start = *cursor;
            let size = usize::from_mmqp_binary(message_binary.borrow_mut(), cursor);
            *cursor += size;
            let end = *cursor;
            message_binary[start..end].to_vec()
        }

        fn get_size(&self) -> usize {
            let mut size = 1usize; // accounts for trailing null byte
            size += 64; // message id bytes
            size += 16; // 128bit received time
            size += 16; // 128bit available time
            size += 32 / 8; // receive count
            size += self.message_group_id.mmqp_binary_size();
            size += self.message.mmqp_binary_size();
            size
        }
    }

    pub trait Receivable {
        fn normalise(&self) -> MmqpNormalisedMessage;
        fn normalise_serialised(&self) -> Box<[u8]>;
    }

    impl Receivable for MmqpMessage {
        fn normalise(&self) -> MmqpNormalisedMessage {
            let current_time_ms = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            MmqpNormalisedMessage {
                message: self.message.clone(),
                available_time: current_time_ms,
                received_time: current_time_ms,
                message_id: [0; 64], //todo: generate a unique ID
                message_group_id: self.message_group.clone(),

                //receive refers to how many times it has been sent to a client
                //0 here because it has never been seent to a client
                //an implementation of this trait would increment this value when sentNormalised->normalised
                receive_count: 0,
            }
        }

        fn normalise_serialised(&self) -> Box<[u8]> {
            self.normalise().serialise(SerialisationStrategy::Wire)
        }
    }
}

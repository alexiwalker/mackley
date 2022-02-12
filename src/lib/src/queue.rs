pub mod queue {
    use crate::memory_storage::queue_readers::RotatingReadBuffers;
    use crate::normalised_message::normalised_message::{MmqpNormalisedMessage, Receivable};
    use crate::serialiser::MmqpSerialisable;
    use std::borrow::{Borrow, BorrowMut};
    use std::collections::{BTreeMap, HashMap};
    use std::net::TcpStream;
    use std::time::SystemTime;

    #[test]
    fn compiles() {
        println!("compiles");
    }

    pub struct Queue {
        queue_name: String,

        approximate_message_count: u64,
        pending_message_count: u64,

        // read: messages are taken from the pending received map first if they exist & are available
        // push: messages are sent to the end of the queue when they become available
        pending_mode: PendingMode,

        readers: RotatingReadBuffers<MmqpNormalisedMessage>,

        //message id -> message. If a message is in this map, it has been sent but a delete command has not been received yet
        pending_sent: HashMap<String, MmqpNormalisedMessage>,

        //message available time -> message. If a message is in this map, it has been received but cannot be added to the queue yet
        pending_received: BTreeMap<u128, Vec<MmqpNormalisedMessage>>,

        long_poll_connections: Vec<TcpStream>,
    }

    impl Queue {
        /**    getters  */
        pub fn name(self) -> String {
            self.queue_name
        }

        pub fn approximate_message_count(self) -> u64 {
            self.approximate_message_count
        }

        pub fn pending_mode(self) -> PendingMode {
            self.pending_mode
        }

        pub fn readers(self) -> RotatingReadBuffers<MmqpNormalisedMessage> {
            self.readers
        }

        pub fn pending_sent(self) -> HashMap<String, MmqpNormalisedMessage> {
            self.pending_sent
        }

        pub fn pending_received(self) -> BTreeMap<u128, Vec<MmqpNormalisedMessage>> {
            self.pending_received
        }

        pub fn long_poll_connections(self) -> Vec<TcpStream> {
            self.long_poll_connections
        }

        /**    mutators */
        pub fn receive_message(&mut self, message: impl Receivable) {
            let readers = self.readers.borrow_mut();
            let norm = message.normalise();
            let current_time_ms = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let available_time = norm.available_time;

            if current_time_ms < available_time {
                //message is not yet available
                self.pending_message_count += 1;
                if self.pending_received.contains_key(&available_time) {
                    self.pending_received
                        .get_mut(&available_time)
                        .unwrap()
                        .push(norm);
                } else {
                    self.pending_received.insert(available_time, vec![norm]);
                }
                return;
            }

            let binary = norm.serialise();

            self.approximate_message_count += 1;
            readers.push_raw(binary);
        }

        pub fn read_next(&mut self) -> Option<MmqpNormalisedMessage> {
            if self.pending_message_count > 0 {
                let current_time_ms = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let map = self.pending_received.borrow_mut();

                let mut has_key = false;
                let mut key = 0u128;
                for k in map.keys().into_iter() {
                    if *k <= current_time_ms {
                        has_key = true;
                        key = *k;
                        break;
                    }
                }

                if has_key {
                    let mut messages = map.remove(&key).unwrap();
                    self.pending_message_count -= 1;
                    return Some(messages.pop().unwrap());
                }
            }

            let readers = self.readers.borrow_mut();
            self.approximate_message_count -= 1;
            readers.next()
        }

        pub fn flush_pending(&mut self) {
            let mut map = self.pending_received.borrow_mut();
            let mut keys: Vec<u128> = vec![];
            for k in map.keys().into_iter() {
                if *k
                    <= SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                {
                    keys.push(*k);
                }
            }

            while keys.len() > 0 {
                let key = keys.pop().unwrap();
                let messages = map.remove(&key).unwrap();
                self.pending_message_count -= 1;
                self.approximate_message_count += 1;
                let readers = self.readers.borrow_mut();

                for message in messages.into_iter() {
                    readers.push_value(message);
                }
            }
        }
    }

    pub enum PendingMode {
        Read,
        Push,
    }
}

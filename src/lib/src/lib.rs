mod memory_storage;

extern crate core;

#[cfg(test)]
mod tests {
	use crate::serialiser::{MmqpEncoding, MmqpMessage, MmqpSerialisable};

	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}

	#[test]
	fn check_string() {
		let string: String = "This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!".to_string();
		let bytes = string.to_mmqp_binary();

		println!("{:?}", bytes)
	}

	#[test]
	fn decode_string() {
		let first: String = "This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!".to_string().repeat(1000).repeat(1423);
		let second: String = "this string is appended to the previous on the packed binary format".to_string().repeat(55).repeat(13);
		let mut bytes = first.to_mmqp_binary().unwrap();
		bytes.extend(second.to_mmqp_binary().unwrap());
		let mut cursor = 0usize;
		let string2 = String::from_mmqp_binary(&bytes, &mut cursor);
		let string3 = String::from_mmqp_binary(&bytes, &mut cursor);

		assert_eq!(string2.to_mmqp_binary().unwrap().len(), string2.mmqp_binary_size());

		assert_eq!(first, string2, "The first string is not properly extracted");
		assert_eq!(string3, second, "The first string is not properly extracted");
		assert_eq!(cursor, bytes.len(), "The cursor is not at the end of the second string after extracting both");
	}

	#[test]
	fn encode_message() {
		let message = MmqpMessage {
			username: "testusername".to_string(),
			password: "testpassword".to_string(),
			target_queue: "testqueue".to_string(),
			message: "this is a message".to_string(),
			version_major: 0,
			version_minor: 1,
			message_group: "mainmessagegroup".to_string(),
		};

		let binary = message.serialise();
		let mut cursor = 0usize;
		let recreated = MmqpMessage::deserialise(&binary, &mut cursor);

		assert_eq!(message, recreated)
	}
}


pub mod serialiser {
	use std::time::SystemTime;

	pub trait MmqpSerialisable {
		fn serialise(&self) -> Box<[u8]>;
		fn deserialise(message_binary: &[u8], cursor: &mut usize) -> Self;
		fn get_size(&self) -> usize;
	}

	pub trait MmqpEncoding {
		fn to_mmqp_binary(&self) -> Result<Vec<u8>, String>;
		fn from_mmqp_binary(message_binary: &[u8], cursor: &mut usize) -> Self;
		fn mmqp_binary_size(&self) -> usize;
	}

	impl MmqpEncoding for String {
		fn to_mmqp_binary(&self) -> Result<Vec<u8>, String> {
			let length = self.len();

			if length == 0 {
				return Result::Ok(vec![0]);
			}

			let mut binary: Vec<u8> = Vec::new();
			let required_size: u8 = match length {
				0 => 0,
				1..=255 => 1,
				256..=65535 => 2,
				65537..=16777215 => 3,
				16777216..=4294967295 => 4,
				429496726..=1099511627775 => 5,
				1099511627776..=281474976710655 => 6,
				281474976710656..=72057594037927935 => 7,
				72057594037927936..=18446744073709551615 => 8,
				_ => {
					return Result::Err(format!("String cannot be encoded as MMQP, it is too long"));
				}
			};

			binary.push(required_size);
			let bytes = length.to_le_bytes();
			for i in (0..required_size).rev() {
				binary.push(bytes[i as usize]);
			}
			binary.extend(self.as_bytes());

			Result::Ok(binary)
		}

		fn from_mmqp_binary(message_binary: &[u8], cursor: &mut usize) -> Self {
			let size_for_size = message_binary[*cursor];
			*cursor += 1;

			if size_for_size == 0 {
				return String::new();
			}

			let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
			for i in 0..size_for_size {
				bytes[8 - (size_for_size - i) as usize] = message_binary[*cursor];
				*cursor += 1
			}
			let actual_size = usize::from_be_bytes(bytes);
			let from = *cursor;
			*cursor += actual_size;
			String::from_utf8(message_binary[from..*cursor].to_vec()).unwrap()
		}

		fn mmqp_binary_size(&self) -> usize {
			let mut size = 0;

			if self.len() == 0 {
				return 1;
			}

			//size for size byte
			size +=1;

			//the actual number of bytes for the size
			size += match self.len() {
				0 => 0,
				1..=255 => 1,
				256..=65535 => 2,
				65537..=16777215 => 3,
				16777216..=4294967295 => 4,
				429496726..=1099511627775 => 5,
				1099511627776..=281474976710655 => 6,
				281474976710656..=72057594037927935 => 7,
				72057594037927936..=18446744073709551615 => 8,
				_ => {
					panic!("String cannot be encoded as MMQP, it is too long");
				}
			};

			//the size of the string itself
			size += self.len();
			size
		}
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

	pub struct MmqpNormalisedMessage {
		//original message field from Mmqp.message
		message: String,

		//auto generated unique ID - 64 random bytes, will be sent as a base64 string
		message_id: [u8; 64],

		message_group_id: String,
		received_time: u128,
		available_time: u128,
		receive_count: u32,
	}

	impl MmqpSerialisable for MmqpNormalisedMessage{
		fn serialise(&self) -> Box<[u8]> {

			let mut message_binary: Vec<u8> = Vec::new();
			message_binary.extend(self.message_id);
			message_binary.extend(self.received_time.to_be_bytes());
			message_binary.extend(self.available_time.to_be_bytes());
			message_binary.extend(self.receive_count.to_be_bytes());
			message_binary.extend(self.message_group_id.to_mmqp_binary().unwrap());
			message_binary.extend(self.message.to_mmqp_binary().unwrap());
			message_binary.push(0x00);
			message_binary.into_boxed_slice()
		}

		fn deserialise(message_binary: &[u8], c: &mut usize) -> Self {
			let mut cursor = *c;

			// 64 bytes for message id
			let mut message_id_binary = [0u8; 64];
			for i in 0..64 {
				message_id_binary[i] = message_binary[cursor];
				cursor+=1
			}

			let mut received_time_bytes = [0u8; 16];
			for i in 0..16 {
				received_time_bytes[i] = message_binary[cursor];
				cursor+=1
			}

			let mut available_time_bytes = [0u8; 16];
			for i in 0..16 {
				available_time_bytes[i] = message_binary[cursor];
				cursor+=1
			}

			let mut receive_count_bytes = [0u8; 4];
			for i in 0..16 {
				receive_count_bytes[i] = message_binary[cursor];
				cursor+=1
			}

			let message_group_id = String::from_mmqp_binary(message_binary,&mut cursor);
			let message = String::from_mmqp_binary(message_binary,&mut cursor);



			*c = cursor;

			MmqpNormalisedMessage {
				message,
				message_id: message_id_binary,
				message_group_id,
				received_time: u128::from_be_bytes(received_time_bytes),
				available_time: u128::from_be_bytes(available_time_bytes),
				receive_count:u32::from_be_bytes(receive_count_bytes),
			}
		}

		fn get_size(&self) -> usize {
			let mut size = 1usize; // accounts for trailing null byte
			size += 64; // message id bytes
			size += 16; // 128bit received time
			size += 16; // 128bit available time
			size += 32/8; // receive count
			size += self.message_group_id.mmqp_binary_size();
			size += self.message.mmqp_binary_size();
			size
		}
	}

	pub trait Receivable {
		fn normalise(&self) -> MmqpNormalisedMessage;
	}

	impl Receivable for MmqpMessage {
		fn normalise(&self) -> MmqpNormalisedMessage {
			let current_time_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();

			MmqpNormalisedMessage {
				message: self.message.clone(),
				available_time: current_time_ms,
				received_time: current_time_ms,
				message_id: [0; 64], //todo: generate a unique ID
				message_group_id: self.message_group.clone(),
				receive_count: 1,
			}
		}
	}

	pub trait Storable {
		fn get_storage_size_bytes(&self) -> usize;
		fn get_storage_bytes(&self) -> Box<[u8]>;
	}

	impl Storable for MmqpNormalisedMessage {
		fn get_storage_size_bytes(&self) -> usize {
			let mut size = 8usize; // size of a usize, required to store size before message
			size += 64; // message id
			size += 128 / 8; //received time ms
			size += 128 / 8; //available time ms
			size += 4; //receive count
			size += self.message_group_id.mmqp_binary_size(); //message group id
			size += self.message.mmqp_binary_size(); //message group id
			size
		}

		fn get_storage_bytes(&self) -> Box<[u8]> {
			let size = self.get_storage_size_bytes();
			let mut bytes: Vec<u8> = Vec::new();
			bytes.resize(size, 0);

			bytes.extend(size.to_be_bytes());
			bytes.extend(self.message_id);
			bytes.extend(self.received_time.to_be_bytes());
			bytes.extend(self.available_time.to_be_bytes());
			bytes.extend(self.receive_count.to_be_bytes());
			bytes.extend(self.message_group_id.to_mmqp_binary().unwrap());
			bytes.extend(self.message.to_mmqp_binary().unwrap());

			Box::from(bytes)
		}
	}

	impl MmqpSerialisable for MmqpMessage {
		fn serialise(&self) -> Box<[u8]> {
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


			message_binary.into_boxed_slice()
		}

		fn deserialise(message_binary: &[u8], c: &mut usize) -> Self {
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
			cursor+=1; //null
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
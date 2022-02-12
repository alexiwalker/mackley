pub mod tests {
	use std::borrow::BorrowMut;
	use crate::encoding::encoding::MmqpEncoding;
	use crate::memory_storage::queue_readers::RotatingReadBuffers;
	use crate::normalised_message::normalised_message::{MmqpNormalisedMessage, Receivable};
	use crate::serialiser::{ MmqpMessage, MmqpSerialisable};
	#[test]
	fn test() {
		println!("compiles")
	}


	#[test]
	fn push_messages() {
		let mut readers: RotatingReadBuffers<MmqpMessage> = RotatingReadBuffers::new(4, 65536);

		readers.push_value(MmqpMessage::new());
		let val = readers.next();
		assert_eq!(val.is_some(), true, "A message exists. should be some");

		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");

		readers.push_value(MmqpMessage::new());
		assert_ne!(val.is_some(), true, "Another message has been added. Should be some");

	}

	#[test]
	fn test_serialise_size() {

		let message =MmqpMessage::new().normalise();

		let bytes = message.serialise();

		println!("size after serialise {}", bytes.len());

		let raw = MmqpNormalisedMessage::raw(bytes.to_vec().borrow_mut(), &mut 0);

		assert_eq!(raw.len(),bytes.len());
	}

	#[test]
	fn raw_receive() {

		let message =MmqpMessage::new().normalise();

		let bytes = message.serialise().to_vec();

		let raw = MmqpNormalisedMessage::raw(bytes.to_vec().borrow_mut(), &mut 0);


		let mut readers: RotatingReadBuffers<MmqpNormalisedMessage> = RotatingReadBuffers::new(4, 65536);

		readers.push_value(message);
		let val = readers.next_raw();
		assert_eq!(val.is_some(), true, "A message exists. should be some");
		let v = val.unwrap();
		assert_eq!(v.len(), raw.len(), "Read raw length should be same as original raw length");

		let val = readers.next_raw();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");


		let message =MmqpMessage::new().normalise();
		readers.push_value(message);
		let message =MmqpMessage::new().normalise();
		readers.push_value(message);
		let message =MmqpMessage::new().normalise();
		readers.push_value(message);
		let message =MmqpMessage::new().normalise();
		readers.push_value(message);


		let val = readers.next();
		assert_eq!(val.is_some(), true, "should have 4 messages here");
		let val = readers.next();
		assert_eq!(val.is_some(), true, "should have 4 messages here");
		let val = readers.next();
		assert_eq!(val.is_some(), true, "should have 4 messages here");
		let val = readers.next();
		assert_eq!(val.is_some(), true, "should have 4 messages here");



		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");
		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");
		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");
		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");
		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");
		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");
		let val = readers.next();
		assert_eq!(val.is_none(), true, "The message has been read. There are none left. Should be none");


	}



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
		let first: String = "This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!This is a string i wish to encode!".to_string().repeat(150).repeat(50);
		let second: String = "this string is appended to the previous on the packed binary format".to_string().repeat(55).repeat(13);

		println!("length: {:?}", first.clone().to_mmqp_binary().unwrap().len());
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
		let recreated = MmqpMessage::deserialise(binary.to_vec().borrow_mut(), &mut cursor);

		assert_eq!(message, recreated)
	}
}
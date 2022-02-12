pub mod encoding {

    pub trait MmqpEncoding {
        fn to_mmqp_binary(&self) -> Result<Vec<u8>, String>;
        fn from_mmqp_binary(message_binary: &[u8], cursor: &mut usize) -> Self;
        fn mmqp_binary_size(&self) -> usize;
        fn raw(message_binary: &[u8], cursor: &mut usize) -> Vec<u8>;
    }

    impl MmqpEncoding for usize {
        fn to_mmqp_binary(&self) -> Result<Vec<u8>, String> {
            let mut binary: Vec<u8> = Vec::with_capacity(9);

            let length = self;
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
                _ => return Err("Impossible usize value".to_string()),
            };

            binary.push(required_size);
            let bytes = length.to_le_bytes();
            for i in (0..required_size).rev() {
                binary.push(bytes[i as usize]);
            }

            binary.shrink_to_fit();

            Ok(binary)
        }

        fn from_mmqp_binary(message_binary: &[u8], cursor: &mut usize) -> Self {
            let mut from = *cursor;
            let size_for_size = message_binary[from];
            from += 1;

            println!("{}", size_for_size);

            if size_for_size == 0 {
                return 0;
            }

            let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
            for i in 0..size_for_size {
                bytes[8 - (size_for_size - i) as usize] = message_binary[from];
                from += 1
            }

            *cursor = from;
            usize::from_be_bytes(bytes)
        }

        fn mmqp_binary_size(&self) -> usize {
            let length = self;
            let required_size: usize = match length {
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
                    panic!("Impossible usize value")
                }
            };
            //n bytes to encode actual bytes, 1 to mark number of bytes
            required_size + 1
        }

        fn raw(message_binary: &[u8], cursor: &mut usize) -> Vec<u8> {
            let value = message_binary[*cursor];
            *cursor += 1;

            //get next N bytes from message
            let mut vec = Vec::with_capacity(value as usize);
            for _ in 0..value {
                vec.push(message_binary[*cursor]);
                *cursor += 1;
            }

            vec
        }
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
                    return Result::Err(format!(
                        "String cannot be encoded as MMQP, it is too long"
                    ));
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
            size += 1;

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

        fn raw(message_binary: &[u8], cursor: &mut usize) -> Vec<u8> {
            let size_for_size = message_binary[*cursor];
            *cursor += 1;

            if size_for_size == 0 {
                return vec![0];
            }

            let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
            for i in 0..size_for_size {
                bytes[8 - (size_for_size - i) as usize] = message_binary[*cursor];
                *cursor += 1
            }
            let actual_size = usize::from_be_bytes(bytes);
            let from = *cursor;
            *cursor += actual_size;
            message_binary[from..*cursor].to_vec()
        }
    }
}

pub mod memory_storage {
	use std::borrow::{ BorrowMut};
	use std::marker::PhantomData;
	use std::sync::{Mutex, MutexGuard};
	use crate::serialiser::{MmqpMessage, MmqpSerialisable};

	#[test]
	fn test() {
		println!("compiles")
	}


	#[test]
	fn checkencodinglengths() {
		let v = MmqpMessage::new();
		let s = v.serialise();
		println!("{:?}", s.len());
		let s2 = v.get_size();
		println!("{:?}", s2);
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

	pub struct RotatingReadBuffers<T: MmqpSerialisable> {
		//wrap the buffer in a mutex
		pub buffers: Vec<Mutex<ReadBuffer<T>>>,

		buffer_size: usize,
		// reading to the end of the current buffer will move this to the next buffer
		pub current_read_buffer: usize,
		pub current_read_buffer_cursor: usize,

		// writing and reaching the end of this buffer will move to the next
		// if the next buffer is the current read buffer, a new buffer will be created to hold the data
		pub current_write_buffer: usize,
		pub num_buffers: usize,
	}

	pub struct ReadBuffer<T: MmqpSerialisable> {
		pub buffer: Vec<u8>,
		pub cursor: usize,
		_p: PhantomData<T>,
	}

	impl<T: MmqpSerialisable> RotatingReadBuffers<T> {
		pub fn new(num_buffers: usize, buffer_size: usize) -> RotatingReadBuffers<T> {
			let mut buffers = Vec::new();
			for _ in 0..num_buffers {
				let b = ReadBuffer::new(buffer_size);
				buffers.push(Mutex::new(b));
			}
			RotatingReadBuffers {
				buffers,
				current_read_buffer: 0,
				current_read_buffer_cursor: 0,
				current_write_buffer: 0,
				num_buffers,
				buffer_size
			}
		}

		pub fn push_value(&mut self, value: T) {
			let mut buffer = self.buffers[self.current_write_buffer].lock().unwrap();
			let size = value.get_size();
			if buffer.has_capacity(size) {
				buffer.push_value(value);
			} else {
				drop(buffer);
				if self.next_write_buffer() == self.current_read_buffer {
					let mut buffer =self.add_and_go_next();
					buffer.push_value(value);
				} else {
					let mut buffer = self.go_next();
					buffer.push_value(value);
				}
			}
		}

		pub fn add_and_go_next(&mut self)->MutexGuard<ReadBuffer<T>>{
			self.num_buffers += 1;
			self.current_write_buffer += 1;
			let new_buffer = Mutex::new(ReadBuffer::<T>::new(self.buffer_size));
			self.buffers.push(new_buffer);

			self.get_writer()
		}

		pub fn go_next(&mut self)->MutexGuard<ReadBuffer<T>>{
			self.current_write_buffer = self.next_write_buffer();
			self.get_writer()
		}

		pub fn get_writer(&mut self) -> MutexGuard<ReadBuffer<T>> {
			self.buffers[self.current_write_buffer].lock().unwrap()
		}

		pub fn next_read_buffer(&mut self) -> usize {
			if self.current_read_buffer + 1 == self.num_buffers {
				0
			} else {
				self.current_read_buffer + 1
			}
		}

		pub fn next_write_buffer(&mut self) -> usize {
			if self.current_write_buffer + 1 == self.num_buffers {
				0
			} else {
				self.current_write_buffer + 1
			}
		}

		pub fn has_next(&mut self) -> bool {
			let next = self.next_read_buffer();
			let current_buffer = self.buffers[self.current_read_buffer].lock().unwrap();
			if current_buffer.cursor == current_buffer.buffer.len() {
				drop(current_buffer);
				let next_buffer = self.buffers[next].lock().unwrap();
				return if next_buffer.cursor != 0 {
					drop(next_buffer);
					true
				} else {
					false
				}
			}

			true
		}

		pub fn next(&mut self) -> Option<T> {
			let mut buffer = self.buffers[self.current_read_buffer].lock().unwrap();
			let value = buffer.next();

			match value {
				Some(val) => {
					Some(val)
				}
				None => {
					drop(buffer);
					if self.has_next() {
						self.current_read_buffer_cursor = 0;
						self.current_read_buffer = self.next_read_buffer();
						let mut buffer = self.buffers[self.current_read_buffer].lock().unwrap();
						let value = buffer.next();
						value
					} else {
						None
					}
				}
			}
		}
	}

	impl<T: MmqpSerialisable> ReadBuffer<T> {
		pub fn new(size: usize) -> ReadBuffer<T> {
			ReadBuffer {
				buffer: Vec::with_capacity(size),
				cursor: 0,
				_p: PhantomData,
			}
		}

		pub fn next(&mut self) -> Option<T> {

			//buffer is empty, return error
			if self.buffer.len() == 0 {
				return None;
			}

			// if the cursor is at the end of the buffer, clear the current read-buffer and move to the next buffer
			if self.cursor == self.buffer.len() {
				self.buffer.clear();
				self.cursor = 0;
				return None;
			}

			Some(T::deserialise(&mut self.buffer, self.cursor.borrow_mut()))
		}

		pub fn push_value(&mut self, value: T) {
			let bytes = value.serialise();
			self.buffer.extend(bytes.to_vec());
		}

		pub fn has_capacity(&self, size: usize) -> bool {
			if (self.buffer.len() + size) > self.buffer.capacity() {
				return false;
			}
			true
		}
	}


	// pages are buffers on disk, flushed directly from a vec<u8>. reading the page will reconstruct the buffer. only the id is required to find and read the page

	pub trait PageWritable {
		fn write_page(&mut self, page_id: u32);
	}

	pub trait PageReadable {
		fn read_page(&mut self, page_id: u32) -> Self;
	}
}
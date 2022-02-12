pub mod page_io {
	use crate::serialiser::MmqpSerialisable;

	pub trait Paged<T:MmqpSerialisable> {
		fn write_page(&mut self, page_id: u32);
		fn read_page(page_id: u32) -> Self;
	}

}
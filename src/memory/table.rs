use memory::entry::*;
use memory::pagetable::ENTRY_COUNT;
use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use memory::FrameAllocator;

pub trait TableLevel {}
pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel : TableLevel {
	type NextLevel: TableLevel;
}
impl HierarchicalLevel for Level4 {
	type NextLevel = Level3;
}
impl HierarchicalLevel for Level3 {
	type NextLevel = Level2;
}
impl HierarchicalLevel for Level2 {
	type NextLevel = Level1;
}

pub struct Table<L: TableLevel> {
	entries: [Entry; ENTRY_COUNT],
	level: PhantomData<L>,
}

impl<L> Table<L> where L: TableLevel {
	pub fn zero(&mut self) {
		for entry in self.entries.iter_mut() {
			entry.set_unused();
		}
	}
}

impl<L> Table<L> where L: HierarchicalLevel {
	//returns the full address (u64) of the next page table down the page table hierarchy (e.g. p4 -> p3)
	fn next_table_address(&self, index: usize) -> Option<usize> {
		let entry_flags = self[index].flags();
		if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
			let table_address = self as *const _ as usize;
			Some((table_address << 9) | (index << 12)) // shift along the table address bits and add in the new index at the end
			//P4	0o177777_777_777_777_777_0000	–
			//P3	0o177777_777_777_777_XXX_0000	XXX is the P4 index
			//P2	0o177777_777_777_XXX_YYY_0000	like above, and YYY is the P3 index
			//P1	0o177777_777_XXX_YYY_ZZZ_0000	like above, and ZZZ is the P2 index
		} else {
			None
		}
	}

	pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
		self.next_table_address(index)
			.map(|address| unsafe { &*(address as *const _) })
	}

	pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
		self.next_table_address(index)
			.map(|address| unsafe { &mut *(address as *mut _) })
	}

	pub fn next_table_create<A>(&mut self, index: usize, allocator: &mut A) -> &mut Table<L::NextLevel>
		where A : FrameAllocator {
		//do we have a page table entry already available for this index?
		if self.next_table(index).is_none() {
			assert!(!self.entries[index].flags().contains(HUGE_PAGE), "huge pages unsupported");
			let frame = allocator.allocate_frame().expect("no frames available");
			self.entries[index].set(frame, PRESENT | WRITABLE);
		}
		self.next_table_mut(index).unwrap()
	}
}

impl<L> Index<usize> for Table<L> where L : TableLevel {
	type Output = Entry;

	fn index(&self, index: usize) -> &Entry {
		&self.entries[index]
	}
}

impl<L> IndexMut<usize> for Table<L> where L : TableLevel {
	fn index_mut(&mut self, index: usize) -> &mut Entry {
		&mut self.entries[index]
	}
}


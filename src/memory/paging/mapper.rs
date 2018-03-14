use memory::{PAGE_SIZE, FrameAllocator};
use core::ptr::Unique;
use x86::structures::paging::*;
use x86::instructions::tlb;
use x86::*;
use super::paging::table::RecTable;

// virtual address of recursively mapped P2
// for protected mode non PAE
// https://wiki.osdev.org/Page_Tables
pub const P2: *mut PageTable = 0xffff_f000 as *mut _;

pub struct Mapper {
    p2: Unique<PageTable>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p2: Unique::new_unchecked(self::P2),
        }
    }

    // the remaining mapping methods, all public
    pub fn p2(&self) -> &PageTable {
        unsafe { self.p2.as_ref() }
    }

    pub fn p2_mut(&mut self) -> &mut PageTable {
        unsafe { self.p2.as_mut() }
    }

    /// virtual addr to physical addr translation
    pub fn translate(&self, virtual_address: VirtAddr) -> Option<PhysAddr>
    {
        let offset = virtual_address.as_u32() % PAGE_SIZE as u32;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.start_address() + offset)

    }

    /// virtual page to physical frame translation
    pub fn translate_page(&self, page: Page) -> Option<PhysFrame> {
        let p1 = self.p2().next_table(page.p2_index());

        let huge_page = || {
            let p2_entry = &self.p2()[page.p2_index()];
            if let Some(start_frame) = p2_entry.pointed_frame() {
                if p2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    // TODO 4MiB alignment check
                    return Some(start_frame + u32::from(page.p1_index()));
                    }
            }
            None
        }; 

        p1.and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page) 
    }

    /// map a virtual page to a physical frame in the page tables
    pub fn map_to<A>(&mut self, page: Page, frame: PhysFrame, flags: PageTableFlags,
                     allocator: &mut A)
        where A: FrameAllocator
        {
            let p2 = self.p2_mut();
            let p1 = p2.next_table_create(page.p2_index(), allocator);

            assert!(p1[page.p1_index()].is_unused());
            p1[page.p1_index()].set(frame, flags | PageTableFlags::PRESENT);
        }

    pub fn map<A>(&mut self, page: Page, flags: PageTableFlags, allocator: &mut A)
        where A: FrameAllocator
        {
            let frame = allocator.allocate_frame().expect("out of memory");
            self.map_to(page, frame, flags, allocator)
        }

    pub fn identity_map<A>(&mut self, frame: PhysFrame, flags: PageTableFlags, allocator: &mut A)
        where A: FrameAllocator
        {
            let virt_addr = VirtAddr::new(frame.start_address().as_u32());
            let page = Page::containing_address(virt_addr);
            self.map_to(page, frame, flags, allocator);
        }

    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
        {
            assert!(self.translate(page.start_address()).is_some());

            let p1 = self.p2_mut()
                .next_table_mut(page.p2_index())
                .expect("mapping code does not support huge pages");
            let frame = p1[page.p1_index()].pointed_frame().unwrap();
            p1[page.p1_index()].set_unused();
            tlb::flush(page.start_address());
            // TODO
            // allocator.deallocate_frame(frame);
        }
}

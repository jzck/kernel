#![allow(dead_code)]

mod table;
mod temporary_page;
mod mapper;

use memory::*;
use self::mapper::Mapper;
use self::temporary_page::TemporaryPage;
use core::ops::{Deref, DerefMut};
use multiboot2::BootInformation;
use x86::*;
use x86::registers::control::Cr3;
use x86::instructions::tlb;

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temporary_page: &mut temporary_page::TemporaryPage,
                   f: F)
        where F: FnOnce(&mut Mapper)
        {
            let (cr3_back, _cr3flags_back) = Cr3::read();

            // map temp page to current p2
            let p2_table = temporary_page.map_table_frame(cr3_back.clone(), self);

            // overwrite recursive map
            self.p2_mut()[1023].set(table.p2_frame.clone(), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
            tlb::flush_all();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p2 table
            p2_table[1023].set(cr3_back, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
        }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {

        let (p2_frame, cr3_flags) = Cr3::read();
        let old_table = InactivePageTable { p2_frame };

        // unsafe { Cr3::write(new_table.p2_frame, cr3_flags); }

        ::console::regs();
        flush!();
        loop {}

        unsafe { asm!("mov $0, %cr3" :: "r" (4096) : "memory"); }
    
        // let addr = new_table.p2_frame.start_address();
        // let value = addr.as_u32() | cr3_flags.bits();
        // println!("value = {}", 0);
        // flush!();
        // loop {}

        old_table
    }
}

pub struct InactivePageTable {
    p2_frame: PhysFrame,
}

impl InactivePageTable {
    pub fn new(frame: PhysFrame,
               active_table: &mut ActivePageTable,
               temporary_page: &mut TemporaryPage)
        -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);

            table.zero();
            // set up recursive mapping for the table
            table[1023].set(frame.clone(), PageTableFlags::PRESENT | PageTableFlags::WRITABLE)
        }
        temporary_page.unmap(active_table);
        InactivePageTable { p2_frame: frame }
        }
}

pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation)
    -> ActivePageTable
    where A: FrameAllocator
{
    let mut temporary_page = TemporaryPage::new(Page{number: 0xcafe}, allocator);
    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };


    active_table.with(&mut new_table, &mut temporary_page, |mapper| {

        // identity map the VGA text buffer
        let vga_buffer_frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
        mapper.identity_map(vga_buffer_frame, PageTableFlags::WRITABLE, allocator);

        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("Memory map tag required");

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                continue;
            }
            assert!(section.start_address() % PAGE_SIZE as u64 == 0,
            "sections need to be page aligned");

            let flags = elf_to_pagetable_flags(&section.flags());
            let start_frame = PhysFrame::containing_address(
                PhysAddr::new(section.start_address() as u32));
            let end_frame = PhysFrame::containing_address(
                PhysAddr::new(section.end_address() as u32 - 1));
            for frame in start_frame..end_frame {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        let multiboot_start = PhysFrame::containing_address(
            PhysAddr::new(boot_info.start_address() as u32));
        let multiboot_end = PhysFrame::containing_address(
            PhysAddr::new(boot_info.end_address() as u32 - 1));
        for frame in multiboot_start..multiboot_end {
            mapper.identity_map(frame, PageTableFlags::PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);

    println!("check!");
    flush!();
    loop {}

    let old_p2_page = Page::containing_address(
        VirtAddr::new(old_table.p2_frame.start_address().as_u32()));

    active_table.unmap(old_p2_page, allocator);

    active_table
}

fn elf_to_pagetable_flags(elf_flags: &multiboot2::ElfSectionFlags) 
    -> PageTableFlags
{
    use multiboot2::ElfSectionFlags;

    let mut flags = PageTableFlags::empty();

    if elf_flags.contains(ElfSectionFlags::ALLOCATED) {
        // section is loaded to memory
        flags = flags | PageTableFlags::PRESENT;
    }
    if elf_flags.contains(ElfSectionFlags::WRITABLE) {
        flags = flags | PageTableFlags::WRITABLE;
    }

    // LONG MODE STUFF
    // if !elf_flags.contains(ELF_SECTION_EXECUTABLE) {
    //     flags = flags | PageTableFlags::NO_EXECUTE;
    // }

    flags
}

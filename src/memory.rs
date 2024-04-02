use x86_64::{
    structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr,
    VirtAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next : usize,
}

  

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map : &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next : 0,
        }
    }

    fn usable_frame(&self) -> impl Iterator<Item = PhysFrame> {
        let regins = self.memory_map.iter();
        let usable_regins = regins.filter(
                        |s| s.region_type == MemoryRegionType::Usable
        );
        let addr_regins = usable_regins.map(
                        |s| s.range.start_addr()..s.range.end_addr()
        );

        let frame_aaddresses = addr_regins.flat_map(
                        |addr| addr.step_by(4096)
        );
        let result = frame_aaddresses.map(
                        |addr| PhysFrame::containing_address(PhysAddr::new(addr))
        );
        result
   }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frame().nth(self.next);
        self.next += 1;
        frame
    }
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}


pub unsafe fn init(physical_memory_offset : VirtAddr) 
    -> OffsetPageTable<'static> 
{
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub fn crate_example_mapping(page : Page, 
                             mapper : &mut OffsetPageTable,
                             frame_allocator : &mut impl FrameAllocator<Size4KiB>)
{
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

unsafe fn active_level_4_table(physical_memory_offset : VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr : *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub unsafe fn translate_addr(addr : VirtAddr, physical_memory_offset : VirtAddr)
    -> Option<PhysAddr>
{
    let translate_addr_inner = (|addr : VirtAddr, physical_memory_offset : VirtAddr| 
        -> Option<PhysAddr> {
        use x86_64::registers::control::Cr3;
        use x86_64::structures::paging::page_table::FrameError;

        let (level_4_table_frame, _) = Cr3::read();
        let table_indexs = [
            addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
        ];

        let mut frame = level_4_table_frame;

        for index in table_indexs {
            let virt : VirtAddr = physical_memory_offset + frame.start_address().as_u64();
            let page_table_ptr : *const PageTable = virt.as_ptr();
            let page_table = unsafe{ &*page_table_ptr};

            let entry = &page_table[index];
            frame = match entry.frame() {
                Ok(frame) => frame,
                Err(FrameError::HugeFrame) => panic!("Huge frame not supposed"),
                Err(FrameError::FrameNotPresent) => return None,
            }
        }

        Some(frame.start_address() + u64::from(addr.page_offset()))
    });

    translate_addr_inner(addr, physical_memory_offset)
}

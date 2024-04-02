use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Size4KiB, Mapper, Page, PageTableFlags, PhysFrame, mapper::MapToError
    },
};


pub const HEAP_START : usize = 0x_4444_4444_0000;
pub const HEAP_SIZE : usize = 100 * 1024;

pub fn init_heap(mapper : &mut impl Mapper<Size4KiB>, 
    frame_allocator : &mut impl FrameAllocator<Size4KiB>)
    -> Result<(), MapToError<Size4KiB>>
{
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::WRITABLE | PageTableFlags::PRESENT;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe { 
        let heap_start = VirtAddr::new(HEAP_START as u64);
        super::ALLOCATOR.lock().init(heap_start.as_mut_ptr(), HEAP_SIZE);
     }

    Ok(())
}



use zodiac::mem::DefaultAllocator;

#[zodiac::global_allocator]
static ALLOCATOR: DefaultAllocator = DefaultAllocator::new();

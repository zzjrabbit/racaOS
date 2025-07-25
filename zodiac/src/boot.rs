use limine::BaseRevision;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

unsafe extern "Rust" {
    fn __zodiac_main() -> !;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {
    super::init();

    unsafe {
        __zodiac_main();
    }
}

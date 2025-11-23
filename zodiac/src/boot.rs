use limine::BaseRevision;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(4);

unsafe extern "Rust" {
    fn __zodiac_main() -> !;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    super::init();

    unsafe {
        __zodiac_main();
    }
}

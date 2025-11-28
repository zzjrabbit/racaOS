#[macro_export]
macro_rules! init_modules {
    ($($name:ident),* $(,)?) => {
        {
            #[used]
            #[unsafe(link_section = ".requests")]
            static KERNEL_MODULES: limine::request::ModuleRequest = limine::request::ModuleRequest::new()
                .with_internal_modules(&[
                    $(
                        &limine::modules::InternalModule::new().with_path(unsafe {
                            core::ffi::CStr::from_bytes_with_nul_unchecked(
                                concat!("modules/", stringify!($name), ".km\0").as_bytes()
                            )
                        })
                    ),*
                ]);

            fn initializer() -> Result<(), zodiac::ZodiacError> {
                let response = KERNEL_MODULES.get_response().unwrap();
                for file in response.modules() {
                    let data = unsafe { core::slice::from_raw_parts_mut(file.addr(), file.size() as usize) };

                    $crate::module::Module::load(data)?;
                }

                Ok(())
            }

            initializer()
        }
    };
}

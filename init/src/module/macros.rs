#[macro_export]
macro_rules! files {
    ($name: ident, $($path: literal),* $(,)?) => {
        #[used]
        #[unsafe(link_section = ".requests")]
        static $name: limine::request::ModuleRequest = limine::request::ModuleRequest::new()
            .with_internal_modules(&[
                $(
                    &limine::modules::InternalModule::new().with_path($path)
                ),*
            ]);
    };
}

#[macro_export]
macro_rules! init_modules {
    ($request: ident) => {{
        fn initializer() -> Result<(), zodiac::ZodiacError> {
            let response = $request.get_response().unwrap();
            for file in response.modules().iter() {
                let Ok(path) = file.path().to_str() else {
                    continue;
                };
                if !path.ends_with(".km") {
                    continue;
                }

                let data =
                    unsafe { core::slice::from_raw_parts_mut(file.addr(), file.size() as usize) };

                $crate::module::Module::load(data)?;
            }

            Ok(())
        }

        initializer()
    }};
}

// native unit tests link without the game, so a test that reaches it fails loudly instead of the linker

#[unsafe(no_mangle)]
extern "C" fn host_call(_pointer: u32, _length: u32) -> u64 { panic!("a unit test called into the game host") }

#[unsafe(no_mangle)]
extern "C" fn host_log(_level: u32, _pointer: u32, _length: u32) {}

use wasm2glulx_spectest_macro::spectest;

spectest!("spec-tests/trivial.wast");
spectest!("spec-tests/local_get.wast");
spectest!("spec-tests/loop.wast");
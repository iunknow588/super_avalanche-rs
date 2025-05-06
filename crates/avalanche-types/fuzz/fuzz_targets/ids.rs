#![no_main]
use libfuzzer_sys::fuzz_target;

use avalanche_types::ids;

// ref. https://rust-fuzz.github.io/book/cargo-fuzz/tutorial.html
fuzz_target!(|data: &[u8]| {
    for batch in data.chunks(ids::LEN) {
        let _ = ids::Id::from_slice(batch);
    }
});

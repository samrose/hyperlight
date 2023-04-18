use super::handle::Handle;
use super::hdl::Hdl;
use super::{arrays::raw_vec::RawVec, context::Context};
use crate::guest::function_call_result::FunctionCallResult;
use crate::validate_context_or_panic;
use anyhow::Result;
use std::mem;

/// Return true if the given handle `hdl` in `ctx` references a `FunctionCallResult` representing a return value from a function call ,
/// and false otherwise
///
/// # Safety
///
/// `ctx` must be a valid pointer to a `Context` created with `context_new`,
/// owned by you, and not yet freed with `context_free`
#[no_mangle]
pub unsafe extern "C" fn handle_is_function_call_result(ctx: *const Context, hdl: Handle) -> bool {
    validate_context_or_panic!(ctx);
    get_function_call_result(&*ctx, hdl).is_ok()
}

/// Return the value of the `FunctionCallResult` in `ctx` referenced by `hdl` as ptr.
/// The ptr references a `flatbuffer` serialistion of a `FunctionCallResult`.
/// If `hdl` does not reference a `FunctionCallResult` in `ctx`, the return value is
/// NULL.
///
/// # Safety
///
/// `ctx` must be a valid pointer to a `Context` created with `context_new`,
/// owned by you, and not yet freed with `context_free`
///
/// The caller is responsible for the memory referenced by the returned
/// pointer. After this function returns, the caller must therefore free
/// this memory when they're done with it by calling `guest_error_raw_free`
/// and passing this pointer.
///
/// **It is not guaranteed that all memory will be correctly freed if you
/// call any other function than `host_function_call_flatbuffer_free`**.
///
/// The Context is still responsible for the byte array memory after this function returns.
#[no_mangle]
pub unsafe extern "C" fn handle_get_function_call_result_flatbuffer(
    ctx: *mut Context,
    hdl: Handle,
) -> *mut u8 {
    validate_context_or_panic!(ctx);

    match get_function_call_result(&*ctx, hdl) {
        Ok(function_call_result) => {
            match Vec::try_from(function_call_result) {
                Ok(fb_bytes) => {
                    // Move the fb_bytes vec into a RawVec, then return the
                    // pointer to that underlying RawVec.
                    //
                    // This means that the memory must be freed by the caller
                    // invoking `host_function_call_flatbuffer_free`.
                    //
                    // The returned Vec is a size prefixed flatbuffer, which
                    // means the first 4 bytes are the size of the buffer
                    // and the capacity of the Vec is the same as the size of
                    // the buffer + 4 bytes for the size field.
                    // therefore `host_function_call_flatbuffer_free` can safely
                    // reconstruct the Vec, bring it back into a RawVec, and
                    // then drop it.
                    let raw_vec = RawVec::from(fb_bytes);
                    raw_vec.to_ptr().0
                }
                Err(e) => {
                    (*ctx).register_err(e);
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            //TODO: Update when we have a GetLastErrorFunction
            (*ctx).register_err(e);
            std::ptr::null_mut()
        }
    }
}

/// Free the memory associated with the `FunctionCallResult`s `ptr`.
///
/// # Safety
///
/// You must only call this function exactly once per `ptr' returned from `handle_get_function_call_result_flatbuffer`, and only
/// call it after you're done using `ptr`. The pointer must be a valid pointer to a `FunctionCallResult` serialised using `flatbuffers`.
#[no_mangle]
pub unsafe extern "C" fn function_call_result_flatbuffer_free(ptr: *mut u8) -> bool {
    // the size of the memory is placed in the first 4 bytes of the memory for a size prefixed flatbuffer
    // the size does not include the size of the size field, so we need to add 4 to the size
    // the capacity of the Vec is the same as the size of the buffer
    let len = std::ptr::read(ptr as *const u32) as usize + mem::size_of::<u32>();
    let raw_vec = RawVec::from_ptr(ptr, len);
    drop(raw_vec);
    true
}

fn get_function_call_result(ctx: &Context, hdl: Handle) -> Result<&FunctionCallResult> {
    Context::get(hdl, &ctx.function_call_results, |hdl| {
        matches!(hdl, Hdl::FunctionCallResult(_))
    })
}
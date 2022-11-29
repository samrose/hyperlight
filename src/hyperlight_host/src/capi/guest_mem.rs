use super::context::Context;
use super::handle::Handle;
use super::hdl::Hdl;
use crate::{
    capi::context::ERR_NULL_CONTEXT, mem::guest_mem::GuestMemory, validate_context,
    validate_context_or_panic,
};
use anyhow::{anyhow, Result};
use std::panic::catch_unwind;

mod impls {
    use crate::capi::{byte_array::get_byte_array, context::Context};
    use crate::{capi::handle::Handle, mem::guest_mem::GuestMemory};
    use anyhow::{bail, Result};
    use std::cell::RefCell;

    pub fn get_address(ctx: &Context, hdl: Handle) -> Result<usize> {
        let guest_mem = super::get_guest_memory(ctx, hdl)?;
        Ok(guest_mem.base_addr())
    }

    pub fn read_int_64(ctx: &Context, hdl: Handle, addr: u64) -> Result<i64> {
        let guest_mem = super::get_guest_memory(ctx, hdl)?;
        (*guest_mem).read_i64(addr)
    }

    pub fn write_int_64(ctx: &mut Context, hdl: Handle, addr: usize, val: usize) -> Result<()> {
        let guest_mem = super::get_guest_memory_mut(ctx, hdl)?;
        (*guest_mem).write_u64(addr, val as u64)
    }

    pub fn read_int_32(ctx: &Context, hdl: Handle, addr: u64) -> Result<i32> {
        let guest_mem = super::get_guest_memory(ctx, hdl)?;
        (*guest_mem).read_i32(addr)
    }

    pub fn write_int_32(ctx: &mut Context, hdl: Handle, addr: usize, val: i32) -> Result<()> {
        let guest_mem = super::get_guest_memory_mut(ctx, hdl)?;
        (*guest_mem).write_i32(addr, val)?;
        Ok(())
    }

    /// Look up the `[u8]` referenced by `byte_arr_hdl` in `ctx`,
    /// get the slice in the range `[arr_start, arr_start + arr_length)`,
    /// wrap that slice in a `RefCell`, and return it
    pub fn copy_from_byte_array(
        ctx: &Context,
        byte_arr_hdl: Handle,
        arr_start: usize,
        arr_length: usize,
    ) -> Result<RefCell<&[u8]>> {
        let byte_arr = get_byte_array(ctx, byte_arr_hdl)?;
        let byte_arr_len = (*byte_arr).len();

        // ensure we're not starting off the end of the byte array
        if arr_start >= byte_arr_len {
            bail!("Array start ({}) is out of bounds", arr_start);
        }

        // ensure we're not ending off the end of the byte array
        let arr_end = arr_start + arr_length;
        if (arr_start + arr_length) > byte_arr_len {
            bail!("Array end ({}) is out of bounds", arr_end);
        }

        // get the slice of byte_arr.
        // slice semantics give a byte_arr in the range: [arr_start, arr_end)
        // (i.e. inclusive of arr_start, exclusive of arr_end)
        let slice = &(*byte_arr)[arr_start..arr_end];

        Ok(RefCell::new(slice))
    }

    /// Attempt to look up the `GuestMemory` referenced by `hdl` in `ctx`,
    /// then if one exists, return it wrapped in a `RefCell`.
    ///
    /// Returns `Err` if no such `GuestMemory` exists.
    ///
    /// This function is useful because you must have access to a
    /// `&mut Context` if you want to do mutable operations on
    /// a `GuestMemory` stored therein.
    /// Instead, when you need to do mutable operations on a `GuestMemory`,
    /// pass a `&Context` (immutable reference) to this function, then
    /// call `try_borrow_mut` on the resulting `RefCell`
    ///
    /// # Example
    ///
    /// ```rust
    /// // assume we have a Context called ctx and a Handle to
    /// // a valid GuestMemory in Context, called hdl
    /// let guest_mem_refcell: RefCell<GuestMemory> = get_guest_memory_ref(ctx, hdl).unwrap();
    /// let guest_mem_ref: RefMut<GuestMemory> = res.try_borrow_mut().unwrap();
    /// let guest_mem_mut: &mut GuestMemory = *guest_mem_ref;
    /// ```
    fn get_guest_memory_ref(ctx: &Context, hdl: Handle) -> Result<RefCell<GuestMemory>> {
        let gm = super::get_guest_memory(ctx, hdl)?;
        // ok to clone the GuestMemory here because internally, it's just
        // a reference-counted pointer, so we're simply incrementing the
        // reference count. Memory won't be deleted until all clones and the
        // original go out of scope. see documentation inside GuestMemory
        // for more detail
        Ok(RefCell::new(gm.clone()))
    }

    /// Copy all values in the byte array referenced by `byte_array_hdl`,
    /// in the range `[arr_start, arr_start + arr_length)` into the
    /// `GuestMemory` referenced by `guest_mem_hdl`
    pub fn copy_byte_array(
        ctx: &mut Context,
        guest_mem_hdl: Handle,
        byte_array_hdl: Handle,
        guest_mem_offset: usize,
        arr_start: usize,
        arr_length: usize,
    ) -> Result<()> {
        // Below is a comprehensive explanation of why we're using
        // `RefCell` below to fetch and access the byte array and guest memory.
        // I'm including it because it took me (arschles) a long time to
        // figure out the best way to get `RefCell` working properly. If you
        // intend to change something inside this function, you should probably
        // read at least until the "stfu borrow checker" part of this comment.
        //
        // # Background on the problem
        //
        // To start, here's a description of the problem we're facing
        // in this function. W
        //
        // We have to fetch two things from `ctx`:
        //
        // 1. The `Vec<u8>` referenced by `byte_array_hdl`, immutably
        // 2. The `GuestMemory` referenced by `guest_mem_offset`, mutably
        //
        // In other words, we're only going to read from the `Vec<u8>`
        // in (1), but we're going to write to the `GuestMemory` in (2).
        //
        // So, to do (1), we have to borrow `ctx` immutably and to do (2)
        // we have to borrow `ctx` mutably. This arrangement violates
        // the borrow checker. We can't copy `ctx` to get around this
        // problem, because that violates the borrow checker rules.
        // (i.e. if you borrow anything mutably, as in (2), you can't borrow
        // anything else, mutably or immutably, as in (1))
        //
        // Of course, we know that this isn't going to be a problem
        // in reality because we're not going to be reading any parts of `ctx`
        // that we're also mutating. In fact, the read -- of the `Vec<u8>`
        // -- happens strictly before the write to the `GuestMemory`.
        //
        // # How `RefCell` helps us solve the problem
        //
        // We don't have a clean way to indicate to the borrow checker
        // that, essentially, we know what we're doing. At the end of the
        // day, you need to pass a `&mut Context` to get a `&mut GuestMemory`,
        // and that means you can't pass a `&mut Context` or a `&Context`
        // anywhere else within that same scope. Also, the borrow checker
        // is smart enough to know that _any_ reference you got from
        // that `&mut Context`, which escapes the scope, could also mutate
        // the `Context` and needs to have exclusive access.
        //
        // All this is to say there may be a very complex way to tell the
        // borrow checker we know what we're doing, or to trick the borrow
        // checker, but it's not worth doing because we have a very well
        // defined and relatively simple way to do the same thing built
        // into the standard library. Read on for more.
        //
        // # Enter `RefCell`
        //
        // `RefCell` is Rust's built-into-the-standard-library way to
        // tell the borrow checker we know what we're doing with respect
        // to mutability. In other words, we can break the exclusive access
        // rules in a well-defined, somewhat-safe way.
        //
        // `RefCell` docs call this somewhat-safe way to break the rules
        // "interior mutability".
        //
        // In the below code, `RefCell` is allowing us to pass a
        // `&Context` to some code that gives us back a `RefCell<GuestMemory>`.
        // We can then, in turn, use this `RefCell` to mutate the contained
        // `GuestMemory.
        //
        // In fun terms, our end goal is to say "stfu borrow checker,
        // I know what I'm doing"
        //
        // >If you want to dive into more details, read on.
        // >Otherwise, you can stop reading.
        //
        // # More about `RefCell`
        //
        // As said above, `RefCell` is how we get around the borrow checker's
        // exclusive mutating access rule. The standard library calls this
        // feature "interior mutability" - outwardly to the borrow checker,
        // you can't mutate the `RefCell`, but if you reach inside to the
        // _interior_ of that `RefCell`, you can mutate it.
        //
        // Again, in fun terms: `interior mutability = "stfu borrow checker"`
        //
        // Recall above that we had to read the `GuestMemory` from `ctx`,
        // but since we're going to fetch that `GuestMemory` for mutation,
        // we had to borrow `ctx` mutably, and that caused the borrow checker
        // to (rightfully) cause a compile error.
        //
        // `RefCell` is precisely what allows us to borrow `ctx` immutably to
        // get the `GuestMemory` we need, and then later allow us to mutate
        // that `GuestMemory` anyway. See that in action in the call below to
        // `get_guest_memory_ref`. In that call, we're passing `ctx` in as
        // a `&Context` -- an immutable reference.
        //
        // That function, in turn, returns a `Result<RefCell<GuestMemory>>`,
        // but let's ignore that outer `Result` here for simplicity. Once we
        // have our `RefCell<GuestMemory>`, we have several useful methods we
        // can call.
        //
        // Since at the end of the day, we want a `&mut GuestMemory`, the one
        // we care about most is `try_borrow_mut`. That function gives us
        // a `Result<RefMut<GuestMemory>>`. Here, that outer `Result` matters
        // because if it returns an `Err`, that means someone else has called
        // `try_borrow_mut` before us. This function is how `RefCell` does
        // borrow checking at runtime, and allowing us to quiet the borrow
        // checker at compile time.
        let data = {
            let data_refcell = copy_from_byte_array(ctx, byte_array_hdl, arr_start, arr_length)?;
            let data_ref = data_refcell.try_borrow()?;
            *data_ref
        };
        let guest_mem = &mut {
            let gm_refcell = get_guest_memory_ref(ctx, guest_mem_hdl)?;
            let gm_refmut = gm_refcell.try_borrow_mut()?;
            // Note: this clone is cheap. It just increments a reference-counter
            // inside the GuestMemory. See docs on GuestMemory for more
            // information
            (*gm_refmut).clone()
        };

        guest_mem.copy_from_slice(data, guest_mem_offset)
    }

    /// Look up the guest memory wrapper referenced by `guest_mem_hdl` in
    /// `ctx`, then copy its contents starting at `offset` into `byte_array`
    pub fn copy_to_byte_array(
        ctx: &mut Context,
        guest_mem_hdl: Handle,
        byte_array: &mut [u8],
        offset: usize,
    ) -> Result<()> {
        let guest_mem = super::get_guest_memory_mut(ctx, guest_mem_hdl)?;
        (*guest_mem).copy_to_slice(byte_array, offset)
    }
}

/// Get the `GuestMemory` stored in `ctx` and referenced by `hdl` and return
/// it inside a `ReadResult` suitable only for read operations.
///
/// Returns `Ok` if `hdl` is a valid `GuestMemory` in `ctx`,
/// `Err` otherwise.
pub fn get_guest_memory(ctx: &Context, hdl: Handle) -> Result<&GuestMemory> {
    Context::get(hdl, &ctx.guest_mems, |g| matches!(g, Hdl::GuestMemory(_)))
}

/// Get the `GuestMemory` stored in `ctx` and referenced by `hdl` and return
/// it inside a `WriteResult` suitable for mutation.
///
/// Returns `Ok` if `hdl` is a valid `GuestMemory` in `ctx`,
/// `Err` otherwise.
pub fn get_guest_memory_mut(ctx: &mut Context, hdl: Handle) -> Result<&mut GuestMemory> {
    Context::get_mut(hdl, &mut ctx.guest_mems, |g| {
        matches!(g, Hdl::GuestMemory(_))
    })
}

/// Create a new instance of guest memory with `min_size` bytes.
///
/// Guest memory is shared memory intended to be shared with a
/// hypervisor partition.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn guest_memory_new(ctx: *mut Context, min_size: u64) -> Handle {
    validate_context!(ctx);

    match GuestMemory::new(min_size as usize) {
        Ok(guest_mem) => Context::register(guest_mem, &mut (*ctx).guest_mems, Hdl::GuestMemory),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Get the starting address of the guest memory referenced
/// by `hdl` in `ctx`, or `0` if the handle is invalid.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn guest_memory_get_address(ctx: *const Context, hdl: Handle) -> usize {
    validate_context_or_panic!(ctx);

    impls::get_address(&*ctx, hdl).unwrap_or(0)
}

/// Fetch the following two strutures:
/// * The byte array in `ctx` referenced by `byte_array_hdl`
/// * The guest memory in `ctx` referenced by `guest_mem_hdl`
///
/// ... then copy the data from the byte array in the range
/// `[arr_start, arr_start + arr_length)` (i.e. the left side is
/// inclusive and the right side is not inclusive) into the guest
/// memory starting at offset `offset`.
///
/// Return an empty `Handle` if both the guest memory and byte array
/// were found and the copy succeeded, and an error handle otherwise.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn guest_memory_copy_from_byte_array(
    ctx: *mut Context,
    guest_mem_hdl: Handle,
    byte_array_hdl: Handle,
    offset: usize,
    arr_start: usize,
    arr_length: usize,
) -> Handle {
    validate_context!(ctx);

    match impls::copy_byte_array(
        &mut *ctx,
        guest_mem_hdl,
        byte_array_hdl,
        offset,
        arr_start,
        arr_length,
    ) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Fetch the guest memory in `ctx` referenced by `guest_mem_hdl`,
/// then copy the data from guest memory starting at address `offset`
/// into the memory between `byte_array` and `(byte_array + length)`
///
/// Return an empty `Handle` if the guest memory and byte array were valid
/// and the copy succeeded, or an error handle otherwise.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - A valid handle to guest memory
/// - A valid offset into the guest memory
/// - A pointer to a byte array
/// - A valid length for the byte array
///
/// The byte array is owned by the caller and must be valid for the lifetime of the call.
#[no_mangle]
pub unsafe extern "C" fn guest_memory_copy_to_byte_array(
    ctx: *mut Context,
    guest_mem_hdl: Handle,
    offset: usize,
    byte_array: *mut u8,
    length: usize,
) -> Handle {
    validate_context!(ctx);

    if byte_array.is_null() {
        return (*ctx).register_err(anyhow!("Invalid byte array"));
    };

    if length < 1 {
        return (*ctx).register_err(anyhow!("Invalid length"));
    };

    let did_it_panic = catch_unwind(|| {
        let buffer: &mut [u8] = std::slice::from_raw_parts_mut(byte_array, length);
        buffer
    });

    let buffer = match did_it_panic {
        Ok(result) => result,
        Err(_) => {
            return (*ctx).register_err(anyhow::anyhow!(
                "failed to get slice from pointer and length in file {} line number {} ",
                file!(),
                line!()
            ))
        }
    };

    match impls::copy_to_byte_array(&mut *ctx, guest_mem_hdl, buffer, offset) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Fetch guest memory from `ctx` referenced by `hdl`, then read
/// a single 64 bit integer from it at address `addr`.
///
/// Return a `Handle` containing the integer if the read succeeded,
/// and an error otherwise.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn guest_memory_read_int_64(
    ctx: *mut Context,
    hdl: Handle,
    addr: u64,
) -> Handle {
    validate_context!(ctx);

    match impls::read_int_64(&*ctx, hdl, addr) {
        Ok(val) => Context::register(val, &mut (*ctx).int64s, Hdl::Int64),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Write a single 64 bit integer `val` to guest memory in `ctx` referenced
/// by `hdl` at `addr`.
///
/// Return an empty `Handle` if the write succeeded,
/// and an error `Handle` otherwise.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn guest_memory_write_int_64(
    ctx: *mut Context,
    hdl: Handle,
    addr: usize,
    val: usize,
) -> Handle {
    validate_context!(ctx);

    match impls::write_int_64(&mut *ctx, hdl, addr, val) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Fetch guest memory from `ctx` referenced by `hdl`, then read
/// a single 32 bit integer from it at address `addr`.
///
/// Return a `Handle` containing the integer if the read succeeded,
/// and an error otherwise.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn guest_memory_read_int_32(
    ctx: *mut Context,
    hdl: Handle,
    addr: u64,
) -> Handle {
    validate_context!(ctx);

    match impls::read_int_32(&*ctx, hdl, addr) {
        Ok(val) => Context::register(val, &mut (*ctx).int32s, Hdl::Int32),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Write a single 32 bit integer `val` to guest memory in `ctx` referenced
/// by `hdl` at `addr`.
///
/// Return an empty `Handle` if the write succeeded,
/// and an error `Handle` otherwise.
///
/// # Safety
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn guest_memory_write_int_32(
    ctx: *mut Context,
    hdl: Handle,
    addr: usize,
    val: i32,
) -> Handle {
    validate_context!(ctx);

    match impls::write_int_32(&mut *ctx, hdl, addr, val) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::impls::copy_byte_array;
    use crate::capi::handle::Handle;
    use crate::{
        capi::{context::Context, hdl::Hdl},
        mem::guest_mem::GuestMemory,
    };
    use anyhow::Result;

    struct TestData {
        // Context used to create all handles herein
        pub ctx: Box<Context>,
        // Handle to guest memory
        pub guest_mem_hdl: Handle,
        // Size of guest memory
        pub guest_mem_size: usize,
        // Handle to byte array
        pub byte_arr_hdl: Handle,
        // length of byte array
        pub barr_len: usize,
    }

    impl TestData {
        pub fn new(barr_vec_len: usize, guest_mem_size: usize) -> Result<Self> {
            let mut ctx = Context::default();
            let barr_vec = {
                let mut v = Vec::new();
                for i in 0..barr_vec_len {
                    v.push(i as u8);
                }
                v
            };
            let barr_hdl = Context::register(barr_vec, &mut ctx.byte_arrays, Hdl::ByteArray);
            let guest_mem_hdl = {
                let gm = GuestMemory::new(guest_mem_size).unwrap();
                Context::register(gm, &mut ctx.guest_mems, Hdl::GuestMemory)
            };
            Ok(Self {
                ctx: Box::new(ctx),
                guest_mem_hdl,
                guest_mem_size,
                byte_arr_hdl: barr_hdl,
                barr_len: barr_vec_len,
            })
        }
    }

    #[test]
    fn copy_byte_array_at_start() {
        // copy an entire byte array into guest memory
        let mut test_data = TestData::new(3, 0x1000).unwrap();
        copy_byte_array(
            test_data.ctx.as_mut(),
            test_data.guest_mem_hdl,
            test_data.byte_arr_hdl,
            0,
            0,
            test_data.barr_len,
        )
        .unwrap();
    }

    #[test]
    fn copy_byte_array_twice() {
        let mut test_data = TestData::new(3, 0x1000).unwrap();
        copy_byte_array(
            test_data.ctx.as_mut(),
            test_data.guest_mem_hdl,
            test_data.byte_arr_hdl,
            0,
            0,
            test_data.barr_len,
        )
        .unwrap();
        copy_byte_array(
            test_data.ctx.as_mut(),
            test_data.guest_mem_hdl,
            test_data.byte_arr_hdl,
            0,
            0,
            test_data.barr_len,
        )
        .unwrap();
    }

    #[test]
    fn copy_byte_array_at_end() {
        // copy byte array to the very end of guest memory
        let mut test_data = TestData::new(3, 0x1000).unwrap();
        copy_byte_array(
            test_data.ctx.as_mut(),
            test_data.guest_mem_hdl,
            test_data.byte_arr_hdl,
            test_data.guest_mem_size - test_data.barr_len - 1,
            0,
            test_data.barr_len,
        )
        .unwrap();
    }

    #[test]
    fn copy_byte_array_invalid_offset() {
        // copy the same small byte array to an invalid offset.
        let mut test_data = TestData::new(3, 0x1000).unwrap();

        let res = copy_byte_array(
            test_data.ctx.as_mut(),
            test_data.guest_mem_hdl,
            test_data.byte_arr_hdl,
            test_data.guest_mem_size,
            0,
            1,
        );

        assert!(matches!(res, Err(_)));
    }

    #[test]
    fn copy_byte_array_too_much() {
        // copy too much of the small byte array
        let mut test_data = TestData::new(3, 0x1000).unwrap();
        let res = copy_byte_array(
            test_data.ctx.as_mut(),
            test_data.guest_mem_hdl,
            test_data.byte_arr_hdl,
            0,
            0,
            test_data.barr_len * 10,
        );
        assert!(matches!(res, Err(_)));
    }
}

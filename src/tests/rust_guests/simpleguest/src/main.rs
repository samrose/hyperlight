#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};
use hyperlight_flatbuffers::flatbuffer_wrappers::{
    function_call::FunctionCall,
    function_types::{ParameterType, ParameterValue, ReturnType},
    guest_function_definition::GuestFunctionDefinition,
};
use hyperlight_guest::{
    entrypoint::halt,
    flatbuffer_utils::{get_flatbuffer_result_from_int, get_flatbuffer_result_from_void},
    guest_functions::register_function,
    host_function_call::{call_host_function, get_host_value_return_as_int},
    DEFAULT_GUEST_STACK_SIZE,
};

extern crate hyperlight_guest;

#[no_mangle]
#[allow(improper_ctypes_definitions, non_camel_case_types)]
pub extern "C" fn simple_print_output(function_call: &FunctionCall) -> Vec<u8> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        call_host_function(
            "HostPrint",
            Some(Vec::from(&[ParameterValue::String(message)])),
            ReturnType::Int,
        );
        let result = get_host_value_return_as_int();
        get_flatbuffer_result_from_int(result)
    } else {
        Vec::new()
    }
}

// TODO: This function could cause a stack overflow, update it once we have stack guards in place.
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn stack_allocate(function_call: &FunctionCall) -> Vec<u8> {
    if let ParameterValue::Int(length) = function_call.parameters.clone().unwrap()[0].clone() {
        let alloc_length = if length == 0 {
            DEFAULT_GUEST_STACK_SIZE + 1
        } else {
            length
        } as usize;

        let _buffer: [u8; (DEFAULT_GUEST_STACK_SIZE + 1) as usize] =
            [0; (DEFAULT_GUEST_STACK_SIZE + 1) as usize];
        // ^^^ this does the stack allocation

        let mut buffer = Vec::with_capacity(alloc_length);

        // Initialize the buffer with zeros. This is necessary because Vec::with_capacity
        // does not initialize the elements, and is equivalent to _alloca that we
        // have in C.
        buffer.resize(alloc_length, 0);

        get_flatbuffer_result_from_int(length)
    } else {
        Vec::new()
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn small_var(_: &FunctionCall) -> Vec<u8> {
    let _buffer: [u8; 1024] = [0; 1024];
    get_flatbuffer_result_from_int(1024)
}

// TODO: This function could cause a stack overflow, update it once we have stack guards in place.
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn call_malloc(function_call: &FunctionCall) -> Vec<u8> {
    if let ParameterValue::Int(size) = function_call.parameters.clone().unwrap()[0].clone() {
        let mut allocated_buffer = Vec::with_capacity(size as usize);
        allocated_buffer.resize(size as usize, 0);

        get_flatbuffer_result_from_int(size)
    } else {
        Vec::new()
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn spin(_: &FunctionCall) -> Vec<u8> {
    loop {
        // Keep the CPU 100% busy forever
    }

    #[allow(unreachable_code)]
    get_flatbuffer_result_from_void()
}

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let simple_print_output_def = GuestFunctionDefinition::new(
        "PrintOutput".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        simple_print_output as i64,
    );
    register_function(simple_print_output_def);

    let stack_allocate_def = GuestFunctionDefinition::new(
        "StackAllocate".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        stack_allocate as i64,
    );
    register_function(stack_allocate_def);

    let small_var_def = GuestFunctionDefinition::new(
        "SmallVar".to_string(),
        Vec::new(),
        ReturnType::Int,
        small_var as i64,
    );
    register_function(small_var_def);

    let call_malloc_def = GuestFunctionDefinition::new(
        "CallMalloc".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        call_malloc as i64,
    );
    register_function(call_malloc_def);

    let spin_def = GuestFunctionDefinition::new(
        "Spin".to_string(),
        Vec::new(),
        ReturnType::Void,
        spin as i64,
    );
    register_function(spin_def);
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn guest_dispatch_function() -> Vec<u8> {
    [0; 0].to_vec()
}

// It looks like rust-analyzer doesn't correctly manage no_std crates,
// and so it displays an error about a duplicate panic_handler.
// See more here: https://github.com/rust-lang/rust-analyzer/issues/4490
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    halt();
    loop {}
}

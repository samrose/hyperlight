use hyperlight_host::{
    func::{ParameterValue, ReturnType, ReturnValue},
    sandbox::uninitialized::UninitializedSandbox,
    sandbox_state::{sandbox::EvolvableSandbox, transition::Noop},
    GuestBinary, MultiUseSandbox, Result,
};
use hyperlight_testing::simple_guest_as_string;
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::prelude::*;

// This example can be run with `cargo run --package hyperlight_host --example chrome-tracing --release`
fn main() -> Result<()> {
    // set up tracer
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    let simple_guest_path =
        simple_guest_as_string().expect("Cannot find the guest binary at the expected location.");

    // Create a new sandbox.
    let usandbox =
        UninitializedSandbox::new(GuestBinary::FilePath(simple_guest_path), None, None, None)?;

    // NOTE: if replacing MultiUseSandbox with SingleUseSandbox, the function call will take ~50x longer because the drop
    // happens inside `call_guest_function_by_name` rather than at the end of of this `main` block.

    let mut sbox = usandbox
        .evolve(Noop::<UninitializedSandbox, MultiUseSandbox>::default())
        .unwrap();

    // do the function call
    let current_time = std::time::Instant::now();
    let res = sbox.call_guest_function_by_name(
        "Echo",
        ReturnType::String,
        Some(vec![ParameterValue::String("Hello, World!".to_string())]),
    )?;
    let elapsed = current_time.elapsed();
    println!("Function call finished in {:?}.", elapsed);
    assert!(matches!(res, ReturnValue::String(s) if s == "Hello, World!"));
    Ok(())
}
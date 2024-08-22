use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crossbeam::atomic::AtomicCell;
use crossbeam_channel::{Receiver, Sender};
#[cfg(target_os = "linux")]
use libc::{c_void, pthread_self, siginfo_t};
use rand::Rng;
use tracing::{instrument, Span};
#[cfg(target_os = "linux")]
use vmm_sys_util::signal::{register_signal_handler, SIGRTMIN};
#[cfg(target_os = "windows")]
use windows::Win32::System::Hypervisor::WHV_PARTITION_HANDLE;

use crate::func::exports::get_os_page_size;
#[cfg(feature = "function_call_metrics")]
use crate::histogram_vec_observe;
use crate::hypervisor::handlers::{MemAccessHandlerWrapper, OutBHandlerWrapper};
use crate::hypervisor::{terminate_execution, Hypervisor};
use crate::mem::ptr::RawPtr;
use crate::sandbox::hypervisor::HypervisorWrapper;
#[cfg(feature = "function_call_metrics")]
use crate::sandbox::metrics::SandboxMetric::GuestFunctionCallDurationMicroseconds;
use crate::sandbox::{SandboxConfiguration, WrapperGetter};
use crate::sandbox_state::sandbox::Sandbox;
#[cfg(all(feature = "seccomp", target_os = "linux"))]
use crate::seccomp::guest::get_seccomp_filter_for_hypervisor_handler;
use crate::HyperlightError::HypervisorHandlerExecutionCancelAttemptOnFinishedExecution;
use crate::{log_then_return, new_error, HyperlightError, Result};

/// Trait to indicate that a type contains to/from receivers/transmitters for `HypervisorHandlerAction`s, and `HandlerMsg`s.
pub trait HasCommunicationChannels {
    /// Get the transmitter for vCPU actions to the handler
    fn get_to_handler_tx(&self) -> Sender<HypervisorHandlerAction>;
    /// Set the transmitter to send messages to the handler
    fn set_to_handler_tx(&mut self, tx: Sender<HypervisorHandlerAction>);
    /// Drop the transmitter to send messages to the handler
    /// This is useful to forcefully terminate a vCPU
    fn drop_to_handler_tx(&mut self);

    /// Get the receiver for messages from the handler
    fn get_from_handler_rx(&self) -> Receiver<HandlerMsg>;
    /// Set the receiver to receive messages from the handler
    fn set_from_handler_rx(&mut self, rx: Receiver<HandlerMsg>);

    /// Get the transmitter for messages from the handler
    fn get_from_handler_tx(&self) -> Sender<HandlerMsg>;
    /// Set the transmitter for messages from the handler
    fn set_from_handler_tx(&mut self, tx: Sender<HandlerMsg>);

    /// Get the receiver for vCPU actions from the handler
    fn get_to_handler_rx(&self) -> Receiver<HypervisorHandlerAction>;

    /// Set the receiver for vCPU actions from the handler
    fn set_to_handler_rx(&mut self, rx: Receiver<HypervisorHandlerAction>);
}

/// `HypervisorHandlerActions` enumerates the
/// possible actions that a Hypervisor
/// handler can execute.
pub enum HypervisorHandlerAction {
    /// Initialise the vCPU
    Initialise(InitArgs),
    /// Execute the vCPU until a HLT instruction
    DispatchCallFromHost(DispatchArgs),
    /// Terminate hypervisor handler thread
    TerminateHandlerThread,
}

// Debug impl for HypervisorHandlerAction:
// - just prints the enum variant type name.
impl std::fmt::Debug for HypervisorHandlerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HypervisorHandlerAction::Initialise(_) => write!(f, "Initialise"),
            HypervisorHandlerAction::DispatchCallFromHost(_) => write!(f, "DispatchCallFromHost"),
            HypervisorHandlerAction::TerminateHandlerThread => write!(f, "TerminateHandlerThread"),
        }
    }
}

/// `HandlerMsg` is structure used by the Hypervisor
/// handler to indicate that the Hypervisor Handler has
/// finished performing an action (i.e., `DispatchCallFromHost`, or
/// `Initialise`).
pub enum HandlerMsg {
    FinishedHypervisorHandlerAction,
    Error(HyperlightError),
}

/// Arguments to initialise the vCPU
pub struct InitArgs {
    peb_addr: RawPtr,
    seed: u64,
    page_size: u32,
    outb_handle_fn: OutBHandlerWrapper,
    mem_access_fn: MemAccessHandlerWrapper,
}

impl InitArgs {
    /// Create a new `InitArgs` instance
    pub fn new(
        peb_addr: RawPtr,
        seed: u64,
        page_size: u32,
        outb_handle_fn: OutBHandlerWrapper,
        mem_access_fn: MemAccessHandlerWrapper,
    ) -> Self {
        Self {
            peb_addr,
            seed,
            page_size,
            outb_handle_fn,
            mem_access_fn,
        }
    }
}

/// Arguments to execute the vCPU
pub struct DispatchArgs {
    function_name: String,
    dispatch_func_addr: RawPtr,
    outb_handle_fn: OutBHandlerWrapper,
    mem_access_fn: MemAccessHandlerWrapper,
}

impl DispatchArgs {
    /// Create a new `DispatchArgs` instance
    pub fn new(
        function_name: String,
        dispatch_func_addr: RawPtr,
        outb_handle_fn: OutBHandlerWrapper,
        mem_access_fn: MemAccessHandlerWrapper,
    ) -> Self {
        Self {
            function_name,
            dispatch_func_addr,
            outb_handle_fn,
            mem_access_fn,
        }
    }
}

// This variable is used to check if the hv handler thread is still alive and receiving signals
// (specifically, `SIGRTMIN`). This field should not be accessed directly and, instead, only through
// the `is_hv_handler_receiving_signals` function. We use this instead of `join_handle` because it
// could be the case the thread was terminated due to a disallowed syscall, which means the thread
// was forcefully terminated and that the hv_lock it held was never dropped as forceful termination
// of threads is UB in Rust.
#[cfg(all(feature = "seccomp", target_os = "linux"))]
static HV_HANDLER_THREAD_RECEIVING_SIGNALS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// This function is used to check if the Hypervisor Handler thread is still alive and receiving signals.
#[cfg(all(feature = "seccomp", target_os = "linux"))]
pub fn is_hv_handler_receiving_signals(thread_id: libc::pthread_t) -> bool {
    HV_HANDLER_THREAD_RECEIVING_SIGNALS.store(false, std::sync::atomic::Ordering::Relaxed);
    // ^^^ reset variable pre-check
    unsafe {
        libc::pthread_kill(thread_id, SIGRTMIN());
        // ^^^ send signal to check if thread is alive
    }
    sleep(Duration::from_millis(
        SandboxConfiguration::DEFAULT_MAX_WAIT_FOR_CANCELLATION as u64,
    ));
    // ^^^ wait for signal to be received. We cannot use message passing transmitters/receivers
    // in signal handlers because it does not capture its context.

    HV_HANDLER_THREAD_RECEIVING_SIGNALS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Sets up a Hypervisor 'handler', designed to listen to
/// messages to execute a specific action, such as:
/// - `initialise` resources,
/// - `dispatch_call_from_host` in the vCPU, and
/// - `terminate_execution` of the vCPU.
///
/// The execution of an action within the handler has to
/// be paired w/ a call to from_handler_tx.recv() to
/// synchronise the completion of the action. Otherwise,
/// the main thread will proceed prior to finishing the
/// required action, and you may encounter unexpected unset
/// pointers or other issues.
#[instrument(err(Debug), skip_all, parent = Span::current(), level = "Trace")]
pub(crate) fn start_hypervisor_handler(hv: Arc<Mutex<Box<dyn Hypervisor>>>) -> crate::Result<()> {
    let hv_clone = hv.clone();
    hv.lock()?.setup_hypervisor_handler_communication_channels();

    let from_handler_tx = {
        let hv_lock = hv.lock().unwrap();
        hv_lock.get_from_handler_tx()
    };

    let to_handler_rx = {
        let hv_lock = hv.lock().unwrap();
        hv_lock.get_to_handler_rx()
    };

    // Other than running initialization and code execution, the handler thread also handles
    // cancellation. When we need to cancel the execution there are 2 possible cases
    // we have to deal with depending on if the vCPU is currently running or not.
    //
    // 1. If the vCPU is executing, then we need to cancel the execution.
    // 2. If the vCPU is not executing, then we need to signal to the thread
    // that it should exit the loop.
    //
    // For the first case, on Linux, we send a signal to the thread running the
    // vCPU to interrupt it and cause an EINTR error on the underlying VM run call.
    //
    // For the second case, we set a flag that is checked on each iteration of the run loop
    // and if it is set to true then the loop will exit.

    // On Linux, we have another problem to deal with. The way we terminate a running vCPU
    // (case 1 above) is to send a signal to the thread running the vCPU to interrupt it.
    //
    // There is a possibility that the signal is sent and received just before the thread
    // calls run on the vCPU (between the check on the cancelled_run variable and the call to run)
    // - see this StackOverflow question for more details
    // https://stackoverflow.com/questions/25799667/fixing-race-condition-when-sending-signal-to-interrupt-system-call)
    //
    // To solve this, we need to keep sending the signal until we know that the spawned thread
    // knows it should cancel the execution. To do this, we will use another `AtomicCell` and `Arc`
    // to communicate between the main thread and the handler thread running the vCPU.
    // This variable will be set when the thread has received the instruction to cancel the
    // execution and will be checked in the code which sends the signal to terminate to know
    // there is no longer a needed to send a signal. Again, we will create this on the main
    // thread the first time we enter this function and TLS its. Then, create a clone which we
    // will move to the handler thread which will then place it in its TLS so that it
    // is accessible in the run loop if we re-enter the function.

    {
        let mut hv_lock = hv.lock().unwrap();

        hv_lock.set_termination_status(false);

        #[cfg(target_os = "linux")]
        {
            hv_lock.set_run_cancelled(false);
        }
    }

    #[cfg(all(feature = "seccomp", target_os = "linux"))]
    let seccomp_filter = get_seccomp_filter_for_hypervisor_handler()?;

    let join_handle = {
        thread::spawn(move || -> Result<()> {
            #[cfg(all(feature = "seccomp", target_os = "linux"))]
            seccompiler::apply_filter(&seccomp_filter)?; // applies seccomp filter on thread creation

            for action in to_handler_rx.clone() {
                match action {
                    HypervisorHandlerAction::Initialise(args) => {
                        log::debug!("Initialising Hypervisor Handler");
                        let mut hv_lock = hv_clone.lock().unwrap();

                        // Reset termination status from a possible previous execution
                        hv_lock.set_termination_status(false);
                        #[cfg(target_os = "linux")]
                        {
                            hv_lock.set_run_cancelled(false);
                        }

                        #[cfg(target_os = "linux")]
                        {
                            // We cannot use the Killable trait, so we get the `pthread_t` via a libc
                            // call.

                            let thread_id = unsafe { pthread_self() };
                            hv_lock.set_thread_id(thread_id);

                            // Register a signal handler to cancel the execution of the vCPU on Linux.
                            // On Windows, we don't need to do anything as we can just call the cancel
                            // function.

                            extern "C" fn handle_signal(_: i32, _: *mut siginfo_t, _: *mut c_void) {
                                #[cfg(feature = "seccomp")]
                                HV_HANDLER_THREAD_RECEIVING_SIGNALS
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }

                            match register_signal_handler(SIGRTMIN(), handle_signal) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(new_error!(
                                        "failed to register signal handler: {:?}",
                                        e
                                    ));
                                }
                            }
                        }

                        let res = hv_lock.initialise(
                            args.peb_addr,
                            args.seed,
                            args.page_size,
                            args.outb_handle_fn,
                            args.mem_access_fn,
                        );

                        match res {
                            Ok(_) => {
                                log::debug!("Initialised Hypervisor Handler");
                                from_handler_tx
                                    .send(HandlerMsg::FinishedHypervisorHandlerAction)
                                    .map_err(|_| {
                                        HyperlightError::HypervisorHandlerCommunicationFailure()
                                    })?;
                            }
                            Err(e) => {
                                log::debug!("Error initialising Hypervisor Handler: {:?}", e);
                                from_handler_tx.send(HandlerMsg::Error(e)).map_err(|_| {
                                    HyperlightError::HypervisorHandlerCommunicationFailure()
                                })?;
                            }
                        }
                    }
                    HypervisorHandlerAction::DispatchCallFromHost(args) => {
                        log::debug!("Dispatching call from host: {}", args.function_name);
                        let mut hv_lock = hv_clone.lock().unwrap();

                        // Reset termination status from a possible previous execution
                        hv_lock.set_termination_status(false);
                        #[cfg(target_os = "linux")]
                        {
                            hv_lock.set_run_cancelled(false);
                        }

                        log::info!("Dispatching call from host: {}", args.function_name);

                        let res = {
                            #[cfg(feature = "function_call_metrics")]
                            {
                                let start = std::time::Instant::now();
                                let result = hv_lock.dispatch_call_from_host(
                                    args.dispatch_func_addr,
                                    args.outb_handle_fn,
                                    args.mem_access_fn,
                                );
                                histogram_vec_observe!(
                                    &GuestFunctionCallDurationMicroseconds,
                                    &[args.function_name.as_str()],
                                    start.elapsed().as_micros() as f64
                                );
                                result
                            }

                            #[cfg(not(feature = "function_call_metrics"))]
                            hv_lock.dispatch_call_from_host(
                                args.dispatch_func_addr,
                                args.outb_handle_fn,
                                args.mem_access_fn,
                            )
                        };

                        match res {
                            Ok(_) => {
                                log::debug!(
                                    "Finished dispatching call from host: {}",
                                    args.function_name
                                );
                                from_handler_tx
                                    .send(HandlerMsg::FinishedHypervisorHandlerAction)
                                    .map_err(|_| {
                                        HyperlightError::HypervisorHandlerCommunicationFailure()
                                    })?;
                            }
                            Err(e) => {
                                log::debug!(
                                    "Error dispatching call from host: {}: {:?}",
                                    args.function_name,
                                    e
                                );
                                from_handler_tx.send(HandlerMsg::Error(e)).map_err(|_| {
                                    HyperlightError::HypervisorHandlerCommunicationFailure()
                                })?;
                            }
                        }
                    }
                    HypervisorHandlerAction::TerminateHandlerThread => {
                        log::info!("Terminating Hypervisor Handler Thread");
                        break;
                    }
                }
            }

            {
                from_handler_tx
                    .send(HandlerMsg::FinishedHypervisorHandlerAction)
                    .map_err(|_| HyperlightError::HypervisorHandlerCommunicationFailure())?;

                let mut hv_lock = hv_clone.lock().unwrap();
                hv_lock.drop_to_handler_tx();
            }

            Ok(())
        })
    };

    {
        // set the join handle in the Hypervisor
        let mut hv_lock = hv.lock().unwrap();
        hv_lock.set_handler_join_handle(join_handle);
    }

    Ok(())
}

/// Try `join` on `HypervisorHandler` thread for `max_execution_time` duration.
/// - Before attempting a join, this function checks if execution isn't already finished.
/// Note: This function call takes ownership of the `JoinHandle`.
#[instrument(err(Debug), skip_all, parent = Span::current(), level = "Trace")]
pub(crate) fn try_join_hypervisor_handler_thread<T>(sbox: &mut T) -> Result<()>
where
    T: Sandbox,
{
    let hv_wrapper = sbox.get_hypervisor_wrapper_mut();
    if let Some(handle) = hv_wrapper
        .try_get_hypervisor_lock()
        .unwrap()
        .get_mut_handler_join_handle()
        .take()
    {
        // check if thread is handle.is_finished for `max_execution_time`
        // note: dropping the transmitter in `kill_hypervisor_handler_thread`
        // should have caused the thread to finish, in here, we are just syncing.
        let now = std::time::Instant::now();
        let timeout = hv_wrapper.max_execution_time;
        while now.elapsed() < timeout {
            if handle.is_finished() {
                match handle.join() {
                    // as per docs, join should return immediately and not hang if finished
                    Ok(Ok(())) => return Ok(()),
                    Ok(Err(e)) => {
                        log_then_return!(e);
                    }
                    Err(e) => {
                        log_then_return!(new_error!("{:?}", e));
                    }
                }
            }
            sleep(Duration::from_millis(1)); // sleep to not busy wait
        }
    }

    return Err(HyperlightError::Error(
        "Failed to finish Hypervisor handler thread".to_string(),
    ));
}

/// Tries to kill the Hypervisor Handler Thread.
///
/// All a Hypervisor Handler Thread does is continuously listen for messages. So, to finish the
/// thread's execution, we just need to `drop` the `to_handler_tx` channel. Once the thread realizes
/// that there are no more active senders, it will finish its execution.
///
/// To drop the `to_handler_tx` channel, we need to call `drop_to_handler_tx` on the `Hypervisor`, which
/// requires us to acquire a lock. This lock shouldn't hang, but, to be extra safe and avoid crashing,
/// we only try to acquire the lock for `max_execution_time` duration. If we can't acquire the lock in that
/// time, we will be leaking a thread.
#[instrument(err(Debug), skip_all, parent = Span::current(), level = "Trace")]
pub(crate) fn kill_hypervisor_handler_thread<T>(sbox: &mut T) -> Result<()>
where
    T: Sandbox,
{
    log::debug!("Killing Hypervisor Handler Thread");
    execute_hypervisor_handler_action(
        sbox.get_hypervisor_wrapper_mut(),
        HypervisorHandlerAction::TerminateHandlerThread,
        None,
    )?;

    try_join_hypervisor_handler_thread(sbox)
}

/// Terminate the execution of the hypervisor handler
///
/// This function is intended to be called after a guest function called has
/// timed-out (i.e., `from_handler_rx.recv_timeout(max_execution_time).is_err()`).
///
/// It is possible that, even after we timed-out, the guest function execution will
/// finish. If that is the case, this function is fundamentally a NOOP, because it
/// will restore the memory snapshot to the last state, and then re-initialise the
/// accidentally terminated vCPU.
///
/// This function, usually, will return one of the following HyperlightError's
/// - `ExecutionCanceledByHost` if the execution was successfully terminated, or
/// - `HypervisorHandlerExecutionCancelAttemptOnFinishedExecution` if the execution
///  finished while we tried to terminate it.
///
/// Hence, common usage of this function would be to match on the result. If you get a
/// `HypervisorHandlerExecutionCancelAttemptOnFinishedExecution`, you can safely ignore
/// retrieve the return value from shared memory.
#[allow(clippy::too_many_arguments)]
pub(crate) fn terminate_hypervisor_handler_execution_and_reinitialise<HvMemMgrT: WrapperGetter>(
    wrapper_getter: &mut HvMemMgrT,
    max_execution_time: Duration,
    termination_status: Arc<AtomicCell<bool>>,
    outb_hdl: OutBHandlerWrapper,
    mem_access_hdl: MemAccessHandlerWrapper,
    #[cfg(target_os = "windows")] partition_handle: WHV_PARTITION_HANDLE,
    #[cfg(target_os = "linux")] thread_id: u64,
    #[cfg(target_os = "linux")] run_cancelled: Arc<AtomicCell<bool>>,
    #[cfg(target_os = "linux")] max_wait_for_cancellation: Duration,
) -> Result<HyperlightError> {
    let seed = {
        let mut rng = rand::thread_rng();
        rng.gen::<u64>()
    };

    let peb_addr = {
        let mem_mgr = wrapper_getter.get_mgr_wrapper_mut().unwrap_mgr_mut();
        let peb_u64 = u64::try_from(mem_mgr.layout.peb_address)?;
        RawPtr::from(peb_u64)
    };
    let page_size = u32::try_from(get_os_page_size())?;

    // we allow this unused mut because it is only used on Linux
    #[allow(unused_mut, unused_variables)]
    let mut host_failed_to_cancel_guest_execution_sending_signals = 0;
    {
        match wrapper_getter.get_hv().try_get_hypervisor_lock() {
            Ok(_) => {
                log::info!("Execution finished while trying to cancel it");
                return Ok(HypervisorHandlerExecutionCancelAttemptOnFinishedExecution());
            }
            Err(e) => {
                log::info!(
                    "Failed to acquire Hypervisor lock: vCPU is spinning ({:?})",
                    e
                );

                log::debug!("Terminating execution of vCPU");
                terminate_execution(
                    max_execution_time,
                    termination_status,
                    #[cfg(target_os = "windows")]
                    partition_handle,
                    #[cfg(target_os = "linux")]
                    run_cancelled,
                    #[cfg(target_os = "linux")]
                    thread_id,
                    #[cfg(target_os = "linux")]
                    max_wait_for_cancellation,
                )?;
            }
        }
    }

    {
        // check the lock to make sure it is free
        match wrapper_getter.get_hv().try_get_hypervisor_lock() {
            Ok(_) => {}
            Err(_) => {
                // If we still fail to acquire the hv_lock, this means that
                // we had actually timed-out on a host function call as the
                // `WHvCancelRunVirtualProcessor` didn't unlock.

                log::debug!("Tried to cancel guest execution on host function call");
                return Err(HyperlightError::GuestExecutionHungOnHostFunctionCall());
            }
        }
    }

    // Receive `ExecutionCancelledByHost` or other
    let res = match try_receive_handler_msg(
        wrapper_getter
            .get_hv()
            .try_get_hypervisor_lock()
            .unwrap()
            .get_from_handler_rx(),
        max_execution_time,
    ) {
        Ok(_) => Ok(new_error!(
            "Expected ExecutionCanceledByHost, but received FinishedHypervisorHandlerAction"
        )),
        Err(e) => match e {
            HyperlightError::ExecutionCanceledByHost() => {
                Ok(HyperlightError::ExecutionCanceledByHost())
            }
            _ => Ok(new_error!(
                "Expected ExecutionCanceledByHost, but received: {:?}",
                e
            )),
        },
    };

    // We cancelled execution, so we restore the state to what it was prior to the bad state
    // that caused the timeout.
    {
        let mem_mgr = wrapper_getter.get_mgr_wrapper_mut().unwrap_mgr_mut();
        mem_mgr.restore_state_from_last_snapshot()?;
    }

    // Re-initialise the vCPU.
    // This is 100% needed because, otherwise, all it takes to cause a DoS is for a
    // function to timeout as the vCPU will be in a bad state without re-init.
    log::debug!("Re-initialising vCPU");
    execute_hypervisor_handler_action(
        wrapper_getter.get_hv(),
        HypervisorHandlerAction::Initialise(InitArgs::new(
            peb_addr,
            seed,
            page_size,
            outb_hdl,
            mem_access_hdl,
        )),
        None,
    )?;

    res
}

/// Send a message to the Hypervisor Handler and wait for a response.
///
/// This function should be used for most interactions with the Hypervisor
/// Handler.
///
/// If no `max_wait_time` is provided, the DEFAULT_MAX_EXECUTION_TIME
/// (1000ms) is used.
pub(crate) fn execute_hypervisor_handler_action(
    hv_wrapper: &HypervisorWrapper,
    hypervisor_handler_action: HypervisorHandlerAction,
    max_wait_time: Option<Duration>,
) -> Result<()> {
    let (to_handler_tx, from_handler_rx) = {
        let hv_lock = hv_wrapper.try_get_hypervisor_lock()?;
        (hv_lock.get_to_handler_tx(), hv_lock.get_from_handler_rx())
    };

    log::debug!(
        "Sending Hypervisor Handler Action: {:?}",
        hypervisor_handler_action
    );
    to_handler_tx
        .send(hypervisor_handler_action)
        .map_err(|_| HyperlightError::HypervisorHandlerCommunicationFailure())?;

    log::debug!("Waiting for Hypervisor Handler Response");
    match max_wait_time {
        Some(wait) => try_receive_handler_msg(from_handler_rx, wait),
        None => try_receive_handler_msg(
            from_handler_rx,
            Duration::from_millis(SandboxConfiguration::DEFAULT_MAX_EXECUTION_TIME as u64),
        ),
    }
}

/// Try to receive a `HandlerMsg` from the Hypervisor Handler Thread.
///
/// Usually, you should use `execute_hypervisor_handler_action` to send and instantly
/// try to receive a message.
///
/// This function is only useful when we time out, handle a timeout,
/// and still have to receive after sorting that out without sending
/// an extra message.
pub(crate) fn try_receive_handler_msg(
    from_handler_rx: Receiver<HandlerMsg>,
    wait: Duration,
) -> Result<()> {
    match from_handler_rx.recv_timeout(wait) {
        Ok(msg) => match msg {
            HandlerMsg::Error(e) => Err(e),
            HandlerMsg::FinishedHypervisorHandlerAction => Ok(()),
        },
        Err(_) => Err(HyperlightError::HypervisorHandlerMessageReceiveTimedout()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    use crossbeam::atomic::AtomicCell;
    use hyperlight_common::flatbuffer_wrappers::function_types::{ParameterValue, ReturnType};
    use hyperlight_testing::simple_guest_as_string;

    use crate::hypervisor::hypervisor_handler::terminate_hypervisor_handler_execution_and_reinitialise;
    use crate::sandbox::{SandboxConfiguration, WrapperGetter};
    use crate::sandbox_state::sandbox::EvolvableSandbox;
    use crate::sandbox_state::transition::MutatingCallback;
    use crate::HyperlightError::HypervisorHandlerExecutionCancelAttemptOnFinishedExecution;
    use crate::{
        is_hypervisor_present, GuestBinary, HyperlightError, MultiUseSandbox, Result,
        UninitializedSandbox,
    };

    fn create_multi_use_sandbox() -> MultiUseSandbox<'static> {
        if !is_hypervisor_present() {
            panic!("Panic on create_multi_use_sandbox because no hypervisor is present");
        }
        let usbox = UninitializedSandbox::new(
            GuestBinary::FilePath(simple_guest_as_string().expect("Guest Binary Missing")),
            None,
            None,
            None,
        )
        .unwrap();

        usbox.evolve(MutatingCallback::from(init)).unwrap()
    }

    fn init(_: &mut UninitializedSandbox) -> Result<()> {
        Ok(())
    }

    #[test]
    #[ignore] // this test runs by itself because it uses a lot of system resources
    fn create_1000_sandboxes() {
        let barrier = Arc::new(Barrier::new(21));

        let mut handles = vec![];

        for _ in 0..20 {
            let c = barrier.clone();

            let handle = thread::spawn(move || {
                c.wait();

                for _ in 0..50 {
                    create_multi_use_sandbox();
                }
            });

            handles.push(handle);
        }

        barrier.wait();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn terminate_execution_then_call_another_function() -> Result<()> {
        let mut sandbox = create_multi_use_sandbox();

        let res = sandbox.call_guest_function_by_name("Spin", ReturnType::Void, None);

        assert!(res.is_err());

        match res.err().unwrap() {
            HyperlightError::ExecutionCanceledByHost() => {}
            _ => panic!("Expected ExecutionTerminated error"),
        }

        let res = sandbox.call_guest_function_by_name(
            "Echo",
            ReturnType::String,
            Some(vec![ParameterValue::String("a".to_string())]),
        );

        assert!(res.is_ok());

        Ok(())
    }

    #[test]
    fn terminate_execution_of_an_already_finished_function_then_call_another_function() -> Result<()>
    {
        let call_print_output = |sandbox: &mut MultiUseSandbox| {
            let msg = "Hello, World!\n".to_string();
            let res = sandbox.call_guest_function_by_name(
                "PrintOutput",
                ReturnType::Int,
                Some(vec![ParameterValue::String(msg.clone())]),
            );

            assert!(res.is_ok());
        };

        let mut sandbox = create_multi_use_sandbox();
        call_print_output(&mut sandbox);

        // this simulates what would happen if a function actually successfully
        // finished while we attempted to terminate execution
        {
            let (outb_hdl, mem_access_hdl) = {
                let hv_wrapper = sandbox.get_hv_mut();
                (
                    hv_wrapper.outb_hdl.clone(),
                    hv_wrapper.mem_access_hdl.clone(),
                )
            };

            #[cfg(target_os = "linux")]
            let thread_id = {
                let hv_lock = sandbox.get_hv_mut().try_get_hypervisor_lock()?;
                hv_lock.get_thread_id()
            };

            #[cfg(target_os = "windows")]
            let partition_handle = {
                let hv_lock = sandbox.get_hv_mut().try_get_hypervisor_lock()?;
                hv_lock.get_partition_handle()
            };

            match terminate_hypervisor_handler_execution_and_reinitialise(
                &mut sandbox,
                Duration::from_millis(SandboxConfiguration::DEFAULT_MAX_EXECUTION_TIME as u64),
                Arc::new(AtomicCell::new(true)),
                outb_hdl,
                mem_access_hdl,
                #[cfg(target_os = "windows")]
                partition_handle,
                #[cfg(target_os = "linux")]
                thread_id,
                #[cfg(target_os = "linux")]
                Arc::new(AtomicCell::new(false)),
                #[cfg(target_os = "linux")]
                Duration::from_millis(100),
            )? {
                HypervisorHandlerExecutionCancelAttemptOnFinishedExecution() => {}
                _ => panic!("Expected error demonstrating execution wasn't cancelled properly"),
            }
        }

        call_print_output(&mut sandbox);
        call_print_output(&mut sandbox);

        Ok(())
    }
}

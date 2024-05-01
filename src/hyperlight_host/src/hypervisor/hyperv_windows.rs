use super::{
    handlers::{MemAccessHandlerWrapper, OutBHandlerWrapper},
    windows_hypervisor_platform::{VMPartition, VMProcessor},
    Hypervisor, CR0_AM, CR0_ET, CR0_MP, CR0_NE, CR0_PE, CR0_PG, CR0_WP, CR4_OSFXSR, CR4_OSXMMEXCPT,
    CR4_PAE, EFER_LMA, EFER_LME,
};
use super::{surrogate_process::SurrogateProcess, surrogate_process_manager::*};
use super::{windows_hypervisor_platform as whp, HyperlightExit};
use crate::mem::layout::SandboxMemoryLayout;
use crate::mem::ptr::GuestPtr;
use crate::Result;
use crate::{
    log_then_return,
    mem::{ptr::RawPtr, shared_mem::PtrCVoidMut},
};
use crate::{
    new_error,
    HyperlightError::{NoHypervisorFound, WindowsErrorHResult},
};
use core::ffi::c_void;
use hyperlight_common::mem::{PAGE_SIZE, PAGE_SIZE_USIZE};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::string::String;
use std::time::Duration;
use std::{any::Any, ops::Range};
use tracing::{instrument, Span};
use windows::Win32::System::Hypervisor::{
    WHvX64RegisterCr0, WHvX64RegisterCr3, WHvX64RegisterCr4, WHvX64RegisterCs, WHvX64RegisterEfer,
    WHvX64RegisterR8, WHvX64RegisterR9, WHvX64RegisterRcx, WHvX64RegisterRdx, WHvX64RegisterRflags,
    WHvX64RegisterRip, WHvX64RegisterRsp, WHV_PARTITION_HANDLE, WHV_REGISTER_NAME,
    WHV_REGISTER_VALUE, WHV_RUN_VP_EXIT_CONTEXT, WHV_RUN_VP_EXIT_REASON, WHV_UINT128,
    WHV_UINT128_0,
};
/// Wrapper around WHV_REGISTER_NAME so we can impl
/// Hash on the struct.
#[derive(PartialEq, Eq)]
pub(super) struct WhvRegisterNameWrapper(pub WHV_REGISTER_NAME);

impl Hash for WhvRegisterNameWrapper {
    #[instrument(skip_all, parent = Span::current(), level= "Trace")]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state);
    }
}

/// A Hypervisor driver for HyperV-on-Windows.
pub(crate) struct HypervWindowsDriver {
    size: usize, // this is the size of the memory region, excluding the 2 surrounding guard pages
    processor: VMProcessor,
    surrogate_process: SurrogateProcess,
    source_address: PtrCVoidMut, // this points into the first guard page
    registers: HashMap<WhvRegisterNameWrapper, WHV_REGISTER_VALUE>,
    orig_rsp: GuestPtr,
    guard_page_region: Range<u64>,
}

impl std::fmt::Debug for HypervWindowsDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HypervLinuxDriver")
            .field("size", &self.size)
            .finish()
    }
}

impl HypervWindowsDriver {
    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    pub(crate) fn new(
        raw_size: usize,
        raw_source_address: *mut c_void,
        sandbox_base_address: u64,
        guard_page_offset: u64,
        pml4_address: u64,
        entry_point: u64,
        rsp: u64,
    ) -> Result<Self> {
        if !whp::is_hypervisor_present() {
            log_then_return!(NoHypervisorFound());
        }

        // create and setup hypervisor partition
        let mut partition = VMPartition::new(1)?;

        // get a surrogate process with preallocated memory of size SharedMemory::raw_mem_size()
        // with guard pages setup
        let surrogate_process = {
            let mgr = get_surrogate_process_manager()?;
            mgr.get_surrogate_process(raw_size, raw_source_address)
        }?;

        // raw_source_address is SharedMem::raw_ptr(), which means it points to
        // the guard page. We need to map the memory starting after the guard page
        let starting_source_address = unsafe { raw_source_address.add(PAGE_SIZE_USIZE) };

        partition.map_gpa_range(
            &surrogate_process.process_handle,
            starting_source_address,
            sandbox_base_address,
            guard_page_offset,
            raw_size,
        )?;

        let proc = VMProcessor::new(partition)?;

        let registers = {
            let mut hm = HashMap::new();

            // prime the registers we will set when to run the workload on a vcpu
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterCr3),
                WHV_REGISTER_VALUE {
                    Reg64: pml4_address,
                },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterCr4),
                WHV_REGISTER_VALUE {
                    Reg64: CR4_PAE | CR4_OSFXSR | CR4_OSXMMEXCPT,
                },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterCr0),
                WHV_REGISTER_VALUE {
                    Reg64: CR0_PE | CR0_MP | CR0_ET | CR0_NE | CR0_WP | CR0_AM | CR0_PG,
                },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterEfer),
                WHV_REGISTER_VALUE {
                    Reg64: EFER_LME | EFER_LMA,
                },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterCs),
                WHV_REGISTER_VALUE {
                    Reg128: WHV_UINT128 {
                        Anonymous: WHV_UINT128_0 {
                            Low64: (0),
                            High64: (0xa09b0008ffffffff),
                        },
                    },
                },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterRflags),
                WHV_REGISTER_VALUE { Reg64: 0x0002 },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterRip),
                WHV_REGISTER_VALUE { Reg64: entry_point },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterRsp),
                WHV_REGISTER_VALUE { Reg64: rsp },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterR8),
                WHV_REGISTER_VALUE { Reg64: 0x0 },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterRdx),
                WHV_REGISTER_VALUE { Reg64: 0x0 },
            );
            hm.insert(
                WhvRegisterNameWrapper(WHvX64RegisterRcx),
                WHV_REGISTER_VALUE { Reg64: 0x0 },
            );
            hm
        };

        // subtract 2 pages for the guard pages, since when we copy memory to and from surrogate process,
        // we don't want to copy the guard pages themselves (that would cause access violation)
        let mem_size = raw_size - 2 * PAGE_SIZE_USIZE;
        Ok(Self {
            size: mem_size,
            processor: proc,
            surrogate_process,
            source_address: PtrCVoidMut::from(raw_source_address),
            registers,
            orig_rsp: GuestPtr::try_from(RawPtr::from(rsp))?,
            guard_page_region: SandboxMemoryLayout::BASE_ADDRESS as u64 + guard_page_offset
                ..SandboxMemoryLayout::BASE_ADDRESS as u64 + guard_page_offset + PAGE_SIZE,
        })
    }

    #[inline]
    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    fn get_exit_details(&self, exit_reason: WHV_RUN_VP_EXIT_REASON) -> Result<String> {
        // get registers
        let register_names = self.registers.keys().map(|x| x.0).collect();
        let registers = self.processor.get_registers(&register_names)?;

        let mut error = String::new();
        error.push_str(&format!(
            "Did not receive a halt from Hypervisor as expected - Received {exit_reason:?}!\n"
        ));
        for (key, value) in registers.iter() {
            unsafe {
                // need access to a union field!
                error.push_str(&format!(
                    "  {:>4?} - 0x{:0>16X} {:0>16X}\n",
                    key.0, value.Reg128.Anonymous.High64, value.Reg128.Anonymous.Low64
                ));
            }
        }
        Ok(error)
    }

    #[instrument(skip_all, parent = Span::current(), level= "Trace")]
    pub(super) fn get_partition_hdl(&self) -> WHV_PARTITION_HANDLE {
        self.processor.get_partition_hdl()
    }
}

impl Hypervisor for HypervWindowsDriver {
    #[instrument(skip_all, parent = Span::current(), level= "Trace")]
    fn as_mut_hypervisor(&mut self) -> &mut dyn Hypervisor {
        self as &mut dyn Hypervisor
    }

    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    fn initialise(
        &mut self,
        peb_address: RawPtr,
        seed: u64,
        page_size: u32,
        outb_hdl: OutBHandlerWrapper,
        mem_access_hdl: MemAccessHandlerWrapper,
        max_execution_time: Duration,
        max_wait_for_cancellation: Duration,
    ) -> Result<()> {
        self.registers.insert(
            WhvRegisterNameWrapper(WHvX64RegisterRcx),
            WHV_REGISTER_VALUE {
                Reg64: peb_address.into(),
            },
        );
        self.registers.insert(
            WhvRegisterNameWrapper(WHvX64RegisterRdx),
            WHV_REGISTER_VALUE { Reg64: seed },
        );
        self.registers.insert(
            WhvRegisterNameWrapper(WHvX64RegisterR8),
            WHV_REGISTER_VALUE { Reg32: page_size },
        );
        self.registers.insert(
            WhvRegisterNameWrapper(WHvX64RegisterR9),
            WHV_REGISTER_VALUE {
                Reg32: self.get_max_log_level(),
            },
        );
        self.processor.set_registers(&self.registers)?;
        self.execute_until_halt(
            outb_hdl,
            mem_access_hdl,
            max_execution_time,
            max_wait_for_cancellation,
        )?;
        // we need to reset the stack pointer once execution is complete
        // the caller is responsible for this in windows x86_64 calling convention and since we are "calling" here we need to reset it
        self.reset_rsp(self.orig_rsp)
    }

    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    fn dispatch_call_from_host(
        &mut self,
        dispatch_func_addr: RawPtr,
        outb_hdl: OutBHandlerWrapper,
        mem_access_hdl: MemAccessHandlerWrapper,
        max_execution_time: Duration,
        max_wait_for_cancellation: Duration,
    ) -> Result<()> {
        let registers = HashMap::from([(
            WhvRegisterNameWrapper(WHvX64RegisterRip),
            WHV_REGISTER_VALUE {
                Reg64: dispatch_func_addr.into(),
            },
        )]);
        self.processor.set_registers(&registers)?;
        // we need to reset the stack pointer once execution is complete
        // the caller is responsible for this in windows x86_64 calling convention and since we are "calling" here we need to reset it
        // so here we get the current RSP value so we can reset it later
        let rsp = self.processor.get_registers(&vec![WHvX64RegisterRsp])?;
        self.execute_until_halt(
            outb_hdl,
            mem_access_hdl,
            max_execution_time,
            max_wait_for_cancellation,
        )?;
        // While there is a function to set the RSP we are not using it because we would end up having to get the value out of the hashmap and then convert it to a u64 only for it to be immediately stored back in a hashmap
        self.processor.set_registers(&rsp)
    }

    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    fn reset_rsp(&mut self, rsp: GuestPtr) -> Result<()> {
        let registers = HashMap::from([(
            WhvRegisterNameWrapper(WHvX64RegisterRsp),
            WHV_REGISTER_VALUE {
                Reg64: rsp.absolute()?,
            },
        )]);
        self.processor.set_registers(&registers)?;
        Ok(())
    }

    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    fn orig_rsp(&self) -> Result<GuestPtr> {
        Ok(self.orig_rsp)
    }

    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    fn handle_io(
        &mut self,
        port: u16,
        data: Vec<u8>,
        rip: u64,
        instruction_length: u64,
        outb_handle_fn: OutBHandlerWrapper,
    ) -> Result<()> {
        let payload = data[..8].try_into()?;
        outb_handle_fn
            .lock()
            .map_err(|e| new_error!("error locking {}", e))?
            .call(port, u64::from_le_bytes(payload))?;
        let registers = HashMap::from([(
            WhvRegisterNameWrapper(WHvX64RegisterRip),
            WHV_REGISTER_VALUE {
                Reg64: rip + instruction_length,
            },
        )]);
        self.processor.set_registers(&registers)
    }

    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    fn run(&mut self) -> Result<super::HyperlightExit> {
        let bytes_written: Option<*mut usize> = None;
        let bytes_read: Option<*mut usize> = None;

        // TODO optimise this
        // the following write to and read from process memory is required as we need to use
        // surrogate processes to allow more than one WHP Partition per process
        // see HyperVSurrogateProcessManager
        // this needs updating so that
        // 1. it only writes to memory that changes between usage
        // 2. memory is allocated in the process once and then only freed and reallocated if the
        // memory needs to grow.

        // - copy stuff to surrogate process
        unsafe {
            if !windows::Win32::System::Diagnostics::Debug::WriteProcessMemory(
                self.surrogate_process.process_handle,
                self.surrogate_process
                    .allocated_address
                    .as_ptr()
                    .add(PAGE_SIZE_USIZE),
                self.source_address.as_ptr().add(PAGE_SIZE_USIZE),
                self.size,
                bytes_written,
            )
            .as_bool()
            {
                let hresult = windows::Win32::Foundation::GetLastError();
                log_then_return!(WindowsErrorHResult(hresult.to_hresult()));
            }
        }

        // - call WHvRunVirtualProcessor
        let exit_context: WHV_RUN_VP_EXIT_CONTEXT = self.processor.run()?;

        // - call read-process memory
        unsafe {
            if !windows::Win32::System::Diagnostics::Debug::ReadProcessMemory(
                self.surrogate_process.process_handle,
                self.surrogate_process
                    .allocated_address
                    .as_ptr()
                    .add(PAGE_SIZE_USIZE),
                self.source_address.as_mut_ptr().add(PAGE_SIZE_USIZE),
                self.size,
                bytes_read,
            )
            .as_bool()
            {
                let hresult = windows::Win32::Foundation::GetLastError();
                log_then_return!(WindowsErrorHResult(hresult.to_hresult()));
            }
        }

        let result = match exit_context.ExitReason {
            // WHvRunVpExitReasonX64IoPortAccess
            WHV_RUN_VP_EXIT_REASON(2i32) => {
                // size of current instruction is in lower byte of _bitfield
                // see https://learn.microsoft.com/en-us/virtualization/api/hypervisor-platform/funcs/whvexitcontextdatatypes)
                let instruction_length = exit_context.VpContext._bitfield & 0xF;
                unsafe {
                    HyperlightExit::IoOut(
                        exit_context.Anonymous.IoPortAccess.PortNumber,
                        exit_context
                            .Anonymous
                            .IoPortAccess
                            .Rax
                            .to_le_bytes()
                            .to_vec(),
                        exit_context.VpContext.Rip,
                        instruction_length as u64,
                    )
                }
            }
            // HvRunVpExitReasonX64Halt
            WHV_RUN_VP_EXIT_REASON(8i32) => HyperlightExit::Halt(),
            // WHvRunVpExitReasonMemoryAccess
            WHV_RUN_VP_EXIT_REASON(1i32) => {
                let gpa = unsafe { exit_context.Anonymous.MemoryAccess.Gpa };
                let access_info =
                    unsafe { exit_context.Anonymous.MemoryAccess.AccessInfo.AsUINT32 };
                if access_info == 0x2 {
                    HyperlightExit::ExecutionAccessViolation(gpa)
                } else if (self.guard_page_region).contains(&gpa) {
                    HyperlightExit::GuardPageViolation(gpa)
                } else {
                    HyperlightExit::Mmio(gpa)
                }
            }
            //  WHvRunVpExitReasonCanceled
            //  Execution was cancelled by the host.
            //  This will happen when guest code runs for too long
            WHV_RUN_VP_EXIT_REASON(8193i32) => HyperlightExit::Cancelled(),
            WHV_RUN_VP_EXIT_REASON(_) => match self.get_exit_details(exit_context.ExitReason) {
                Ok(error) => HyperlightExit::Unknown(error),
                Err(e) => HyperlightExit::Unknown(format!("Error getting exit details: {}", e)),
            },
        };

        Ok(result)
    }

    #[instrument(skip_all, parent = Span::current(), level= "Trace")]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
pub mod tests {
    use crate::Result;
    use crate::{
        hypervisor::{
            handlers::{MemAccessHandler, OutBHandler},
            tests::test_initialise,
        },
        mem::{layout::SandboxMemoryLayout, ptr::GuestPtr, ptr_offset::Offset},
    };
    use serial_test::serial;
    use std::sync::{Arc, Mutex};

    use super::HypervWindowsDriver;

    extern "C" fn outb_fn(_port: u16, _payload: u64) {}
    extern "C" fn mem_access_fn() {}

    #[test]
    #[serial]
    fn test_init() {
        let outb_handler = {
            let func: Box<dyn FnMut(u16, u64) -> Result<()> + Send> =
                Box::new(|_, _| -> Result<()> { Ok(()) });
            Arc::new(Mutex::new(OutBHandler::from(func)))
        };
        let mem_access_handler = {
            let func: Box<dyn FnMut() -> Result<()> + Send> = Box::new(|| -> Result<()> { Ok(()) });
            Arc::new(Mutex::new(MemAccessHandler::from(func)))
        };
        test_initialise(
            outb_handler,
            mem_access_handler,
            |mgr, rsp_ptr, pml4_ptr| {
                let host_addr = mgr.shared_mem.raw_ptr();
                let rsp = rsp_ptr.absolute()?;
                let _guest_pfn = u64::try_from(SandboxMemoryLayout::BASE_ADDRESS << 12)?;
                let entrypoint = {
                    let load_addr = mgr.load_addr.clone();
                    let load_offset_u64 =
                        u64::from(load_addr) - u64::try_from(SandboxMemoryLayout::BASE_ADDRESS)?;
                    let total_offset = Offset::from(load_offset_u64) + mgr.entrypoint_offset;
                    GuestPtr::try_from(total_offset)
                }?;
                let driver = HypervWindowsDriver::new(
                    mgr.shared_mem.raw_mem_size(),
                    host_addr,
                    u64::try_from(SandboxMemoryLayout::BASE_ADDRESS)?,
                    mgr.layout.get_guard_page_offset().into(),
                    pml4_ptr.absolute()?,
                    entrypoint.absolute().unwrap(),
                    rsp,
                )?;

                Ok(Box::new(driver))
            },
        )
        .unwrap();
    }
}

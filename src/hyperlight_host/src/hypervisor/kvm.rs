use super::kvm_regs::{Regs, SRegs};
use anyhow::{anyhow, bail, Result};
use kvm_ioctls::{Cap::UserMemory, Kvm, VcpuExit, VcpuFd, VmFd};

/// The type of the output from a KVM vCPU
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum KvmRunMessageType {
    /// IO Output
    IOOut,
    /// Halt
    Halt,
}

impl<'a> TryFrom<VcpuExit<'a>> for KvmRunMessageType {
    type Error = anyhow::Error;
    fn try_from(e: VcpuExit) -> Result<Self> {
        match e {
            VcpuExit::Hlt => Ok(KvmRunMessageType::Halt),
            VcpuExit::IoOut(_, _) => Ok(KvmRunMessageType::IOOut),
            VcpuExit::InternalError => bail!("KVM internal error"),
            default => bail!("unsupported message type {:?}", default),
        }
    }
}

/// A description of the results of a KVM vpu execution
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct KvmRunMessage {
    /// The exit reason of the vCPU. Will be one
    /// of the KvmMessageType constants.
    pub message_type: KvmRunMessageType,
    /// The value of the RAX register.
    pub rax: u64,
    /// The value of the RIP register.
    pub rip: u64,
    /// The port number when the reason is
    /// KVM_MESSAGE_TYPE_X64_IO_OUT. Otherwise this is set to 0
    pub port_number: u16,
}

/// Return `Ok(())` if the KVM API is available, or `Err` otherwise
pub fn is_present() -> Result<()> {
    let kvm = Kvm::new()?;
    let ver = kvm.get_api_version();
    if -1 == ver {
        bail!("KVM_GET_API_VERSION returned -1");
    } else if ver != 12 {
        bail!("KVM_GET_API_VERSION returned {}, expected 12", ver);
    }
    let cap_user_mem = kvm.check_extension(UserMemory);
    if !cap_user_mem {
        bail!("KVM_CAP_USER_MEMORY not supported");
    }
    Ok(())
}

/// Check if KVM exists on the machine and, if so, open the file
/// descriptor and return a reference to it. Returns `Err` if there
/// were any issues during this process.
pub fn open() -> Result<Kvm> {
    match is_present() {
        Ok(_) => Kvm::new().map_err(|e| anyhow!("Failed to open KVM: {}", e)),
        Err(_) => bail!("KVM is not present"),
    }
}

/// Create a new VM using the given `kvm` handle.
///
/// Returns `Ok` if the creation was successful, `Err` otherwise.
pub fn create_vm(kvm: &Kvm) -> Result<VmFd> {
    kvm.create_vm().map_err(|e| anyhow!(e))
}

/// Create a new virtual CPU from the given `vmfd`
pub fn create_vcpu(vmfd: &VmFd) -> Result<VcpuFd> {
    vmfd.create_vcpu(0).map_err(|e| anyhow!(e))
}

/// Get the registers from the vcpu referenced by `vcpu_fd`.
pub fn get_registers(vcpu_fd: &VcpuFd) -> Result<Regs> {
    vcpu_fd
        .get_regs()
        .map(|r| Regs::from(&r))
        .map_err(|e| anyhow!(e))
}

/// Get the segment registers from the vcpu referenced by `vcpu_fd`.
pub fn get_sregisters(vcpu_fd: &VcpuFd) -> Result<SRegs> {
    vcpu_fd
        .get_sregs()
        .map(|r| SRegs::from(&r))
        .map_err(|e| anyhow!(e))
}

fn get_port_num(vcpu_exit: &VcpuExit) -> Result<u16> {
    match vcpu_exit {
        VcpuExit::IoOut(addr, _) => Ok(*addr as u16),
        _ => bail!("no port num for VcpuExit {:?}", vcpu_exit),
    }
}

fn get_rax(vcpu_fd: &VcpuFd) -> Result<u64> {
    vcpu_fd.get_regs().map(|r| r.rax).map_err(|e| anyhow!(e))
}

fn get_rip(vcpu_fd: &VcpuFd) -> Result<u64> {
    vcpu_fd.get_regs().map(|r| r.rip).map_err(|e| anyhow!(e))
}

/// Run the vcpu referenced by `vcpu_fd` until it exits, and return
/// a `kvm_run_message` indicating what happened.
pub fn run_vcpu(vcpu_fd: &VcpuFd) -> Result<KvmRunMessage> {
    match (vcpu_fd).run() {
        Ok(vcpu_exit) => {
            let port_number = get_port_num(&vcpu_exit).unwrap_or(0);
            let rax = get_rax(vcpu_fd).unwrap_or(0);
            let rip = get_rip(vcpu_fd).unwrap_or(0);
            let message_type = KvmRunMessageType::try_from(vcpu_exit)?;
            Ok(KvmRunMessage {
                message_type,
                rax,
                rip,
                port_number,
            })
        }
        Err(e) => bail!(e),
    }
}

/// Set the given registers `regs` on the vcpu referenced by `vcpu_fd`.
///
/// Return `Ok(())` if the set operation succeeded, or an `Err` if it
/// failed.
pub fn set_registers(vcpu_fd: &VcpuFd, regs: &Regs) -> Result<()> {
    let native_regs = kvm_bindings::kvm_regs::from(regs);
    vcpu_fd.set_regs(&native_regs).map_err(|e| anyhow!(e))
}

/// Set special registers `sregs` on the vcpu referenced by `vcpu_fd`.
///
/// Return `Ok(())` if the set operation succeeded, or an `Err` if it
/// failed.
pub fn set_sregisters(vcpu_fd: &VcpuFd, sregs: &SRegs) -> Result<()> {
    let native_regs = kvm_bindings::kvm_sregs::from(sregs);
    vcpu_fd.set_sregs(&native_regs).map_err(|e| anyhow!(e))
}

#[cfg(test)]
mod tests {
    use crate::{
        hypervisor::kvm_mem::{
            map_vm_memory_region, map_vm_memory_region_raw, unmap_vm_memory_region_raw,
        },
        hypervisor::kvm_regs,
        mem::guest_mem::GuestMemory,
    };
    use anyhow::Result;
    use kvm_ioctls::{Kvm, VcpuFd, VmFd};

    const SHOULD_BE_PRESENT_VAR: &str = "KVM_SHOULD_BE_PRESENT";

    macro_rules! presence_check {
        () => {{
            if !should_be_present() {
                return;
            }
        }};
    }

    fn should_be_present() -> bool {
        std::env::var(SHOULD_BE_PRESENT_VAR).is_ok()
    }

    #[test]
    fn is_present() {
        let pres = super::is_present().is_ok();
        match (should_be_present(), pres) {
            (true, true) => (),
            (false, true) => panic!("KVM was present but should not be"),
            (true, false) => panic!("KVM was not present but should be"),
            (false, false) => (),
        }
    }

    #[test]
    fn create_vm_vcpu() {
        presence_check!();
        let kvm = super::open().unwrap();
        let vm = super::create_vm(&kvm).unwrap();
        let vcpu = super::create_vcpu(&vm).unwrap();
        super::get_registers(&vcpu).unwrap();
        super::get_sregisters(&vcpu).unwrap();
    }

    const GUEST_PHYS_ADDR: u64 = 0x1000;
    const SIZE: usize = 0x4000;
    fn setup_run_vcpu_test() -> Result<(Kvm, VmFd, VcpuFd, GuestMemory)> {
        #[rustfmt::skip]
        const CODE: [u8; 12] = [
            // mov $0x3f8, %dx
            0xba, 0xf8, 0x03,
            // add %bl, %al
            0x00, 0xd8,
            // add $'0', %al
            0x04, b'0',
            // out %al, (%dx)
            0xee,
            // mov $'\n', %al
            0xb0, b'\0',
            // out %al, (%dx)
            0xee,
            // hlt
            0xf4,
        ];
        let kvm = super::open()?;
        let vm = super::create_vm(&kvm)?;
        let vcpu = super::create_vcpu(&vm)?;
        let mut mem = GuestMemory::new(SIZE).unwrap();
        mem.copy_from_slice(&CODE, 0).unwrap();
        Ok((kvm, vm, vcpu, mem))
    }

    fn set_vcpu_registers(vcpu_fd: &VcpuFd) -> Result<()> {
        let regs = kvm_regs::Regs {
            rip: GUEST_PHYS_ADDR,
            rax: 2,
            rbx: 2,
            rflags: 0x2,
            rsp: 0,
            rcx: 0,
        };
        super::set_registers(vcpu_fd, &regs)?;
        let mut sregs = super::get_sregisters(vcpu_fd)?;
        sregs.cs.base = 0;
        sregs.cs.selector = 0;
        super::set_sregisters(vcpu_fd, &sregs)
    }

    fn run_code(vcpu_fd: &VcpuFd) -> Result<()> {
        {
            // first run should be the first IO_OUT
            let run_res = super::run_vcpu(vcpu_fd)?;
            assert_eq!(super::KvmRunMessageType::IOOut, run_res.message_type);
            assert_eq!('4' as u64, run_res.rax);
            assert_eq!(0x3f8, run_res.port_number);
            let regs_after = super::get_registers(vcpu_fd)?;
            assert_eq!(run_res.rip, regs_after.rip);
        }
        {
            // second run should be the second IO_OUT
            let run_res = super::run_vcpu(vcpu_fd)?;
            assert_eq!(super::KvmRunMessageType::IOOut, run_res.message_type);
            assert_eq!(0, run_res.rax);
            assert_eq!(0x3f8, run_res.port_number);
        }
        {
            // third run should be the HLT
            let run_res = super::run_vcpu(vcpu_fd)?;
            assert_eq!(super::KvmRunMessageType::Halt, run_res.message_type);
        }
        Ok(())
    }

    #[test]
    fn run_vcpu_raw() {
        presence_check!();
        let (_kvm, vm_fd, vcpu_fd, mem) = setup_run_vcpu_test().unwrap();
        let mut mem_region = map_vm_memory_region_raw(
            &vm_fd,
            GUEST_PHYS_ADDR,
            mem.raw_ptr(),
            mem.mem_size() as u64,
        )
        .unwrap();
        set_vcpu_registers(&vcpu_fd).unwrap();
        run_code(&vcpu_fd).unwrap();
        unmap_vm_memory_region_raw(&vm_fd, &mut mem_region).unwrap();
    }

    #[test]
    fn run_vcpu() {
        presence_check!();
        let (_kvm, vm_fd, vcpu_fd, mem) = setup_run_vcpu_test().unwrap();
        let _mem_region = map_vm_memory_region(&vm_fd, GUEST_PHYS_ADDR, &mem).unwrap();
        set_vcpu_registers(&vcpu_fd).unwrap();
        run_code(&vcpu_fd).unwrap();
    }
}

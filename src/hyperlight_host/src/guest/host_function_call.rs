extern crate flatbuffers;
use super::function_call::{ReadFunctionCallFromMemory, WriteFunctionCallToMemory};
#[cfg(debug_assertions)]
use crate::flatbuffers::hyperlight::generated::{
    size_prefixed_root_as_function_call, FunctionCallType as FBFunctionCallType,
};
use crate::mem::layout::SandboxMemoryLayout;
use crate::mem::shared_mem::SharedMemory;
use anyhow::{anyhow, Result};
/// A host function call is a function call from theguest to the host.
#[derive(Default)]
pub struct HostFunctionCall {}

impl WriteFunctionCallToMemory for HostFunctionCall {
    fn write(
        &self,
        function_call_buffer: &[u8],
        shared_memory: &mut SharedMemory,
        layout: &SandboxMemoryLayout,
    ) -> Result<()> {
        let buffer_size = {
            let size_u64 = shared_memory.read_u64(layout.get_output_data_size_offset())?;
            usize::try_from(size_u64)
                .map_err(|_| anyhow!("could not convert buffer size u64 ({}) to usize", size_u64))
        }?;

        if function_call_buffer.len() > buffer_size {
            return Err(anyhow!(
                "Host function call buffer is too big for the output data buffer"
            ));
        }

        #[cfg(debug_assertions)]
        validate_host_function_call_buffer(function_call_buffer)?;
        shared_memory.copy_from_slice(function_call_buffer, layout.output_data_buffer_offset)?;

        Ok(())
    }
}

impl ReadFunctionCallFromMemory for HostFunctionCall {
    fn read(&self, shared_memory: &SharedMemory, layout: &SandboxMemoryLayout) -> Result<Vec<u8>> {
        // Get th size of the flatbuffer buffer from memory

        let fb_buffer_size = {
            let size_i32 = shared_memory.read_i32(layout.output_data_buffer_offset)? + 4;
            usize::try_from(size_i32)
                .map_err(|_| anyhow!("could not convert buffer size i32 ({}) to usize", size_i32))
        }?;

        let mut function_call_buffer = vec![0; fb_buffer_size];
        shared_memory.copy_to_slice(&mut function_call_buffer, layout.output_data_buffer_offset)?;
        #[cfg(debug_assertions)]
        validate_host_function_call_buffer(&function_call_buffer)?;

        Ok(function_call_buffer)
    }
}

#[cfg(debug_assertions)]
fn validate_host_function_call_buffer(function_call_buffer: &[u8]) -> Result<()> {
    let host_function_call_fb =
        size_prefixed_root_as_function_call(function_call_buffer).map_err(|e| anyhow!(e))?;
    match host_function_call_fb.function_call_type() {
        FBFunctionCallType::host => Ok(()),
        _ => anyhow::bail!("Unexpected function call type"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::config::SandboxMemoryConfiguration;
    use crate::testing::get_host_function_call_test_data;
    use anyhow::Result;

    #[test]
    fn write_to_memory() -> Result<()> {
        let test_data = get_host_function_call_test_data();
        let host_function_call = HostFunctionCall {};
        let memory_config = SandboxMemoryConfiguration::new(0, 0, 0, 0, 0, None, None);
        let mut shared_memory = SharedMemory::new(32768)?;
        let memory_layout = SandboxMemoryLayout::new(memory_config, 4096, 4096, 4096)?;
        let result = host_function_call.write(&test_data, &mut shared_memory, &memory_layout);
        assert!(result.is_err());
        assert_eq!(
            "Host function call buffer is too big for the output data buffer",
            result.err().unwrap().to_string()
        );

        let test_data = get_host_function_call_test_data();
        let host_function_call = HostFunctionCall {};
        let memory_config = SandboxMemoryConfiguration::new(0, 1024, 0, 0, 0, None, None);
        let memory_layout = SandboxMemoryLayout::new(memory_config, 4096, 4096, 4096)?;
        let mem_size = memory_layout.get_memory_size()?;
        let mut shared_memory = SharedMemory::new(mem_size)?;
        let offset = shared_memory.base_addr();
        memory_layout.write(&mut shared_memory, offset, mem_size)?;

        let result = host_function_call.write(&test_data, &mut shared_memory, &memory_layout);
        assert!(result.is_ok());

        Ok(())
    }
}

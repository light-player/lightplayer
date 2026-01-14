use crate::error::Error;
use crate::nodes::NodeRuntime;
use crate::runtime::contexts::{NodeInitContext, RenderContext};
use alloc::vec::Vec;

/// Output node runtime
pub struct OutputRuntime {
    /// Channel data buffer (DMX-style, sequential bytes)
    channel_data: Vec<u8>,
}

impl OutputRuntime {
    pub fn new() -> Self {
        Self {
            channel_data: Vec::new(),
        }
    }

    /// Get mutable slice to channel data, extending if needed
    pub fn get_buffer_mut(&mut self, start_ch: u32, ch_count: u32) -> &mut [u8] {
        let end = (start_ch + ch_count) as usize;
        if end > self.channel_data.len() {
            self.channel_data.resize(end, 0);
        }
        &mut self.channel_data[start_ch as usize..end]
    }

    /// Get channel data (for state extraction)
    pub fn get_channel_data(&self) -> &[u8] {
        &self.channel_data
    }
}

impl NodeRuntime for OutputRuntime {
    fn init(&mut self, _ctx: &dyn NodeInitContext) -> Result<(), Error> {
        // todo!("Output initialization - setup GPIO, etc.")
        Ok(())
    }

    fn render(&mut self, _ctx: &mut dyn RenderContext) -> Result<(), Error> {
        // todo!("Output rendering - flush buffers")
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_runtime_creation() {
        let runtime = OutputRuntime::new();
        let _boxed: alloc::boxed::Box<dyn NodeRuntime> = alloc::boxed::Box::new(runtime);
    }
}

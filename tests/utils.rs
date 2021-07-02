extern crate anyhow;
extern crate log;
extern crate simple_logger;

use rustacuda::{
    context::{Context, ContextFlags},
    device::Device,
};

use anyhow::Result;

#[macro_export]
macro_rules! info_ctx {
    ($ctx:expr, $($arg:tt)+) => {
        log::info!(target: &format!("{}::{}", std::module_path!(), $ctx), $($arg)+)
    };
}

pub fn init_cuda_ctx() -> Result<Context> {
    rustacuda::init(rustacuda::CudaFlags::empty())?;
    let device = Device::get_device(0)?;
    let context =
        Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;
    Ok(context)
}

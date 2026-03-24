use cudarc::driver::{
    sys::{cuCtxPopCurrent_v2, cuCtxPushCurrent_v2, CUcontext},
    CudaContext, DriverError,
};
use std::ptr::null_mut;

pub(crate) struct ContextStack;

impl ContextStack {
    pub(crate) fn push(ctx: &CudaContext) -> Result<(), DriverError> {
        let cu_ctx = ctx.cu_ctx();
        let result = unsafe { cuCtxPushCurrent_v2(cu_ctx) };
        result.result()
    }

    pub(crate) fn pop() -> Result<(), DriverError> {
        let mut cu_ctx: CUcontext = null_mut();
        let result = unsafe { cuCtxPopCurrent_v2(&raw mut cu_ctx) };

        result.result()
    }
}

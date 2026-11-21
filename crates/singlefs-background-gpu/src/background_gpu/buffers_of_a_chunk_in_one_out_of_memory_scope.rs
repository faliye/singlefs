//! 一段的缓冲与两份回读在同一个显存不足作用域里建（wgpu 设备那一份实现），建缓冲时显存不足交回建不成而不 panic。

use super::*;

impl DeviceCreatingTheBuffersOfAChunk for wgpu::Device {
    type Buffer = wgpu::Buffer;
    type OutOfMemoryScope = wgpu::ErrorScopeGuard;

    fn open_the_out_of_memory_scope(&self) -> wgpu::ErrorScopeGuard {
        self.push_error_scope(wgpu::ErrorFilter::OutOfMemory)
    }

    fn readback_of_one_kernel(&self, outcome_bytes: GpuMemoryInBytes) -> wgpu::Buffer {
        self.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: outcome_bytes.0,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn bytes_of_the_buffer(&self, buffer: &wgpu::Buffer) -> GpuMemoryInBytes {
        GpuMemoryInBytes(buffer.size())
    }

    fn out_of_memory_captured_by(
        &self,
        scope: wgpu::ErrorScopeGuard,
    ) -> impl Future<Output = Option<String>> {
        let popped = scope.pop();
        async move { popped.await.map(|out_of_memory| format!("{out_of_memory}")) }
    }
}

/// 在同一个只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存（`outcome_bytes` 字节）：回读暂存算在这一段的占用里
/// （`bytes_held_by_this_chunk`），建在范围之外时它显存不足报成没接住的错、这一批出错停下，不走「这一批转给别的卡」。
/// 账核真建的缓冲：关上范围之后把这一段真建的每一份缓冲要的字节加起来（`create_the_buffers` 建的都映射着建，各连同它的暂存算
/// [`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`] 份；两份回读暂存不映射着建，各算一份），与记在这张卡账上的 `bytes_held_by_this_chunk`
/// 比（账在建之前按切批的占用记下、记的时候断言过不超上限）；多建一块、少记一块都在这里断言，不带着比账多的显存往下交。
/// 建得成交回缓冲与回读暂存；范围接住了显存不足就丢掉建出来的、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]。
/// 等设备交回那个范围用这张卡等一批的结果的期限（`budget`，[`GpuBudgetUnderTheSettings::deadline_of_a_wait_of_the_batch_starting_now`]，
/// 同交一段之后等它那几等）：等满此刻设置给的上限，或调用方在停而放弃了那一等（D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交」），
/// 照那一等的错交回。不动这一段的账（调用方放）。
///
/// # Errors
/// 上面那三种。
///
/// # Panics
/// 真建的缓冲合起来的字节数与 `bytes_held_by_this_chunk` 不等：切批按的占用与建缓冲对不上，是 bug。
pub(super) fn buffers_and_both_readbacks_created_within_one_out_of_memory_scope<
    Device,
    Buffers,
    Submission,
>(
    device: &Device,
    budget: &GpuBudgetUnderTheSettings<Submission>,
    bytes_held_by_this_chunk: GpuMemoryInBytes,
    outcome_bytes: GpuMemoryInBytes,
    create_the_buffers: impl FnOnce() -> Buffers,
) -> Result<BuffersOfAChunk<Buffers, Device::Buffer>, GpuFailureCause>
where
    Device: DeviceCreatingTheBuffersOfAChunk,
    Buffers: AsRef<[Device::Buffer]>,
{
    let out_of_memory_scope = device.open_the_out_of_memory_scope();
    let buffers = create_the_buffers();
    let readbacks: [Device::Buffer; 2] =
        std::array::from_fn(|_kernel| device.readback_of_one_kernel(outcome_bytes));
    let out_of_memory_captured = device.out_of_memory_captured_by(out_of_memory_scope);
    let clock = SteadyClockOfThisProcess::starting_now();
    let captured = wait_for_the_wgpu_future_before_the_deadline(
        out_of_memory_captured,
        &budget.deadline_of_a_wait_of_the_batch_starting_now(&clock),
        &clock,
    );
    let bytes_of_the_buffers_created: GpuMemoryInBytes = buffers
        .as_ref()
        .iter()
        .map(|buffer_mapped_at_creation| {
            COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
                .bytes_of_the_copies(device.bytes_of_the_buffer(buffer_mapped_at_creation))
        })
        .chain(
            readbacks
                .iter()
                .map(|readback| device.bytes_of_the_buffer(readback)),
        )
        .sum();
    assert_eq!(
        bytes_of_the_buffers_created, bytes_held_by_this_chunk,
        "这一段真建的缓冲合起来的字节数与记在这张卡账上的不等：切批按的占用与建缓冲对不上，账上的峰值管不住卡上真占的"
    );
    match captured {
        Ok(None) => Ok(BuffersOfAChunk { buffers, readbacks }),
        Ok(Some(reason)) => Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
            bytes_held_by_the_chunk: bytes_held_by_this_chunk,
            reason,
        }),
        Err(cause) => Err(cause),
    }
}

/// 在卡上建一份 `bytes` 字节的缓冲（`usage`），建的时候映射着：映射得上才交给 `write_the_contents` 写、再解开映射。
/// 不用 wgpu 的 `create_buffer_init`：它建完就对映射区 `expect`，缓冲因显存不足建不成时那一句 panic（wgpu 30.0.1
/// `src/util/device.rs`），调用方开着的显存不足的错误范围没机会关、走不到「切小重切、这一批转给别的卡」。映射不上只有一种：缓冲没建成（显存不足），
/// 那一条错已归调用方开着的那个范围，这里不写、不解开映射（对建不成的缓冲解开映射会另报一条没接住的错）。
/// 用法里没有 `MAP_WRITE`：wgpu-core 30.0.1 为它另建一份同尺寸的暂存，解开映射时挂进队列的待写，冲掉它的那一次提交做完才放
/// （[`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`]，切批与记账都算这一份）。
pub(super) fn buffer_mapped_at_creation_within_the_scope_of_the_caller(
    device: &wgpu::Device,
    bytes: GpuMemoryInBytes,
    usage: wgpu::BufferUsages,
    write_the_contents: impl FnOnce(&mut wgpu::BufferViewMut),
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytes.0,
        usage,
        mapped_at_creation: true,
    });
    if let Ok(mut mapped) = buffer.slice(..).get_mapped_range_mut() {
        write_the_contents(&mut mapped);
        drop(mapped);
        buffer.unmap();
    }
    buffer
}

/// 在卡上建一份装着 `contents` 的缓冲（`usage`），字节数按整字向上补、至少一个整字（`memory_limit_and_chunks::input_buffer_bytes`，与切批记账同一种算法），
/// 建法见 [`buffer_mapped_at_creation_within_the_scope_of_the_caller`]。
pub(super) fn buffer_with_contents_created_within_the_scope_of_the_caller(
    device: &wgpu::Device,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    buffer_mapped_at_creation_within_the_scope_of_the_caller(
        device,
        input_buffer_bytes(bytes_in_a_buffer(contents)),
        usage,
        |mapped| mapped.slice(..contents.len()).copy_from_slice(contents),
    )
}

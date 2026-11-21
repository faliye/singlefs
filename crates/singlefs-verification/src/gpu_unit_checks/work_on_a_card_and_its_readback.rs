//! 一张起好的卡上的活：给崩溃放量的额度、设备与队列、按源码摘要只编一次的管线、在卡上算几段字节的 CRC-32C，
//! 与派活之后等卡、回读映射（等不成先把设备记成丢了）。

use super::*;

/// 一串 32 位字按小端摊成字节：交给 GPU 的缓冲区按这一种排法写。
pub(crate) fn words_to_bytes(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

/// 回读缓冲区映射好了（或映射不成）时 wgpu 调的回调（`map_async` 收的那一个）：把结果送给等它的那一头。
/// 那一头已经不等了（等卡没等成、panic 了，展开时先丢了收的那一头）就丢掉这次的结果、不 panic：wgpu 在收拾这块回读缓冲区时
/// （展开途中它被丢掉）才调起这个回调，这里再 panic 就是析构途中再 panic，整个进程中止，同一二进制里余下的用例一条都不跑
/// （m4 验证段：等卡 300 秒超时之后 `回调送得出去: SendError`、SIGABRT）。判器派活、事实表核对、单元校验和、持久掩码探针都用这一份。
pub fn readback_mapping_callback(
    sender: std::sync::mpsc::Sender<Result<(), wgpu::BufferAsyncError>>,
) -> impl FnOnce(Result<(), wgpu::BufferAsyncError>) + Send + 'static {
    move |mapping_result| {
        // 送不出去只有一种：收的那一头已经丢了，这次的结果没人要
        let _result_nobody_waits_for_any_more = sender.send(mapping_result);
    }
}

/// 一次提交之后的等（`polled` 是 `device.poll(Wait { … })` 的结局）与回读映射的结果（[`readback_mapping_callback`] 送进 `receiver` 的那一个）。
/// 等不成（超时、设备丢了）先把这张卡的设备记成丢了（`condition`，这台设备每个克隆共用的那一份；wgpu 的回调已经记过的不改，它记的缘由更准），
/// 卡池下一次借这张卡时照它重起一个（`DeviceStartedOnDemand::started_or_start` 看 `is_lost`），后面的用例不再往这台设备上提交；
/// 再 panic（`what_was_waited_for` 交的那一句），只让这一次派活所在的用例红。
///
/// # Panics
/// 等不成；等成了而回调没送到，或映射不成。
pub fn take_the_readback_after_the_wait_or_record_the_device_lost(
    condition: &DeviceConditionOfACard,
    polled: Result<wgpu::PollStatus, wgpu::PollError>,
    receiver: &std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    what_was_waited_for: impl FnOnce(&wgpu::PollError) -> String,
) {
    if let Err(error) = polled {
        let failure = what_was_waited_for(&error);
        if !condition.is_lost() {
            condition.record_the_device_lost(
                format!("等卡没等成：{failure}"),
                CauseOfTheDeviceLoss::Other,
            );
        }
        panic!("{failure}");
    }
    receiver
        .recv()
        .expect("回调送达：等成了，映射的回调在 poll 里调过")
        .expect("回读映射成功");
}

impl GpuCard {
    /// 崩溃放量在这张卡上最多占多少显存（MiB）：给了额度的取额度与空闲显存里小的那个，
    /// 没给额度的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]。
    #[must_use]
    pub fn memory_budget_mebibytes(&self) -> u64 {
        memory_budget_mebibytes_of(self.memory_quota_mebibytes, self.free_memory_mebibytes)
    }

    /// 这张卡空着时崩溃放量在上面最多占多少显存（MiB，[`memory_budget_when_the_card_is_empty_mebibytes_of`]）：与
    /// [`Self::memory_budget_mebibytes`] 同一个算法，空闲换成空卡上能用的。
    #[must_use]
    pub fn memory_budget_when_the_card_is_empty_mebibytes(&self) -> u64 {
        memory_budget_when_the_card_is_empty_mebibytes_of(
            self.memory_quota_mebibytes,
            self.free_memory_mebibytes,
            self.usable_memory_of_the_empty_card_mebibytes,
        )
    }

    /// 这张卡的设备（崩溃放量第 ② 段的核对内核在同一张卡上另起自己的管线）。
    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// 这张卡的队列。
    #[must_use]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// 这个设备上 wgpu 报过的 Out of Memory 与设备丢没丢（几个克隆共用一份）。
    #[must_use]
    pub fn device_condition(&self) -> &DeviceConditionOfACard {
        &self.device_condition
    }

    /// 这个设备上按 `source_digest` 编好的那组管线：这个设备（这张卡与它的每个克隆）上第一次要这个摘要时调 `compile` 编，
    /// 别的线程同时要同一个摘要就等这一次编完、拿同一组；以后再要直接拿。管线绑在设备上，另起的设备（另一次 [`gpu_cards_by_priority`]）另有一份。
    /// 冷编一套判器内核要几分钟、驱动编的时候吃几 GiB 主机内存：几十条用例共用一张卡时，每条各编一遍既慢又撞主机内存上限。
    /// 编的那一段在设备上记着是为这条线程上的哪段活编的（[`DeviceConditionOfACard::compile_for_the_work_of_this_thread`]）：
    /// `compile` 另起的线程上没接住的 Out of Memory 算那段活的。真编了（缓存没命中）打一行 [`PIPELINE_COMPILE_LIFECYCLE_PHASE`]。
    ///
    /// # Panics
    /// `compile` panic：原样传出，这一格照旧空着，下一个要这个摘要的再编一次。
    #[must_use]
    pub fn pipelines_compiled_once_on_this_device(
        &self,
        source_digest: [u8; DIGEST_BYTES],
        compile: impl FnOnce() -> Vec<wgpu::ComputePipeline>,
    ) -> Vec<wgpu::ComputePipeline> {
        compiled_once_for_the_source_digest(
            &self.compiled_pipelines_by_source_digest,
            source_digest,
            || {
                self.device_condition
                    .compile_for_the_work_of_this_thread(compile)
            },
            |pipelines, compile_microseconds| {
                gpu_lifecycle_trace(
                    self,
                    PIPELINE_COMPILE_LIFECYCLE_PHASE,
                    format_args!(
                        "source_digest={} pipelines={} compile_microseconds={compile_microseconds}",
                        crate::hexadecimal::hexadecimal_text(&source_digest),
                        pipelines.len()
                    ),
                );
            },
        )
    }

    /// 一批字节段（每段不超过 32 KiB）在这张卡上算 CRC-32C，按次序交回。
    ///
    /// # Panics
    /// 有一段超过 32 KiB；回读的结果序号对不上（着色器与派发错了，不许带着错结果往下走）。
    #[must_use]
    pub fn crc32c_of_ranges(&self, ranges: &[&[u8]]) -> Vec<u32> {
        if ranges.is_empty() {
            return Vec::new();
        }
        let count = ranges.len();
        let mut slots = vec![0u8; count * SLOT_BYTES];
        let mut lengths = Vec::with_capacity(count);
        for (index, range) in ranges.iter().enumerate() {
            assert!(
                range.len() <= SLOT_BYTES,
                "一段 {} 字节，超过一个槽的 {SLOT_BYTES}",
                range.len()
            );
            slots[index * SLOT_BYTES..index * SLOT_BYTES + range.len()].copy_from_slice(range);
            lengths.push(u32::try_from(range.len()).expect("不超过 32 KiB"));
        }
        let words_to_bytes = |values: &[u32]| {
            values
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<u8>>()
        };
        let count_u32 = u32::try_from(count).expect("一批的段数装得进 u32");
        let device = &self.device;
        let parameters = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&[count_u32, 0, 0, 0]),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let input = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &slots,
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let length_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&lengths),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let result_bytes = u64::try_from(count * RESULT_ENTRY_BYTES).expect("装得进 u64");
        let results = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &vec![0xFFu8; count * RESULT_ENTRY_BYTES],
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            },
        );
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: result_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: parameters.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: length_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: results.as_entire_binding(),
                },
            ],
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                u32::try_from(count.div_ceil(UNIT_CHECKSUM_KERNEL_WORKGROUP_SIZE))
                    .expect("工作组数装得进 u32"),
                1,
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&results, 0, &staging, 0, result_bytes);
        let submission = self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, readback_mapping_callback(sender));
        // 只等这一次提交：几条用例共用这张卡的设备时，不等别的用例提交的活
        let polled = device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        });
        take_the_readback_after_the_wait_or_record_the_device_lost(
            &self.device_condition,
            polled,
            &receiver,
            |error| format!("卡 {:?} 上等单元校验和算完没等成：{error:?}", self.name),
        );
        let mapped = slice.get_mapped_range().expect("读得到回读缓冲区");
        let mut crcs = Vec::with_capacity(count);
        let (entries, remainder) = mapped.as_chunks::<RESULT_ENTRY_BYTES>();
        assert!(
            remainder.is_empty(),
            "回读缓冲区的长度是每条结果 8 字节的整数倍"
        );
        for (index, chunk) in entries.iter().enumerate() {
            let crc = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let local_offset = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            assert_eq!(
                usize::try_from(local_offset).expect("装得进 usize"),
                index,
                "GPU 回来的第 {index} 条结果序号不对"
            );
            crcs.push(crc);
        }
        drop(mapped);
        staging.unmap();
        crcs
    }
}

#[cfg(test)]
mod tests;

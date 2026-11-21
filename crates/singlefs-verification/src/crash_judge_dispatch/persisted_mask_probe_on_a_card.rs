//! 对拍用的持久掩码探针：在一张卡上跑序号 → 持久掩码的探针内核，按序号次序交回每个状态的探针字。

use super::*;

/// 对拍用：在这张卡上跑序号 → 持久掩码的探针内核（`crate::crash_judge_gpu::persisted_mask_probe_kernel_source`，三个判器内核拆掩码的同一份
/// `begin_state`），一次派活拆完 `ordinals` 这一段，按序号次序交回每个状态的探针字（持久掩码 64 个字与判不了的原因）。
///
/// # Panics
/// 打包的表超过这张卡的绑定上限；这一段多于一次派活的工作组上限装得下的状态；回读长度对不上。
#[must_use]
pub fn persisted_masks_on_the_card(
    card: &GpuCard,
    tables: &PackedJudgeTables,
    ordinals: Range<u64>,
) -> Vec<[u32; PERSISTED_MASK_PROBE_WORDS]> {
    let count = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
    if count == 0 {
        return Vec::new();
    }
    let device = card.device();
    let packed = &tables.words;
    let table_bytes = u64::try_from(packed.len() * 4).expect("装得进 u64");
    let limit = device.limits().max_storage_buffer_binding_size;
    assert!(
        table_bytes <= limit,
        "判器的表 {table_bytes} 字节超过这张卡的绑定上限 {limit}"
    );
    let pipeline = compute_pipeline(
        card,
        "crash-judge-persisted-mask-probe",
        &persisted_mask_probe_kernel_source(),
        KERNEL_ENTRY_POINT,
    );
    let tables_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("crash-judge-persisted-mask-probe-tables"),
            contents: &words_to_bytes(packed),
            usage: wgpu::BufferUsages::STORAGE,
        },
    );
    let dispatch_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("crash-judge-persisted-mask-probe-dispatch"),
            contents: &words_to_bytes(&dispatch_words(ordinals.start, count)),
            usage: wgpu::BufferUsages::UNIFORM,
        },
    );
    let result_bytes = u64::try_from(count * PERSISTED_MASK_PROBE_WORDS * 4).expect("装得进 u64");
    let masks = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("crash-judge-persisted-mask-probe-masks"),
            contents: &words_to_bytes(&vec![
                VERDICT_WORD_NOT_WRITTEN;
                count * PERSISTED_MASK_PROBE_WORDS
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        },
    );
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("crash-judge-persisted-mask-probe-staging"),
        size: result_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("crash-judge-persisted-mask-probe"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: DISPATCH_BINDING,
                resource: dispatch_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: TABLES_BINDING,
                resource: tables_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: VERDICTS_BINDING,
                resource: masks.as_entire_binding(),
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
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            u32::try_from(count.div_ceil(JUDGE_KERNEL_WORKGROUP_SIZE)).expect("工作组数装得进 u32"),
            1,
            1,
        );
    }
    encoder.copy_buffer_to_buffer(&masks, 0, &staging, 0, result_bytes);
    let submission = card.queue().submit(std::iter::once(encoder.finish()));
    let slice = staging.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, readback_mapping_callback(sender));
    let polled = device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(std::time::Duration::from_secs(
            LONGEST_WAIT_FOR_THE_CARD_SECONDS,
        )),
    });
    take_the_readback_after_the_wait_or_record_the_device_lost(
        card.device_condition(),
        polled,
        &receiver,
        |error| {
            format!(
                "等了 {LONGEST_WAIT_FOR_THE_CARD_SECONDS} 秒，卡 {:?} 没把持久掩码探针算完：{error:?}",
                card.name
            )
        },
    );
    let mapped = slice.get_mapped_range().expect("读得到回读缓冲区");
    let (entries, remainder) = mapped.as_chunks::<4>();
    assert!(remainder.is_empty(), "回读缓冲区的长度是 4 的整数倍");
    assert_eq!(
        entries.len(),
        count * PERSISTED_MASK_PROBE_WORDS,
        "回读的字数与派的相同"
    );
    let out = entries
        .chunks(PERSISTED_MASK_PROBE_WORDS)
        .map(|state| {
            let mut words = [0u32; PERSISTED_MASK_PROBE_WORDS];
            for (word, chunk) in words.iter_mut().zip(state) {
                *word = u32::from_le_bytes(*chunk);
            }
            words
        })
        .collect();
    drop(mapped);
    staging.unmap();
    out
}

use super::*;

// ============================================================================
// 五、实例 1：mkfs → 可写挂载 → 第一个文件 s 单元 → p 次 `Overwrite`，最后一次发布就是被藏根 R_h。
//    登记第一节：第一个文件 s 单元 = 先发一个单元的第一个文件（`publish_first_file` 只收一个数据单元），s 大于 1 再顺序写 s 个单元的整份内容；
//    `Overwrite` 是 `publish_overwrite`（一个数据单元以内的内容，所以第一次覆盖写之后文件只剩一个单元）。
// ============================================================================

/// 实例 1 里每次覆盖写要写几个数据单元：登记修订 12.4 的读法是一个（`publish_overwrite` 的契约），
/// 修订 12.11 的补臂是第一个文件的 s 个。
fn replacing_write_unit_count(specification: &HistorySpecification) -> u64 {
    match specification.overwrite_shape {
        OverwriteShape::OneUnit => 1,
        OverwriteShape::KeepingTheFirstFileUnits => specification.first_file_units,
    }
}

/// 实例 1 里的一次覆盖写：读法 12.4 走 `publish_overwrite`（一个数据单元）；补臂 12.11 走 `publish_sequential_write`
/// （整份 s 个数据单元，文件仍是 s 个单元）。内容按标签与单元数确定，两次逐字节相同。
fn publish_the_replacing_write(
    writer: &mut PoolWriter<'_, RecordingBlockDevice<ReadsZeroDevice>>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    specification: &HistorySpecification,
    content_tag: u64,
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let content = deterministic_content(
        content_tag,
        usize::try_from(replacing_write_unit_count(specification)).expect("单元数装得进 usize")
            * DATA_UNIT_PAYLOAD_BYTES_LOCAL,
    );
    let file = FirstFile {
        content: &content,
        write_time_seconds,
    };
    match specification.overwrite_shape {
        OverwriteShape::OneUnit => publish_overwrite(writer, allocator, previous, file, instance),
        OverwriteShape::KeepingTheFirstFileUnits => {
            publish_sequential_write(writer, allocator, previous, file, instance)
        }
    }
}

pub(super) struct FirstInstance {
    /// R_h 那次发布之前的镜像。
    pub(super) pool_before_the_last_publish: MemoryPool,
    pub(super) operations: Vec<RetainedOperation>,
    pub(super) last_publish_start: usize,
    pub(super) last_root: RootKey,
    pub(super) roots: BTreeMap<RootKey, RootLog>,
}

pub(super) fn run_the_first_instance(
    specification: &HistorySpecification,
) -> Result<FirstInstance, String> {
    let parameters = pool_parameters();
    let device_bytes = specification.geometry.device_bytes();
    let stream = SharedStream::retaining_contents();
    let state = Rc::new(RefCell::new(ReadsZeroState::default()));
    let mut devices = recording_devices(&empty_pool(device_bytes), &stream, &state);
    make_filesystem(&parameters, devices.as_mut_slice())
        .map_err(|error| format!("实例 1 mkfs：{error:?}"))?;
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("实例 1 可写挂载：{error:?}"))?;
    let instance = output.instance;
    let PoolVersion::WithoutFile(version) = &current else {
        return Err("实例 1 可写挂载之后现行那一版应当还没有文件".to_string());
    };
    let first_content = deterministic_content(0, DATA_UNIT_PAYLOAD_BYTES_LOCAL);
    let mut previous: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            current.record_bytes(),
        )
        .map_err(|error| format!("第一个文件：{error:?}"))?
    };
    if specification.first_file_units > 1 {
        let content = deterministic_content(
            1,
            usize::try_from(specification.first_file_units).expect("单元数装得进 usize")
                * DATA_UNIT_PAYLOAD_BYTES_LOCAL,
        );
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        previous = publish_sequential_write(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .map_err(|error| {
            format!(
                "顺序写 {} 个单元：{error:?}",
                specification.first_file_units
            )
        })?;
    }
    let mut last_publish_start = stream.operation_count();
    for overwrite_index in 0..specification.overwrites_before_the_hidden_publish {
        last_publish_start = stream.operation_count();
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        previous = publish_the_replacing_write(
            &mut writer,
            &mut allocator,
            &previous,
            specification,
            100 + overwrite_index,
            FIXED_WRITE_TIME_SECONDS + 120 + overwrite_index,
            instance,
        )
        .map_err(|error| format!("覆盖写 {overwrite_index}：{error:?}"))?;
    }
    let operations = stream.retained_operations();
    let mut pool_before_the_last_publish = empty_pool(device_bytes);
    pool_before_the_last_publish.apply(&operations[..last_publish_start]);
    let roots = log_the_roots_of_a_stream(&empty_pool(device_bytes), &operations);
    Ok(FirstInstance {
        pool_before_the_last_publish,
        operations,
        last_publish_start,
        last_root: root_key_of(&previous.root),
        roots,
    })
}

#[cfg(test)]
mod tests;

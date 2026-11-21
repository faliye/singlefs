//! Q2-2a：固定脚本上，这一份 `crates/` 每次发布按结构种类写的字节。

use super::*;

// ============================================================================
// 十四、Q2-2a（跑前登记岔路单第 2 行 ②）：固定脚本上，这一份 `crates/` 每次发布按结构种类写的字节。
//
//    口径上的收窄：`WritesByStructureKind` 的写法本身按「一次写调用 = 交给一块盘的一次写，镜像的
//    两份各算一次」计（`write_accounting.rs` 顶部注释），与登记原文「每块盘收到的写的长度之和」是
//    同一个数——这一段不必另建按 (设备, 偏移) 分类的字节计数器。「该臂减今天」由报告在两份产物之间
//    外部 diff 完成，装置本身只报各自的原始值。
// ============================================================================

fn writes_of_transaction_output(output: &TransactionOutput) -> WritesByStructureKind {
    output.writes.clone()
}

fn emit_writes_by_kind(publish_label: &str, writes: &WritesByStructureKind) {
    let total = writes.total();
    emit_result(&format!(
        "name=q2_2a_publish_writes publish={publish_label} kind=total write_calls={} written_bytes={}",
        total.write_calls, total.written_bytes
    ));
    for kind in WrittenStructureKind::IN_REPORT_ORDER {
        let value = writes.of(kind);
        if value.write_calls > 0 {
            emit_result(&format!(
                "name=q2_2a_publish_writes publish={publish_label} kind={kind:?} write_calls={} written_bytes={}",
                value.write_calls, value.written_bytes
            ));
        }
    }
}

/// Q2-2a 的固定脚本（跑前登记 5.8 表 Q2-2）：mkfs → 可写挂载 → 首个文件 → 3 次覆盖写 → 关闭 →
/// 可写挂载 → 2 次覆盖写 → 关闭 → 回退到首个文件那条根 → 1 次覆盖写。逐次发布报字节，按发布种类
/// （写行、暖机、覆盖写）分行。
pub(super) fn run_fixed_publication_byte_script(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let node = match bootstrap(geometry, 3) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!("name=q2_2a_bootstrap_failed error={error:?}"));
            return;
        }
    };
    let Some(first_file_root) = node
        .timeline
        .iter()
        .find(|root| matches!(node.content_by_root.get(root), Some(Some(_))))
        .copied()
    else {
        emit_result("name=q2_2a_bootstrap_failed reason=\"timeline 里没有带文件的一版\"");
        return;
    };

    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            emit_result(&format!(
                "name=q2_2a_second_mount_writable_failed error={error:?}"
            ));
            return;
        }
    };
    let row_publish_writes = match &mounted.output.row_publish {
        PoolVersion::WithoutFile(version) => version.writes.clone(),
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
            version.writes.clone()
        }
    };
    emit_writes_by_kind("second_mount_row_publish", &row_publish_writes);
    for (index, warm_up_publish) in mounted.output.warm_up_publishes.iter().enumerate() {
        let writes = match warm_up_publish {
            PoolVersion::WithoutFile(version) => version.writes.clone(),
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                version.writes.clone()
            }
        };
        emit_writes_by_kind(&format!("second_mount_warm_up_publish_{index}"), &writes);
    }

    let mut allocator = mounted.allocator;
    let mut current = mounted.current;
    let instance = mounted.output.instance;
    for step in 0..2u64 {
        let PoolVersion::WithFile(previous) = &current else {
            emit_result("name=q2_2a_overwrite_failed reason=\"现行版本没有文件\"");
            return;
        };
        let content = deterministic_content(1000 + step, OVERWRITE_FILE_BYTES);
        let output = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            match publish_overwrite(
                &mut writer,
                &mut allocator,
                previous,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 100 + step,
                },
                instance,
            ) {
                Ok(output) => output,
                Err(error) => {
                    emit_result(&format!("name=q2_2a_overwrite_failed error={error:?}"));
                    return;
                }
            }
        };
        emit_writes_by_kind(
            &format!("second_mount_overwrite_{step}"),
            &writes_of_transaction_output(&output),
        );
        current = PoolVersion::WithFile(std::sync::Arc::new(output));
    }

    let target = RollbackTarget {
        instance: InstanceGeneration(first_file_root.0),
        checkpoint_txg: CheckpointTxg(first_file_root.1),
    };
    let rollback_mounted = match mount_rollback(&parameters, &mut devices, target, ShadowLedger::On)
    {
        Ok(rollback_mount_output) => rollback_mount_output,
        Err(error) => {
            emit_result(&format!("name=q2_2a_mount_rollback_failed error={error:?}"));
            return;
        }
    };
    let rollback_row_publish_writes = match &rollback_mounted.output.row_publish {
        PoolVersion::WithoutFile(version) => version.writes.clone(),
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
            version.writes.clone()
        }
    };
    emit_writes_by_kind("rollback_row_publish", &rollback_row_publish_writes);
    for (index, warm_up_publish) in rollback_mounted.output.warm_up_publishes.iter().enumerate() {
        let writes = match warm_up_publish {
            PoolVersion::WithoutFile(version) => version.writes.clone(),
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                version.writes.clone()
            }
        };
        emit_writes_by_kind(&format!("rollback_warm_up_publish_{index}"), &writes);
    }

    let mut rollback_allocator = rollback_mounted.allocator;
    let PoolVersion::WithFile(previous) = &rollback_mounted.current else {
        emit_result("name=q2_2a_final_overwrite_failed reason=\"回退之后现行版本没有文件\"");
        return;
    };
    let content = deterministic_content(2000, OVERWRITE_FILE_BYTES);
    let output = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        match publish_overwrite(
            &mut writer,
            &mut rollback_allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 9999,
            },
            rollback_mounted.output.instance,
        ) {
            Ok(output) => output,
            Err(error) => {
                emit_result(&format!(
                    "name=q2_2a_final_overwrite_failed error={error:?}"
                ));
                return;
            }
        }
    };
    emit_writes_by_kind(
        "final_overwrite_after_rollback",
        &writes_of_transaction_output(&output),
    );
}

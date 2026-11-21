//! 计时命令与重新挂载、一轮文件系统负载与裸设备负载、内层行的重发与到达时间、外层相位包含内层相位的核对、本系统（singlefs）的一轮。

use super::*;

struct TimedCommandOutcome {
    elapsed_nanoseconds: u128,
    pub(super) delta: BlockLayerDelta,
}

/// 一串命令合起来计时、合起来读块层计数（md 配置的格式化 = 建阵列 + 格式化，登记第十一节）。
fn run_timed_checked(commands: &[CommandLine], stage: &'static str, device_paths: &[String]) -> Result<TimedCommandOutcome, GuestFailure> {
    let before = read_summed_block_layer_counters(device_paths)?;
    let started = Instant::now();
    for command in commands {
        run_checked(command, stage)?;
    }
    let elapsed_nanoseconds = started.elapsed().as_nanos();
    let after = read_summed_block_layer_counters(device_paths)?;
    Ok(TimedCommandOutcome {
        elapsed_nanoseconds,
        delta: after.delta_since(before),
    })
}

fn drop_page_cache() -> Result<(), GuestFailure> {
    run_checked(&CommandLine::new("/bin/sync", &[]), "sync_before_drop_caches")?;
    std::fs::write("/proc/sys/vm/drop_caches", "3").map_err(|error| GuestFailure::new("drop_caches", error.to_string()))
}

/// 登记第四节第 4、5 步：卸载、清缓存、再挂（zfs 是 export 再 import，ARC 跟着池一起走）。
fn remount(configuration: Configuration, device_paths: &[String]) -> Result<(), GuestFailure> {
    let unmount = configuration.unmount_command().expect("六家文件系统每一家都有卸载命令");
    run_checked(&unmount, "unmount")?;
    drop_page_cache()?;
    let mount = configuration.mount_command(device_paths).expect("六家文件系统每一家都有挂载命令");
    run_checked(&mount, "remount")?;
    Ok(())
}

pub(super) fn run_file_system_round(configuration: Configuration, device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let format = configuration.format_command(device_paths).expect("六家文件系统每一家都有格式化命令");
    // 登记第十一节：md 配置的格式化从建阵列开始算
    let mut preparation = Vec::new();
    if configuration.uses_software_raid() {
        preparation.push(software_raid_creation_command(device_paths));
    }
    preparation.push(format);
    let formatted = run_timed_checked(&preparation, "format", device_paths)?;
    emitter.emit(
        "format",
        &format!("elapsed_nanoseconds={} {}", formatted.elapsed_nanoseconds, formatted.delta.describe()),
    );
    if configuration.uses_software_raid() {
        check_software_raid_state("after_create", emitter)?;
    }
    if !configuration.format_also_mounts() {
        let mount = configuration.mount_command(device_paths).expect("六家文件系统每一家都有挂载命令");
        let mounted = run_timed_checked(std::slice::from_ref(&mount), "first_mount", device_paths)?;
        emitter.emit(
            "first_mount",
            &format!("elapsed_nanoseconds={} {}", mounted.elapsed_nanoseconds, mounted.delta.describe()),
        );
    }
    emit_capacity(device_paths, emitter)?;
    run_persist_workload(device_paths, emitter)?;
    run_fio_job(FioJob::WriteSequentialLarge, LARGE_FILE_PATH, device_paths, emitter)?;
    remount(configuration, device_paths)?;
    run_fio_job(FioJob::ReadSequentialLarge, LARGE_FILE_PATH, device_paths, emitter)?;
    remount(configuration, device_paths)?;
    run_fio_job(FioJob::ReadRandomFourKibibytes, LARGE_FILE_PATH, device_paths, emitter)?;
    run_control_pair(LARGE_FILE_PATH, device_paths, emitter)?;
    run_fio_job(FioJob::WriteRandomFourKibibytes, LARGE_FILE_PATH, device_paths, emitter)?;
    std::fs::create_dir_all(METADATA_DIRECTORY).map_err(|error| GuestFailure::new("metadata_directory", error.to_string()))?;
    run_fio_job(FioJob::MetadataCreate, METADATA_DIRECTORY, device_paths, emitter)?;
    run_fio_job(FioJob::MetadataFileStatus, METADATA_DIRECTORY, device_paths, emitter)?;
    run_fio_job(FioJob::MetadataDelete, METADATA_DIRECTORY, device_paths, emitter)?;
    std::fs::create_dir_all(SMALL_FILE_DIRECTORY).map_err(|error| GuestFailure::new("small_file_directory", error.to_string()))?;
    run_fio_job(FioJob::SmallFileWriteWithFsync, SMALL_FILE_DIRECTORY, device_paths, emitter)?;
    // 登记第四节第 9 步：冷挂载
    let unmount = configuration.unmount_command().expect("六家文件系统每一家都有卸载命令");
    run_checked(&unmount, "unmount_before_cold_mount")?;
    drop_page_cache()?;
    let mount = configuration.mount_command(device_paths).expect("六家文件系统每一家都有挂载命令");
    let mounted = run_timed_checked(std::slice::from_ref(&mount), "cold_mount", device_paths)?;
    emitter.emit(
        "cold_mount",
        &format!("elapsed_nanoseconds={} {}", mounted.elapsed_nanoseconds, mounted.delta.describe()),
    );
    if configuration.uses_software_raid() {
        check_software_raid_state("end_of_round", emitter)?;
    }
    Ok(())
}

/// 裸盘只跑登记第四节第 3、4、5、6、7 步，fio 直接打块设备，不重挂；镜像的裸阵列先建 md（登记第十一节）。
pub(super) fn run_raw_device_round(configuration: Configuration, device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    if configuration.uses_software_raid() {
        run_checked(&software_raid_creation_command(device_paths), "software_raid_create")?;
        check_software_raid_state("after_create", emitter)?;
    }
    let target = configuration.raw_target(device_paths).expect("裸盘配置都有 fio 直接打的块设备");
    run_fio_job(FioJob::WriteSequentialLarge, &target, device_paths, emitter)?;
    run_fio_job(FioJob::ReadSequentialLarge, &target, device_paths, emitter)?;
    run_fio_job(FioJob::ReadRandomFourKibibytes, &target, device_paths, emitter)?;
    run_control_pair(&target, device_paths, emitter)?;
    run_fio_job(FioJob::WriteRandomFourKibibytes, &target, device_paths, emitter)?;
    if configuration.uses_software_raid() {
        check_software_raid_state("end_of_round", emitter)?;
    }
    Ok(())
}

/// 子进程的结果行改名转发：`name=` 换成 `inner=`，免得子进程的 `name=done` 被 vm-bench.sh 当成本进程的收尾行。
fn reemit_inner_line(at_nanoseconds: u128, inner_body: &str) -> String {
    let without_name = inner_body.strip_prefix("name=").unwrap_or(inner_body);
    format!("at_nanoseconds={at_nanoseconds} inner={without_name}")
}

/// 子进程的一行输出，连同它从管道里读出来的那一刻（从起点算的纳秒）。
#[derive(Debug, PartialEq, Eq)]
struct ArrivedLine {
    at_nanoseconds: u128,
    text: String,
}

/// 把子进程的输出一口气读到 EOF，每行记下读出来那一刻；签名里不给任何输出句柄，读的循环里就写不出转打。
/// 为什么：虚机里转打一行走串口要阻塞几毫秒，而子进程往管道里写不被挡，边读边打时后面每行的时刻都带着前面的打印积压。
fn read_lines_with_arrival_times<Reader: BufRead>(reader: Reader, started: Instant) -> std::io::Result<Vec<ArrivedLine>> {
    let mut arrived_lines = Vec::new();
    for line in reader.lines() {
        let text = line?;
        arrived_lines.push(ArrivedLine { at_nanoseconds: started.elapsed().as_nanos(), text });
    }
    Ok(arrived_lines)
}

/// 两个到达时刻之差；缺一个就没有这一段。
fn phase_nanoseconds(later: Option<u128>, earlier: Option<u128>) -> Option<u128> {
    match (later, earlier) {
        (Some(later_value), Some(earlier_value)) => Some(later_value - earlier_value),
        (Some(_), None) | (None, Some(_)) | (None, None) => None,
    }
}

/// 外层按行到达时刻切的那一段包不包住子进程自己计的那一段（外层 ≥ 里层为真）；缺一个值就判不了。
/// 子进程先打 `transaction` 行、再开始计时、做发布 B、停表、最后才打 `file_overwrite` 行，所以两行到达的间隔物理上不会小于它自己计的数。
fn outer_phase_contains_inner_phase(outer_nanoseconds: Option<u128>, inner_nanoseconds: Option<u128>) -> Option<bool> {
    match (outer_nanoseconds, inner_nanoseconds) {
        (Some(outer_value), Some(inner_value)) => Some(outer_value >= inner_value),
        (Some(_), None) | (None, Some(_)) | (None, None) => None,
    }
}

/// `singlefs_timing` 行末尾的两个字段：子进程在 `name=file_overwrite` 行里自己计的纳秒，与外层那一段包不包住它。
fn file_overwrite_containment_fields(
    outer_nanoseconds: Option<u128>,
    file_overwrite_fields: Option<&BTreeMap<String, String>>,
) -> String {
    let inner_nanoseconds =
        file_overwrite_fields.and_then(|fields| fields.get("nanoseconds")?.parse::<u128>().ok());
    let containment = outer_phase_contains_inner_phase(outer_nanoseconds, inner_nanoseconds);
    format!(
        "file_overwrite_inner_nanoseconds={} file_overwrite_outer_contains_inner={}",
        inner_nanoseconds.map_or_else(|| "NA".to_string(), |nanoseconds| nanoseconds.to_string()),
        containment.map_or_else(|| "NA".to_string(), |contains| contains.to_string())
    )
}

/// 登记第四节 singlefs 那一段（第十二节起模式换成 `file-overwrite`）：门禁 55 号那个真设备二进制当子进程跑，
/// 按结果行到达的时刻切三段挂钟：新池新建文件的写路（mkfs + 取号 + 暖机 + 新池新建文件）、覆盖写、释放、回退与复用（覆盖写 B）、冷恢复；
/// B 交给设备的写请求、字节、屏障、FUA 从它自己报的那一行取（程序计数，门禁 55 号已证录制流与设备侧日志逐项相等）。
/// 子进程的输出先读到 EOF 再转打（`read_lines_with_arrival_times`）；外层覆盖写、释放、回退与复用那一段与子进程自己计的数比一次包含关系，一并报出。
pub(super) fn run_singlefs(device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let before = device_paths
        .iter()
        .map(|path| read_block_layer_counters(path))
        .collect::<Result<Vec<_>, _>>()?;
    let started = Instant::now();
    let mut child = Command::new(SINGLEFS_DEVICE_BINARY)
        .args(device_paths)
        .arg("file-overwrite")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| GuestFailure::new("singlefs_start", error.to_string()))?;
    let child_output = child.stdout.take().expect("刚用 Stdio::piped() 起的子进程一定有 stdout");
    let arrived_lines = read_lines_with_arrival_times(BufReader::new(child_output), started)
        .map_err(|error| GuestFailure::new("singlefs_read", error.to_string()))?;
    let status = child.wait().map_err(|error| GuestFailure::new("singlefs_wait", error.to_string()))?;
    let after = device_paths
        .iter()
        .map(|path| read_block_layer_counters(path))
        .collect::<Result<Vec<_>, _>>()?;
    let mut geometry_at = None;
    let mut transaction_at = None;
    let mut file_overwrite_at = None;
    let mut file_overwrite_fields: Option<BTreeMap<String, String>> = None;
    let mut recovery_at = None;
    let mut content_matches = false;
    for arrived_line in &arrived_lines {
        let at_nanoseconds = arrived_line.at_nanoseconds;
        let Some(inner_body) = arrived_line.text.strip_prefix("E7RESULT ") else {
            continue;
        };
        if inner_body.starts_with("name=geometry ") {
            geometry_at = Some(at_nanoseconds);
        }
        if inner_body.starts_with("name=transaction ") {
            transaction_at = Some(at_nanoseconds);
        }
        if inner_body.starts_with("name=file_overwrite ") {
            file_overwrite_at = Some(at_nanoseconds);
            file_overwrite_fields = Some(
                parse_key_values(inner_body)
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
            );
        }
        if inner_body.starts_with("name=recover_cold ") {
            recovery_at = Some(at_nanoseconds);
            content_matches = inner_body.contains("content_matches=true");
        }
        emitter.emit("singlefs_inner", &reemit_inner_line(at_nanoseconds, inner_body));
    }
    let per_device: Vec<String> = after
        .iter()
        .zip(&before)
        .enumerate()
        .map(|(index, (after_counters, before_counters))| {
            let delta = after_counters.delta_since(*before_counters);
            format!(
                "device_{index}_read_bytes={} device_{index}_read_requests={} device_{index}_write_bytes={} device_{index}_write_requests={} device_{index}_flush_requests={}",
                delta.read_bytes, delta.read_requests, delta.write_bytes, delta.write_requests, delta.flush_requests
            )
        })
        .collect();
    let phase = |later: Option<u128>, earlier: Option<u128>| {
        phase_nanoseconds(later, earlier).map_or_else(|| "NA".to_string(), |nanoseconds| nanoseconds.to_string())
    };
    // B 两盘合计的写请求、字节、屏障与 FUA：从二进制自己报的 `name=file_overwrite` 行里逐盘相加；那一行没到就写 NA。
    let second_sum = |suffix: &str| -> String {
        file_overwrite_fields
            .as_ref()
            .and_then(|fields| {
                let first = fields.get(&format!("device_0_{suffix}"))?.parse::<u64>().ok()?;
                let second = fields.get(&format!("device_1_{suffix}"))?.parse::<u64>().ok()?;
                Some((first + second).to_string())
            })
            .unwrap_or_else(|| "NA".to_string())
    };
    emitter.emit(
        "singlefs_timing",
        &format!(
            "write_path_nanoseconds={} file_overwrite_nanoseconds={} recovery_nanoseconds={} file_overwrite_writes_both_devices={} file_overwrite_written_bytes_both_devices={} file_overwrite_barriers_both_devices={} file_overwrite_force_unit_access_writes_both_devices={} child_exit={} content_matches={content_matches} {} {}",
            phase(transaction_at, geometry_at),
            phase(file_overwrite_at, transaction_at),
            phase(recovery_at, file_overwrite_at.or(transaction_at)),
            second_sum("writes"),
            second_sum("written_bytes"),
            second_sum("barriers"),
            second_sum("force_unit_access_writes"),
            status.code().map_or_else(|| "signal".to_string(), |code| code.to_string()),
            per_device.join(" "),
            file_overwrite_containment_fields(
                phase_nanoseconds(file_overwrite_at, transaction_at),
                file_overwrite_fields.as_ref()
            )
        ),
    );
    if !status.success() || !content_matches {
        return Err(GuestFailure::new(
            "singlefs_outcome",
            format!("子进程退出 {:?}，读回内容相符 {content_matches}", status.code()),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "file_system_and_raw_device_rounds_tests.rs"]
mod tests;

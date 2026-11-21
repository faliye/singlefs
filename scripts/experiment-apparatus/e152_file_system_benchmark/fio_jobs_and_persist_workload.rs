//! fio 作业的参数与运行、对照两趟、持久化负载与它的汇总、文件系统容量。

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FioJob {
    WriteSequentialLarge,
    ReadSequentialLarge,
    ReadRandomFourKibibytes,
    ControlBufferedReadFirstPass,
    ControlBufferedReadSecondPass,
    WriteRandomFourKibibytes,
    MetadataCreate,
    MetadataFileStatus,
    MetadataDelete,
    SmallFileWriteWithFsync,
}

pub(super) fn words(first: String, rest: &[&str]) -> Vec<String> {
    std::iter::once(first)
        .chain(rest.iter().map(|word| (*word).to_string()))
        .collect()
}

impl FioJob {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::WriteSequentialLarge => "write_sequential_large",
            Self::ReadSequentialLarge => "read_sequential_large",
            Self::ReadRandomFourKibibytes => "read_random_4k",
            Self::ControlBufferedReadFirstPass => "control_buffered_read_first_pass",
            Self::ControlBufferedReadSecondPass => "control_buffered_read_second_pass",
            Self::WriteRandomFourKibibytes => "write_random_4k",
            Self::MetadataCreate => "metadata_create",
            Self::MetadataFileStatus => "metadata_stat",
            Self::MetadataDelete => "metadata_delete",
            Self::SmallFileWriteWithFsync => "small_file_write_fsync",
        }
    }

    pub(super) fn check(self) -> DirectInputOutputCheck {
        match self {
            Self::WriteSequentialLarge | Self::WriteRandomFourKibibytes => DirectInputOutputCheck::DirectWrite,
            Self::ReadSequentialLarge | Self::ReadRandomFourKibibytes | Self::ControlBufferedReadSecondPass => {
                DirectInputOutputCheck::DirectRead
            }
            Self::ControlBufferedReadFirstPass
            | Self::MetadataCreate
            | Self::MetadataFileStatus
            | Self::MetadataDelete
            | Self::SmallFileWriteWithFsync => DirectInputOutputCheck::NotChecked,
        }
    }

    /// 参数逐字照登记第四节（第 8 步按第十节修订二）。
    pub(super) fn arguments(self, target: &str) -> Vec<String> {
        let file = format!("--filename={target}");
        let directory = format!("--directory={target}");
        match self {
            Self::WriteSequentialLarge => words(
                file,
                &["--rw=write", "--bs=1M", "--size=4G", "--ioengine=libaio", "--iodepth=16", "--direct=1", "--end_fsync=1"],
            ),
            Self::ReadSequentialLarge => {
                words(file, &["--rw=read", "--bs=1M", "--size=4G", "--ioengine=libaio", "--iodepth=16", "--direct=1"])
            }
            Self::ReadRandomFourKibibytes => words(
                file,
                &[
                    "--rw=randread",
                    "--bs=4k",
                    "--size=4G",
                    "--ioengine=libaio",
                    "--iodepth=32",
                    "--direct=1",
                    "--time_based",
                    "--runtime=20",
                ],
            ),
            // fio 默认 invalidate=1，每个作业开始前先丢掉文件的页缓存，第二遍就读不到缓存（登记第十节修订四）
            Self::ControlBufferedReadFirstPass | Self::ControlBufferedReadSecondPass => {
                words(file, &["--rw=read", "--bs=1M", "--size=256M", "--ioengine=psync", "--invalidate=0"])
            }
            Self::WriteRandomFourKibibytes => words(
                file,
                &[
                    "--rw=randwrite",
                    "--bs=4k",
                    "--size=4G",
                    "--ioengine=libaio",
                    "--iodepth=32",
                    "--direct=1",
                    "--time_based",
                    "--runtime=20",
                    "--end_fsync=1",
                ],
            ),
            Self::MetadataCreate => words(
                directory,
                &[
                    "--ioengine=filecreate",
                    "--nrfiles=10000",
                    "--filesize=4k",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--create_on_open=1",
                    METADATA_FILENAME_FORMAT,
                ],
            ),
            Self::MetadataFileStatus => words(
                directory,
                &[
                    "--ioengine=filestat",
                    "--nrfiles=10000",
                    "--filesize=4k",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--create_on_open=1",
                    METADATA_FILENAME_FORMAT,
                ],
            ),
            Self::MetadataDelete => words(
                directory,
                &[
                    "--ioengine=filedelete",
                    "--nrfiles=10000",
                    "--filesize=4k",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--create_on_open=1",
                    METADATA_FILENAME_FORMAT,
                ],
            ),
            Self::SmallFileWriteWithFsync => words(
                directory,
                &[
                    "--ioengine=psync",
                    "--rw=write",
                    "--bs=4k",
                    "--filesize=4k",
                    "--nrfiles=2000",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--fsync=1",
                    "--create_on_open=1",
                ],
            ),
        }
    }
}

fn optional_number(value: Option<u64>) -> String {
    value.map_or_else(|| "NA".to_string(), |number| number.to_string())
}

pub(super) fn run_fio_job(job: FioJob, target: &str, device_paths: &[String], emitter: &mut Emitter) -> Result<CellVerdict, GuestFailure> {
    let before = read_summed_block_layer_counters(device_paths)?;
    let started = Instant::now();
    let output = Command::new("/usr/bin/fio")
        .args(["--thread", "--output-format=terse", "--terse-version=3"])
        .arg(format!("--name={}", job.label()))
        .args(job.arguments(target))
        .output()
        .map_err(|error| GuestFailure::new("fio_start", format!("{}：{error}", job.label())))?;
    let elapsed_nanoseconds = started.elapsed().as_nanos();
    let after = read_summed_block_layer_counters(device_paths)?;
    let standard_output = String::from_utf8_lossy(&output.stdout);
    let report = standard_output.lines().find_map(parse_fio_terse_version_3).ok_or_else(|| {
        GuestFailure::new(
            "fio_output",
            format!(
                "{} 没有 terse 行；退出 {:?}；{}",
                job.label(),
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ),
        )
    })?;
    let delta = after.delta_since(before);
    let verdict = if report.error_code != 0 {
        CellVerdict::FioError
    } else {
        let (fio_bytes, block_bytes) = match job.check() {
            DirectInputOutputCheck::DirectRead => (report.read_kibibytes * 1024, delta.read_bytes),
            DirectInputOutputCheck::DirectWrite => {
                let write_kibibytes = report
                    .write_kibibytes
                    .ok_or_else(|| GuestFailure::new("fio_output", format!("{} 的 terse 行没有写那一侧", job.label())))?;
                (write_kibibytes * 1024, delta.write_bytes)
            }
            DirectInputOutputCheck::NotChecked => (0, 0),
        };
        judge_direct_input_output(job.check(), fio_bytes, block_bytes)
    };
    emitter.emit(
        "fio",
        &format!(
            "job={} error={} read_kibibytes={} read_bandwidth_kibibytes_per_second={} read_operations_per_second={} read_percentile_99_microseconds={} write_kibibytes={} write_bandwidth_kibibytes_per_second={} write_operations_per_second={} write_percentile_99_microseconds={} elapsed_nanoseconds={elapsed_nanoseconds} {} verdict={}",
            job.label(),
            report.error_code,
            report.read_kibibytes,
            report.read_bandwidth_kibibytes_per_second,
            report.read_operations_per_second,
            optional_number(report.read_completion_latency_percentile_99_microseconds),
            optional_number(report.write_kibibytes),
            optional_number(report.write_bandwidth_kibibytes_per_second),
            optional_number(report.write_operations_per_second),
            optional_number(report.write_completion_latency_percentile_99_microseconds),
            delta.describe(),
            verdict.label()
        ),
    );
    Ok(verdict)
}

/// 登记第六节的自证：同一段先不带 direct 读一遍，再读一遍；第二遍块层几乎不读，判定函数必须判它「被缓存顶替」。
pub(super) fn run_control_pair(target: &str, device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    // 两遍之间一直开着目标：块设备最后一个打开者关闭时内核丢掉它的页缓存，两个 fio 进程各开各关，裸盘的第二遍就读不到缓存（登记第十节修订六）
    let target_held_open =
        std::fs::File::open(target).map_err(|error| GuestFailure::new("control_hold_open", format!("{target}：{error}")))?;
    run_fio_job(FioJob::ControlBufferedReadFirstPass, target, device_paths, emitter)?;
    let second_pass = run_fio_job(FioJob::ControlBufferedReadSecondPass, target, device_paths, emitter)?;
    drop(target_held_open);
    let control_has_teeth = second_pass == CellVerdict::CacheSubstituted;
    emitter.emit(
        "control",
        &format!("second_pass_verdict={} control_has_teeth={control_has_teeth}", second_pass.label()),
    );
    Ok(())
}

struct PersistMeasurement {
    latency_nanoseconds: u128,
    pub(super) delta: BlockLayerDelta,
}

#[derive(Debug, PartialEq, Eq)]
struct PersistSummary {
    operations: usize,
    median_latency_nanoseconds: u128,
    percentile_99_latency_nanoseconds: u128,
    mean_write_requests_thousandths: u64,
    mean_write_bytes: u64,
    mean_flush_requests_thousandths: u64,
    first_latency_nanoseconds: u128,
    first_write_requests: u64,
    first_write_bytes: u64,
    first_flush_requests: u64,
}

/// 最近秩分位：秩 = ⌈p × n ÷ 100⌉，至少 1。
fn nearest_rank_percentile(sorted_values: &[u128], percentile: u64) -> u128 {
    assert!(!sorted_values.is_empty(), "空样本没有分位数");
    let count = u64::try_from(sorted_values.len()).expect("样本数装得进 u64");
    let rank = (percentile * count).div_ceil(100).max(1);
    sorted_values[usize::try_from(rank - 1).expect("名次装得进 usize")]
}

fn summarize_persist(measurements: &[PersistMeasurement]) -> PersistSummary {
    assert!(!measurements.is_empty(), "一次都没量就没有汇总");
    let mut latencies: Vec<u128> = measurements.iter().map(|measurement| measurement.latency_nanoseconds).collect();
    latencies.sort_unstable();
    let count = u64::try_from(measurements.len()).expect("样本数装得进 u64");
    let total = |pick: fn(&BlockLayerDelta) -> u64| -> u64 {
        measurements.iter().map(|measurement| pick(&measurement.delta)).sum()
    };
    let first = &measurements[0];
    PersistSummary {
        operations: measurements.len(),
        median_latency_nanoseconds: nearest_rank_percentile(&latencies, 50),
        percentile_99_latency_nanoseconds: nearest_rank_percentile(&latencies, 99),
        mean_write_requests_thousandths: total(|delta| delta.write_requests) * 1000 / count,
        mean_write_bytes: total(|delta| delta.write_bytes) / count,
        mean_flush_requests_thousandths: total(|delta| delta.flush_requests) * 1000 / count,
        first_latency_nanoseconds: first.latency_nanoseconds,
        first_write_requests: first.delta.write_requests,
        first_write_bytes: first.delta.write_bytes,
        first_flush_requests: first.delta.flush_requests,
    }
}

impl PersistSummary {
    pub(super) fn describe(&self) -> String {
        format!(
            "operations={} file_bytes={PERSIST_FILE_BYTES} median_latency_nanoseconds={} percentile_99_latency_nanoseconds={} mean_write_requests_thousandths={} mean_write_bytes={} mean_flush_requests_thousandths={} first_latency_nanoseconds={} first_write_requests={} first_write_bytes={} first_flush_requests={}",
            self.operations,
            self.median_latency_nanoseconds,
            self.percentile_99_latency_nanoseconds,
            self.mean_write_requests_thousandths,
            self.mean_write_bytes,
            self.mean_flush_requests_thousandths,
            self.first_latency_nanoseconds,
            self.first_write_requests,
            self.first_write_bytes,
            self.first_flush_requests
        )
    }
}

fn persist_file_content() -> Vec<u8> {
    (0..PERSIST_FILE_BYTES)
        .map(|position| u8::try_from(position % 251).expect("对 251 取余一定小于 256"))
        .collect()
}

/// 登记第四节第 2 步：刚格式化的空文件系统上，每次「新建、写 3000 字节、fsync 文件、fsync 目录」单独计时、单独读块层计数。
pub(super) fn run_persist_workload(device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let failure = |stage: &'static str| move |error: std::io::Error| GuestFailure::new(stage, error.to_string());
    std::fs::create_dir_all(PERSIST_DIRECTORY).map_err(failure("persist_directory"))?;
    run_checked(&CommandLine::new("/bin/sync", &[]), "sync_before_persist")?;
    let directory_handle = std::fs::File::open(PERSIST_DIRECTORY).map_err(failure("persist_directory_open"))?;
    let content = persist_file_content();
    let mut measurements = Vec::with_capacity(PERSIST_FILE_COUNT);
    for index in 0..PERSIST_FILE_COUNT {
        let path = Path::new(PERSIST_DIRECTORY).join(format!("file_{index:03}"));
        let before = read_summed_block_layer_counters(device_paths)?;
        let started = Instant::now();
        {
            let mut file = std::fs::File::create(&path).map_err(failure("persist_create"))?;
            file.write_all(&content).map_err(failure("persist_write"))?;
            file.sync_all().map_err(failure("persist_fsync_file"))?;
        }
        directory_handle.sync_all().map_err(failure("persist_fsync_directory"))?;
        let latency_nanoseconds = started.elapsed().as_nanos();
        let after = read_summed_block_layer_counters(device_paths)?;
        measurements.push(PersistMeasurement {
            latency_nanoseconds,
            delta: after.delta_since(before),
        });
    }
    emitter.emit("persist", &summarize_persist(&measurements).describe());
    Ok(())
}

/// busybox `stat -f -c '%b %a %S'`：总块数、非特权可用块数、块大小（f_frsize）。
fn parse_file_system_capacity(text: &str) -> Option<(u64, u64)> {
    let numbers = text
        .split_whitespace()
        .map(|word| word.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()?;
    match numbers.as_slice() {
        [total_blocks, available_blocks, block_bytes] => Some((total_blocks * block_bytes, available_blocks * block_bytes)),
        _ => None,
    }
}

pub(super) fn emit_capacity(device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let output = run_checked(&CommandLine::new("/bin/stat", &["-f", "-c", "%b %a %S", MOUNT_POINT]), "statfs")?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (total_bytes, available_bytes) = parse_file_system_capacity(&text)
        .ok_or_else(|| GuestFailure::new("statfs", format!("解析不出三个数：{text}")))?;
    // 登记第十一节：分母是所有成员盘的总字节，镜像配置与 singlefs 的「两盘镜像后是总量的 47.6%」同口径
    let mut disk_bytes = 0;
    for device_path in device_paths {
        disk_bytes += read_device_bytes(device_path)?;
    }
    emitter.emit(
        "capacity",
        &format!("total_bytes={total_bytes} available_bytes={available_bytes} disk_bytes={disk_bytes}"),
    );
    Ok(())
}

#[cfg(test)]
#[path = "fio_jobs_and_persist_workload_tests.rs"]
mod tests;

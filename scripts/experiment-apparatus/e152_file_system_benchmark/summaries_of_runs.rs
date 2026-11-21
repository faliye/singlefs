//! 宿主侧的汇总：产物里键值与轮次块的解析、每个指标的读数、轮统计与整个产品的汇总。

use super::*;

// ── 宿主一侧：把多轮产物汇成中位、最小、最大与离散（登记第七节） ──

pub(super) fn parse_key_values(body: &str) -> BTreeMap<&str, &str> {
    body.split_whitespace().filter_map(|field| field.split_once('=')).collect()
}

struct RunBlock {
    pub(super) configuration: String,
    pub(super) round: u32,
    virtual_machine_exit: i64,
    result_bodies: Vec<String>,
}

/// 产物由 e152-run.sh 拼成：每一次虚机跑先一行 `E152RUN`，后面跟这一次抓到的全部 `E7RESULT` 行。
fn parse_run_blocks(text: &str) -> Vec<RunBlock> {
    let mut blocks: Vec<RunBlock> = Vec::new();
    for line in text.lines() {
        if let Some(header) = line.strip_prefix("E152RUN ") {
            let fields = parse_key_values(header);
            blocks.push(RunBlock {
                configuration: fields.get("configuration").copied().unwrap_or("unknown").to_string(),
                round: fields.get("round").and_then(|value| value.parse().ok()).unwrap_or(0),
                virtual_machine_exit: fields.get("vm_exit").and_then(|value| value.parse().ok()).unwrap_or(-1),
                result_bodies: Vec::new(),
            });
        } else if let Some(body) = line.strip_prefix("E7RESULT ") {
            if let Some(current) = blocks.last_mut() {
                current.result_bodies.push(body.to_string());
            }
        }
    }
    blocks
}

struct MetricReading {
    metric: &'static str,
    value: Result<f64, String>,
    is_read_cell: bool,
}

fn metric_reading(metric: &'static str, value: Result<f64, String>, is_read_cell: bool) -> MetricReading {
    MetricReading { metric, value, is_read_cell }
}

fn number_field(fields: &BTreeMap<&str, &str>, key: &str) -> Result<f64, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing_{key}"))?
        .parse::<f64>()
        .map_err(|_| format!("not_a_number_{key}"))
}

fn fio_metric_readings(fields: &BTreeMap<&str, &str>) -> Vec<MetricReading> {
    let verdict = fields.get("verdict").copied().unwrap_or("missing_verdict");
    let usable = |value: Result<f64, String>| -> Result<f64, String> {
        if verdict == "counted" || verdict == "not_checked" {
            value
        } else {
            Err(verdict.to_string())
        }
    };
    let mebibytes = |key: &str| number_field(fields, key).map(|kibibytes| kibibytes / 1024.0);
    match fields.get("job").copied() {
        Some("write_sequential_large") => vec![metric_reading(
            "write_sequential_mebibytes_per_second",
            usable(mebibytes("write_bandwidth_kibibytes_per_second")),
            false,
        )],
        Some("read_sequential_large") => vec![metric_reading(
            "read_sequential_mebibytes_per_second",
            usable(mebibytes("read_bandwidth_kibibytes_per_second")),
            true,
        )],
        Some("read_random_4k") => vec![
            metric_reading(
                "read_random_4k_operations_per_second",
                usable(number_field(fields, "read_operations_per_second")),
                true,
            ),
            metric_reading(
                "read_random_4k_percentile_99_microseconds",
                usable(number_field(fields, "read_percentile_99_microseconds")),
                true,
            ),
        ],
        Some("write_random_4k") => vec![
            metric_reading(
                "write_random_4k_operations_per_second",
                usable(number_field(fields, "write_operations_per_second")),
                false,
            ),
            metric_reading(
                "write_random_4k_percentile_99_microseconds",
                usable(number_field(fields, "write_percentile_99_microseconds")),
                false,
            ),
        ],
        Some("metadata_create") => vec![metric_reading(
            "metadata_create_per_second",
            usable(number_field(fields, "read_operations_per_second")),
            false,
        )],
        Some("metadata_stat") => vec![metric_reading(
            "metadata_stat_per_second",
            usable(number_field(fields, "read_operations_per_second")),
            false,
        )],
        Some("metadata_delete") => vec![metric_reading(
            "metadata_delete_per_second",
            usable(number_field(fields, "read_operations_per_second")),
            false,
        )],
        Some("small_file_write_fsync") => vec![metric_reading(
            "small_file_write_fsync_per_second",
            usable(number_field(fields, "write_operations_per_second")),
            false,
        )],
        // 对照两遍不是指标；不认识的作业名不猜
        _ => Vec::new(),
    }
}

fn divided(numerator: Result<f64, String>, denominator: Result<f64, String>, scale: f64) -> Result<f64, String> {
    Ok(numerator? / denominator? * scale)
}

/// 覆盖写、释放、回退与复用的外层挂钟只收包含检查判真的轮：判假、判不了（NA）、这一行没有这个字段，都照排除的格子报出理由，不进中位。
fn file_overwrite_wall_clock_if_contained(
    outer_contains_inner: Option<&str>,
    wall_clock_milliseconds: Result<f64, String>,
) -> Result<f64, String> {
    match outer_contains_inner {
        Some("true") => wall_clock_milliseconds,
        Some("false") => Err("outer_does_not_contain_inner".to_string()),
        Some("NA") => Err("containment_not_available".to_string()),
        Some(_) => Err("not_a_boolean_file_overwrite_outer_contains_inner".to_string()),
        None => Err("missing_file_overwrite_outer_contains_inner".to_string()),
    }
}

fn metric_readings(fields: &BTreeMap<&str, &str>) -> Vec<MetricReading> {
    let nanoseconds_as = |key: &str, scale: f64| number_field(fields, key).map(|nanoseconds| nanoseconds / scale);
    match fields.get("name").copied() {
        Some("fio") => fio_metric_readings(fields),
        Some("persist") => vec![
            metric_reading("persist_median_microseconds", nanoseconds_as("median_latency_nanoseconds", 1e3), false),
            metric_reading("persist_percentile_99_microseconds", nanoseconds_as("percentile_99_latency_nanoseconds", 1e3), false),
            metric_reading(
                "persist_mean_write_requests",
                number_field(fields, "mean_write_requests_thousandths").map(|thousandths| thousandths / 1e3),
                false,
            ),
            metric_reading("persist_mean_write_bytes", number_field(fields, "mean_write_bytes"), false),
            metric_reading(
                "persist_mean_flush_requests",
                number_field(fields, "mean_flush_requests_thousandths").map(|thousandths| thousandths / 1e3),
                false,
            ),
            metric_reading("persist_first_write_requests", number_field(fields, "first_write_requests"), false),
            metric_reading("persist_first_write_bytes", number_field(fields, "first_write_bytes"), false),
            metric_reading("persist_first_flush_requests", number_field(fields, "first_flush_requests"), false),
        ],
        Some("cold_mount") => vec![
            metric_reading("cold_mount_milliseconds", nanoseconds_as("elapsed_nanoseconds", 1e6), false),
            metric_reading("cold_mount_read_bytes", number_field(fields, "block_read_bytes"), false),
            metric_reading("cold_mount_read_requests", number_field(fields, "block_read_requests"), false),
        ],
        Some("format") => vec![
            metric_reading("format_milliseconds", nanoseconds_as("elapsed_nanoseconds", 1e6), false),
            metric_reading("format_write_bytes", number_field(fields, "block_write_bytes"), false),
            metric_reading("format_flush_requests", number_field(fields, "block_flush_requests"), false),
        ],
        Some("capacity") => vec![
            metric_reading(
                "capacity_total_percent",
                divided(number_field(fields, "total_bytes"), number_field(fields, "disk_bytes"), 100.0),
                false,
            ),
            metric_reading(
                "capacity_available_percent",
                divided(number_field(fields, "available_bytes"), number_field(fields, "disk_bytes"), 100.0),
                false,
            ),
        ],
        Some("singlefs_timing") => vec![
            metric_reading("singlefs_write_path_milliseconds", nanoseconds_as("write_path_nanoseconds", 1e6), false),
            metric_reading(
                "singlefs_file_overwrite_milliseconds",
                file_overwrite_wall_clock_if_contained(
                    fields.get("file_overwrite_outer_contains_inner").copied(),
                    nanoseconds_as("file_overwrite_nanoseconds", 1e6),
                ),
                false,
            ),
            metric_reading(
                "singlefs_file_overwrite_inner_milliseconds",
                nanoseconds_as("file_overwrite_inner_nanoseconds", 1e6),
                false,
            ),
            metric_reading("singlefs_recovery_milliseconds", nanoseconds_as("recovery_nanoseconds", 1e6), false),
            metric_reading("singlefs_file_overwrite_writes_both_devices", number_field(fields, "file_overwrite_writes_both_devices"), false),
            metric_reading("singlefs_file_overwrite_written_bytes_both_devices", number_field(fields, "file_overwrite_written_bytes_both_devices"), false),
            metric_reading("singlefs_file_overwrite_barriers_both_devices", number_field(fields, "file_overwrite_barriers_both_devices"), false),
            metric_reading("singlefs_file_overwrite_force_unit_access_writes_both_devices", number_field(fields, "file_overwrite_force_unit_access_writes_both_devices"), false),
            metric_reading(
                "singlefs_read_bytes_both_devices",
                number_field(fields, "device_0_read_bytes")
                    .and_then(|first| number_field(fields, "device_1_read_bytes").map(|second| first + second)),
                false,
            ),
            metric_reading(
                "singlefs_read_requests_both_devices",
                number_field(fields, "device_0_read_requests")
                    .and_then(|first| number_field(fields, "device_1_read_requests").map(|second| first + second)),
                false,
            ),
        ],
        _ => Vec::new(),
    }
}

struct RoundStatistics {
    median: f64,
    minimum: f64,
    maximum: f64,
    spread_percent: f64,
    is_stable: bool,
}

/// 登记第七节：中位、最小、最大，离散 = (最大 − 最小) ÷ 中位。
fn round_statistics(values: &[f64]) -> RoundStatistics {
    assert!(!values.is_empty(), "空样本没有中位");
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let count = sorted.len();
    let median = if count % 2 == 1 {
        sorted[count / 2]
    } else {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    };
    let minimum = sorted[0];
    let maximum = sorted[count - 1];
    let spread_percent = if median == 0.0 {
        if maximum == minimum {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        (maximum - minimum) * 100.0 / median
    };
    RoundStatistics {
        median,
        minimum,
        maximum,
        spread_percent,
        is_stable: spread_percent <= STABLE_SPREAD_PERCENT,
    }
}

fn summarize_product(text: &str) -> Vec<String> {
    let blocks = parse_run_blocks(text);
    let mut attempts: BTreeMap<(String, u32), u32> = BTreeMap::new();
    let mut selected: BTreeMap<(String, u32), &RunBlock> = BTreeMap::new();
    for block in &blocks {
        let key = (block.configuration.clone(), block.round);
        *attempts.entry(key.clone()).or_insert(0) += 1;
        if block.virtual_machine_exit == 0 {
            selected.entry(key).or_insert(block);
        }
    }
    let mut values: BTreeMap<(String, &'static str), Vec<(u32, f64)>> = BTreeMap::new();
    let mut controls: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut lines = Vec::new();
    for ((configuration, round), attempt_count) in &attempts {
        let Some(block) = selected.get(&(configuration.clone(), *round)) else {
            lines.push(format!(
                "name=summary_failed_round configuration={configuration} round={round} attempts={attempt_count}"
            ));
            continue;
        };
        let parsed_bodies: Vec<BTreeMap<&str, &str>> =
            block.result_bodies.iter().map(|body| parse_key_values(body)).collect();
        let control_has_teeth = parsed_bodies
            .iter()
            .find(|fields| fields.get("name") == Some(&"control"))
            .map(|fields| fields.get("control_has_teeth") == Some(&"true"));
        if let Some(has_teeth) = control_has_teeth {
            let tally = controls.entry(configuration.clone()).or_insert((0, 0));
            tally.0 += 1;
            if has_teeth {
                tally.1 += 1;
            }
        }
        for fields in &parsed_bodies {
            for reading in metric_readings(fields) {
                // 登记第八节第 3 条：对照没判成「被缓存顶替」，这一轮的读格作废
                let value = if reading.is_read_cell && control_has_teeth == Some(false) {
                    Err("control_without_teeth".to_string())
                } else {
                    reading.value
                };
                match value {
                    Ok(number) => values
                        .entry((configuration.clone(), reading.metric))
                        .or_default()
                        .push((*round, number)),
                    Err(reason) => lines.push(format!(
                        "name=summary_excluded configuration={configuration} metric={} round={round} reason={reason}",
                        reading.metric
                    )),
                }
            }
        }
    }
    for (configuration, (rounds_checked, rounds_with_teeth)) in &controls {
        lines.push(format!(
            "name=summary_control configuration={configuration} rounds_checked={rounds_checked} rounds_with_teeth={rounds_with_teeth}"
        ));
    }
    for ((configuration, metric), readings) in &values {
        let numbers: Vec<f64> = readings.iter().map(|(_, number)| *number).collect();
        let statistics = round_statistics(&numbers);
        let listed = readings
            .iter()
            .map(|(round, number)| format!("{round}:{number:.3}"))
            .collect::<Vec<_>>()
            .join(",");
        lines.push(format!(
            "name=summary configuration={configuration} metric={metric} rounds={} median={:.3} minimum={:.3} maximum={:.3} spread_percent={:.1} stability={} values={listed}",
            numbers.len(),
            statistics.median,
            statistics.minimum,
            statistics.maximum,
            statistics.spread_percent,
            if statistics.is_stable { "stable" } else { "unstable" }
        ));
    }
    let mut output: Vec<String> = lines.into_iter().map(|line| format!("E7RESULT {line}")).collect();
    let emitted = output.len() + 1;
    output.push(format!("E7RESULT name=summary_done emitted={emitted}"));
    output
}

pub(super) fn run_summarize(arguments: &[String]) -> ExitCode {
    let Some(product_path) = arguments.first() else {
        eprintln!("用法：e152-file-system-benchmark summarize <产物>");
        return ExitCode::from(2);
    };
    let text = match std::fs::read_to_string(product_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("读不了 {product_path}：{error}");
            return ExitCode::from(2);
        }
    };
    for line in summarize_product(&text) {
        println!("{line}");
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
#[path = "summaries_of_runs_tests.rs"]
mod tests;

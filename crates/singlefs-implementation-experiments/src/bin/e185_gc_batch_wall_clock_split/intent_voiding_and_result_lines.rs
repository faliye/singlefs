use super::*;

// ───────────────────────────── 一条意图的作废、停机与产物行 ─────────────────────────────

/// 一条意图的读数要不要作废（登记第十一节作废条款里「那一轮作废、重跑」那几条）；交回第一条原因。
pub(super) fn invalid_reason_of(reading: &IntentReading) -> Option<&'static str> {
    if reading.mount_failure.is_some() {
        return Some("mount_failed");
    }
    if reading.calls.iter().any(|call| call.is_the_cpu_unreadable) {
        return Some("thread_cpu_clock_unreadable");
    }
    if reading
        .calls
        .iter()
        .any(|call| call.inclusion == InclusionVerdict::UnaccountedAboveTenPercent)
    {
        return Some(UNACCOUNTED_INVALID_REASON);
    }
    if reading.calls.iter().any(|call| {
        call.scan_read_bytes_in_the_region == 0
            && call
                .expected_scan_read_bytes
                .is_some_and(|expected| expected > 0)
    }) {
        return Some("no_scan_read_while_the_region_holds_live_units");
    }
    None
}

/// 停机条款第 2、3、5、6 条在一条意图上的判定（第 1 条在开跑前、第 4 条在第二段）：交回触发了的那几条，每条一句。
/// `data_units` 是几何造成了时 R 里的数据单元数 m（没造成交 `None`：第 3 条的 ⌈m ÷ K⌉ 与第 5 条只在几何造成了的点上判）。
pub(super) fn stop_findings_of(
    reading: &IntentReading,
    data_units: Option<u64>,
    batch_units: u64,
) -> Vec<String> {
    let mut findings = Vec::new();
    for (call_index, call) in reading.calls.iter().enumerate() {
        if call.inclusion == InclusionVerdict::CpuPlusWaitAboveTheWall {
            findings.push(format!(
                "clause_6_cpu_plus_wait_above_the_wall_at_call_{call_index}:wall={}:cpu={}:wait={}",
                call.wall_nanoseconds,
                call.cpu_nanoseconds,
                call.wait_nanoseconds()
            ));
        }
        if let Some(expected) = call.expected_scan_read_bytes {
            if call.scan_read_bytes_in_the_region != expected {
                findings.push(format!(
                    "clause_2_scan_read_bytes_at_call_{call_index}:read={}:records={expected}",
                    call.scan_read_bytes_in_the_region
                ));
            }
        }
    }
    let batches_counted = u64::try_from(
        reading
            .calls
            .iter()
            .filter(|call| counts_as_a_batch(call.kind))
            .count(),
    )
    .expect("批数");
    let batches_observed: u64 = reading.calls.iter().map(|call| call.batches_observed).sum();
    if batches_counted != batches_observed {
        findings.push(format!(
            "clause_3_batches_counted_by_the_device_{batches_counted}_observed_by_the_session_{batches_observed}"
        ));
    }
    if let Some(data_units) = data_units {
        let expected_batches = batches_of_one_intent(data_units, batch_units);
        if batches_counted != expected_batches {
            findings.push(format!(
                "clause_3_batches_{batches_counted}_not_the_ceiling_of_m_over_k_{expected_batches}"
            ));
        }
        if let Some(first_batch) = reading
            .calls
            .iter()
            .find(|call| counts_as_a_batch(call.kind))
        {
            let expected_copy = first_batch_copy_bytes(data_units, batch_units);
            if first_batch.relocation_copy_bytes != expected_copy
                || !first_batch.is_every_destination_written_whole
            {
                findings.push(format!(
                    "clause_5_first_batch_copy_bytes_{}_expected_{expected_copy}_destinations_whole_{}",
                    first_batch.relocation_copy_bytes, first_batch.is_every_destination_written_whole
                ));
            }
        }
    }
    findings
}

/// 一次测量是在哪儿、怎么量的（产物行的前缀）。
#[derive(Clone, Copy, Debug)]
pub(super) struct MeasurementLabel {
    pub(super) arm: DeviceArm,
    pub(super) point: SamplingPoint,
    pub(super) batch_units: u64,
    pub(super) round: usize,
    pub(super) attempt: usize,
    pub(super) injection: WriteInjection,
    pub(super) choice: ChoiceOfTheRegion,
}

impl MeasurementLabel {
    pub(super) fn text(&self) -> String {
        format!(
            "arm={} {} batch_units={} round={} attempt={} injection={} choice={}",
            self.arm.label(),
            self.point.label(),
            self.batch_units,
            self.round,
            self.attempt,
            self.injection.label(),
            self.choice.label()
        )
    }
}

/// 一条意图的产物行：每次调用一行 `name=call`（含各种设备调用的累计），一行 `name=intent`。
pub(super) fn emit_the_intent(
    emitter: &mut Emitter,
    label: &MeasurementLabel,
    reading: &IntentReading,
    invalid_reason: Option<&str>,
) {
    let region_text = reading.region.map_or_else(
        || "none".to_string(),
        |region| region.start_slot().0.to_string(),
    );
    for (call_index, call) in reading.calls.iter().enumerate() {
        let methods: String = DeviceMethod::ALL
            .iter()
            .map(|method| {
                let totals = call.per_method.get(method).copied().unwrap_or_default();
                format!(
                    " {name}_calls={} {name}_bytes={} {name}_wall_nanoseconds={} {name}_cpu_nanoseconds={} {name}_wait_nanoseconds={}",
                    totals.calls,
                    totals.bytes,
                    totals.wall_nanoseconds,
                    totals.cpu_nanoseconds,
                    totals.wait_nanoseconds,
                    name = method.label()
                )
            })
            .collect();
        emitter.line(&format!(
            "name=call {} call_index={call_index} kind={} region_start_slot={region_text} wall_nanoseconds={} cpu_nanoseconds={} wait_nanoseconds={} device_wall_nanoseconds={} device_cpu_nanoseconds={} scan_read_bytes_in_the_region={} expected_scan_read_bytes={} scan_read_bytes_elsewhere={} release_check_read_bytes={} read_bytes_after_the_first_write={} write_bytes={} relocation_copy_bytes={} metadata_write_bytes={} destinations_written_whole={} relocated_units={} batches_observed={} inclusion={} cpu_unreadable={} outcome={}{methods}",
            label.text(),
            call.kind.label(),
            call.wall_nanoseconds,
            call.cpu_nanoseconds,
            call.wait_nanoseconds(),
            call.device_totals.wall_nanoseconds,
            call.device_totals.cpu_nanoseconds,
            call.scan_read_bytes_in_the_region,
            call.expected_scan_read_bytes
                .map_or_else(|| "none".to_string(), |expected| expected.to_string()),
            call.scan_read_bytes_elsewhere,
            call.release_check_read_bytes,
            call.read_bytes_after_the_first_write,
            call.write_bytes,
            call.relocation_copy_bytes,
            call.write_bytes - call.relocation_copy_bytes.min(call.write_bytes),
            call.is_every_destination_written_whole,
            call.relocated_units,
            call.batches_observed,
            call.inclusion.label(),
            call.is_the_cpu_unreadable,
            call.outcome,
        ));
    }
    emitter.line(&format!(
        "name=intent {} region_start_slot={region_text} calls={} batches={} valid={} invalid_reason={} mount_failure={}",
        label.text(),
        reading.calls.len(),
        reading.calls.iter().filter(|call| counts_as_a_batch(call.kind)).count(),
        invalid_reason.is_none(),
        invalid_reason.unwrap_or("none"),
        reading.mount_failure.as_deref().unwrap_or("none"),
    ));
}

#[cfg(test)]
mod tests;

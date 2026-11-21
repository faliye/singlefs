//! 回应占的待处理额度的两格：`cell_delete_prefix_response_within_pending_budget`（删前缀的回应先记进额度）、`cell_worker_responses_within_pending_budget`（工作线程边编回应边记额度）。

use super::*;

/// 删前缀回应那一格的数：每条连接额度 64 KiB、前缀下一把 1 MiB 的长键、一条连接上一次排 8 帧删前缀；
/// 组提交等满 8 帧（至多 2 秒）再做，8 帧都进了准入才轮到头一帧，头一帧做的时候额度里还记着别的 7 帧。
const BUDGETED_DELETE_PER_CONNECTION_BYTES: usize = 64 * 1024;
const BUDGETED_DELETE_LONG_KEY_BYTES: usize = 1024 * 1024;
const BUDGETED_DELETE_PIPELINED_FRAMES: u64 = 8;
const BUDGETED_DELETE_GROUP_WINDOW_MILLISECONDS: u64 = 2_000;
/// 每帧删前缀的请求与忙的回应（服务自己写的短句）在额度里留的余量。
const BUDGETED_DELETE_SLACK_BYTES_PER_FRAME: usize = 1024;
/// 删前缀那一格的前缀。
const BUDGETED_DELETE_PREFIX: &[u8] = b"budgeted-delete/";

/// 从一条裸连接上读一帧回应、解开。
fn read_raw_response(stream: &mut UnixStream) -> Result<crate::protocol::Response, String> {
    let mut length_field = [0_u8; FRAME_LENGTH_FIELD_BYTES];
    stream
        .read_exact(&mut length_field)
        .map_err(|error| format!("读不到回应的长度字段：{error}"))?;
    let body_bytes = usize::try_from(u32::from_le_bytes(length_field))
        .map_err(|_| "回应帧体长度放不进 usize".to_string())?;
    let mut body = vec![0_u8; body_bytes];
    stream
        .read_exact(&mut body)
        .map_err(|error| format!("读不全 {body_bytes} 字节的回应帧体：{error}"))?;
    decode_response_body(&body)
}

/// 一次写出一串删前缀帧（`count_only` 逐帧给，序号从 `first_sequence` 起），再按序号读回全部回应；回（序号，结局）按序号排好。
fn delete_prefix_frames(
    stream: &mut UnixStream,
    first_sequence: u64,
    count_only: &[bool],
) -> Result<Vec<(u64, ResponseOutcome)>, String> {
    let mut frames = Vec::new();
    for (sequence, &is_count_only) in (first_sequence..).zip(count_only) {
        frames.extend(encode_delete_prefix_frame(
            sequence,
            BUDGETED_DELETE_PREFIX,
            b"",
            1,
            is_count_only,
        ));
    }
    stream
        .write_all(&frames)
        .map_err(|error| format!("一次写不出 {} 帧删前缀：{error}", count_only.len()))?;
    let mut answers = Vec::with_capacity(count_only.len());
    for _ in count_only {
        let response = read_raw_response(stream)?;
        answers.push((response.sequence, response.outcome));
    }
    answers.sort_by_key(|(sequence, _)| *sequence);
    Ok(answers)
}

/// 删前缀的回应先记进额度、记不进整帧回忙一把都不删（`committer.rs` 文件头「删前缀」、`server.rs` 文件头「额度」）。
pub(crate) fn cell_delete_prefix_response_within_pending_budget(
    context: &CellContext,
) -> Result<String, String> {
    let per_connection = BUDGETED_DELETE_PER_CONNECTION_BYTES.to_string();
    let group_frames = BUDGETED_DELETE_PIPELINED_FRAMES.to_string();
    let group_window = BUDGETED_DELETE_GROUP_WINDOW_MILLISECONDS.to_string();
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::DeletePrefixResponseUnbudgeted),
        &[
            "--maximum-pending-bytes-per-connection",
            &per_connection,
            "--group-commit-frames",
            &group_frames,
            "--group-commit-milliseconds",
            &group_window,
        ],
    )?;
    let mut long_key = BUDGETED_DELETE_PREFIX.to_vec();
    long_key.resize(BUDGETED_DELETE_LONG_KEY_BYTES, b'k');
    let mut other = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    other.write(&[(Table::Conclusions, long_key.clone(), bytes("kept"))])?;
    let mut pipelining = connect_raw(&server.socket_path())?;
    pipelining
        .set_read_timeout(Some(Duration::from_secs(60)))
        .map_err(|error| format!("设不上读超时：{error}"))?;
    let frame_count = usize::try_from(BUDGETED_DELETE_PIPELINED_FRAMES).expect("8 帧");
    // 只数不删 8 帧：至多一帧的回应记得进，别的回忙
    let counting = delete_prefix_frames(&mut pipelining, 1, &vec![true; frame_count])?;
    let mut counted_frames = 0_usize;
    let mut busy_frames = 0_usize;
    for (sequence, outcome) in &counting {
        match outcome {
            ResponseOutcome::Busy { .. } => busy_frames += 1,
            ResponseOutcome::PrefixDeleted {
                keys_in_frame: 1,
                last_key,
                ..
            } if *last_key == long_key => counted_frames += 1,
            other_outcome @ (ResponseOutcome::PointValues(_)
            | ResponseOutcome::WriteAcknowledged { .. }
            | ResponseOutcome::ScanEntries { .. }
            | ResponseOutcome::Statistics(_)
            | ResponseOutcome::Claims(_)
            | ResponseOutcome::ClaimsReleased(_)
            | ResponseOutcome::ClaimListing { .. }
            | ResponseOutcome::PrefixDeleted { .. }
            | ResponseOutcome::ConnectedClients(_)
            | ResponseOutcome::ServerStopping { .. }
            | ResponseOutcome::Rejected { .. }
            | ResponseOutcome::GuardRefused { .. }) => {
                return Err(format!(
                    "只数不删的序号 {sequence} 回 {other_outcome:?}，该是忙或数到 1 把、最后一把是那把长键"
                ))
            }
        }
    }
    if busy_frames == 0 {
        return Err(format!(
            "一条连接上同时排 {frame_count} 帧只数不删、每帧回应带一把 {BUDGETED_DELETE_LONG_KEY_BYTES} 字节的长键、每条连接额度 \
             {BUDGETED_DELETE_PER_CONNECTION_BYTES} 字节：{counted_frames} 帧都回了长键、一帧忙都没有：回应不看额度、强行记下了"
        ));
    }
    // 删的那一帧排在头一个：做它的时候额度里还记着别的 7 帧，记不进、回忙、一把都不删
    let mut deleting_pattern = vec![true; frame_count];
    deleting_pattern[0] = false;
    let deleting = delete_prefix_frames(
        &mut pipelining,
        BUDGETED_DELETE_PIPELINED_FRAMES + 1,
        &deleting_pattern,
    )?;
    let first_answer = deleting.first().map(|(_, outcome)| outcome);
    if !matches!(first_answer, Some(ResponseOutcome::Busy { .. })) {
        return Err(format!(
            "排在头一个的删前缀（额度里还记着别的 7 帧）回 {first_answer:?}，该是忙：回应记不进额度时不许删"
        ));
    }
    let still_there = other.read(&[(Table::Conclusions, long_key.clone())])?;
    if still_there.first().cloned().flatten().as_deref() != Some(b"kept".as_slice()) {
        return Err("删前缀回了忙，那把长键却从库里没了：记不进额度的那一帧照样删了".to_string());
    }
    other
        .write(&[entry("budgeted-delete-other/after", "served")])
        .map_err(|message| format!("删前缀回忙之后别的连接写不进：{message}"))?;
    let served = read_value(&server.route(), "budgeted-delete-other/after")?;
    if served.as_deref() != Some(b"served".as_slice()) {
        return Err(format!(
            "删前缀回忙之后别的连接读回 {}（该是 served）",
            shown_value(served.as_deref())
        ));
    }
    let pending_peak = counter(&other.statistics()?, PENDING_BYTES_PEAK_COUNTER)?;
    let long_response_bytes = BUDGETED_DELETE_LONG_KEY_BYTES + 64;
    let pending_bound = u64::try_from(
        BUDGETED_DELETE_PER_CONNECTION_BYTES
            + long_response_bytes
            + frame_count * 2 * BUDGETED_DELETE_SLACK_BYTES_PER_FRAME,
    )
    .expect("一两 MiB");
    if pending_peak > pending_bound {
        return Err(format!(
            "额度里记着的字节峰值 {pending_peak}，超过每条连接额度加一个长键回应加每帧余量（{pending_bound}）：删前缀回应在额度之外叠了一层"
        ));
    }
    drop(pipelining);
    Ok(format!(
        "每条连接额度 {BUDGETED_DELETE_PER_CONNECTION_BYTES} 字节、前缀下一把 {BUDGETED_DELETE_LONG_KEY_BYTES} 字节的长键：同时排 {frame_count} 帧只数不删，\
         {busy_frames} 帧回忙、{counted_frames} 帧回了长键；排在头一个的删前缀回忙、长键还在，别的连接照常读写；额度峰值 {pending_peak} 字节（上界 {pending_bound}）"
    ))
}

/// 工作线程额度那一格：服务整份额度（每条连接的额度照缺省，比它大）。
const WORKER_BUDGET_PENDING_BYTES: usize = 1024 * 1024;
/// 同一格：每把值的字节数（不分段的整值，一帧装得下两把）。
const WORKER_BUDGET_VALUE_BYTES: usize = 20 * 1024 * 1024;
/// 同一格：每个前缀下几把值（一帧扫回它们全部，回应约 40 MiB）。
const WORKER_BUDGET_VALUES_PER_PREFIX: usize = 2;
/// 同一格：同时扫的连接数，与自证服务的工作线程数相同（`selftest::SELFTEST_WORKER_THREADS`）。
const WORKER_BUDGET_SCANNERS: usize = 4;
/// 同一格：每条连接头一帧（序号 1）在工作线程里压这么久再做，几个工作线程同时开始编回应。
const WORKER_BUDGET_HOLD_MILLISECONDS: u64 = 600;
/// 同一格：扫完之后额度里还许记着的字节（统计那一帧自己的请求与回应）。
const WORKER_BUDGET_SETTLED_SLACK_BYTES: u64 = 64 * 1024;

/// 一个前缀下的值：`prefix` 的序号与值的序号定每个字节。
fn worker_budget_value(prefix_index: usize, value_index: usize) -> Vec<u8> {
    let filler = u8::try_from((prefix_index * WORKER_BUDGET_VALUES_PER_PREFIX + value_index) % 251)
        .expect("取模 251 放得进 u8");
    vec![filler; WORKER_BUDGET_VALUE_BYTES]
}

fn worker_budget_prefix(prefix_index: usize) -> Vec<u8> {
    bytes(&format!("worker-budget/{prefix_index}/"))
}

/// 工作线程边编边记额度（`server.rs` 文件头「额度」）：整份额度 1 MiB，四条连接同时按前缀扫各约 40 MiB 的回应（工作线程四个，头一帧都压 600 毫秒再做，
/// 四个工作线程同时开始编）：工作线程手里编着的回应合计的峰值不超额度加一帧（`WORKER_RESPONSE_BYTES_PEAK_COUNTER`），记不进的帧回忙、
/// 容错客户端退避重发，四条连接扫回的值逐字节对；扫完之后别的连接照常写读、额度放回。
pub(crate) fn cell_worker_responses_within_pending_budget(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::ResponseBuiltBeforeBudget);
    let pending_bytes = WORKER_BUDGET_PENDING_BYTES.to_string();
    let hold = format!("1:{WORKER_BUDGET_HOLD_MILLISECONDS}");
    let server = HardeningServer::start(
        context,
        switches,
        &[
            "--maximum-pending-bytes",
            &pending_bytes,
            "--selftest-delay",
            &hold,
        ],
    )?;
    let route = server.route();
    let mut writer = SimpleClient::connect(&route, BreakSwitches::none())?;
    for prefix_index in 0..WORKER_BUDGET_SCANNERS {
        let entries: Vec<(Table, Vec<u8>, Vec<u8>)> = (0..WORKER_BUDGET_VALUES_PER_PREFIX)
            .map(|value_index| {
                let mut key = worker_budget_prefix(prefix_index);
                key.extend_from_slice(format!("value-{value_index}").as_bytes());
                (
                    Table::Conclusions,
                    key,
                    worker_budget_value(prefix_index, value_index),
                )
            })
            .collect();
        writer.write(&entries)?;
    }
    let scanners: Vec<thread::JoinHandle<Result<(), String>>> = (0..WORKER_BUDGET_SCANNERS)
        .map(|prefix_index| {
            let route = route.clone();
            let journal = server.path().join(format!("scanner-{prefix_index}.journal"));
            thread::spawn(move || {
                let mut client = ReliableClient::open(plain_client_settings(
                    route,
                    journal,
                    &BreakSwitches::none(),
                ))
                .map_err(|failure| format!("扫的客户端开不了：{failure}"))?;
                let prefix = worker_budget_prefix(prefix_index);
                let (is_truncated, entries) = client
                    .scan(Table::Conclusions, &prefix, 100)
                    .map_err(|failure| format!("扫前缀 {} 不成：{failure}", String::from_utf8_lossy(&prefix)))?;
                let matched = !is_truncated
                    && entries.len() == WORKER_BUDGET_VALUES_PER_PREFIX
                    && entries
                        .iter()
                        .enumerate()
                        .all(|(value_index, entry)| entry.value == worker_budget_value(prefix_index, value_index));
                if !matched {
                    return Err(format!(
                        "扫前缀 {} 回 {} 条（截断 {is_truncated}），该是 {WORKER_BUDGET_VALUES_PER_PREFIX} 条、逐字节与写进去的相同",
                        String::from_utf8_lossy(&prefix),
                        entries.len()
                    ));
                }
                Ok(())
            })
        })
        .collect();
    for scanner in scanners {
        scanner
            .join()
            .map_err(|_| "扫的线程 panic 了".to_string())??;
    }
    writer.write(&[entry("worker-budget/after", "still-serving")])?;
    let after = writer.read(&[(Table::Conclusions, bytes("worker-budget/after"))])?;
    if after != [Some(bytes("still-serving"))] {
        return Err(format!(
            "扫完之后别的连接写了又读回 {after:?}，该读回 still-serving"
        ));
    }
    let statistics = writer.statistics()?;
    let peak = counter(&statistics, WORKER_RESPONSE_BYTES_PEAK_COUNTER)?;
    let busy = counter(&statistics, BUSY_REJECTIONS_COUNTER)?;
    let settled = counter(&statistics, PENDING_BYTES_COUNTER)?;
    let bound = u64::try_from(WORKER_BUDGET_PENDING_BYTES).unwrap_or(u64::MAX)
        + u64::from(MAXIMUM_FRAME_BODY_BYTES);
    if peak > bound {
        return Err(format!(
            "工作线程手里编着的回应合计峰值 {peak} 字节，超过额度 {WORKER_BUDGET_PENDING_BYTES} 加一帧 {MAXIMUM_FRAME_BODY_BYTES} = {bound}：\
             回应先编好整份才记额度，几个工作线程同时编的那几份都在额度之外"
        ));
    }
    if busy == 0 || settled > WORKER_BUDGET_SETTLED_SLACK_BYTES {
        return Err(format!(
            "四条连接同时扫各约 40 MiB：回忙 {busy} 次（该至少 1 次，额度 1 MiB 装不下两份回应）、扫完之后额度里还记着 {settled} 字节（该至多 {WORKER_BUDGET_SETTLED_SLACK_BYTES}）"
        ));
    }
    Ok(format!(
        "额度 {WORKER_BUDGET_PENDING_BYTES} 字节、四条连接同时扫各 {} 字节：工作线程手里编着的回应合计峰值 {peak} 字节（上界 {bound}）、回忙 {busy} 次、四份都扫回且逐字节对、扫完别的连接照常写读、额度放回到 {settled} 字节",
        WORKER_BUDGET_VALUE_BYTES * WORKER_BUDGET_VALUES_PER_PREFIX
    ))
}

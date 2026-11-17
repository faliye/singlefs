//! 自证里 GPU 判的三族那九格（`gpu_judging_rows` 文件头，帧在 `protocol.rs` 文件头「不在才写」「写结论」）：
//! 写结论核认领（别人持着、租期没到拒；租期过了照写；自己持着、没人持着照写）、核守卫那一行（两份都绿而不同不覆盖、不悄悄丢）、
//! 核放在去重之后、待判行不在才写、写结论删待判行、删的待判行读完之后被改过整帧拒、强制现判另写那一行。
//! 每格起自己的服务，服务与客户端只带这一格要证红的那一项弄坏开关（`BreakSwitches::only_these`）：别的格的开关不碰这一格。
//! 每格只核自己点名的那一样，不顺带核别的格核的（例：只有删待判行那一格读待判行删没删），一项开关只红它点名的那一格。

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::SimpleClient;
use crate::gpu_judging_rows::{
    gpu_claim_key, gpu_pending_key, gpu_pending_prefix_of_stream, gpu_rejudged_key,
    GpuBlockAddress, GpuClaimHolder, GpuConclusion, GpuPendingRow, GpuRejudgedRow,
};
use crate::protocol::{
    encode_claim_frame, encode_guarded_conclusion_write_frame, ClaimOutcome, GuardRefusal,
    GuardRowState, GuardedConclusionWrite, ResponseOutcome, Table,
};
use crate::reliable_client::{GuardedConclusionWriteOutcome, ReliableClient, WriteAcknowledgement};
use crate::selftest::{start_test_server_with_switches, CellContext, CellDirectory, TestServer};
use crate::selftest_fault_cells::{client_settings, failure_text};
use crate::sha256::sha256_digest;

/// 这几格的流键：以表标签那样的字节起头，与别的格的键不撞。
const SELFTEST_STREAM_KEY: &[u8] = b"\x1cgpu-selftest-stream";
/// 认领的长租期：比一格跑的时间长得多，格里「租期没到」那一问都问在它里面。
const LONG_LEASE_MILLISECONDS: u32 = 60_000;
/// 「租期过了」那一格的短租期。
const SHORT_LEASE_MILLISECONDS: u32 = 300;
/// 同一格：认领之后等这么久再写，短租期一定过了（服务按自己的时钟判）。
const WAIT_PAST_SHORT_LEASE: Duration = Duration::from_millis(1_000);
/// 样本待判行一块的状态数（只用来造区间）。
const STATES_PER_SAMPLE_BLOCK: u64 = 65_536;
/// 这几格的持有者都取同一个服务进程随机号，按卡序号分人。
const SELFTEST_SERVICE_START_RANDOM: u64 = 0x5e1f_7e57;

fn holder_on_card(card_index: u32) -> GpuClaimHolder {
    GpuClaimHolder {
        service_start_random: SELFTEST_SERVICE_START_RANDOM,
        card_index,
        judging_thread_index: 0,
    }
}

fn block_of(block_number: u64) -> GpuBlockAddress {
    GpuBlockAddress {
        stream_key: SELFTEST_STREAM_KEY.to_vec(),
        block_number,
    }
}

fn conclusion_block_key(block_number: u64) -> Vec<u8> {
    format!("\x10gpu-selftest-conclusion/{block_number:04}").into_bytes()
}

fn side_row_key(block_number: u64) -> Vec<u8> {
    format!("\x11gpu-selftest-side/{block_number:04}").into_bytes()
}

fn sample_pending_row(block_number: u64, recording_table_key: &[u8]) -> GpuPendingRow {
    GpuPendingRow {
        recording_table_key: recording_table_key.to_vec(),
        judge_table_digest: sha256_digest(b"selftest judge table").to_vec(),
        start_state_sequence: block_number * STATES_PER_SAMPLE_BLOCK,
        end_state_sequence: (block_number + 1) * STATES_PER_SAMPLE_BLOCK,
        expected_gpu_judge_group_key: b"selftest-gpu-judge-group".to_vec(),
        expected_conclusion_version_key: b"selftest-conclusion-version".to_vec(),
        is_forced: false,
        pool_sequence: 1,
    }
}

/// 一块的结论：一条旁表行在前、结论块那一行排最后；`value` 给 `None` 是不重写结论块（库里那一份与判出的相同）。
/// 写的一方读到的待判行是「不在」（这几格里没写待判行的那几块）；写了待判行的格另设 `pending_row_as_read`。
fn sample_conclusion(
    holder: GpuClaimHolder,
    block_number: u64,
    guard_expectation: GuardRowState,
    value: Option<&[u8]>,
    forced_pool_sequence: Option<u64>,
) -> GpuConclusion {
    GpuConclusion {
        block: block_of(block_number),
        holder,
        conclusion_block_key: conclusion_block_key(block_number),
        guard_expectation,
        side_entries: vec![(
            Table::Conclusions,
            side_row_key(block_number),
            b"side row".to_vec(),
        )],
        conclusion_block_value: value.map(<[u8]>::to_vec),
        forced_pool_sequence,
        pending_row_as_read: GuardRowState::Absent,
    }
}

/// 一格的服务与一个容错客户端（追加日志在 `directory` 里）；服务与客户端都只带 `switches`。
fn server_and_client(
    context: &CellContext,
    switches: &BreakSwitches,
    directory: &Path,
) -> Result<(TestServer, ReliableClient), String> {
    let server = start_test_server_with_switches(context, switches)?;
    let client = open_client(&server, switches, directory, "writer.journal")?;
    Ok((server, client))
}

fn open_client(
    server: &TestServer,
    switches: &BreakSwitches,
    directory: &Path,
    journal_name: &str,
) -> Result<ReliableClient, String> {
    let stop = Arc::new(AtomicBool::new(false));
    ReliableClient::open(client_settings(
        server.unix_route(),
        directory.join(journal_name),
        switches,
        &stop,
    ))
    .map_err(|failure| failure_text(&failure))
}

fn write_conclusion(
    client: &mut ReliableClient,
    conclusion: &GpuConclusion,
) -> Result<GuardedConclusionWriteOutcome, String> {
    client
        .write_gpu_conclusion(conclusion)
        .map_err(|failure| failure_text(&failure))
}

fn claim_blocks(
    client: &mut ReliableClient,
    holder: &GpuClaimHolder,
    block_numbers: &[u64],
    lease_milliseconds: u32,
) -> Result<Vec<ClaimOutcome>, String> {
    let blocks: Vec<(Vec<u8>, u64)> = block_numbers
        .iter()
        .map(|block_number| (SELFTEST_STREAM_KEY.to_vec(), *block_number))
        .collect();
    client
        .claim_gpu_blocks(holder, &blocks, lease_milliseconds)
        .map_err(|failure| failure_text(&failure))
}

fn read_keys(
    client: &mut ReliableClient,
    keys: &[Vec<u8>],
) -> Result<Vec<Option<Vec<u8>>>, String> {
    let keys_in_table: Vec<(Table, Vec<u8>)> = keys
        .iter()
        .map(|key| (Table::Conclusions, key.clone()))
        .collect();
    client
        .read(&keys_in_table)
        .map_err(|failure| failure_text(&failure))
}

fn is_written_fresh(outcome: &GuardedConclusionWriteOutcome) -> bool {
    matches!(
        outcome,
        GuardedConclusionWriteOutcome::Written(WriteAcknowledgement {
            is_already_applied: false,
            ..
        })
    )
}

/// 别的持有者持着、租期没到：写结论整帧拒（状态 7 种类 1，带持有者与剩余租期），库里结论块、旁表行一条都没落，
/// 拒掉的那一帧从追加日志里勾掉（重开客户端不重放）。判别力：弄坏开关 conclusion-guard-claim-unchecked。
pub(crate) fn cell_conclusion_write_refused_while_another_holder_leases(
    context: &CellContext,
) -> Result<String, String> {
    const BLOCK: u64 = 1;
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::ConclusionGuardClaimUnchecked]);
    let directory = CellDirectory::create()?;
    let (server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let lease_holder = holder_on_card(0);
    let writer = holder_on_card(1);
    let claimed = claim_blocks(
        &mut client,
        &lease_holder,
        &[BLOCK],
        LONG_LEASE_MILLISECONDS,
    )?;
    if claimed != vec![ClaimOutcome::Claimed] {
        return Err(format!("持有者认领这一块该认领到，回的是 {claimed:?}"));
    }
    let outcome = write_conclusion(
        &mut client,
        &sample_conclusion(
            writer,
            BLOCK,
            GuardRowState::Absent,
            Some(b"verdicts of the writer"),
            None,
        ),
    )?;
    let lease_holder_identity = lease_holder.identity();
    let is_refused_for_the_lease_holder = matches!(
        &outcome,
        GuardedConclusionWriteOutcome::Refused {
            refusal: GuardRefusal::ClaimHeldByOther { holder_identity, remaining_lease_milliseconds },
            ..
        } if *holder_identity == lease_holder_identity && *remaining_lease_milliseconds > 0
    );
    if !is_refused_for_the_lease_holder {
        return Err(format!(
            "别的持有者持着、租期没到时写结论该整帧拒（认领被别人持着，持有者是 {}），回的是 {outcome:?}",
            String::from_utf8_lossy(&lease_holder_identity)
        ));
    }
    let stored = read_keys(
        &mut client,
        &[conclusion_block_key(BLOCK), side_row_key(BLOCK)],
    )?;
    if stored.iter().any(Option::is_some) {
        return Err(format!("整帧拒了，结论块与旁表行却落了：{stored:?}"));
    }
    drop(client);
    let reopened = open_client(&server, &switches, &directory.path, "writer.journal")?;
    let replayed = reopened.diagnostics().journal_recovered_writes;
    if replayed != 0 {
        return Err(format!(
            "守卫不过的那一帧该从追加日志里勾掉，重开客户端却重放了 {replayed} 帧"
        ));
    }
    Ok("别的持有者持着、租期没到：写结论整帧拒、报持有者与剩余租期，结论块与旁表行都没落，重开客户端不重放那一帧".to_string())
}

/// 别的持有者的认领租期过了：写结论照写（结论块落了）。判别力：弄坏开关 conclusion-guard-expired-claim-refused。
pub(crate) fn cell_conclusion_write_proceeds_after_the_lease_expires(
    context: &CellContext,
) -> Result<String, String> {
    const BLOCK: u64 = 2;
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::ConclusionGuardExpiredClaimRefused]);
    let directory = CellDirectory::create()?;
    let (_server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let claimed = claim_blocks(
        &mut client,
        &holder_on_card(0),
        &[BLOCK],
        SHORT_LEASE_MILLISECONDS,
    )?;
    if claimed != vec![ClaimOutcome::Claimed] {
        return Err(format!("短租期的持有者该认领到，回的是 {claimed:?}"));
    }
    thread::sleep(WAIT_PAST_SHORT_LEASE);
    let outcome = write_conclusion(
        &mut client,
        &sample_conclusion(
            holder_on_card(1),
            BLOCK,
            GuardRowState::Absent,
            Some(b"verdicts after the lease"),
            None,
        ),
    )?;
    if !is_written_fresh(&outcome) {
        return Err(format!(
            "别人的认领租期（{SHORT_LEASE_MILLISECONDS} 毫秒）过了 {} 毫秒之后写结论该照写，回的是 {outcome:?}",
            WAIT_PAST_SHORT_LEASE.as_millis()
        ));
    }
    let stored = read_keys(&mut client, &[conclusion_block_key(BLOCK)])?;
    if stored != vec![Some(b"verdicts after the lease".to_vec())] {
        return Err(format!("写结论回了照写，结论块读回的是 {stored:?}"));
    }
    Ok(format!(
        "别人的认领租期 {SHORT_LEASE_MILLISECONDS} 毫秒、过了 {} 毫秒：写结论照写，结论块落了",
        WAIT_PAST_SHORT_LEASE.as_millis()
    ))
}

/// 没人持着照写；帧里的持有者自己持着、租期没到照写，写完认领释放（别的持有者随即认领到）。
/// 判别力：弄坏开关 conclusion-guard-own-claim-refused（核成「没人持着才写」，自己持着那一半红）。
pub(crate) fn cell_conclusion_write_proceeds_for_its_own_holder(
    context: &CellContext,
) -> Result<String, String> {
    const UNCLAIMED_BLOCK: u64 = 3;
    const OWN_BLOCK: u64 = 4;
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::ConclusionGuardOwnClaimRefused]);
    let directory = CellDirectory::create()?;
    let (_server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let owner = holder_on_card(0);
    let unclaimed = write_conclusion(
        &mut client,
        &sample_conclusion(
            owner,
            UNCLAIMED_BLOCK,
            GuardRowState::Absent,
            Some(b"unclaimed"),
            None,
        ),
    )?;
    if !is_written_fresh(&unclaimed) {
        return Err(format!("没人持着时写结论该照写，回的是 {unclaimed:?}"));
    }
    let claimed = claim_blocks(&mut client, &owner, &[OWN_BLOCK], LONG_LEASE_MILLISECONDS)?;
    if claimed != vec![ClaimOutcome::Claimed] {
        return Err(format!("持有者认领这一块该认领到，回的是 {claimed:?}"));
    }
    let own = write_conclusion(
        &mut client,
        &sample_conclusion(owner, OWN_BLOCK, GuardRowState::Absent, Some(b"own"), None),
    )?;
    if !is_written_fresh(&own) {
        return Err(format!(
            "帧里的持有者自己持着、租期没到时写结论该照写，回的是 {own:?}"
        ));
    }
    let after = claim_blocks(
        &mut client,
        &holder_on_card(1),
        &[OWN_BLOCK],
        LONG_LEASE_MILLISECONDS,
    )?;
    if after != vec![ClaimOutcome::Claimed] {
        return Err(format!(
            "写结论那一帧该释放这一块的认领，别的持有者随即认领却回 {after:?}"
        ));
    }
    Ok("没人持着照写；自己持着、租期没到照写，写完认领释放、别的持有者随即认领到".to_string())
}

/// 两个写的一方都读到「不在」、判出两份不同的结论：先到的落下，后到的整帧拒（状态 7 种类 2，带库里那一份的 SHA-256），
/// 不覆盖先到的、也不悄悄丢掉（写的一方拿到拒绝，照自己的规则比：两份都绿而不同报红）；带重读到的那一份再写才换上（有一份红整份换的那一路）。
/// 判别力：弄坏开关 conclusion-guard-row-unchecked。
pub(crate) fn cell_differing_conclusion_refused_not_dropped(
    context: &CellContext,
) -> Result<String, String> {
    const BLOCK: u64 = 5;
    const EARLIER: &[u8] = b"green verdicts of the earlier writer";
    const LATER: &[u8] = b"different green verdicts of the later writer";
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::ConclusionGuardRowUnchecked]);
    let directory = CellDirectory::create()?;
    let (_server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let earlier = write_conclusion(
        &mut client,
        &sample_conclusion(
            holder_on_card(0),
            BLOCK,
            GuardRowState::Absent,
            Some(EARLIER),
            None,
        ),
    )?;
    if !is_written_fresh(&earlier) {
        return Err(format!("先到的写结论该落下，回的是 {earlier:?}"));
    }
    let later = write_conclusion(
        &mut client,
        &sample_conclusion(
            holder_on_card(1),
            BLOCK,
            GuardRowState::Absent,
            Some(LATER),
            None,
        ),
    )?;
    let stored_state = GuardRowState::of_stored_value(Some(EARLIER));
    let is_refused_with_the_stored_one = matches!(
        &later,
        GuardedConclusionWriteOutcome::Refused {
            refusal: GuardRefusal::GuardRowChanged { current },
            ..
        } if *current == stored_state
    );
    if !is_refused_with_the_stored_one {
        return Err(format!(
            "后到的写的一方读到的是「不在」、库里已是先到的那一份：该整帧拒、带先到那一份的 SHA-256，回的是 {later:?}"
        ));
    }
    let after_refusal = read_keys(&mut client, &[conclusion_block_key(BLOCK)])?;
    if after_refusal != vec![Some(EARLIER.to_vec())] {
        return Err(format!(
            "后到的被拒之后结论块该还是先到的那一份，读回 {after_refusal:?}"
        ));
    }
    let replaced = write_conclusion(
        &mut client,
        &sample_conclusion(holder_on_card(1), BLOCK, stored_state, Some(LATER), None),
    )?;
    let after_replacement = read_keys(&mut client, &[conclusion_block_key(BLOCK)])?;
    if !is_written_fresh(&replaced) || after_replacement != vec![Some(LATER.to_vec())] {
        return Err(format!(
            "带重读到的那一份再写该换上：回的是 {replaced:?}，结论块读回 {after_replacement:?}"
        ));
    }
    Ok("两个写的一方都读到「不在」：先到的落下，后到的整帧拒、带库里那一份的 SHA-256，库里还是先到的；带重读到的那一份再写才换上".to_string())
}

/// 用一问一答的客户端照给的请求号发一帧写结论（这一格要重发同一请求号）。
fn send_guarded_conclusion_write(
    client: &mut SimpleClient,
    request_number: u64,
    request: &GuardedConclusionWrite,
) -> Result<ResponseOutcome, String> {
    let client_identity = client.client_identity.clone();
    client.call(|sequence| {
        encode_guarded_conclusion_write_frame(sequence, &client_identity, request_number, request)
    })
}

/// 核放在去重之后：一帧写结论落下之后，别的持有者认领了这一块、结论块也已是它自己写的值，同一请求号重发回「早已生效」，不当守卫不过拒。
/// 判别力：弄坏开关 conclusion-guard-before-deduplication。
pub(crate) fn cell_conclusion_guard_checked_after_deduplication(
    context: &CellContext,
) -> Result<String, String> {
    const BLOCK: u64 = 6;
    const REQUEST_NUMBER: u64 = 1;
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::ConclusionGuardBeforeDeduplication]);
    let server = start_test_server_with_switches(context, &switches)?;
    let mut client = SimpleClient::connect(&server.unix_route(), switches)?;
    let request = sample_conclusion(
        holder_on_card(0),
        BLOCK,
        GuardRowState::Absent,
        Some(b"once"),
        None,
    )
    .guarded_write()?;
    let initial_answer = send_guarded_conclusion_write(&mut client, REQUEST_NUMBER, &request)?;
    let written_fresh = ResponseOutcome::WriteAcknowledged {
        entries_written: 2,
        is_already_applied: false,
    };
    if initial_answer != written_fresh {
        return Err(format!(
            "没生效过的那一次写结论该落下 2 条，回的是 {initial_answer:?}"
        ));
    }
    let next_holder_identity = holder_on_card(1).identity();
    let claim_key_of_the_block = gpu_claim_key(SELFTEST_STREAM_KEY, BLOCK);
    let next_claim = client.call(|sequence| {
        encode_claim_frame(
            sequence,
            &next_holder_identity,
            LONG_LEASE_MILLISECONDS,
            &[(Table::Conclusions, claim_key_of_the_block.as_slice())],
        )
    })?;
    if next_claim != ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]) {
        return Err(format!(
            "写结论释放了认领，别的持有者该认领到，回的是 {next_claim:?}"
        ));
    }
    let resent = send_guarded_conclusion_write(&mut client, REQUEST_NUMBER, &request)?;
    let already_applied = ResponseOutcome::WriteAcknowledged {
        entries_written: 2,
        is_already_applied: true,
    };
    if resent != already_applied {
        return Err(format!(
            "同一请求号重发早已落下的写结论该回「早已生效」（核放在去重之后），回的是 {resent:?}"
        ));
    }
    Ok("写结论落下、别的持有者接着认领之后，同一请求号重发回「早已生效」（2 条），没被当成守卫不过".to_string())
}

/// 待判行不在才写：录两块的那一帧两块都写；重录那一帧一块已在、一块新的，只写新的那一块，已在的不改；按流的前缀扫回三块、按块号升序。
/// 判别力：弄坏开关 write-if-absent-overwrites。
pub(crate) fn cell_pending_rows_written_only_if_absent(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::WriteIfAbsentOverwrites]);
    let directory = CellDirectory::create()?;
    let (_server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let recorded = b"\x04recorded".as_slice();
    let rerecorded = b"\x04re-recorded".as_slice();
    let rows =
        |block_numbers: &[u64], recording_table_key: &[u8]| -> Vec<(Vec<u8>, u64, GpuPendingRow)> {
            block_numbers
                .iter()
                .map(|block_number| {
                    (
                        SELFTEST_STREAM_KEY.to_vec(),
                        *block_number,
                        sample_pending_row(*block_number, recording_table_key),
                    )
                })
                .collect()
        };
    let initial = client
        .write_gpu_pending_rows(&rows(&[7, 8], recorded))
        .map_err(|failure| failure_text(&failure))?;
    let again = client
        .write_gpu_pending_rows(&rows(&[7, 9], rerecorded))
        .map_err(|failure| failure_text(&failure))?;
    if initial.entries_written != 2 || again.entries_written != 1 {
        return Err(format!(
            "录两块的那一帧两块都不在该写 2 条（回 {initial:?}），重录那一帧块 7 已在、块 9 不在该只写 1 条（回 {again:?}）"
        ));
    }
    let stored = read_keys(
        &mut client,
        &[
            gpu_pending_key(SELFTEST_STREAM_KEY, 7),
            gpu_pending_key(SELFTEST_STREAM_KEY, 9),
        ],
    )?;
    let expected = vec![
        Some(sample_pending_row(7, recorded).encode()),
        Some(sample_pending_row(9, rerecorded).encode()),
    ];
    if stored != expected {
        return Err(format!(
            "块 7 该还是录两块那一帧写的那一行、块 9 是重录那一帧写的：读回 {stored:?}"
        ));
    }
    let (_, scanned) = client
        .scan(
            Table::Conclusions,
            &gpu_pending_prefix_of_stream(SELFTEST_STREAM_KEY),
            100,
        )
        .map_err(|failure| failure_text(&failure))?;
    let scanned_keys: Vec<Vec<u8>> = scanned.into_iter().map(|entry| entry.key).collect();
    let expected_keys: Vec<Vec<u8>> = [7, 8, 9]
        .iter()
        .map(|block_number| gpu_pending_key(SELFTEST_STREAM_KEY, *block_number))
        .collect();
    if scanned_keys != expected_keys {
        return Err(format!(
            "按流的前缀扫该回块 7、8、9 三行、按块号升序，回的是 {:?}",
            scanned_keys
                .iter()
                .map(|key| String::from_utf8_lossy(key).into_owned())
                .collect::<Vec<_>>()
        ));
    }
    Ok("待判行不在才写：录两块的那一帧写 2 条；重录那一帧块 7 已在只写块 9（1 条），块 7 还是录两块那一帧写的；按流的前缀扫回三行、按块号升序".to_string())
}

/// 写结论那一帧删掉这一块的待判行，结论块与旁表行同一帧落下。判别力：弄坏开关 conclusion-write-pending-rows-kept。
pub(crate) fn cell_conclusion_write_deletes_the_pending_row(
    context: &CellContext,
) -> Result<String, String> {
    const BLOCK: u64 = 10;
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::ConclusionWritePendingRowsKept]);
    let directory = CellDirectory::create()?;
    let (_server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let pending_row = sample_pending_row(BLOCK, b"\x04recording");
    client
        .write_gpu_pending_rows(&[(SELFTEST_STREAM_KEY.to_vec(), BLOCK, pending_row.clone())])
        .map_err(|failure| failure_text(&failure))?;
    let mut conclusion = sample_conclusion(
        holder_on_card(0),
        BLOCK,
        GuardRowState::Absent,
        Some(b"concluded"),
        None,
    );
    conclusion.pending_row_as_read = GuardRowState::of_stored_value(Some(&pending_row.encode()));
    let outcome = write_conclusion(&mut client, &conclusion)?;
    if !is_written_fresh(&outcome) {
        return Err(format!("写结论该落下，回的是 {outcome:?}"));
    }
    let stored = read_keys(
        &mut client,
        &[
            gpu_pending_key(SELFTEST_STREAM_KEY, BLOCK),
            side_row_key(BLOCK),
            conclusion_block_key(BLOCK),
        ],
    )?;
    let expected = vec![
        None,
        Some(b"side row".to_vec()),
        Some(b"concluded".to_vec()),
    ];
    if stored != expected {
        return Err(format!(
            "写结论之后待判行该没了、旁表行与结论块在：读回 {stored:?}"
        ));
    }
    Ok("写结论那一帧删掉了待判行，旁表行与结论块同一帧落下".to_string())
}

/// 写的一方读到的待判行（不强制）在它写结论之前被判段升成强制现判：那一帧整帧拒（状态 7 种类 3，带待判行那把键与库里此刻那一份的 SHA-256），
/// 结论块与旁表行都没落、升过的那一行还在；带重读到的那一份再写才落下（删没删待判行归 conclusion-write-deletes-the-pending-row 那一格核）。
/// 判别力：弄坏开关 conclusion-guard-deleted-row-unchecked。
pub(crate) fn cell_conclusion_write_refused_when_the_pending_row_changed(
    context: &CellContext,
) -> Result<String, String> {
    const BLOCK: u64 = 12;
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::ConclusionGuardDeletedRowUnchecked]);
    let directory = CellDirectory::create()?;
    let (_server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let pending_key = gpu_pending_key(SELFTEST_STREAM_KEY, BLOCK);
    let read_by_the_writer = sample_pending_row(BLOCK, b"\x04recording");
    let mut raised_to_forced = read_by_the_writer.clone();
    raised_to_forced.is_forced = true;
    raised_to_forced.pool_sequence = 2;
    client
        .write_gpu_pending_rows(&[(
            SELFTEST_STREAM_KEY.to_vec(),
            BLOCK,
            read_by_the_writer.clone(),
        )])
        .map_err(|failure| failure_text(&failure))?;
    client
        .submit_write_entries(&[(
            Table::Conclusions,
            pending_key.clone(),
            raised_to_forced.encode(),
        )])
        .and_then(|_| client.wait_all_acknowledged())
        .map_err(|failure| failure_text(&failure))?;
    let mut stale = sample_conclusion(
        holder_on_card(0),
        BLOCK,
        GuardRowState::Absent,
        Some(b"judged from the row as read"),
        None,
    );
    stale.pending_row_as_read = GuardRowState::of_stored_value(Some(&read_by_the_writer.encode()));
    let refused = write_conclusion(&mut client, &stale)?;
    let raised_state = GuardRowState::of_stored_value(Some(&raised_to_forced.encode()));
    let is_refused_with_the_raised_row = matches!(
        &refused,
        GuardedConclusionWriteOutcome::Refused {
            refusal: GuardRefusal::DeletedRowChanged { key, current },
            ..
        } if *key == pending_key && *current == raised_state
    );
    if !is_refused_with_the_raised_row {
        return Err(format!(
            "写的一方读到的是不强制的待判行、库里已升成强制现判：该整帧拒、带待判行那把键与升过那一份的 SHA-256，回的是 {refused:?}"
        ));
    }
    let after_refusal = read_keys(
        &mut client,
        &[
            pending_key,
            side_row_key(BLOCK),
            conclusion_block_key(BLOCK),
        ],
    )?;
    if after_refusal != vec![Some(raised_to_forced.encode()), None, None] {
        return Err(format!(
            "整帧拒了：升过的待判行该还在、旁表行与结论块都没落，读回 {after_refusal:?}"
        ));
    }
    let mut reread = sample_conclusion(
        holder_on_card(0),
        BLOCK,
        GuardRowState::Absent,
        Some(b"judged from the row as read"),
        Some(raised_to_forced.pool_sequence),
    );
    reread.pending_row_as_read = raised_state;
    let written = write_conclusion(&mut client, &reread)?;
    let after_rewrite = read_keys(&mut client, &[conclusion_block_key(BLOCK)])?;
    if !is_written_fresh(&written)
        || after_rewrite != vec![Some(b"judged from the row as read".to_vec())]
    {
        return Err(format!(
            "带重读到的那一份再写该落下：回的是 {written:?}，结论块读回 {after_rewrite:?}"
        ));
    }
    Ok("待判行在写的一方读完之后被升成强制现判：写结论整帧拒、带那把键与升过那一份的 SHA-256，升过的那一行还在、结论没落；带重读到的那一份再写才落下".to_string())
}

/// 强制现判：写结论那一帧另写 `gpu-rejudged/<入池序号>/<流键>/<块号>`，值里的持有者是判它的那一个、结论摘要是结论块的 SHA-256；
/// 库里那一份与判出的相同、不重写结论块时，摘要取读到的那一份。判别力：弄坏开关 rejudged-row-dropped（容错客户端那一侧）。
pub(crate) fn cell_forced_rejudge_writes_the_rejudged_row(
    context: &CellContext,
) -> Result<String, String> {
    const BLOCK: u64 = 11;
    const FRESH_POOL_SEQUENCE: u64 = 4;
    const UNCHANGED_POOL_SEQUENCE: u64 = 5;
    const VERDICTS: &[u8] = b"forced verdicts";
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::RejudgedRowDropped]);
    let directory = CellDirectory::create()?;
    let (_server, mut client) = server_and_client(context, &switches, &directory.path)?;
    let fresh_judge = holder_on_card(0);
    let unchanged_judge = holder_on_card(1);
    let fresh = write_conclusion(
        &mut client,
        &sample_conclusion(
            fresh_judge,
            BLOCK,
            GuardRowState::Absent,
            Some(VERDICTS),
            Some(FRESH_POOL_SEQUENCE),
        ),
    )?;
    let unchanged = write_conclusion(
        &mut client,
        &sample_conclusion(
            unchanged_judge,
            BLOCK,
            GuardRowState::of_stored_value(Some(VERDICTS)),
            None,
            Some(UNCHANGED_POOL_SEQUENCE),
        ),
    )?;
    if !is_written_fresh(&fresh) || !is_written_fresh(&unchanged) {
        return Err(format!(
            "两次强制现判的写结论都该落下：头一次回 {fresh:?}，不重写结论块的那一次回 {unchanged:?}"
        ));
    }
    let stored = read_keys(
        &mut client,
        &[
            gpu_rejudged_key(FRESH_POOL_SEQUENCE, SELFTEST_STREAM_KEY, BLOCK),
            gpu_rejudged_key(UNCHANGED_POOL_SEQUENCE, SELFTEST_STREAM_KEY, BLOCK),
        ],
    )?;
    let expected_digest = sha256_digest(VERDICTS);
    for ((stored_value, judge), pool_sequence) in stored
        .iter()
        .zip([fresh_judge, unchanged_judge])
        .zip([FRESH_POOL_SEQUENCE, UNCHANGED_POOL_SEQUENCE])
    {
        let Some(value) = stored_value else {
            return Err(format!(
                "入池序号 {pool_sequence} 的强制现判行该在，读回不在"
            ));
        };
        let row = GpuRejudgedRow::decode(value)?;
        if row.holder_identity != judge.identity() || row.conclusion_digest != expected_digest {
            return Err(format!(
                "入池序号 {pool_sequence} 的强制现判行该带判它的持有者 {} 与结论块的 SHA-256，读回持有者 {}、摘要对不对 {}",
                String::from_utf8_lossy(&judge.identity()),
                String::from_utf8_lossy(&row.holder_identity),
                row.conclusion_digest == expected_digest
            ));
        }
    }
    Ok("强制现判两次（写结论块的、库里相同不重写的）：两行 gpu-rejudged/ 都在，持有者是判它的那一个、结论摘要是结论块的 SHA-256".to_string())
}

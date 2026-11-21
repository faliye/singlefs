//! 认领回到调用方手里之后到期盖住整段租期的一格 `cell_claim_lease_covers_the_holder_after_the_answer`：回应丢了重发、忙回应之后重发、续租不把到期往前挪。

use super::*;

/// 认领租期那一格的数：租期 4 秒（缺省 30 秒按比例缩短）。丢了回应那一半让重发先撞一条不读的连接、等满帧超时 2.5 秒，
/// 认领落下之后至少过 2.5 秒才回到调用方手里；忙那一半让一帧点读在工作线程里压 2.8 秒、服务只许 1 帧待处理，认领一直回忙到压着的那一帧做完。
/// 回来之后过租期的 0.6（2.4 秒）别的身份再认领：持有者从回来那一刻起按整段租期写，这时它的写还没到服务。
const LEASE_COVER_LEASE_MILLISECONDS: u32 = 4_000;
const LEASE_COVER_RESEND_FRAME_TIMEOUT: Duration = Duration::from_millis(2_500);
const LEASE_COVER_HOLDING_READ_MILLISECONDS: u64 = 2_800;
/// 压着的那一帧点读的序号：别的帧（容错客户端与一问一答客户端的序号都从 1 起）撞不上。
const LEASE_COVER_HOLDING_READ_SEQUENCE: u64 = 777_777;
const LEASE_COVER_PROBE_AFTER_ANSWER: Duration = Duration::from_millis(2_400);
/// 同一身份再认领时给的长短两种租期（到期只许往后挪那一问）。
const LEASE_COVER_LONG_LEASE_MILLISECONDS: u32 = 600_000;
const LEASE_COVER_SHORT_LEASE_MILLISECONDS: u32 = 1;
/// 每一半做完的限时。
const LEASE_COVER_DEADLINE: Duration = Duration::from_secs(60);

/// 别的身份（新连的一问一答客户端）认领 `key`，回那一把键的结局。
fn claim_as_another_identity(route: &RouteAddress, key: &[u8]) -> Result<ClaimOutcome, String> {
    let mut other = SimpleClient::connect(route, BreakSwitches::none())?;
    let outcomes = other.claim(
        &[(Table::Conclusions, key.to_vec())],
        LEASE_COVER_LEASE_MILLISECONDS,
    )?;
    match outcomes.as_slice() {
        [outcome] => Ok(outcome.clone()),
        _ => Err(format!(
            "认领一把键回了 {} 个结局：{outcomes:?}",
            outcomes.len()
        )),
    }
}

/// 认领回到持有者手里之后：立刻（还没到头一次落下时算的到期）与过 `LEASE_COVER_PROBE_AFTER_ANSWER`（持有者按整段租期还在写）
/// 各让别的身份认领一次，两次都该是「别人持有」。
fn expect_held_after_the_answer(
    route: &RouteAddress,
    key: &[u8],
    answered: Instant,
    what: &str,
) -> Result<(), String> {
    let right_after = claim_as_another_identity(route, key)?;
    if !matches!(right_after, ClaimOutcome::HeldByOther { .. }) {
        return Err(format!(
            "{what}：认领回来之后立刻（头一次落下时算的到期之前）别的身份认领回 {right_after:?}，该是别人持有"
        ));
    }
    thread::sleep(
        (answered + LEASE_COVER_PROBE_AFTER_ANSWER).saturating_duration_since(Instant::now()),
    );
    let while_writing = claim_as_another_identity(route, key)?;
    if !matches!(while_writing, ClaimOutcome::HeldByOther { .. }) {
        return Err(format!(
            "{what}：认领回来之后 {} 毫秒（持有者从回来那一刻起按整段 {LEASE_COVER_LEASE_MILLISECONDS} 毫秒租期写，这时还没写到服务）\
             别的身份认领回 {while_writing:?}，该是别人持有：服务给的到期没盖住回来之后的整段租期",
            LEASE_COVER_PROBE_AFTER_ANSWER.as_millis()
        ));
    }
    Ok(())
}

/// 头一次认领落下、回应被代理吞掉再断；重发先撞一条不读的连接、等满帧超时，再重连重发才回来。
fn lease_after_a_lost_answer(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let server = HardeningServer::start(context, switches.clone(), &[])?;
    let directory = CellDirectory::create()?;
    let mut proxy = FaultProxy::start(
        ProxyListen::UnixSocket(directory.path.join("proxy.sock")),
        server.route(),
        &BreakSwitches::none(),
    )?;
    let control = proxy.shared_control();
    let stop = Arc::new(AtomicBool::new(false));
    let mut settings = plain_client_settings(
        proxy.route(),
        directory.path.join("lost-answer.journal"),
        switches,
    );
    settings.frame_timeout = LEASE_COVER_RESEND_FRAME_TIMEOUT;
    settings.reconnect = ReconnectPolicy {
        initial_delay: Duration::from_millis(20),
        maximum_delay: Duration::from_millis(200),
        maximum_attempts: 100,
        maximum_total: Duration::from_secs(30),
    };
    let stop_for_client = Arc::clone(&stop);
    settings.stop_check = Arc::new(move || stop_for_client.load(Ordering::SeqCst));
    let key = bytes("lease-cover/lost-answer");
    let control_for_client = Arc::clone(&control);
    let claimed = within_deadline(
        LEASE_COVER_DEADLINE,
        || stop.store(true, Ordering::SeqCst),
        || -> Result<(Vec<ClaimOutcome>, Instant, Instant), ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            control_for_client
                .swallow_responses_then_cut_requests
                .store(1, Ordering::SeqCst);
            control_for_client
                .stall_reading_connections_remaining
                .store(1, Ordering::SeqCst);
            let submitted = Instant::now();
            let outcomes = client.claim(
                &[(Table::Conclusions, key.clone())],
                LEASE_COVER_LEASE_MILLISECONDS,
            )?;
            Ok((outcomes, submitted, Instant::now()))
        },
    )?;
    let cuts = control.cuts_done.load(Ordering::SeqCst);
    let stalls = control.stalls_done.load(Ordering::SeqCst);
    proxy.stop();
    let (outcomes, submitted, answered) =
        claimed.map_err(|failure| format!("丢了回应的认领：客户端出错：{failure}"))?;
    if outcomes != [ClaimOutcome::Claimed] {
        return Err(format!("丢了回应、重发之后认领回 {outcomes:?}，该是认领到"));
    }
    let answer_delay = answered.duration_since(submitted);
    if cuts != 1 || stalls != 1 || answer_delay < LEASE_COVER_RESEND_FRAME_TIMEOUT {
        return Err(format!(
            "代理吞回应再断 {cuts} 次、停读 {stalls} 条连接（都该是 1）、认领 {} 毫秒回来（该至少等满帧超时 {} 毫秒）：故障没注入上，这一格没测到重发",
            answer_delay.as_millis(),
            LEASE_COVER_RESEND_FRAME_TIMEOUT.as_millis()
        ));
    }
    expect_held_after_the_answer(
        &server.route(),
        &key,
        answered,
        "丢了回应、重发才回来的认领",
    )?;
    Ok(format!(
        "丢了回应的认领 {} 毫秒后重发才回来：回来时与回来之后 {} 毫秒别的身份都回别人持有",
        answer_delay.as_millis(),
        LEASE_COVER_PROBE_AFTER_ANSWER.as_millis()
    ))
}

/// 服务只许 1 帧待处理、一帧点读在工作线程里压着：认领一直回忙，压着的那一帧做完才落下。
fn lease_after_a_long_busy(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let holding_delay =
        format!("{LEASE_COVER_HOLDING_READ_SEQUENCE}:{LEASE_COVER_HOLDING_READ_MILLISECONDS}");
    let server = HardeningServer::start(
        context,
        switches.clone(),
        &[
            "--maximum-pending-total",
            "1",
            "--busy-retry-milliseconds",
            "100",
            "--selftest-delay",
            &holding_delay,
        ],
    )?;
    let directory = CellDirectory::create()?;
    let stop = Arc::new(AtomicBool::new(false));
    let mut settings = plain_client_settings(
        server.route(),
        directory.path.join("long-busy.journal"),
        switches,
    );
    let stop_for_client = Arc::clone(&stop);
    settings.stop_check = Arc::new(move || stop_for_client.load(Ordering::SeqCst));
    let mut holding = connect_raw(&server.socket_path())?;
    let key = bytes("lease-cover/long-busy");
    let claimed = within_deadline(
        LEASE_COVER_DEADLINE,
        || stop.store(true, Ordering::SeqCst),
        || -> Result<(Vec<ClaimOutcome>, u64, Instant, Instant), String> {
            let mut client =
                ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
            let mut builder = begin_point_read_frame(LEASE_COVER_HOLDING_READ_SEQUENCE, 1, 32);
            builder.key(Table::Conclusions, b"lease-cover/holding");
            holding
                .write_all(&builder.finish())
                .map_err(|error| format!("压着的那一帧点读发不出去：{error}"))?;
            thread::sleep(Duration::from_millis(100));
            let submitted = Instant::now();
            let outcomes = client
                .claim(
                    &[(Table::Conclusions, key.clone())],
                    LEASE_COVER_LEASE_MILLISECONDS,
                )
                .map_err(|failure| failure.to_string())?;
            Ok((
                outcomes,
                client.diagnostics().busy_responses,
                submitted,
                Instant::now(),
            ))
        },
    )?;
    let (outcomes, busy_responses, submitted, answered) =
        claimed.map_err(|message| format!("忙了很久的认领：{message}"))?;
    drop(holding);
    if outcomes != [ClaimOutcome::Claimed] {
        return Err(format!("忙了很久之后认领回 {outcomes:?}，该是认领到"));
    }
    let answer_delay = answered.duration_since(submitted);
    let half_lease = Duration::from_millis(u64::from(LEASE_COVER_LEASE_MILLISECONDS / 2));
    if busy_responses == 0 || answer_delay < half_lease {
        return Err(format!(
            "认领回了 {busy_responses} 次忙（该至少 1 次）、{} 毫秒回来（该过了租期的一半 {} 毫秒）：忙没造出来，这一格没测到",
            answer_delay.as_millis(),
            half_lease.as_millis()
        ));
    }
    expect_held_after_the_answer(&server.route(), &key, answered, "忙了大半租期才落下的认领")?;
    Ok(format!(
        "认领回忙 {busy_responses} 次、{} 毫秒才落下：回来时与回来之后 {} 毫秒别的身份都回别人持有",
        answer_delay.as_millis(),
        LEASE_COVER_PROBE_AFTER_ANSWER.as_millis()
    ))
}

/// 同一身份先用长租期认领，再用 1 毫秒的租期认领同一把键：到期不往前挪，别的身份过一会照样回别人持有、剩余租期还是长的。
fn renewal_never_moves_the_expiry_earlier(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let server = HardeningServer::start(context, switches.clone(), &[])?;
    let key = bytes("lease-cover/renewed-shorter");
    let mut holder = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    for lease in [
        LEASE_COVER_LONG_LEASE_MILLISECONDS,
        LEASE_COVER_SHORT_LEASE_MILLISECONDS,
    ] {
        let outcomes = holder.claim(&[(Table::Conclusions, key.clone())], lease)?;
        if outcomes != [ClaimOutcome::Claimed] {
            return Err(format!(
                "同一身份用 {lease} 毫秒的租期认领自己的键回 {outcomes:?}，该是认领到"
            ));
        }
    }
    thread::sleep(Duration::from_millis(100));
    let probe = claim_as_another_identity(&server.route(), &key)?;
    let half_long_lease = u64::from(LEASE_COVER_LONG_LEASE_MILLISECONDS / 2);
    match probe {
        ClaimOutcome::HeldByOther {
            remaining_lease_milliseconds,
            ..
        } if remaining_lease_milliseconds > half_long_lease => Ok(format!(
            "同一身份 {LEASE_COVER_LONG_LEASE_MILLISECONDS} 毫秒认领之后再用 {LEASE_COVER_SHORT_LEASE_MILLISECONDS} 毫秒认领：\
             别的身份回别人持有、剩余 {remaining_lease_milliseconds} 毫秒"
        )),
        other @ (ClaimOutcome::Claimed
        | ClaimOutcome::HeldByOther { .. }
        | ClaimOutcome::AlreadyConcluded) => Err(format!(
            "同一身份 {LEASE_COVER_LONG_LEASE_MILLISECONDS} 毫秒认领之后再用 {LEASE_COVER_SHORT_LEASE_MILLISECONDS} 毫秒认领，\
             过 100 毫秒别的身份认领回 {other:?}，该是别人持有、剩余超过 {half_long_lease} 毫秒：到期被帧里更短的租期往前挪了"
        )),
    }
}

/// 认领回到调用方手里之后，服务给的到期盖住调用方从回来那一刻起按整段租期写的那一段（`committer.rs` 文件头「认领」、
/// `reliable_client.rs` 文件头「认领」）。三条输入：丢了回应重发才回来、忙了大半租期才落下、同一身份用更短的租期再认领。
pub(crate) fn cell_claim_lease_covers_the_holder_after_the_answer(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context.break_switches.only_these(&[
        BreakSwitch::ClaimLeaseShrunkOnResend,
        BreakSwitch::ClaimExpiryMovedEarlier,
    ]);
    let lost_answer = lease_after_a_lost_answer(context, &switches)?;
    let long_busy = lease_after_a_long_busy(context, &switches)?;
    let renewal = renewal_never_moves_the_expiry_earlier(context, &switches)?;
    Ok(format!(
        "租期 {LEASE_COVER_LEASE_MILLISECONDS} 毫秒：{lost_answer}；{long_busy}；{renewal}"
    ))
}

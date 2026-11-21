//! 帧长上限的两格：`cell_claim_response_within_frame_limit`（认领回应不越过帧长上限）、`cell_oversize_frame_reported_at_once`（大过帧长上限的一帧当场报协议错）。

use super::*;

/// 认领那一格：持有者身份 255 字节、大批里重复查同一把短键的次数。
const OVERSIZE_CLAIM_REPEATS: usize = 260_000;
/// 认领回应不越过帧长上限那一格，每一帧认领的租期。
const OVERSIZE_CLAIM_LEASE_MILLISECONDS: u32 = 60_000;

fn claim_with_identity(
    client: &mut SimpleClient,
    identity: &[u8],
    keys: &[&[u8]],
) -> Result<ResponseOutcome, String> {
    let borrowed: Vec<(Table, &[u8])> = keys.iter().map(|key| (Table::Conclusions, *key)).collect();
    client.call(|sequence| {
        encode_claim_frame(
            sequence,
            identity,
            OVERSIZE_CLAIM_LEASE_MILLISECONDS,
            &borrowed,
        )
    })
}

pub(crate) fn cell_claim_response_within_frame_limit(
    context: &CellContext,
) -> Result<String, String> {
    let fitting = maximum_response_payload_bytes().saturating_sub(CLAIM_OUTCOME_COUNT_BYTES)
        / held_by_other_outcome_bytes(MAXIMUM_CLIENT_IDENTITY_BYTES);
    if fitting != 253_240 {
        return Err(format!(
            "255 字节身份的「别人持有」一帧回应放得下 {fitting} 个，已知答案是 253240 个（(67108864 − 10 − 4) / 265）"
        ));
    }
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::ClaimResponseUnbudgeted),
        &[],
    )?;
    let mut client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    let holder = vec![b'a'; MAXIMUM_CLIENT_IDENTITY_BYTES];
    let claimed = claim_with_identity(&mut client, &holder, &[b"claim/held".as_slice()])?;
    if claimed != ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]) {
        return Err(format!(
            "255 字节身份认领一把空闲的键该认领到，回的是 {claimed:?}"
        ));
    }
    let mixed = claim_with_identity(
        &mut client,
        b"other",
        &[b"claim/held".as_slice(), b"claim/mixed-free".as_slice()],
    )?;
    let ResponseOutcome::Claims(mixed_outcomes) = &mixed else {
        return Err(format!("混合认领回的不是认领结局：{mixed:?}"));
    };
    let is_mixed_right = matches!(
        mixed_outcomes.as_slice(),
        [ClaimOutcome::HeldByOther { holder_identity, .. }, ClaimOutcome::Claimed] if *holder_identity == holder
    );
    if !is_mixed_right {
        return Err(format!(
            "别人持有的键加一把空闲的键：该回别人持有（带 255 字节身份）、认领到，回的是 {mixed_outcomes:?}"
        ));
    }
    let mut oversize_keys: Vec<&[u8]> = vec![b"claim/oversize-free".as_slice()];
    oversize_keys.extend(std::iter::repeat_n(
        b"claim/held".as_slice(),
        OVERSIZE_CLAIM_REPEATS,
    ));
    let oversize = claim_with_identity(&mut client, b"other", &oversize_keys).map_err(|message| {
        format!("一帧 {} 个键的认领（{OVERSIZE_CLAIM_REPEATS} 个别人持有、身份 255 字节）：{message}", oversize_keys.len())
    })?;
    let ResponseOutcome::Rejected {
        status: RejectionStatus::MalformedPayload,
        message,
    } = &oversize
    else {
        let received = if let ResponseOutcome::Claims(outcomes) = &oversize {
            format!("{} 个结局", outcomes.len())
        } else {
            format!("{oversize:?}")
        };
        return Err(format!(
            "回应会越过帧长上限的认领该整帧回状态 1，回的是 {received}"
        ));
    };
    if !message.contains("超过帧长上限") {
        return Err(format!("整帧拒的说明该写回应会超过帧长上限：{message}"));
    }
    let after = claim_with_identity(&mut client, b"third", &[b"claim/oversize-free".as_slice()])?;
    if after != ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]) {
        return Err(format!(
            "被整帧拒的那一帧里的空闲键不该被认领（不留部分副作用），别的身份再认领该认领到，回的是 {after:?}"
        ));
    }
    Ok(format!(
        "255 字节身份的「别人持有」一帧至多 {fitting} 个；混合认领照常；{} 个键的认领（回应会越过帧长上限）整帧回状态 1、说明写了上限，里面的空闲键没被认领",
        oversize_keys.len()
    ))
}

/// oversize-frame-reported-at-once：容错客户端的重连总时长上限（弄坏开关下重连重发到这么久回「连不上」）。
const OVERSIZE_RECONNECT_TOTAL: Duration = Duration::from_secs(2);

/// 单条超限当场报（`reliable_client.rs`、`client.rs` 文件头）：键长超过帧长上限的一条，容错客户端在重连总时长之内回协议错、
/// 说帧长上限、追加日志里没有这一帧；一问一答的客户端当场回错、说没发，同一条连接随后照常读写。
pub(crate) fn cell_oversize_frame_reported_at_once(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::OversizeFrameSentAndRetried);
    let server = HardeningServer::start(context, BreakSwitches::none(), &[])?;
    let oversize_bytes = usize::try_from(MAXIMUM_FRAME_BODY_BYTES).expect("u32 放得进 usize") + 1;
    let journal_path = server.path().join("oversize.journal");
    let stop = Arc::new(AtomicBool::new(false));
    let settings = bounded_connect_settings(
        server.route(),
        journal_path.clone(),
        &switches,
        &stop,
        OVERSIZE_RECONNECT_TOTAL,
    );
    let started = Instant::now();
    let reliable_outcome = within_deadline(
        CONNECT_CELL_DEADLINE,
        || stop.store(true, Ordering::SeqCst),
        move || -> Result<(), ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            client.submit_write_entries(&[(
                Table::Conclusions,
                vec![b'k'; oversize_bytes],
                bytes("v"),
            )])?;
            client.wait_all_acknowledged()
        },
    )
    .map_err(|message| format!("容错客户端写键长 {oversize_bytes} 字节的一条：{message}"))?;
    let reliable_elapsed = started.elapsed();
    let is_reliable_right = matches!(
        &reliable_outcome,
        Err(ClientFailure::Protocol(message)) if message.contains("帧长上限")
    ) && reliable_elapsed < OVERSIZE_RECONNECT_TOTAL;
    if !is_reliable_right {
        return Err(format!(
            "容错客户端写键长 {oversize_bytes} 字节的一条：{} 毫秒时回 {reliable_outcome:?}（该在 {} 毫秒内回协议错、说帧长上限，不重连）",
            reliable_elapsed.as_millis(),
            OVERSIZE_RECONNECT_TOTAL.as_millis()
        ));
    }
    // 重开一次追加日志、数要重放的写帧；这一句一结束客户端放手，日志的锁跟着放。
    let replayed = ReliableClient::open(plain_client_settings(
        server.route(),
        journal_path,
        &BreakSwitches::none(),
    ))
    .map_err(|failure| format!("重开追加日志：{failure}"))?
    .outstanding_writes();
    if replayed != 0 {
        return Err(format!(
            "超限的那一帧不该记进追加日志，重开之后要重放的写帧有 {replayed} 帧"
        ));
    }
    let mut simple = SimpleClient::connect(&server.route(), switches)?;
    let simple_outcome =
        simple.write(&[(Table::Conclusions, vec![b'k'; oversize_bytes], bytes("v"))]);
    let is_simple_right = matches!(
        &simple_outcome,
        Err(message) if message.contains("帧长上限") && message.contains("没发")
    );
    if !is_simple_right {
        return Err(format!(
            "一问一答的客户端写键长 {oversize_bytes} 字节的一条：回 {simple_outcome:?}（该当场回错、说帧长上限与没发）"
        ));
    }
    let written_after = simple.write(&[entry("oversize/after", "1")])?;
    let read_after = read_value(&server.route(), "oversize/after")?;
    if written_after != 1 || read_after.as_deref() != Some(b"1".as_slice()) {
        return Err(format!(
            "超限那一条报错之后同一条连接再写一条：写了 {written_after} 条、读回 {}（该写 1 条、读回 1）",
            shown_value(read_after.as_deref())
        ));
    }
    Ok(format!(
        "键长 {oversize_bytes} 字节的一条：容错客户端 {} 毫秒回协议错、追加日志里没有这一帧；一问一答的客户端当场回错，同一条连接随后照常读写",
        reliable_elapsed.as_millis()
    ))
}

//! 追加日志的两格：`cell_journal_high_water_survives_compaction`（请求号高水位整理之后还在）、`cell_journal_lock_survives_compaction`（排他锁整理之后还在），连同它们用的检查函数。

use super::*;

pub(crate) fn cell_journal_high_water_survives_compaction(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::JournalCompactionDropsHighWater);
    let server = HardeningServer::start(context, BreakSwitches::none(), &[])?;
    let full = high_water_after_full_acknowledgement(&server, &switches)?;
    let partial = high_water_with_lower_request_pending(&server, &switches)?;
    Ok(format!("{full}；{partial}"))
}

/// 全确认之后整理、重开再写新键：新写要分到没用过的请求号、落库。
fn high_water_after_full_acknowledgement(
    server: &HardeningServer,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let mut settings = plain_client_settings(
        server.route(),
        server.path().join("full-acknowledgement.journal"),
        switches,
    );
    settings.journal_compaction_bytes = 1;
    let request_before_reopen = {
        let mut client =
            ReliableClient::open(settings.clone()).map_err(|failure| failure.to_string())?;
        let (_, request_number) = client
            .submit_write_entries(&[entry("high-water/full/before", "before")])
            .map_err(|failure| failure.to_string())?;
        client
            .wait_all_acknowledged()
            .map_err(|failure| failure.to_string())?;
        if client.journal_statistics().compactions == 0 {
            return Err(
                "整理门槛 1 字节、确认之后该整理过，整理了 0 次：没走到「整理之后重开」的局面"
                    .to_string(),
            );
        }
        request_number
    };
    let mut reopened = ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
    let (operation, request_number) = reopened
        .submit_write_entries(&[entry("high-water/full/after", "after")])
        .map_err(|failure| failure.to_string())?;
    let is_already_applied = wait_write_acknowledgement(&mut reopened, operation)?;
    let stored = read_value(&server.route(), "high-water/full/after")?;
    if request_number <= request_before_reopen
        || is_already_applied
        || stored.as_deref() != Some(b"after".as_slice())
    {
        return Err(format!(
            "全确认之后整理、重开：新写分到请求号 {request_number}（整理之前用到 {request_before_reopen}），服务回「早已生效」{is_already_applied}，\
             库里读回 {}（该是 after）：整理丢了请求号高水位，新写被当成早已生效的重发",
            shown_value(stored.as_deref())
        ));
    }
    Ok(format!(
        "全确认之后整理、重开：新写分到请求号 {request_number}（整理之前用到 {request_before_reopen}），服务照新写落库"
    ))
}

/// 低号没确认、高号确认了之后整理、重开：新写要分到比高号还大的请求号、落库。
fn high_water_with_lower_request_pending(
    server: &HardeningServer,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let journal_path = server.path().join("lower-pending.journal");
    let recovered = Journal::open(
        &journal_path,
        JournalOptions::from_break_switches(1, switches),
    )?;
    let mut journal = recovered.journal;
    let identity = recovered.client_identity;
    let pending_frame = write_frame(&identity, 1, "high-water/partial/pending", "pending");
    let acknowledged_frame = write_frame(
        &identity,
        2,
        "high-water/partial/acknowledged",
        "acknowledged",
    );
    journal.append_write(1, &pending_frame)?;
    journal.append_write(2, &acknowledged_frame)?;
    let mut direct = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    let first_write = send_frame(&mut direct, &acknowledged_frame)?;
    if !matches!(
        first_write,
        ResponseOutcome::WriteAcknowledged {
            is_already_applied: false,
            ..
        }
    ) {
        return Err(format!(
            "请求号 2 头一次写该回做完、没生效过，回的是 {first_write:?}"
        ));
    }
    journal.append_acknowledgement(2, std::iter::once((1, pending_frame.as_slice())))?;
    if journal.statistics().compactions != 1 {
        return Err(format!(
            "确认请求号 2 时该整理一次（门槛 1 字节），整理了 {} 次：没走到「低号没确认、高号确认了」之后整理的局面",
            journal.statistics().compactions
        ));
    }
    drop(journal);
    let mut settings = plain_client_settings(server.route(), journal_path, switches);
    settings.journal_compaction_bytes = 1;
    let mut reopened = ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
    let (operation, request_number) = reopened
        .submit_write_entries(&[entry("high-water/partial/after", "after")])
        .map_err(|failure| failure.to_string())?;
    let is_already_applied = wait_write_acknowledgement(&mut reopened, operation)?;
    reopened
        .wait_all_acknowledged()
        .map_err(|failure| failure.to_string())?;
    let after = read_value(&server.route(), "high-water/partial/after")?;
    let pending = read_value(&server.route(), "high-water/partial/pending")?;
    if request_number <= 2
        || is_already_applied
        || after.as_deref() != Some(b"after".as_slice())
        || pending.as_deref() != Some(b"pending".as_slice())
    {
        return Err(format!(
            "请求号 1 没确认、2 确认了之后整理、重开：新写分到请求号 {request_number}（该大于 2），服务回「早已生效」{is_already_applied}，\
             新键读回 {}（该是 after），重放的请求号 1 读回 {}（该是 pending）",
            shown_value(after.as_deref()),
            shown_value(pending.as_deref())
        ));
    }
    Ok(format!(
        "低号没确认、高号确认了之后整理、重开：重放请求号 1，新写分到请求号 {request_number}，两条都落库"
    ))
}

pub(crate) fn cell_journal_lock_survives_compaction(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::JournalLocksRenamedFile);
    let directory = CellDirectory::create()?;
    let path = directory.path.join("locked.journal");
    let compacting_options = JournalOptions::from_break_switches(1, &switches);
    let keeping_options = JournalOptions::from_break_switches(u64::MAX, &switches);
    let mut holder = Journal::open(&path, compacting_options)?.journal;
    holder.append_write(1, b"holder-frame")?;
    let (opened_sender, opened) = mpsc::channel::<()>();
    let (resume_sender, resume) = mpsc::channel::<()>();
    let racer_path = path.clone();
    let racer = thread::spawn(move || -> Result<String, String> {
        let pause = move || {
            // 主线程收不到也照样往下走：它到点会判这一格红。
            let _ = opened_sender.send(());
            let _ = resume.recv_timeout(Duration::from_secs(30));
        };
        match Journal::open_pausing_before_lock(&racer_path, keeping_options, pause) {
            Err(refusal) => Ok(refusal),
            Ok(mut recovered) => {
                recovered
                    .journal
                    .append_write(2, b"racer-frame")
                    .map_err(|message| format!("第二个打开者拿到锁之后追加出错：{message}"))?;
                Err("第二个打开者卡在开文件与加锁之间、第一个整理之后放行，它拿到了锁、追加了请求号 2".to_string())
            }
        }
    });
    let was_paused = opened.recv_timeout(Duration::from_secs(30)).is_ok();
    let compaction = holder.append_acknowledgement(1, std::iter::empty());
    let _ = resume_sender.send(());
    let racer_outcome = racer
        .join()
        .map_err(|_| "第二个打开者的线程 panic 了".to_string())?;
    compaction?;
    let compactions = holder.statistics().compactions;
    drop(holder);
    if !was_paused || compactions != 1 {
        return Err(format!(
            "第二个打开者停在加锁之前 {was_paused}（该是 true）、第一个整理了 {compactions} 次（该是 1）：没走到要判的交错"
        ));
    }
    let recovered = Journal::open(&path, keeping_options)?;
    let pending: Vec<u64> = recovered
        .unacknowledged
        .iter()
        .map(|write| write.request_number)
        .collect();
    match racer_outcome {
        Ok(refusal) if pending.is_empty() => Ok(format!(
            "第二个打开者卡在开文件与加锁之间、第一个整理（改名换上新文件）之后放行：被同一把锁挡住（{refusal}）"
        )),
        Ok(refusal) => Err(format!(
            "第二个打开者被拒了（{refusal}），可从原路径重开读回没确认的写帧 {pending:?}（该是空的）"
        )),
        Err(why) => Err(format!(
            "{why}；从原路径重开读回没确认的写帧 {pending:?}：它追加的那一帧写进了改名之后没人再读的旧文件"
        )),
    }
}


// ── 攻方腿探针（gpu-oom-card-governance-r2 第 9、10 问），rerun.sh 把这一段接在运行目录那棵树的
//    gpu-verification-server/tests/card_lease_on_a_temporary_library.rs 末尾：用那一份里的 LeaseCase、借的进程与临时判定库。
//    断言写的是本腿推出的结局；断言红就是本腿推错了。

/// 对自己起的借的进程（它自己的进程组）整组发一个信号。
fn probe_signal_the_borrowing_group(borrowing: &BorrowingProcess, signal: &str) -> ExitStatus {
    Command::new(SIGNAL_PROGRAM)
        .args(["-s", signal, "--", &format!("-{}", borrowing.child.id())])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap_or_else(|error| panic!("起不了 {SIGNAL_PROGRAM}：{error}"))
}

/// 第 9 问：借的进程被 SIGSTOP 停住（不是 lease take 停住）：续租进程照 `crate::service_process` 的判法把停着的进程当在跑
/// （只认不在、僵尸、启动刻不同），一直续，过了一整个编进来的认领租期卡键还在它名下；借卡没有上限。
#[test]
fn probe_a_stopped_borrowing_process_keeps_the_cards_past_a_whole_claim_lease() {
    let case = LeaseCase::with_the_cards("probe-stopped-borrower", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let borrowing = case.listed_borrowing_process();
    let borrower_value = borrowing.borrower.variable_value();
    let holder = case.holder_identity_of(&borrowing.borrower);
    let (status, _output, error) = case.run_lease(Some(&borrower_value), &["take"]);
    assert_eq!(status.code(), Some(0), "借到：{error}");
    let mut lease = TakenLease {
        case: &case,
        borrower: borrower_value.clone(),
        is_returned: false,
    };
    let stopped = probe_signal_the_borrowing_group(&borrowing, STOP_SIGNAL);
    assert!(stopped.success(), "停得了借的进程：{stopped}");
    let waited = compiled_claim_lease() + Duration::from_secs(10);
    std::thread::sleep(waited);
    let card_key_holder = case.holder_of(case.keys.card_key());
    let request_key_holder = case.holder_of(case.keys.request_key());
    let continued = probe_signal_the_borrowing_group(&borrowing, CONTINUE_SIGNAL);
    println!(
        "CARD_LEASE_ATTACK_PROBE name=stopped_borrower_past_a_lease waited_seconds={} card_key_held_by_the_stopped_borrower={} request_key_held_by_the_stopped_borrower={}",
        waited.as_secs(),
        card_key_holder.as_deref() == Some(holder.as_slice()),
        request_key_holder.as_deref() == Some(holder.as_slice())
    );
    assert!(continued.success(), "接着跑得了借的进程：{continued}");
    assert_eq!(
        card_key_holder,
        Some(holder.clone()),
        "本腿推的：借的进程停着，续租进程照续，过了一整个租期卡键仍在它名下；不在就是本腿推错了"
    );
    lease.return_now();
}

/// 第 10 问：持着卡键的身份不以 `card-lease/` 起头的，`lease list` 一律写 `held_by=service`：
/// 取活循环那一条测试里借的一方用的身份（`card-lease-test-borrower`，经判定库直接认领、不经 lease take）持着卡键时，
/// lease list 把借方说成服务。
#[test]
fn probe_lease_list_calls_a_borrower_whose_identity_lacks_the_card_lease_prefix_the_service() {
    let case = LeaseCase::with_the_cards("probe-list-direct-borrower", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let mut borrower = case.library.open_client();
    assert_eq!(
        claim_a_card_lease_key_as(
            &mut borrower,
            BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
            case.keys.card_key(),
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了卡键"),
        CardLeaseKeyAnswer::Claimed,
        "借的一方直接认领卡键"
    );
    let (status, output, error) = case.run_lease(None, &["list"]);
    assert_eq!(status.code(), Some(0), "lease list 退 0：{error}");
    let held_line = output
        .lines()
        .find(|line| line.starts_with(&format!("{CARD_LEASE_HELD_LINE_PREFIX} host={THIS_HOST} ")))
        .unwrap_or_else(|| panic!("没有这一台的卡键那一行：{output}"))
        .to_string();
    println!("CARD_LEASE_ATTACK_PROBE name=direct_borrower_listed line={held_line}");
    assert!(
        held_line.contains(" held_by=service "),
        "本腿推的：借方以不带 card-lease/ 的身份持着卡键，lease list 写成服务持着；不是就是本腿推错了：{held_line}"
    );
}

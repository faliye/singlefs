//! P5：自动触发的发布点落地之后，RefFS 今天的允许集合在合法的恢复态上判不判红（冻结副本的 RefFS 原样调用，不改它）。
//! 允许集合今天是 {S_j : j = lo 或 j 恰是某次显式发布或回退的末尾，lo ≤ j ≤ hi}（RefFS `AllowedSet` 的文档）。
//! 三种交法逐一判：
//! - 照今天的翻译交：每一步内容改动后面跟一个显式发布点（harness `reference_filesystem_translation.rs` 的 `publish_with_content`），
//!   攒批之后入口返回成功而那一批还没发布；
//! - 只交内容改动：攒批的改动不跟显式发布点，自动发布点 RefFS 不知道；
//! - 放宽成 [lo, hi] 里任意 S_j（这里自己算，不是 RefFS 的代码）；
//! - 再加按单元的键序前缀（这里自己算）：一次请求被触发切开时恢复读出前几个单元是新的、后面是旧的。
//! 由用户决定的两样（崩溃之前做了几次覆盖写、自动发布落在哪一次之后）放开扫，只固定起点（建第一个文件并发布）。
use singlefs_reference_filesystem::{
    CrashNotAccepted, FileContent, Operation, OperationOutcome, ReferenceFileSystem, VisibleState,
};

const UNIT_BYTES: usize = 4;
const UNITS_PER_FILE: usize = 3;

fn content_filled_with(byte: u8) -> Vec<u8> {
    vec![byte; UNIT_BYTES * UNITS_PER_FILE]
}

fn file(bytes: &[u8]) -> VisibleState {
    VisibleState::OneFile(FileContent(bytes.to_vec()))
}

fn submit(reference: &mut ReferenceFileSystem, operation: Operation, outcome: OperationOutcome) {
    reference
        .submit(operation, outcome)
        .expect("这段历史里的每个操作 RefFS 都收");
}

/// 起点：建第一个文件并发布（照今天的翻译交）。
fn after_the_first_file(first: &[u8]) -> ReferenceFileSystem {
    let mut reference = ReferenceFileSystem::on_a_freshly_formatted_pool();
    submit(
        &mut reference,
        Operation::CreateTheFirstFile(FileContent(first.to_vec())),
        OperationOutcome::Succeeded,
    );
    submit(
        &mut reference,
        Operation::ExplicitPublish,
        OperationOutcome::Succeeded,
    );
    reference
}

/// RefFS 对一次「所有操作都返回之后才崩、恢复读出 `recovered`」判什么：绿、红，或判不了。
fn judge_of_the_reference_filesystem(reference: &ReferenceFileSystem, recovered: &VisibleState) -> String {
    match reference.allowed_set_for_a_crash_in(reference.crash_after_every_operation_returned()) {
        Ok(allowed) if allowed.contains(recovered) => "green".to_string(),
        Ok(allowed) => format!("red(allowed_states={})", allowed.allowed.len()),
        Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion { .. }) => {
            "not_accepted(operation_after_a_failure_after_writing)".to_string()
        }
        Err(other) => format!("not_accepted({other:?})"),
    }
}

fn main() {
    let first = content_filled_with(b'X');
    let overwrite_bytes = [b'A', b'B', b'C', b'D'];

    // 自动发布点落在两次覆盖写之间：覆盖写 k 次（1..=4），自动发布落在第 p 次之后（0..=k），恢复读出 S_p。
    let mut red_counts = [0_u32; 3];
    let mut cases = 0_u32;
    for overwrites in 1..=overwrite_bytes.len() {
        for published_after in 0..=overwrites {
            cases += 1;
            let states: Vec<Vec<u8>> = std::iter::once(first.clone())
                .chain(overwrite_bytes[..overwrites].iter().map(|byte| content_filled_with(*byte)))
                .collect();
            let recovered = file(&states[published_after]);
            let mut translated_today = after_the_first_file(&first);
            let mut content_only = after_the_first_file(&first);
            for content in &states[1..] {
                submit(&mut translated_today, Operation::OverwriteTheFile(FileContent(content.clone())), OperationOutcome::Succeeded);
                submit(&mut translated_today, Operation::ExplicitPublish, OperationOutcome::Succeeded);
                submit(&mut content_only, Operation::OverwriteTheFile(FileContent(content.clone())), OperationOutcome::Succeeded);
            }
            let any_state_between = if states.iter().any(|state| file(state) == recovered) { "green" } else { "red" };
            let verdicts = [
                judge_of_the_reference_filesystem(&translated_today, &recovered),
                judge_of_the_reference_filesystem(&content_only, &recovered),
                any_state_between.to_string(),
            ];
            for (count, verdict) in red_counts.iter_mut().zip(&verdicts) {
                if verdict.starts_with("red") {
                    *count += 1;
                }
            }
            println!(
                "name=auto_publish_between_changes overwrites={overwrites} auto_publish_after={published_after} translated_today={} content_only={} any_state_between_lo_and_hi={}",
                verdicts[0], verdicts[1], verdicts[2]
            );
        }
    }
    println!(
        "name=auto_publish_between_changes_summary cases={cases} red_translated_today={} red_content_only={} red_any_state_between_lo_and_hi={}",
        red_counts[0], red_counts[1], red_counts[2]
    );

    // 一次覆盖写整份三个单元，触发在第 m 个单元之后关掉成员资格（D16 已定项 5 的取号口径），恢复读出前 m 个单元新、其余旧。
    let new_content = content_filled_with(b'Y');
    let mut split_red_counts = [0_u32; 4];
    for units_published in 0..=UNITS_PER_FILE {
        let mut recovered_bytes = new_content[..units_published * UNIT_BYTES].to_vec();
        recovered_bytes.extend_from_slice(&first[units_published * UNIT_BYTES..]);
        let recovered = file(&recovered_bytes);
        let mut translated_today = after_the_first_file(&first);
        submit(&mut translated_today, Operation::OverwriteTheFile(FileContent(new_content.clone())), OperationOutcome::Succeeded);
        submit(&mut translated_today, Operation::ExplicitPublish, OperationOutcome::Succeeded);
        let mut content_only = after_the_first_file(&first);
        submit(&mut content_only, Operation::OverwriteTheFile(FileContent(new_content.clone())), OperationOutcome::Succeeded);
        let any_state_between = if recovered == file(&first) || recovered == file(&new_content) { "green" } else { "red" };
        let unit_key_prefix = if (0..=UNITS_PER_FILE).any(|prefix| {
            let mut candidate = new_content[..prefix * UNIT_BYTES].to_vec();
            candidate.extend_from_slice(&first[prefix * UNIT_BYTES..]);
            file(&candidate) == recovered
        }) { "green" } else { "red" };
        let verdicts = [
            judge_of_the_reference_filesystem(&translated_today, &recovered),
            judge_of_the_reference_filesystem(&content_only, &recovered),
            any_state_between.to_string(),
            unit_key_prefix.to_string(),
        ];
        for (count, verdict) in split_red_counts.iter_mut().zip(&verdicts) {
            if verdict.starts_with("red") {
                *count += 1;
            }
        }
        println!(
            "name=one_request_split_by_the_trigger units_published={units_published} translated_today={} content_only={} any_state_between_lo_and_hi={} unit_key_prefix={}",
            verdicts[0], verdicts[1], verdicts[2], verdicts[3]
        );
    }
    println!(
        "name=one_request_split_by_the_trigger_summary cases={} red_translated_today={} red_content_only={} red_any_state_between_lo_and_hi={} red_unit_key_prefix={}",
        UNITS_PER_FILE + 1, split_red_counts[0], split_red_counts[1], split_red_counts[2], split_red_counts[3]
    );

    // 自动发布写了之后报错（D16 已定项 17：可以出现也可以不出现），攒批照旧收下一次改动，再崩。
    // 报错只能挂在一个操作上：挂在一个显式发布点上（写了之后报错），或不交（只交内容改动）。
    let failed_content = content_filled_with(b'A');
    let later_content = content_filled_with(b'B');
    let mut failure_on_a_publish_point = after_the_first_file(&first);
    submit(&mut failure_on_a_publish_point, Operation::OverwriteTheFile(FileContent(failed_content.clone())), OperationOutcome::Succeeded);
    submit(&mut failure_on_a_publish_point, Operation::ExplicitPublish, OperationOutcome::FailedAfterWriting);
    submit(&mut failure_on_a_publish_point, Operation::OverwriteTheFile(FileContent(later_content.clone())), OperationOutcome::Succeeded);
    let mut failure_not_submitted = after_the_first_file(&first);
    submit(&mut failure_not_submitted, Operation::OverwriteTheFile(FileContent(failed_content.clone())), OperationOutcome::Succeeded);
    submit(&mut failure_not_submitted, Operation::OverwriteTheFile(FileContent(later_content)), OperationOutcome::Succeeded);
    for (recovered_name, recovered) in [("first", file(&first)), ("failed_publish_landed", file(&failed_content))] {
        println!(
            "name=accept_after_a_failed_auto_publish recovered={recovered_name} failure_on_a_publish_point={} failure_not_submitted={}",
            judge_of_the_reference_filesystem(&failure_on_a_publish_point, &recovered),
            judge_of_the_reference_filesystem(&failure_not_submitted, &recovered)
        );
    }
}

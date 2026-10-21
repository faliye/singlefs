#!/usr/bin/env python3
"""攻方腿 m3-close-c693-r3 T2：在仓副本里给 F2 的五个退回点加只记不改的探针（今天的行为一字不变）。

用法：python3 -I apply_probe.py --repo <仓副本根>
每次 F2 退回分支被走到，往环境变量 C693_PROBE_LOG 点名的文件追加一行（没设不写）：
  C693F2 site=<recovery|read_only|writable|judge_read_recovery_fields> thread=<测试名> failure=<所选那一版的失败> \
    draft_whitelist_retreats=<草稿白名单补丁在这一处会不会退> judge=<F2 判法交回 Some/None> retreated=<今天实际退到哪一版/none>
draft_whitelist_retreats 照草稿 research/prompts/m3-close-c693-r2-sonnet-model/v2/apply_variant.py --variant whitelist 三处的条件逐字搬：
  恢复与可写挂载认 UnitUnreadable / MappingMiss / MappingStillUnreadable（可写挂载只认包在 MountError::Recovery 里的），
  只读挂载另认 OpenPoolForReadFailure::UnitMalformed。判官那一处草稿没改（不分失败种类），这里照恢复那一处的分法记。
退出码：0；2 原文命中不是恰好一次。
"""
import sys

args = dict(zip(sys.argv[1::2], sys.argv[2::2]))
repo = args.get("--repo")
if not repo:
    print(__doc__); sys.exit(2)
core = repo + "/crates/singlefs-core/src/"
harness = repo + "/crates/singlefs-harness/src/"


def sub(path, old, new):
    text = open(path).read()
    if text.count(old) != 1:
        print(f"原文命中 {text.count(old)} 次：{path} | {old[:80]}"); sys.exit(2)
    open(path, "w").write(text.replace(old, new))


HELPERS = '''

/// 探针（攻方腿 m3-close-c693-r3 草稿副本，只记不改）：草稿白名单的分法。
#[must_use]
pub fn c693_draft_whitelist_retreats(failure: &RecoveryFailure) -> bool {
    matches!(
        failure,
        RecoveryFailure::UnitUnreadable { .. }
            | RecoveryFailure::MappingMiss { .. }
            | RecoveryFailure::MappingStillUnreadable { .. }
    )
}

/// 探针：往 C693_PROBE_LOG 追加一行。
pub fn c693_probe(site: &str, failure: &str, draft_whitelist_retreats: bool, judge: &str, retreated: &str) {
    let Ok(path) = std::env::var("C693_PROBE_LOG") else { return };
    let thread = std::thread::current().name().unwrap_or("unnamed").to_string();
    let line = format!(
        "C693F2 site={site} thread={thread} failure={} draft_whitelist_retreats={draft_whitelist_retreats} judge={judge} retreated={retreated}\\n",
        failure.replace('\\n', " ")
    );
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}
'''

# 恢复
sub(core + "recovery.rs", '''        Err(failure) => {
            let version_fallen_back_to_walked = match policy {
                JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
                    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                        reader,
                        &system_configuration,
                        &root,
                        &effective_root,
                        &scan_journal(reader, &system_configuration),
                    )
                }
                JournalPolicy::Ignore => None,
            }
            .and_then(''', '''        Err(failure) => {
            let c693_judge = match policy {
                JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
                    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                        reader,
                        &system_configuration,
                        &root,
                        &effective_root,
                        &scan_journal(reader, &system_configuration),
                    )
                }
                JournalPolicy::Ignore => None,
            };
            let c693_judge_text = format!("{:?}", c693_judge.map(|r| (r.instance, r.checkpoint_txg)));
            let version_fallen_back_to_walked = c693_judge
            .and_then(''')
sub(core + "recovery.rs", '''            match version_fallen_back_to_walked {
                Some((version_fallen_back_to, content)) => {''', '''            c693_probe(
                "recovery",
                &format!("{failure:?}"),
                c693_draft_whitelist_retreats(&failure),
                &c693_judge_text,
                &version_fallen_back_to_walked
                    .as_ref()
                    .map_or("none".to_string(), |(v, _)| format!("{:?}", (v.instance, v.checkpoint_txg))),
            );
            match version_fallen_back_to_walked {
                Some((version_fallen_back_to, content)) => {''')
open(core + "recovery.rs", "a").write(HELPERS)

# 只读挂载
sub(core + "mounted_read.rs", '''            Err(failure_of_the_selected_version) => {
                let version_fallen_back_to_opened =
                newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                    reader,
                    &system_configuration,
                    &chosen_root,
                    &effective_root,
                    &records,
                )
                .and_then(''', '''            Err(failure_of_the_selected_version) => {
                let c693_judge =
                newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                    reader,
                    &system_configuration,
                    &chosen_root,
                    &effective_root,
                    &records,
                );
                let c693_judge_text = format!("{:?}", c693_judge.map(|r| (r.instance, r.checkpoint_txg)));
                let version_fallen_back_to_opened = c693_judge
                .and_then(''')
sub(core + "mounted_read.rs", '''                match version_fallen_back_to_opened {
                    Some((mounted, version_fallen_back_to)) => {''', '''                crate::recovery::c693_probe(
                    "read_only",
                    &format!("{failure_of_the_selected_version:?}"),
                    match &failure_of_the_selected_version {
                        OpenPoolForReadFailure::Walk(inner) => crate::recovery::c693_draft_whitelist_retreats(inner),
                        OpenPoolForReadFailure::UnitMalformed { .. } => true,
                        _ => false,
                    },
                    &c693_judge_text,
                    &version_fallen_back_to_opened
                        .as_ref()
                        .map_or("none".to_string(), |(_, v)| format!("{:?}", (v.instance, v.checkpoint_txg))),
                );
                match version_fallen_back_to_opened {
                    Some((mounted, version_fallen_back_to)) => {''')

# 可写挂载
W_CLASS = '''matches!(&failure_of_the_selected_version, MountError::Recovery(inner) if crate::recovery::c693_draft_whitelist_retreats(inner))'''
sub(core + "mount.rs", '''                        &effective_root,
                        &records,
                    )
                else {
                    return Err(failure_of_the_selected_version);
                };
                let record_standing_for_the_version_fallen_back_to =''', '''                        &effective_root,
                        &records,
                    )
                else {
                    crate::recovery::c693_probe("writable", &format!("{failure_of_the_selected_version:?}"), %s, "None", "none");
                    return Err(failure_of_the_selected_version);
                };
                let record_standing_for_the_version_fallen_back_to =''' % W_CLASS)
sub(core + "mount.rs", '''                    Ok(previous) => {
                        // F1 乙按退到的那一版的水位再判一次（`F1_AFTER_F2`）：读阶段判 F1 时水位是退之前的所选那一版。''', '''                    Ok(previous) => {
                        crate::recovery::c693_probe("writable", &format!("{failure_of_the_selected_version:?}"), %s, &format!("{:?}", Some((version_fallen_back_to.instance, version_fallen_back_to.checkpoint_txg))), &format!("{:?}", (version_fallen_back_to.instance, version_fallen_back_to.checkpoint_txg)));
                        // F1 乙按退到的那一版的水位再判一次（`F1_AFTER_F2`）：读阶段判 F1 时水位是退之前的所选那一版。''' % W_CLASS)
sub(core + "mount.rs", '''                    Err(_failure_of_the_version_fallen_back_to) => {
                        return Err(failure_of_the_selected_version)''', '''                    Err(_failure_of_the_version_fallen_back_to) => {
                        crate::recovery::c693_probe("writable", &format!("{failure_of_the_selected_version:?}"), %s, &format!("{:?}", Some((version_fallen_back_to.instance, version_fallen_back_to.checkpoint_txg))), "none");
                        return Err(failure_of_the_selected_version)''' % W_CLASS)

# harness 的挂载透明判官（read_recovery_fields）
sub(harness + "mount_transparency.rs", '''    let walks_to_a_file_or_no_file = |root: &RootRecord| {
        let mut mapping_fallbacks = 0usize;
        walk_to_file(reader, root, &mut mapping_fallbacks).is_ok()
    };''', '''    let walks_to_a_file_or_no_file = |root: &RootRecord| {
        let mut mapping_fallbacks = 0usize;
        let walked = walk_to_file(reader, root, &mut mapping_fallbacks);
        if let Err(failure) = &walked {
            if (root.instance, root.checkpoint_txg) == (replayed_root.instance, replayed_root.checkpoint_txg) {
                singlefs_core::recovery::c693_probe(
                    "judge_read_recovery_fields",
                    &format!("{failure:?}"),
                    singlefs_core::recovery::c693_draft_whitelist_retreats(failure),
                    "see_recovery",
                    "see_recovery",
                );
            }
        }
        walked.is_ok()
    };''')
print("name=probe_applied")

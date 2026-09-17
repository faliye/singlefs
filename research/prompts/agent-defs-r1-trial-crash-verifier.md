# 崩溃一致性验证员（crash-verifier）试跑报告

派发角色：`.claude/agents/crash-verifier.md`（依据 `.claude/rules/implementation-workflow.md` 第 3 步、`.claude/singlefs-ai-sop/rules/test-discipline.md`）。

## 输入核对

- 主 agent 给的改动范围：`git diff --stat -- crates litmus` 末行 ` 14 files changed, 1008 insertions(+), 368 deletions(-)`
- 我开跑前（2026-09-17 01:19 UTC 左右）自己取的同一条命令，结果**逐字相同**：
  ```
  $ git diff --stat -- crates litmus | tail -1
   14 files changed, 1008 insertions(+), 368 deletions(-)
  ```
- 我跑完四个阶段之后（2026-09-17 02:09 UTC）再取一次同一条命令，**已经不同**：
  ```
  $ git diff --stat -- crates litmus | tail -1
   18 files changed, 1677 insertions(+), 406 deletions(-)
  ```
  另外 `git status --porcelain -- crates litmus` 在结束时多出 5 个未跟踪新文件（`crates/singlefs-core/src/instance_table.rs`、
  `crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/tests/second_transaction_step_{three_second_instance,four_rollback,five_reuse}.rs`），
  这些从一开始就不出现在 `git diff --stat` 里（该命令本来就不统计未跟踪文件）。详细分析见「试跑观察」第 1 条。

## 开跑前环境检查（定义第 1 步）

```
$ ps aux | grep -E "cargo|qemu-system|herd7|gate.sh" | grep -v grep
fy5090   1402348  0.0  0.0   5704  2256 ?        SN   01:17   0:00 timeout 120 cargo test --release --bin e142-first-txn-dry-run
fy5090   1402349  0.0  0.0 245276 29412 ?        SNl  01:17   0:00 /home/fy5090/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo test --release --bin e142-first-txn-dry-run
```
这是另一会话在 `research/e7-index-bench`（独立 workspace、独立 `target/`）跑变异复跑，与我要跑的 `crates/` workspace 不共用
`target/` 目录，不构成编译锁冲突。主 agent 已在派发提示的「这次给的例外」里明确允许我照常跑（加 `nice -n 19`）。
除此之外没有看到 `qemu-system`、`herd7`、`gate.sh` 在跑。

环境自检（`bash .claude/scripts/env.sh`）：只报 gawk / sha256sum / timeout / cargo / rustc / git / dmsetup 通过，
shellcheck 缺失（非阻塞）；**它完全不检查 herd7 / KVM / qemu**（`grep -n "herd7\|kvm\|qemu" .claude/scripts/env.sh .claude/singlefs-ai-sop/scripts/env.sh` 零命中）。
我这次的四个阶段都没有落到「环境」判定（详见下表，均退出码 0），所以定义第 5 条没有被实际触发；
但这条本身指向一个可能的定义缺口，写在「试跑观察」第 4 条。

我自己直接查的环境事实（现查命令与结果）：
- herd7：`which herd7` 在裸 shell 里找不到；`opam list | grep herd` → `herdtools7 7.58`；
  `eval "$(opam env --root=$HOME/.opam --set-root)"` 之后 `command -v herd7` → `/home/fy5090/.opam/default/bin/herd7`。
  `.claude/scripts/lkmm.sh` 第 283–286 行自己会做这个 `opam env` 激活，所以直接跑 57 号阶段时能找到 herd7（见下表，确实通过）。
- KVM：`ls -la /dev/kvm` 存在，`groups` 显示当前用户在 `kvm` 组。
- `which qemu-system-x86_64` → `/usr/bin/qemu-system-x86_64`。
- `rustup target list --installed` 含 `x86_64-unknown-linux-musl`。
- `bash research/scripts/vm-kernel.sh --check` → 退出码 0，`/boot/vmlinuz-6.17.0-lockdep`。

## 阶段表（`.claude/gate.d/stage-owners.tsv` 登记给 crash-verifier 的四个阶段，逐个跑，一次只跑一个）

取号命令：
```
$ awk -F'\t' -v me=crash-verifier '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv
54-layer0-replay.sh
55-qemu-first-transaction.sh
57-lkmm.sh
59-crates-mutation-replay.sh
```

| 阶段 | 退出码 | 开始（UTC） | 结束（UTC） | 耗时 |
|---|---|---|---|---|
| 54-layer0-replay.sh | 0 | 01:19:58 | 02:06:46 | 46 分 48 秒 |
| 55-qemu-first-transaction.sh | 0 | 02:07:19 | 02:07:38 | 19 秒 |
| 57-lkmm.sh | 0 | 02:07:58 | 02:07:58 | < 1 秒 |
| 59-crates-mutation-replay.sh | 0 | 02:08:17 | 02:08:55 | 38 秒 |

没有一个阶段落到退出码 77（本次未跑）或非 0；四个都是「本次跑了且通过」。

### 54-layer0-replay.sh（层 0 崩溃点重放）—— 原样抄的 ✓ 行与计数行

命令：`nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`（内部跑两条 `cargo test --release`）。

第一条流（第一个事务）跑到一半时，同一台机器上出现了**另一个会话独立起的同名阶段**
（`ps` 看到第二份 `.claude/gate.d/54-layer0-replay.sh` 进程，工作目录指向另一个会话的 scratchpad），
两边各自跑各自的 `cargo test`，互不共享 `target/`（都是本仓 `crates/` workspace 根的同一个 `target/`，
实际上是共享的，但因为都已经编译完只在跑测试二进制，没有观察到「Blocking waiting for file lock」提示，
`nice -n 19` 下 32 核机器负载 `uptime` 显示 3.6–5.8，未见 CPU 饥饿迹象）。这段并发不算进我的阶段耗时里，
因为它没有造成可观测的等待——如实记录在这里，供交叉核对。

原样输出：
```
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle）：states=789555 closed_form=789555 exhaustive=true violations=0 root_persisted_states=4 no_file=262158 file_read=527397 failed=0 journal_differing=9 verification_ran=18 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 first_violation=none
EXITCODE=0
```
两条 `states=closed_form`、`violations=0`、`exhaustive=true` 都对上；第二条流的 `789555` 与
`.claude/kb/first-txn-layout.md:401`、`.claude/kb/milestone/02-second-txn.md:58` 里现查到的「到 C（26 段、789555）」逐字相符
（这两份 kb 文件在开跑时的 `git status` 里已经是 `M`，即工作区正在被改的文件之一）。

### 55-qemu-first-transaction.sh（QEMU 真设备）—— 原样抄的 ✓ 行

命令：`nice -n 19 bash .claude/gate.d/55-qemu-first-transaction.sh`。前置检查（qemu-system-x86_64、/dev/kvm、
vm-kernel.sh --check、E142 产物里的 `back_chain`）都在脚本内部自动过了，没有触发任何 `fail`。

原样输出：
```
  ✓ QEMU 真设备上的第一个事务：3 次虚机跑、18 项检查全过；direct 设备侧逐项对得上、两个对照都红在该红的地方
EXITCODE=0
```
脚本自带的两个阳性对照（`skip-first-transaction-barrier` 应该判红、`page-cache` 应该判红）都按脚本的判据核过了
（否则会在 `fail` 分支里打印 `✗` 与 `→`，这次没有出现）。

### 57-lkmm.sh（herd7 内存序）—— 原样抄的 ✓ 行

命令：`nice -n 19 bash .claude/gate.d/57-lkmm.sh`。`litmus/` 下没有 `.lkmm-static-only` 标记，走的是完整 herd7 路径。

原样输出：
```
══ LKMM（herd7） ══
  ✓ 内核树 /home/fy5090/linux-bug-fix/linux
  ✓ herd7  7.58, Rev: exported

  ✓ litmus/commit-publish-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/commit-publish.litmus  Never（符合声明）
  ✓ litmus/first-txn-journal-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-journal-implies-units.litmus  Never（符合声明）
  ✓ litmus/first-txn-root-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-root-implies-units.litmus  Never（符合声明）

  ✓ LKMM 通过（3 条 Never：每条都有内容对得上的对照组，2 条绑到代码、1 条声明不对应代码；共 3 条 Sometimes）
  ✓ 内存序：litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符
EXITCODE=0
```
我另外现查了 3 条 `Never` litmus 锚点指向的函数在当前源码里还在不在（这是 lkmm.sh 第 4 条要求「锚点后的代码今天还在」
的一半，判得对不对要人看，我只核了「存在」这一半）：
```
$ grep -n "fn publish_first_file\|fn choose_root\|fn scan_journal\|fn replay_journal" crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/recovery.rs
crates/singlefs-core/src/recovery.rs:285:pub fn choose_root(...)
crates/singlefs-core/src/recovery.rs:564:pub fn scan_journal(
crates/singlefs-core/src/recovery.rs:599:pub fn replay_journal(
crates/singlefs-core/src/transaction.rs:815:pub fn publish_first_file<Device: BlockDevice>(
```
四个锚点函数都在。

### 59-crates-mutation-replay.sh（crates 变异表复跑）—— 原样抄的 ✓ 行

命令：`nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh`。

⚠️ **重要**：我在开跑早期（约 01:24 UTC）用 `Read` 工具看过一次 `crates/mutations.tsv`，当时是 25 条变异
（`grep -vc '^#\|^$' crates/mutations.tsv` = 25）。等我实际跑到 59 号（02:08 UTC）时，该文件已被另一会话在
01:28:25 UTC 改过（`stat -c '%y' crates/mutations.tsv` = `2026-09-17 01:28:25`），变成 45 行、40 条变异。
脚本本身没有问题——它每次都读**当前**文件内容，跑出的结果对应的是 02:08 那一刻的 40 条表，不是我最初看到的 25 条。
这只是提醒：报告里任何「我读过一次文件长什么样」的记忆，在这个多会话仓里随时可能已经过期，要以**实际执行那一刻**的读数为准。

原样输出（40 条全部「红了」，逐条列出）：
```
  ✓ 逐盘核退回只核总数：placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even 红了
  ✓ 同盘槽号不核唯一：two_records_for_the_same_slot_on_the_same_device_are_rejected 红了
  ✓ 空闲不随分配减（I-5.2 要红）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ✓ 释放时清位图（立即复用）：released_placements_are_not_handed_out_again_before_reclaim_exists 红了
  ✓ 分配记录树装不下不判：repeated_overwrites_report_a_full_allocation_node_instead_of_panicking 红了
  ✓ 释放退回按提示：release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released 红了
  ✓ 映射查到 key 就算（落点指错不核记录）：release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking 红了
  ✓ 失败的发布不退回分配器：publish_running_out_of_space_midway_leaves_the_allocator_as_it_was 红了
  ✓ 内容超长不判：content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched 红了
  ✓ oracle 只比 txg 不比实例：landing_on_the_lower_instance_of_the_same_txg_is_a_violation 红了
  ✓ oracle 只按 txg 找版本：the_same_txg_from_two_instances_are_two_versions 红了
  ✓ oracle 放过没版本的更新根：newer_root_without_any_version_reporting_no_file_is_violation 红了
  ✓ Ignore 那一遍恢复不过 oracle：targeted_controls_on_the_second_publish_go_red_where_they_should 红了
  ✓ 步 1 变异：不释放旧落点：release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue 红了
  ✓ 步 1 变异：defer 行写 0：release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue 红了
  ✓ 步 1 变异：事务号不加一：overwrite_publishes_the_second_version_through_the_same_commit_shape 红了
  ✓ 步 1 变异：改动计数留 1：overwrite_publishes_the_second_version_through_the_same_commit_shape 红了
  ✓ 步 2 变异：已分配行不随分配更新（I-3.1 要红）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ✓ 步 2 变异：忘了改写分配记录：release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue 红了
  ✓ 步 3：不写行（实例表照抄）：remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version 红了
  ✓ 步 3：暖机只推一次：damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device 红了
  ✓ 步 3：前缀跨实例边界（把别的实例的记录也接上）：layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape 红了
  ✓ 步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）：stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row 红了
  ✓ 步 3：I-3.8 判定恒真：checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root 红了
  ✓ 步 3：重建分配器时不把已释放的放进 defer 队列：remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version 红了
  ✓ 步 4：回退行不带回退位：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：只写被退回的实例那一行、不写中间实例行：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：回退之后 jsn 接可读链末尾而不是 R_old 那条之后（C340 的 P2）：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：影子账一个槽都不隔离：without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit 红了
  ✓ 步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）：rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused 红了
  ✓ 步 4：前缀第五条不判（回退行的 W 不封顶）：the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water 红了
  ✓ 步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：记录核对器把被后来记录合法覆盖的记录槽当成记录流有洞：every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims 红了
  ✓ 步 5：回收不看释放代（复用窗口置 0）：remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version 红了
  ✓ 步 5：抬 F 的上限不看第 4 新的非空根：raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused 红了
  ✓ 步 5：抬 F 的空发布只推一次（F 只落在一块盘上）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：复用时追加记录而不改写（同盘同槽两条）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：checker 的候选集不按 F 收（F 之下的根照走）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：F_生效 取各盘 F 最大值的最大值而不是最小值：one_device_carrying_the_floor_alone_does_not_take_effect_on_remount 红了
  ✓ 步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条：torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg 红了
  ✓ crates 变异表复跑：40 条变异各自红在点名的测试上（原文都恰好命中一次）
EXITCODE=0
```
40 条锚点全部恰好命中一次（脚本自己的前置检查，没有触发「锚点腐化」分支）；40 条改坏之后点名的测试全部判红、
还原之后基线仍绿（脚本收尾的最后一行 `✓ crates 变异表复跑：40 条变异各自红在点名的测试上`）。

## 没做什么

- 没修任何一处红——这次四个阶段全绿，没有红需要处理，也就没有「判红是不是这一轮改动造成的」这一步。
- 全绿只说明这四道阶段各自的证据要求被满足；层 0（54 号）覆盖到哪几条流、哪些崩溃场景没进来，
  看 54 号脚本头部注释（第 5–10 行）与它引的 `.claude/kb/decisions/13-验证路线.md` 已定项 4，不由我外推。
  我这次现查到的边界：54 号跑的第二条流是「到 C」（789555 个状态），`.claude/kb/milestone/02-second-txn.md:58`
  写着完整脚本已经做到「到 E」（54 段、2104413 个状态，含回退 D、抬 F、E），但那条**没有**被 54 号脚本当前调用的
  `second_transaction_step_zero_layer0` 全量用例覆盖（该用例的 ignored 全量测试标的是到 C 那条，到 D/E 只由
  「只跑 prepare」的用例钉住段序列，不是层 0 全量重放）——这句话是我读 kb 与脚本源码得到的**观察**，
  层 0 的准确覆盖范围仍以 54 号脚本自己的头部与 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`
  源码为准，我不代它下结论。
- 没有跑 `gate.sh` 全量、没有跑 `--staged` 模式，也没有跑登记给别的 agent 的任何阶段（10/15/20…33/40…52/56/58/60/61/62/63 等）；
  按 `.claude/agent-common.md`「提交前的整轮门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑」，这些不归我。
- 没有修 `.claude/scripts/fetch-deps.sh` 里的路径 bug（`source "$(dirname .../lib.sh"` 指向不存在的
  `.claude/scripts/lib.sh`，实际该指 `.claude/singlefs-ai-sop/scripts/lib.sh`）——只读不改是我的写范围，这条记在
  「试跑观察」里交主 agent 判断要不要另开工单。
- 没有对「改动范围在我跑的这 50 分钟里从 14 files/1008+/368- 长到 18 files/1677+/406-，外加 5 个新未跟踪文件」
  这件事做任何判断（是不是该等改动停下再跑、这次的四个绿是不是对应同一个快照）——这是「试跑观察」要报的现象，
  怎么处置由主 agent 或 `gate-triage` 定。
- 没有验证语义正确性，只验证了证据要求满足（`show-me-test.md`「门禁能证明什么，不能证明什么」）。

## 试跑观察

这一节是主 agent 特别要求的，写定义（`.claude/agents/crash-verifier.md`）里哪些步骤不清楚、做不下去、与实际对不上，
以及每个阶段的墙钟耗时（后者已经在上面的表里）。

### 1. 「改动范围」在我这一轮（约 50 分钟）里持续漂移，而定义没有说清怎么处理

定义给的输入是「`git diff --stat -- crates litmus` 原样」，一个**单次快照**。但 54 号阶段本身要跑 47 分钟
（见下条），这 47 分钟里本仓另有会话在持续往 `crates/` 写代码（我现查到的证据：`crates/mutations.tsv` 在
01:28:25 UTC 被改过，从 25 条变成 40 条；`git diff --stat -- crates litmus` 从开跑时的
`14 files changed, 1008 insertions(+), 368 deletions(-)` 变成收尾时的
`18 files changed, 1677 insertions(+), 406 deletions(-)`；`git status --porcelain -- crates litmus` 收尾时
多出 5 个未跟踪新文件，其中 `crates/singlefs-core/src/mount.rs` 从我开跑前的 `git status` 系统提示里就已经存在，
说明**未跟踪的新文件从一开始就不在 `git diff --stat` 的统计范围内**——这个命令本来就只统计已跟踪文件的修改）。

后果：我这四个阶段的绿，分别对应的是**各自实际执行那一刻**的工作区状态，而不是同一个统一快照。
54 号（跑了 47 分钟）大概率横跨了好几次别的会话的保存动作；55/57/59 号跑得很快（19 秒、<1 秒、38 秒），
更接近各自开跑那一刻的快照。`gate.sh`（整轮门禁的顶层脚本）对这个问题有专门的机制——开跑与收尾各算一次
`worktree_fingerprint`（`.claude/singlefs-ai-sop/scripts/gate.sh:113-114,374-380`），对不上就判红；但我按
`.claude/agent-common.md`「门禁」一节的指示，是**直接跑单个 `.claude/gate.d/<文件>`**，不经过 `gate.sh`，
所以这层保护对我不生效。**定义没有告诉我在这种情况下该怎么办**：是要我自己在开跑前后各记一次
`worktree_fingerprint` 并报告有没有变，还是这件事本来就该留给 `gate-triage` 在最终整轮门禁时兜底。
我这次采取的处置是：**如实记录两次 diff-stat 的差异，不代任何一方下结论**。

### 2. 54 号阶段的第二条流耗时远超定义里暗示的量级，且比第一条流不成比例地慢

54 号脚本自己的头部注释（第 7 行）写「全量 262165 个状态在 debug 下要几分钟，所以…这里在 release 下跑它」，
暗示 release 下应该比「debug 几分钟」更快。我实测：第一条流（262165 个状态）release 下约 3 分钟 CPU 时间；
第二条流（789555 个状态，约 3 倍状态数、含多版本 oracle）却跑了约 44 分钟 CPU 时间——不是线性的 3 倍，
是接近 15 倍。这不排除是「多版本 oracle 对每个状态的验证开销本来就更贵」这一合理原因（第二条流的
`verification_ran=18` 对第一条流的 `verification_ran=6`，且 `file_read=527397` 远高于第一条流的 `file_read=7`，
说明第二条流绝大多数崩溃状态都落在「文件已发布、需要读回内容比对」这个更贵的分支），但这不是我能判断的——
我只能如实报告：**这一阶段单次跑完要 47 分钟**，比定义或 kb 里任何地方暗示的都长得多，派发这个角色的人
如果按「几分钟」预估排期会明显超时。这件事我在中途也重新起了三次内部等待循环（因为单次 Bash 调用有
超时上限，10 分钟一到就被系统挪进后台），不算是失败，但值得记录：**这个阶段的耗时量级需要更新到定义或
kb 里，不然下一个执行者会以为它卡住了**。

### 3. 定义第 5 条「缺 herd7、缺 KVM…判环境，抄 env.sh 里对应的那几行」——env.sh 里没有对应的那几行

我现查了 `.claude/scripts/env.sh`（转发到 `.claude/singlefs-ai-sop/scripts/env.sh`）的完整输出与源码，
它只检查 gawk / sha256sum / timeout / cargo / rustc / git / dmsetup / shellcheck，**一个字都没提 herd7、KVM 或 qemu**。
这次我的四个阶段没有触发环境判定（herd7 装了、KVM 能读写、qemu 在），所以没有实际影响这次的报告，
但如果下次真的缺 herd7 或 KVM，我会发现「按定义要抄的那几行」根本不存在——env.sh 对这三样是沉默的。
这些检查实际分散在 `.claude/gate.d/55-qemu-first-transaction.sh`（第 22–24 行，qemu-system-x86_64 / /dev/kvm /
vm-kernel.sh --check）与 `.claude/scripts/lkmm.sh`（第 283–319 行，herd7 / 内核树）内部，不在 env.sh。
建议：要么把这三样加进 env.sh，要么把定义第 5 条改成「抄触发失败的那个阶段脚本自己打印的 `howto` 行」，
不再指向 env.sh。

### 4. `fetch-deps.sh --check` 本身跑不起来（与我的阶段无直接关系，但会误导下一个想手动排查 herd7 的人）

我在核实 herd7 环境时尝试 `bash .claude/scripts/fetch-deps.sh --check`，它在第 10 行
`source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"` 处失败——`.claude/scripts/lib.sh` 不存在
（`.claude/scripts/lkmm.sh` 用的是 `.claude/singlefs-ai-sop/scripts/lib.sh`，路径不一样）。
报的错是一串 `command not found`（`head1`、`ok`、`warn` 未定义），不是「lib.sh 缺失」这句人话，
症状指向命令本身而不是路径错误，容易把人带偏。这不在我的写范围内，只记录，不修。

### 5. 定义第 2 步「一次只跑一个」在多会话仓里管不住别的会话

我自己严格做到了一次只跑一个（54 → 55 → 57 → 59 顺序，逐个等完成再开下一个）。但 `ps` 里能看到
**另一个会话独立起了同一个 `54-layer0-replay.sh`**（约在我的 54 号跑到一半时开始），这不是我能控制的，
定义也没有提供「发现别的会话在跑同一个阶段时该怎么办」的指引（继续跑、等它、还是报告给主 agent）。
这次我选择了继续跑（两边互不阻塞，也没有产生可观测的等待），如实记录在阶段 54 的详情里。

### 6. 试跑本身的元问题：这次跑的代码是不是「稳定」的

因为改动范围在漂移，我没有办法证明「我在 54 号跑到 46 分钟时读到的 `crates/` 源码」与
「我在 59 号开跑时读到的 `crates/` 源码」是同一份——两者之间隔了另一会话至少一次已确认的写入
（`crates/mutations.tsv` 01:28:25 那次，以及从 diff-stat 差异反推出的、发生在 `allocator.rs`、
`recovery.rs`、`transaction.rs` 等文件上的更多次写入）。这四个阶段的绿**没有被这件事污染**——
每个阶段测的都是它自己开跑那一刻的真实源码，不存在「测了 A 却汇报 B」的问题——但**四个绿放在一起
不构成「同一份 diff 的证据链」**，这一点在报告表格之外必须单独说清楚，否则读报告的人会默认这是
针对一次静止的 diff 快照跑出的结果。

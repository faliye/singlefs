# 崩溃一致性验证：m2-anchor-fix，仅门禁 59 号（crates 变异表整表复跑）

范围：`research/prompts/m2-anchor-fix-crash-verifier-scope.txt`。与上一趟
（`research/prompts/m2-wave2-crash-verifier-report.md`）相比，仅
`crates/mutations.tsv` 第 32、37、41、42、69、107 行有改动（实现员报告：
`research/prompts/m2-anchor-fix-implementer-report.md`）。按派发要求，本轮
只跑 54、55、57 号之外的门禁 59 号（`.claude/gate.d/59-crates-mutation-replay.sh`）。

开跑前 `ps -o pid,args -u "$(id -u)" | grep -Ei 'qemu-system|vm-bench|e152-file-system-benchmark|fio|cargo|gate.sh|gate.d'`
无命中，无别的 gate.sh / cargo / 性能负载在跑，直接起。

## 指纹（开跑与结束时各一次，与起阶段同一条命令）

| 时刻 | HEAD | `git diff HEAD -- crates litmus \| sha256sum` | 未跟踪文件内容 sha256sum |
|---|---|---|---|
| 开跑 2026-09-18T08:50:08Z | `502ba80e8065351d968d061b8dd28698e3463c8d` | `ac71e3be38b55250ec41d69a46a659a45f747103e36199040e352418e090dcda` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`（空，无未跟踪文件命中 crates/litmus） |
| 结束 2026-09-18T08:52:25Z | `502ba80e8065351d968d061b8dd28698e3463c8d` | `ac71e3be38b55250ec41d69a46a659a45f747103e36199040e352418e090dcda` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

三项开跑与结束完全一致，这一趟的绿对应同一份源码，跑的过程中没有别的会话动过 `crates`/`litmus`。

## 阶段表

| 阶段 | 退出码 | 耗时 | 判定 |
|---|---|---|---|
| 59-crates-mutation-replay.sh | 0 | 08:50:08Z–08:52:25Z（约 2 分 17 秒），期间没有出现 `Blocking waiting for file lock`（已用 `grep -c` 核过，命中 0） | 全绿，见下 |

日志：`/tmp/claude-1000/crash-m2-anchor-fix/59-crates-mutation-replay.log`（125 行，末行 `exit=0`，退出码只认这一行）。

## 预扫过没过

`59-crates-mutation-replay.sh` 在真正改坏代码之前先做锚点预扫：对表里每一条变异，
在**当前源码**里数原文命中次数，命中次数不是 1 就判「锚点腐化」、直接打印
`  ✗ 变异表的锚点腐化：` 并 `exit 1`，整道跳过后面的改坏—跑测—复原。

日志里没有一处 `✗`、没有 `锚点腐化`（两条 `grep` 都命中 0 行），且日志跑到了最后一行
`  ✓ crates 变异表复跑：123 条变异各自红在点名的测试上（原文都恰好命中一次）`、`exit=0`。
脚本里预扫失败会在第一步就 `exit 1`、后面 123 条实测一条都不会跑；能跑到这条汇总行、
123 条全部报出结果，就说明预扫这一步过了，123 条原文全部在当前源码里恰好命中 1 次。
**预扫：过。**

## 123 条的判读：红在点名测试上 / 没红 / 没跑到

脚本对每一条变异分三类判：`run.returncode != 0` 且点名测试匹配 `... FAILED` 记「红了」（打 `✓`）；
点名测试确实跑过但没匹配到 `FAILED`（含退出码为 0，或点名之外的测试红了）记「没红」；
连点名测试的 `test ... ...` 起始行都没出现，记「没跑到」，两者都进 `failures` 列表、汇总打
`  ✗ 有变异没红：` 后跟每条明细，任何一条非空整道就红、`exit 1`，不会再打印末尾的汇总 `✓` 行。

现查计数（对日志原样统计，不手数）：

```
$ grep -c '^  ✓' 59-crates-mutation-replay.log
124
$ grep -c '没红\|没跑到' 59-crates-mutation-replay.log
2
$ grep -n '没红\|没跑到' 59-crates-mutation-replay.log
93:  ✓ 步 6：I-3.8 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）：every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims 红了
94:  ✓ 步 6：I-2.1 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）：every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims 红了
$ tail -1 59-crates-mutation-replay.log
exit=0
```

那两处「没跑到」命中的是**变异名文本里描述的 bug 场景**（「判定没跑到」是这条变异要模拟的缺陷），
不是脚本自己判失败的「没跑到」——两行都以 `✓` 起头、以「… 红了」收尾，判读上属于「红在点名测试上」。
日志里没有任何一行 `✗`、没有 `  ✗ 有变异没红：`，末行 `exit=0`，且末尾出现汇总行
`  ✓ crates 变异表复跑：123 条变异各自红在点名的测试上（原文都恰好命中一次）`——
按脚本逻辑，`failures` 非空时这条汇总行根本不会打印、也不会 `exit 0`；能看到它就等于
`failures == []`。

**计数：123 条红在点名测试上，0 条没红，0 条没跑到。** 没红与没跑到本该逐条列出的清单为空，
这里如实记「无」，不是漏查——上面贴的是脚本判定这两类的代码路径与日志里唯一疑似命中的
两行（判读后不算数的理由已给出）。

## 第 121–128 行（表尾 8 条，原样结果行）

```
  ✓ Z1-d：复用时新记录罩住的已回收记录不删（I-5.4 要红）：reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds 红了
  ✓ Z1-d：新落点罩住没回收的记录不断言：placement_over_record_that_was_not_reclaimed_is_asserted_on_the_records 红了
  ✓ I-5.4：checker 判不出分配记录重叠：an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant 红了
  ✓ I-5.4：只判最新根那棵账：an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant 红了
  ✓ Z1-a：取号之前不算实例表：writable_mounts_fill_the_instance_table_page_and_the_next_one_is_refused_before_acquisition 红了
  ✓ Z1-a：实例表准入漏算链指针：writable_mounts_fill_the_instance_table_page_and_the_next_one_is_refused_before_acquisition 红了
  ✓ Z1-a：实例表准入把正好写满一片也拒掉：mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it 红了
  ✓ Z1-a：实例表准入按每次挂载只写一行算：mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it 红了
```

（`crates/mutations.tsv` 第 121–128 行依次对应上面 8 行，逐条用变异名 `grep -F` 从日志取出，
每条恰好命中 1 次；第 121–128 行本身就是全表的最后 8 行，全表共 128 行、前 5 行是表头注释、
123 行数据行，`121–128` 落在数据区最末。）

## 第 32、37、41、42、69、107 行（这一轮改动过的行，原样结果行）

```
  ✓ 步 4：只写被退回的实例那一行、不写中间实例行：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 5：复用时追加记录而不改写（同盘同槽两条）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：checker 的候选集不按 F 收（F 之下的根照走）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 3：树表 0 条的一版上要写行时，拒绝挪到取号之后（超级块已写进新号才返回）：writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again 红了
  ✓ 增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉）：a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired 红了
```

依次对应表的第 32、37、41、42、69、107 行（现查行号：
`awk -F'\t' 'NR==32||NR==37||NR==41||NR==42||NR==69||NR==107{print NR"\t"$1"\t"$6}' crates/mutations.tsv`，
逐条名字 `grep -F -- "  ✓ <名字>："` 从日志取出，每条恰好命中 1 次）。6 条全部「红了」。

## 起止时间（UTC）

- 开跑：2026-09-18T08:50:08Z（起阶段那条 Bash 命令里的 `date -u`，与指纹同一条命令）
- 结束：2026-09-18T08:52:25Z（收到后台完成通知之后现查的 `date -u`；后台任务由系统标记
  `completed, exit code 0`，即起后台命令那条 Bash 自身的退出码，仅作旁证——真正认的是
  日志文件末行 `exit=0`）

## 没做什么

- 只跑了派发指定的门禁 59 号；54、55、57 号按要求本轮不跑，`herd7`/KVM/QEMU 环境齐不齐
  没有核查，也没有跑 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64`。
- 没有修任何一处红——这一轮 59 号本身全绿，没有红需要处理。
- 不判 59 号覆盖到哪些流、哪些没进来；层 0 覆盖范围看 54 号头部，不由本报告外推。
- 没有跑 `gate.sh` 全量，只跑登记给 crash-verifier 的单一阶段。
- 没有查这次改动本身（`crates/mutations.tsv` 第 32、37、41、42、69、107 行）在语义上改得对不对——
  只核实预扫通过、123 条变异逐条红在各自点名的测试上，语义判断不在本轮职责内。

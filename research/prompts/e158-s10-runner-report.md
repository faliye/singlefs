# E158 session s10 交回报告

## 一、op1 第三种变体（岔路单第 1 行）

**已完成**。新增 `FaultedOperationKind::MountWritableThenRaiseFloorTo(CheckpointTxg)`、
`floor_targets_between`（纯函数，(F, 上限] 半开区间）、`raise_floor_targets_for`（先不注入地做一次
`mount_writable` 探测 F 与上限，用完即弃）、`attempt_mount_writable_then_raise_floor`（第一步不注入
打开会话，受注入的一步是 `raise_rollback_floor` 本身），接进 `run_ledger_fault_family` 主循环。

按跑前登记 Q1-1c 原文「抬 F 那一处单独一行」，新增 `raise_floor_pairs`/`raise_floor_trigger_count`/
`raise_floor_error_members`/`raise_floor_by_aspect_severity`/`raise_floor_history_nodes_
without_room` 五个独立字段与三类输出行（`q1_1a_raise_floor_trigger_summary`/`q1_1a_raise_floor_
by_aspect_severity`/`q1_1c_raise_floor_error_member`），**不并入**既有的 `pairs`/`trigger_count`/
`error_members`/`by_aspect_severity`。

**没有做**「把 raise_rollback_floor 里若干次空发布各自的中间根追加进 node.timeline」——现查发现这一
步不需要：`attempt_faulted_operation` 整体是「试一次看结局，用完即弃」语义（与既有的 `MountRollbackTo`
分支同理，那个分支交回的中间根同样从不追加进任何时间线），`RaisedFloor::abandoned_roots_unreadable`
是从盘上现读实例表 + 现行根的候选集算出来的（`isolate_slots_referenced_only_by_abandoned_roots`，
`crates/singlefs-core/src/mount.rs` 第 1027 行起，与 `mount_writable`/`mount_rollback` 那两种 op1
共用同一条计数管道），不依赖装置自己的内存时间线。省掉了 session s9 估计的这部分工程量。

**产物读数（G0）**：
```
E7RESULT name=q1_1a_raise_floor_trigger_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) pairs=96 trigger_count=32 history_nodes_without_room=26
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=both_copies pairs=16 trigger=16
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk0_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk1_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=both_copies pairs=16 trigger=16
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk0_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk1_only pairs=16 trigger=0
```
（`research/results/e158-root-choice-repair-2026-09-25-q1-g0-today.out`）。G0 与 S16 逐字节相同
（`research/results/e158-root-choice-repair-2026-09-25-q1-s16.out`），S4 不同
（`pairs=132 trigger_count=44 history_nodes_without_room=25`，`research/results/e158-root-choice-
repair-2026-09-25-q1-s4.out`，几何不同不要求相同）。

**单测**：新增 4 条（`floor_targets_between_starts_strictly_above_the_current_floor`、
`floor_targets_between_includes_the_ceiling_itself`、`floor_targets_between_is_empty_when_the_
ceiling_does_not_exceed_the_current_floor`、`mount_writable_then_raise_floor_reaches_the_
injected_fault`），39→43。命令与原样输出：

```
$ cargo test --release -p singlefs-harness --bin e158_root_choice_repair
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
```

`cargo clippy --release -p singlefs-harness --bin e158_root_choice_repair --all-features
--all-targets` 对这个 bin 清零（新增 `#[allow(clippy::enum_variant_names, ...)]`）。

**变异**：`crates/mutations.tsv` 追加 3 行（`floor_targets_between` 下界 off-by-one、上界排他性、
`raise_rollback_floor` 调用点传错 floor 值），逐条手工改坏→点名测试红→还原→复绿，核过（详见「三、
变异手工验证」）。

## 二、`replay.sh` 全量重出与新产物

`bash research/scripts/replay.sh E158`（14 行）：**字节一致 4／对不上 10／跑不了 0**。

- 1 行（`driver_e158_q2_1_g0`）是 session s4 遗留、driver 自己的注释已说明不再当依据引用，本来就
  该对不上，不新存。
- 4 行字节一致：`driver_e158_q2_1_pc2`、`driver_e158_q2_1_g0_session_s5`（行 2「够判」的证据链不
  受这一刻并行线改动影响）、`driver_e158_q2_1_hc1`、`driver_e158_q2_1_hc1_lower_bound`。
- 9 行对不上，按今天日期另存新产物，`replay.sh` 登记表已改指向新文件、旧文件原样留着：
  `research/results/e158-root-choice-repair-2026-09-25-segment1-rerun.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q3-1-g0.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q3-1-s16.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q3-1-small-ring.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q3-1-s4.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q1-g0-today.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q1-s16.out`、
  `research/results/e158-root-choice-repair-2026-09-25-q1-s4.out`。

**没有做第二次确认性复跑**：新存的 9 份文件是这一次真实运行的原始输出，逐字节复跑要再等两个
`q2-1-g0` 全穷举跑一遍（约 1～2.5 小时），crates/ 这一刻仍可能继续变，投入产出比低，续不续跑交主
agent 定。

## 三、变异手工验证（3 条新行）

1. `floor_targets_between` 下界改成不严格大于：改 `(current_floor.0 + 1)` → `(current_floor.0)`，
   `cargo test ... floor_targets_between_starts_strictly_above_the_current_floor` 红
   （`left: [CheckpointTxg(2), CheckpointTxg(3), CheckpointTxg(4)] right: [CheckpointTxg(3),
   CheckpointTxg(4)]`）→还原→复绿。
2. `floor_targets_between` 上界改成不含上限本身：改 `..=` → `..`，
   `floor_targets_between_includes_the_ceiling_itself` 红
   （「上限本身也是一个合法目标，得到 [CheckpointTxg(3)]」）→还原→复绿。
3. `raise_rollback_floor` 调用点目标 floor 改成远超上限：`new_floor` → `CheckpointTxg(new_floor.0
   + 1_000_000)`，`mount_writable_then_raise_floor_reaches_the_injected_fault` 红
   （「探测出的 floor 目标不注入任何故障应当成功，得到 Some("RollbackFloorAboveCeiling")」）→还原→
   复绿。

三条改坏与还原之后都用 `diff` 核对逐字节恢复到改动前。`bash .claude/gate.d/33-mutation-tables.sh`
复跑：
```
✓ 147 个实验二进制都有成形的变异表，1631 条变异的原文各命中源码一次；crates/mutations.tsv 703 条
  的原文各命中源码一次
```

**额外发现并修复**：`rustfmt --edition 2021` 单独格式化 `e158_root_choice_repair.rs`（只这一个文
件，不跑 `cargo fmt -p singlefs-harness`——那会连带格式化同一个 crate 里另一条并行线还没提交的文件）
之后，`crates/mutations.tsv` 第 565 行（session s9 的 `mount_writable_trajectory` 那条）锚点因为
重排漂移，33 号从绿变红；改法不变，只把锚点从两行改成 rustfmt 重排之后的一行，手工验证：改坏
（`||`→`&&`）→该条测试红→还原→复绿；33 号复跑绿（703 条）。

## 四、`crates/` 快照三次对比与岔路单第 3 行的重要发现

**协调者点名事项**：主 agent 知会另一条并行线（「实二六」）在改 `mount.rs`/`allocator.rs`/
`allocation_record_tree.rs`（抬 F、挂载读、分配记录树根层）。按要求取了两次快照：

- 重出产物之前：`research/results/e158-root-choice-repair-2026-09-25-s10-crates-sha256.out`
  （`snapshot_time_jst=2026-09-25 10:22:11`、`git_status_crates_lines=84`）。
- 交回前：`research/results/e158-root-choice-repair-2026-09-25-s10-crates-sha256-final.out`
  （`snapshot_time_jst=2026-09-25 11:50:20`、`git_status_crates_lines=85`）。

两次对比：`mount.rs`/`allocator.rs`/`allocation_record_tree.rs` 等 10 个源文件哈希不同（另有
`mutations.tsv` 与若干测试文件，不影响 e158 二进制的编译图）。产物重出中途又取了一次快照，与交回
前那次相比，`mount.rs`/`allocator.rs`/`allocation_record_tree.rs` 没有再变——说明这批变化在产物重
出的中途就已经稳定，降低了「同一轮产物内部互相不一致」的风险，但不能完全排除（14 个驱动一次并发
派发，`cargo` 构建时机与源码变化时机谁先谁后现查不到更细粒度）。

**⚠️ 重要发现——可能动到岔路单第 3 行「候选 3 = 今天」这个前提**：Q3-1（H3×Φ3 违例枚举）在 G0/S16/
小环三点，`pairs_with_any_violation` 从旧产物的 213 变成 0（`cold_recover_with_fault_failed`/
`mount_writable_with_fault_failed` 两点仍是 0，不是新增报错，是判定本身不再违例）；S4 点除了同一项
从 1355 变成 0，`cold_recover_with_fault_failed`/`mount_writable_with_fault_failed` 还从 0 变成
239——同一批构造里，239 个从「成功但违例」变成「调用直接报错」，失效形态本身变了。

C332（回退实例两个根都读不出时回退被撤销） 正文钉的「2 个故障就能撤销回退」在今天这份 `crates/`
（含另一条并行线还没提交的改动）上现在测不出来了。**这可能意味着**：① F2（岔路单第 3 行的失败条
款，「C332 正文的『2 个故障』在 H3 上不成立」）现在触发了；② 或者「候选 3 = 今天的 crates/」这个
跑前登记的核心前提，在这一刻的 `crates/` 状态下已经不成立（那条并行线可能已经实现了类似候选 1/2
的机制，或者只是这一刻的中间状态还没稳定）。**本段不越权下结论、不改任何判据或臂定义**，交主 agent
判断：① 是否要重新核对 F2；② 是否要等那条并行线落定、提交之后再复核 Q3-1；③ 是否要在跑前登记里为
这个新发现的现象登记一条新的岔路或修订。

其余漂移（与本段代码改动无关，方向与 session s9 报告一致，重复坐实）：`driver_e158` 的 `pc3` 一行从
`recover_verdict=pass recover_root=Some((1, 6))` 变成 `recover_verdict=fail recover_root=
Some((0, 0))`；`q1_2_tree_table_crates_has_a_path`（`verify_named_units=false`）从「`did_not_
reach_target=108`+`reached_target_with_matching_pointer=81`」变成「`did_not_reach_target=189`」；
`pc1_a` 的 `device_write_bytes` 从 157184 变 353792；`q2_2a_g0` 的 `AllocationRecordTreeNode` 每
次发布 `write_calls`/`written_bytes` 全面上涨（如 `write_calls=2`→`10`）。

## 五、doc-lint 第 497 行附近的位置指代——现查没有找到

`bash .claude/scripts/doc-lint.sh`（全仓，非 verbose）对 158 这份实验页给出 **0 处违规**；第 498
行「详见 `.claude/kb/experiments/158-择根与修复四岔路.md`「2026-09-24（续，session s9）」一节」指
向的标题在第 759 行确实存在，是写明文件名与小节标题的有效引用，不是「见上文」「本页」那类被判据词
表抓的指代。派发点名的问题在这一刻的正文里没有找到，大概率是主 agent 派发时看到的是 session s9 那
两处自指还没修的中间状态——s9 自己的交回报告写明「交回前已经把『见『它答不了的』岔路 2 那一条』与
『详见本页』两处自指改成写死文件名与小节标题、复跑确认过 0 处」。**不强行改一处现查看不出毛病的文
字**，如实记这一条现查结论，交主 agent 核对（也许主 agent 手头的版本更旧，或者点名的是另一处我没
找到的位置）。

写 kb 实验页新增内容时顺带触发了 4 处新的 doc-lint 违规（自己新写的句子里的「以前是」历史陈述式、
「见上」位置指代、两处裸引用 `G0`/`E158` 没带简称），已在写的过程中逐条改完（用「G0（主几何点）」
「E158（择根与修复四岔路）」等登记简称、把「见上」改写成点名具体小节标题），复跑 `bash .claude/
scripts/doc-lint.sh` 现在**全仓通过**（`✓ 文档铁律检查通过（检查 485，跳过 0）`，含此前已经红着
的 `156-alloc-basis四条岔路的代价数.md`——那份文件不是本段改的，现查它是在本段工作期间被另一条线
自己修好的）。

## 六、门禁自查（归属表登记给执行员的阶段）

`awk -F'\t' -v me="experiment-runner" ... .claude/gate.d/stage-owners.tsv` 给出 13 个阶段：
27/33/34/40/52/69/75/80/85/86/88/96/99。逐个跑过，**10 项绿**：27、33（703 条）、34、52、75、85、
86、88（84 行）、96、99。

**3 项红，全部现查确认与本段无关**：

- **40 号**（实验产物有没有写回）：剩 7 份未点名产物——3 份 `e156-alloc-basis-counts-2026-09-22-
  stage{1,2,3}.out`、4 份 e158 旧文件（`2026-09-23-segment1-tests.out`、`2026-09-24-q3-1-
  {s16,s4,small-ring}.out`）。session s9 已经记过这 4 份 e158 旧文件「不在的不修」，本段没有再动
  它们；本段新增的全部 09-25 产物已在 kb 实验页点名（第一次跑 40 号时漏了 2 份，因为写的时候手工
  换行把文件名切成两截、`grep -F` 按行匹配找不到，已改成每个文件名单独一行，复跑确认全部点到）。
- **69 号**（禁止拿 /tmp 当依据）：剩 2 处，均确认不是本段引入——① `research/prompts/m2-final-
  code-r3-main-verification.md:16`（`git status --short` 显示 `??` 未跟踪，属另一条并行线，本段
  没有写过这个文件）；② `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` 改
  动之后没有对应新产物（这是另一条正在跑的实验线，与 e158 无关，`git status --short` 显示我没有
  碰过这个文件）。
- **80 号**（每个实验二进制都要有绝对值断言）：`crates/singlefs-harness/src/bin/e142_first_
  transaction_write_dump.rs` 一条断言都没有——`git status --short` 确认这是 `??` 未跟踪文件，本段
  两次跑 80 号之间才出现（第一次跑时不存在），是另一条并行线新增的文件，本段没有碰过它。

## 七、岔路表（第 1 行、第 3 行的现查影响）

| 岔路单行 | 已够判 | 还差什么 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1（C393） | ①③（对条文的判断）仍待主 agent；量本身：Q1-1a/b/c、Q1-4 主体已够（session s3-s9）；op1 三种都已实现 | 小环、`G_默认` 两个几何点（S16/S4 已补）；候选 (b) 分配记录树「反推占用集合」的「走全」版本仍差实例表这一项结构差集（session s9 已定位根因）；op1 第三种变体的「持续」故障轨迹（当前实现只做了单次「试一次看结局」，没有像 `mount_writable_trajectory` 那样对第三种 op1 做持续/瞬时分离——`raise_rollback_floor` 本身不产生「后续挂载」的自然延续点，这一项待主 agent 判断是否需要） | 小环/`G_默认` 历史上与 G0/S16 逐字节相同，外推大概率不翻面，需真跑一遍才算数（未跑）；op1 三种变体的数据本身不太可能反转第 1 行已有的判断方向 |
| 2（C331） | 「今天/甲-txg/丙 与 乙丁三臂、崩溃档 0、任一种打中」这一切面够判（session s9，本段复跑确认 `driver_e158_q2_1_g0_session_s5` 字节一致，不受这一刻并行线影响） | H2-R、H2c、n2=2、Q2-2b 仍是「够判后未跑」 | 与本段无关，状态未变 |
| 3（C332） | Q3-1 的比例数值本身已经算出，但**「候选 3 = 今天」这个前提本身可能已经不成立**（本段新发现，见「四」） | 需要主 agent 判断：① 是否重新核对 F2；② 是否等另一条并行线落定提交后再复核；③ 是否需要为这个新发现登记一条修订或新岔路 | **可能翻面**——如果确认这一刻的 `crates/` 已经不再表现出 C332 描述的行为，候选 3「维持现状、登记双故障不保」这一句本身要么不再准确，要么这一刻的 `crates/` 状态不该被当作候选 3 的代表，这是本段最需要主 agent 关注的一格 |
| 4（C334） | 前置未满足（`crates/` 里仍没有挂载内实例切换与失败表两步判别子） | 同上，未变 | 与本段无关 |

## 八、没做什么

- 没判这个实验的结论能不能推翻或确立决策（是推论，走三方，不是这一轮的事）。
- 没跑门禁全量（`gate.sh` 整轮归 `gate-triage`）；15 号与 87 号不归我；只跑了归属表登记给我的 13
  个阶段（第「六」节）。
- 没提交。
- 没有做 `replay.sh` 的第二次确认性复跑（理由见「二」）。
- 岔路单第 1 行：小环、`G_默认` 两个几何点仍未跑；op1 第三种变体的持续故障轨迹仍未做。
- 岔路单第 2 行：H2-R、H2c、n2=2、Q2-2b（够判后未跑，session s9 遗留）。
- 岔路单第 3 行：新发现的「候选 3 前提可能不成立」这件事没有进一步深挖根因（不越权改判据，也不去
  读另一条并行线还没提交的代码细节做推断）。
- 岔路单第 4 行：前置条件未满足，未跑（`crates/` 里没有实例切换与失败表判别子）。
- 门禁 40/69/80 三项残留红：全部现查确认与本段无关（另一条或多条并行线的未提交改动），详见「六」。

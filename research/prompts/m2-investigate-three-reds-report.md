# 调查：合入后验证一剩下的三条红（fsync_drop 两条、warm_up_counter 的 c366）

写于 2026-09-27，UTC 12:1x–12:4x（JST 21:1x–21:4x）。只查、不修；主工作区一个字没动，所有跑都在草稿目录的副本上。
每条 cargo 都经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4`，加 `nice -n 19`。原样日志在 `/tmp/claude-1000/investigate-three-reds/logs/`。

## 一、结论

1. 三条红都出在 **C554 乙**（`m2-impl-c554-yi` 那一件，补丁 `/tmp/claude-1000/impl-c554-yi/patch/crates.patch`，sha256 `440fa87c…67c4`），
   不是 A3b、善后一、I-9.16、乙-配置续，也不是合入后验证一自己的三处 src。
   - 定位是今天 `crates/singlefs-core/src/mount.rs` 的两行（行号取自 12:13:54Z 的主工作区副本）。两条 fsync_drop 红在第 376 行
     `} => self.witness.witnessed_journal_counter > selected_version_last_record_counter,`，
     c366 红在第 384 行 `WitnessedCounterComparison::Undecidable => true,`。
     判真之后，拒绝在第 4088 行 `return Err(MountError::NewerStateStillUnreadableAfterOneReread(` 发出；
     这是读阶段（第 4147 行调用），排在 `establish_instance`（第 4203 行调用）里的逐盘核（第 3509 行）前面。
     r2 快照里同样这两行在第 316、324 行，r1 加乙补丁的副本里在第 304、312 行。
2. 第 ④ 问：
   - fsync_drop 两条是**断言钉的成员过时了，不是行为回归**。这两格照样拒、照样在取号之前拒，只是报的成员从逐盘核的
     `WritableMountRefusedByDevicesWithoutTheSelectedVersion` 换成了乙的 `NewerStateStillUnreadableAfterOneReread`。
     按 D23 第 387 行乙的定义，两格都该判真（见证 5 > 末条 4、见证 4 > 末条 3）。去掉乙这一判之后，
     这两条测试的其余断言（所选那一版、jsn、点名的盘、盘上逐字节不变）在今天的代码上全过（第四节变体 B）。
   - c366 **不是钉值问题**：同一盘面上，可写挂载从「做成」变成了「拒」。按 D23 第 387 行字面「两条都读不出按真」，拒是对的；
     但测试依据的 D23 第 421 行（C366，用户 2026-09-23 定「前缀末计数器取环里读得出的最大那一个」）要求这一格挂得上。
     这一格就是欠账 C579（`.claude/kb/checks-owed.md` 第 491 行，「条款没写要不要改这一判，交主 agent」）。
     所以不是代码写错了，是两条定案在这一格上冲突，要改代码还是改用例由主 agent 或用户定。
     去掉「Undecidable 按真」之后，c366 的其余断言（txg 5/6/7 对 jsn 4/5/6 等）在今天的代码上全过（第四节变体 A）。
3. 推翻条件（第五节，每条都真造过一次）：r1 快照上这三条会红；r1 加乙补丁之后有任一条不红；
   在今天的代码上只改第 376 行，fsync 两条仍红，或者 c366 变绿；只改第 384 行，c366 仍红，或者 fsync 两条变绿。
   四格实测都与预测相符，所以这些推翻现象都没出现。

## 二、第 ① 问：各红在哪一行、断言原文与实际值

测试文件在 r1、r2、主工作区三处的差别只有 `use` 路径（`crash::` 改成 `memory_pool::`，拆 checker-tier 带的），用例本体没变
（`diff -u` 原样见第三节）。

**fsync_drop 两条**，都红在 `tests/second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs:1556:9`。
这一行是帮手 `refusal_by_devices_without_the_selected_version` 的 `let … else { panic!(…) }`，断言原文（第 1557 行）：
`"{what}：该报 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 {error:?}"`。

实际值，原样取自主工作区副本（`logs/now.log`，与 r2、r1yi 逐字相同，也与合入后验证一的 run1 逐字相同）：
```
盘 0 是空盘：该报 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, witness: NewerPublishWitness { witnessed_journal_counter: 5, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 4 } } }, reread: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, witness: NewerPublishWitness { witnessed_journal_counter: 5, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 4 } } } })
盘 1 每次读都报错：该报 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 4, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 3 } } }, reread: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 4, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 3 } } } })
```
- `blank_device_refuses_the_writable_mount_by_name_before_any_write`：循环里第一格「盘 1 是空盘」过了，红在第二格「盘 0 是空盘」。
  txg 5 的根只在盘 0，盘 0 空了，所以所选 (1, 4)；盘 1 的系统配置见证了 jsn 5，5 > 4 ⇒ 乙判真 ⇒ 重读仍真 ⇒ 拒。
- `mount_writable_while_every_read_of_device_one_fails_is_refused_naming_device_one`：txg 4 的根只在盘 1，盘 1 读不出，所以所选 (1, 3)；
  盘 0 的系统配置见证了 jsn 4，4 > 3 ⇒ 拒。
- `second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version` 整个二进制：`15 passed; 2 failed`。

**c366 那一条**（`c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record`），
红在 `tests/second_transaction_supplement_two_warm_up_counter.rs:231:63`，也就是
`let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");`。实际值（`logs/now-warmup.log`，与 run1 逐字相同）：
```
可写挂载: NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, witness: NewerPublishWitness { witnessed_journal_counter: 4, comparison: Undecidable } }, reread: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, witness: NewerPublishWitness { witnessed_journal_counter: 4, comparison: Undecidable } } })
```
用例把 jsn 4 那一格在两块盘上各改坏一个字节。所选根 (1, 4) 自己那条末条记录读不出，计数器等于见证值 4 的记录也是这一条，
所以比较结果是 `Undecidable`，按真 ⇒ 重读仍真 ⇒ 拒。整个二进制 `1 passed; 1 failed`。

## 三、第 ② 问：二分

两个测试文件在 r1 与 r2 之间逐字相同（`git diff --quiet refs/sop/m2-closeout-code-r1-snapshot refs/sop/m2-closeout-code-r2-snapshot -- <两份>` 退 0）。
与主工作区现状比，只差 `use` 那一行（两份各 4 行 diff，原样）：
```
62d61
< use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SparseDevice};
66a66
> use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
28c28
< use singlefs_harness::crash::SparseBlockDevice;
---
> use singlefs_harness::memory_pool::SparseBlockDevice;
```
所以红只能来自 src。各端的做法与结果（`logs/exit-codes.txt` 原样摘行，时刻 UTC）：

| 端 | 怎么造 | 退出码 | fsync_drop | warm_up_counter |
|---|---|---|---|---|
| r1 | `git archive refs/sop/m2-closeout-code-r1-snapshot`（67f447de） | `r1 0 2026-09-27T12:15:21Z` | `ok. 17 passed; 0 failed` | `ok. 2 passed; 0 failed` |
| r1yi | r1 + C554 乙补丁（`git apply --check` 退 0，apply 退 0，偏移 −1 到 −18 行） | `r1yi 101 2026-09-27T12:25:31Z` | `FAILED. 15 passed; 2 failed`，同两条、:1556:9 | `FAILED. 1 passed; 1 failed`，c366 :231:63 |
| r2 | `git archive refs/sop/m2-closeout-code-r2-snapshot`（30c08413） | `r2 101 …12:18:14Z`；`r2-warmup 101 …12:21:59Z` | 同上 | 同上 |
| now | 12:13:54Z 主工作区 `rsync -a --exclude target --exclude .git crates Cargo.toml Cargo.lock` | `now 101 …12:21:26Z`；`now-warmup 101 …12:22:08Z` | 同上 | 同上 |

panic 消息逐字比过：r1yi 三条与 r2 三条 `diff` 为空；r2 的 fsync 两条与合入后验证一 run1 那两条 `diff` 为空；now 的 c366 与 run1 那条 `diff` 为空。
（run-three 那一次 cargo 没带 `--no-fail-fast`，r2 与 now 在 fsync 那个二进制红了之后就停了，warm_up 另跑，所以多出 `-warmup` 两行。）

**r1 加上乙这一件，三条全红，消息与 r2 逐字相同**。所以「乙单独就够」，候选里的其余几批都不是必需的。
反过来，「乙在今天的代码上是必需的」由第四节的变体给出：在主工作区现状上只拿掉乙的两处判定，三条全绿。
所以 A3b、善后一、I-9.16、乙-配置续、合入后验证一自己的三处 src 都既不是必需、也不是充分；乙-配置续的报告里也写过
这几条在它的基线上「同红」（`research/prompts/m2-impl-c554-yi-carry-implementer-report.md` 第 135–136 行）。

## 四、定位：两处判定逐个改坏（副本 `nowvar` = now 副本，改的是 `crates/singlefs-core/src/mount.rs`）

跑之前写下的预测，原样见 `progress.md`：只改 Undecidable ⇒ c366 绿、fsync 两条照旧红；只改末条比较 ⇒ fsync 两条绿、c366 照旧红；两处都改 ⇒ 全绿。

| 变体 | 改法（`logs/variant-*.diff`） | 退出码 | fsync_drop | warm_up_counter |
|---|---|---|---|---|
| A undecidable-false | 第 384 行 `Undecidable => true` 改成 `=> false` | `now-undecidable-false 101 …12:28:51Z` | `FAILED. 15 passed; 2 failed`，消息不变 | `ok. 2 passed; 0 failed` |
| B last-record-false | 第 376 行 `witnessed_journal_counter > selected_version_last_record_counter` 改成恒 `false` | `now-last-record-false 101 …12:31:43Z` | `ok. 17 passed; 0 failed` | `FAILED. 1 passed; 1 failed`，c366 :231:63，消息不变 |
| C both-false | 两处都改 | `now-both-false 0 …12:34:55Z` | `ok. 17 passed; 0 failed` | `ok. 2 passed; 0 failed` |

四条预测全中：每一条红各自只跟一行走，另一行改了不影响它。
变体 B 下 fsync 两条整条过，说明乙一旦不拒，逐盘核报的成员、所选那一版 (1, 4) / (1, 3)、jsn、点名的盘、盘上不变，在今天的代码上都照测试写的那样；
变体 A 下 c366 整条过，说明 txg 5/6/7 对 jsn 4/5/6、tail 与干净重开那一段，在今天的代码上都照测试写的那样。
所以这三条除了「乙先拒」之外，没有别的差异（包括 C577 的屏障、A3b 的几何）让它们红。

## 五、最小复现与推翻条件

最小复现：在任意一份主工作区现状的副本里跑下面这条命令（原样输出是上表 now 那两行；`logs/now.log`、`logs/now-warmup.log`）：
```
nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 4 \
  cargo test --offline --no-fail-fast -p singlefs-harness \
  --test second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version \
  --test second_transaction_supplement_two_warm_up_counter
```
更窄的一种：r1 快照加 `/tmp/claude-1000/impl-c554-yi/patch/crates.patch`，同一条命令，三条全红（r1yi 那一行）。

推翻「红是乙那两行判定引起的」的现象，与造它的那一次：
- r1（没有乙）上这三条会红。造了：r1 三条都绿（`r1 0`）。
- r1 加乙之后有任一条不红。造了：三条都红（`r1yi 101`）。
- 只改第 376 行之后 fsync 两条仍红，或 c366 变绿；只改第 384 行之后 c366 仍红，或 fsync 两条变绿。造了：变体 A、B 与预测一致。
- 两处都改之后仍有红。造了：变体 C 全绿。
以上四个推翻现象，这一次一个都没出现。

推翻第 ④ 问判断的现象：
- fsync 两条「只是成员过时」：如果 kb 里有一句定了「逐盘核排在乙这一判之前」，或者乙在这两格按 D23 第 387 行该判假，这个判断就不成立。
  查到的：D23 第 387 行的比较规则在这两格判真（见证 > 末条）；D18 第 313 行「可写挂载的顺序」没提乙；`mount.rs` 第 3506–3508 行注释写的是
  「排在别的准入之后，那几条先判出来的仍报各自的成员」。没查到定这两道先后的句子（只查了这三处，没有全 kb 搜「逐盘核」）。
- c366「是两条定案冲突、不是钉值」：如果 C579 已经定了（改成「按假」，或维持「按真」并注明 C366 的可写一格作废），它就变成钉值或代码改动之一。
  查到的：`.claude/kb/checks-owed.md` 第 491 行 C579 仍写「条款没写要不要改这一判，交主 agent」。

## 六、排除掉的解释与各自凭的观测

| 解释 | 排除凭什么 |
|---|---|
| 测试文件本身改过 | r1 与 r2 两份逐字相同；与现状比只差 `use` 路径（第三节 diff） |
| C577 那道屏障 | r1yi 里没有 C577（C577 在 r1 之后、乙之后才进），照样三条全红；合入后验证一的 noc577 日志也红 |
| A3b、善后一、I-9.16、乙-配置续、合入后验证一的三处 src | r1yi 里一样都没有，照样全红（不必需）；现状里它们都在，拿掉乙两行就全绿（不充分） |
| 乙之外、同一个补丁里别的改动（读缓存、钩子、`tests/common` 帮手） | 在现状上只改判定那两行就全绿，读缓存与帮手原样留着 |
| 负载或偶发 | 是确定性内存盘，同一消息在 run1、r2、now、r1yi 四次逐字相同（照 test-discipline，这只说明没有隐藏状态，不当统计证据） |

## 七、给主 agent 的材料（只判、不修）

- fsync 两条：乙之后这两格走不到逐盘核。改法大致两类，要哪一类由主 agent 定：
  ① 改成钉乙的成员；那样「盘 0 是空盘」「盘 1 每次读都报错」两格就不再钉逐盘核的点名。
     逐盘核点名空盘这件事，在同一二进制里还有「盘 1 是空盘」那一格与 `stale_device_one_…` 那一条钉着，两条这次都过了。
  ② 改用例的盘面（例如藏掉见证那一槽），让乙判假、留住逐盘核。
- c366：这就是 C579 那一格。它的文件头第 6–7 行写的是「可写挂载这一层唯一分得开两个量的可达盘面」。乙「两条都读不出按真」照今天的字面留着的话，
  C366 在可写挂载这一层就没有可达的盘面了；把它改成「按假」，c366 原样变绿（变体 A）。这是定案取舍，不是钉值。

## 八、跑的环境

- 开跑前 `ps` 看到别的会话在跑：`cargo test -p singlefs-harness --tests`（24G、16 线程那一条）、`cargo build --all-targets`、
  `/tmp/claude-1000/impl-r2-fixes-a` 上的 `checker_known_bad_images`、一个 `e161_crash_state_dedup_and_time_split` 测试二进制、
  变异跑道的 `e142_first_txn_dry_run`。没有 qemu、vm-bench、e152、fio。
  每个副本用各自的 target，没有等锁。
- 主工作区现状的副本取于 2026-09-27T12:13:54Z；那一刻 `git status --short crates Cargo.toml Cargo.lock` 119 行（`now.git-status.txt`）。
  取之后主工作区还在变：交回前复查时，fsync 那份测试的 `use` 行已经从原位替换改成挪了位置。所以第一节的行号只对那一刻的副本。

## 九、没做什么

- 没修，没改主工作区任何文件，副本里的改动不回主工作区。
- 没跑重型测试（名字带 layer0 的、checker 包、54/55/57/59/87 号、全量 cargo test）。
- 没跑门禁阶段（调查员在 stage-owners.tsv 里没有登记的阶段）。
- 没有在乙补丁里再往下二分到更小的 hunk：变体 A、B 已经把红钉到两行判定。
- 没有全 kb 搜逐盘核与乙的先后；只读了 D23 第 387、421、433 行、D18 第 313 行、checks-owed 第 491 行。
- 没核合入后验证一 run1 里别的 18 个红目标。

## 十、删了的副本与编译目录

- `/tmp/claude-1000/investigate-three-reds/r1` 1.7G，删了
- `/tmp/claude-1000/investigate-three-reds/r2` 2.1G，删了
- `/tmp/claude-1000/investigate-three-reds/now` 764M，删了
- `/tmp/claude-1000/investigate-three-reds/nowvar` 886M，删了
- `/tmp/claude-1000/investigate-three-reds/r1yi` 1.8G，删了
- 留着的都不是副本：`logs/`（原样日志、`exit-codes.txt`、`variant-*.diff`、`testdiff-*.txt`）、`progress.md`、`run-*.sh`、`mount.rs.now-original`（变体的原件，主 agent 复核时对照用）、`now.git-status.txt`、`now.snapshot-time`、这份报告。

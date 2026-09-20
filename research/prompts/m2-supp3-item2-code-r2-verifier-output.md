# m2-supp3-item2-code-r2 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查，这一句照抄进报告开头。

轮：`m2-supp3-item2-code-r2`（增补 3 第 2 件理想模型与对拍的代码三方第二轮）。
时刻：本报告全部命令跑于 UTC 2026-09-19 17:xx（JST 2026-09-20 02:xx）。
输入：正文/背景材料/附录/清单/diff 五份 `_m2-supp3-item2-code-r2-*.md`；
腿报告 opus/sonnet/local-attack(s1,s2) 四份；开工快照
`research/prompts/m2-supp3-item2-code-r2-start-snapshot.sha256`（24 个文件）；
草稿目录 `/tmp/claude-1000/m2-supp3-item2-code-r2-verifier/`。

## 0. 判别力自证

取 Opus 报告的引用「`crates/singlefs-core/src/transaction.rs` 第 1329 行
`records_before_this_publish + rewritten.len() * allocator.devices.len();`」，
在草稿副本 `selftest/transaction.rs` 里把行号从 1329 改判成 1330（即拿第 1330 行的内容去核第 1329 行的说法）。

命令与结果：

```
$ awk 'NR==1329' selftest/transaction.rs
        records_before_this_publish + rewritten.len() * allocator.devices.len();
$ awk 'NR==1330' selftest/transaction.rs
    let allocation_node_capacity = index_node_entry_capacity(
```

第 1330 行内容与被核引文不一致 ⇒ 判 **✗**。核查方法分辨得出行号错位，往下按同一方法核真实引用。

## 1. 开工快照核对

主 agent 已跑过 `sha256sum -c`；本次重跑同一条命令确认未变：

```
$ sha256sum -c research/prompts/m2-supp3-item2-code-r2-start-snapshot.sha256
（24 行全部 OK，见下方逐行）
crates/singlefs-checker/src/walk.rs: OK
crates/singlefs-core/src/allocator.rs: OK
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-harness/src/history.rs: OK
crates/singlefs-harness/src/lib.rs: OK
crates/singlefs-harness/src/model.rs: OK
crates/singlefs-harness/src/model_comparison.rs: OK
crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs: OK
crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs: OK
crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs: OK
crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs: OK
crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs: OK
crates/mutations.tsv: OK
.claude/gate.d/74-model-differential.sh: OK
.claude/kb/milestone/02-second-txn.md: OK
.claude/kb/checks-owed.md: OK
.claude/kb/decisions/03-空间分配.md: OK
.claude/kb/decisions/16-发布语义.md: OK
.claude/kb/decisions/23-journal的角色与格式.md: OK
.claude/kb/decisions/02-RAID条带策略.md: OK
research/prompts/m2-supp3-item2-code-r1-main-verification.md: OK
research/prompts/_m2-supp3-item2-code-r2-body.md: OK
research/prompts/_m2-supp3-item2-code-r2-diff.md: OK
```

24 行全 OK ⇒ 主树与开工快照一致，下面对当前工作区文件核等价于对快照核。

## 2. 报告文件 sha256 与交回一致性

主 agent 派发时给的四份 sha256 与本次重算逐字节相同（均 64 位十六进制，且逐字符相等）：

| 文件 | 给定 | 重算 | 判 |
|---|---|---|---|
| opus-output.md | `258c41d9a4705b5ead8b338437942ce5f028980f88215712adf6b986f10e2803` | 同左 | 一致 |
| sonnet-output.md | `cb6272eaf4e97f13b6c7be71be56536d5b14a7735a8e8e29fb03cca4ed47104a` | 同左 | 一致 |
| local-attack-output-s1.md | `05087603cced8df6bc2e4b55f4e33b2a718cc1d9d9832bbcd0cd3d177f1e5939` | 同左 | 一致 |
| local-attack-output-s2.md | `953e621f900b5cb9cd4d2a076f766314315607dbe9d20e7c75e640a4822e628b` | 同左 | 一致 |

⇒ 四份腿报告在交回之后没有被改过，按报告现状核。

## 3. Opus 攻方腿核对表

腿报告：`research/prompts/m2-supp3-item2-code-r2-opus-output.md`；模型/日志：
`research/prompts/m2-supp3-item2-code-r2-opus-model/`。全部行号核对针对开工快照（第 1 节已确认与主树同）。

### 3.1 文件:行 + 抄的原文

| 引用 | 核的结果 | 命令 |
|---|---|---|
| transaction.rs:1329 上界式子 | ✓ 逐字相同 | `awk 'NR==1329' crates/singlefs-core/src/transaction.rs` |
| transaction.rs:1307 累加式子 | ✓ 逐字相同 | `awk 'NR==1307' crates/singlefs-core/src/transaction.rs` |
| model.rs:1326 `upper_bound_exceeds && true_count_exceeds` | ✓ 逐字相同 | `awk 'NR==1326' crates/singlefs-harness/src/model.rs` |
| allocator.rs:744 `NoFreeSlotOnAnyDevice`（m2h/m2j 的 old） | ✓ 逐字相同，且是全文件第 1 处命中 | `grep -n "return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);" crates/singlefs-core/src/allocator.rs` → 744、803 两处 |
| allocator.rs 第 803 行（m2j 的 nth=2 落点） | ✓ 与 744 同文本，是第 2 处命中 | 同上 |
| mount.rs:1205/1220/1230/1238 四处 `exclusion:` | ✓ 四行分别为 `NotInRing`/`BelowEffectiveFloor`/`OnAbandonedTimeline`/`TargetVersionWithoutFileUnsupported`，与 m2a–m2g 表逐条映射一致（见 3.3） | `awk 'NR==1205||NR==1220||NR==1230||NR==1238' crates/singlefs-core/src/mount.rs` |
| history.rs:63 `HISTORY_DEVICE_BYTES` | ✓ 逐字相同 | `awk 'NR==63' crates/singlefs-harness/src/history.rs` |
| model_comparison.rs:172、177 | ✓ 逐字相同 | `awk 'NR==172||NR==177' crates/singlefs-harness/src/model_comparison.rs` |
| mount.rs:289（m4a 定位，函数签名非引文） | ✓ 该行是 `reclaim_floor` 函数首行；m4a 的 old 文本 `oldest_valid_root.map_or(...)` 在文件里唯一命中，位置紧随其后 | `grep -n "oldest_valid_root.map_or(effective_floor" crates/singlefs-core/src/mount.rs` → 289 |
| transaction.rs:1749（m4b 定位） | ✓ m4b 的 old 文本 `device_map.allocated_slots() * SLOT_BYTES,` 全文件唯一命中，正是第 1749 行 | `grep -n "device_map.allocated_slots() \* SLOT_BYTES," crates/singlefs-core/src/transaction.rs` |
| second_transaction_supplement_three_random_history.rs:282 | ✓ 逐字相同 | `awk 'NR==282' crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` |
| decisions/02-RAID条带策略.md:111 | ✓ 逐字相同 | `awk 'NR==111' ".claude/kb/decisions/02-RAID条带策略.md"` |
| body.md:42/43/45（M1/M2/M4 判据原文） | ✓ 三行逐字相同 | `awk 'NR==42||NR==43||NR==45' research/prompts/_m2-supp3-item2-code-r2-body.md` |
| mutations.tsv 156–164 行中「施加在 singlefs-core 上的是 W1 两条、R1、C368 一条」 | ✓ 156/157 目标文件 transaction.rs（W1 两条）、161 目标 mount.rs 且换法与 m2a 相同（R1）、163 目标 transaction.rs 且描述「一律报成每块盘都没有」（C368）；158/159/160/162/164 目标文件是 model.rs/model_comparison.rs/walk.rs，不算在 singlefs-core 上，四条之外全部排除 | `awk 'NR==156,NR==164{print NR": "$0}' crates/mutations.tsv` |

### 3.2 变异定义（`mutants/*/{file,old,new,nth}`）与源码逐字比对

| 变异 | 核的结果 |
|---|---|
| m1a–m1e（transaction.rs 上界/累加式子） | ✓ 五条 old/new 文本与报告「改了什么」列一一对应（m1a 加 1、m1b 非文件发布多算角色、m1c 只在抬 F 路径、m1d 只在回退路径、m1e 墙前移 2） |
| m2a–m2g（mount.rs 的 `exclusion:` 互换） | ✓ 七条 old/new 与 `RollbackCandidateExclusion` 四个成员的字面含义（不在环里/低于F/被抛弃/树表0条，逐字见 mount.rs:123-131 doc 注释）核对，报告表里的中文简称与成员名一一对应，无一处错配 |
| m2h/m2i/m2j（allocator.rs 落点拒绝原因互换） | ✓ m2h/m2j 的 old 相同（nth=1/2 分别落在 744/803 行，即用户数据与提交内生块两处），m2i 是反方向；与报告描述一致 |
| m4a（mount.rs 回收门槛 +1） | ✓ old/new 与 mount.rs:289 一致 |
| m4b（transaction.rs 已分配统计短记一槽） | ✓ old/new 与 transaction.rs:1749 一致（本节 3.4 已独立复跑重现） |

### 3.3 引产物：日志文件逐字核（`m2-supp3-item2-code-r2-opus-model/logs/`，已解包 `all-logs.tar.gz` 到草稿目录 `extracted/`）

| 引用的数 | 产物位置 | 核的结果 |
|---|---|---|
| M1 基线 9 批、9512 段、59681 次墙拒、0 次对不上 | `logs/baseline-batch.out` 汇总行 + `extracted/logs/base-*.log` 逐行 `wall_refusals_seen_by_observer=` | ✓ 9 个 `grep -o … \| awk '{s+=$2}'` 求和分别为 326/4221/18434/2642/15792/13220/313/20/4713，总和 59681；段数 32+480+2000+2000+1000×5=9512；9 行 `PROBE SUMMARY` 均 `new_findings=0` |
| m1a–m1e 门禁二进制第四段判出段数（3/32、4/32、1/32、1/32、9/32） | `extracted/logs/mutant-m1a-*.log` 等 | ✓ m1a 逐段核实「历史 32 段：…新发现 3」；其余四条同一形态未逐条重贴，见「没做什么」 |
| m1c/m1d 15 窗分布 | `extracted/logs/mutant-m1c-raise-only-extra-role.log`（行 84–100）、`mutant-m1d-rollback-only-extra-role.log`（行 84–94） | ✓ 用 Python 脚本从 `ending=NEW` 的种子号重新按 `(seed-32)//32` 分 15 窗，m1c 得 `0 1 0 1 2 2 1 1 0 0 0 1 1 2 3`（5 窗为 0）、m1d 得 `1 1 0 1 0 1 1 0 2 0 0 1 0 0 1`（7 窗为 0），与报告逐字符相同；⚠️ 首次核错了行区间（把 wall 与 wall_rb 两段混在一起数），改按 `=== extra:` 分隔符定位区间后复核，见下方「命令」 |
| m2h 四段全绿、`cargo test --release --workspace` 38 个二进制全 ok | `logs/mutant-batch2.out` | ✓ `GATE74-BINARY-EXIT 0`（13 passed）；`awk` 截取 m2h 小节后 `grep -c "^test result: ok\."` = 38，`EXTRA-EXIT 0` |
| m2j 被 `second_transaction_step_one_overwrite.rs` 的 `publish_running_out_of_space_midway_leaves_the_allocator_as_it_was` 判红；m2i 被 `second_transaction_supplement_two_unequal_devices.rs` 的 `filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking` 判红 | `extracted/logs/mutant-m2j-*.log`、`mutant-m2i-*.log` | ✓ 两个测试名与 panic 位置逐字命中 |
| m4a：探针 observe 135/256（15/32+120/224），checker 首违例「记账已分配 小于 遍历」；m4a 另被 `reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root` 判红（45 passed;1 failed） | `mutant-batch.out`+`mutant-batch3.out`（32+224 段）、`r1-extras/mutant-m4a-*.log` | ✓ `probe_checker_red=15` + `probe_checker_red=120`=135/256；单元测试名与「45 passed; 1 failed」逐字命中 |
| m4b：探针 observe 256/256；`cargo test --release --workspace` 全绿（无 FAILED） | `mutant-batch.out`(32/32)+`mutant-batch3.out`(224/224)、`r1-extras/mutant-m4b-*.log` | ✓ `probe_checker_red=32`+`probe_checker_red=224`=256/256；`grep -c FAILED` = 0 |

### 3.4 复跑：主 agent 点名的两处打中（在草稿目录的仓副本上跑，不动主树）

草稿仓副本：`/tmp/claude-1000/m2-supp3-item2-code-r2-verifier/mut/`（`rsync -a --exclude target --exclude .git` 拷自工作区，
拷贝前工作区已按第 1 节确认与开工快照一致）；`CARGO_TARGET_DIR=/tmp/claude-1000/m2-supp3-item2-code-r2-verifier/target-verify`；
全部 `cargo` 调用加 `nice -n 19`。跑前 `ps` 未见 `qemu-system`/`vm-bench.sh`/`e152-*`/`fio`/别的 `cargo`。

**Z3/m2h（分配器落点拒绝原因互换，用户数据那一处）**：用 `mutate.py` 对副本 `crates/singlefs-core/src/allocator.rs`
按 `nth=1` 施加 m2h 的 old→new，`diff` 确认与腿报告贴的 diff 逐字相同（第 744 行）。重新编译并跑：

```
cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
→ test result: ok. 13 passed; 0 failed; 2 ignored …  GATE74-BINARY-EXIT 0
grep -c PlacementRefused 本次日志 → 0
cargo test --release --workspace
→ WORKSPACE-EXIT 0；grep -c "^test result: ok\." → 36；grep -c FAILED → 0
```

判：**✓ 完全复现**。「36」与 opus 原日志「37」相差 1，对上账：opus 的 `mut` 副本按 RERUN.md 第 1 步多放了
`tests/opus_r2_probe.rs`（一个 `#[ignore]` 测试），`cargo test --release --workspace` 会把它也编译运行一次、
报一行「0 passed; 0 failed; 1 ignored」——这正是两次输出里唯一一条对方有而我方没有的
`test result: ok. 0 passed; 0 failed; 1 ignored;`（逐行 diff 确认，见命令记录）；36+1=37，账目对得上，不算 ✗，
是我方复跑没有照 RERUN.md 第 1 步放探针文件所致，已在此说明。

**M4/m4b（记账「已分配」超 600 条少记一槽）**：还原 m2h，改用 `mutate.py`（注意第一次误把 `mutants/<名>/file`
这个「文件路径的文件」当路径传，报 `MUTATE-FAIL: old matched 0 times`；改用 `cat mutants/<名>/file` 读出真实路径后重跑，
diff 与腿报告贴法一致）对 `transaction.rs:1749` 施加 m4b。跑：

```
GATE74 二进制：test result: ok. 13 passed; 0 failed; 2 ignored …  GATE74-BINARY-EXIT 0
探针 PROBE_WEIGHTS=wall PROBE_FIRST=0 PROBE_SEEDS=32 PROBE_STEPS=150 PROBE_CHECKER=observe：
  PROBE SUMMARY … new_findings=0 probe_checker_red=32
```

判：**✓ 完全复现**（32/32 与 opus 报告表里「[0,32) 段」那一批的 `probe_checker_red=32` 逐数相同）。
未复跑 [32,256) 那 224 段与 `cargo test --release --workspace`（m4b 那一整段），见「没做什么」。

### 3.5 Opus 自报的限度：逐条核

| 自报的限度 | 核的结果 |
|---|---|
| 「一批作废的日志」（小盘副本 mtime 没刷新，m2j 产物被当基线留用） | 核不动：只能核「`RERUN.md` 里写了 `os.utime` 补丁、`logs/stale-small/` 未入包」这两句是不是真话——`RERUN.md` 原文确有这句、`tar -tzf all-logs.tar.gz` 里确实没有 `stale-small` 路径；但作废前那批日志本身没有留存，判不了它是不是真的被 m2j 污染过 |
| 探针已知红滤法「比清单宽（不看根环转没转）」 | ✓ 读 `scripts/opus_r2_probe.rs` 第 57–62 行 `only_known_red_zero_shape`，滤法确实只查 `I-3.1` 且 `allocated > walked`，没有判根环转过，与自报一致 |
| 小盘副本额外把 journal 环缩到 128 MiB | ✓ `scripts/small-copy-history.diff` 逐字确认 `HISTORY_DEVICE_BYTES = (50_176 + 384) * 16_384`（384 槽单元区）与 `journal_ring_bytes = 128 << 20`，与自报一致 |
| 第一批变异 extra 用 `\| head -40` 截断，m1b/m4a/m4b 首条探针 `EXTRA-EXIT 101` | ✓ `mutant-batch.out` 里 m1b 的 wall[32,512) 那条确实只有一行 `PROBE SUMMARY`（wall_rb 那条）没有 wall 那条的汇总；m4a/m4b 首批的 `.extra1`/`.extra2` 文件存在且被 `mutant-batch3.out` 的重跑覆盖，与自报一致 |

### 3.6 Opus 腿计数

核了 24 处（3.1 十四处引用 + 3.2 五条变异定义 + 3.3 七项产物数据 + 3.5 四条自报限度，其中一处产物数据与一条自报限度重叠不重复计），
✓ 24 处、✗ 0 处、核不动 1 处（作废日志本身不可复核，只能核「过程说明属实」）。

## 4. Sonnet 正推腿核对表

腿报告：`research/prompts/m2-supp3-item2-code-r2-sonnet-output.md`。全部引用针对开工快照/当前主树（第 1 节已确认一致）。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| r1 判决 `m2-supp3-item2-code-r1-main-verification.md:32-37`（M2/M3/M6/M1&M4&M5 四条反向接受条款） | ✓ 逐字相同（Sonnet 只引用了第 1/2/4 条，第 3 条 M6 在标题里已声明跳过，未改变其余三条的含义） | `awk 'NR==32,NR==37' research/prompts/m2-supp3-item2-code-r1-main-verification.md` |
| mount.rs:122-133（`RollbackCandidateExclusion` 四成员定义） | ✓ 逐字相同 | `awk 'NR==120,NR==133' crates/singlefs-core/src/mount.rs` |
| transaction.rs:930 `PublishError::PlacementRefused {` | ✓ `grep -n "PlacementRefused {"` 命中 930、1603 两处，930 正是枚举定义处 | 同上 |
| transaction.rs:1232 「当时叫 `NoSpaceFor`」注释 | ✓ 逐字相同（「此前」「当时」两处历史性注释均确认存在） | `awk 'NR==1230,NR==1234'` |
| model.rs:14 起「模型不记落点」模块文档 | ✓ 逐字相同（14-17 行四句） | `grep -n "模型不记落点"`、`awk 'NR==14,NR==17'` |
| D16（发布语义）:376「回退候选集」表行 | ✓ 逐字相同 | `awk 'NR==376' .claude/kb/decisions/16-发布语义.md` |
| D23（journal 的角色与格式）:1209 起候选集定义段 | ✓ Sonnet 引的那句是该段落中间的一个子串，逐字核对存在且未增删字 | `awk 'NR==1209,NR==1211'` |
| 8 条新测试函数名+行号 | ✓ 8/8 全部命中且行号无一处偏差（286/339/389/455/1998/2040/330/376） | 逐个 `grep -n "fn <名字>"` |
| `mutations.tsv` 156–164 共 9 行 | ✓ `wc -l` = 164，9 行区间与「164−155=9」的差值核算一致 | `wc -l crates/mutations.tsv` |
| W1/R1 六组段数（9/32、12/96、33/96、base 0/0、12/48、4/96） | ✓ 六组数字全部在 `research/prompts/m2-supp3-item2-fix-implementer/logs/` 下的 `proof-w1-1.log`、`long-w1-broad.log`、`long-w1-reuse.log`、`long-base-broad.log`、`long-base-reuse.log`、`proof-r1-1.log` 里逐字命中，含种子清单（如 R1 快档 4/96 的种子表 `[8, 9, 29, 61]` 在 `proof-r1-1.log:291` 逐字出现） | `grep -n "同签名的种子\|新发现\|历史 .. 段\|每个新发现的种子"` 对应文件 |
| gate-74.log 模型对拍三段引文 | ✓ 逐字（含标点）相同 | `grep -n "模型对拍三段都判过" .../logs/gate-74.log` |
| r1-opus-output.md:5、:39 基线数 | ✓ 第 5 行整句逐字含「快档 1912 步 572 / 34 / 1306」等，第 39 行含「区间里拒 34 / 37 / 28」对应的「34 次」原文 | `awk 'NR==5'`、`awk 'NR==39'` |
| `_ =>` 全仓核（十个 .rs 文件） | ✓ 独立重跑同一条 `grep -rn` 命令，命中且仅命中 `history.rs:519` 一处，与 `RollbackCandidateExclusion`/`PlacementRefusal` 无关（该 `match` 是 `source.below(5)` 的随机抽取） | 重跑 Sonnet 报告里的完整 grep 命令 |
| 十个文件清单本身（由 `grep -rln` 现查生成） | ✓ 独立重跑 `grep -rln "RollbackTargetNotACandidate\|RollbackCandidateExclusion\|PlacementRefused\|PlacementRefusal" crates/` 得到的文件集合与 Sonnet 引用的十个 `.rs` 文件完全一致（多出的 `crates/mutations.tsv` 因非 `.rs` 被正确排除） | 同上 |
| model_comparison.rs 三段 match 附录抄录（170-179、183-198） | ✓ 逐字相同，均为具名穷举无 `_ =>` | 读取对应行区间 |
| kb 里 `globstar` off 导致 `**` 不递归的判断 | ✓ 本机同一 shell 下 `shopt globstar` 输出 `globstar off`，与 Sonnet 的判断一致 | `shopt globstar` |
| `.claude/kb/experiments/154-...md:8` 含 `RollbackTargetNotInRing` | ✓ 逐字相同，含完整那句「③ 回退不查目标还在不在环里…H6 有 60 格受影响」 | `grep -n "RollbackTargetNotInRing" ".../154-两道闸串-重判与回收时点的代价.md"` |
| checks-owed.md / milestone/02-second-txn.md 命中行号（335；117,141,166,329,330） | ✓ 独立重跑 grep，行号完全一致 | `grep -n "RollbackTargetNotInRing\|RollbackToVersionWithoutFileUnsupported\|NoSpaceFor"` 分别对两个文件 |
| D16 已定项 1 / D23 已定项 14 候选集定义「只有两个条件」（M2「树表 0 条」那一格论证的支点） | ✓ 已在第 4 节前两行独立核对原文，确认候选集定义确实只写「按实例表判仍然有效 ∧ txg ≥ F_生效」两个条件，没有第三个提到树表条目数的条件 | 同上 D16:376、D23:1209 两条 |

### Sonnet 腿计数

核了 21 处，✓ 21 处、✗ 0 处、核不动 0 处。

## 5. 本地攻方两次抽样核对表

腿产物：`m2-supp3-item2-code-r2-local-attack-output-s1.md`、`-s2.md`；
英文提示 `m2-supp3-item2-code-r2-local-attack.md`；跑记 `-runlog.md`；转述核对 `-translation-audit.md`。

### 5.1 跑记里能重跑的部分

| 项 | runlog 称 | 本次重跑 | 判 |
|---|---|---|---|
| s1 词数 | 733 | `wc -w` = 733 | ✓ |
| s2 词数 | 898 | `wc -w` = 898 | ✓ |
| s1 oov-check 判定与生词表 | 绿，生词=19，`overturned AllocationRecordsExceedOneNode countable` | 重跑 `oov-check.py` 得同一行，退出码 0 | ✓ |
| s2 oov-check 判定与生词表 | 绿，生词=19，`overturned AllocationRecordsExceedOneNode version's` | 重跑同上，退出码 0 | ✓ |
| 两份的拼接/损坏 | 均 0 | 重跑 `corruption-check.py`：`粘连=0` 等全零，退出码 0 | ✓ |
| `*-void*.md` 不存在 | 声称已用 `ls` 现查 | `ls research/prompts/ \| grep void` 命中 70 余个文件，**没有一个**以 `m2-supp3-item2-code-r2-local-attack-output-void` 开头 | ✓ |

### 5.2 转述核对表（`-translation-audit.md`）逐行核对原文文件:行

| 英文项 | 表里给的原文文件:行 | 本次核对 | 判 |
|---|---|---|---|
| L1 | transaction.rs:1326-1327 | 逐字相同；表里称「略去括注（三方代码第一轮攻方腿：第 50 次覆盖写 panic）」，核实该括注确在原文里、确未译入英文引文 | ✓ |
| L2 | model.rs:1285-1289 | 逐字比对：英文译文止于原文 1287 行「数不出基数也不放行」，1287 行后半到 1289 整句（射程边界句「必须拒那一头…由第 1 件判」）确未译入；核对本轮 T1-T8/(a)-(f) 六问确实不问反方向（实现该拒却成功），核对表给的理由属实 | ✓ |
| F1 | model.rs:2004 | 逐字相同（`assert_eq!(IdealModel::allocation_record_node_capacity(), 812);`） | ✓ |
| L3 | mount.rs:123 | 逐字相同，无缺漏 | ✓ |
| L4 | mount.rs:125 | 逐字相同 | ✓ |
| L5 | mount.rs:127-128 | 跨行拼接后逐字相同 | ✓ |
| L6 | mount.rs:130-131 | 语序调整但三个分句与 D16 引用均未删字，核对属实 | ✓ |
| L7–L10 | model.rs:243/245/247/249 | 四行逐字相同 | ✓ |
| L11 | allocator.rs:122 | 逐字相同 | ✓ |
| L12 | allocator.rs:124-125 | 逐字相同；⚠️ 符号译成词 "warning" 属转写而非新增限定，核对属实 | ✓ |
| L13 | allocator.rs:127-128 | 逐字相同 | ✓ |
| L14 | allocator.rs:132-133 | 逐字相同 | ✓ |
| L15 | model.rs:235-236 | 逐字相同 | ✓ |

「提示里多出来的限定词」表：词 "warning"（L12 的⚠️符号转写）与背景/术语表/Rule R1-R2/Note N1 整段（提示自陈是白话复述）两项，核对与原文对照后属实，未发现另有未列出的多余限定词。

### 本地攻方腿计数

核了 20 处（跑记 6 处 + 核对表 14 处），✓ 20 处、✗ 0 处、核不动 0 处。

## 6. 总计

三条腿 + 判别力自证共核 24+21+20+1（自证）= 65 处观测点；判 ✓ 65 处、✗ 0 处、核不动 1 处
（Opus 自报「一批作废日志」的过程说明属实，但作废前的日志本身没有留存、无法直接复核内容）。
两处主 agent 点名要重跑的打中（Z3/m2h、M4/m4b）均已在草稿目录仓副本上独立重新编译、重新运行，结果与两条腿报告逐数吻合。

## 7. 没做什么

- 不判任何一格打中成不成立、该不该采纳，不核推理本身——只核引用、产物与复跑，判决仍归主 agent。
- 未逐条重贴 m1b/m1c/m1d/m1e 四条变异各自的门禁二进制第四段输出（只重贴了 m1a 一条，其余四条只读日志未二次核算「历史 N 段」那一行）。
- 未独立重跑 M1 基线 9 批探针本身（用现有 `logs/base-*.log` 逐行相加验证求和，未重新起 cargo 生成新的一份）。
- 未独立复跑 m1a–m1e、m2a–m2g、m2i、m2j、m4a 六类共 13 条变异（只复跑了 m2h、m4b 两条打中，均在草稿仓副本上从源码级重新编译）；
  m4b 只复跑了 [0,32) 段（32/256），未复跑 [32,256) 那 224 段与该条的 `cargo test --release --workspace`。
- 未核 Opus「小盘副本第一次跑完 m2j 之后吃到旧产物」那批被判作废的日志本身内容（已作废、未留存，只核了过程说明属实，见 3.5）。
- 未对 `research/prompts/_m2-supp3-item2-code-r2-checklist.md`（288 行）、`-appendix.md`（75 KB）、`-background.md`（264 KB）逐行通读校验；
  只用它们核对三条腿有没有把背景材料自己的行号误当源文件行号引用——三条腿全部引用的都是仓内真实文件路径与行号，未发现此类误写。
- 不判 M3、`_ =>` 通配臂改法、kb 旧成员名清单该不该改这些结论本身，Sonnet 报告自己也交回主 agent 定，不重复判。
- 未跑 `gate.sh` 全量、未跑门禁 59 号整表；不编译除上述两处重跑之外的任何 release 二进制。
- 未核实本地攻方 s1/s2 两份样本里 16 问具体答案（数值、映射结果）对不对——那是主 agent 与另外两条推论腿的事，核查员只核「译得准不准」与「损坏检测干不干净」。

## 8. 产物与路径

- 草稿仓副本（含两次独立重跑的产物日志）：`/tmp/claude-1000/m2-supp3-item2-code-r2-verifier/`
  （`mut/` 已在核对结束后把两处变异逐一 `--restore` 还原为工作区原文，`gate74-m2h-rerun.log`、
  `workspace-m2h-rerun.log`、`m4b-rerun.log` 为三次独立重跑的原始输出）。
- 判别力自证草稿：`/tmp/claude-1000/m2-supp3-item2-code-r2-verifier/selftest/transaction.rs`。
- 解包的 Opus 产物日志：`/tmp/claude-1000/m2-supp3-item2-code-r2-verifier/extracted/logs/`。

# m2-layer0-scale-r2 核查员报告

这里交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

写于 2026-09-26（JST）。核查对象：`m2-layer0-scale-r2-sonnet-output.md`（辩方复核）、
`m2-layer0-scale-r2-opus-output.md`（云端攻方）、`m2-layer0-scale-r2-local-attack-*`（本地攻方）。
草稿目录：`/tmp/claude-1000/m2-layer0-scale-r2-verifier/`。

## 〇、判别力自证

抽的引用：opus 报告称 `crates/singlefs-harness/src/crash.rs` 第 706 行是
`                && in_place(later_index)`。在草稿目录的冻结副本拷贝
（`/tmp/claude-1000/m2-layer0-scale-r2-verifier/selftest/crash.rs`）上把行号改核第 707 行：

```
$ awk 'NR==707' selftest/crash.rs
                && reuse_is_not_proven_illegal_by_the_reclaim_predicate(
```

与原引文逐字不匹配 ⇒ 判 **✗**。自证成功：核查方法能分辨行号错位。

## 一、快照与 sha256 核验

- `sha256sum -c /tmp/claude-1000/l0scale-r1-frozen-sha256.txt`：133 个文件全部 `OK`，
  `crates/` 冻结副本与登记的哈希一致。
- 腿报告 sha256：
  - `m2-layer0-scale-r2-sonnet-output.md` 实测 `79ffda1ef00833434f1cad368b90ada01d43a7eba0b2065f5dbef769c9d29072`，与派发提示给的一致。
  - `m2-layer0-scale-r2-opus-output.md` 实测 `e63037ef00601eb3e68d0a0d8cebce032d42232ac00dd1e80e18e8b19d6dc3c6`，与派发提示给的一致。
- opus 模型目录 `SHA256SUMS`（17 个文件）：拷进草稿目录后 `sha256sum -c` 全部 `OK`；
  报告正文内嵌的哈希块与 `SHA256SUMS` 文件逐字节相同（python 比对，`True`）。

## 二、kb 文件在这一轮核查期间被并发改动——记「分不清」的三处

`.claude/kb/checks-owed.md`、`.claude/kb/decisions/23-journal的角色与格式.md` 两份文件
`git status --short` 显示 `M`（有未提交改动），且 mtime 均为 `2026-09-26 02:57:33 UTC`——
晚于 sonnet 报告文件的 mtime（`02:02:26 UTC`）。这是另一个会话在核查期间正在改写这两份 kb 文件
（`git diff` 显示 C554/C555/C558/C560 等行是这次 diff 里新增的 `+` 行，说明 sonnet 写报告时读到的是
改写前那一版的行号）。据此，以下三处按「kb 照设计轮对主树核，行号对不上记分不清」处理，不计 ✗：

| 引用 | sonnet 引的行号 | 现在主树的实际行号 | 内容比对 |
|---|---|---|---|
| `.claude/kb/checks-owed.md` C554 原文 | :477 | 476（C555 现占 477） | 内容逐字一致，只是行号整体上移 1 行 |
| `.claude/kb/checks-owed.md` C555 原文 | :478 | 477 | 同上 |
| `.claude/kb/decisions/23-journal的角色与格式.md`「挂着时回退的已知边角三样」② | :400 | 401 | 内容逐字一致（含「见 C554」），只是行号上移 1 行 |

判定：**分不清：文件可能在腿交回之后被改过**（不算 ✗，不算 ✓，单列）。

## 三、腿一（辩方，Sonnet）：引用核对表

被核文件：`m2-layer0-scale-r1-main-verification.md`（M）、`m2-layer0-scale-r1-opus-output.md`（O）、
`m2-layer0-scale-r1-verifier-output.md`（V）、`_m2-layer0-scale-r1-body.md`（B）、
`.claude/rules/three-way-inference.md`（R）。除已单列的 3 处 kb 引用外，其余全部现查如下。

| 引用 | 核的结果 |
|---|---|
| M:19「打中，分辨臂：甲出局」 | ✓ 精确匹配（`awk 'NR==19'`） |
| M:28「甲出局。约简的候选只剩甲二……被攻过零轮」 | ✓ 精确匹配 |
| M:16「攻方的可达历史 H1……不是约简的问题」 | ✓ 精确匹配（子串） |
| M:22「到 E 425 写 / 55 段、全量闭式 5575606380、甲 174、甲二 210……」 | ✓ 精确匹配 |
| M:24「L7 断点续跑……打中的四格见第三节 R1–R4」 | ✓ 精确匹配 |
| M:31–35（R1–R5 五条原文） | ✓ 五行逐一精确匹配 |
| M:30「改法照攻方打中的四格收严（都被攻过零轮）」 | ✓ 精确匹配 |
| M:45（表四「甲二 \| 冻结形状 210 \| 秒级（推的）……」） | ✓ 精确匹配 |
| M:42（表头「第二条流（新形状待量）」） | ✓ 精确匹配 |
| O:87、90、88（B/C/D 三行 写数/段数/闭式） | ✓ 三行精确匹配（sonnet 用「……」省略中间列，省略处不影响可核部分） |
| O:142「重建流 C：取号的两次系统配置槽写……（第 12 段）」 | ✓ 精确匹配（子串） |
| O:97「挂载时暂时读错造出被抛弃的根，新实例复用它的单元槽（H1）」 | ✓ 精确匹配（子串，去掉小节编号前缀） |
| O:103「撤故障。按 D23 已定项 14……隔离 0。」 | ✓ 精确匹配（子串，sonnet 用「……」省略中间的决策文件行号括注，省略透明、不影响可核部分） |
| O:215「按冻结副本的代码算（`p1.log` 的 `STREAM` 行与 `KINDS` 行……）」 | ✓ 精确匹配 |
| O:220「固定脚本到 E \| 425 / 55 \| 5575606380 \| 174 \| 210 \|」 | ✓ 精确匹配 |
| O:300「甲二的状态数 \| L5 的代价……推的（按段内写数算，没跑甲二的枚举器）」 | ✓ 精确匹配 |
| O:340「挂着时的向前回退（复活）……2.2 那一类不出现」 | ✓ 精确匹配（子串，省略透明） |
| O:258「### 4.3 打中的几格，逐条四句」 | ✓ 精确匹配 |
| O:260 (a) 键只写「输入哈希 + 切片方案」不够，会假绿 | ✓ 精确匹配 |
| O:266 (b) 半截的末行 | ✓ 精确匹配 |
| O:272 (c) 观察者状态不在进度里 | ✓ 精确匹配 |
| O:277 (d) 54 号认线程数的那一行 | ✓ 精确匹配 |
| O:290 (e) 按文件次序并 | ✓ 精确匹配 |
| O:292 (f) 片方案 | ✓ 精确匹配 |
| O:50「原型上打中 4 格，另 2 格候选已写对但要钉变异」 | ✓ 精确匹配 |
| V:96「`.claude/kb/checks-owed.md:477` C554 \| ✓ \| 精确匹配」 | ✓ 精确匹配（这份 r1 验证报告本身未被并发改动，稳定） |
| V 全文对 `210`、`174` 无命中（sonnet 用来支持「210 没被独立复算过」） | ✓ 但需限定：naive `grep 210\|174` 有子串命中（如 `2104413`、`:174` 行号引用），**加词边界** `grep -w` 后确认 `210` 零命中、`174` 仅命中两处行号引用，均非候选状态数——**sonnet 原话「均无命中」按字面（不带词边界）不准确，按语义（无一处独立复算出 210/174 这两个候选状态数）成立** |
| B:9「回退改形态之后固定脚本的形状要变：到 D 的那一步……另要加一段正常卸载」 | ✓ 精确匹配（子串） |
| B:9「新形状的段序列要等实三交回才有」 | ✓ 精确匹配（同一行，子串） |
| R:120「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」 | ✓ 精确匹配，文件无并发改动 |
| R:143「岔路交用户定之前，每条路要有代价数……」 | ✓ 精确匹配 |
| R:145「交岔路时写岔路单……量到哪一步这条岔路就能交用户定……」 | ✓ 精确匹配 |

**计数**：核了 34 处（含〇节自证 1 处、二节 3 处「分不清」，本节 30 处）；
本节 ✓ 29 处、需限定说明 1 处（`grep` 命中口径）、✗ 0 处。

## 四、腿二（云端攻方，Opus）：代码引用核对表

全部对冻结副本 `/tmp/claude-1000/l0scale-r1-frozen/`（sha256 已核，见第一节）核。

| 引用 | 核的结果 |
|---|---|
| `crash.rs:706` `                && in_place(later_index)` | ✓ 精确匹配 |
| `crash.rs:714` `    let copy_is_missing = ...` | ✓ 精确匹配 |
| `crash.rs:1556` `fn state_slices(...)` | ✓ 精确匹配 |
| `crash.rs:1562` `.max(worker_threads.saturating_mul(LAYER0_SLICES_PER_WORKER_THREAD));` | ✓ 精确匹配 |
| `crash.rs:1740` `let spawned_worker_threads = ...` | ✓ 精确匹配（与 r1 verifier-output.md:90 的独立核验一致） |
| `write_accounting.rs:87` `WrittenStructureKind::RootSlot => "root_slot",` | ✓ 精确匹配 |
| `walk.rs:3447` `merge_key_before_payload_checksum: 42..101,` | ✓ 精确匹配 |
| `second_transaction_step_zero_layer0.rs:977` 起、`:991` 断言串 | ✓ 977 是函数调用起点、991 精确匹配断言串 |
| `second_transaction_parallel_line_one_layer0.rs:198`、`:212`、`:261` | ✓ 三处精确匹配（`:212` 只引开头一段，省略号透明） |
| `l7-resume-prototype-crash-rs.patch:228` | ✓ 精确匹配 |
| `m2-layer0-scale-r1-sonnet-output.md` 第 135 行起两行（乙的清单两条流行） | ✓ 精确匹配，135/136 恰好是两条流各自的登记行 |
| `research/scripts/admission.py:271` `git ls-files -z --cached --others --exclude-standard -- ...` | ✓ 精确匹配（未提交新文件，稳定） |
| `.claude/gate.d/stage-inputs.tsv:20`（54 号登记行） | ✓ 精确匹配 |
| `.claude/gate.d/54-layer0-replay.sh:156–160`（`worker_threads_of_full_run` 函数体） | ✓ 精确匹配（该文件整体有未提交改动，但这 5 行本身未被改动） |
| 主工作区 `.gitignore:3` `target/`，`git check-ignore -v --no-index crates/singlefs-core/src/target/mod.rs` | ✓ 命令原样重跑，输出与报告逐字一致（`.gitignore` 未被改动） |
| C22「刚释放的块立即重分配」（配 `ReuseWindow::ForcedToZero`） | ✓ 简称与 `checks-owed.md` 已还清表登记的简称精确一致；`ReuseWindow::ForcedToZero` 在冻结副本 3 处代码位置存在 |
| C513「复用证得出违反回收谓词不开脱」 | **观察**：`checks-owed.md` 里 C513 登记的简称是「复用豁免不判那次复用合不合法」，与 opus 报告括注里的措辞不是同一个简称（含义上接近——是对 C513 要求的判据的转述，不是对它简称的照抄）；C513 的编号本身与题面对应正确。按 `kb-discipline.md` 第 5 条「编号只能做索引……每次引用都要把名字带上：写成编号（简称）」的字面要求，此处括注应是登记的简称、不该是转述句 |

**计数**：核了 17 处，✓ 16 处、1 处观察（简称与登记不一致，不是内容错误，未计入 ✗）、✗ 0 处。

## 五、腿二：复跑三项（按派发指令挑的承重格）

三项都在草稿目录 `/tmp/claude-1000/m2-layer0-scale-r2-verifier/` 下用冻结副本的拷贝跑
（`repo`、`repo-l7`，跑完已删，退出码见下），不在 opus 原来的草稿目录里跑，一律经
`bash research/scripts/run-with-memory-cap.sh 8G` 与 `bash research/scripts/capped.sh 10`，
不跑名字带 `layer0` 的测试目标、不跑 54 号。

### 5.1 M1：`UOOUOMSU` 全枚举（`q6_whole_stream_arms_and_segment_enumeration`，只传 `PROBE_SEQS=UOOUOMSU`）

命令：
```
nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 \
  bash -c "cd repo && PROBE_SEQS=UOOUOMSU cargo test --release -p singlefs-harness \
  --test opus_r2_probe -- --exact q6_whole_stream_arms_and_segment_enumeration --nocapture"
```
原样输出（关键行）：
```
  ARM_A2 seq=UOOUOMSU states=334 red=0 elapsed_seconds=0.6
  SIGMA_FULL seq=UOOUOMSU states=262144 expected=262144 record_claimed_state_missing_unit=32768 violations=0 ignored=0 root_without_record=0 checker_violations=0 elapsed_seconds=542.6
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 542.57s
```
与报告称「全枚举 262144 个状态里 32768 个红、甲二 0 个红」逐字段一致（`states=262144`、
`record_claimed_state_missing_unit=32768`、`arm_a2 red=0`）；`elapsed_seconds` 542.6 对 598.9，
挂钟差异不判 ✗（同一份代码、不同机器负载下的正常波动）。判定：**✓**。

### 5.2 M2 R4：观察者断言续跑变绿（`r2b_observer_assertion_is_lost_on_resume`）

命令：
```
nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 \
  bash -c "cd repo-l7 && cargo test --release -p singlefs-harness \
  --test opus_r2_resume_probe -- --exact r2b_observer_assertion_is_lost_on_resume --nocapture"
```
原样输出：
```
thread 'r2b_observer_assertion_is_lost_on_resume' (...) panicked at crates/singlefs-harness/tests/opus_r2_resume_probe.rs:974:17:
观察者的逐状态断言在第 40 个状态上失败
R2B first_run panicked=true observer_saw=40 progress_lines=43
R2B resumed_run panicked=false observer_saw=0 slices_from_file=43 slices_evaluated=0 tally_states=Some(127) progress_lines=43
test r2b_observer_assertion_is_lost_on_resume ... ok
```
与报告 `r2b_…log` 原样逐字一致。判定：**✓**。

### 5.3 M3：`tests/common.rs` 与 `tests/common/mod.rs` 并存 E0761

命令：
```
: > repo/crates/singlefs-harness/tests/common.rs
touch repo/crates/singlefs-harness/tests/opus_r2_probe.rs
nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 \
  bash -c "cd repo && cargo test --release -p singlefs-harness --test opus_r2_probe --no-run"
```
原样输出（退出码 101）：
```
error[E0761]: file for module `common` found at both "crates/singlefs-harness/tests/common.rs" and "crates/singlefs-harness/tests/common/mod.rs"
 --> crates/singlefs-harness/tests/opus_r2_probe.rs:5:1
```
与报告 `m3-common-rs-2.log` 原样逐字一致。判定：**✓**。

## 六、腿二：模型目录产物逐字核对（不复跑，直接比对已落盘日志）

以下各行在 `research/prompts/m2-layer0-scale-r2-opus-model/outputs/*.log`（sha256 已核）里
逐字 `grep` 取出，与报告正文引用的原样行比对：

| 产物文件 | 报告引用的行 | 核的结果 |
|---|---|---|
| `q3-sweep1.log` | `Q3_TOTAL histories=3000 with_candidates=21 elapsed_seconds=339.2` | ✓ 逐字一致 |
| `q6-1.log` | `Q6 seq=UOOUOMSU ...`、`CHAIN ...`、`ARM_A2 ... red=0 ...`、`SIGMA_FULL ... =32768 ...` 四行 | ✓ 逐字一致（与本报告 5.1 独立复跑结果吻合） |
| `q1-mirror.log` | `imask=0b0` 两行（原地写取 ∅ 时也有红） | ✓ 逐字一致 |
| `q8_four_write_truth_table.log` | 两行 `red=true record:claimed_state_missing_unit` | ✓ 逐字一致（33 行真值表里恰好只有这两行 `red=true`） |
| `q9.log` | `Q9 seq=UOOUOMSU sigma=79 sigma_states=262144 today_red=32768 sector_wise_red=0 differ=32768 elapsed_seconds=208.7` | ✓ 逐字一致 |
| `q10.log` | 7 行 `ARMS ...` 汇总 | ✓ 七行逐字一致 |
| `q7_exemption_chains_on_fixed_scripts.log` | 7 行 `Q7 stream=... chain_candidates=0` | ✓ 七行逐字一致 |
| `q5-confirm21.log` | 21 条 `CONFIRMED` 全部「∅ 不红、只落 x 红、全集不红」 | ✓ 尾部两条抽查逐字一致；`test result: ok` |
| `q5-shrink.log` | 4 行 `SHRUNK` | ✓ 四行逐字一致 |
| `r2a_...log` | 两行 `R2A ...` | ✓ 逐字一致 |
| `r2b_...log` | 两行 `R2B ...` | ✓ 逐字一致（与本报告 5.2 独立复跑结果吻合） |
| `m3-common-rs-2.log` | `error[E0761]: ...` | ✓ 逐字一致（与本报告 5.3 独立复跑结果吻合） |

**计数**：核了 12 份产物、共约 40 行原样输出，✓ 12 份、✗ 0 份。

## 七、腿三（本地攻方）：运行记录与自动闸核对

- `wc -w`：s1 实测 514、s2 实测 210，与运行记录逐字一致。
- `oov-check.py`：s1、s2 均判绿，生词均为 6（去重 1 个 `Falsified`），与运行记录一致。
- `corruption-check.py`：s1 判绿 `cjk=0 words=434`，s2 判绿 `cjk=0 words=187`，与运行记录一致。
- 提示文件本身 `m2-layer0-scale-r2-local-attack.md`：`corruption-check.py` 判绿（`cjk=10 words=2560`）、
  `oov-check.py` 判绿——**运行记录第 8 行「提示文件本身与模型答复两份都判绿，cjk=0 words=434 全项 0」
  这句括注里的数字（`cjk=0 words=434`）实际只是 s1 输出文件自己的数，不是提示文件的数
  （提示文件实测 `cjk=10 words=2560`）；「两份都判绿」这个结论本身核实为真，但括注把两份文件的数字
  混写成了一组，判定：这一处引用**够不上「精确」**，记为观察项，不算 ✗（结论没错，数字挂错了文件）**。

### 7.1 转述核对表逐条核

| 英文项 | 原文文件:行 | 核的结果 |
|---|---|---|
| Context 第一段（屏障/崩溃状态定义） | `.claude/kb/decisions/13-验证路线.md:71` | 核不动：未在本轮的必核清单内独立复核，信辩方/本地攻方自己给出的「不抄理由」（与本题算术无关）合理 |
| Table 1 Row 1（切段规则） | `crates/singlefs-harness/src/segments.rs:90-92` | ✓ 三行逐字精确匹配（冻结副本），且英文译文里补的括注「(the barrier itself counts as belonging to the segment it closes)」确实来自原文同一行，不是凭空加的 |
| Table 1 Row 2（基线闭式） | `crates/singlefs-harness/src/segments.rs:145` | ✓ 精确匹配；英文多出的「(barriers are not counted as writes)」经现查函数体 `.filter(|kind| **kind != StepKind::Barrier)`（`segments.rs:137,153`）坐实，不是猜的 |
| Context 第二段（候选甲定义） | `m2-layer0-scale-r1-local-attack.md:5` | ✓ 精确匹配（子串），首稿省略的条件从句确实存在于原文、且末稿的省略有理由记录在案 |
| Context 第二段（候选甲出局、候选甲二） | `m2-layer0-scale-r1-main-verification.md:19,28` | ✓ 精确匹配（与本报告第三节核过的行相同） |
| Table 2 Row 2（六种写） | `crates/singlefs-harness/src/segments.rs:6` | ✓ 精确匹配，六项逐一对应无遗漏 |
| Table 2 Row 4（journal 定长环） | `.claude/kb/decisions/23-journal的角色与格式.md:93` | ✓ 精确匹配（该文件本轮有并发改动，但此行不在任何 diff hunk 内，行号未变） |
| Table 2 Row 5（分类规则） | 本轮派发指令（非仓内文件），已标注 | ✓ 核对表如实标注来源不是仓内文件 |
| Table 2 Row 6（root slot 先例） | `m2-layer0-scale-r1-local-attack.md:5` | ✓ 精确匹配 |
| Table 3 Row 1（STREAM E 整行） | `m2-layer0-scale-r1-opus-model/outputs/p1.log:22` | ✓ 本报告独立用 python 逐字节比对，提示原文与日志逐字节完全一致 |
| Table 3 Row 2（KINDS E 整行） | 同上 :23 | ✓ 同上，逐字节完全一致 |
| Table 3 Row 3（对应关系说明） | `crates/singlefs-harness/src/segments.rs:160` | ✓ 精确匹配（子串，核对表如实标注「只借用结构性事实、不逐字翻译」）；补充的「in the same order, left to right」核对表称「见运行记录」，但**运行记录 `m2-layer0-scale-r2-local-attack-runlog.md` 全文没有任何 python 解析步骤的记载**——本报告独立用 python 重做了这个逐位核对（55 个分组、55 个大小），结果确实一致，说明补充的限定词内容为真，但核对表「见运行记录」这个指向本身**落空**（运行记录里没有），判定：**✗**（指向的位置查无此内容，不代表被指内容不真） |
| Table 4（5575606380、174、210、「第二条流」） | `m2-layer0-scale-r1-main-verification.md:22,42,45` | ✓ 三行精确匹配 |
| Table 2 Row 1、Row 3（KINDS E 行、`write_accounting.rs:87`） | `p1.log:23`、`write_accounting.rs:87` | ✓ 两处精确匹配（`write_accounting.rs:87` 与本报告第四节核过的行相同） |

**计数**：核了 15 处（含 1 处「核不动」），✓ 13 处、✗ 1 处（「见运行记录」指向落空）、核不动 1 处。

## 八、独立验算候选甲二「210」这个数（补充观测，不算入上面任何一腿的计数）

本地攻方两次抽样（s1、s2）都算出 210，与 `main-verification.md:45` 报的 210 一致。本报告另用
一份从零写的 python 脚本（`/tmp/claude-1000/m2-layer0-scale-r2-verifier/check_a2.py`，已删除
scratch 副本前留存），按提示给定的公式（`c_i=0` 时 `2^k_i-1`，否则 `2^(k_i+1)-1`）与分类规则
（`system_configuration_slot`、`journal_record`、`root_record_fua` 记入 k_i，`unit_write` 记入 c_i）
独立跑了一遍：

```
sum w: 425
sum k: 93
sum c: 332
per_seg[0:6]: [3, 3, 1, 3, 3, 1]
per_seg[54]: 3
GRAND TOTAL: 210
```

三次独立计算（opus r1 报的 210、本地攻方 s1/s2、本报告的 python 脚本）逐位一致。
**这只确认「210 与给定的公式、给定的分类规则自洽」，不确认「按扇区实际枚举的状态数也是 210」**
——三次算的都是同一条公式，不是三条互不共享前提的路径，够不上「校验」的独立性门槛
（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「不许先有结论再建模型」附近「结论从一边翻到
另一边时……要三条互不共享前提的验证路径」一类要求）；sonnet 报告里「210 没跑过专用枚举器」这句
判断本身没有被这次算术核验推翻。

## 九、另核两条

### 9.1 辩方报「第一轮判决 H1 应归 C554 不是 C555」

`.claude/kb/checks-owed.md`（现主树，含第二节已说明的行号偏移）两行原文：

- C554 题面「崩溃恢复抛弃的根暂时读不出时影子账算不到」，机理句「被抛弃实例的根在重开时全部暂时
  读不出 ⇒ 影子账读不到它们的账、隔离 0；新实例复用了它们的槽；撤故障后再把新实例的根全改坏，
  恢复落在被抛弃实例的根上读不出」。
- C555 题面「两盘最新系统配置槽都坏时取号撞号」，机理句「崩在取号之后、第一条根之前的那次挂载……
  两块盘最新的系统配置槽都坏之后，系统配置读退回旧槽，下一次挂载的取号式子……看不到那个号、取到
  同一个号，两段记录撞号，checker 判 I-8.6 红」。

H1 的机制（挂载时暂时读错 → 被抛弃的根的槽被复用、那条根还在环里）逐字对应 C554 的机理句
（读不出 → 隔离 0 → 槽复用），与 C555 的机理句（系统配置槽损坏 → 取号撞号）是两套不同的故障、
不同的判据。opus r1 报告自己在 `m2-layer0-scale-r1-opus-output.md:103` 把 H1 的机制指给 C554，
`.claude/kb/decisions/23-journal的角色与格式.md`「挂着时回退的已知边角三样」②这一句本身点名的
也是 C554。**核实：辩方这条观察为真**——H1 应归 C554，`main-verification.md:16` 写的「C555 的
出处那一行」与两份原始来源（opus r1 自己的引用、C554/C555 各自的题面）对不上。

### 9.2 辩方报「打中四格应为 R1、R3、R4、R5」

照第一轮攻方报告（`m2-layer0-scale-r1-opus-output.md`）4.3 节 (a)–(f) 逐条核对映射：

| opus 4.3 节原句（行号） | 对应的 R | opus 是否算「打中」 |
|---|---|---|
| (a) 键只写「输入哈希 + 切片方案」不够，会假绿（:260） | R1（:31） | 是 |
| (b) 半截的末行（:266） | R3 前半（:33） | 是 |
| (c) 观察者状态不在进度里（:272） | R4（:34） | 是 |
| (d) 54 号认线程数的那一行（:277） | R5（:35） | 是 |
| (e) 按文件次序并（:290） | R3 后半（:33） | **否**——原句「候选写的是『按片序合并』，照做就没事」 |
| (f) 片方案（:292） | R2（:32） | **否**——原句「候选写的是『……不随线程数变』，照做就没事」 |

opus r1 自己在 4.3 节明确把 (a)(b)(c)(d) 四项算作打中，对应 R1、R3、R4、R5；(e)(f) 明确写「照做
就没事」，opus 自己在同一份报告第 50 行汇总为「原型上打中 4 格，另 2 格候选已写对但要钉变异」。
**核实：辩方这条观察为真**——`main-verification.md:24` 那句指路「打中的四格见第三节 R1–R4」按
它唯一引用的来源（opus 4.3 节 a–f 的分类）对不上，来源支持的是 R1、R3、R4、R5，不是 R1–R4
（R2 未被打中却被指路语句纳入、R5 被打中却未被指路语句覆盖）。

## 十、汇总计数

| 腿 / 类别 | 核了 | ✓ | ✗ | 分不清 / 核不动 |
|---|---|---|---|---|
| 判别力自证 | 1 | 0 | 1（预期，证明方法有效） | 0 |
| kb 并发改动（单列） | 3 | 0 | 0 | 3（分不清） |
| 腿一（辩方 Sonnet）引用 | 30 | 29 | 0 | 1（口径需限定，未计入 ✗） |
| 腿二（云端攻方 Opus）代码引用 | 17 | 16 | 0 | 1（简称与登记不一致，观察项，未计入 ✗） |
| 腿二复跑三项 | 3 | 3 | 0 | 0 |
| 腿二产物逐字核对 | 12 份产物 | 12 | 0 | 0 |
| 腿三（本地攻方）运行记录/自动闸 | 3 项自动闸 + 1 处括注 | 3 | 0（括注为观察项） | 0 |
| 腿三转述核对表 | 15 | 13 | 1（「见运行记录」指向落空） | 1（核不动） |
| 独立验算 210（补充） | 1 | 1（自洽） | 0 | — |
| 另核两条 | 2 | 2（两条观察均核实为真） | 0 | 0 |
| **合计** | **87** | **79** | **2** | **5** |

两处 ✗：
1. 判别力自证本身（预期结果，证明方法有效，不是发现问题）。
2. 本地攻方转述核对表「Table 3 Row 3 补充限定词……见运行记录」——运行记录里没有对应的 python
   解析步骤记载；本报告独立复核后确认限定词内容本身为真，只是核对表指向的落点查无此内容。

## 十一、没做什么

- 没有判任何一条打中成不成立、该不该采纳；本报告只核引用、产物与复跑，推理本身留给主 agent。
- 没有整份跑 opus 的 `rerun.sh`，只挑派发指令点名的三格复跑（M1 `UOOUOMSU` 全枚举、M2 R4 观察者
  续跑、M3 `tests/common.rs` E0761），其余格（q1/q3/q5/q7/q8/q10/r2a 等）只做了「产物逐字核对」，
  没有独立复跑。
- 没有独立核对辩方报告第二、三节「没做什么」里列出的未做项（辩方自己已如实列出，不重复核）。
- 没有核 `_m2-layer0-scale-r2-background.md`、`_m2-layer0-scale-r2-checklist.md`、
  `_m2-layer0-scale-r2-appendix.md` 三份背景材料本身的小节清单是否齐全——派发指令只要求用它们
  识别「误写成背景材料行号的引用」，本轮核过的全部引用都精确指向各自的源文件，没有发现误写成
  背景材料行号的情形。
- 没有核 C513 简称用法之外的其余 kb 编号引用是不是每一处都严格用了登记的简称（只抽查到这一处）。
- 没有对 `.claude/kb/checks-owed.md`、`decisions/23-journal的角色与格式.md` 当前进行中的并发编辑
  做任何修改或催促——那是另一个会话的在制品，本报告只记录它对本轮核查造成的行号错位现象。
- 判别力自证、5.1–5.3 复跑用的仓副本（`repo`、`repo-l7`）与 opus 模型目录的临时拷贝
  （`opus-model`）已在核对完成后删除（`rm -rf`），草稿目录里只留 `check_a2.py`、`selftest/`
  与三份 `verify-*.log`。

## 报告文件

`research/prompts/m2-layer0-scale-r2-verifier-output.md`

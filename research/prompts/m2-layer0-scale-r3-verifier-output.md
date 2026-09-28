# m2-layer0-scale-r3 核查员报告

这是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

写于 2026-09-26。核的对象：云端辩方 `m2-layer0-scale-r3-sonnet-output.md`（含模型目录）、
云端攻方 `m2-layer0-scale-r3-opus-output.md`（含模型目录）、本地攻方 `-local-attack-output-s1.md`/`-s2.md`
（提示 `-local-attack.md`、转述核对表 `-local-attack-translation-audit.md`、运行记录 `-local-attack-runlog.md`）。

## 一、判别力自证

从待核引用里挑一条：辩方报告第 24 行「冻结副本 `crates/singlefs-harness/src/crash.rs` 第 706、714 行」
（引文分别是 `                && in_place(later_index)` 与
`    let copy_is_missing = |copy: usize| !in_place(copy) && !written_over_later(copy);`）。
在草稿目录的副本（`/tmp/claude-1000/m2-layer0-scale-r3-verifier/self-proof-shifted-citation.txt`）里把行号各加 1，
改写成「第 707、715 行」，按正文第 2 步核对冻结副本 `crash.rs`：

```
第 707 行实际内容：                && reuse_is_not_proven_illegal_by_the_reclaim_predicate(
第 715 行实际内容：    let mut check = RecordCheck::default();
```

两处都对不上引文，判 **✗**。核查方法能分辨错误行号，往下按此方法核。

## 二、sha256 与快照核验

- 报告文件 sha256（现算，`sha256sum`）：
  - `m2-layer0-scale-r3-sonnet-output.md` = `4abcd85ce969cfd8b18344f69848a2a91a19a34736aa99d1cde8eae60e24803e` —
    与派发提示给的一致。
  - `m2-layer0-scale-r3-opus-output.md` = `27274c106c8809290a3c7e1f71c144c66163c6991a59aa58663e7cdf61b35012` —
    与派发提示给的一致。
- 冻结快照 `/tmp/claude-1000/l0scale-r1-frozen/`：只含 `Cargo.toml`、`Cargo.lock`、`crates/`（无 `.claude/kb/`）。
  这一轮引 `.claude/kb/` 的行号按规则对主树核，对不上记「分不清」；引 `crates/` 的行号对冻结快照核。
- 两条云端腿模型目录的 `SHA256SUMS`：`sha256sum -c` 全部 `OK`（辩方 8 个文件、攻方 13 个文件）。

## 三、云端辩方（Sonnet）：引用核对表

对快照的判法：`crates/` 与 `research/prompts/`、`records/` 下的历史文件对主树现查（这些是已提交的历史产物，
不在这一轮改动范围内，没有漂移的理由）；`.claude/kb/`、`.claude/rules/`、`.claude/agent-common.md` 因无本轮快照，
对主树核，对不上才记「分不清」。

| 引用（报告行号） | 内容 | 结果 |
|---|---|---|
| L24-25 `crash.rs:694,706,714` | `written_over_later`/`copy_is_missing` 定义 | ✓ 三处行号与内容对快照逐字符合 |
| L46 `opus_r2_probe.rs:1266` | `pub fn variant_claimed_state_missing_unit(...)` | ✓ `grep -n` 命中同一行 |
| L52 `_background.md:21` 「这一轮可以接着用」 | 探针可沿用的出处 | 行号 ✓ 命中；**但归属有误**，见下方「发现」第 1 条 |
| L87 `r2-opus-output.md:103` | 「这个状态本身是合法的……oracle 两遍违例 0、checker 违例 0」 | ✓ |
| L106 `r1-main-verification.md:22` | 「第一条流 41 写 / 10 段……到 E 425 写 / 55 段」 | ✓（首尾片段对上，中间用省略号） |
| L108 `r2-opus-output.md:130` | 「每条最小的都要两次卸载」 | ✓ |
| L156 `r2-opus-output.md:150` | COW 写按落点分两类的论证 | ✓ 内容与「推的：读代码」的定性一致 |
| L158 `r2-opus-output.md:163` | 「Q1_TOTAL streams=120 streams_with_non_monotone=0」 | ✓ |
| L174 `r2-opus-output.md:151` | 「用记录核对器两条 + 池级 checker 判」 | ✓ |
| L175 `invariants.md:57/58/31` | I-7.7 / I-7.8 / I-1.8 三条登记位 | ✓ 三个编号与三个行号一一对应 |
| L177 `three-way-inference.md:147` | 「一条腿只抽一次样不算一次观测」标题 | ✓ `grep -n` 命中同一行 |
| L200 `checks-owed.md:480` | C561 验收句 | ✓ |
| L213 `agent-common.md:53` | 「崩溃点测试不衡量时间成本」 | ✓ |
| L214 `r1-main-verification.md:38` | 「约简与否是用户的定……甲二的等价还没证出来」 | **✗ 实际在第 40 行**，见「发现」第 2 条 |
| L227 `checks-owed.md:445` | C513 条目 | ✓ |
| L229 `mutations.tsv:306-308` | C507 的三条变异 | **✗ 实际在主树第 268-270 行、冻结快照第 274-276 行**，见「发现」第 3 条 |
| L229 `_background.md:12` | N2 三样零轮形态 | ✓ |
| L232 `_background.md:41` | 「两条攻方腿不重叠」 | ✓ |
| L242 `checks-owed.md:515` | C507 条目 | ✓ |

小计：核 19 处（`invariants.md` 三个编号按一处引用记一次），✓ 17，✗ 2。

## 四、云端辩方（Sonnet）：产物核对

报告里逐字引用的 5 处日志摘录，在模型目录 `outputs/` 下逐字符核对（`grep -n` 命中同一行）：

| 报告行号 | 产物文件 | 结果 |
|---|---|---|
| L70 | `outputs/q9-baseline-unpatched.log` | ✓ `Q9 seq=UOOUOMSU ... elapsed_seconds=372.5` 逐字命中 |
| L82-83 | `outputs/n13-sigma-full-and-arm-a2-patched.log` | ✓ `N13_SIGMA_FULL`、`N13_ARM_A2` 两行逐字命中 |
| L119 | `outputs/q1-120-seed11-patched.log` | ✓ `Q1_TOTAL ... elapsed_seconds=1270.4` 逐字命中 |
| L130-136（7 行 ARMS） | `outputs/q10-forced-zero-reuse-window-patched.log` | ✓ 7 行逐字命中 |
| L170 | `outputs/n12-150-seed53-patched.log` | ✓ `N12_TOTAL ... elapsed_seconds=2054.4` 逐字命中 |

跨轮比对（报告称与第二轮攻方产物「逐字一致」）：

- L73-74：`q9-baseline-unpatched.log` 与 `research/prompts/m2-layer0-scale-r2-opus-model/outputs/q9.log`
  第 17 行比对，除 `elapsed_seconds` 外逐字一致（`Q9 seq=UOOUOMSU sigma=79 sigma_states=262144
  today_red=32768 sector_wise_red=0 differ=32768`）。✓
- L109-110：`q7_exemption_chains_on_fixed_scripts` 在 r2-opus-model 里的 7 行（B/C/D/E/E18/H1/H2）
  candidates 全 0，与本轮静态扫描候选数一致。✓
- L122-123：`Q1_TOTAL streams=120 streams_with_non_monotone=0 cheap_checked=51720` 与
  `m2-layer0-scale-r2-opus-model/outputs/q1-sweep120.log` 逐字一致（`elapsed_seconds` 除外）。✓
- L139-140：q10 的 7 行 `arm_a_bad`（31/23/39/39/23/31/51）与 `arm_a2_bad`
  （43/32/54/54/32/43/63）与 `m2-layer0-scale-r2-opus-model/outputs/q10.log` 第 66/106/170/234/274/326/387
  行逐字相同。✓

## 五、云端辩方（Sonnet）：复跑

草稿目录：`/tmp/claude-1000/m2-layer0-scale-r3-verifier/`（重跑用的仓副本已在核完之后删除，
只留日志于 `kept-logs/`）。

**发现 0（复跑命令本身按文档默认参数不可复现）**：`bash research/prompts/m2-layer0-scale-r3-sonnet-model/rerun.sh
/tmp/claude-1000/l0scale-r1-frozen <草稿目录>`（脚本自带的默认调用形态）在全新副本上原样跑，
在 `patch -s -d "$scratch/repo" -p1 < .../crash-sector-wise.patch` 这一步失败退出：

```
File to patch:
Skip this patch? [y]
1 out of 1 hunk ignored
SCRIPT EXIT CODE: 1
```

原因：补丁文件头写的是绝对路径（`--- /tmp/claude-1000/l0scale-r1-frozen/crates/.../crash.rs`），
不是 `a/`、`b/` 前缀形态；`-p1` 只剥掉一层，剥不到 `crates/...`，实测要 `-p4` 才能命中
（`patch --dry-run -p4 ...` 输出 `checking file crates/singlefs-harness/src/crash.rs`，
`-p0`到`-p3` 均报「can't find file to patch」）。`rerun.sh` 开了 `set -euo pipefail`，
这一步失败即整个脚本退出，**六个测试用例一个都不会跑，`outputs/` 不会被重新生成**。
这不是我的环境特有：冻结副本路径与本轮所有腿共用的路径完全相同。

用 `-p4` 手动修正补丁应用之后（其余步骤——rsync、拷贝探针文件、cargo test 命令——照 `rerun.sh` 原样），
派发提示点名要看的两组数：

| 派发提示要看的数 | 命令 | 我的复跑结果 | 报告/产物里的数 | 结果 |
|---|---|---|---|---|
| 补丁后 σ=79 段 32768→0、甲二红 0 | `PROBE_SEQS=UOOUOMSU cargo test --release -p singlefs-harness --test n1_sonnet_probe -- --exact n13_patched_sigma_full_enumeration_and_arm_a2 --nocapture`（16 线程、`run-with-memory-cap.sh 8G`、`capped.sh 16`） | `record_claimed_state_missing_unit=0 violations=0 ignored=0 root_without_record=0 checker_violations=0`；`N13_ARM_A2 states=334 red=0` | 与 `outputs/n13-sigma-full-and-arm-a2-patched.log` 逐字段相同（仅 `elapsed_seconds` 因线程数不同而不同：570s vs 1370s） | ✓ |
| 7 条回收窗口置 0 的历史与补丁前逐字相同 | `PROBE_SEQS=ZOOOO,OZOOO,UZOOOSO,ZSOSOS,MZOOO,UOZOSOS,ZOUZOO cargo test --release -p singlefs-harness --test n1_sonnet_probe -- --exact q10_forced_zero_reuse_window_arms --nocapture` | 7 行 `ARMS ...` 与 `outputs/q10-forced-zero-reuse-window-patched.log` **逐字节相同**（`diff` 退出码 0） | 同上 | ✓ |

结论：**在按 `-p4` 手工修正之后，报告点名的两组数都能独立复现**；但报告与 `rerun.sh` 头部注释都没有提到
需要非默认的 `-p` 层级，**按文档字面复跑会在第一步就失败**，这一条算「复跑命令按给定参数不可复现」，记入下面第六节「发现清单」，不影响已经用修正参数验证过的两组数值本身。

## 六、云端辩方（Sonnet）：发现清单

**发现 1（归属误标，行号本身对）**：报告 L51-52「探针文件：把第二轮攻方的 `opus_r2_probe.rs`
（...，第二轮判决明写「这一轮可以接着用」，`_m2-layer0-scale-r3-background.md:21`）」——引的行号
（background.md 第 21 行）确实逐字含有「这一轮可以接着用」，但这句话**只出现在背景材料里**，
不出现在它归属的「第二轮判决」（`m2-layer0-scale-r2-main-verification.md`）正文里：

```
grep -n "这一轮可以接着用" research/prompts/m2-layer0-scale-r2-main-verification.md
research/prompts/m2-layer0-scale-r2-opus-output.md
```

两处 0 命中。这句话是背景材料编写者（不是第二轮判决本身）加的一句衔接，把它算成「第二轮判决明写」
是把背景材料的话当成了判决原文。判据行号 ✓，但归属陈述与实际来源不符，单列，不计入 ✗ 也不计入 ✓。

**发现 2（`r1-main-verification.md:38` 行号错）**：报告两处（L214）引「约简与否是用户的定……甲二的等价还没证出来」
并标 `r1-main-verification.md:38`，该文件第 38 行实际是「## 四、交用户的（第三轮之后交，现在不交）」这一节标题，
与引文无关；`grep -n "约简与否是用户的定"` 命中该文件第 **40** 行，内容逐字相符。判 ✗，原文件实为第 40 行。

**发现 3（`mutations.tsv:306-308` 行号错，源头在 kb 本身）**：报告 L229 引「`crates/mutations.tsv:306-308`
的三条变异」，这三个数字来自 `.claude/kb/checks-owed.md` 第 515 行（C507）自己的文字「三条变异在
`crates/mutations.tsv` 第 306–308 行」——但这三条变异（用例名
`a_unit_whose_only_later_write_to_the_same_slot_never_landed_is_reported_missing_by_the_record_checker`）
`grep -n` 命中的实际位置是**主树第 268 行、冻结快照第 274 行**（各自往下两行是另外两条变异），不是 306-308：

```
主树 268: C507（...）：复用豁免不看更晚那次写持没持久（回到落地版，...）
冻结快照 274: 同上
```

背景材料与附录里这句同样抄的是「306–308」（`_background.md:859`、`_appendix.md:443`），说明这个错误
行号是 2026-09-23 写下 C507 条目时留下的、此后 `mutations.tsv` 增长导致行号漂移，**辩方与背景材料都只是
照抄了这条已经过时的 kb 引用，没有现查**。判 ✗（对 crates/ 文件用冻结快照核，非 kb 行号漂移例外）。
云端攻方（Opus）没有引用 `mutations.tsv`，不受此影响。

**云端辩方小计**：引用核了 19 处，✓ 17，✗ 2；产物摘录核了 5 处，✓ 5；跨轮产物比对核了 4 处，✓ 4；
复跑 2 条命令，数值本身 ✓ 2，但按文档给定参数复跑本身 ✗ 1（`-p1` 应为 `-p4`）；归属误标单列 1 处
（不计入 ✓/✗）。

## 七、云端攻方（Opus）：引用核对表

判快照的规则同上（`crates/` 与历史 `research/prompts/` 对主树/冻结快照，`.claude/kb/`、`.claude/gate.d/`、
`research/scripts/` 因无本轮快照对主树核，主树是当前唯一可查的版本，均命中，无需上「分不清」）。

| 引用 | 内容 | 结果 |
|---|---|---|
| `checks-owed.md:480` | C561 定义 | ✓ |
| `opus_r2_probe.rs:1280` | `persisted[later] && l.device == w.device && ...`（字面代码引用） | ✓ `grep -n` 命中同一行 |
| `decisions/13-验证路线.md:131` | 「需要第二个输入的核对归记录核对器」 | ✓ |
| `crash.rs:656-657`（冻结） | 记录核对器入参三样的文档注释 | ✓ |
| `crash.rs:590`（冻结） | `if *is_persisted && write.device.0 == device && write.kind == StepKind::UnitWrite {` | ✓ 字面代码逐字符命中 |
| `second_transaction_step_zero_layer0.rs:1289`（冻结） | 「手搭的状态（层 0 段枚举产生不出它…」 | ✓ |
| `r2-main-verification.md:31` | 「断点续跑收严（实六实现，被攻过零轮）」 | ✓ |
| `crash.rs:955`（冻结） | `fn absorb_following_slice` | ✓ |
| `first_transaction_step_seven_layer0.rs:466`（冻结） | `read_set_counts.states_judging_allocation_generations(),` | ✓ 字面代码命中 |
| `.claude/gate.d/stage-inputs.tsv:20` | 54 号登记 `crates/ Cargo.toml Cargo.lock` | ✓ |
| `research/scripts/admission.py:271` | `git ls-files -z --cached --others --exclude-standard` | ✓ |
| `admission.py:276`（引用两次） | `os.path.isfile` 过滤 | ✓ |
| `Cargo.toml:17`（主树与冻结） | `overflow-checks = true` | ✓ 两处都命中 |
| `.claude/gate.d/54-layer0-replay.sh:224` | `layer0_full_base="$(mktemp -d)"` | ✓（嵌在 echo 字符串里，字面仍命中） |
| `tests/common/mod.rs:40`（冻结） | `std::env::temp_dir()` | ✓ |
| `second_transaction_step_zero_layer0.rs` 第 456/457/458/526/534/535/538/589/977/1156 行（冻结） | prepare 调用、expand 谓词、`.content = third_content()`、五处枚举调用 | ✓ 十处全部逐字符命中，`457` 与 `535` 两行逐字相同（与「逐字相同」的说法一致） |
| `l7-resume-prototype-crash-rs.patch` 第 56/79/82 行 | `slice_line`、`Layer0Tally::default()`、`unwrap_or(0)` | ✓ 三处全部命中 |
| `.gitignore:4,5`（主树） | `*.img`、`*.qcow2` | ✓ |

小计：核 18 组（部分组含多行）、展开为 30 个具体行号点，全部 ✓，0 ✗，0 分不清。

## 八、云端攻方（Opus）：产物核对

`SHA256SUMS`（模型目录自带）对目录里 13 个文件 `sha256sum -c` 全部 `OK`。报告正文逐字引用的日志行
（`r3a-uooouomsu.log` 的 R3A/R3RULE/R3STATE 行、`r3a-sweep23.log` 的 21 条 `l_nonzero_over_u2_range=0`
统计、`r3b.log` 的三行 R3STATE、`rerun-summary.log` 的四行统计与 `images_identical` 行、
`build-env-scenarios.log` 的 11 行 SCENARIO）逐一 `grep -n` 命中同一份文件，字面全部相符，含
23 条历史列表本身数了一遍等于 23（`OSMUMUOURSMMUR,...,USOUSMSU` 用 `tr ',' '\n' | wc -l` = 23）、
`r3a-sweep23.log` 里 `grep -c "l_nonzero_over_u2_range=0"` = 21，与「21 条里 21 条」的说法一致。

## 九、云端攻方（Opus）：复跑

```
bash research/prompts/m2-layer0-scale-r3-opus-model/rerun.sh /tmp/claude-1000/l0scale-r1-frozen <草稿目录>
```

按文档给定参数、默认 `-p` 层级（这份 patch 只改两个函数可见性、不涉及路径剥离问题），**原样两次独立跑通**
（一次前台直接跑、一次调用 `rerun.sh` 本体跑，草稿目录都在 `/tmp/claude-1000/m2-layer0-scale-r3-verifier/`
下，已在核完后删除，日志摘要留存于 `kept-logs/`），耗时与报告估计的「约 3 分钟」一致。

派发提示点名要看的两组数：

| 派发提示要看的数 | 我的复跑结果 | 报告/产物里的数 | 结果 |
|---|---|---|---|
| 状态 X 与 Y 的崩溃镜像逐字节相同（21/21，差异扇区 0） | `images_identical(X,Y)=true differing_sectors=0`（单条历史）；`images_identical(X,Y)=true in 21 of 21 histories with a u2`（23 条历史扫描） | 与 `outputs/r3a-uooouomsu.log`、`outputs/rerun-summary.log` 逐字节相同 | ✓ |
| 今天的判法把 l 整份开脱、整份层 0 判绿（21/21） | `Yp_only:Today=0 ... Yp_only:layer0_red=0`（`Today` 判据判缺席 0/21＝全部开脱，`layer0_red=0`＝层 0 判绿 0/21＝全部绿），`Yp_only:SecPers=21 SecPers513=21 SecDisk=21 SecDisk513=21 SecDiskRec=21 SecDiskRec513=21`（七种按扇区判法全红） | 与 `outputs/rerun-summary.log` 逐字节相同 | ✓ |

除时间戳、编译进度行（依赖本地 target 目录缓存状态）与 `elapsed_seconds` 外，两次复跑与模型目录里
`outputs/r3a-uooouomsu.log`、`outputs/r3b.log`、`outputs/rerun-summary.log`、`outputs/build-env-scenarios.log`
逐行 `diff` 全部一致（`diff` 退出码 0，除时间戳行外）。**这条腿的复跑命令按文档给定参数原样可复现**，
未发现类似辩方那样的路径级失败。

## 十、本地攻方：转述核对表逐条核

`m2-layer0-scale-r3-local-attack-translation-audit.md` 列了 22 行「原文文件:行」引用（四节表格），
逐条按「行号在不在、抄的是不是原文」核对，判快照规则同上：

| 节 | 引用条数 | 结果 |
|---|---|---|
| 一（Context 段与 Table 1） | 5 | ✓ 5：`decisions/13:71`、`r1-main-verification.md:28`（×2 处引用同一行）、`segments.rs:145`、`segments.rs:151-155` 全部逐字符命中；括注补充（屏障过滤、`not under this one`）经现查代码/原文对比后属实，非无据添加 |
| 二（Table 2） | 4 | ✓ 4：`opus_scale_probe.rs:200`、`p1.log:22`、`records/2026-09-24-里程碑二收尾调度.md:162` 两处子句全部逐字符命中 |
| 三（Table 3） | 8 | ✓ 8：`records/...:162` 另外三个子句、`r2-main-verification.md:38`、`r1-main-verification.md:45`、`r2-main-verification.md:10` 全部逐字符命中 |
| 四（Table 4） | 5 | ✓ 5：`r2-opus-output.md:314`、`r2-main-verification.md:9`、`r2-main-verification.md:16`（含 txg 25、2 个原地写、16 个 COW 写、32768/262144/334/0 全部数字核对） |

共 22 处，**0 处行号误写成背景材料行号，0 处摘句失真，0 ✗**。核对表里「首稿缺的」栏点出的三处遗漏
（Context 段的撕裂语义、D18/D13 分项编号引用、UOOUOMSU 完整步骤括注）与「定稿说明」栏给的不抄理由
（与本题算术无关）逐条核对，理由能被验证为真（现查过对应的 kb/decisions 原文，撕裂语义与分项编号
确实与十道纯算术题无关）。

**局限**：转述核对表本身是本地攻方腿自己产出的，不是独立第三方核的；这里做的是核对表**逐行的行号与
内容**本身，不代表提示原文 `local-attack.md` 里除核对表覆盖之外还有没有别的转述（现查发现两者引用集合
一致，见下）。

`research/prompts/m2-layer0-scale-r3-local-attack.md` 正文里的 `Source:` 标注与核对表的「原文文件:行」列
逐一比对，两者引用的文件与行号集合一致，无核对表未覆盖的额外引用。

## 十一、本地攻方：运行记录与字词损坏闸核对

运行记录 `-local-attack-runlog.md` 的字数声明与判定，用命令现核（不采信手数）：

| 样本 | 记录声称的词数 | `wc -w` 现算 | 记录声称的判定 | 现跑闸重判 |
|---|---|---|---|---|
| s1 | 513 | 513 ✓ | 干净（退出 0） | `oov-check.py` 绿；`corruption-check.py` 绿 |
| void1（占 s2 号第一次，作废） | 517 | 517 ✓ | 拼接 `statesstates`，退出 5 | `oov-check.py` **红**（`拼接: statesstates(=states+states)`）；`corruption-check.py` 绿（该检查器不查拼接，符合「一个检测器只查得了一类」） |
| void2（占 s2 号第二次，作废） | 422 | 422 ✓ | 实词自复读 `higher`，退出 5 | `corruption-check.py` **红**（`实词自复读: higher`） |
| s2（第三次，占号成功） | 652 | 652 ✓ | 干净（退出 0） | `oov-check.py` 绿；`corruption-check.py` 绿 |

四个词数与两条闸重判结果均与运行记录一致，**0 处不符**。作废样本文件权限 `-rw-------`（0600）与运行记录
「模式 0600，不算样本」一致（`ls -la` 现查）。

`m2-layer0-scale-r3-local-attack-output-void1.md`、`-void2.md` 两份文件因权限 0600、且运行记录已注明
「被损坏闸判红作废」，这一轮判决不应把它们当样本引用——现查 `m2-layer0-scale-r3-sonnet-output.md`、
`m2-layer0-scale-r3-opus-output.md` 均未引用这两份文件。

**干净样本达到 2 份（s1、s2）即停止**，与规则「一条腿只抽一次样不算一次观测」的「两次都没打中才记没打中」
不冲突——这里核的是字词损坏闸的机械判定，不涉及模型答复内容的「打中/没打中」判断，那部分不归本报告，
按定义留给主 agent。

## 十二、总计数

| 腿 | 引用核数 | ✓ | ✗ | 产物核数 | ✓ | 复跑数 | 命令本身可复现 | 数值匹配 |
|---|---|---|---|---|---|---|---|---|
| 云端辩方（Sonnet） | 19（另 1 处归属误标单列不计入✓/✗） | 17 | 2 | 9（5 处自身摘录 + 4 处跨轮比对） | 9 | 2 | 1/2（`-p1` 应为 `-p4`，文档给的参数不可复现） | 2/2（手工改参数后数值全部相符） |
| 云端攻方（Opus） | 30 个行号点（18 组） | 30 | 0 | 若干（SHA256SUMS 13 项 + 逐行摘录全部核对） | 全部 ✓ | 2（前台一次、脚本一次） | 2/2 | 2/2 |
| 本地攻方（转述核对表） | 22 | 22 | 0 | — | — | — | — | — |
| 本地攻方（运行记录） | — | — | — | 4 处词数 + 2 处闸判定 | 6 | — | — | — |

**没有出现「分不清：文件可能在腿交回之后被改过」的情形**——两处 ✗（`r1-main-verification.md:38`、
`mutations.tsv:306-308`）都定位到了确切的替代行号，不是行号漂移导致的模糊。

## 十三、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 没有复跑辩方 `rerun.sh` 里另外 4 条测试（`n11`、`q7`、`q1` 完整 120 条、`n12` 150 条、`q9` 基线）——
  这些数已经通过「产物核对」（逐字比对报告摘录与 `outputs/` 原文、以及与第二轮攻方历史产物比对）覆盖，
  派发提示点名要复跑的只有 σ=79（n13）与 7 条回收窗口置 0（q10）两组，已复跑。
- 没有复跑攻方 `rerun.sh` 之外的任何东西（例如没有自己另造扇区判实现去验证 1.2 节「三样入参下两条要求
  互相矛盾」这句话本身的证明），那是推理正确性，不归本报告。
- 没有判断本地攻方 s1/s2 两份样本对十道算术题的具体作答对不对（是否正确分类「量的/推的」），按定义
  这部分留给主 agent 逐条现查。
- 没有跑任何名字带 `layer0` 的测试目标、没跑 54 号。
- 没有编译或跑 `gate.sh`，没做 git 写操作。
- 草稿目录已清理：删除了 `/tmp/claude-1000/m2-layer0-scale-r3-verifier/{sonnet-rerun,sonnet-rerun-q10,
  sonnet-rerun-direct,opus-rerun}` 四份仓副本（各 800M/800M/5.9M/160K），只留 `kept-logs/`（5 份小日志，
  共约 300K）与 `self-proof-shifted-citation.txt` 作为佐证。

## 十四、报告与产物路径

- 本报告：`research/prompts/m2-layer0-scale-r3-verifier-output.md`。
- 草稿目录：`/tmp/claude-1000/m2-layer0-scale-r3-verifier/`（`kept-logs/` 与判别力自证文件）。

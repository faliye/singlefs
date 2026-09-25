# m2-safety-r1 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

本轮由第三任核查员接手（前两任被会话中断打断，原因是别的进程把整机内存吃满，不是核查本身）。
第一步核现场：草稿目录 `/tmp/claude-1000/m2-safety-r1-verifier/` 里已有的产物（`crates-check.log`、
`kb-check.log`、`kb-check-retry.log`、`opus-sha-check.log`、`sonnet-sha-check.log`、
`opus-rerun/`、`sonnet-rerun/`）原样保留、不重做；两份交接摘要第五节各记的 1 个未收到完成通知的
后台任务，按进程号查均已不在（`batht77bh` exit=1、`b2338zabg` exit=0、`bbr0jlitj` exit=0、
`bavfcomy8` killed、`bec65is28` exit=0），其中 `bavfcomy8`（`SONNET_S4_NO_DEFER_TERM=1` 的
z19b 首次重跑，日志 `z19b-no-defer-term.log` 只有 28 行、停在 k=11）已被第二任重跑并跑完
（`bec65is28`，日志 `z19b-no-defer-term-retry.log`，104 行，`test result: ok ... finished in 723.95s`），
不再重跑；其余四个都已跑完并留有完整日志。

## 〇、判别力自证（沿用前任产物，原样结果）

草稿目录 `/tmp/claude-1000/m2-safety-r1-verifier/self-test/citation-mutated.txt`：

```
原引用（Opus 报告第 53 行）：冻结副本 `root_ring.rs` 第 115 行 `slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),`
判别力自证：把行号加 1 → 第 116 行
```

命令与原样输出：

```
$ sed -n '116p' /tmp/claude-1000/m2-safety-r1/tree/crates/singlefs-core/src/root_ring.rs
    }
```

引用抄的内容是 `slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),`，
第 116 行实际是 `}`，两者不同 → **判 ✗**。方法可分辨，往下按同一套方法核。

## 一、sha256 与快照核对

| 检查 | 命令 | 结果 |
|---|---|---|
| `crates/` 快照完整性 | `sha256sum -c .../crates-sha256.txt`（草稿 `crates-check.log`，前任已跑） | 122/122 OK |
| `.claude/kb/` 快照完整性（首次，cwd 错） | 同上（`kb-check.log`） | 0/5 OK（`No such file or directory`，cwd 在 `/tmp/claude-1000/m2-safety-r1` 而不是 `kb-snapshot/`） |
| `.claude/kb/` 快照完整性（重试，cwd 对） | 前任在 `kb-snapshot/` 下重跑（`kb-check-retry.log`） | 5/5 OK |
| Opus 模型目录 `SHA256SUMS` | `opus-sha-check.log` | 257/257 OK |
| Sonnet 模型目录 `SHA256SUMS` | `sonnet-sha-check.log` | 10/10 OK |
| Opus 报告现 sha256 与交回时报的对比 | 本轮现跑 `sha256sum m2-safety-r1-opus-output.md` | `5c01aa55c50bbe861b8855db0cee386bcf6f160cb0b9b9ff6207471e46f4d226`，与交接摘要给的交回哈希逐字节相同 |
| Sonnet 报告现 sha256 与派发提示给的「现」哈希对比 | 本轮现跑 `sha256sum m2-safety-r1-sonnet-output.md` | `d019b35737866d948651fc8aa9f3501118bbd4c3d00a358755473076ab1ffb50`，前缀与派发提示给的 `d019b35737866d94…` 相同（该报告交回后主 agent 只改了第 108 行的路径，现哈希与「交回时」哈希本就不同，是预期内的） |

Sonnet 报告第 108 行只改路径的说明：现查第 106–109 行，`admission.rs.patch` 第一段 hunk头
`@@ -363,10 +363,15 @@`（改的是原文件 `admission.rs:366-373` 那段 `own_terms` 数组，`terms.deferred`
换成按环境变量置零的 `deferred_term_s4_candidate_1`）——与报告第 107 行「原文件 `admission.rs:366-373`
那段 `own_terms` 数组」的说法一致，补丁确实指向那一段，只改路径不改内容的说明成立。

## 二、云端攻方（Opus）`m2-safety-r1-opus-output.md`

### 2.1 派发提示点名的优先复跑项

| 引用 | 核的结果 | 命令 |
|---|---|---|
| S1-乙「离环 0 段」（表 1–3，草稿 `opus-rerun/logs/s1chk/s1w-yi-4g-*.tsv`） | ✓ 320 行全 `target_in_ring=true` | `grep -oE "target_in_ring=(true\|false)" logs/s1chk/s1w-yi-4g-*.tsv \| sort \| uniq -c` → 全 `true` |
| S1-乙「重试全部做成」 | ✓ `retry=` 与 `writable=` 全部是 `ok#3`/`ok#4`，没有任何 `NotInRing` 等错误成员 | `grep -oE "retry=[a-zA-Z0-9#]+" ...` / `writable=...` |
| S1-乙「checker 960 次全绿」 | ✓ 320 行 × 3 个 checker 字段 = 960 次，全部 `[]` | `grep -oE "checker_(first\|retry\|writable)=\[\]" logs/s1chk/s1w-yi-4g-*.tsv \| wc -l` → 960 |
| S1「今天、乙、甲+乙三臂各 320 段，判红 0/0/0」（报告第 171 行） | ✓ 乙臂本轮独立复核如上；今天/甲+乙两臂用 Opus 模型目录里 sha256 已核对过的原始产物核对（草稿目录未重跑这两臂），640 行（8 文件×40+8 文件×40／文件各 41 行含 1 控制行）checker 三字段同样全 `[]` | `grep -ohE "checker_(first\|retry\|writable)=\[[^]]*\]" logs/s1chk/s1w-jiayi-*.tsv logs/s1chk/s1w-today-*.tsv \| sort -u` → 只有 `checker_first=[]`/`checker_retry=[]`/`checker_writable=[]` 三种 |
| S2「收窄补法漏补，C332 撤销的历史」（k=30、坏槽开着那一行，报告第 194–199 行表） | ✓ 四臂（对照/今天/S2-甲/S2-丙）逐格与报告表一致：对照与 S2-甲 `landed=Some((1,5))`、`C332_hit=true`；今天与 S2-丙 `witness_after=[(2,1,3)]`、`landed=None` | `grep -n "k=30" opus-rerun/logs/s2hole/*.log`（原样输出见下） |
| S3-乙「行形逐字相同」三行原样输出 | ✓ `diff` 退出码 0 | `diff report-s3-lines.txt rerun-s3-lines.txt` |

S2 收窄反例原样输出（坏槽开着那一档，`degrade=true`）：

```
s2hole-none.log:87:      witness_before_final=[] final_mount=Ok((3, 7)) witness_after=[] landed=Some((1, 5)) C332_hit=true
s2hole-today.log:87:     witness_before_final=[] final_mount=Ok((3, 7)) witness_after=[(2, 1, 3)] landed=None C332_hit=false
s2hole-narrow.log:87:    witness_before_final=[] final_mount=Ok((3, 7)) witness_after=[] landed=Some((1, 5)) C332_hit=true
s2hole-clearflag.log:87: witness_before_final=[] final_mount=Ok((3, 7)) witness_after=[(2, 1, 3)] landed=None C332_hit=false
```

这与报告第 196–199 行的表（对照/今天/S2-甲/S2-丙）逐字段一致。

### 2.2 文件:行引用逐条核（against `/tmp/claude-1000/m2-safety-r1/tree/crates/` 与 kb-snapshot）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `root_ring.rs:115`（整行抄，报告第 53 行） | ✓ 逐字相同 | `awk 'NR==115' root_ring.rs` |
| `mount.rs:2327`（`first_txg = first_txg_of_new_instance(...)`） | ✓ | `awk 'NR==2327' mount.rs` |
| `transaction.rs:2729-2743`（D19 已定项 5 重读一次） | ✓ 内容与「先重读一次、两次都没对上才隔离」相符 | `awk 'NR==2729,NR==2743' transaction.rs` |
| kb `23-journal的角色与格式.md:387`（整行抄，S1 判读引用） | ✓ 逐字相同 | `awk 'NR==387' 23-journal的角色与格式.md` |
| kb `23-journal的角色与格式.md:378`（转述：回退候选集与写行落点） | ✓ 内容与转述一致 | `awk 'NR==378' ...` |
| kb `18-块里携带什么信息.md:314`（转述：预演不读盘核） | ✓ 内容与转述一致，含「差别见 C542」 | `awk 'NR==314' 18-块里携带什么信息.md` |
| kb `18-块里携带什么信息.md:305`（转述：实例 0 不写行） | ✓ 内容与转述一致 | `awk 'NR==305' ...` |
| `records/2026-09-24-里程碑二收尾调度.md` 第 145 行第三列 ②（转述：取号之后发布失败不许离环） | **✗ 位置对不上，但记「分不清」不记硬 ✗** —— 见下 | 见下 |

**这一条单列说明**：`records/` 不在派发提示给的两份快照（`crates/`、`.claude/kb/`）射程内，没有给这份文件的快照。
现查 `git status --short -- "records/2026-09-24-...md"` 显示这份文件当前有 **未提交的改动**（`73 insertions(+), 11 deletions(-)`，
`git diff` 显示在文件顶部插入了一整节「## 零、收尾出口（用户 2026-09-24 JST 24 点后定的九步）」，约 17 行），
而这份文件历史上只有一次提交（`e980a21`，2026-09-24T10:36 UTC）。Opus 报告落笔在「2026-09-25 04:3x UTC」，
在这次提交之后；工作区当前这份改动是本机同时在跑的别的会话（收尾调度）留下的、尚未提交，
无法确认 Opus 读到的是提交时的版本还是中途某个未保留快照的版本。
现查**当前**工作区第 146 行才是「取号之后发布失败，不许让回退目标离环、管理员没法重试」那句
（`| fsync 失败掉盘与 C542（用户 2026-09-25 JST 12:1x）| ... ② 推翻 2026-09-24「C542 记欠账不改」：
回退必须能退回去——取号之后发布失败，不许让回退目标离环、管理员没法重试...`），第 145 行是空行；
背景材料三份文件（`_m2-safety-r1-background.md`、`-appendix.md`、`-body.md`）里都没有把这条转述
钉在数字「145」上（只写「第三节『fsync 失败掉盘与 C542』那一行」，不带行号），所以也不是「误写成
背景材料行号」那一类。**记「分不清：文件可能在腿交回之后被改过」，不计入 ✗，也不计入 ✓。**

## 三、云端正推（Sonnet）`m2-safety-r1-sonnet-output.md`

### 3.1 派发提示点名的优先复跑项

| 引用 | 核的结果 | 命令 |
|---|---|---|
| S4 归因：检查时点 + defer 重复扣（报告第 67–99 行 `S4TRACE` 原样输出与叙述） | **部分 ✗，见下** | 见下 |
| 候选 A「384 槽更糟」（k=20、21 回退 0/24，报告第 120–125 行） | ✓ 逐字节相同 | 见下 |
| 基线（240/256/384 一次会话放行次数 5/6/10）与候选 A（11/11/21） | ✓ 与报告第 113 行数字相符 | 见下 |

**候选 A「384 槽更糟」**：草稿 `sonnet-rerun/logs/z19b-no-defer-term-retry.log` 是本轮开工前
第二任已经完整跑完的复跑（`bec65is28`，退出码 0，`finished in 723.95s`，本轮不重跑）：

```
$ grep "width=384 selector=2999 k=21" z19b-no-defer-term-retry.log
Z19-B width=384 selector=2999 k=21 session_all_admitted=true next_writable_mount=MountError::SpaceAdmissionRefusedBeforeAcquisition again=MountError::SpaceAdmissionRefusedBeforeAcquisition rollbacks_applied=0/24 rollback_members={"MountError::SpaceAdmissionRefusedBeforeAcquisition": 24} tail=["Applied", "MountError::SpaceAdmissionRefusedBeforeAcquisition"] ending=Completed checker_runs=23
$ diff report-384-quote.txt rerun-384-quote.txt; echo $?
0
```

与报告第 122 行引用逐字节相同；k=20 行同样 `rollbacks_applied=0/24`，与报告「k=20、21」的说法一致。

**基线与候选 A 的次数**：`z19b-baseline.log`（无环境变量，第二任之前已跑完）与
`z19b-no-defer-term-retry.log` 里 `overwrites_admitted_in_one_session` 字段：

```
基线：width=240→5，width=256→6，width=384→10（z19b-baseline.log）
候选A：width=240→11，width=256→11，width=384→21（z19b-no-defer-term-retry.log）
```

与报告第 113 行「240/256/384 槽小盘一次会话放行的覆盖写次数从 5/6/10 涨到 11/11/21」逐字段相符。

**S4 归因（本轮新跑，`systemd-run --user --scope -p MemoryMax=20G -p MemorySwapMax=0` 套着跑）**：

```
$ cd /tmp/claude-1000/m2-safety-r1-verifier/sonnet-rerun/tree-s4
$ export CARGO_TARGET_DIR=/tmp/claude-1000/m2-safety-r1-verifier/sonnet-rerun/target-s4
$ systemd-run --user --scope -p MemoryMax=20G -p MemorySwapMax=0 --quiet \
    nice -n 19 bash research/scripts/capped.sh 12 \
    cargo test -q -p singlefs-harness --test s4_trace_probe -- --nocapture \
    > .../logs/s4-trace-probe.log 2>&1
退出码 0（前台跑完，未撞内存上限）
```

（`s4_trace_probe.rs` 的 sha256 与模型目录 `SHA256SUMS` 里登记的 `15c83f72643ad22fa5d872eb67050ddd83dfa6a85f9be5b42bedb827953dd2e6` 一致，是报告点名的那份原件。）

复跑输出里，报告第 71–74 行那段 `==PUBLISH_CHECK==`（`rows0=1`、`allocated=1327104`、`deferred=1064960`、
`available=344064`）逐字节复现（第三次 `PUBLISH_CHECK` 出现在复跑日志第 24 行附近）。

**但报告紧接着第 75–79 行那段 `==MOUNT_CHECK==`（`rows0=2`、`allocated=1556480`、`deferred=1294336`、
`available=-114688`）在复跑里找不到**——复跑日志里跟在那个 `PUBLISH_CHECK` 后面的下一段仍然是
`==PUBLISH_CHECK==`（不是 `MOUNT_CHECK`），`rows0=1`（不是 2），只是 `allocated`/`deferred`/
`mount_time_commitment`/`available` 四个数值恰好与报告的 `MOUNT_CHECK` 段相同：

```
$ grep -n "==PUBLISH_CHECK==\|==MOUNT_CHECK==" logs/s4-trace-probe.log
3:S4TRACE ==PUBLISH_CHECK==
10:S4TRACE ==MOUNT_CHECK==
17:S4TRACE ==PUBLISH_CHECK==
24:S4TRACE ==PUBLISH_CHECK==
31:S4TRACE ==PUBLISH_CHECK==
38:S4TRACE ==PUBLISH_CHECK==
45:S4TRACE ==PUBLISH_CHECK==
52:S4TRACE ==PUBLISH_CHECK==
```

全程只出现一次 `MOUNT_CHECK`（第 10 行，在最开头 `CloseAndMountWritable` 那一步），
`rows0=1`、`allocated=245760`，与报告任何一段引用的数字都不同。原因现查：这份被 sha256 核对过、
报告第 62–65 行给出的命令与测试文件（`s4_trace_next_mount_after_admitted_overwrites`）里，
`operations` 向量只有 `[CloseAndMountWritable]` 加 `k=6` 次覆盖写，**没有第二次挂载**——
全程停在同一个会话里，第 6 次覆盖写本身就在 `PUBLISH_CHECK` 分支被拒绝（`step=6 outcome=Refused`）。
报告第 94–99 行自己也描述了这个「同一会话内再做第 6 次覆盖写」的替代实验，并说它与「关闭再挂载」
的 `MOUNT_CHECK` 数字「逐字节相同」——复跑证实了这半句（两处数值确实相同），但报告开头第 70–80 行
作为主证据展示的那段 `PUBLISH_CHECK` + `MOUNT_CHECK` 配对输出，**用报告点名的命令和测试文件复跑
不出「真的做了第二次挂载」这一半**，只能得到「同一会话再写一次」的版本。

**判 ✗ 的准确位置**：报告第 75–79 行（`S4TRACE ==MOUNT_CHECK==` 起，`rows0=2` 那一段）与
第 62–65 行给出的命令、`s4_trace_probe.rs`（sha256 已核对为原件）复跑不出来；复跑里对应位置
是 `==PUBLISH_CHECK==`、`rows0=1`。第 71–74 行那段 `PUBLISH_CHECK` 本身逐字节可复现，判 ✓。

### 3.2 文件:行引用逐条核

| 引用 | 核的结果 |
|---|---|
| `transaction.rs:4572`、`mount.rs:1866`（两处都调 `admission_reading_before_a_publish`） | ✓ 逐字相同 |
| `admission.rs:581,322,323,138,325,589-592,327,142,147,593-596,487,185,220-221,366-373` | ✓ 全部逐字相同（含 `220-221` 的 `///` 注释、`366-373` 的 `own_terms` 数组） |
| `admission.rs:224`（`⚠️ 它含 defer...` 整行抄） | ✓ 逐字相同 |
| `mount.rs:1851-1853,1841,483,614/728,674,2117` | ✓ 全部逐字相同 |
| `recovery.rs:669-692,681,691`（`choose_root`） | ✓ 逐字相同，含 :681 的 `abandons` 判断、:691 的 `best` 返回值 |
| kb `28-挂载期承诺量.md:19`（准入不等式，整行抄） | ✓ 逐字相同 |
| kb `23-journal的角色与格式.md:380`（回退见证段，整行抄，长段） | ✓ 逐字相同（`diff` 退出码 0） |
| kb `23-journal的角色与格式.md:353`（「定案」句，整行抄） | ✓ 逐字相同 |
| `checks-owed.md:248`（C283，转述） | ✓ 内容一致（C283 行确实含「384 槽的小盘...第 11 次覆盖写起、256 槽的小盘第 5 次起一直被拒」） |
| `research/prompts/e158-s10-runner-report.md:124-125`（整行抄） | ✓ 逐字相同 |
| `research/results/e158-root-choice-repair-2026-09-25-q3-1-s4.out:57`（整行抄） | ✓ 逐字相同 |
| `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:5129-5131,345-350,1217,1270,1286` | ✓ 全部逐字相同 |
| S5 探针产物 `sonnet-model-copy/s5/logs/s5-probe4.log`（报告第 221-226 行引用） | ✓ 34 行 `S5TRACE mount_writable_with_fault error=Recovery(NoValidRoot)`，汇总行 `E7RESULT ...cold_recover_with_fault_failed=34 mount_writable_with_fault_failed=34...` 逐字节匹配报告 |

## 四、本地攻方（Qwen）两次抽样与译文核对表

### 4.1 抽样与自检产物

| 检查 | 命令 | 结果 |
|---|---|---|
| s1 字数/行数（`runlog.md` 声明 1621 词、165 行） | `wc -w`/`wc -l` `-output-s1.md` | 1621 词、165 行，一致 |
| s2 字数/行数（`runlog.md` 声明 1489 词、201 行） | `wc -w`/`wc -l` `-output-s2.md` | 1489 词、201 行，一致 |
| s1 `oov-check.py`（`runlog.md` 声明退出码 0、绿） | `systemd-run ... python3 research/scripts/oov-check.py -output-s1.md` | `绿 ... 生词=0 拼接=0`，退出码 0，一致 |
| s2 `oov-check.py`（同上） | 同上，换 s2 文件 | `绿 ... 生词=0 拼接=0`，退出码 0，一致 |

### 4.2 译文核对表 `m2-safety-r1-local-attack-translation-audit.md` 逐条核

对着 `/tmp/claude-1000/m2-safety-r1/kb-snapshot/` 与 `/tmp/claude-1000/m2-safety-r1/tree/crates/` 现查表里每一行「原文文件:行」：

| 英文项 | 原文文件:行 | 核的结果 |
|---|---|---|
| Fact 2 | `admission.rs:302,322`、`allocator.rs:235-236`、`format/lib.rs:285` | ✓ 全部命中 |
| Fact 8：instance_switch_reserve/one_switch | `28-挂载期承诺量.md:52-58` | **✗ 行号不对**，见下 |
| Fact 9 | `18-块里携带什么信息.md:306` | ✓ 命中（「恢复写的行」条款） |
| Fact 12 | `format/lib.rs:138,144` | ✓ 命中 |
| Fact 15 | `history.rs:74-85` | ✓ 命中，三档小盘注释内容与表述一致 |
| Fact 19 | `root_ring.rs:114-115`、`format/lib.rs:236,254` | ✓ 命中 |
| Fact 20/21 | `23-journal的角色与格式.md:394` | ✓ 命中（① 删除规则、被罩住的删） |
| Fact 22 | `23-journal的角色与格式.md:395` | ✓ 命中（② 写满） |
| Fact 23 | `23-journal的角色与格式.md:399` | ✓ 命中（⑥ 补写） |
| Fact 24 | `make_filesystem.rs:40,206` | ✓ 命中 |
| Fact 25 | `18-块里携带什么信息.md:307` | ✓ 命中（回退写的行、「表的底版」） |

**Fact 8 的 ✗ 详情**：核对表写「`.claude/kb/decisions/28-挂载期承诺量.md:52-58`「已定项 3」附录段
（rows0/pages_of_chain/chain_rewrite/warm_up/one_switch...）」。现查该文件第 52–58 行，内容是
「实验」小节（E19、E82、E141、E150 与 2026-09-23 用户定案 C355/C375），**与 rows0、链重写、暖机、
one_switch 毫无关系**：

```
$ awk 'NR==52,NR==58' 28-挂载期承诺量.md
实验：
- E19（defer 窗口下的假性 ENOSPC）：保留池不进准入时 checkpoint 会卡死——第八项。
- E82（准入的在飞合成）：只读已发布值会同窗超卖——这个式子要配已定项 2。
- E141（切换预留的挂载准入自证）：...
- E150（回退复用被抛弃的根引用的单元）：...
- 用户定案 2026-09-23：准入逐设备合取...
```

真正含 rows0/链重写/暖机/one_switch 的「#### 已定项 3：实例切换的预留」一节实际在**第 79–89 行**：

```
$ awk 'NR>=79 && NR<=86' 28-挂载期承诺量.md
79: #### 已定项 3：实例切换的预留
81: **定案**：预留 = (N_switch + 1) × 一次切换的最坏量，按设备算；一次切换的最坏量 = 实例表链重写 + 暖机空发布。
83: - **链重写**：...rows0 = 挂载时读到的行数加写行那次要写的行数...
84: - **暖机**：至多 R 次空发布 × 现算 c_max...
85: - **N_switch + 1 份**：...
```

背景材料三份文件（`_m2-safety-r1-background.md`、`-appendix.md`、`-body.md`）第 52–58 行都不含这段内容
（现查过，分别是「三、每格要交的」小节标题、`回退见证`一带内容、`S1/S2/S4 另要回答` 小节），
不是「误写成背景材料行号」那一类，就是这份核对表自己把行号写错了 27 行左右，判**硬 ✗**（这份文件在给定快照射程内，不适用「分不清」那条）。

## 五、计数与没做什么

### 计数（按本报告表格逐行数出，不手数）

| 腿 / 类别 | 核了几处 | ✓ | ✗ | 分不清 / 核不动 |
|---|---|---|---|---|
| sha256 与快照（含 108 行补丁定位） | 7 | 7 | 0 | 0 |
| Opus：优先复跑项（2.1） | 6 | 6 | 0 | 0 |
| Opus：文件:行引用（2.2） | 8 | 7 | 0 | 1（`records/` 第 145 行，无快照、文件已被并发改动，记「分不清」） |
| Sonnet：优先复跑项（3.1） | 3 | 2 | 1（S4 归因的 `MOUNT_CHECK` 段复跑不出来） | 0 |
| Sonnet：文件:行引用（3.2） | 14 | 14 | 0 | 0 |
| 本地攻方：抽样自检（4.1） | 4 | 4 | 0 | 0 |
| 本地攻方：译文核对表（4.2） | 11 | 10 | 1（Fact 8 行号错 27 行） | 0 |
| **合计** | **53** | **50** | **2** | **1** |

## 没做什么

- 不判一条打中成不成立、该不该采纳（乙/S2-丙/S3-丙这些候选推荐由主 agent 判）；不核推理本身，只核引用、产物与复跑。
- Opus 报告里表 1、表 2、表 4、表 5 的整批扫描数字（4160/4551/3132/666/1840/14038 段等）与 `campaign` 一节的签名统计没有逐条重新扫描复核——那需要重新起 S1 整轮扫描（几千段 × 多个候选），超出派发提示点名的优先复跑范围，只核了点名的 S1-乙/S2/S3-乙三处与 S1 checker 960 次全绿这一处。
- Opus「越格线索」种子 540096 的 I-8.6 判红没有复现，报告自己也写明未查机理，不属于本轮核查范围。
- Sonnet 候选 B（预先计入本次释放）报告自己写明「未实现、未测量」，无产物可核，不适用。
- Sonnet「S4 里两处结构性口径缺口没有找到能让它们露头的历史」与「候选 A 对回退路径的影响没有系统扫描」——报告自己列在「没做什么」里，不是核查缺口。
- E158 的 239 格全量（1,116,897 对）没有复现，报告自己写明是重型负载、按定义不该由子 agent 全量重跑，只核了报告自己造的 34 格子集与登记产物的对应关系（`cold_recover_with_fault_failed == mount_writable_with_fault_failed` 这个等式两边相符）。
- 本地攻方两份样本（S1、S2 抽样）的答案本身（Fact 1、3、7、9、10 等的引用位置）除译文核对表逐条核过的那些之外，没有把样本正文里每一句代入算术再重算一遍——那是判打中不打中的推理，不归本轮职责；样本自称的行号（`wc -l` 数出的行数、正文里的 t 值序号）按 `runlog.md` 自己的说明「均为模型自给、未核」处理，本轮同样不当依据引用。
- 崩不动的重型项（层 0、QEMU、herd7、`cargo test --workspace`、`gate.sh` 整轮）本轮未涉及，报告里也没有引用它们的产物。
- 没有对 `research/prompts/_m2-safety-r1-background.md`、`-appendix.md`、`-body.md` 三份背景材料本身做「校验路径本身要证明它会红」以外的通读式核对；仅在判「是否误写成背景材料行号」时定点查过。

## 产出路径

- 报告：`research/prompts/m2-safety-r1-verifier-output.md`（本文件）。
- 草稿目录：`/tmp/claude-1000/m2-safety-r1-verifier/`（本轮新增 `logs/s4-trace-probe.log`、`report-384-quote.txt`、`rerun-384-quote.txt`；其余为前两任留下的原样产物，未改动）。

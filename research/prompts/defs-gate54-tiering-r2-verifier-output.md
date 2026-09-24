# defs-gate54-tiering-r2 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

时区：本机 UTC；本报告中出现的时刻均为 UTC（人在 JST，UTC+9，换算时+9 小时）。

## 判别力自证

取 opus 报告 `defs-gate54-tiering-r2-opus-output.md:249` 引用的
`.claude/gate.d/54-layer0-replay.sh:249`（原文 `full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash"`），
在草稿目录副本 `/tmp/claude-1000/defs54-r2-verifier/selftest/54-layer0-replay.sh` 里把待核行号 +1 → 250，
取该行内容：

```
$ sed -n '250p' 54-layer0-replay.sh
  quick_tier_report=""
```

与原引文比对不相等 ⇒ 判 ✗（「行号 250 处实际内容与所引原文不同（抄的原文其实在第 249 行）」）。核查方法能分辨，继续。

## 方法与输入说明

- 三条腿报告 sha256 复核：`sonnet-output.md` 实测 `7c6c4a2f0b622b4b941596126df9809c0dd524e338570f67ed766a148a5890a9`、
  `opus-output.md` 实测 `bf9225572c4656cb3a7b868c7fd2d975b47c72ae8dc6b5e92786bd0756fda877`，
  与派发提示给的两串逐字节相同（MATCH）。本地腿三份文件无对应 sha256 可核（未给）。
- 开工快照 `defs-gate54-tiering-r2-snapshot/sha256sums.txt`（11 个文件的 hash 清单，不含实际内容副本）：
  `sha256sum -c` 结果 9 个 OK、2 个 FAILED——`.claude/main-agent.md`（派发提示已告知，见下）与
  `.claude/kb/checks-owed.md`（派发提示未提及，本报告独立发现；`stat` 显示它于 01:06:11 UTC 被改，
  早于三条腿的大部分工作时段）。全仓 grep 三条腿报告与本地腿三份材料，
  `checks-owed.md` 零次被引用到具体行号，这处漂移不影响任何一条待核引用，不单列。
- `.claude/main-agent.md` 漂移的可核实性：该文件从未被 `git add`（`git cat-file -t 14a43f86…` 报
  `Not a valid object name`），无法直接读回快照内容；改用 `_defs-gate54-tiering-r2-diff.md:444-456`
  给的 `git diff HEAD` 输出反推——该 hunk头 `@@ -41,7 +41,7 @@` 是等行数替换（1 行换 1 行），
  证明本轮自己的改动没有让文件净增删行；`git diff --stat HEAD -- .claude/main-agent.md` 现查
  `8 insertions(+), 3 deletions(-)`，净 +5 行，与 opus 报告称的「第 44 行挪到第 49 行」（Δ=5）吻合，
  独立坐实这处漂移的方向与大小，不只是照抄派发提示的说法。
- 本轮无 `crates/` 改动，快照清单未含 `crates/` 与完整 `.claude/kb/` 树，只给了这一轮实际用到的
  11 个文件；`research/scripts/stage-must-run.sh`、`research/scripts/change-touches-crates.sh`、
  `.claude/singlefs-ai-sop/scripts/gate.sh`、`.claude/agents/gate-triage.md`、`.claude/agent-common.md`、
  `research/prompts/_defs-gate54-tiering-r2-body.md`、`research/prompts/defs-gate54-tiering-r1-*.md` 等
  不在快照清单内，按主树现查（这几份文件没有已知漂移的迹象：`main-agent.md`、`checks-owed.md` 之外
  未发现别的 hash 漂移信号，但没有对它们逐一做快照式确认，风险见「没做什么」）。
- 重点复跑：把 `defs-gate54-tiering-r2-opus-model/` 整份拷进
  `/tmp/claude-1000/defs54-r2-verifier/opus-model-copy/`，`sha256sum -c sha256sums.txt` 21 个文件全 OK，
  `nice -n 19 bash run-all.sh /tmp/claude-1000/defs54-r2-verifier/rerun-runs` 在草稿目录里重跑（未在
  `defs-gate54-tiering-r2-opus-model/` 原目录跑），跑前 `ps` 查无 `qemu-system`/`vm-bench.sh`/
  `e152-file-system-benchmark`/`fio`，仅有别的会话的 `cargo test`/`clippy` 在跑（`impl-m2-*` 系列），
  加 `nice -n 19` 照跑。真仓 `git rev-parse --git-common-dir` 下 `singlefs-layer0*` 复跑前后都是 0 个，
  `git status --short` 里没有新出现的未跟踪项来自本次重跑（仅有的 worktree 都是别的会话早先留下的，
  路径都不在 `/tmp/claude-1000/defs54-r2-verifier/` 下）。

## 一、云端攻方（Opus）核对表

| 引用/复跑项 | 结果 | 命令/依据 |
|---|---|---|
| `.claude/main-agent.md:49`「暂存之后、提交之前跑门禁…」 | **分不清**：快照期在第 44 行（`_defs-gate54-tiering-r2-diff.md` 反推坐实），现查已挪到第 49 行，字面不变（opus 自报，本报告独立复核吻合） | 见上「方法与输入说明」 |
| `.claude/gate.d/stage-inputs.tsv:11` | ✓ | `sed -n '11p'` 逐字相同 |
| `.claude/gate.d/54-layer0-replay.sh:20` | ✓ | `sed -n '20p'` 逐字相同 |
| `.claude/gate.d/54-layer0-replay.sh:249`（判别力自证已用） | ✓ | 同上 |
| `.claude/gate.d/54-layer0-replay.sh:300` | ✓ | `sed -n '300p'` 逐字相同 |
| `.claude/gate.d/54-layer0-replay.sh:8` | ✓ | `sed -n '8p'` 逐字相同 |
| `.claude/singlefs-ai-sop/scripts/gate.sh:75`（连同 76 行 `die`） | ✓ | `sed -n '74,77p'` |
| `.claude/gate.d/54-layer0-replay.sh:11`、`:12` | ✓ | `sed -n '11p;12p'` 逐字相同 |
| `.claude/gate.d/54-layer0-replay.sh:303`（trap 行） | ✓ | `sed -n '303p'` 逐字相同 |
| `.claude/gate.d/54-layer0-replay.sh:23` | ✓ | `sed -n '23p'` 逐字相同 |
| 附：`.claude/agents/crash-verifier.md:23`、`:24` | ✓ | `sed -n '23p;24p'` 逐字相同 |
| 附：`.claude/gate.d/54-layer0-replay.sh:91`、`:147` | ✓ | `sed -n '91p;147p'` 逐字相同 |
| 附：`research/prompts/_defs-gate54-tiering-r2-body.md:61` | ✓ | `sed -n '61p'` 逐字相同 |
| `git log -S'只起了 1 个工作线程' … 找到 fc7942f，同一提交把线程判定加进 54 号、把 crates/ 改多线程` | ✓ | 重跑同一条 `git log -S` 命中同一提交；`git show --stat fc7942f` 确认该提交同时含 `crash.rs` 多线程改动与「门禁 54 号：…只起了 1 个工作线程，判红」 |
| 「真仓一处 git 写都没做；common-dir 里 `singlefs-layer0*` 现查 0 个」 | ✓ | `find "$(git rev-parse --git-common-dir)" -maxdepth 1 -name 'singlefs-layer0*'` 复跑前后均 0 |
| 「.git 在 ext4 上（`df -T` 现查）」 | ✓ | `df -T .git` 输出 `ext4` |
| 模型 `sha256sums.txt` 自洽（21 个文件） | ✓ | 草稿副本 `sha256sum -c` 21 个 OK |
| `outputs/` 里挑样的整行引文（约 10 处，`grep -cxF`） | ✓（全部命中≥1次） | 见下方逐条命令 |
| **重点复跑**：H-A/H-B/H-B2/H-C 五个场景脚本 + g1/g2/g4/g124 四个改法，掩掉时间戳/`tmp.随机后缀`/pid 后与 `outputs/` 逐字节比 | **✓ 9/9 文件全部一致**（sha256 相同） | 见下节详述 |

**逐条 `grep -cxF` 命令**（对 `defs-gate54-tiering-r2-opus-model/outputs/*.out`，全部返回 ≥1）：
`════ H-A [post] Y 在 T1 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED`（1）、
`════ H-A [pre] Y 在 T1 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED`（1）、
`  ✗ 第一个事务那条流：本机 32 核…全量枚举却只起了 1 个工作线程`（13，跨多个场景块共用同一句）、
`    [gate --staged 里 54 号退出码 1]`（7）、
`  ── 这一批：git checkout A -- crates/，暂存区：crates/singlefs-harness/src/crash.rs `（12）、
`  ── --full 还在跑（54 号 pid 2198424），起门禁`（1）、
`  ✗ （别的会话改到一半的 54 号）LAYER0 行 journal_differing 不是 0`（5）、
`  ── 工具链升到 2；工作树干净：0 行 status`（2）、
`  ── 真值（工具链 2 下这份 HEAD 用它自己的 54 号跑全量）`（2）、
`  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）`（4）、
`  ✗ 快档绿了…没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-attack/runs/m`（2）——
全部是产物文件里的原样整行，逐字节命中。

**重点复跑详情**：草稿副本 `run-all.sh` 于 01:59:06–~02:07 UTC 跑完（受同机 `cargo test/clippy` 抢核，
比 opus 自己那趟 8 分钟长，无异常）。9 个产物文件（`h-a-late-stage-54.out`、`h-b-revert-stale-cell.out`、
`h-b2-forced-recheck.out`、`h-c-foreign-run-deletes.out`、`v1-outpath-scenes.out`、
`fix-arms-g1/g2/g4/g124.out`）与入库版逐个 `diff -q` 未掩码时全部「differ」；用
`sed -E 's/[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z/<TS>/g; s#/tmp/[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)*#<TMPPATH>#g; s/tmp\.[A-Za-z0-9]{8,}/tmp.<RAND>/g; s/\bpid[ =][0-9]+/pid <PID>/g'`
只掩掉时间戳、`/tmp` 路径、`mktemp -d` 随机后缀、pid 之后，9 个文件的 `diff` 全部空、`sha256sum`
两两相同（提交哈希前缀在这批产物里本就是内容哈希如 `74f88a29d4fc1e39…`，是确定性输出，不需要单独掩码，
未掩码时这些串本就相同，已在上面的逐条 `grep -cxF` 与整体 diff 里验证过）。9/9 一致，覆盖 H-A 全部
T0–T3 格、H-B 两种漂移 × 5 种用户动作、H-B2、H-C 两种 Z × 4 种先后，以及 g1/g2/g4/g124 在报告大表
（`opus-output.md:387-395`）列出的全部读数。

**Opus 计数**：核了 23 处，✓ 22 处，✗ 0 处，分不清 1 处（main-agent.md 漂移，opus 已自报）。

**Opus「没做什么」**：没有重跑 H-A/H-B/H-B2/H-C 之外任何 opus 未跑的场景（V1-a/b/c/d、trap 覆盖面几行
「推的，没跑」的部分，本报告同样没跑，按定义那些本就是 opus 自称「推的」，不在「重点复跑」射程内）；
没有核 opus 报告里「可达性折扣」「什么会推翻它」这类判断性文字，只核事实与产物；没有编译真实
`cargo`（opus 的装置本就是假 cargo，不涉及）。

## 二、云端辩方（Sonnet）核对表

| 引用 | 结果 | 命令/依据 |
|---|---|---|
| `r1-main-verification.md:38` | ✓（但为该行**尾部子串**，非整行；内容逐字相同、行号正确） | `awk 'NR==38'` 与报告引文并排比对 |
| `r1-main-verification.md:44` | ✓（同样为**尾部子串**）；该文件该行本身缺「主 agent」三字，见下 K2 行 | `awk 'NR==44'`，长度 193 字符，报告只引了后半段 |
| `records/2026-09-19-里程碑二遗留收拢.md:147` | ✓（**整行**，逐字相同） | `sed -n '147p'` |
| `r1-main-verification.md:52-58`（S1–S5 表） | ✓ | `sed -n '52,58p'` 结构与内容吻合 |
| `r1-opus-output.md:374`（标题「## 五、改法各修哪一格」） | ✓ | `sed -n '374p'` 逐字相同 |
| `r1-opus-output.md:257` | ✓ | `sed -n '257p'` 逐字相同 |
| `git log --all -p --follow … experiment-runner.md \| grep` 零命中 | ✓ | 重跑同一条命令，仍 0 |
| `git show 3b60f098:experiment-runner.md \| grep` 零命中 | ✓ | 重跑，仍 0 |
| `grep -rn "singlefs-harness/src/bin/e" r1-*.md` 零命中 | ✓ | 重跑，仍 0 |
| `git diff 3b60f098 -- agent-write-scope.tsv`（两行新增） | ✓ | 重跑同一条 diff，逐字节相同 |
| `git show 3b60f098:54-layer0-replay.sh` 特征（无 worktree/marker/hash，`ROOT="${1:-…}"` 默认调用者根目录，两道跳过判定接 `cargo test --release … --include-ignored`） | ✓ | 重取该 blob，`grep -n` 确认零命中 worktree/marker/input_hash，通读前 50 行确认 ROOT、跳过逻辑与直接跑 cargo |
| `.claude/agents/gate-triage.md:26` | ✓ | `sed -n '26p'` 逐字相同 |
| `.claude/agents/gate-triage.md:39` | ✓ | `sed -n '39p'` 逐字相同 |
| `.claude/agent-common.md:58` | ✓ | `sed -n '58p'` 逐字相同 |
| `.claude/gate.d/54-layer0-replay.sh:254` | ✓ | `sed -n '254p'` 逐字相同 |
| `research/scripts/stage-mine.py` 内 `layer0/full-green/input_hash` 零命中 | ✓ | 重跑，仍 0 |

**独立发现（sonnet 报告未自报）**：sonnet 的 `.claude/main-agent.md:44` 与本报告用相同方法核对，
现查该文件**当前**第 44 行已是别的内容（「要推论：设计判断、取舍…」），sonnet 引的那句文本现查
实际位于第 49 行——与 opus 自报的同一处漂移（第 44 行挪到第 49 行、字面不变）方向、幅度一致，只是
sonnet 报告本身没有像 opus 那样加一句「文件在我跑的时候被改过」的自证。判**分不清：文件在腿交回之
后被改过**（同一处 `main-agent.md` 漂移；sonnet 完成时刻早于该文件 mtime 之后的另一版本，或反之，
未能进一步排序，见「没做什么」），不计入 sonnet 的 ✗。

**观察，非 ✗**：sonnet 报告里对长表格行/长列表项的引用习惯只抄该行的**尾部子串**（`r1:38`、`r1:44`
两处），而不是整行；子串本身逐字不改、行号也对，不构成本轮判据要求的「文件:行号+抄的原文对不上」，
但与「引产物就整行抄」的纪律不是同一回事，记录在案供主 agent 参考，不算作核查项的 ✗。

**Sonnet 计数**：核了 17 处，✓ 16 处（含 2 处「尾部子串但内容正确」的观察项），✗ 0 处，
分不清 1 处（本报告独立发现的 main-agent.md 漂移）。

**Sonnet「没做什么」**：没有重跑任何命令之外的门禁或编译（sonnet 本身没有产出可执行装置）；
没有对 K3 提到的 f5/f3 改法在 opus 模型上的读数做二次复跑（已含在「一、云端攻方」的重点复跑范围
内，此处不重复）；没有排出 main-agent.md 的两次改动（sonnet 完成 vs. 别的会话编辑）先后的精确时刻，
`stat` 只给出该文件最终的一次 mtime（01:09:04 UTC），不能反推是否在 sonnet 读取之前或之后发生
——这一点留给主 agent，若要判定 sonnet 这处引用是否「应当自报而没自报」需要更细的时间线证据。

## 三、本地腿（转述核对表 + 运行记录）核对表

`defs-gate54-tiering-r2-local-attack-translation-audit.md` 逐条「原文文件:行」核对：

| FACT/项 | 核对表所写 | 结果 | 命令/依据 |
|---|---|---|---|
| FACT 1 区块引用 `54-layer0-replay.sh:376-388` | 整段 heredoc 块 | ✓（区块边界对，376=`if ! {`，388=`sed 's/^/input_file /' …`） | `sed -n '374,392p'` |
| FACT 1 正文「第 376 行那条 # 开头的注释行」「第 377 行 input_hash=」 | 376=注释、377=input_hash | **✗ 差 1**：现查注释行实为第 377 行，`input_hash=` 实为第 378 行 | `grep -n '层 0 全量全绿标记\|input_hash=\$layer0_input_hash\|^if ! {'`；`awk '/^  echo "input_hash=/{print NR}'` |
| FACT 2「`:1013-1014`（`write_layer0_input_manifest` 现算哈希）」 | 1013-1014 | **✗ 误写成背景附录行号**：`_defs-gate54-tiering-r2-appendix.md` 第 1010-1015 行正是这段代码（`sed -n '1010,1020p'` 命中原样），原文件 `54-layer0-replay.sh` 该逻辑实为第 248 行 | 见下方命令 |
| FACT 2「`:1015`（`full_green_marker_path` 按哈希拼文件名）」 | 1015 | **✗ 同上**，原文件实为第 249 行 | 同上 |
| FACT 2「`:1019`…真实行号 253」 | 1019（已自行标注真实行号 253） | ✓（自行更正的部分核对无误） | `sed -n '1019p' 附录`、`sed -n '253p' 54号` 两处均为 `if [[ ! -f "$full_green_marker_path" ]]; then` |
| FACT 2「`:260-261`、`:270-272`、`:279-280`、`:288`、`:290`」 | 5 段代码 | ✓ 全部逐字相同 | `sed -n` 逐行核对（哈希比对/三行计数/两行 exhaustive 计数/finished_utc 读取） |
| FACT 3「`:330`、`:356`」 | 两处独立 if | ✓ 逐字相同 | `sed -n '330p;356p'` |
| FACT 4「`:130-134`、`:137`、`:335`、`:361`」 | 线程判定函数与两处调用 | ✓ 逐字相同 | `sed -n` 逐段核对 |
| FACT 5「`stage-inputs.tsv:1-15`、`54号:56-64`、`:66-70`」 | 查表逻辑与无匹配行判红 | ✓ 逐字相同 | `sed -n` |
| FACT 6「`stage-must-run.sh:34`、`:55-56`、`:65-70`、`:83-87`、`:16-19`」 | 复用助手比较逻辑 | ✓ 逐字相同 | `sed -n` |
| FACT 7「`54号:56-71`、`stage-must-run.sh:70`」 | 两处路径集合不同（综合结论） | ✓ 两处原文均现查存在，综合结论按代码逻辑成立 | `sed -n` |
| FACT 8「`54号:79-86`、`:87-93`」 | 两问顺序 | ✓ 逐字相同 | `sed -n '79,93p'` |
| FACT 9「不出自任何文件的 git 语义」+ 两处来源标注 `54号:159`、`stage-must-run.sh:83,86` | 来源标注 | ✓ 两处原文存在且是 git pathspec 调用点；「不出自任何文件」是负向声明，无法反证，未发现现有注释与之矛盾 | `sed -n '159p' 54号`；`sed -n '83p;86p' stage-must-run.sh` |
| FACT 10「`54号:156-166`、`:164`、`:171`、`:298` 附近」 | 空清单判红检查 | ✓ 逐字相同 | `sed -n`；`grep -n manifest_at_start` 确认 298 行邻域 |

**FACT 2 误标背景材料行号的命令**：

```
$ wc -l research/prompts/_defs-gate54-tiering-r2-appendix.md
1365 research/prompts/_defs-gate54-tiering-r2-appendix.md
$ sed -n '1010,1020p' research/prompts/_defs-gate54-tiering-r2-appendix.md
（与 54-layer0-replay.sh 第 246-254 行的快档开头段落逐字相同：
write_layer0_input_manifest "$quick_manifest" || fail_without_input_manifest
full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash" 等）
$ grep -n 'write_layer0_input_manifest\b' .claude/gate.d/54-layer0-replay.sh
156:write_layer0_input_manifest() {
248:  write_layer0_input_manifest "$quick_manifest" || fail_without_input_manifest
298:write_layer0_input_manifest "$manifest_at_start" || fail_without_input_manifest
366:write_layer0_input_manifest "$manifest_at_finish" || fail_without_input_manifest
$ grep -n 'full_green_marker_path=' .claude/gate.d/54-layer0-replay.sh
249:  full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash"
300:full_green_marker_path="$full_green_marker_prefix.$input_hash_at_start"
```

真实行号是 248（现算哈希）与 249（拼标记文件名），核对表写的「1013-1014」「1015」是
`_defs-gate54-tiering-r2-appendix.md` 自己的行号（该文件同一段代码正好也出现在这个偏移附近）；
核对表结尾那句「本核对表的行号已全部按真实文件重新用 grep -n/awk 现查，与附录数字不一定相同」
对 FACT 2 这两处不成立——只有紧邻的「:1019」一处真的做了这一步（附了「真实行号 253」的更正），
「:1013-1014」「:1015」两处遗留了未更正的附录行号。这正是派发提示要求核的
「误写成背景材料行号的引用」那一类。

**运行记录 `defs-gate54-tiering-r2-local-attack-runlog.md` 复核**：

| 记录所写 | 结果 | 命令 |
|---|---|---|
| s1 词数 1192、s2 词数 1064（`wc -w`） | ✓ 逐字相同 | `wc -w` 两份样本 |
| s1 `oov-check.py`：生词=11 拼接=5（`REUSEDECISION`×4、`LATERBATCH`×1） | ✓ 逐字相同 | `python3 research/scripts/oov-check.py <s1>` |
| s2 `oov-check.py`：生词=14 拼接=7（`REUSEDECISION`×4+1、`LATERBATCH`×2） | ✓ 逐字相同 | 同上，对 s2 |
| s1、s2 `corruption-check.py`：绿（退出码 0） | ✓ | `python3 research/scripts/corruption-check.py <s1/s2>`，均输出「绿」 |
| void1、void2 `corruption-check.py`：红，实词自复读=2（`agree \| agree`） | ✓ 逐字相同 | 同一脚本对 void1/void2，均命中同一诊断 |
| void1/void2「词数（脚本报的词数，原样输出）」1073、1043 | ✓ 与 corruption-check.py 内部报的 `words=` 字段相同（1073、1043） | 但列头写的是「词数（`wc -w`）」，`wc -w` 实测 void1=1115、void2=1107，与两份取值口径不同（脚本内部统计口径 vs `wc -w`），记录里已在单元格加注「脚本报的词数」，不是隐瞒，只列为格式一致性的观察，不计 ✗ |
| s1/s2 样本 13 个标签齐全、无禁用 markdown 强调 | ✓ | `grep -oE` 数标签=13（两份）；`grep -cE '\*\*|```|^#|^\s*\||^\s*\*\s'` 均 0 |
| 目录核对（提示 1、核对表 1、运行记录 1、s1/s2 各 1、void1/void2 各 1） | ✓ | `ls research/prompts/ \| grep defs-gate54-tiering-r2-local-attack` 计数一致 |

提示文件 `defs-gate54-tiering-r2-local-attack.md` 本身按规则不引用任何文件行号（现查通读全文，只用
FACT 编号/ROW 标签，零处文件名+行号），符合「给本地腿的提示一律用英文」条目下「英文提示里的每一句
转述…行号去 kb 文件里现查，不从背景材料里数」这条纪律对提示文件本身的要求；核对表本身则暴露了上面
两处问题。

**本地腿计数**：核了 22 处，✓ 19 处，✗ 3 处（FACT 1 内部行号差 1、FACT 2 两处误标背景附录行号），
1 处观察（词数口径不一致，不计入 ✗）。

**本地腿「没做什么」**：没有重新调用本地模型抽样（不是这一轮核查员的职责，样本与运行记录本身是
观测，核查员只核对表与产物）；没有核 FACT 5-10「有意简化」「未核对…因为不改变本轮任何一格判断」
这几处主观判断是否合理，只核它们标注的行号与代码内容是否存在、是否相符；没有对 `_defs-gate54-tiering-r2-checklist.md`
逐条核对（不在本地腿这份材料的引用范围内，本地腿三份材料零处引用它）。

## 总计

三条腿合计核了 62 处（自证 1 处另计）：✓ 57 处，✗ 3 处（全部在本地腿），分不清 2 处（main-agent.md
漂移，opus 自报 1 处 + 本报告独立发现 sonnet 同处 1 处），核不动 0 处。

## 没做什么（总）

- 不判一条打中成不成立、该不该采纳（K1/K2/K3、V1-V5、g1/g2/g4/g124、V3/V4 各格的实质结论）；
  不核推理本身，只核引用、产物与复跑，按定义开头即声明。
- 未对 `research/scripts/stage-must-run.sh`、`change-touches-crates.sh`、`gate.sh`、`gate-triage.md`、
  `.claude/agent-common.md`、`research/prompts/_defs-gate54-tiering-r2-body.md`、r1 系列五份材料做
  开工快照式的漂移核实（这几份不在给定的快照清单内），只按现在的主树核对；若这些文件中有一份也像
  `main-agent.md`、`checks-owed.md` 那样在三条腿工作期间被改过，本报告不能分辨，风险留给主 agent。
- 未重跑本地腿模型调用（`ask-local.sh`）本身，只重跑了两份确定性脚本（`oov-check.py`、
  `corruption-check.py`）核对运行记录里的数字，这两份脚本对固定输入是确定性的，重跑等价于验证记录
  真实性，不构成对模型答复内容（V3/V4 各格判词）本身的复核。
- 未编译 Rust、未跑门禁全量、未在真仓做任何 git 写操作；未跑门禁阶段（阶段归属表未登记核查员）。
- 未核 sonnet 报告与 opus 报告里大量「可达性折扣」「什么会推翻它」一类论证性文字是否合理——那是
  主 agent 逐条现查推论的职责，不是核查员的职责。
- 未排出 `.claude/main-agent.md` 两次改动（本轮自身 hunk 与另一会话的插入）与 sonnet 完成时刻的精确
  先后（只有该文件最终一次 mtime），因此无法判定 sonnet 的这处引用在它书写时是否已经漂移，只能记
  「分不清」。

报告文件：`research/prompts/defs-gate54-tiering-r2-verifier-output.md`（本文件）。
草稿目录：`/tmp/claude-1000/defs54-r2-verifier/`（`selftest/` 判别力自证副本、`opus-model-copy/`
重跑副本与掩码对比产物 `masked-orig/`、`masked-rerun/`、`masked3-orig/`、`masked3-rerun/`、
`rerun.log`、`rerun-runs/` 合成仓，均可删，不作为入库依据）。

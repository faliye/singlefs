# defs-m2-closeout-r2 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

写于 2026-09-26。核的是三条腿的报告：辩方（Sonnet）、云端攻方（Opus，E1/E2）、本地攻方（E1 的 F14 格，两次抽样 s1/s2）。方法：`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」「引产物就整行抄」。

## 判别力自证（先做，必须判 ✗）

取 Opus 报告引用 `.claude/agents/gate-triage.md:21` 的一条，在草稿目录的副本里把行号加 1（改核 `:22`），核对结果：

```
--- 原始 line 21（Opus 报告声称的内容）---
- `gate.sh` 整条经内存包装的上限（第 1b 步用）。
--- 副本 line 22（自证目标：把行号加 1 之后核）---
- 报告路径与草稿目录。
```

两行内容完全不同，按第 2 步核对判 **✗**。核查方法能分辨——继续往下做。

## 快照与报回哈希核对

- `research/prompts/defs-m2-closeout-r2-snapshot/sha256sums.txt`（24 个文件）：`sha256sum -c` 全部 `OK`（现跑，见下）。这是一份「设计轮」快照，只含 24 个具体文件的哈希，不含 `crates/` 或 `.claude/kb/` 全量；这 24 个文件现在的主树内容与开工快照完全一致，可以直接对主树核。24 个文件之外的引用（`.claude/hooks/runner-dispatch-guard.sh`、`.claude/gate.d/55-qemu-first-transaction.sh`、`research/scripts/*.sh`、`crates/singlefs-harness/src/crash.rs` 等）没有快照，按「设计轮无快照」规则对主树核；下面逐条核对均未出现「文件在腿交回之后被改过」的分不清情形（凡是能核到的都能对上）。
- 三份给定 sha256：`defs-m2-closeout-r2-sonnet-output.md`、`defs-m2-closeout-r2-opus-output.md`、`defs-m2-closeout-r2-opus-model/SHA256SUMS` 现跑全部与派发提示给的值一致，不触发「整份记为分不清」。
- `defs-m2-closeout-r2-opus-model/` 目录内 13 个文件逐个 `sha256sum -c SHA256SUMS`：全部 `OK`。

```
$ sha256sum -c research/prompts/defs-m2-closeout-r2-snapshot/sha256sums.txt
（24 行全部 OK，见下方命令原样）
$ sha256sum research/prompts/defs-m2-closeout-r2-sonnet-output.md
125d51a06653cbea8a7ef816a24906506f40fbf15750032b32c3ea1dcd5b2af4  research/prompts/defs-m2-closeout-r2-sonnet-output.md
$ sha256sum research/prompts/defs-m2-closeout-r2-opus-output.md
f3dcebfb39aed0d68beeb7ba6d10be6870c3c78729360756ba1cb8a9cb620062  research/prompts/defs-m2-closeout-r2-opus-output.md
$ sha256sum research/prompts/defs-m2-closeout-r2-opus-model/SHA256SUMS
b572216edfaa661c90fc5ec411f5724e2a489ee75f76c412fe023b42c3028c46  research/prompts/defs-m2-closeout-r2-opus-model/SHA256SUMS
```

三个都与派发提示里给的值逐字一致。

## 表一：辩方腿（Sonnet）引用核对

方法：逐条 `awk -v n=<行号> 'NR==n' <文件>` 现取，与报告里的抄文逐字比。

| 引用 | 结果 | 命令/说明 |
|---|---|---|
| `defs-m2-closeout-r1-opus-output.md:15`（三档定义「打中/部分/边角」原样） | **✗ 行号错，实为第 25 行** | `:15` 实际内容是 `f1f646615e...cases.json`（SHA256SUMS 的一行）；`grep -n '「打中」='` 命中第 25 行，文字与报告抄的完全一致，只是行号写错（差 10 行）。不是背景材料行号误写（`_defs-m2-closeout-r2-background.md:15`、`_defs-m2-closeout-r1-background.md:15`、`_defs-m2-closeout-r1-appendix.md:15` 均核过，都不是这句） |
| `defs-m2-closeout-r1-main-verification.md:9,19,24,25,27,28,57,58`（8 处） | ✓ 全部 8 处逐字一致 | `awk -v n=<N> 'NR==n' research/prompts/defs-m2-closeout-r1-main-verification.md` 逐个核 |
| `defs-m2-closeout-r1-opus-output.md:115,236,238,253,278,295,154`（7 处） | ✓ 全部 7 处逐字一致 | 同上，逐行 `awk` 核 |
| `defs-m2-closeout-r1-verifier-output.md:82,83,84,92,101,105`（6 处） | ✓ 全部 6 处逐字一致 | 同上 |
| `defs-m2-closeout-r1-verifier-output.md` 表三数据行（`:124`-`:131`，共 8 行）与总计表（`:139`-`:142`） | ✓ 结构与文字一致；**且辩方对这张表的重新加总本身是对的** | 独立数：`:124` 开头 `**✗` 一行、`:125`-`:130` 开头 `✓` 共 6 行、`:131` 开头「核不动」一行 = 8 行、6 个 ✓，不是 `:133`/`:141` 写的 7 个；`27+49+6=82`，不是 `:142` 写的 83 |
| `.claude/agents/three-way-attack.md:27` | ✓ | `awk 'NR==27' .claude/agents/three-way-attack.md` |
| `.claude/agents/gate-triage.md:27`（F9 后现状） | ✓ | 现查主树，含「单跑的门禁阶段照第 2 步直接跑，外面不包一层」原句 |
| `records/2026-09-16-subagent拆分提案.md:951`（item 40，640 字节） | ✓ 逐字一致，`wc -c` = 640 | `sed -n '951p' ... \| wc -c` → 640；末句「未改：要改得先定『静态分支』……」逐字对上 |
| `.claude/gate.d/84-verdict-false-named.sh`（`value == 'false'` 判定逻辑，报告写「84 行左右」，非精确行号引用） | 逻辑核实无误；**行号是松散说法，不是精确引用**：真实行号是第 99 行，不是 84 | `grep -n "value == 'false'"` → `99:                if value == 'false':`；报告用「84 行左右」这种带「左右」的hedge，不构成「文件:行号」的精确引用格式，不计入 ✗ 统计，单独记此备注 |
| O5 grep 复跑：`grep -rhoE 'name=verdict.*' research/results/ \| tr ' ' '\n' \| grep -E '_(violations\|mismatches\|failed)=\|ambiguous=' \| grep -v '=0\b' \| grep -v '=false\|=true'` | ✓ 复跑结果与报告一致：`layer0_violations=not_run` 17 处、`layer0_violations=zero` 1 处，无非零数字 | 现跑同一条命令，`uniq -c` 输出 `17 layer0_violations=not_run` `1 layer0_violations=zero` |
| `grep -rln check.sh .claude/gate.d/*.sh` | ✓ 0 命中 | 现跑，exit=1（grep 未命中），计数 0 |
| `.claude/agent-common.md:58,60`（F11 后现状） | ✓ 两行内容与报告引述一致 | 现查主树 |
| `_defs-m2-closeout-r1-diff.md` 第二个 hunk `@@ -54,7 +55,19 @@`（新文件行号 58、60 的映射） | ✓ 手工按 diff 加减行逐行推算：新文件第 58 行落在 `+` 行「等的时候看到进程不动……run_in_background 里的等待循环只记检出，交给主 agent 判断」，第 60 行落在「Bash 检出 hook……④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒」，与报告引述逐字一致 | `grep -n "^@@" _defs-m2-closeout-r1-diff.md` 定位 hunk 起点，逐行数 context/+/- 三类推算新文件行号 |
| `.claude/gate.d/55-qemu-first-transaction.sh:39,207`、`research/scripts/vm-bench.sh:14`（O8 节再次现查） | ✓ 3 处均一致 | `awk` 现取 |
| `.claude/rules/implementation-workflow.md`（含「`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7」，D4/O18 节引用） | ✓ | 现查主树第 59 行，`fetch-deps.sh:81` 现查确为 `herd7 -version` 调用行 |

**计数**（命令数出）：核了 37 处，✓ 36 处，✗ 1 处，另有 1 处非精确引用单列（不计入 ✓/✗）。

## 表二：云端攻方腿（Opus）引用核对与复跑

### 附录 A 逐字引文（62 处），独立重跑比对

Opus 报告附录 A 给了一条可独立重跑的命令，从它自己的 `quotes.list`（62 行 `文件 行号`）现取每一行原文。把 `quotes.list` 拷进草稿目录，独立重跑同一条命令（不用报告里现成的输出）：

```
$ cp research/prompts/defs-m2-closeout-r2-opus-model/quotes.list /tmp/claude-1000/defs-m2-closeout-r2-verifier/quotes.list
$ nice -n 19 bash -c 'while read -r f n; do printf "%s:%s\t" "$f" "$n"; awk -v n="$n" "NR==n" "$f"; done < .../quotes.list' > quotes-rerun.txt
$ wc -l quotes-rerun.txt
62
$ sed -n '385,446p' research/prompts/defs-m2-closeout-r2-opus-output.md > quotes-report.txt
$ diff quotes-report.txt quotes-rerun.txt
（无输出，diff exit=0）
```

62 处逐字引文与我独立重跑取到的现状**逐字节相同**（`diff` 为空）。**62/62 ✓**，0 处不符。

### `rerun.sh` 整份复跑

命令（按派发提示原样，脚本用自身相对路径推算仓根，因此在其原位置调用、把临时目录参数指向本核查员的草稿目录）：

```
$ nice -n 19 bash research/prompts/defs-m2-closeout-r2-opus-model/rerun.sh /tmp/claude-1000/defs-m2-closeout-r2-verifier/opus-rerun
退出码：0（与报告「114 行，退出码 0」的退出码一致）
```

我的原样输出 116 行（报告留存的 `rerun-output.txt` 114 行）。`diff` 出的差异全部落在两类，均不构成 ✗：

1. **内嵌临时路径**（3 处 `probe-detections-*.jsonl` 的目录前缀不同，是这次调用自己的草稿目录）：按字段比，检出行数 `0/1/0` 三处计数与报告完全一致。
2. **两处内存/OOM 时序相关的非确定性行**：`nest-demo.sh` 的峰值 MiB 读数（`13/10/11` vs 我这次的 `12/11/12`）与 `oom-demo.sh` 的 `wrapped` 分支里 `step2`/`step3` 是否来得及在 systemd 停止整个 scope 之前打印（报告的原样只打出 `step1 ran` 就撞停，我这次多打出了 `step2 exit=137` 与 `step3 ran`）——这两处报告正文都**没有引用这几个具体数字或这两行**，只引用了 `wrapper_exit=250`（两次跑都是 250，一致）。这正是 H1 论证「撞顶后 systemd 停整个 scope」机制本身固有的时序竞争，不影响报告结论。

承重的退出码与计数逐一核对：

| 探针/演示 | 报告字段 | 我的复跑 | 结果 |
|---|---|---|---|
| `cases-f13.json`（Q1–Q9，head/now 各一次，共 18 行） | 18 个 exit 值 | 18 个 exit 值 | ✓ 逐条相同 |
| `cases-hooks.json`（P1–P3、T1–T3、G1–G3、B1–B4，共 13 行） | 13 个 exit 值 | 13 个 exit 值 | ✓ 逐条相同 |
| `cases-dispatch.json`（D1–D6，共 6 行） | 6 个 exit 值 | 6 个 exit 值 | ✓ 逐条相同 |
| `f13-fix-g1.sh`（自检 + Q1–Q9-fixG1，共 10 行） | 10 个 exit 值 | 10 个 exit 值 | ✓ 逐条相同 |
| `verdict-scan.py` 全量扫描（18 处 UNNAMED + 4 处 NAMED-COUNT + 合计行） | 22 行 + 1 行合计 | 22 行 + 1 行合计 | ✓ 逐字节相同 |
| `stage-demo.sh`（40/86 号三时点） | 8 行 | 8 行 | ✓ 逐字节相同 |

**计数**：这一部分核了 6 组（覆盖 62 处引文 + 63 个探针/演示字段），✓ 全部，✗ 0。

### 系统/仓状态类断言核对（非引文，独立现查）

| 断言 | 结果 | 命令 |
|---|---|---|
| 本机 `nproc` = 32 | ✓ | `nproc` → 32 |
| slice 总上限 40.1 GiB | ✓ | `nice -n 19 bash research/scripts/run-with-memory-cap.sh --status` → `slice singlefs-heavy.slice 的总上限 40.1 GiB` |
| `memory-peaks.tsv` 里 `checker_known_bad_images`\|`74-model-differential` 键共 46 行，其中「`-- ` 点名一两个用例」36 行、「只编不跑」1 行、`2026-09-26` 不带 `capped.sh` 撞 4G 顶 1 行、H1 节列出的 8 行 | ✓ 46/36/8 三个数字均现跑复核一致（8 行按内容逐条比对与报告列出的 8 行完全相同） | `grep -v '^#' research/scripts/memory-peaks.tsv \| awk -F'\t' '$4 ~ /checker_known_bad_images\|74-model-differential/' \| wc -l` → 46；`grep -c -- '-- '` → 36 |
| `defs-m2-closeout-r2-opus-model/` 目录 13 个文件与自带 `SHA256SUMS` 一致 | ✓ 13/13 `OK` | `sha256sum -c SHA256SUMS`（在原目录跑，只读校验，不改动） |
| 包装演示用的临时 slice 已停用、无残留 | ✓ 0 个残留 | `systemctl --user list-units --all \| grep -c singlefs_` → 0 |
| `crates/singlefs-harness/src/crash.rs` 含 `SparseDevice`、`SparseBlockDevice`、`MemoryPool`、`closed_form_state_count`、`enumerate_layer0_versions`、`enumerate_layer0`（「没打中的形状」F3 一条） | ✓ 6/6 全部存在 | `grep -n 'fn enumerate_layer0\b\|fn enumerate_layer0_versions\|fn closed_form_state_count\|struct SparseBlockDevice\|struct SparseDevice\|struct MemoryPool' crates/singlefs-harness/src/crash.rs` |
| `research/results/` 5 处判决行引文（H5 节 `verdict-scan.py` 输出对应的源产物） | ✓ 5/5 逐字一致 | `awk -v n=<N> 'NR==n' <文件>` 分别核 `e142-first-txn-dry-run-2026-09-25-r17-layer0.out:608`、`e142-r18-controls-2026-09-26.out:226`、`e146-livelist-entry-width-2026-09-16-tree-table-200.out:129`、`e145-self-describing-node-header-2026-09-16-tree-table-200.out:56`、`e147-system-configuration-recompute-from-layout-2026-09-13.out:13` |
| `runner-dispatch-guard.sh:2,3,16,44`、`heavy-test-guard.sh:131`、`check.sh:47`、`gate.sh:457`（「没打中的形状」引用） | ✓ 全部一致（后两处也在附录 A 的 62 处之内，未重复计数） | `awk` 现取 |

**计数**：这一部分核了 7 组（`nproc`、`--status`、峰值表三个数字、SHA256SUMS 13 个文件、systemd 残留、6 个函数/结构体、5 处 `research/results/` 引文、4+1 处 hook/脚本行号），全部 ✓，✗ 0。

**Opus 腿总计**：62（附录 A）+ 6 组探针/演示复跑 + 7 组系统与产物断言 = **76 处核对，✓ 76，✗ 0**（用命令数出，见上各表逐行）。

## 专项核对：辩方报的「第一轮判决第 9 行 ✓83 应为 82」

派发提示单独点名要核这一条。逐行数 `defs-m2-closeout-r1-verifier-output.md` 的「表三：本地攻方腿」：

```
$ awk 'NR==118,NR==133' research/prompts/defs-m2-closeout-r1-verifier-output.md
## 表三：本地攻方腿（3 次抽样：s1 干净、s2 带损坏、s3 干净）
...
| 引用/结论 | 结果 | 命令 |
|---|---|---|
| ... :124 ...  | **✗ 与 `git diff` 矛盾**：...              ← 第 1 行，✗
| ... :125 ...  | ✓ 全部 10 行逐字match...                    ← 第 2 行，✓
| ... :126 ...  | ✓                                            ← 第 3 行，✓
| ... :127 ...  | ✓                                            ← 第 4 行，✓
| ... :128 ...  | ✓                                            ← 第 5 行，✓
| ... :129 ...  | ✓ 完全复现...                                ← 第 6 行，✓
| ... :130 ...  | ✓（`ls ...`）                                ← 第 7 行，✓
| ... :131 ...  | 核不动：草稿目录属临时目录...                 ← 第 8 行，核不动
**计数**：核了 8 处，✓ 7 处，✗ 1 处，核不动 1 处。   ← 原文自称（:133）
```

数据行合计 8 行：✗ 1（:124）、✓ 6（:125–:130）、核不动 1（:131）。原文 `:133` 自称「✓ 7 处」与数据行不符（7+1+1=9 ≠ 8；6+1+1=8 才配得上「核了 8 处」）。这处矛盾传进 `:141` 总计表「本地攻方 8/7/1/1」一行，再传进 `:142` 合计「96/**83**/12/2」。用真实数据行重新加总：正推 27 + 云端攻方 49 + 本地攻方 **6** = **82**，不是 83。

**结论：辩方这一条核实为真**（独立重数结果与辩方报告 `:89` 的数字完全一致：6 个 ✓，应为 82）。这处偏差是核查员自己产出的总计表内部算术错误，与任何一个 O 项判定所依赖的具体引文核实结果无关（见 r1 表三前 6 行 ✓ 的引文本身都核对无误，只是数错了 1）。

## 表三：本地攻方腿（转述核对表 + 运行记录复跑）

### 转述核对表：每处「原文文件:行」逐条核

方法：`.claude/rules/implementation-workflow.md` 与 `.claude/hooks/lib_heavy_tests.py` 都在 24 文件快照里、现状与快照一致，直接对主树核。

| 表 | 引用（原文文件:行） | 结果 |
|---|---|---|
| 表一 | `implementation-workflow.md:48,50,51,52,53,54`（6 处） | ✓ 6/6 逐字一致 |
| 表二 | `implementation-workflow.md:55,56,57,59`（C9/C10+C11/C12 三行同引 `:59` 的三个分句，共 6 处） | ✓ 6/6 逐字一致，含 C9「六种包装」、C10+C11「逐行照同一个 classify 判……按名字判、不读正文」两个并列分句、C12「54、55、57、59、87 之外的阶段不是重型」 |
| 表三 | `lib_heavy_tests.py:285,159,256,55`（4 处，文档字符串与注释） | ✓ 4/4 逐字一致 |
| 表四 | `lib_heavy_tests.py:289,292,296,305,308,312`（6 处，中文消息串） | ✓ 6/6 逐字一致 |
| 表五 | `lib_heavy_tests.py:206,217,227-228,233,238`（5 处，中文消息串） | ✓ 5/5 逐字一致（`:227`-`:228` 跨行 f-string 整句核对） |
| 「首稿改正」备注 | `lib_heavy_tests.py:210`（`nearest_manifest` 调用行） | ✓ |
| 「首稿改正」备注 | `.claude/scripts/fetch-deps.sh:81`（`herd7 -version` 调用行，原文行号「第 81 行」） | ✓ `grep -n "herd7 -version" .claude/scripts/fetch-deps.sh` → 精确命中第 81 行 |

英文转述里「多出来的限定词」（C1.5、C5.5，两条逻辑补集）与「首稿缺的/改正的」两处，核对表自己写明了理由和依据行号，现查依据行号均属实（`:50`、`:54` 现查内容与核对表描述的判法一致）。未发现漏写限定词或多加原文没有的括注。

**计数**：核了 29 处「原文文件:行」，✓ 29 处，✗ 0 处。

### 运行记录复跑

| 断言 | 结果 | 命令 |
|---|---|---|
| s1 词数 607、s2 词数 571 | ✓ | `wc -w` 两份样本，结果 607/571 |
| s1 `oov-check.py`：绿，生词=1（`CONTRADICTED`） | **✗ 应为生词=2**：`CONTRADICTED` 在 s1 里出现 2 次（Q10 与 Q13 各一次），`len(oov)` 按出现次数计数、不去重，工具原样输出是「生词=2」，只有打印的去重词表只显示 1 个不同的词 | `python3 research/scripts/oov-check.py .../output-s1.md` → `生词=2 拼接=0`；`grep -o CONTRADICTED .../output-s1.md \| wc -l` → 2 |
| s2 `oov-check.py`：绿，生词=1（`CONTRADICTED`） | ✓ | 现跑同一命令 → `生词=1 拼接=0`；`CONTRADICTED` 在 s2 只出现 1 次（`grep -c` → 1） |
| s1/s2 `corruption-check.py`：绿，全部计数为 0 | ✓ 2/2 | 现跑，两份输出各字段均为 0 |
| 目录清点：提示 1、核对表 1、运行记录 1、`-output-s1/s2.md` 2 份，无 `-output-void*.md` | ✓ | `ls research/prompts/ \| grep defs-m2-closeout-r2-local-attack` → 5 个文件，与声称一致 |
| 网关文件 `~/code/ai-center/.env.tenants` 存在 | ✓（现在仍存在） | `ls -la ~/code/ai-center/.env.tenants` |
| 「跑之前 `ps` 查过重型进程两次都零命中」「网关 `curl` 探测有响应」 | 核不动：这是两次调用发生那一刻的进程/网络状态，事后无法重放同一时刻 | 现查当下：`:8200/v1/chat/completions` 现在仍有响应（405），与「有响应」这一描述不矛盾，但不能证明「那两次调用时」确实如此 |

**计数**：核了 8 处，✓ 6 处，✗ 1 处（s1 的 `oov-check` 生词计数），核不动 1 处。

**本地攻方腿总计**：29（转述表）+ 8（运行记录）= **37 处，✓ 35，✗ 1，核不动 1**。

## 总计

| 腿 | 核了 | ✓ | ✗ | 核不动/分不清 |
|---|---|---|---|---|
| 辩方（Sonnet） | 37 | 36 | 1 | 0 |
| 云端攻方（Opus） | 76 | 76 | 0 | 0 |
| 本地攻方（转述表+运行记录） | 37 | 35 | 1 | 1 |
| **合计** | **150** | **147** | **2** | **1** |

两处 ✗：
1. 辩方报告把 `defs-m2-closeout-r1-opus-output.md` 里三档定义原文的行号写成 `:15`，实际在 `:25`（差 10 行；文字本身抄对了）。
2. 本地攻方运行记录把 s1 样本 `oov-check.py` 的「生词」计数写成 1，工具原样输出是 2（`CONTRADICTED` 在 s1 出现 2 次，脚本按出现次数计数、不去重；s2 的同一条断言核对无误）。

一处专项复核（派发提示单独点名）：辩方指出「第一轮判决第 9 行 ✓83 应为 82」——**核实为真**，见「专项核对」一节；根因是第一轮核查员报告自己的表三总结句（:133）把 6 个 ✓ 数成了 7 个，一路传进总计表与判决第 9 行。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（O3/O8/O9/O11/O12 的严重度判断、H1–H9 各格的四句论证是否站得住），只核引用、产物与复跑。**表里的 ✗ 不免除主 agent 对推论的逐条现查**——尤其是辩方报告 O9 节里「情形已经不只是读法分歧」这类判断性文字，本报告未核。
- 没有重新判定 Opus 报告「四句」（分辨臂/系统看不看得到/字面分句/改法在哪一格）本身是否成立，也没有评估 F9/F7/F4/F13/F5/F1/F16/F10 各条改法建议是否可行——那是主 agent 的事。
- 本地攻方 Q1–Q16 的具体判词内容（哪个 C 码对应哪个 K 码、MATCH/CONTRADICTED 的方向对不对）未核——按分工，转述核对表管的是「英文转述与中文原文是否一致」，不是「探针结果本身对不对」；后者需要重新对 `lib_heavy_tests.py` 的 `classify`/`cargo_use` 跑一遍探针，不在派发范围内（这份提示与报告没有另附探针日志 K1–K29 的原始脚本，只有转述核对表与运行记录）。
- `.claude/gate.d/84-verdict-false-named.sh` 的「84 行左右」不构成精确的「文件:行号」引用，核实了代码逻辑但没有把它计入 ✗/✓ 统计，单独记备注。
- Opus 报告里标「推的，没实现没跑」的全部改法（甲/乙/丙、F7 改、F4 改、F5 改、F1 改、F16 改、辩方前缀写死等）未验证是否可行、是否真能修复对应的格——这些本来就没有可复跑的产物。
- 未对 r1 判决第一节「正推 11 个 ✗（10 个行号错位，1 个……）」这一句做加总复核（辩方报告自己也未做这一步，只核了「96/83/12/2」这一行）。
- 未跑重型测试、未编译 Rust、未做任何 git 写操作；`.claude/gate.d/` 下没有登记给核查员的阶段（`stage-owners.tsv` 未列 three-way-verifier 名字对应任何阶段）。
- 草稿目录 `/tmp/claude-1000/defs-m2-closeout-r2-verifier/`：`opus-rerun/` 下 `probe.py` 自建的探针临时文件（`probe-ask-head-*`、`probe-detections-*.jsonl`，几 KB）与本报告自己的比对文件（`quotes.list`、`quotes-rerun.txt`、`quotes-report.txt`、`rerun-output-mine.txt`、`mp-all46.txt`、`selftest/gate-triage.md`）留在草稿目录，均为只读比对产生的文本，无编译目录、无仓副本（`rerun.sh`/`stage-demo.sh` 自己 `rm -rf` 掉了它们建的仓副本，已现查 `systemctl --user list-units` 确认 0 残留 slice）；主 agent 用不上可以整个删除 `/tmp/claude-1000/defs-m2-closeout-r2-verifier/`。

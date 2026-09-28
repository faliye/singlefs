# defs-m2-closeout-r1 核查员报告

本报告是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

轮名：`defs-m2-closeout-r1`。三条腿（正推 Sonnet、云端攻方 Opus、本地攻方 3 次抽样）均已交齐。

## 输入核对

- 三份报告 sha256：与派发提示给的一致（现查 `sha256sum`）：
  - `defs-m2-closeout-r1-sonnet-output.md` = `5349305696700983d113a31daf1b78e47a175ec486bd9db288f06b300bb86a9f` ✓
  - `defs-m2-closeout-r1-opus-output.md` = `938c66332edc58fe9c1630b72e9dadbac3032d9cbf331b458304fd65f6bd53ea` ✓
- 快照 `research/prompts/defs-m2-closeout-r1-snapshot/sha256sums.txt`（22 个文件：被判的 16 份 agent 定义、`agent-common.md`、`main-agent.md`、`stage-owners.tsv`、3 份 hook）：`sha256sum -c` 全部 `OK`，与当前工作区（快照覆盖的这 22 个文件）逐字节一致——本报告对这 22 个文件按当前工作区核，等同于对快照核。
- `crates/` 与 `records/2026-09-16-subagent拆分提案.md`、`.claude/kb/checks-owed.md`、`research/scripts/replay.sh` 不在快照清单里，且 `git status --short` 现查均为工作区未提交改动（`crates/singlefs-harness/src/crash.rs`、`.claude/kb/checks-owed.md`、`records/2026-09-16-subagent拆分提案.md`、`research/scripts/replay.sh` 均标 `M`，`.claude/gate.d/84-verdict-false-named.sh` 标 `??`）：按派发提示「没给快照的轮对主树核，行号对不上时记『分不清』」处理，crash.rs 按派发提示明写的口径单列。

## 判别力自证

方法：取 Sonnet 报告 D1-a 引用 `.claude/agents/three-way-local-attack.md:34` 的原文核对，在草稿副本里把行号从 34 改成 35（该文件第 35 行为空行），按同一比对方法核：

```
$ sed -n '34p' .claude/agents/three-way-local-attack.md
- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）、「产出」一节的运行记录（`research/prompts/<轮>-local-attack-runlog.md`）、草稿目录。除此之外不写。
$ sed -n '35p' .claude/agents/three-way-local-attack.md
（空行）
```

改后的行号（35）取到的是空行，与 Sonnet 报告引用的原文（「提示文件、核对表、样本文件……」）不匹配 → **判 ✗**。核查方法能分辨行号错位，自证通过，往下开工。

## 表一：正推腿（Sonnet）

方法：逐条对 `defs-m2-closeout-r1-sonnet-output.md` 里「文件:行号 + 抄的原文」，用 `sed -n '<N>p'` / `awk 'NR==<N>'` 现取该文件当前内容（22 个快照文件核当前工作区，等同核快照；`crates/singlefs-harness/src/crash.rs` 按派发提示核主工作区，行号对不上记「分不清」）逐字比对。

| 引用 | 结果 | 命令 |
|---|---|---|
| `records/…:912`（D1-a，第 1 行原文） | ✓ 逐字match | `awk 'NR==912' "records/2026-09-16-subagent拆分提案.md"` |
| `.claude/agents/three-way-local-attack.md:34` | ✓ | `sed -n '34p' .claude/agents/three-way-local-attack.md` |
| `.claude/agents/three-way-local-defense.md:11` | ✓ | `sed -n '11p' .claude/agents/three-way-local-defense.md` |
| `.claude/agents/three-way-local-defense.md:19` | ✓ | `sed -n '19p' .claude/agents/three-way-local-defense.md` |
| `records/…:940`（D1-b，第 29 行现状列） | ✓ 逐字match | `awk 'NR==940' "records/…"` |
| `.claude/agents/three-way-attack.md:29-33`（3b 整块） | ✓ 5 行逐字match | `sed -n '29,33p' .claude/agents/three-way-attack.md` |
| `crates/…/crash.rs` grep `SparseBlockDevice` 113/119/133 | ✓ | `grep -n SparseBlockDevice crates/singlefs-harness/src/crash.rs` |
| `crates/…/crash.rs:913-936`（`checker_evaluated_states` 与 `checker_counts_by_invariant`） | ✓（对当前主工作区核，与派发口径一致；该文件正被并行改动，仅证明此刻仍对得上，非快照） | `sed -n '913,936p' crates/singlefs-harness/src/crash.rs` |
| `.claude/agent-common.md:53`（崩溃点测试不衡量时间成本） | ✓ | `sed -n '53p' .claude/agent-common.md` |
| `records/…:944`（D1-c，第 33 行） | ✓ 逐字match | `awk 'NR==944' "records/…"` |
| `.claude/agents/experiment-runner.md:33`（4c 步） | ✓ | `sed -n '33p' .claude/agents/experiment-runner.md` |
| `.claude/agents/experiment-runner.md:44`（「产出」一节引文，报告写作第44行） | **✗ 行号错位**：44 行是空行，实际内容在第 45 行 | `awk 'NR==44' .claude/agents/experiment-runner.md`（空）→`awk 'NR==45'`命中 |
| `records/…:941`（D2-a，第 30 行④段「原文（节选，逐字见于背景材料附录）」） | **✗ 引文被改写**：报告写「外面不再包**一层**」，记录第 941 行原文（及背景材料 `_background.md:498`、附录 `_appendix.md:206`）均为「外面不再包」，没有「一层」二字；「一层」出自当前 `agent-common.md:48`（这份定义的现状），不是记录原文——而背景材料 D2 分工表（`_background.md:18`）明写这一点正是要核的疑点「『外面不再包』的嵌套理由（推的，没量过）」，报告的引文把这处待核差异悄悄抹平了 | `grep -no "外面不再包一层" "records/…"`（0 命中）；`grep -no "外面不再包" "records/…"`（941 命中，无「一层」） |
| `.claude/agent-common.md:48` | ✓ | `sed -n '48p' .claude/agent-common.md` |
| `research/scripts/replay.sh:24` | ✓ | `sed -n '24p' research/scripts/replay.sh` |
| `.claude/agents/crash-verifier.md`「第 15-18 行」（D2-a，「输入」三项） | **✗ 范围错位**：三项实际在 18、19、20 行（16 行是「## 输入」标题，15/17 空行），「15-18」只覆盖了第一项，另两项（「提交还是用户要求」「报告路径与草稿目录」）落在引用范围之外 | `sed -n '15,20p' .claude/agents/crash-verifier.md` |
| `.claude/kb/checks-owed.md:394`（C455） | ✓ 逐字match | `sed -n '394p' .claude/kb/checks-owed.md` |
| `records/…:943`（D2-b，第 32 行 ⑥⑦ 两句） | ✓ 逐字match | `awk 'NR==943' "records/…"` |
| `.claude/agent-common.md:50-57`（D2-b，「执行前拒绝的写法」总纲＋①-⑦） | **✗ 行号错位**：该节实际起于第 59 行（`- 执行前拒绝的写法：…`），①-⑦ 在 61-67 行，50-57 行是另一段内容（负载检查、读大文件、线程上限、崩溃点测试、任务范围等，与「执行前拒绝的写法」无关） | `grep -n "执行前拒绝的写法" .claude/agent-common.md` → 59 行 |
| `.claude/agents/crash-verifier.md:22`（D2-c，1b 引文） | **✗ 行号错位**：22 行是「## 做什么」标题，1b 引文实际在第 25 行 | `sed -n '22p;25p' .claude/agents/crash-verifier.md` |
| `.claude/agents/experiment-runner.md:26`（D2-c，1b 引文） | **✗ 行号错位**：26 行是第 1 步（看负载），1b 引文实际在第 27 行 | `sed -n '26p;27p' .claude/agents/experiment-runner.md` |
| `.claude/agents/gate-triage.md:26` | ✓ | `sed -n '26p' .claude/agents/gate-triage.md` |
| `.claude/agents/implementation-writer.md:26` | ✓ | `sed -n '26p' .claude/agents/implementation-writer.md` |
| `.claude/agents/three-way-attack.md:34-40`（3c 整块） | ✓ 7 行逐字match | `sed -n '34,40p' .claude/agents/three-way-attack.md` |
| `.claude/gate.d/stage-owners.tsv:48` | ✓ | `sed -n '48p' .claude/gate.d/stage-owners.tsv` |
| `.claude/agents/experiment-runner.md:35`（第 6 步） | ✓ | `sed -n '35p' .claude/agents/experiment-runner.md` |
| `.claude/agents/experiment-runner.md:36`（第 7 步） | ✓ | `sed -n '36p' .claude/agents/experiment-runner.md` |
| `.claude/gate.d/84-verdict-false-named.sh` 第 24-25 行（「已知会红的一格」引文） | **✗ 行号错位**：该段实际在第 28-29 行；24-25 行是「出路」段（内容不同） | `sed -n '24,29p' .claude/gate.d/84-verdict-false-named.sh` |
| 同上第 138-142 行（「`experiment_page` 返回 None → unnamed」判据） | **✗ 范围偏窄**：138-142 行只是准备代码（`page_cache = {}` 起），`if page is None:` 判据实际在 143 行、`unnamed.append` 在 144-145 行 | `sed -n '136,146p' .claude/gate.d/84-verdict-false-named.sh` |
| 同上第 163-165 行（「最终判红退出 1」） | **✗ 范围偏窄**：163-165 行是统计打印与 `if unnamed:`，`sys.exit(1)` 实际在第 171 行 | `sed -n '163,171p' .claude/gate.d/84-verdict-false-named.sh` |
| `.claude/main-agent.md:22`（第 5 条出处） | ✓ | `sed -n '22p' .claude/main-agent.md` |
| `.claude/hooks/ask-user-claim-guard.sh:1-27` | ✓（内容与逻辑描述一致；引用钩子「为什么」段落时用省略号摘了原句中段与末段，未计入 ✗，因为报告本身用「……」标出了省略，不冒充逐字） | `sed -n '1,27p' .claude/hooks/ask-user-claim-guard.sh` |
| `.claude/main-agent.md:14` | ✓ | `sed -n '14p' .claude/main-agent.md` |
| `.claude/main-agent.md:20` | ✓ | `sed -n '20p' .claude/main-agent.md` |
| `.claude/rules/implementation-workflow.md:46-47` | ✓（46 是标题、47 是内容，范围覆盖两者，8 类计数核对准确） | `sed -n '46,47p' .claude/rules/implementation-workflow.md` |
| `.claude/main-agent.md:29`（D3-c 引文） | **✗ 行号错位**：29 行是空行，引文实际在第 30 行 | `sed -n '29p;30p' .claude/main-agent.md` |
| `.claude/main-agent.md:52`（D3-d 引文） | **✗ 行号错位**：52 行是表格标题行（`| 什么时候 | 派谁、按什么次序 |`），引文实际在第 59 行 | `sed -n '52p;59p' .claude/main-agent.md` |
| `.claude/agents/implementation-writer.md:28`（D3-f 引文） | **✗ 行号错位**：28 行是第 3 步（证红方法），引文（「其余行照样追加进…59 号」）实际在第 29 行 | `sed -n '28p;29p' .claude/agents/implementation-writer.md` |

**计数**：核了 38 处，✓ 27 处，✗ 11 处。

## 表二：云端攻方腿（Opus）

Opus 报告末尾自带「引文原行」附录（`awk 'NR==行号'` 现取），逐条脚本化比对：27 处全部逐字节一致，0 处不符（脚本核对，见下方命令）。另外单独抽查报告正文引用但不在「引文原行」附录里的行号与产物数据，逐条现查。

| 引用 | 结果 | 命令 |
|---|---|---|
| 「引文原行」附录 27 处（`agent-common.md` 7 处、`three-way-attack.md` 4 处、`experiment-runner.md` 2 处、`implementation-writer.md`/`mutation-triage.md`/`three-way-local-defense.md`/`three-way-local-attack.md`/`gate-triage.md`/`crash-verifier.md`/`main-agent.md`/`implementation-workflow.md` 各若干） | ✓ 27/27 逐字节一致，0 处不符 | 见下方 Python 脚本，逐行 `filelines[lineno-1]` 与附录文本比对 |
| `crates/…/crash.rs:1`（层 0 崩溃点重放文档注释） | ✓（对当前主工作区核） | `awk 'NR==1' crates/singlefs-harness/src/crash.rs` |
| `crates/…/crash.rs:524`（`closed_form_state_count` 定义） | ✓ | `grep -n closed_form_state_count crates/singlefs-harness/src/crash.rs` |
| `crates/…/crash.rs:1321`（`check_pool_image` 调用） | ✓ | `awk 'NR==1321' crates/singlefs-harness/src/crash.rs` |
| `crates/…/crash.rs:37`（`SparseDevice` struct + `derive(Clone)`） | ✓ | `sed -n '35,37p' crates/singlefs-harness/src/crash.rs` |
| `crates/…/crash.rs:172`（`MemoryPool` struct + `derive(Clone)`） | ✓ | `sed -n '170,172p' crates/singlefs-harness/src/crash.rs` |
| `crates/…/crash.rs:113`（`SparseBlockDevice` 无 `derive(Clone)`，靠 `pub image: SparseDevice` 字段拷贝） | ✓（现查 100-113 行确认无 derive 属性） | `sed -n '100,113p' crates/singlefs-harness/src/crash.rs` |
| `.claude/gate.d/stage-owners.tsv:62,78` | ✓ | `sed -n '62p;78p' .claude/gate.d/stage-owners.tsv` |
| `.claude/singlefs-ai-sop/scripts/check.sh:47` | ✓ | `sed -n '47p' .claude/singlefs-ai-sop/scripts/check.sh` |
| `.claude/gate.d/15-research-build.sh:25` | ✓ | `sed -n '25p' .claude/gate.d/15-research-build.sh` |
| `.claude/gate.d/74-model-differential.sh:43` | ✓ | `sed -n '43p' .claude/gate.d/74-model-differential.sh` |
| `research/scripts/run-with-memory-cap.sh:9,24,62,72` | ✓ 全部 4 处 | `sed -n '9p;24p;62p;72p' research/scripts/run-with-memory-cap.sh` |
| `.claude/gate.d/57-lkmm.sh:49` | ✓ | `sed -n '49p' .claude/gate.d/57-lkmm.sh` |
| `.claude/scripts/fetch-deps.sh:81` | ✓ | `sed -n '81p' .claude/scripts/fetch-deps.sh` |
| `.claude/agents/crash-verifier.md:18-20`（「输入」三项，与 Sonnet 的「15-18」形成对照） | ✓ 三项精确落在 18/19/20 行 | `sed -n '18,20p' .claude/agents/crash-verifier.md` |
| `.claude/main-agent.md:46`（线程上限一行） | ✓ | `sed -n '46p' .claude/main-agent.md` |
| `.claude/gate.d/55-qemu-first-transaction.sh:39,207` | ✓ | `sed -n '39p;207p' .claude/gate.d/55-qemu-first-transaction.sh` |
| `research/scripts/vm-bench.sh:14` | ✓ | `sed -n '14p' research/scripts/vm-bench.sh` |
| `.claude/hooks/bash-command-detector.sh:199,201` | ✓ | `sed -n '199p;201p' .claude/hooks/bash-command-detector.sh` |
| `research/scripts/agent-watch.py:1580` | ✓ | `sed -n '1580p' research/scripts/agent-watch.py` |
| `.claude/settings.json` PreToolUse 8 道钩子清单（O12） | ✓ 逐条一致（顺序、matcher、脚本名全部相同） | Python 读 `.claude/settings.json` 的 `hooks` 键原样列出，见下方 |
| O13：`.claude/kb records` 下 273 个跟踪文件、252 个非 ASCII 文件名 | ✓ 逐字match（273/252） | `git -c core.quotepath=off ls-files .claude/kb records \| wc -l`；Python 逐行 `str.isascii()` |
| O5：判决字段值分布（`true`213/`not_run`51/`false`41/`0`30） | ✓ 前四行逐字match | `grep -rhoE 'name=verdict.*' research/results/ \| tr ' ' '\n' \| grep '=' \| sed 's/^[^=]*=//' \| sort \| uniq -c \| sort -rn` |
| O5：门禁 84 号「点名 3 个」、退出 0 | ✓ 复跑逐字match（见下） | `bash .claude/gate.d/84-verdict-false-named.sh` |
| O8：`research/scripts/memory-peaks.tsv` 含 `gate.d/54`\\`55`\\`57` 键的行数 | ✓ 核心断言（0 行）成立 | `grep -c 'gate.d/54\|gate.d/55\|gate.d/57' research/scripts/memory-peaks.tsv` → 0 |
| O8：`memory-peaks.tsv` 总行数「781 行」 | **分不清：该文件不进 git（`.gitignore:14`），是本机随其他会话跑内存包装命令持续追加的活文件，不在快照范围内**；此刻现查为 800 行，比报告写作时多 19 行，判断为期间别的会话新增的记账行，不算 Opus 引用有误 | `git check-ignore -v research/scripts/memory-peaks.tsv`；`grep -c . research/scripts/memory-peaks.tsv` → 800 |

**复跑**：`bash research/prompts/defs-m2-closeout-r1-opus-model/rerun.sh /tmp/claude-1000/defs-m2-closeout-r1-verifier/opus-rerun`，退出码 0，输出 61 行。与模型目录里 `probe-output.txt`（61 行）逐字段比对（检出文件路径这一类嵌临时路径的字段按存在与否比，不按整份哈希判 ✗）：**0 处不符**。

```
$ bash research/prompts/defs-m2-closeout-r1-opus-model/rerun.sh /tmp/claude-1000/defs-m2-closeout-r1-verifier/opus-rerun
（退出码 0）
```
（比对脚本：把两份输出的 `# detections file: …` 行统一替换成占位符后逐行比较，61/61 一致，0 处不符——见判定一览上一节「复跑」命令。）

**计数**：核了 50 处（含「引文原行」27 处按 27 次比对计），✓ 49 处，✗ 0 处，分不清 1 处（`memory-peaks.tsv` 总行数，见上）。复跑 1 次，与留存产物一致。

## 表三：本地攻方腿（3 次抽样：s1 干净、s2 带损坏、s3 干净）

方法：核 `defs-m2-closeout-r1-local-attack-translation-audit.md` 的转述核对表逐行「原文文件:行」，核 `defs-m2-closeout-r1-local-attack-runlog.md` 的复跑与检测器结论，逐条现查复跑。

| 引用/结论 | 结果 | 命令 |
|---|---|---|
| 转述核对表：「「执行前拒绝的写法」那十行——59-70 行——…本轮 diff 未动这十行」（核对表第 6-9 行，该表用这句话论证 D2 攻击面所依据的原文不在这一轮被判的改动范围内） | **✗ 与 `git diff` 矛盾**：`git diff HEAD -- .claude/agent-common.md` 显示第二个 hunk `@@ -54,7 +55,19 @@` 里，59-70 行（含「执行前拒绝的写法」总纲与全部①-⑦、pattern-process-guard/heavy-test-guard/write-guard 三行）全部标 `+`，即全部是本轮新增内容，旧版（HEAD）里这一段原文是一整段散文（「…hook 对五种写法在执行前拒绝：…」），没有①-⑦结构化清单；核对表这句自证过的「不在任何一个 hunk 里」的说法与现查结果相反 | `git diff HEAD -- .claude/agent-common.md \| grep "^@@"` → `@@ -45,7 +45,8 @@` 与 `@@ -54,7 +55,19 @@`；`git diff HEAD -- .claude/agent-common.md \| grep -n "① 起看门狗\|⑤ 整份覆盖\|⑥ 终止进程\|⑦ 在同一个"` 全部命中 `+` 开头行 |
| `.claude/agent-common.md:61-70`（C1-C10 各条引用行号，①-⑦与三行 hook 说明） | ✓ 全部 10 行逐字match（去除反引号与空格差异后内容一致；C1 引文中「主agent起」对应原文「主 agent 起」，去除了原文的空格，属转写时去标点，不改变内容，不计入✗） | `grep -n "^    ①\|^    ②\|^    ③\|^    ④\|^    ⑤\|^    ⑥\|^    ⑦\|^  - 上游的\|^  - 重型测试闸\|^  - 写闸" .claude/agent-common.md` |
| `_defs-m2-closeout-r1-diff.md:706-708`（P1-P3） | ✓ | `awk 'NR==706 \|\| NR==707 \|\| NR==708' research/prompts/_defs-m2-closeout-r1-diff.md` |
| `_defs-m2-closeout-r1-diff.md:742,747`（P37/P42 边界） | ✓ | `awk 'NR==742 \|\| NR==747' research/prompts/_defs-m2-closeout-r1-diff.md` |
| `_defs-m2-closeout-r1-diff.md:752,769`（P47/P64 边界） | ✓ | `awk 'NR==752 \|\| NR==769' research/prompts/_defs-m2-closeout-r1-diff.md` |
| 运行记录：s2 样本「UNTESTED」全部拼成「UNTESFED」，出现 27 次，两个检测器均判绿 | ✓ 完全复现：`grep -c UNTESFED` = 27，`grep -c UNTESTED` = 0，`oov-check.py`（绿，生词=0 拼接=0）与 `corruption-check.py`（绿，全部计数为 0）复跑均返回退出码 0 | `grep -c "UNTESFED\|UNTESTED" research/prompts/defs-m2-closeout-r1-local-attack-output-s2.md`；`python3 research/scripts/oov-check.py …`；`python3 research/scripts/corruption-check.py …` |
| 运行记录：三次调用退出码均为 0、无退出码 5、未超连续五次上限、目录清点与本文件逐行一致 | ✓（`ls research/prompts/ \| grep defs-m2-closeout-r1-local-attack` 现查：提示 1、核对表 1、运行记录 1、`-output-s1/s2/s3.md` 3 份，无 `-output-void*.md`，与记录一致） | `ls research/prompts/ \| grep defs-m2-closeout-r1-local-attack \| wc -l` → 6（不含快照与 opus-model 目录） |
| 运行记录：草稿目录 `/tmp/claude-1000/defs-m2-closeout-r1-local-attack/` 留空未用 | 核不动：草稿目录属临时目录，不在本报告写范围内核实其内容（不影响判定：留空与否不改变已交定稿产物） | 未核 |

**计数**：核了 8 处，✓ 7 处，✗ 1 处（转述核对表关于 diff 范围的自证性陈述与 `git diff` 矛盾），核不动 1 处。

## 总计

| 腿 | 核了 | ✓ | ✗ | 分不清/核不动 |
|---|---|---|---|---|
| 正推（Sonnet） | 38 | 27 | 11 | 0 |
| 云端攻方（Opus） | 50 | 49 | 0 | 1（分不清） |
| 本地攻方（3 次抽样） | 8 | 7 | 1 | 1（核不动） |
| **合计** | **96** | **83** | **12** | **2** |

Opus 报告另做了一次原样复跑（`rerun.sh`），61/61 字段与留存 `probe-output.txt` 一致，0 处不符。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（Opus 报告里「四句」自检、「什么会推翻」是否站得住，Sonnet 报告里「判定：一致/基本一致/做了一半」是否恰当，都不归这份报告）。
- D4（静态分支怎么判甲/乙/丙一/丙二）与本地攻方分到的「D2 事实表逐格填」不在这份核查任务点名的复核范围内，未逐格重复核对 Exhibit A/B 里全部 74 个 C 码、64 个 P 码与 Q1-Q12 的具体判词（那是转述准不准与产物有没有对上的核对，翻译核对表本身已做过一轮，我只抽查了行号与几条关键结论）。
- Sonnet 报告 D3-g 列的 17 处「只被通查扫过」的块、Opus 报告「没打中的形状」列的 10 条、「这条腿自己的限度」列的各项，均未逐条重新验证——这些是两条腿自己交代的欠账，不是需要核查员补的引用。
- 未核 Sonnet 报告与 Opus 报告里全部推理性的「情形」「派发情形」「四句」段落是否符合 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 的判据自证规则——那是主 agent 逐条现查的对象。
- 未跑 `gate.sh` 全量或任何重型测试阶段；除 84 号（读产物、不改文件，一次只读命令）外未跑任何门禁阶段。
- 未核实 `research/prompts/defs-m2-closeout-r1-local-attack-output-s1.md`、`-s3.md`（干净样本）与 s2（带损坏样本）里 Q1-Q12 对 74 个 C 码给出的具体判词方向是否一致——按分工这是主 agent 的事。
- 未核 `/tmp/claude-1000/defs-m2-closeout-r1-local-attack/` 草稿目录内容（据运行记录，留空未用）。
- 未核 `.claude/agents/three-way-verifier.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/prior-art.md`、`.claude/agents/sweep.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-materials.md`、`.claude/agents/mutation-triage.md`（除 O15 引用的两行外）这些快照里给了但三条腿报告都未大量引用的定义全文，只核了三条腿实际引用到的行。

## 文件

- 报告：`research/prompts/defs-m2-closeout-r1-verifier-output.md`（本文件）。
- 草稿：`/tmp/claude-1000/defs-m2-closeout-r1-verifier/`（快照校验日志、哈希比对脚本、Opus rerun 输出）。

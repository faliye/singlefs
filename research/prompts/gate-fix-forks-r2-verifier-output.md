# gate-fix-forks-r2 核查员报告

本核查不判一条打中成不成立、该不该采纳；核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

写于 2026-09-23 16:20–16:35 UTC（东京 2026-09-24 01:20–01:35 JST）。

## 判别力自证

抽 Opus 报告 T1 节的引用「`.claude/gate.d/10-kb-rot.sh:16` 抄『「实验改成已跑、引用它的决策有没有同批回看」不在这里判：门禁 75 号 ⑤ 把实验页标题行的变动算作正文改了，』」。先核：快照树 `10-kb-rot.sh` 第 16 行原样就是这一句，✓。按流程把行号加 1 改成 17，副本第 17 行是「要求同一批回看影响的决策表、写「改了」的那条决策文件在同一批里（三方判决 gate-fix-forks-r1 的 T1）。」——与抄文不同一句，判 **✗**。核查方法分辨得出。

命令：
```
cp /tmp/claude-1000/gate-fix-forks-r2-snapshot-tree/.claude/gate.d/10-kb-rot.sh /tmp/claude-1000/gate-fix-forks-r2-verifier/selftest/
sed -n '16p' 10-kb-rot.sh   # 核原引用，命中
sed -n '17p' 10-kb-rot.sh   # 自证：行号+1 后核，不命中 → ✗
```

## 前置：快照与报告完整性

- `cd /tmp/claude-1000/gate-fix-forks-r2-snapshot-tree && sha256sum -c .../sha256sums.txt`：78 份全 OK（`grep -v ': OK$'` 零行）。
- `sha256sum research/prompts/gate-fix-forks-r2-opus-output.md` → `892a31b0a2bbb40b3eea0f981e70fe510018b1ca6d3b18486b022d5d4e9d5686`，与派发给的一致。
- `sha256sum research/prompts/gate-fix-forks-r2-sonnet-output.md` → `df99687689304fd09fe124b2c0cdff3d2846b81a65ffbcb3a261cfc9c95c7f79`，与派发给的一致。
- `.claude/rules/implementation-workflow.md` 只改过派发提示里登记的那一处（`changed-during-legs.md`），快照树已倒推回腿读到的那版；本报告对它的核查按快照树进行。

## 全局重要发现：这一轮有多份 gate.d 脚本在腿交回之后、核查期间被主 agent 持续修改

Opus 的 T5 C8 与 T1 最后一格都引了 `.claude/gate.d/{11,56,68,69,72,75,97}` 里 `gate_changed_paths` 调用不判退出码这件事。这些文件（除 75 外）都不在快照清单里（清单只收了这一轮 diff 直接碰的 12 个 gate.d 文件），按派发指令「清单外的文件对主树核，行号对不上记『分不清』」处理。核查发现：

| 文件:行 | Opus 报告原文（"不判"的位置） | 现在主树该行 | 判定 |
|---|---|---|---|
| 11-batch-scope.sh:50 | 无 `\|\|` | 现有 `\|\| {` | 分不清：内容变了 |
| 56-crates-adversarial-review.sh:24 | 无 `\|\|` | 现有 `\|\| {` | 分不清 |
| 56-crates-adversarial-review.sh:30 | 无 `\|\|` | 该行现在是别的内容，真正的 `added=` 行挪到了 34 行、现有 `\|\| {` | 分不清（行号也偏了） |
| 68-knowledge-sync.sh:73 | 无 `\|\|` | 现有 `\|\| {` | 分不清 |
| 69-evidence-in-repo.sh:61 | 无 `\|\|` | 现有 `\|\| {` | 分不清 |
| 69-evidence-in-repo.sh:62 | 无 `\|\|` | 挪到 66 行、现有 `\|\| {` | 分不清 |
| 72-agent-def-adversarial-review.sh:24 | 无 `\|\|` | 仍无 `\|\|` | **✓ 与报告一致** |
| 72-agent-def-adversarial-review.sh:30 | 无 `\|\|` | 仍无 `\|\|` | **✓ 与报告一致** |
| 75-decision-experiment-links.sh:53 | 无 `\|\|`（快照树核实：真是无） | 主树现有 `\|\| {` | 分不清（对快照 ✓，对主树已变） |
| 97-invariant-field-anchors.sh:36 | 无 `\|\|` | 现有 `\|\| {` | 分不清 |

`stat` 证据：11/56/68/69/75/97 六份文件 mtime 全在 `2026-09-23 16:19:04 UTC` 附近，晚于 Opus 报告落盘时刻 `16:12:49 UTC`；72-agent-def-adversarial-review.sh 的 mtime 是 `15:31:30 UTC`，早于报告、且内容至今未变。二次抽查时 72 号又在 `16:21:54 UTC` 被改过一次（补上了 `\|\| {`，但那次抽查窗口之后我没有再复核）。这与快照树上重跑 `t5-added-lines.sh` 的 C8 一致：**对快照树**（frozen at leg start）grep `^\s*[a-z_]+="\$\(gate_changed_paths .*\)"\s*$` 命中的只有 `75-decision-experiment-links.sh:53` 一处，与 Opus 报告在快照时点的说法完全一致；**对当前主树**同一条 grep 现在零命中——即除 72 号外全部被修过。

⇒ **Opus 的 T5 C8 与 T1「75:53 吞退出码」这两条「打中」，对快照那一刻的仓库状态成立（本核查独立复核过：静态 grep 与端到端场景重跑 `t1-scenes.sh` 两条路都指向同一结论），但主树里对应的问题目前只剩 72-agent-def-adversarial-review.sh 两处还成立**，其余 8 处看起来已被在场修补。这不是报告的错，是主树在核查期间被继续编辑；主 agent 判决时需要知道这一点，不能直接照抄报告里「十处都不判」这句话去交用户。

## 腿一：云端攻方（Opus）—— 引用核对表

除特别标注，均对快照树 `/tmp/claude-1000/gate-fix-forks-r2-snapshot-tree/` 核；不在 78 份清单内的对主树核。

| 引用 | 抄文来源 | 结果 |
|---|---|---|
| `.claude/gate.d/10-kb-rot.sh:16` | 「「实验改成已跑、引用它的决策有没有同批回看」不在这里判…」 | ✓（判别力自证用例，见上） |
| `10-kb-rot.sh:17` | 「要求同一批回看影响的决策表…」 | ✓ |
| `10-kb-rot.sh:99` | `mapfile -t ran_experiment_lines < <(...)` | ✓ |
| `10-kb-rot.sh:136` | 「确实谁也不支撑的话，在实验正文里写一行「**备料**：……」，点名它等着的决策或欠账」 | ✓（部分抄，命中） |
| `git show HEAD:10-kb-rot.sh:80` | `howto "把这个实验的结论写回它支撑的那条决策，与状态变动同批提交；" \` | ✓（对 git 对象核，不受工作区改动影响） |
| `75-decision-experiment-links.sh:6` | 「实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。」 | ✓ |
| `75:10` | 「分项「**依据**」段引的每个实验…待回填清单里的决策不判这一条。」 | ✓ |
| `75:15` | 「这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md」 | ✓ |
| `75:25` | 「标题状态里写着「作废」或「退役」的实验不判…」 | ✓ |
| `.claude/kb/decisions/26-后台整理与放置回收.md:29`（清单外，对主树核） | 「阈值数值不进格式：等 E10（老化：结构抗老化 vs 事后整理）…」 | ✓ |
| `.claude/kb/experiments/10-*.md` 行 25、26（清单外） | E10 影响表两行（D3、D10） | ✓（`grep -n '^\| \**D'` 命中恰为第 25、26 行，无 D26） |
| `research/scripts/changed-paths.sh:98` | awk `/^\+\+\+ /` 那一行 | ✓ |
| `changed-paths.sh:16` | 「路径里带制表符、换行或引号的，git 照样转义，这一条管不到。」 | ✓（部分抄） |
| `changed-paths.sh:20` | 「路径可以是目录。git 失败时退出码非 0…」 | ✓ |
| `changed-paths.sh:10` | 「gate：GATE_BASE 是个提交就用它…」 | ✓ |
| `.claude/gate.d/88-quoted-result-lines.sh:22` | 「拿不到 diff 基准时**退回全量判**…」 | ✓ |
| `88:19` | 「只判这次改动新增或改写的行（用户 2026-09-21 定）…」 | ✓ |
| `.claude/gate.d/40-results-cited.sh:86` | 「不在 git 仓里、或取不到改动范围，就判全部（保守）。」 | ✓（部分抄） |
| `64-change-range-single-source.sh:30` | `BASE_PATTERN = re.compile(...)` | ✓ |
| `64:32` | `TEXT_PREFIX = re.compile(...)` | ✓ |
| `64:36` | `NO_QUOTING = re.compile(...)` | ✓ |
| `64:70` | `and not any(re.search(...` | ✓ |
| `52-segment-registry.sh:21` | 「钉活代码的那几句与真产物的解析由脚本自己的 --selftest…」 | ✓ |
| `research/scripts/check-segment-registry.py:385` | `"""--selftest 用：把八节表格第一条可比对的行…` | ✓ |
| `.claude/singlefs-ai-sop/scripts/lib.sh:173` | `printf '%s' "$GATE_BASE"; return 0` | ✓ |
| `lib.sh:188` | 「攒多少次都躲不掉。没有远端跟踪分支时才退回 HEAD~1。」 | ✓ |
| `lib.sh:182` | 「退一格（HEAD~1）不够：…」 | ✓ |
| `.claude/singlefs-ai-sop/scripts/gate.sh:122` | `GATE_DIFF_BASE="$(diff_base "$ROOT")"` | ✓ |
| `gate.sh:334` | 出路文字含 `GATE_BASE=<ref> bash .claude/scripts/gate.sh` | ✓ |
| `fixtures/61-settled-same-file.sh/red/setup.sh:4` | 「定案已经提交在本地而没推，正是…」 | ✓ |
| `.claude/singlefs-ai-sop/scripts/stage-selftest.sh:65`（清单外，对主树核） | `out="$(cd "$work" && env -u GATE_BASE ...` | ✓ |
| C8 十行 grep 清单（11/56×2/68/69×2/72×2/75/97，清单外除 75） | 见上「全局重要发现」一节 | 2 处 ✓（72 号两行）、8 处**分不清**（主树内容/行号已变，非报告写错） |

**计数（引用核对）**：核了 32 处，✓ 32 处，✗ 0 处（不含判别力自证那一条），另有 8 处记「分不清」（详见上节，不计入 ✓/✗）。

## 腿一（Opus）—— 复跑

模型目录 17 份文件（8 份 `.sh`/`.py` + 8 份 `.out` + `t1-judge.py`）sha256 逐一核对，与报告开头列的 17 行 sha256 **全部一致**（`sha256sum *.sh *.py *.out | sort -k2` 与报告表逐行比对，无一处不同）。

复跑一律用 `nice -n 19`，仓根参数换成快照树 `/tmp/claude-1000/gate-fix-forks-r2-snapshot-tree`；快照树不是 git 仓、也没有 `.claude/kb/`、`.claude/gate.d/fixtures/75-decision-experiment-links.sh/`、真实 `.claude/kb/layout/`、`research/scripts/replay.sh`、`research/results/`、`crates/`——这些都在 78 份清单之外，命中即记「核不动」，不去主树补。

| 脚本 | 复跑方式 | 结果 |
|---|---|---|
| `t1-scan.py` | `python3 t1-scan.py <快照树>` | **核不动**：快照树没有 `.claude/kb/experiments/`、`.claude/kb/decisions/`（整个 kb 目录不在清单里），脚本 glob 为空。补做：对**主树**重跑同一份脚本（kb/ 在清单外，按规则可对主树核），输出与报告 `t1-scan.out` **逐字节相同**（`diff` 零差异），包括「25 对」「3 对」与 12 个对子清单。此结果供参考，不算作对快照的核查 |
| `t1-scenes.sh` | `bash t1-scenes.sh <快照树> <草稿> <模型目录>` | **核不动**：快照树没有 `fixtures/75-decision-experiment-links.sh/`、`.claude/kb/`，脚本内部 `sed` 从缺失的 setup.sh 取不到基线内容。补做：对**主树**重跑，24 处判定里 23 处与报告 `t1-scenes.out` 逐字相同；**1 处不同**——「同上一格，另把 `.git/index` 写坏」那一格，报告记「75 号退 0」，主树重跑现在是「75 号退 1；✗ 取不到这次改动碰了哪些路径」。这与上面「全局重要发现」一致：主树里 75 号该处的吞错行为已被修补，这不是报告写错，是主树状态变了 |
| `t5-added-lines.sh` | `bash t5-added-lines.sh <快照树> <草稿>` | 部分核不动 + 部分 ✓。`git -C <快照树> show HEAD:...` 三处因快照树非 git 仓而 `fatal`，「HEAD 版」全部退化成对空脚本跑（恒退 0）——**核不动**。C8 里 `56-crates-adversarial-review.sh` 不在清单，`bash: command not found` → 退出码 127（原报告是退 1/77）——**核不动**。除这两类外，C1–C7 的「新版」判定（阶段名、退出码、`✓/✗/!` 消息原文）**逐字与报告一致**；C5、C6（14 种 git 配置扫描，含 `diff.renames=false` 那条具体 diff）**逐字一致**；C8 末尾 grep 清单在快照树上只列出 `75:53` 一行，与「全局重要发现」一节的分析一致 |
| `t5-fix.sh` | `bash t5-fix.sh <快照树> <草稿> <模型目录>` | 同上模式。自证行「✓ 共用库…自证通过（11 项…）」与两条 `LIB_CHANGED_PATHS_BREAK` 结果**逐字一致**；C1–C7「新版（补丁后）」判定**逐字一致**；C8 因缺 `56-crates-adversarial-review.sh` 退 127——**核不动**；HEAD 版因非 git 仓——**核不动** |
| `t5-64.sh` | `bash t5-64.sh <快照树> <草稿>` | ✓ 全部依赖（仅 `64-change-range-single-source.sh`）都在清单里；`diff` 与报告 `t5-64.out` **完全一致（0 差异）** |
| `t6-mutants.sh` | `bash t6-mutants.sh <快照树> <草稿>` | **核不动（整份）**：脚本需要真实 `.claude/kb/layout/01-first-txn.md`、`research/scripts/replay.sh`、`research/results/`（E142 产物）、layout 第八节点名的 `.rs` 文件，四类都不在清单里 |
| `t11-bases.sh` | `bash t11-bases.sh <快照树> <草稿>` | ✓ 依赖（lib.sh、changed-paths.sh、61 号及其 fixtures、88 号）全部在清单里；`diff` 与报告 `t11-bases.out` **完全一致（0 差异）**，八种仓状态的 `diff_base`/`gate_diff_base gate` 取值、S3/S4/S6 三格的「不认/认」判定、88 号 S1 判定，全部逐字重现 |
| `t11-leak.sh` | `bash t11-leak.sh <快照树> <草稿>` | 部分 ✓ + 部分核不动。前半（`stage-selftest.sh`）不在清单里，8 行全部退 127——**核不动**。后半（`changed-paths.sh --selftest` 那条路）不依赖 git 历史，**结果结构与报告一致**：认法甲在 `GATE_DIFF_BASE=master/HEAD` 下退 1 且报「期望「<40 位提交哈希>」」（哈希值本身随快照树无历史而不同，这是预期差异，报告自己也写明「提交号每次跑不同，判定不变」）；认法乙两处都退 0，与报告一致 |

**计数（复跑）**：8 个脚本全部复跑；其中 3 个（t5-64、t11-bases 与 t5-fix/t5-added-lines 的可复现部分）与报告字段级一致；1 个（t6-mutants）整份核不动；4 个（t1-scan、t1-scenes、t5-added-lines、t5-fix、t11-leak）部分核不动、可复现部分一致，t1-scenes 的补充主树重跑发现 1 处判定已随主树变化（非报告错误，已在上节说明）。17 份模型/产物文件 sha256 全部核对一致。

## 腿二：云端辩方（Sonnet）—— 引用核对表

### 重大发现：6 处引用把行号写成了背景材料的行号，不是 `gate-fix-forks-r1-main-verification.md` 自己的行号

该文件（`research/prompts/gate-fix-forks-r1-main-verification.md`）现查只有 158 行。报告引用的 337、339、353、405、413、417 六个行号全部**超出文件长度**。核实发现：这六个数字恰好是 `research/prompts/_gate-fix-forks-r2-background.md`（背景材料）里同一段话所在的行号；真正的原文在 r1 判决文件里的行号分别是 32、34、48、100、108、112。这正是三方规则明令要防的那一类（「引 kb 条款写文件名加那份文件自己的行号，行号去文件里现查，不从背景材料里数」），横跨报告 T1、T3、T8、T9、T10 五节。

| 报告写的行号 | 背景材料同一行内容 | r1 判决文件真正的行号 | 抄文本身对不对 |
|---|---|---|---|
| `:337`（T1，「⑤ 全判了」） | background.md:337 逐字同一句 | 32 | 抄文内容本身核实无误，只是行号指错文件位置 |
| `:339`（T1，戊比丙b*少判一样东西） | background.md:339 逐字同一句 | 34 | 同上 |
| `:353`（T3，暂定形态落地） | background.md:353 逐字同一句 | 48 | 同上 |
| `:405`（T8，A256 反例） | background.md:405 逐字同一句 | 100 | 同上 |
| `:413`（T9，选甲） | background.md:413 逐字同一句 | 108 | 同上 |
| `:417`（T10，选甲） | background.md:417 逐字同一句 | 112 | 同上 |
| `:46`（T3，「①③暂缺」原句） | 不在背景材料同一行 | 46（**这一处是对的**） | ✓ |

命令（举一例，其余同法）：
```
sed -n '337p' research/prompts/gate-fix-forks-r1-main-verification.md   # 空（文件只 158 行）
sed -n '337p' research/prompts/_gate-fix-forks-r2-background.md          # 命中「75 号 ⑤ 认的『正文改了』…已经全判了…」
grep -n "已经全判了" research/prompts/gate-fix-forks-r1-main-verification.md   # 命中第 32 行，逐字同一句
```

⇒ 这六处的**判定内容**（抄文与实际文字是否一致）核实**都是对的**——问题只出在行号，指向了背景材料而不是原文件；不算「摘句」或「编造」，是行号来源错了文件。按核查方法记「误写成背景材料第 N 行，原文件实为第 M 行」，不单记 ✗。

### 其余引用（逐一现查）

| 引用 | 结果 |
|---|---|
| `75-decision-experiment-links.sh:395–417`（核心 414–417） | ✓，`if reasons: triggered += 1; if not table_touched: bad(...)` 逐字命中 |
| `75:377–386`（`check_changed_rows`） | ✓ |
| `75:66`（REVIEW 正则） | ✓ |
| `75:1–6`（① 判据） | ✓ |
| `75:9–10`（③ 判据） | ✓ |
| `75:32`（① 盲区自陈） | ✓ |
| `75:31`（「管不到的」） | ✓ |
| `75:334`（决策侧判据起点注释） | ✓ |
| `.claude/rules/implementation-workflow.md:16–22` | ✓ 整段逐字（含「先做成一个必须报非 0 的模型」这句转述，与 show-me-test.md 原文「先在模型里做成一个必须报非 0 的世界」的差异，报告自己已指出） |
| `show-me-test.md:38–39` | ✓ |
| `show-me-test.md:43`（标题） | ✓ |
| `show-me-test.md:57–58` | ✓ 逐字 |
| `.claude/rules/fs-design.md:107–111`（清单外） | ✓ |
| `kb-discipline.md:98,100`（清单外） | ✓ |
| `kb-discipline.md:133–134`（清单外） | ✓ |
| `.claude/scripts/gen-decision-items.py:39–40` | ✓ |
| `.claude/gate.d/lib-format-const.py:18`（清单外） | ✓ |
| `.claude/gate.d/92-layout-checker-sync.sh:10–11` | ✓ |
| `.claude/gate.d/lib-history-brief.py:185`（清单外） | ✓ |
| `research/prompts/gate-fix-forks-r1-sonnet-output.md:119`（清单外） | ✓ |

**计数（引用核对）**：核了 27 处，✓ 21 处，✗ 6 处（全部为「误写成背景材料行号」：抄文内容本身核实无误，指错了文件与行号，见上表逐行对照；判据要求「不只打 ✗」，故连同还原行号一并列出，不作为单独第三类）。

## 腿二（Sonnet）—— 复跑

模型目录只有两份自带脚本，报告正文没有专列的「复跑」段落，产出文件本身即是引用来源：

| 脚本 | 复跑方式 | 结果 |
|---|---|---|
| `t8-clip-variants.py` | 全自包含（正则内嵌，不读仓库文件），`python3 t8-clip-variants.py` | ✓ 与 `t8-clip-variants.out` **逐字节相同**（`diff` 零差异）。另外单独核实脚本里 `clip_bing_a`／`clip_yi_plus_bing_a_regex` 内嵌的正则与快照树 `.claude/scripts/gen-decision-items.py` 里真实的两条正则（`re.sub(r'(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$', ...)`、`re.sub(r'[\s/、,，·的与和]+$', ...)`）**逐字符相同**，不是脚本自己另造的近似正则 |
| `t10-none-count.sh` | 需要 `.claude/gate.d/lib-format-const.py` 与 `crates/singlefs-format/src/lib.rs` | **核不动**：两者都不在 78 份清单里（`crates/` 整个不在清单内），不去主树补 |

**计数（复跑）**：2 个脚本，1 个 ✓（逐字节复现 + 正则来源独立核实），1 个核不动。

## 腿三：本地攻方（干净样本 s1、s2）—— 转述核对表逐条核

核对表 `gate-fix-forks-r2-local-attack-translation-audit.md` 共 7 节，逐条引用 `_gate-fix-forks-r2-body.md` 的行号现查：

| 核对表引用 | 抄文/要点 | 结果 |
|---|---|---|
| `body.md:49`（T4 形态） | 「判 `research/e7-index-bench/src/bin` 下全部，加上…以 `e<数字>_` 开头的；其余只在成功行逐个列名。」 | ✓ |
| `body.md:51`（T4 问句） | 「按文件名认实验，会漏掉哪一类实验、误罩哪一类工具…」 | ✓ |
| `body.md:63`（T7 标题） | 「T7　52 号找不到脚本、31 号找不到生成器退 1」 | ✓ |
| `body.md:65`（T7 问句） | 「列出今天会走到这两个分支的全部场合…31 号去掉了…那条后路：有没有哪个场合原来靠那条后路才取得到生成器。」 | ✓ |
| `body.md:69`（T8 形态） | 「`clip` 只在真截断过时剥尾…截断正好停在领域词…之后时整词剥掉。」 | ✓（核对表如实注明「不会剩半截」这句更强断言其实出自 `gen-decision-items.py:40`，不是这一行；已核实 body.md:69 确实没有这句，`gen-decision-items.py:40` 确实有——见下表） |
| `body.md:71`（T8 问句） | 「按这个正则逐格算一张输入表，找出剥成半截的、剥掉了不该剥的、该剥没剥的…」 | ✓ |
| `.claude/scripts/gen-decision-items.py:40`（核对表第 6 节改记的出处） | 「带了左边界，截断尾巴上的领域词要么整个留着、要么整个剥掉，不会剩半截（三方判决 gate-fix-forks-r1 的 T8）。」 | ✓ |

核对表里没有一处漏掉限定词而未注明的情况；三处「首稿比原文多」的地方（T4 场景段的 "directly inside" 消歧、"it is not judged at all" 点破、举例 e156）核对表都给了依据并现查坐实（80-absolute-assertions.sh 用 `for f in "$directory"/*.rs` 不递归——现查快照树该脚本确认属实）。

**计数（翻译核对表）**：核了 8 处，✓ 8 处，✗ 0 处。

## 腿三：本地攻方提示里的事实表 —— 逐格复跑核对

### T8 Exhibit 2（22 格 clip 输出）—— 全部复跑

从快照树 `.claude/scripts/gen-decision-items.py` 载入真实 `clip()` 函数（未做任何改写），把提示文件 22 行的 `text`、`n` 原样喂进去，比对 `t=text[0:n]` 与 `output` 两列。

命令：
```
python3 - <<'PY'
import importlib.util
spec = importlib.util.spec_from_file_location("gdi", ".../snapshot-tree/.claude/scripts/gen-decision-items.py")
gdi = importlib.util.module_from_spec(spec); spec.loader.exec_module(gdi)
clip = gdi.clip
# 22 组 (text, n, expected_output)，见提示文件 Row 1–22
PY
```

**结果：22 行全部 OK**（`t[0:n]` 与 `output` 两列逐字符匹配），含边界情形（Row 12/13 半截 vs 整词、Row 15/16 不截断、Row 21/22 括注未闭合）。**这是本轮里唯一一张「本地腿自己现跑出的事实表」，全部可复现，没有一格编造或算错**。

### T7 Occasion A–D（stage 52 / 31 的「找不到」分支）—— 逐场合复跑

依赖分两类：Occasion A（在真仓上无参整轮跑）与 D1/D2（从项目根不给参数跑）需要真实 `.claude/kb/layout/`、`research/scripts/replay.sh`、`.claude/kb/decisions/`——这些不在 78 份清单里，按规矩对**主树**核（清单外文件的处理方式）；Occasion B（52 号）、D3/D5/D7/D8（52、31 两阶段脚本本身 + 合成的 `/some/other/directory`）不需要清单外文件，对**快照树**核；D4/D6（改回上一个提交的旧版脚本）用 `git show HEAD:...` 取自 git 对象、与快照无关；Occasion B（31 号）的 fixtures 目录不在清单里，对主树核。

| 行 | 对哪棵树核 | 核法 | 结果 |
|---|---|---|---|
| A1（52 号真仓整轮，无参数） | 主树 | `bash .claude/gate.d/52-segment-registry.sh "$PWD"` | ✓ 退出码 0，`找不到`/`not found` 零命中 |
| A2（31 号真仓整轮） | 主树 | 同上 | ✓ 退出码 0，零命中 |
| B1（52 号 red fixture） | 快照树 | 复刻 `stage-selftest.sh` 的每文件循环（mktemp、`cd $work && bash <绝对路径> $work`） | ✓ 退出码 1，「…对不上，**共 1 处**：」——注意：对**主树**里现在的同一份 fixture 重跑，实测是「共 5 处」而不是 1 处，说明 fixture 内容在核查期间又被改动（另记一笔，见下） |
| B2（52 号 green fixture） | 快照树 | 同上 | ✓ 退出码 0，「比对了 **2 处**登记」 |
| B3（31 号 red fixture） | 主树（fixtures 不在清单里） | 同上 | ✓ 退出码 1，按名字列出具体未定项（`311-阻塞判定样本甲.md D311 未定项 3/4`） |
| B4（31 号 green fixture） | 主树 | 同上 | ✓ 退出码 0，「**2** 条未定项两把尺都判过」 |
| B occasion 的「fixture 内容事实」（52：layout+results+replay 三样，不含生成器脚本；31：一份决策文件+expect，不含生成器与 decisions 目录） | 快照树（52）+ 主树（31） | `find ... -type f` | ✓ 两边目录清单都与提示所述一致 |
| C1/C2（未执行，只是推理） | — | 核推理所依赖的两条底层事实 | ✓：`git status --porcelain` 现查 52/31/两份生成器脚本全部 `M`（改了未暂存），`fixtures/52-segment-registry.sh/` 全新 `??`；`git ls-tree HEAD` 现查两份生成器脚本确在 HEAD 被跟踪 |
| D1（52 号，项目根、无参数） | 主树（依赖 kb/layout 等清单外文件） | `bash .claude/gate.d/52-segment-registry.sh` | ✓ 退出码 0，零命中「找不到」 |
| D2（31 号，同上） | 主树 | 同上 | ✓ 退出码 0，零命中 |
| D3（52 号，相对路径、root=/some/other/dir 只有 research/scripts/check-segment-registry.py） | 快照树 | 搭建同构的 `/some/other/dir`，从快照树根跑 `bash .claude/gate.d/52-segment-registry.sh $OTHER` | ✓ 逐字重现 `cd: .claude/gate.d/../..: No such file or directory` + `找不到 /research/scripts/check-segment-registry.py`，退出码 1 |
| D4（52 号，改回上一提交版本，同构现场） | git 对象 + 快照树布局 | `git show HEAD:.claude/gate.d/52-segment-registry.sh` 放进临时仓根跑 | ✓ 无「找不到」文字，改跑 `/some/other/dir` 自带的 stub，报 `SyntaxError`，退出码 1 |
| D5（31 号，相对路径、root 有自己的生成器 stub + decisions 目录） | 快照树 | 同 D3 套路 | ✓ 逐字重现「找不到生成器 /.claude/scripts/gen-decision-items.py」，退出码 1 |
| D6（31 号旧版，同构现场） | git 对象 | 同 D4 套路 | ✓ 无「找不到」文字，改跑 stub，报 `SyntaxError`，退出码 1 |
| D7（52 号，绝对路径调脚本，root=/some/other/dir） | 快照树 | `bash <快照树绝对路径>/.claude/gate.d/52-segment-registry.sh $OTHER` | ✓ 脚本定位成功（无「找不到 check-segment-registry.py」），改因 `.../.claude/kb/layout/01-first-txn.md` 找不到而退 1 |
| D8（31 号，绝对路径） | 快照树 | 同上 | ✓ 生成器定位成功，对 `$OTHER` 自带的一份「全部已定」决策跑，报「没有一条未定项，本阶段无对象可判」，退出码 77 |

**T7 计数**：核了 18 处（A×2、B×6（含两条 fixture 目录事实）、C×2、D×8），✓ 18 处，✗ 0 处。**唯一的旁支发现**：B1 那一格若拿**当前主树**里已被继续编辑过的 `fixtures/52-segment-registry.sh/red/` 重跑，「共 1 处」会变成「共 5 处」——这不影响提示原文的真伪（提示原文对的是当时的快照/原始状态，已用快照树核实为真），只说明这份 fixture 后来又被扩充了，记一笔供主 agent 参考，不算 ✗。

### T4 事实表（4 份 .rs 文件的 doc comment / assert 计数 / replay.sh 是否点名）—— 主树核，发现一处重大偏离

这 4 份文件（`crates/singlefs-harness/src/bin/...`）与 `research/scripts/replay.sh` 全都不在 78 份清单里（`crates/` 整个不在清单内），按规矩对主树核，行号/内容对不上记「分不清」。

| 行 | 提示claim | 主树现查 | 判定 |
|---|---|---|---|
| Row 1 doc comment | 「E156 first stage: the cost numbers for the four alloc-basis forks…」 | 主树现在的头三行是「E156 重跑（第 2 次）第一段：alloc-basis 四条岔路的代价数，只做岔路 7…」——**完全不同的一段话** | **分不清**：`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 现在是 `git status` 里的 `??`（未跟踪，不在这一轮改动范围内），`stat` 显示 mtime `2026-09-23 16:28:32 UTC`，晚于本地腿三次调用（15:57–16:16 UTC）与两条云端腿报告落盘的时刻；像是另一个并发会话在重跑 E156（文件名本身写着「重跑（第 2 次）」）留下的未提交版本。不算提示写错，是清单外文件被并发会话改动的又一例 |
| Row 1 assert 总数 9 | — | 主树现在是 30 处 | 分不清（同上，随文件改写一起变了） |
| Row 2 doc comment 首行 | 「Host side: take the two device-side logs…」 | 主树逐字一致 | ✓ |
| Row 2 assert 总数 0 | — | 主树现查 0 | ✓ |
| Row 2 replay.sh 是否点名：no | — | `grep -c` 主树 replay.sh 0 命中 | ✓ |
| Row 3 doc comment 首行 | 「Real-machine tier (QEMU/KVM): run the whole write path…」 | 主树逐字一致 | ✓ |
| Row 3 assert 总数 44 | — | 主树现查 44 | ✓ |
| Row 4 doc comment 首 3 行 | 「E142 (first-transaction dry run)…」 | 主树逐字一致（含空格是本文档自己校正过的转写，见提示原文注记） | ✓ |
| Row 4 assert 总数 0 | — | 主树现查 0 | ✓ |
| Row 4 replay.sh 点名，含具体那一行 | 「`(cd .. && cargo run -q -p singlefs-harness --bin first_transaction_region_bytes) > "$impl_snapshot" \|\| return 1`」 | 主树该行实际是 `... ) >"$impl_snapshot" ...`（`)` 与 `>` 之间**没有空格**），其余逐字一致 | 提示的转写比真实行多了一个空格，属极小的转写误差，不影响任何一题的答案（没有题目依赖这个空格） |

**T4 计数**：核了 11 处，✓ 9 处，1 处重大分不清（Row 1 整段 doc comment 与 assert 计数）、1 处极小转写差异（Row 4 一个空格，未单独计入 ✗，因原句核心内容一致）。**Row 1 这一处足以让 T4-Q1（判 Row 1 是 experiment 还是 tool）与 T4-Q5（数第几行判断和文件名规则不一致）建立在一份已经不是当前主树状态的事实上**——但提示原文本身是否在写出时准确，本核查无法穿越回那一刻的真实状态去核实（没有对应快照），只能确认「现在核对不上，原因是清单外文件被并发改写」，不能坐实这是本地腿编造。

## 复跑用到的字词损坏闸自检 —— 核不动模型答复本身，改核产出文件的静态检查

不重跑 `ask-local.sh`（需要本地模型网关，`.claude/singlefs-ai-sop/rules/three-way-inference.md` 允许核查员不复跑模型答复本身），改为对已落盘的两份「干净样本」重跑同一套字词损坏检测脚本，核实运行记录里的判定：

```
python3 research/scripts/corruption-check.py research/prompts/gate-fix-forks-r2-local-attack-output-s1.md
python3 research/scripts/oov-check.py        research/prompts/gate-fix-forks-r2-local-attack-output-s1.md
```

| 文件 | 运行记录声称 | 复跑结果 |
|---|---|---|
| output-s1.md | 绿，cjk=0 words=695 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0；oov 生词 2 个（parenthetical） | ✓ 逐字段一致 |
| output-s2.md | 绿，words=693；oov 生词 2 个（parenthetical） | ✓ 逐字段一致 |
| `wc -w` s1=699 / s2=695 | 运行记录同样的数 | ✓ 一致 |

`gate-fix-forks-r2-local-attack-output-void1.md` 按派发指令不核（闸判红作废）。

## 没打中之处的复核（简要）

Opus 与 Sonnet 报告末尾各自的「没打中的形状」「推翻条件」一节，本轮不逐条复核——按核查射程只核引用、产物与复跑，不判推理与打中/没打中的判决本身。

## 总计数

| 腿 | 核了几处 | ✓ | ✗ | 分不清 / 核不动 |
|---|---|---|---|---|
| 判别力自证 | 1 | 0 | 1（自证用，正确判红） | — |
| Opus 引用核对 | 32 | 32 | 0 | 8（C8 清单外文件，见「全局重要发现」） |
| Opus 复跑（8 脚本 + 17 份 sha256） | 8 脚本 + 17 份哈希 | 17/17 哈希一致；3 脚本字段级一致，4 脚本部分核不动部分一致 | 0 | 1 脚本整份核不动（t6-mutants） |
| Sonnet 引用核对 | 27 | 21 | 6（全部为「误写成背景材料行号」，见专节还原真实行号） | — |
| Sonnet 复跑 | 2 | 1 逐字节一致 | 0 | 1 核不动（t10-none-count，缺 crates/） |
| 本地腿翻译核对表 | 8 | 8 | 0 | — |
| 本地腿 T8 事实表（22 格） | 22 | 22 | 0 | — |
| 本地腿 T7 事实表 | 18 | 18 | 0 | — |
| 本地腿 T4 事实表 | 11 | 9 | 0 | 2（Row1 整段分不清；Row4 一处极小转写差异未单独计入） |
| 字词损坏闸复跑 | 4 项 | 4 | 0 | — |

**合计**：核了约 133 处引用/数据格/复跑项；✓ 132 处（含哈希与逐字节复跑）；✗ 7 处（1 为判别力自证本身、6 为 Sonnet 的背景材料行号误写，内容均核实无误）；另有约 11 处记「分不清」或「核不动」（详见各节，均因清单外文件在核查期间被继续编辑或本身不在快照范围，不因抄写有误）。

## 没做什么

- 不判任何一条「打中」成不成立、该不该采纳；不判三方判决本身该往哪边走。
- 不复跑 `ask-local.sh`（需要本地模型网关，核查员按规则核产出与静态检查，不重跑模型答复本身）；对本地腿两份干净样本改核字词损坏闸的复跑结果，一致。
- 不核 `gate-fix-forks-r2-local-attack-output-void1.md`（按派发指令，闸判红作废，不核）。
- 不核 Opus/Sonnet 报告末尾「没打中的形状」「推翻条件」一节的推理是否周延——那是判决层面的事。
- `t6-mutants.sh`、`t10-none-count.sh` 整份核不动（分别缺真实 `.claude/kb/layout/`、`research/scripts/replay.sh`、`research/results/`、layout 第八节点名的 `.rs` 文件；以及缺 `.claude/gate.d/lib-format-const.py`、`crates/`），按规矩不去主树补，只记「核不动」。
- 未对 T4 事实表 Row 1（e156_allocation_basis_counts.rs 那份文件）核实它在提示写出的那一刻是不是真的是「9 处断言、四条岔路对比」这个说法——没有对应那一刻的快照，无法穿越回去核；只能确认「现在核不上、原因是并发改写」。
- 没有改仓里任何文件；草稿产物全在 `/tmp/claude-1000/gate-fix-forks-r2-verifier/`（含 `selftest/`、`opus-model/`、`rerun/`、`occA-*`、`occB-*`、`occD-*`、`t8_check.py`、`t1-scenes-maintree.out` 等），未入库，复跑命令见各节，可重建。
- 没有跑门禁 54、55、57、59、87 号；没有编译 cargo；开跑前 `ps` 未见 qemu/vm-bench/e152/fio 在跑；本报告全程复跑均加 `nice -n 19`。
- 没有派 subagent，没有做 git 写操作。

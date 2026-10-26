# 第十批钩子：草稿目录一行收成一份认法、交回闸命名检查只认真跑完的、三方腿报告路径一处认、⑳ 自证读真提示（2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#10。依据：research/prompts/landing-audit-r4-main-verification.md 节「二、逐格判」K2-4、K2-5、K3-1、K3-2 与 K1 ⑳ 那一行；拆分提案第九十节 ⑳、㉗、㉘。

## 结论

四条的验收都达成，改动都已经经 scripts/gate/hook-swap.py 定点换进主工作区，换上后在主工作区重跑了三份钩子与新库的 --selftest，都是绿的。

- K2-4 并 K3-2：新建共用库 `.claude/hooks/lib_prompt_scratch_line.py`，只认行首「草稿目录：」那一行（取骨架那一种）。四处都改成调它：派发闸 ⑯、⑰，完工清理闸，交回闸 ⑥。攻方三条探针改后：relay-comma 2→0，relay-list 2→0，own-zai 改前改后都是 0（没有规范行，按裸文件名判，路径带目录就不判）。
- K3-1：交回闸 ⑤ 只认真跑完的那两条。攻方 naming_probe.out 里的五种从「退 0」变成「退 2」；「对照：两条都真跑：退 0」与「根写成 "$CLAUDE_PROJECT_DIR" 拒」照旧。
- K2-5：三方腿报告路径的认法收进 `.claude/hooks/lib_three_way_legs.py`（is_leg_report_path、leg_report_mentions），派发闸 ⑰ 与交回闸 ② 都调它。没填的 `<轮>` 占位不认，子目录下的也不认。攻方 leg-form-placeholder 0→2；交回闸 ② 遇到只写子目录路径或只写占位的交回，0→2。
- ⑳：派发闸自证现在读 `research/prompts/landing-audit-r3-opus-model/prompts/` 下的 14 份真提示，逐份核 sha256 开头和报告类两种拒。目录不在时那一格报「本次未跑」，不记绿，有 GATE_NOT_RUN_FILE 时也写进去。`grep -c 'landing-audit-r3-opus-model' .claude/hooks/runner-dispatch-guard.sh` 现在是 1。

要主 agent 拍板的一处（K3-1 退出码）：判决原话是「只认工具结果退出码 0」，但派发提示又要「两条都真跑：退 0」照旧，而攻方这一格里 naming-lint 是判红退 1 的；共用约束也写明命中在别的文件里的照写、照样交回。照字面只认 0，就会把这种合法交回拒掉，交不回去。所以我按两种都认来做：退 0 的照收；退 1 只在退出码确实是那条命名检查自己的时才收——整条命令里只有赋值、env、cd、export 和命名检查，最后一条是命名检查，它前面没有用 && 接别的命令。3、77、124、2 这些退出码一条都不收。写法在 sequential_naming_text 的文档串里。要是主 agent 判照字面只认 0，把 NAMING_COMPLETED_EXIT_CODES 改成 (0,)，再把自证「两条用 ; 连着、后一条判红退 1 放行」这一格和 naming-run 那一格改成拒。

推翻条件：
- 下列任一条在主工作区复跑出来，就说明这一版不对：三份钩子的 --selftest 有红；攻方 scratch_prompts.sh 里 relay-comma 或 relay-list 退 2；naming_probe.py 那五种里有一种退 0；leg-form-placeholder 退 0。
- 真会话里照骨架写的提示被 ⑯ 漏建写者锁（行首带列表记号或粗体的「草稿目录：」，现在不认），说明收严收过头了。

## 改过的文件

主工作区，都以开工时的原文为基线定点换上。原文备份在 `originals/`，清单在 `originals/manifest.tsv`。

| 文件 | 改前 sha256 | 现在 sha256 | 我这一批改了几行（diff 对备份，`<`/`>` 行数） |
|---|---|---|---|
| .claude/hooks/runner-dispatch-guard.sh | ce5cc261… | 03b8c2d13b5d64204cbc7fa7f904e60d3411ddb4f1503805a21dd9ef7aa3d5b2 | 213 |
| .claude/hooks/handback-guard.sh | bab11748… | 01fa3f9c87dea9fd19b09a0e4d4b96cc58e81d830d0919ebec51d3faaadcdfb0 | 412 |
| .claude/hooks/completion-cleanup-guard.sh | 0ca8de769e5c…（换之前核过，还是这个开头） | 09143da12d99934cd7ffd60a21d5fbf793ada8310186de7af6c50acc10e2be27 | 26（只改认草稿目录那一行的读法，加了模块路径一行和两格自证；运行目录归属判法没碰） |
| .claude/hooks/lib_three_way_legs.py | be473212… | f67d526a033d60663f4fc2ed98da32c3cbe2893c27df61a3d4445eea0d265089 | 20 |
| .claude/hooks/lib_prompt_scratch_line.py | 新建 | d38184ae3014bd70e1713308a30c0655f918b8c39f7432deb0e48caa58215c22 | 173 行 |
| .claude/hooks/selftest-registry.tsv | 6f5afcca… | 7d27bea6e927a194d2c0c2bc33b3fa6304c5814797ea657b88f630518db0d32e | 1（加 lib_prompt_scratch_line.py 一行 light） |

`git diff --stat` 原样（工作区对暂存区。selftest-registry.tsv 那 9 行里只有 1 行是我加的，其余 8 行是别的会话在我备份之前改的、没暂存；新库还没进 git，不在这里）：

```
 .claude/hooks/completion-cleanup-guard.sh |  26 +-
 .claude/hooks/handback-guard.sh           | 412 +++++++++++++++++++++++++++---
 .claude/hooks/lib_three_way_legs.py       |  20 ++
 .claude/hooks/runner-dispatch-guard.sh    | 213 +++++++++++----
 .claude/hooks/selftest-registry.tsv       |   9 +-
 5 files changed, 593 insertions(+), 87 deletions(-)
?? .claude/hooks/lib_prompt_scratch_line.py
```

换上的记录：hook-swap 真换了六次（另有一次 --dry-run），每次都是草稿里先验、换上后在主工作区再验，全部退 0，一次都没回滚。规格在 `specs/`，喂法在 `feeds/`，喂法输出和反向规格在 `hook-swap/`：dispatch-1（派发闸加 lib_three_way_legs，27 处）、dispatch-2（派发闸文件头开关表，3 处）、handback-1（35 处）、handback-2（文件头 1 处）、completion-1（5 处）、legs-2（放宽报告名字的字符，2 处）。

新写的 `# gate-similar:` / `# hook-events:` 行：没有。新建的是共用库，不是钩子，也不是门禁阶段。三份钩子原有的这两种行都没动。

## 一、K2-4 并 K3-2：「草稿目录：」一行收成一份认法

改法：新库 `.claude/hooks/lib_prompt_scratch_line.py` 提供两样，认法写在文件头「认法」一段。
- `scratch_line_values(prompt)`：一行开头（前面只许空格和制表符）是「草稿目录」，紧接全角或半角冒号，后面跟路径（可带反引号）。路径到空白、引号、中文标点、括号或尖括号为止，末尾的 . , ; : 去掉。`<填 …>` 占位不认。闭合了的围栏代码块里的行不认。
- `scratch_directories(prompt, include_roots=False)`：用同目录 lib_work_root.py 展开路径，只留落在工作根或过渡期旧根底下的目录，根本身不算。

四处改调它：
- 派发闸 ⑰：`report_scratch_directories`，原来的 `REPORT_SCRATCH_MENTION` 删了。
- 派发闸 ⑯：`scratch_directories_named`，原来的 `WRITER_LOCK_DIRECTORY_LINE` 删了；开关 writer-lock-root-locked 改成经 include_roots 传。两处都走新加的 `prompt_scratch_directories`：读不到库、或库抛异常，就交空表，stderr 写一句。
- 完工清理闸：`subagent_roots` 改调 `scratch_line_values`，`SCRATCH_LINE_PATTERN` 删了，MODULE_PATHS 加一行。
- 交回闸 ⑥：`scribe_spec_entries` 改调新加的 `prompt_scratch_directories`。

和原来四种认法比，变了这些：
- ⑰ 原来不限行首、冒号可有可无，现在收严，这是这一条要的。
- ⑯ 和交回闸 ⑥ 原来认行首带「- * > _」的，现在不认。
- 完工清理闸原来路径到空白或反引号为止，现在也在中文标点和括号前截断。

攻方探针 `research/prompts/landing-audit-r4-opus-model/probes/scratch_prompts.sh`。改前那一份是在 `before/` 里拷开工时的树跑的，和攻方 `out/scratch_prompts.out` 用 `diff -q` 比，逐行相同。改前原样（探针自己用 `cut -c1-200` 截行，有几处截在多字节字中间，写进这份报告时换成了 U+FFFD，全报告共 3 处）：

```
relay-comma（clerk）：退 2；✗ 派 clerk 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：「/home/fy5090/.local/share/singlefs/work/k3-probe-previous/report.md」。
relay-control（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 18 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的
relay-list（clerk）：退 2；✗ 派 clerk 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：「/home/fy5090/.local/share/singlefs/work/k3-probe-previous/report.md」。
own-zai（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 17 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）
own-control（clerk）：退 2；✗ 派 clerk 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：「/home/fy5090/.local/share/singlefs/work/k3-probe-mine/report.md」。
leg-filled（three-way-attack）：退 0；  ✓ 派发闸照这份提示放行（three-way-attack，查了 21 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者�
leg-form-placeholder（three-way-attack）：退 0；  ✓ 派发闸照这份提示放行（three-way-attack，查了 21 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者�
leg-subdir（three-way-attack）：退 2；✗ 派 three-way-attack 的提示整份提示里没有一处 research/prompts/<名>-output.md（报告行点名的，或不在「」『』“”与反引号里的）；报告行点名的「research/p
交回闸 ② 的 LEG_REPORT_MENTION 认出： ['research/prompts/x-r1/x-r1-opus-output.md']
交回闸 ② 的 LEG_REPORT_MENTION 认出： ['research/prompts/<轮>-opus-output.md']
```

改后（`final/` 里拷主工作区换上之后的树跑的）原样：

```
relay-comma（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 18 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的
relay-control（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 18 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的
relay-list（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 20 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的
own-zai（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 17 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）
own-control（clerk）：退 2；✗ 派 clerk 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：「/home/fy5090/.local/share/singlefs/work/k3-probe-mine/report.md」。
leg-filled（three-way-attack）：退 0；  ✓ 派发闸照这份提示放行（three-way-attack，查了 21 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者�
leg-form-placeholder（three-way-attack）：退 2；✗ 派 three-way-attack 的提示整份提示里没有一处 research/prompts/<名>-output.md（报告行点名的，或不在「」『』“”与反引号里的）；报告行点名的「research/p
leg-subdir（three-way-attack）：退 2；✗ 派 three-way-attack 的提示整份提示里没有一处 research/prompts/<名>-output.md（报告行点名的，或不在「」『』“”与反引号里的）；报告行点名的「research/p
交回闸 ② 的 LEG_REPORT_MENTION 认出： ['research/prompts/x-r1/x-r1-opus-output.md']
交回闸 ② 的 LEG_REPORT_MENTION 认出： ['research/prompts/<轮>-opus-output.md']
```

逐条判（都照派发提示写的验收）：
- relay-comma、relay-list：2→0，转述里的不拦了。
- own-zai：0→0。提示里没有规范行，只按裸文件名判；报告路径带目录，所以不判。派发提示写明这一种判对。
- own-control：2→2，规范行之下的 report.md 照拒。
- 末两行是探针脚本里抄死的交回闸旧正则，不读钩子，所以改前改后一样。交回闸 ② 的真读数在第三节。

自证（全在主工作区跑，都绿）：
- 新库 `python3 .claude/hooks/lib_prompt_scratch_line.py --selftest`，18 格绿。
- 派发闸新加：
  - 「草稿目录一行:」4 格：转述的「，草稿目录 <别处>」不判；列表的「- 草稿目录 <别处>」不判；「草稿目录在」带目录的不判；「草稿目录在」的提示照判裸文件名。
  - 「草稿目录一行:⑰ 与 ⑯ 调的是 lib_prompt_scratch_line.py 的 scratch_directories」1 格：把库函数换成替身，两处交回的都是替身给的。
  - 原有一格改了期望：「报告名:草稿目录只写在句中（没有行首「草稿目录：」一行），带目录的报告路径不判」，从 2 改成 0，依据判决 K2-4。
  - ⑰ 样本的 report_scratch_line 改成骨架那一行「草稿目录：」。走 stdin 的那一格另换一份不带草稿目录行的提示（「报告：report.md」），免得在真工作根里建锁。
- 完工清理闸新加 2 格：subagent-assigned-relay-ignored，转述里的两个目录里有仓副本也不算这个子 agent 的；subagent-assigned-shared-reading，认法换成替身后拦下替身给的目录。
- 交回闸新加 2 格：「写回员提示里列表起头的「- 草稿目录：」是转述，不在那里找起草的规格」；「草稿目录一行:⑥ 调的是 lib_prompt_scratch_line.py 的 scratch_directories」（替身）。

弄坏开关 `LIB_PROMPT_SCRATCH_LINE_BREAK=<项>`（新库文件头三条都写全了；平常跑设了就退 2，点名只在自证里认，见第六节）：

| 项 | 弄坏哪一处 | 实跑红在哪 |
|---|---|---|
| label-anywhere | 退回第九批 ⑰ 的认法：哪都认，冒号可有可无 | 新库 4 格（点名的两格在里面）。派发闸 4 格：转述两格、句中那一格、真提示 a9717cf07fb6e7c21，派发闸文件头点名前三格。交回闸 1 格（列表那一格，文件头点名了）。完工清理闸 subagent-assigned-relay-ignored（文件头没点名，见第七节） |
| code-blocks-read | 闭合围栏里的那一行也认 | 新库 1 格（点名的那一格）；三份钩子没有格红 |
| roots-unfiltered | 不看在不在工作根下 | 新库 2 格（点名的两格）；派发闸 1 格「写者锁:草稿目录写成工作根本身不建锁」 |

## 二、K3-1：交回闸 ⑤ 只认真跑完的命名检查

改法（`.claude/hooks/handback-guard.sh`）：
- `tool_calls` 多交一样退出码：不是错误的记 0，「Exit code N」的记 N，别的记 None。
- 每条 Bash 命令先过 `completed_naming_text`，这一步调 `sequential_naming_text`，只留下真跑完的简单命令。留下的再交给原来的 `naming_check_runs` 认根。
- 顶层结构由新加的 `naming_sequence` 切，词法用 lib_shell_words 的 `shell_tokens`、`strip_data_heredocs`、`skip_prefixes`，不另抄一份切词。
- 原来的识别拆成两层：`naming_check_occurrences` 认是哪一条、根是什么，并且带上路过的前缀名（看有没有 timeout）；`naming_check_runs` 在它上面包一层。

只认顺序执行位置上的那两条：
- 不在 `||` 右侧。
- 不在 `if`、`while`、`until`、`for`、`select`、`case` 里，不在函数体里（`f() {…}` 与 `function f {…}`）。
- 同一段里，前面用 `&&` 连着的只能是 cd、pushd、popd、export、赋值、env，或另一条命名检查。
- 不放后台（`&`）。
- 前面没有 exit、return、logout，也没有接着命令的 exec。

再按这一条 Bash 的退出码收：
- 退 0：符合上面的都收。有两种例外：前面用 `&&` 连着 cd 或命名检查的，要在顶层最后一段；包着 timeout 的，要是最后一条。
- 退 1：只在整条里只有赋值、env、cd、export 和命名检查、最后一条是命名检查时收，而且只收同一段里前面没有接 `&&` 的那几条。
- 别的退出码一条都不收。
- `bash -c` 那一段照同一套规则往里收一层；外层不传退出码的时候，按认不出退出码办：只收前面没有可能失败的命令、也不包 timeout 的那几条。

为什么也认退 1，见开头的「要主 agent 拍板」。

攻方探针 `probes/naming_probe.py`。改前那一份和攻方 `out/naming_probe.out` 用 `diff -q` 比，逐行相同。改前原样：

```
对照：改了 .rs 没跑：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
对照：两条都真跑：退 0
false && 两条（一条都没执行，退 1）：退 0
true || 两条（一条都没执行，退 0）：退 0
if false; then 两条; fi：退 0
bash -n 只查语法：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
timeout 0.1 包着、被杀退 124：退 0
两条写进函数定义不调用：退 0
根写成 "$CLAUDE_PROJECT_DIR"（真跑了两条）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
worktree 里改、根写主工作区（应拒）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
```

改后原样：

```
对照：改了 .rs 没跑：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
对照：两条都真跑：退 0
false && 两条（一条都没执行，退 1）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
true || 两条（一条都没执行，退 0）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
if false; then 两条; fi：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
bash -n 只查语法：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
timeout 0.1 包着、被杀退 124：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
两条写进函数定义不调用：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
根写成 "$CLAUDE_PROJECT_DIR"（真跑了两条）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
worktree 里改、根写主工作区（应拒）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
```

那五种（false &&、true ||、if false、函数定义不调用、timeout 0.1 退 124）都从 0 变成 2；两个对照、`bash -n`、`$CLAUDE_PROJECT_DIR` 那一格、worktree 那一格都照旧。

为了核我自己定的这几条规则，另写了形状探针 `probes/naming_shapes.py`，21 种写法，按真入口喂进去。改后 21 种全对；改前用同一份探针，13 种不对。改后原样：

```
对：cd 之后 && 连着两条、接管道，退 0：退 0（应当 0）
对：两条用 ; 连着，后一条判红退 1：退 0（应当 0）
对：两条用 && 连着、退 1（后一条可能被跳过）：退 2（应当 2）
对：cd && 一条、退 1（认不出是 cd 失败还是判红），另一条单独跑：退 2（应当 2）
对：前面有 exit 0：退 2（应当 2）
对：一条放后台：退 2（应当 2）
对：timeout 包着、是最后一条、退 0：退 0（应当 0）
对：timeout 包着、不是最后一条：退 2（应当 2）
对：bash -c 里 false && 起：退 2（应当 2）
对：bash -c 里单独起：退 0（应当 0）
对：two-logs.sh 包着、判红退 1：退 0（应当 0）
对：写在命令替换里：退 2（应当 2）
对：for 循环里：退 2（应当 2）
对：until 条件里：退 2（应当 2）
对：case 里：退 2（应当 2）
对：function 关键字定义不调用：退 2（应当 2）
对：|| 之后再 && 接：退 2（应当 2）
对：一条退 2（用法错）：退 2（应当 2）
对：一条退 77（本次未跑）：退 2（应当 2）
对：赋值与 export 之后 && 起、退 0：退 0（应当 0）
对：if 判完之后另起一行跑：退 0（应当 0）
```

自证：交回闸新加 8 格，都在主工作区跑绿。
- 五种各一格：「false && 起的两条没执行，不算跑过」「true || 起的两条没执行，不算跑过」「if false; then 两条; fi 不算跑过」「两条写进函数定义不调用，不算跑过」「timeout 0.1 包着、被杀退 124，不算跑过」。
- 另三格：「exit 0 之后的两条没执行，不算跑过」「两条用 ; 连着、后一条判红退 1 放行」「两条用 && 连着、退 1（后一条可能被跳过）拒」。
- 原有的「改了 .rs 之后两条命名检查都跑了放行」（naming-run，lint 退 1）、「经 two-logs.sh 包着跑命名检查放行」、「cd 之后根写成 $(pwd)、接管道跑命名检查放行」照旧是绿的。

弄坏开关（交回闸文件头都写了，实跑核过点名的项都在失败行里）：
- `HANDBACK_GUARD_BREAK=naming-position-unchecked`：不看位置，退 0 或 1 的整条都收。红 6 格：false &&、true ||、if、函数、exit 之后、&& 连着退 1。
- `HANDBACK_GUARD_BREAK=naming-exit-code-unchecked`：退出码一律当 0。红 2 格：timeout 被杀退 124、&& 连着退 1。

## 三、K2-5：三方腿报告路径一处认

改法（`.claude/hooks/lib_three_way_legs.py`，加在文件头和末尾）：
- 形态是 `research/prompts/<名>-output.md`，`LEG_REPORT_NAME` 是 `[^\s/<>`'"，。；：、（）()「」『』“”,;*|]+`：没填的占位（带 < 或 >）不认，子目录下的也不认。名字里有中文照认，因为骨架自证 `scripts/agents/dispatch-prompt.py --selftest` 把占位填成「已填」，起初只认 ASCII 时它那三格红了，放宽后转绿。
- `is_leg_report_path(relative)` 整个比，`leg_report_mentions(text)` 在一段文字里找，前后都有边界。

两处调用：
- 派发闸 ⑰ 报告行那一支改调 `leg_report_path_verdict`，它里面调 is_leg_report_path。整份提示那一支改调 leg_report_mentions。删了 THREE_WAY_REPORT_FORM 和 THREE_WAY_REPORT_MENTION。
- 交回闸 ② 改调新加的 `leg_reports_named`，它里面调 leg_report_mentions。LEG_REPORT_MENTION 只留在弄坏开关下用。

读数：
- 派发闸那一侧见第一节攻方原样：leg-form-placeholder 从退 0 变成退 2，leg-subdir 照旧退 2，leg-filled 照旧退 0。
- 交回闸那一侧另写了探针 `probes/leg_report_handback.py`。它拿攻方末两行的那两句交回正文，加一句填好的对照，喂真入口。

改前原样（`before/`）：

```
「报告 research/prompts/x-r1/x-r1-opus-output.md sha256 …」：交回闸退 0；! handback-guard：cite-check 退出码 2，引文这一条没判（放行）：✗ 读不了 /home/fy5090/.local/share/singlefs/work/landing-audit-b10-；lib_three_way_legs.leg_report_mentions 认出 （这棵树的库里没有 leg_report_mentions）
「报告 research/prompts/<轮>-opus-output.md」：交回闸退 0；! handback-guard：cite-check 退出码 2，引文这一条没判（放行）：✗ 读不了 /home/fy5090/.local/share/singlefs/work/landing-audit-b10-；lib_three_way_legs.leg_report_mentions 认出 （这棵树的库里没有 leg_report_mentions）
「报告 research/prompts/x-r1-opus-output.md sha256 …」：交回闸退 0；! handback-guard：cite-check 退出码 2，引文这一条没判（放行）：✗ 读不了 /home/fy5090/.local/share/singlefs/work/landing-audit-b10-；lib_three_way_legs.leg_report_mentions 认出 （这棵树的库里没有 leg_report_mentions）
```

改后原样：

```
「报告 research/prompts/x-r1/x-r1-opus-output.md sha256 …」：交回闸退 2；✗ 报告里的引文对不上（scripts/agents/cite-check.py）：；lib_three_way_legs.leg_report_mentions 认出 []
「报告 research/prompts/<轮>-opus-output.md」：交回闸退 2；✗ 报告里的引文对不上（scripts/agents/cite-check.py）：；lib_three_way_legs.leg_report_mentions 认出 []
「报告 research/prompts/x-r1-opus-output.md sha256 …」：交回闸退 0；! handback-guard：cite-check 退出码 2，引文这一条没判（放行）：✗ 读不了 /home/fy5090/.local/share/singlefs/work/landing-audit-b10-；lib_three_way_legs.leg_report_mentions 认出 ['research/prompts/x-r1-opus-output.md']
```

只点名子目录路径的、只点名 `<轮>` 占位的，改前都退 0：路径被认成报告，交给 cite-check，cite-check 读不了、退 2，于是放行。改后这两种都退 2，拒绝说明里有「交回里没点名报告文件」。填好的那一句照旧交给 cite-check。

自证：
- 派发闸新加 4 格：「三方报告:报告行写没填的 <轮> 占位、整份提示没有别的 -output.md，拒」「三方报告:报告行写子目录下的 -output.md，拒」「三方报告:整份提示里只在子目录下有 -output.md，拒」，以及「三方报告:报告行与整份提示两处形态调的是 lib_three_way_legs.py 的 is_leg_report_path 与 leg_report_mentions」（两个函数换成替身来核）。
- 交回闸新加 3 格：「三方腿报告路径:只点名子目录下的 -output.md 不认，按没点名报告拒」「三方腿报告路径:只点名没填的 <轮> 占位不认，按没点名报告拒」，以及「三方腿报告路径:交回闸调的是 lib_three_way_legs.py 的 leg_report_mentions」（替身）。

弄坏开关：
- `RUNNER_DISPATCH_GUARD_BREAK=leg-report-placeholder-kept`：报告行那一支退回改前的正则。红 2 格，占位那一格和替身那一格。
- `HANDBACK_GUARD_BREAK=leg-report-subdirectory-kept`：交回闸退回改前的 LEG_REPORT_MENTION。红 3 格。

## 四、⑳ 派发闸自证读真提示

改法（`.claude/hooks/runner-dispatch-guard.sh`）：
- 新加 `REAL_PROMPTS_DIRECTORY`、`REAL_PROMPT_EXPECTATIONS`（14 份）和 `real_prompt_cells`。
- 每份核三样：sha256 开头 16 位，和 `research/prompts/landing-audit-r3-opus-output.md` 列的逐份相同；报告文件名拒不拒；三方腿报告路径拒不拒。三方腿形态按本仓定义 frontmatter 判。
- 期望照九十节 ⑳、第三轮判决 K2-1、K2-2、K2-3 和第四轮判决 K2-4 写，依据写在每一格的标签里。a86c5c8fb8ec28789 两轮判决都没点到，期望照第三轮攻方的判：自己的报告行，拒是对的。
- 目录不在时这一格返回「本次未跑：…」，不加任何一格。成功那句改报「本次未跑 N 格：…」，并逐行打 `!`；GATE_NOT_RUN_FILE 在时各追加一行。

读数：派发闸在主工作区 `--check-prompt` 逐份判，改前（`before/`）与改后原样（`probes/real_prompts.py`）：

```
a0a2dfee322e26786.md（experiment-designer）：改前 ['报告文件名']  改后 ['报告文件名']
a28fe10b58d88beec.md（kb-scribe）：改前 ['报告文件名']  改后 ['报告文件名']
a303d33b0ccd5e68f.md（tooling-writer）：改前 ['放行']  改后 ['放行']
a3652270eb80e4fd0-delivery-colon.md（three-way-attack）：改前 ['放行']  改后 ['放行']
a3652270eb80e4fd0-section-colon.md（three-way-attack）：改前 ['放行']  改后 ['放行']
a3652270eb80e4fd0.md（three-way-attack）：改前 ['放行']  改后 ['放行']
a4737c6f3660f4f7e-own-result.md（experiment-runner）：改前 ['放行']  改后 ['放行']
a4737c6f3660f4f7e.md（experiment-runner）：改前 ['报告文件名']  改后 ['报告文件名']
a789e7091d91430aa.md（experiment-designer）：改前 ['报告文件名']  改后 ['报告文件名']
a86c5c8fb8ec28789.md（implementation-writer）：改前 ['报告文件名']  改后 ['报告文件名']
a8afb2185cfa840cc.md（experiment-designer）：改前 ['报告文件名']  改后 ['报告文件名']
a9717cf07fb6e7c21.md（tooling-writer）：改前 ['报告文件名']  改后 ['放行']
aa9ec3423c1f54bba-own-result.md（implementation-writer）：改前 ['放行']  改后 ['放行']
aa9ec3423c1f54bba.md（implementation-writer）：改前 ['报告文件名']  改后 ['报告文件名']
```

只有 a9717cf07fb6e7c21 一份变了，从拒变成放行。它的「草稿目录」只写在句中，没有行首那一行。判决 K2-4 写明「⑰ 放宽是为第三轮攻方那份不照骨架写的工具员提示」，收成骨架那一行之后就不再罩它，自证里这一格的期望照此写成放行。

自证：
- 派发闸新加 15 格：真提示 14 格，加「真提示:目录不在时报本次未跑、一格都不判绿」1 格（拿一个不在的目录调一次）。
- 成功那句末尾现在是「⑳ 第三轮攻方模型里的真提示 14 份逐份对上九十节 ⑳ 与两轮判决的期望」。

弄坏开关：
- `RUNNER_DISPATCH_GUARD_BREAK=real-prompts-absent-green`：目录不在时也当跑过，每份记一格绿。红 1 格「真提示:目录不在时报本次未跑、一格都不判绿 应当返回 (True, 0)，实际 (False, 14)」。
- 真提示那几格也跟着已有的开关红，已经写进文件头：report-name-unchecked 下 a28fe10b58d88beec 等 7 份；report-line-position-anchored 下 delivery-colon 等 7 份；LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere 下 a9717cf07fb6e7c21。

## 五、各份自证总数（主工作区，换上之后；改前的数在 baseline/ 下）

| 起法 | 改前 | 改后 | 原样末行在 |
|---|---|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 483 种，退 0 | 507 种，退 0，⑳ 14 份都跑了，没有本次未跑 | final/dispatch-selftest.out |
| `bash .claude/hooks/handback-guard.sh --selftest` | 49 种，退 0 | 62 种，退 0 | final/handback-selftest.out |
| `bash .claude/hooks/completion-cleanup-guard.sh --selftest` | 判了 34 格、25 个开关，退 0 | 判了 36 格、25 个开关各红在点名的格，退 0 | after/completion-selftest-1.out |
| `python3 .claude/hooks/lib_prompt_scratch_line.py --selftest` | 新建 | 18 格，退 0 | 本节末尾 |
| `python3 scripts/agents/dispatch-prompt.py --selftest`（它 import 了 lib_three_way_legs） | — | 92 项，退 0 | final/dispatch-prompt-selftest.out |
| `python3 scripts/agents/agent-cost.py --selftest`（同上） | — | 23 项，退 0 | final/agent-cost-selftest.out |

新库末行原样：`  ✓ lib_prompt_scratch_line 自检通过（查了 18 格）：只认行首「草稿目录：」一行（全角半角冒号、反引号、末尾标点都照认），转述里的、列表与粗体起头的、「草稿目录在」、占位、闭合了的围栏代码块里的都不认；展开 ~ 之后只留工作根与过渡期旧根底下的，根本身不算`

语法检查：hook-swap 每次换上之前、换上之后都跑了 `bash -n`，内嵌的 python 也编译过。新库另跑了 `python3 -m py_compile`，生成的 __pycache__ 已删。

## 六、弄坏开关复跑（照门禁 code-tooling 的 break-switch-replay 格的比法，自己核过）

脚本：
- `probes/switch_wheel.sh` 每个开关单独设上，跑一次 --selftest，每件的退出码各写一份文件；派几件收几件，对不上就整批作废。
- `probes/check_named.py` 用那一道里的 BREAK_SWITCH_TABLE_LIBRARY 读文件头，用同一套 quoted_items、appears 核「点名的项都在失败行里」。

结果：
- 派发闸：跑了文件头里和这一批相关的 30 条（一共 101 条），30 条都退 1，点名的都在失败行里；另把新库的 code-blocks-read、roots-unfiltered 两条也设上跑了一次。输出在 `wheel-dispatch-a`、`-b`、`-c`、`-d`。另外 71 条点名的格都不在这一批的改动里，没跑。
- 交回闸：文件头 23 条全跑，22 条加上 LIB 那一条都「对」。输出在 `wheel-handback`、`-e`、`-lib`。
- 新库：3 条都「对」，输出在 `wheel-lib`。
- 完工清理闸：自证自带开关轮，25 条各红在点名的格。
- 文件头开关表三样写全（break-switch-headers 那一套读法）：新库 3 条、派发闸 101 条、交回闸 23 条、完工清理闸 25 条，「不全」都是 0 条。
- 平常跑设了开关要被拒（break-switch-outside-selftest 格的探针写法）：`LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere python3 -B .claude/hooks/lib_prompt_scratch_line.py --force --break-guard-probe-unknown-option` 退 2，原样：
  ```
  ✗ lib_prompt_scratch_line.py：LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒
  → 怎么办：unset LIB_PROMPT_SCRATCH_LINE_BREAK 再来；要看开关红在哪一格，跑 python3 /home/fy5090/code/singlefs/.claude/hooks/lib_prompt_scratch_line.py --selftest
  ```
  三份钩子本来就登记在 `.claude/gate.d/break-switch-outside-selftest-pending.tsv` 里，这一批没改它们的守卫。

这一批新加、改动的开关：
- 新加：派发闸 leg-report-placeholder-kept、real-prompts-absent-green；交回闸 naming-position-unchecked、naming-exit-code-unchecked、leg-report-subdirectory-kept；新库 label-anywhere、code-blocks-read、roots-unfiltered。
- 派发闸和交回闸的文件头另加了一条 LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere。
- 派发闸 report-name-unchecked、report-line-position-anchored、writer-lock-root-locked 三条的点名改了：原来点名的那一格换了标签和期望，另补上真提示与替身那几格。
- 交回闸 SINGLEFS_WORK_ROOT_BREAK=new-root-unknown 的点名补了「写回员自己起草的 spec.json 目标不在拒」。现在草稿目录也只留工作根下的，这个开关下这一格会红。

## 七、要主 agent 跑的命令与要接的登记

工作目录都是主工作区 `/home/fy5090/code/singlefs`。都不编译，不用内存包装，不起重活。

1. 门禁表里我这一次不能改的两处（`.claude/gate.d/` 不在这一批的写范围里），请派工具员加，或主 agent 自己加：
   - `.claude/gate.d/code-tooling.sh` 的 `SELFTEST_RUNNERS` 加一条 `"python3 .claude/hooks/lib_prompt_scratch_line.py --selftest"`（照 `lib_workspace_cargo_lock.py` 那一条的写法）。不加的话，新库自己那 18 格只有我这一次跑过，门禁不跑；三份钩子的自证会经各自的替身格核到「调的是它」，但核不到它自己的红绿格。
   - `.claude/gate.d/break-switch-replay-files.tsv` 加一行 `.claude/hooks/lib_prompt_scratch_line.py	60	# 每条开关逐个跑过：点名的失败行原文都在失败行里，且自证跑得快（单跑一条不到 1 秒）`。
2. 复跑方那一格，包括这一批新加、改了点名的开关：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay、break-switch-headers、break-switch-outside-selftest 三格; 范围=3 格; 为什么=第十批改了派发闸、交回闸、完工清理闸的文件头开关表，新建 lib_prompt_scratch_line.py' bash .claude/gate.d/code-tooling.sh --check break-switch-replay,break-switch-headers,break-switch-outside-selftest /home/fy5090/code/singlefs`
   - 判法：末行是 ✓。break-switch-replay 那一格不该有派发闸、交回闸、完工清理闸的「不符」「没红」行；第 1 条接上之前，新库会记「还没接复跑」那一行，是本次未跑。
3. 钩子注册和自证（上游 hooks-registered 那一道，提交前那一趟里会跑）：照 `.claude/skills/gate/staged-gate.md` 的整轮来，不单列。

## 八、没做什么、认不出的

- 没碰（照派发提示）：
  - `.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/bash-command-detector.sh`。
  - completion-cleanup-guard.sh 里认草稿目录之外的部分。
  - `.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/`、`scripts/`、`.claude/gate.d/`、`records/`、`research/`。
- 「草稿目录：」还有另外三种认法，不在这一批的四处里，都在 `scripts/` 下，派发提示写明不碰。要收成一份的话，记进下一批：
  - `scripts/agents/dispatch-prompt.py:228` 的 `DRAFT_LINE`：只认第 0 列、全角冒号。
  - `scripts/agents/agent-cost.py:763`：一行里哪都认，冒号可有可无。
  - `scripts/agents/agent-watch.py` 的 `progress_file_of`：认的是提示里点名的工作根下的路径。
- K3-1 还有认不出的写法（交回闸文件头已经写了）：
  - 只往 `bash -c` 里收一层。套在 capped.sh、two-logs.sh 这类包装里的 `bash -c` 那一段不再分位置。
  - `( cd X && 两条 )` 这种写在子 shell 里、前面接着 && 的，一律不收，是严的那一边。
  - 写在 `$(…)` 里的不收，哪怕它其实跑完了。
  - set -e 之下前面的命令失败、后面的不跑、整条退 1 的，只在整条里还有别的命令时认得出。
- ⑯ 收严之后，行首带列表记号、引用记号或粗体的「草稿目录：」不再建写者锁。照骨架派的不受影响，手写成这种样子的会少一把锁。这是推出来的，没量过真提示里有几份这样写；要量，就拿主会话记录里全部派发提示的首条，数这几种起头的「草稿目录：」行。
- a9717cf07fb6e7c21 那一类提示（草稿目录只写在句中、报告路径带目录）现在 ⑰ 不再拦，这是判决 K2-4 定的收严代价；真提示那一格照这一条写了期望。
- 交回闸那一侧的「派前、交回时一致」只核了攻方的两句和一句对照。没扫真会话记录里三方腿交回正文的写法：子目录或占位写法真出现过几次，不知道。
- 第六节的复跑只跑了派发闸里和这一批相关的 30 条；另外 71 条照第七节第 2 条交主 agent。
- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没改 `.rs`，所以没跑交回前的两条命名检查（共用约束只要求改了 `.rs` 的跑）。
- 草稿目录清理：三棵钩子树（before、after、final 各 9.6M）和喂法建的 `feed-mine/`（8.0K，里面只有一份写者锁）都删了，删之前 `du -sh` 记了这几个大小。剩下的 2.8M 是原文备份、探针、规格、喂法、开关轮输出和 hook-swap 日志，都是文本，没有仓副本、编译目录或库拷贝。
- 攻方 naming_probe.py 在 before、after、final 三处各留了假仓 `fake-repo/`（48K）、假工作树 `fake-worktree/`（20K）和假会话记录 `fake-session/`，交回时完工清理闸拦下了，已全部删掉；大小是删之前 `du -sh` 记的。

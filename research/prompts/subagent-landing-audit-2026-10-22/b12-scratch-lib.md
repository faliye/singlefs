# 第十二批任务 40：草稿目录共用库与账表（拆分提案第九十节 ㉚、㉝、㉟）

日期：2026-10-23。草稿目录：`~/.local/share/singlefs/work/landing-audit-b12-scratch-lib`。

## 结论

六条都做到。前三条是改法，后三条是实测的数。

1. ㉝(a) 围栏照 CommonMark 认：库 `lib_prompt_scratch_line.py` 改了、经 `hook-swap.py` 换上。样本取 `landing-audit-r5-opus-output.md` K2-1b 的写法（语言名里带反引号的起头行）；没收尾的围栏改成一直到文末（原来「照认」，这一条与原判法相反，见「没做什么」里的取舍）。
2. ㉝(b) 相对路径按仓根解：改了。真派发提示里「草稿目录：」写相对路径的 **0 份**（4006 份提示）。
3. ㉚ `agent-cost.py` 的草稿目录认法改调共用库：改了。3394 个子 agent 里认出的草稿目录变了 258 个（163 个改前认不出、改后认出；95 个改前认出、改后认不出），结局桶只因此换了 1 个（`acff57c83a78c1a91`，`stopped-salvaged` → `stopped-abandoned`）。
4. ㉚ ⑯ 收严的影响：4006 份派发提示里行首带列表记号的「草稿目录：」有 **84 份**（引用记号 0、粗体 0），其中 76 份起来了；84 份都没有行首那一行，⑯ 都不建锁；日期最晚 2026-10-12，2026-10-14 起 0 份。
5. ㉟ 账表三方腿两桶读登记：`verdict-archived` 699 → 647，`three-way-report-or-verdict-missing` 133 → 4，换桶 181 个，全是定义里没登记 `leg: three-way` 的 `three-way-local-attack`（65）、`three-way-materials`（68）、`three-way-verifier`（48）。
6. ㉟ 报告名前缀实测：九十节 ㉑ 写的「不分大小写」对 Write 工具成立（混合大小写、前缀后紧跟字母、`.MD`、别的目录都拒），但有两处没覆盖：`.markdown` 不拒；Edit 新建与 Bash 写都不拦。另：任务提示把这条拒绝记在 `write-guard.sh` 上不对，`write-guard.sh` 拷一份喂 22 条输入全放；这条拒绝在工具层（Write）和派发闸 ⑰ 上，⑰ 对 `.MD` 放行（工具层拒）。

## 改过的文件

- `.claude/hooks/lib_prompt_scratch_line.py`（经 `scripts/gate/hook-swap.py` 换上两次：第一次 12 处，第二次 2 处补开关表里的点名行；换上之后在主工作区再验过）
- `scripts/agents/agent-cost.py`（`scripts/kb/replace-batch.py` 一次 10 处）

写之前的基线：库 sha256 `2e212a85125a5d8c8fee55685b845acb63c1b4553ab1847e5a07fdd662862525`；`agent-cost.py` 在我开工后被别的会话改过两行（只改了 `records_between` 的一句文档串与一条 `clock-times:allow` 注释，不影响行为），我是在新内容上批量替换的。

现在的 sha256：

```
5b4cbc18f647c28cad29a188a0a6a572867a418e253e6ad3023361968822a6f0  .claude/hooks/lib_prompt_scratch_line.py
d93e280f87213b92734e2067ed0437a08f11648a1e23a3ca4f434cf8ef499d26  scripts/agents/agent-cost.py
```

`git diff --stat -- .claude/hooks/lib_prompt_scratch_line.py scripts/agents/agent-cost.py` 原样（对暂存区，库的暂存版本比我开工时的工作区版本还旧一档，所以数里含别的会话那一档的 2 行文档改动）：

```
 .claude/hooks/lib_prompt_scratch_line.py | 107 ++++++++++++++++++++++++++-----
 scripts/agents/agent-cost.py             |  54 ++++++++++++++--
 2 files changed, 137 insertions(+), 24 deletions(-)
```

库的函数名与签名没变（`scratch_line_values(prompt)`、`scratch_directories(prompt, include_roots=False)`、`fenced_line_numbers(lines)`）；新加 `repository_root()`。

## 一、㉝(a) 围栏照 CommonMark

改法（`fenced_line_numbers`，文件头「认法」一段同步改了）：

- 起头一行前面至多三个空格（四个以上是缩进代码块，不算围栏）；``` 或 ~~~ 至少三个；反引号围栏的语言名位置有反引号的不开块（那是行内代码）。
- 收尾一行前面至多三个空格，同一种字符、不比起头短、后面只有空白（后面接字的不收尾）。
- 没收尾的围栏一直到文末，里面的行不认。
- 文件头写明：列表项与引用块里的围栏不单独认；派发闸 ⑭⑮ 的 `without_code_blocks` 只遮闭合了的围栏，没闭合的这里遮到文末，两处在没闭合上不同。

样本，红绿各自实测（`probes/samples.py` 的输出在 `out/samples.txt`；下表是 `scratch_line_values` 取到的值，`W` 是工作根，`m` 是本提示要认的目录，`o` 是围栏里不该认的）：

| 样本（提示正文） | 现在 | 开关 `fence-info-with-backtick` | 开关 `unclosed-fence-read` |
|---|---|---|---|
| ``` a`b ＋ 草稿目录：m ＋ ```（r5 K2-1b 的写法） | W/m | 空（红） | W/m |
| ``` ＋ 草稿目录：m（没收尾） | 空 | 空 | W/m（红） |
| ``` ＋ 草稿目录：o ＋ ``` 说明 ＋ 草稿目录：o | 空 | 空 | W/o（红） |
| ```` ＋ 草稿目录：o ＋ ``` ＋ 草稿目录：o（收尾比起头短） | 空 | 空 | W/o（红） |
| ~~~ ＋ o ＋ ~~~ ＋ 草稿目录：m | W/m | W/m | W/m |
| 缩进 4 格的 ``` 包着 草稿目录：m | W/m | W/m | W/m |
| 缩进 3 格的 ``` 包着 o，其后 草稿目录：m | W/m | W/m | W/m |

（表里「现在」一列是绿；两个开关列里标红的是被弄坏之后与「现在」不同的格。）

自证：`python3 .claude/hooks/lib_prompt_scratch_line.py --selftest` 现在 39 格全绿；新加 10 格围栏样本（上表各行，另有「没收尾的围栏之前的那一行照认」「收尾比起头长照收尾」「波浪号围栏收尾要同一种字符」），外加一格与 markdown-it-py 的 commonmark 预设逐样本比围栏行（本机 3.0.0 装了；没装时这格不查，成功句里写明）。

弄坏开关（文件头开关表已写三样）：

- `LIB_PROMPT_SCRATCH_LINE_BREAK=fence-info-with-backtick`：红在「✗ 自检：反引号围栏的起头行在语言名的位置有反引号，那是行内代码，不开块」与「✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（反引号围栏……）」两格。
- `LIB_PROMPT_SCRATCH_LINE_BREAK=unclosed-fence-read`：红在「✗ 自检：没收尾的围栏一直到文末，里面的行不认」「✗ 自检：收尾行后面接了字不算收尾，围栏一直到文末」「✗ 自检：收尾比起头短不算收尾……」「✗ 自检：缩进 4 格的收尾不算收尾……」等 5 格，另有 markdown-it-py 对照的 6 格。
- 原有三项（`label-anywhere`、`code-blocks-read`、`roots-unfiltered`）各自仍红在原来点名的格；逐项跑了，输出见本报告「自证原样」一节。

真提示上的影响：4006 份提示逐份比改前改后的 `scratch_line_values` 与 `scratch_directories`，**0 份不同**（`probes/diff_real.py`）；其中含围栏起头行的提示 61 份，起头行语言名里带反引号的 0 份；在含「草稿目录」的那些行上，新库认的围栏行与 markdown-it-py 认的不同的提示 0 份。就是说这一改在现有的真提示上不改任何一个结果。

## 二、㉝(b) 相对路径按仓根解

先数（改前）：`probes/extract_prompts.py` 取主会话记录里全部 Agent / Task 调用的提示（4006 份，其中 3394 份有子 agent 记录，两处的正文逐字相同）。

- 行首「草稿目录：」一行认出路径的提示 2354 份；这些值里展开 `~`、`$HOME` 之后不是绝对路径的：**0 份**。
- 放宽到一行里哪都认（`草稿目录[：:]` 后面接任何路径写法），不以 `/`、`~`、`$` 起头的有 3 份，都是正文里说「草稿目录：不碰」「草稿目录：用……」的话，不是路径行。

所以真派发提示里写相对路径的是 0 份；改的是堵住「手写提示写了相对路径、结果按钩子当时的目录解」这条路。

改法：`scratch_directories` 里展开之后还不是绝对路径的，拼在 `repository_root()`（库文件往上三层，即仓根）后面，不再交给 `os.path.abspath` 按当前目录解。完工清理闸 `completion-cleanup-guard.sh` 自己取 `scratch_line_values` 之后另有一处 `os.path.abspath`（该文件 337 行），那一处不在这一批的文件里，没动。

样本红绿（自证两格，另在 `out/samples.txt` 里原样）：

| 样本 | 现在 | 开关 `relative-by-cwd` |
|---|---|---|
| 当前目录 `/`，写 `草稿目录：<从仓根到 m 的相对路径>` | W/m | 空（红） |
| 当前目录是工作根，写 `草稿目录：b12-relative-probe` | 空（解到仓根下，不在工作根底下） | W/b12-relative-probe（红） |

自证格：「相对路径按仓根解，不按调用时的当前目录（当前目录在仓外）」「……（当前目录在工作根）」；弄坏开关 `LIB_PROMPT_SCRATCH_LINE_BREAK=relative-by-cwd` 两格都红（输出见「自证原样」）。`roots-unfiltered` 打开时「当前目录在工作根」那格也红（因为仓根下的路径不再被根过滤挡掉），这是新增的次要红，不影响原来点名的两格。

## 三、㉚ `agent-cost.py` 的草稿目录认法改调共用库

改前的行为（读代码）：`re.search(r"草稿目录[：:]\s*(" + PATH_CHARS + "+)", prompt)`，一行里哪都认（句中转述、列表、闭合了的围栏代码块里的都认），取第一个；冒号**不是**可有可无（任务提示和 b10 记录里的「冒号可有可无」说的是第九批 ⑰ 那一份的认法，这一处冒号必须有）；路径字符集 `PATH_CHARS` 不含反引号，所以 `草稿目录：` 后面把路径包在反引号里的取不到。

改法：新函数 `draft_directory_of(prompt)`，调库的 `scratch_line_values`、取第一个；`scan_subagent` 里的 `draft` 改用它。弄坏开关 `AGENT_COST_BREAK=draft-anywhere-in-line` 退回改前的正则。文件头「口径」加了一行草稿目录的认法，开关表加了一项。

样本红绿（`selftest_draft_directory`，写成会话记录、经 `scan_subagent` 读，6 格）：

| 样本提示 | 现在取到 | 开关打开后 |
|---|---|---|
| 句中转述在前 `前一段的草稿目录：o（它写的探针）`，行首 `草稿目录：m` 在后 | m | o（红） |
| 只有 `- 草稿目录：o`（列表） | 无 | o（红） |
| 闭合了的围栏代码块里 `草稿目录：o`，块后行首 `草稿目录：m` | m | o（红） |
| 骨架那一行、半角冒号带括注、没有那一行 | m / m / 无 | 同 |

自证：`python3 scripts/agents/agent-cost.py --selftest` 现在 29 项全绿（比原来多 `selftest_draft_directory` 的 6 项，成功句里加了「草稿目录只认派发提示行首」）；`AGENT_COST_BREAK=draft-anywhere-in-line` 红 3 行，点名「✗ 自检：草稿目录认法：句中转述在前、行首那一行在后……」「……列表里转述的不认」「……闭合了的围栏代码块里的不认」。其余 11 个旧开关逐个复跑，都还红（红的行数 1、1、1、1、1、4、2、2、1、2、1）。

### 真会话记录上改前改后认出的草稿目录

范围：3394 个有子 agent 记录的派发（`probes/draft_changes.py`，改前用原正则 + `normalize_path`，改后用 `draft_directory_of` + `normalize_path`）。

- 改前认出 1993 个，改后认出 2061 个，**不同 258 个**：
  - 163 个改前认不出、改后认出：`草稿目录：` 后面把路径包在反引号里（如 ``草稿目录：`/tmp/claude-1000/m3-formal-13-3/` ``）。
  - 95 个改前认出、改后认不出：91 个是同一行前面还有别的话（`报告：…；草稿目录：…`、`……。草稿目录：…`），4 个是 `- ` 列表起头。
- 两边都认出而值不同的：0 个。
- 逐份清单见本报告「附：草稿目录认法变了的 258 个子 agent」。

对结局桶的影响（`agent-cost.py outcome --since 2026-09-01` 在改前的脚本拷贝和现在的脚本上各跑一遍，隔几分钟，按 agent 号对）：3398 个共有，桶不同 2 个：

- `acff57c83a78c1a91`（tooling-writer，2026-10-13，「doc-text 三语格与样本」）：`stopped-salvaged` → `stopped-abandoned`。它的提示是 `报告：…；草稿目录：…` 一行里写的，改后认不出草稿目录，按目录名点到它的引用就没了。真换桶，只有这一个。
- `a5453c31dba596575`（tooling-writer，2026-10-23，还在跑的）：`in-flight` → `written-landed`，是它还在跑、自己往前走了，不是认法造成的（它的提示是 `草稿目录：<绝对路径>` 一行，改前改后取到同一个值，不在那 258 个里）。

读 3394 个会话记录跑一遍 `outcome --jobs 31` 约 60 秒。

## 四、㉚ ⑯ 收严的影响：行首带记号的「草稿目录：」

范围：主会话记录里全部 Agent / Task 调用的提示 4006 份（含没起来、被派发闸拒了的），其中起来的 3394 份在 `~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/` 下有记录，提示逐字相同，所以不重复数。判法（`probes/marked_lines.py`）：不在闭合围栏块里的行，行首（前面只许空白）是列表记号（`- `、`* `、`+ `、`• `、`1. `、`1) `）、引用记号（`>`）或粗体（`**`、`__`）、再接「草稿目录」与冒号。

| 项 | 数 |
|---|---|
| 带这几种记号的「草稿目录：」行的提示 | **84** 份 |
| 其中起来了（有 agent 号） | 76 份 |
| 没起来 | 8 份 |
| 按记号：列表记号 / 引用记号 / 粗体 | 84 / 0 / 0 |
| 这 84 份里同时有行首（不带记号）那一行的 | 0 份，即 ⑯ 收严后都不建写者锁 |
| 日期最晚 | 2026-10-12；2026-10-14 起 0 份 |

按类型：experiment-designer 19、implementation-writer 15、sweep 11、three-way-attack 10、tooling-writer 8、three-way-local-attack 7、clerk 5、three-way-defense 5、three-way-forward 2、three-way-verifier 2。按日期：2026-09-22 到 2026-09-30 共 44 份，2026-10-01 到 2026-10-12 共 40 份（2026-10-06 一天 23 份）。

起来的 76 份与 landing-audit-r5 辩方量到的「⑯ 少建的锁 76 份」数字相同。逐份清单见本报告「附：行首带列表记号的草稿目录：84 份」。

结论：⑯ 收严的影响在现有记录上落在 2026-10-14 之前的 84 份、都是列表记号写法；写者锁上线之后的提示里一份都没有。手写提示写成列表样式的会少一把锁，这个风险目前只在理论上存在。

## 五、㉟ 账表三方腿两桶：改读 `lib_three_way_legs.py` 的登记之前与之后

做法：`agent-cost.py outcome --since 2026-09-01 --jobs 31` 在同一份会话记录上跑两遍，隔几分钟、用同一份脚本，只差 `AGENT_COST_BREAK=leg-types-by-prefix`（改前：类型名以 `three-way` 开头的都算腿）与不设（现在：读定义 frontmatter 写了 `leg: three-way` 的，本仓是 three-way-attack、three-way-defense、three-way-forward 三种）。按 agent 号对，共有 3398 个。

| 桶 | 改前（按名字） | 改后（读登记） | 差 |
|---|---|---|---|
| `verdict-archived` | 699 | 647 | -52 |
| `three-way-report-or-verdict-missing` | 133 | 4 | -129 |
| 换桶的个数 | | | 181（另有 1 个 tooling-writer 是还在跑的、两遍之间自己前进的，不算） |

换出两桶的 181 个，类型分布：

| 类型 | 换桶数 | 改前 | 改后落在 |
|---|---|---|---|
| three-way-local-attack | 65 | missing 60、verdict-archived 5 | written-landed 56、report-used-by-main 7、report-unreferenced 1、written-overwritten 1 |
| three-way-materials | 68 | missing 66、verdict-archived 2 | written-landed 35、report-used-by-main 30、written-overwritten 1、report-unreferenced 1、written-partial 1 |
| three-way-verifier | 48 | missing 3、verdict-archived 45 | written-landed 46、written-overwritten 1、report-used-by-later-sub 1 |

登记的三种腿一份没换：three-way-attack 331+2、three-way-defense 85+1、three-way-forward 231+1（verdict-archived + missing），两遍相同。改后两桶里剩下的 4 个 missing 就是这三种里的 4 个。

读法：改前 133 个 missing 里有 129 个本来就不是三方腿（`three-way-local-attack` 本地攻方腿、`three-way-materials` 材料员、`three-way-verifier` 核查员），按引用与落地率分桶才合适；改后这些进了 `written-*` 与 `report-*` 桶。逐份清单见本报告「附：三方腿两桶换桶的 181 个」。

## 六、㉟ 报告名三个前缀的拒绝实测

先纠正出处：`write-guard.sh` 里没有这条判法（拷一份喂 22 条输入全放，见下）。「Subagents should return findings as text, not write report files」是 Claude Code 工具层对子 agent 的 Write 工具的拒绝（`b9-prefix-probe.md` 同样记的是工具层）；钩子里对应的是派发闸 `runner-dispatch-guard.sh` 的 ⑰，判提示里「报告：」一行点名的文件名。所以下面三张表分开：工具层真调（我是子 agent，直接调工具）、写闸拷贝喂输入、派发闸拷贝喂输入。

### 6.1 工具层真调（草稿目录 `prefix-probe/`，探测文件内容一行「探测」）

| 名字 | 调用 | 结果 |
|---|---|---|
| `Summary.md`（混合大小写） | Write | 拒（逐字：`Subagents should return findings as text, not write report files. Include this content in your final response instead.`） |
| `summaryx.md`（前缀后紧跟字母） | Write | 拒 |
| `SUMMARY.MD`（`.MD`） | Write | 拒 |
| `summary.markdown` | Write | 放（写成） |
| `aNaLySiS.md`（混写） | Write | 拒 |
| `Report.md`（阳性对照） | Write | 拒 |
| `probe.md`（阴性对照） | Write | 放 |
| 另一目录 `/tmp/claude-1000/…/scratchpad/Summary.md` | Write | 拒 |
| 嵌套目录 `prefix-probe/sub/dir/Findings.md` | Write | 拒（目录没建出来） |
| `Summary-edit.md` | Edit（`old_string` 为空，新建） | **放**（写成） |
| `probe-edit.md`（阴性对照） | Edit（同上） | 放 |
| `Summary-bash.md` | Bash `echo 探测 > …`（noclobber） | **放**（写成） |
| `probe-bash.md`（阴性对照） | Bash | 放 |

阳性（`Report.md`）、阴性（`probe.md`、`probe-edit.md`、`probe-bash.md`）对照都对，这一轮有效。

### 6.2 写闸 `write-guard.sh` 拷贝（`hooktree/`，`CLAUDE_PROJECT_DIR` 指拷贝，输入 `agent_type=tooling-writer`、目标在草稿目录里；`out/feed-prefix-write-guard.tsv`）

11 个名字（Summary.md、summaryx.md、SUMMARY.MD、summary.markdown、Findings.md、aNaLySiS.md、Report.md、probe.md、x-summary.md、summary.txt、result.md）× Write、Edit 两种 = 22 条：**22 条全放（退 0）**，包括 `Report.md` 阳性对照。写闸没有这条判法。（第一遍带了 `agent_id`，22 条全拒，是因为草稿目录有我的写者锁、喂的假 agent_id 不是持锁者，与报告名无关，去掉 `agent_id` 重喂得上面的结果。）

### 6.3 派发闸 `runner-dispatch-guard.sh` 拷贝的 ⑰（拿本任务自己的派发提示做底、只换「草稿目录：」「认领：」「报告：」三行，`out/feed-prefix.tsv`、`out/feed-prefix2.tsv`）

| 报告行点名的文件 | ⑰ |
|---|---|
| 草稿目录下 `Summary.md` | 拒 |
| 草稿目录下 `summaryx.md` | 拒 |
| 草稿目录下 `SUMMARY.MD` | **放** |
| 草稿目录下 `summary.markdown` | 放 |
| 草稿目录下 `Findings.md`、`aNaLySiS.md`、`Report.md` | 拒 |
| 草稿目录下 `probe.md`、`x-summary.md`、`summary.txt`、`result.md` | 放 |
| 草稿目录的子目录下 `Summary.md` | 拒 |
| 别的工作根目录下 `SUMMARY.md` | 放（按设计：转述别的目录不判） |
| 仓外 `/tmp/claude-1000/…/Summary.md` | 放（同上） |

### 6.4 九十节 ㉑ 写的「不分大小写」对不对

- 对 Write：对。混合大小写（`Summary.md`、`aNaLySiS.md`）、全大写扩展名（`.MD`）、前缀后紧跟字母、别的目录与嵌套目录都拒，`.markdown` 不拒。`b9-prefix-probe.md` 的推翻条件（混写或别的目录有一例写成）这一轮实测没有出现。
- 补两条覆盖不到的：Edit 新建与 Bash 写不被拦（`agent-common.md` 那一句写的是「用 Write 建」，与此一致）；`.markdown` 不拦。
- 一处钩子与工具层不一致：⑰ 的 `REPORT_REFUSED_PREFIX_NAME` 只认小写 `.md`（`(?i:summary|findings|analysis)[^/]*\.md`），`SUMMARY.MD` 工具层拒、⑰ 放。是不是要改 ⑰（任务 39 的文件），交主 agent 判；这里只交数。

## 自证原样（主工作区现在这一份；红的运行里省去了每次都相同的「→ 看…」「→ 怎么办…」提示行）

```
$ python3 .claude/hooks/lib_prompt_scratch_line.py --selftest
  ✓ lib_prompt_scratch_line 自检通过（查了 39 格）：只认行首「草稿目录：」一行（全角半角冒号、反引号、末尾标点都照认），转述里的、列表与粗体起头的、「草稿目录在」、占位、围栏代码块（照 CommonMark，没收尾的到文末）里的都不认；展开 ~ 之后写相对路径的按仓根解，只留工作根与过渡期旧根底下的，根本身不算
rc=0

$ LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere python3 .claude/hooks/lib_prompt_scratch_line.py --selftest
  ✗ 自检：转述里「，草稿目录 <别处>」不认 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine', '/home/fy5090/.local/share/singlefs/work/b10-selftest-previous/']
  ✗ 自检：列表里「- 草稿目录 <别处>」不认 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine', '/home/fy5090/.local/share/singlefs/work/b10-selftest-previous/']
  ✗ 自检：列表与粗体起头的那一行不认 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：「草稿目录在 <路径>」不是骨架那一行，不认 应当返回 []，实际 ['在']
rc=1

$ LIB_PROMPT_SCRATCH_LINE_BREAK=code-blocks-read python3 .claude/hooks/lib_prompt_scratch_line.py --selftest
  ✗ 自检：闭合了的围栏代码块里的那一行不认 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine', '/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：没收尾的围栏一直到文末，里面的行不认 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']
  ✗ 自检：没收尾的围栏之前的那一行照认 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine', '/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：收尾行后面接了字不算收尾，围栏一直到文末 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：收尾比起头短不算收尾，围栏一直到文末 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：收尾比起头长照收尾 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous', '/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']
  ✗ 自检：波浪号围栏照认、收尾要同一种字符 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous', '/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']
  ✗ 自检：缩进 3 格的围栏照认 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous', '/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']
  ✗ 自检：缩进 4 格的收尾不算收尾，围栏一直到文末 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
rc=1

$ LIB_PROMPT_SCRATCH_LINE_BREAK=roots-unfiltered python3 .claude/hooks/lib_prompt_scratch_line.py --selftest
  ✗ 自检：相对路径按仓根解，不按调用时的当前目录（当前目录在工作根） 应当返回 ([], ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine'])，实际 (['/home/fy5090/code/singlefs/b12-relative-probe'], ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine'])
  ✗ 自检：工作根本身不算草稿目录 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work']
  ✗ 自检：工作根与旧根之外的目录不算 应当返回 []，实际 ['/var/b10-selftest-elsewhere']
rc=1

$ LIB_PROMPT_SCRATCH_LINE_BREAK=fence-info-with-backtick python3 .claude/hooks/lib_prompt_scratch_line.py --selftest
  ✗ 自检：反引号围栏的起头行在语言名的位置有反引号，那是行内代码，不开块 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 []
  ✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（反引号围栏的起头行在语言名的位置有反引号，那是行内代码，不开块） 应当返回 [2]，实际 [0, 1, 2]
rc=1

$ LIB_PROMPT_SCRATCH_LINE_BREAK=unclosed-fence-read python3 .claude/hooks/lib_prompt_scratch_line.py --selftest
  ✗ 自检：没收尾的围栏一直到文末，里面的行不认 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']
  ✗ 自检：没收尾的围栏之前的那一行照认 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine', '/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：收尾行后面接了字不算收尾，围栏一直到文末 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：收尾比起头短不算收尾，围栏一直到文末 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：缩进 4 格的收尾不算收尾，围栏一直到文末 应当返回 []，实际 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-previous']
  ✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（反引号围栏的起头行在语言名的位置有反引号，那是行内代码，不开块） 应当返回 [2]，实际 []
  ✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（没收尾的围栏一直到文末，里面的行不认） 应当返回 [0, 1]，实际 []
  ✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（没收尾的围栏之前的那一行照认） 应当返回 [1, 2]，实际 []
  ✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（收尾行后面接了字不算收尾，围栏一直到文末） 应当返回 [0, 1, 2, 3]，实际 []
  ✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（收尾比起头短不算收尾，围栏一直到文末） 应当返回 [0, 1, 2, 3]，实际 []
  ✗ 自检：围栏行与 markdown-it-py 的 commonmark 预设认的一致（缩进 4 格的收尾不算收尾，围栏一直到文末） 应当返回 [0, 1, 2, 3]，实际 []
rc=1

$ LIB_PROMPT_SCRATCH_LINE_BREAK=relative-by-cwd python3 .claude/hooks/lib_prompt_scratch_line.py --selftest
  ✗ 自检：相对路径按仓根解，不按调用时的当前目录（当前目录在仓外） 应当返回 ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine']，实际 []
  ✗ 自检：相对路径按仓根解，不按调用时的当前目录（当前目录在工作根） 应当返回 ([], ['/home/fy5090/.local/share/singlefs/work/b10-selftest-mine'])，实际 (['/home/fy5090/.local/share/singlefs/work/b12-relative-probe'], [])
rc=1

$ python3 scripts/agents/agent-cost.py --selftest
  ✓ 自检通过（查了 29 项，其中 outcome 的 21 个子 agent）：调用按消息 id 去重、读缓存与两档缓存写分开、等待按 3 分钟间隔算、派发只数真正起来的、派发闸的拒绝按钩子归类、每个结局桶都对得上、路径归一化、Bash 里的写入认得出、「修订」与 fixture 这类不算返工；三方腿报告在仓且同轮有判决才记 verdict-archived（判决只在 git 历史里也算），三方腿类型读定义 frontmatter 的 leg: three-way（假仓与本仓读到的都与 frontmatter 逐个相同）；交回按送达认（先被拒后送达的算、被拒之后再没送达的不算）、起点按首条记录、--until 含那一天、--since 与 --until 按东京日期切天；主会话按每条记录的时刻切天（跨两天的主会话只收窗口里那几条）；outcome --out 排他落账、回读与标准输出逐字相同、已在就拒；草稿目录只认派发提示行首「草稿目录：」那一行（认法同 lib_prompt_scratch_line.py）
rc=0

$ AGENT_COST_BREAK=draft-anywhere-in-line python3 scripts/agents/agent-cost.py --selftest
  ✗ 自检：草稿目录认法：句中转述在前、行首那一行在后，取行首那一行 应当取到 ~/.local/share/singlefs/work/draft-selftest-mine，实际 ~/.local/share/singlefs/work/draft-selftest-previous
  ✗ 自检：草稿目录认法：列表里转述的不认 应当取到 None，实际 ~/.local/share/singlefs/work/draft-selftest-previous
  ✗ 自检：草稿目录认法：闭合了的围栏代码块里的不认 应当取到 ~/.local/share/singlefs/work/draft-selftest-mine，实际 ~/.local/share/singlefs/work/draft-selftest-previous
rc=1
```

## 调这份库的其余自证（改后在主工作区跑，都登记为 light）

- `bash .claude/hooks/runner-dispatch-guard.sh --selftest`：退 0，「查了 525 种」。
- `bash .claude/hooks/handback-guard.sh --selftest`：退 0，「查了 78 种」。
- `bash .claude/hooks/completion-cleanup-guard.sh --selftest`：退 0，「各格都对（判了 43 格）；27 个弄坏开关各红在点名的格」。
- `python3 scripts/agents/agent-watch.py --selftest`：退 0。
- 语法：`python3 -m py_compile` 两份改过的 `.py` 过。
- 弄坏开关复跑表（门禁 break-switch-replay 格，两份都在 `.claude/gate.d/break-switch-replay-files.tsv` 里）我没跑格函数；按它的判法手验了新开关：点名的失败行原文（见上面「自证原样」）都在失败行里，单跑一条都不到 1 秒（`time` 实测库 0.03 秒、`agent-cost.py` 0.09 秒，在登记的上限 60 秒内）。

## 要主 agent 接的

- 没改 agent 定义、共用约束与规则，不需要走定义三方。
- 完工清理闸 `.claude/hooks/completion-cleanup-guard.sh` 337 行自己用 `os.path.abspath` 解草稿目录，相对路径仍按钩子当前目录解，要不要也改调 `scratch_directories`，交任务 39 或下一批判。
- `runner-dispatch-guard.sh` 文件头 122 行写「闭合了的围栏代码块（照 CommonMark，without_code_blocks）里的行不认」说的是 ⑰ 报告行，没有说「草稿目录：」那一行；那一行现在是「没收尾的遮到文末」，与 ⑭⑮ 只遮闭合了的在没收尾上不同。要不要让两处一致（两个方向都能改：库改成只遮闭合的，或闸改成没收尾的也遮），交主 agent 判；真提示里这两种差别 0 份撞上。
- ⑰ 对 `SUMMARY.MD` 放行而工具层拒（6.4），交任务 39 的文件判。

## 没做什么

- 没跑重型测试、`gate.sh`、各类 lint；没走定义三方；没提交；没改 agent 定义、共用约束与规则。
- 没改 `write-guard.sh`、`runner-dispatch-guard.sh`、`dispatch-prompt.py`、`lib_three_way_legs.py`、`handback-guard.sh`、`agent-watch.py`，也没动 `scripts/kb/` 下的脚本（只调用 `replace-batch.py`）。
- 没改完工清理闸（它自己的 `os.path.abspath` 见上）。
- 没加列表项与引用块里围栏的单独判法：库文件头写明不单独认；与 markdown-it-py 的对照格只含顶格样本。真提示里含「草稿目录」的行上两边 0 份不同，但那是 4006 份样本，不证明列表项里的围栏永远一致。
- 取舍：没收尾的围栏「照认」改成「遮到文末」是照任务写的「没闭合的围栏到文末」与 CommonMark 做的，方向上会让「草稿目录：」写在没收尾的围栏之后的提示认不出草稿目录（⑯ 不建锁、完工清理闸与交回闸不看那个目录）。真提示里 0 份受影响（逐份比改前改后 0 份不同）。推翻它的现象：手写提示在行首「草稿目录：」之前误带一个没收尾的 ``` 行，发现后果是锁没建——那时该退回「没收尾的照认」，把开关 `unclosed-fence-read` 的行为设为默认。
- `.claude/hooks/__pycache__/` 下我用 `py_compile` 生成了两个 pyc（git 忽略）；没单独清。

## 推翻条件

- 条 1：真提示出现「草稿目录：」行落在没收尾围栏之后而被漏认的，或与 markdown-it-py 在顶格围栏样本上不同的，这一改就错了一半。
- 条 3：258 个变了的里有哪个子 agent 的结局桶因此被判错（本次只 1 个换桶，`acff57c83a78c1a91`）。
- 条 5：登记表里漏了应当算腿的类型（本次登记三种：three-way-attack、three-way-defense、three-way-forward）。

## 清理

删之前 `du -sh`：`hooktree/`（拷来的 `.claude/hooks` 加指向主工作区的符号链接，写闸与派发闸喂输入用）2.9M；`oldtree/`（改前的 `agent-cost.py` 拷贝加符号链接，跑改前的账表用）140K；`feedprobe-*`（派发闸喂输入时建的写者锁小目录）各几 K。三样都已删。`hook-swap/`（84K，日志、喂法输出、反向规格）、`probes/`（26M，含 `prompts.jsonl` 4006 份提示的抽取）、`out/`（3.4M）、`prefix-probe/`（28K）都是文本，没有仓副本、编译目录或库拷贝，留作证据。

## 附：行首带列表记号的草稿目录：84 份

| 派发（agent 号，没起来的写 tool_use 号） | 类型 | 日期 | 行首记号 | 行首「草稿目录：」那一行 | 那一行的原文 |
|---|---|---|---|---|---|
| a3c652678dd1d3ffa | sweep | 2026-09-22 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/c468-evidence-judge-a/` |
| a4f529b8194efcbb1 | sweep | 2026-09-22 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/c468-evidence-sweep/`（事实表、候选表、`--changes` 的输出都落在这里，交回时把路径写全） |
| af391f13e8b3fa006 | sweep | 2026-09-22 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/c472-batch-scope-judge/` |
| ac0e27d7e2cff3dc5 | sweep | 2026-09-22 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/gate-reuse-judge/` |
| a02e1ab2e01fe8fd0 | sweep | 2026-09-22 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/c472-batch-scope-sweep/` |
| a80eee396c953468b | sweep | 2026-09-22 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/gate-reuse-sweep/` |
| a5cb05924ec3552d1 | sweep | 2026-09-22 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/c468-evidence-judge-b/` |
| a5e73a10496f5937e | three-way-attack | 2026-09-22 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/agentdef-r3-opus/`。 |
| aa579b08001b66d5f | three-way-defense | 2026-09-22 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/agentdef-r3-sonnet/`。 |
| a3d04b4971c5e459f | three-way-local-attack | 2026-09-22 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/agentdef-r3-local-attack/`。 |
| a983efeaf6b47abf0 | experiment-designer | 2026-09-24 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/prereg-e142-e49/`。 |
| a4fbb5800bbe251b8 | experiment-designer | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e142-r17-design/`。 |
| a134642d46795a1f9 | experiment-designer | 2026-09-25 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/e142-r18-design/`。 |
| a64cf78c30be6a8fb | experiment-designer | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e142-r16-design/`。 |
| a41a8a8402d5445aa | three-way-attack | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r2-opus/`。 |
| a27bae3b68ec10a12 | three-way-attack | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r3-opus/`。 |
| af0be68ea23633fb6 | three-way-defense | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r3-sonnet/`。 |
| a52b002e107267549 | three-way-defense | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r2-sonnet/`。 |
| a4138e191b0ee378e | three-way-local-attack | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/defs-memory-migration-r1-local-attack/` |
| a753d37a57a536efe | three-way-local-attack | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/defs-memory-migration-r2-local-attack/` |
| aa0c6e9e4ba2556d1 | three-way-local-attack | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r3-local-attack/`。 |
| a358d468ad00b5e4e | three-way-local-attack | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r2-local-attack/`。 |
| acecaade85592f5cc | three-way-verifier | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r3-verifier/`，交回前删掉里面的 `target/`。 |
| a22a88716afb29d88 | three-way-verifier | 2026-09-25 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m2-rollback-forward-r2-verifier/`，交回前删掉里面的 `target/`。 |
| adf2cb9c002807d89 | experiment-designer | 2026-09-26 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e158-r2-designer/`。 |
| a0bb7a0f5417414f7 | experiment-designer | 2026-09-26 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e156-r4-designer/`。 |
| ae090b327febf5cce | experiment-designer | 2026-09-26 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/e158-r4-designer/`，报告写到同目录的 `report.md`。 |
| a66613c02b8ef49d7 | experiment-designer | 2026-09-27 | 列表记号 | 没有 | - **草稿目录**：`/tmp/claude-1000/e142-r19-designer/`，报告写到同目录 `report.md`。 |
| a64c8602543cb495f | experiment-designer | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e-crash-store-designer/`。 |
| a96e85bb2439e35fa | experiment-designer | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e-gpu-multicard-designer/`。 |
| toolu_01CqrrHweTBYfG85BuYLodr8（没起来） | experiment-designer | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e-gpu-dedup-designer/`。 |
| a174d93c7351573c8 | experiment-designer | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e-gpu-dedup-designer/`。 |
| a9e3282b553c7be7a | three-way-attack | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/runner-compile-first-r1-opus2/`。 |
| a4a350563ef7cf6c1 | three-way-attack | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/runner-compile-first-r1-opus/`。 |
| a5f17cf0698ff4852 | three-way-forward | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/runner-compile-first-r1-sonnet/`。 |
| a983f41b9eade1699 | three-way-local-attack | 2026-09-27 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/runner-compile-first-r1-local/`。 |
| a2683c1559f8807c6 | three-way-attack | 2026-09-28 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/c245-r3-opus/`。 |
| a508ed30cd7badd74 | three-way-forward | 2026-09-28 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/c245-r3-sonnet/`。 |
| a5ef129fea46aa7c6 | three-way-local-attack | 2026-09-28 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/c245-r3-local/`。 |
| abcd968eff0790ead | clerk | 2026-09-30 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-formal-proto-kani/` |
| a7e9d12988341c8df | clerk | 2026-09-30 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-formal-verus-wire-in/` |
| afc70ad4f5ce8d081 | clerk | 2026-09-30 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-formal-proto-verus/` |
| afe2e493aad9c05be | clerk | 2026-09-30 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-formal-proto-z3/` |
| ad37c95205ca30c92 | clerk | 2026-09-30 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-formal-amplification-reuse-facts/` |
| ae2a2308bba07a113 | experiment-designer | 2026-10-01 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e165-r2-designer/`。 |
| ab806d25e29160783 | experiment-designer | 2026-10-01 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/o3-designer/`。 |
| a2ccbedb400bcc4fb | experiment-designer | 2026-10-01 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e165-r1-designer/`。 |
| a3ea8bea1076bfe49 | experiment-designer | 2026-10-01 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/e165-r2-designer/`。 |
| a312537b5bb8636f9 | three-way-attack | 2026-10-05 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-batch10-code-r2-opus/`，接着用。 |
| ac950c8502e762221 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-r1/batch10r/` |
| ab96ec4f56d877151 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10o/` |
| a79b6a2835751f185 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-r1/batch10r/` |
| a2e289e69bdf5e3a5 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10n/` |
| ab38ba2fca74ee130 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10m/` |
| a606128ec6fe63a6a | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10j/` |
| af05c672e8f16bfd8 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10p/` |
| toolu_01LDsYcDMY2RA3j8p26Bnrhh（没起来） | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10l/` |
| toolu_01LqgYzuA7fdJfgi84q1KzD1（没起来） | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10i/`。 |
| toolu_01Lsk6L3XwothEWsMhSE1YEz（没起来） | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-r1/batch10q/` |
| a2ffe165ee1dfc335 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-r1/batch10q/` |
| ae422392deddf8622 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10i/` |
| a57b2c847598b7c80 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10l/` |
| a0ca347cd4c3a0436 | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10k/` |
| toolu_01T448VtbbS17pRL2q4HZzcK（没起来） | implementation-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-r1/batch10j/` |
| ab0a2b9311fcf6e71 | three-way-attack | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-batch10i-code-r1-opus/` |
| af8c697d2b52cb7d8 | three-way-attack | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-batch10-code-r3-opus/` |
| aa7aa7658e12c10a5 | three-way-defense | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`/tmp/claude-1000/m3-batch10-code-r3-sonnet/`。 |
| toolu_01151VPtTQDMRGGnW6tq4BVw（没起来） | tooling-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t1/`。 |
| abcadc82fb962afae | tooling-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t3/`。 |
| a0535113df1fee586 | tooling-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t4/`。 |
| aacb477ecc86b85b2 | tooling-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t1/`。 |
| a29fe1df8ddbfecc9 | tooling-writer | 2026-10-06 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t2/`。 |
| a79fb5d5fbaa57aa9 | tooling-writer | 2026-10-07 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t6/`。 |
| aa9e276eb073c2787 | tooling-writer | 2026-10-07 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t7/`。 |
| aafcdb49893fbd5a9 | tooling-writer | 2026-10-07 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/root-ring-tooling-t5/`。 |
| toolu_01EtjCQCoKjFfSEcMifR352G（没起来） | experiment-designer | 2026-10-08 | 列表记号 | 没有 | - 草稿目录：~/.local/share/singlefs/work/e171-r1-reuse-design/；报告：同目录 report.md，交回时附 rev11.md 与 |
| a3a61b5c1e2d51a3a | experiment-designer | 2026-10-08 | 列表记号 | 没有 | - 草稿目录：~/.local/share/singlefs/work/e171-r1-reuse-design/；报告：同目录 report.md，交回时附 rev11.md 与 |
| a0d036d7c350ddb3c | three-way-attack | 2026-10-10 | 列表记号 | 没有 | - 草稿目录：~/.local/share/singlefs/work/m3-crash-enumeration-merge-r3-opus/，草稿与临时文件都放这里。 |
| af28cc40e4bfabe0a | three-way-defense | 2026-10-10 | 列表记号 | 没有 | - 草稿目录：~/.local/share/singlefs/work/m3-crash-enumeration-merge-r3-sonnet/，草稿与临时文件都放这里。 |
| aba5b413f57858ea5 | experiment-designer | 2026-10-12 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/e142-r25-design/`。 |
| toolu_013U2Gzr553wgrFEbzcW7DYb（没起来） | sweep | 2026-10-12 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-tooling-b1-sync-facts/`；事实表写 `facts.tsv`、候选表 `cand |
| ad446c7d6cd3dfcfb | sweep | 2026-10-12 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-tooling-b1-sync-judge-b/`；报告写那里的 `report.md`。交回前跑  |
| a181ed4839c5e23be | sweep | 2026-10-12 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-tooling-b1-sync-judge-a/`；报告写那里的 `report.md`。交回前跑  |
| a60f078991dc269cc | sweep | 2026-10-12 | 列表记号 | 没有 | - 草稿目录：`~/.local/share/singlefs/work/m3-tooling-b1-sync-facts/`；事实表写 `facts.tsv`、候选表 `cand |

## 附：三方腿两桶换桶的 181 个

| agent | 类型 | 开工日期 | 改前（按 three-way 起头） | 改后（读登记） |
|---|---|---|---|---|
| a41837d3999a1a5df | three-way-local-attack | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| a7a80b8110a62355b | three-way-local-attack | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| aa1e6dabb11b999c7 | three-way-local-attack | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| af0de971efc6d2ea0 | three-way-local-attack | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| a3d04b4971c5e459f | three-way-local-attack | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a3ddb3a87f1f2ac54 | three-way-local-attack | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a42acb07cc8f55cf8 | three-way-local-attack | 2026-09-22 | three-way-report-or-verdict-missing | report-unreferenced |
| a4f3116d455c02fff | three-way-local-attack | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a810fb51cf8f2ab57 | three-way-local-attack | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a9351efd7f3ca9f99 | three-way-local-attack | 2026-09-22 | verdict-archived | report-used-by-main |
| ad901b18629326e97 | three-way-local-attack | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a2897b1f61accb683 | three-way-local-attack | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| a917e77a0e7044f86 | three-way-local-attack | 2026-09-23 | verdict-archived | written-landed |
| a96dab9cb8775079b | three-way-local-attack | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| ac2cf2ca6f2dc5bdb | three-way-local-attack | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| adccdc7439c1bf4c0 | three-way-local-attack | 2026-09-23 | verdict-archived | report-used-by-main |
| ae3e8fe15d6d17096 | three-way-local-attack | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| af2ab131b87bdca8d | three-way-local-attack | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| a222a7dd2f1643348 | three-way-local-attack | 2026-09-24 | verdict-archived | written-landed |
| a31a4ede6c7bbd3b9 | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a32e03e1259d890e0 | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a4502498d89192997 | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | written-overwritten |
| a45dabe51e5887ab4 | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a4aa77cd7c7a9842a | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a4f62f50ce06c651e | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a51ecbdee1b31cae5 | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | report-used-by-main |
| ac3ff8c865b119969 | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | report-used-by-main |
| ae7e84b66469ff8df | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | report-used-by-main |
| af40eb78d3bde24cf | three-way-local-attack | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a22a922ab9b169d01 | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| a358d468ad00b5e4e | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| a554082c4adf4a0d8 | three-way-local-attack | 2026-09-25 | verdict-archived | written-landed |
| a7ff771c86c1f6df3 | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| a9ecc02bc347f7e5c | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| aa753b3c5ad740780 | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| adc840346394aaa94 | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| ae2aae0cc262be9ef | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| af800866dd22ffe2f | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| af81237ab26e481cc | three-way-local-attack | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| a03480de4b0f89c1e | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| a11666664990ea159 | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| a4138e191b0ee378e | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| a4340137ed670a9ca | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| a689419a9cfdb8490 | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| a6bfd79638f825f55 | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| aa055bf96b49dde73 | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| aa0c6e9e4ba2556d1 | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| ab59faba4fae8cf17 | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| ad2549de56722cdd8 | three-way-local-attack | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| a48f2f806614c1a41 | three-way-local-attack | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| a4c8b85bc38d28879 | three-way-local-attack | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| a85afd254ef1f25a3 | three-way-local-attack | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| a983f41b9eade1699 | three-way-local-attack | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| aabb689c285fcaedc | three-way-local-attack | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| af5018193fb78b98e | three-way-local-attack | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| a2db2967e25e5a75d | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| a3146b869fa830d74 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| a5ef129fea46aa7c6 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| a802b98b02d21a720 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| ab9073eb075b877c9 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| ac59c2325d3c42a6f | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| af479c5042c665d52 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| af6ae5e11613b0775 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| afa8328eb6c59d857 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| afe7282ed60949364 | three-way-local-attack | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| a12d72124fc1a047f | three-way-materials | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| a2223af802d3043cc | three-way-materials | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| a67244c8c152deacf | three-way-materials | 2026-09-21 | three-way-report-or-verdict-missing | report-used-by-main |
| afecb38bc25315733 | three-way-materials | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| a640f08ec59f9045b | three-way-materials | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a6aa1761cd6f21f4f | three-way-materials | 2026-09-22 | three-way-report-or-verdict-missing | report-used-by-main |
| a89e9209bc0b5c510 | three-way-materials | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a9db7187bfd2c9830 | three-way-materials | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| ad80f9e1c7a07afde | three-way-materials | 2026-09-22 | verdict-archived | report-used-by-main |
| af1463dfaf8086c4a | three-way-materials | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| af24f8222596c2606 | three-way-materials | 2026-09-22 | three-way-report-or-verdict-missing | written-landed |
| a294fc16f52f2bd81 | three-way-materials | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| a3f82410e0860ebc4 | three-way-materials | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| a6cdc1061ba3bb639 | three-way-materials | 2026-09-23 | three-way-report-or-verdict-missing | report-used-by-main |
| a91fa4279e4674d79 | three-way-materials | 2026-09-23 | three-way-report-or-verdict-missing | written-landed |
| ad86fd971b9016f4b | three-way-materials | 2026-09-23 | three-way-report-or-verdict-missing | report-used-by-main |
| a2b5e87afb4222001 | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | report-used-by-main |
| a53eb0284f5484dff | three-way-materials | 2026-09-24 | verdict-archived | written-landed |
| a54854b9a591644cc | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a5dd8f65269e3ff0d | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a6b63a8958c16e5ba | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | report-used-by-main |
| a71d707240148442c | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| aae688e79559cc8e4 | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| ae145d20db87a3b41 | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | report-used-by-main |
| af438fe39fa20ede2 | three-way-materials | 2026-09-24 | three-way-report-or-verdict-missing | written-landed |
| a281042c1decca2ed | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| a4430569fbdf7822b | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | written-overwritten |
| a48d9a123c9d9a1dc | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| a63785490e8bb5354 | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| a64b8b70da03d1371 | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | report-used-by-main |
| a9fdbd59dd94b55b5 | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| aa382352f2755cd5e | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| ae4eaa58e19e7d034 | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | written-landed |
| afb99c46ce7ee4527 | three-way-materials | 2026-09-25 | three-way-report-or-verdict-missing | report-used-by-main |
| a2e34a44c0baa8fef | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| a403f6b8640ea3980 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| a4966b22412d9aa11 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| a538c689a2cfb0683 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| a70fa337b0ecdb3cf | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| a8b9f07ed06c8cd28 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| a8f68261c6aa27de3 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| a9b213ed18b7a544e | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| aa05babe6335dfa9c | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| aa774e6f9d1cc5452 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | written-landed |
| ab0f093d2d5db8c32 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | written-partial |
| ada8381c7b52b4769 | three-way-materials | 2026-09-26 | three-way-report-or-verdict-missing | report-used-by-main |
| a4a2e1084b2bb72c5 | three-way-materials | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| a969727da30eb2742 | three-way-materials | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| acdb9e6375b28e868 | three-way-materials | 2026-09-27 | three-way-report-or-verdict-missing | report-used-by-main |
| ae1fca40cebbb91ac | three-way-materials | 2026-09-27 | three-way-report-or-verdict-missing | written-landed |
| ae909dbafec8a730b | three-way-materials | 2026-09-27 | three-way-report-or-verdict-missing | report-used-by-main |
| a0813226b907d657d | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| a20193032dc1f767c | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| a2385ba5c5f12bd98 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| a39e2d462d355bcea | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| a454224f07824ec88 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-unreferenced |
| a503ccfb90d3fec9e | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| a591d8848d5849e1b | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| a5d3cb2a3e18e24d4 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| a6cc971438b82e201 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| a80064dc7ff98763f | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| aa0e30ead47266735 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| ac5276bdb1c0adc36 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| ad5d734e5a2f9584b | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| ae9136b388208c024 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| af41793243c22b855 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| af95eff40aca0e0bb | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | written-landed |
| afcae494a7a3ddf09 | three-way-materials | 2026-09-28 | three-way-report-or-verdict-missing | report-used-by-main |
| a16c3c681ee283cf0 | three-way-verifier | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| a7b1f4fc17dde8b4a | three-way-verifier | 2026-09-21 | three-way-report-or-verdict-missing | written-landed |
| af65ce59c36b98675 | three-way-verifier | 2026-09-21 | verdict-archived | written-landed |
| a3acb9ecd252633d5 | three-way-verifier | 2026-09-22 | verdict-archived | written-landed |
| a5717a31b58a2d3cc | three-way-verifier | 2026-09-22 | verdict-archived | written-landed |
| ab2a69c6c47a5744b | three-way-verifier | 2026-09-22 | verdict-archived | written-landed |
| ae7b14219cc5b31a2 | three-way-verifier | 2026-09-22 | verdict-archived | written-landed |
| afd65623a37123cfe | three-way-verifier | 2026-09-22 | verdict-archived | report-used-by-later-sub |
| a980380834fd14232 | three-way-verifier | 2026-09-23 | verdict-archived | written-landed |
| ab83c319e0ad15110 | three-way-verifier | 2026-09-23 | verdict-archived | written-landed |
| ac43130f0dfd650f2 | three-way-verifier | 2026-09-23 | verdict-archived | written-landed |
| adeae666a813586e9 | three-way-verifier | 2026-09-23 | verdict-archived | written-landed |
| a02ac6f6a768226cb | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a1595a22aced685a7 | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a1e40fb9e504b1bca | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a2b2fadf7feae1fd2 | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a30b212708b8f1fbb | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a36f298a52a1a4804 | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a3be9cd6dec4abb44 | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a644afd1f59a8d17f | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| ae2777eacdafc723b | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| af5a56855fbc8976d | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| af890814a7003333e | three-way-verifier | 2026-09-24 | verdict-archived | written-landed |
| a0587d8f8a23382e9 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| a11a71b6c0c7a5a27 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| a13de3524b2d17fe7 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| a2836b5f5e219df1a | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| aa561c32b8a0af9b8 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| aaa54e56d878b0969 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| acaf1d76afaf7f0a1 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| afe8ca4cf3e164ca7 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| afe9abee2c610de32 | three-way-verifier | 2026-09-25 | verdict-archived | written-landed |
| a069d276ad3ef3c0b | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| a22a88716afb29d88 | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| a234bf073b8be221c | three-way-verifier | 2026-09-26 | three-way-report-or-verdict-missing | written-overwritten |
| a3fbaeb3339847365 | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| a5c738e49e5ed7205 | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| a64b95bd94cde728e | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| ac8c0cebee1236a53 | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| acecaade85592f5cc | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| ad91961b54532505a | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| adbca8a79502abb32 | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| aff9aa03335fdfbdf | three-way-verifier | 2026-09-26 | verdict-archived | written-landed |
| a3c63007082179005 | three-way-verifier | 2026-09-27 | verdict-archived | written-landed |
| a3e05b5d1704c033e | three-way-verifier | 2026-09-27 | verdict-archived | written-landed |
| a4effa19734680b51 | three-way-verifier | 2026-09-27 | verdict-archived | written-landed |
| a7f867dc9f5c35d9f | three-way-verifier | 2026-09-27 | verdict-archived | written-landed |
| add61f47132c9b1b8 | three-way-verifier | 2026-09-27 | verdict-archived | written-landed |

## 附：草稿目录认法变了的 258 个子 agent（`W` 是 `~/.local/share/singlefs/work`）

| agent | 类型 | 开工日期 | 改前 | 改后 |
|---|---|---|---|---|
| a23cec4c87ad4502b | three-way-attack | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i4-r1-opus |
| a400b96a58f68a963 | three-way-forward | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i4-r1-sonnet |
| a69d50f1a8390ef7f | three-way-attack | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i3-r2-opus |
| a6e2eebeae3a1af87 | three-way-forward | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i3-r2-sonnet |
| a7a80b8110a62355b | three-way-local-attack | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i3-r2-local |
| ab2a69c6c47a5744b | three-way-verifier | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i4-r1-verifier |
| ad901b18629326e97 | three-way-local-attack | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i4-r1-local |
| af65ce59c36b98675 | three-way-verifier | 2026-09-21 | （无） | /tmp/claude-1000/m2s3i3-r2-verifier |
| a2871990a1fd54c60 | experiment-runner | 2026-09-23 | （无） | /tmp/claude-1000/e158-runner |
| a656f030a4a2fabb2 | experiment-runner | 2026-09-23 | （无） | /tmp/claude-1000/e156-r2-runner |
| ab7a9784c3cca2bdd | experiment-runner | 2026-09-25 | 用 | （无） |
| af81237ab26e481cc | three-way-local-attack | 2026-09-25 | （无） | /tmp/claude-1000/m2-rollback-forward-r2-local-attack-2 |
| a03480de4b0f89c1e | three-way-local-attack | 2026-09-26 | （无） | /tmp/claude-1000/m2-layer0-scale-r3-local |
| a689419a9cfdb8490 | three-way-local-attack | 2026-09-26 | （无） | /tmp/claude-1000/m2-safety-r3-local |
| a6bfd79638f825f55 | three-way-local-attack | 2026-09-26 | （无） | /tmp/claude-1000/defs-gatebatch-r1-local |
| aa055bf96b49dde73 | three-way-local-attack | 2026-09-26 | （无） | /tmp/claude-1000/defs-closeout-r3-local |
| abb8824a27cf9c341 | three-way-local-attack | 2026-09-26 | （无） | /tmp/claude-1000/defs-gatebatch-r1-local |
| a1d982eaad55ecf86 | three-way-materials | 2026-09-27 | /tmp/claude-1000/three-way-materials/verification-split-r1 | （无） |
| a35058f2401bdc9c8 | three-way-local-attack | 2026-09-27 | /tmp/claude-1000/three-way-local-attack/verification-split-r1 | （无） |
| a5a8b7ae9f4715cfd | three-way-attack | 2026-09-27 | /tmp/claude-1000/three-way-attack/verification-split-r1 | （无） |
| a6f179c06502064e7 | kb-spec-drafter | 2026-09-27 | （无） | /tmp/claude-1000/kb-batch9-drafter |
| a86f9cd1084fa2558 | kb-spec-drafter | 2026-09-27 | （无） | /tmp/claude-1000/kb-batch7a-drafter |
| a877c8d22773122a2 | kb-spec-drafter | 2026-09-27 | （无） | /tmp/claude-1000/kb-writeback-batch2 |
| a970116c2a5cf3c23 | three-way-forward | 2026-09-27 | /tmp/claude-1000/three-way-forward/verification-split-r1 | （无） |
| aae747cb641ad131a | experiment-designer | 2026-09-27 | （无） | /tmp/claude-1000/e162-s3-s4-designer |
| adc5007ca39e07c13 | kb-spec-drafter | 2026-09-27 | （无） | /tmp/claude-1000/kb-writeback-batch3 |
| add61f47132c9b1b8 | three-way-verifier | 2026-09-27 | （无） | /tmp/claude-1000/three-way-verifier-r2 |
| a07cfc86ea0ed72a4 | investigator | 2026-09-28 | （无） | /tmp/claude-1000/closeout-recheck-2026-09-28/c393 |
| a43454e5d3c187ef6 | sweep | 2026-09-28 | （无） | /tmp/claude-1000/m3-sweep-layer1 |
| a74ade2abfb20c3c8 | investigator | 2026-09-28 | （无） | /tmp/claude-1000/closeout-recheck-2026-09-28/c335 |
| a802b98b02d21a720 | three-way-local-attack | 2026-09-28 | （无） | /tmp/claude-1000/unreadable-at-mount-r2/local-attack |
| a8381bdc3e30b5072 | investigator | 2026-09-28 | （无） | /tmp/claude-1000/closeout-recheck-2026-09-28/row43 |
| ab40af107bb08729b | investigator | 2026-09-28 | （无） | /tmp/claude-1000/closeout-recheck-2026-09-28/c331 |
| ab510bf79c24e5c39 | experiment-designer | 2026-09-28 | （无） | /tmp/claude-1000/m3-function-test-designer |
| ab9073eb075b877c9 | three-way-local-attack | 2026-09-28 | （无） | /tmp/claude-1000/unreadable-at-mount-r1/local-attack |
| ad77703ad3eb5f34a | tooling-writer | 2026-09-28 | （无） | /tmp/claude-1000/tool-prove-red-should-panic |
| afa8328eb6c59d857 | three-way-local-attack | 2026-09-28 | （无） | /tmp/claude-1000/abandoned-floor-r1/local-attack |
| a581ceda0b22e00f8 | prior-art | 2026-09-30 | /tmp/claude-1000/m3-r1/formal-prior-art | （无） |
| a698f40ef4547f722 | investigator | 2026-09-30 | /tmp/claude-1000/m3-r1/memory-cap-investigation | （无） |
| aa128e7df20420d25 | investigator | 2026-09-30 | /tmp/claude-1000/m3-r1/memory-cap-investigation | （无） |
| aaebbb0be32d4bd07 | investigator | 2026-09-30 | /tmp/claude-1000/m3-r1/memory-cap-investigation | （无） |
| ab1940e0fb194886e | investigator | 2026-09-30 | /tmp/claude-1000/m3-r1/memory-cap-investigation | （无） |
| a09bdc58758ecf0e0 | three-way-attack | 2026-10-01 | /tmp/claude-1000/test-script-align-r2-opus | （无） |
| a1192d8c11a34f234 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/G5 |
| a139bd6fda2c9ded9 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/G2 |
| a19a3132ed67f29c1 | investigator | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/investigate-qemu-zero-fill |
| a248bf7f145540eee | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/E |
| a267ce516bf7fef18 | investigator | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/investigate-qemu-eio |
| a3b3a339183d19989 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/rerecord-qemu |
| a4114088198317078 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/G4 |
| a4bfef418fc18908f | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/fix-qemu-patterns |
| a4d0a4614638926ca | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/G3 |
| a4fc3d2233fa06db2 | investigator | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/investigate-qemu-segments |
| a575291f5727e1006 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/G1 |
| a5ffe3ab891ed0e98 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/C |
| a6250304f0fb7d245 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/register-qemu-selftests |
| a735388ffa34fb8bf | three-way-forward | 2026-10-01 | /tmp/claude-1000/test-script-align-r1-sonnet | （无） |
| a7f6bd9fce835d569 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/A |
| a8be108683ccebf5f | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/fix-guard2 |
| a92eb1b4d8a2be010 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/heavy-qemu-build-2 |
| a94be86575f1c259a | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/D |
| a9749db7fb530ebe6 | three-way-forward | 2026-10-01 | /tmp/claude-1000/test-script-align-r3-sonnet | （无） |
| a9f229af0cd3c9e2f | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/heavy-qemu-build |
| aa371edd03121b6ee | kb-scribe | 2026-10-01 | （无） | /tmp/claude-1000/m3-r1/batch4-kb |
| aa6e8bd2ab3f0dd36 | implementation-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/fix-zero-fill-fold |
| aa8e9cf8fe9554cbd | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/F-formal |
| aacc96695a66771a3 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/fix-qemu |
| ab4098c83f53efca1 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/F-code-tooling |
| abc89e734b6b6c6a7 | three-way-attack | 2026-10-01 | /tmp/claude-1000/test-script-align-r1-opus | （无） |
| ac9d46b6646be7d83 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/F-code-tooling-2 |
| acb78cbab796fd7b5 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/G2 |
| adfd1a6bd59d5a583 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/B |
| ae5bdc93c3a9f3004 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/patch-qemu |
| ae9d9efd6f4c11b14 | three-way-defense | 2026-10-01 | /tmp/claude-1000/test-script-align-r2-sonnet | （无） |
| aeb6ea2b61209729d | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/fix-guard |
| af1899d9aaba3ffda | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/fix-docexp |
| af2a3ff0e7e291861 | three-way-attack | 2026-10-01 | /tmp/claude-1000/test-script-align-r3-opus | （无） |
| afdcfb78860a09fc7 | tooling-writer | 2026-10-01 | （无） | /tmp/claude-1000/gate-comment-distill/F-layer0-mutation |
| a026ffbbc7128772e | experiment-runner | 2026-10-02 | /tmp/claude-1000/e170-r1-segment1 | （无） |
| a09dfe801ffc1e272 | three-way-defense | 2026-10-02 | /tmp/claude-1000/m3-birth-floor-r2-sonnet | （无） |
| a0b4bdcbe427d6f06 | three-way-defense | 2026-10-02 | /tmp/claude-1000/m3-formal-code-r2-defense | （无） |
| a0ea178241809b824 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r4-segment1 | （无） |
| a0f4bcfb72e3d97b0 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e171-segment1 | （无） |
| a15dcbaca0e762061 | three-way-defense | 2026-10-02 | /tmp/claude-1000/m3-birth-floor-r3-sonnet | （无） |
| a17c505a7c41c7bc9 | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-broad-gate-gov-r1-opus | （无） |
| a182943aead9a9e13 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r3-segment3 | （无） |
| a1da032edaf33a873 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r3-segment2 | （无） |
| a204ef7a3166e83b1 | three-way-forward | 2026-10-02 | /tmp/claude-1000/m3-formal-code-r1-forward | （无） |
| a28d7c8a2a6b5f919 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r3-segment3b | （无） |
| a28e6fa1dfe13eef7 | three-way-forward | 2026-10-02 | /tmp/claude-1000/m3-batch4-code-r1-sonnet | （无） |
| a2c335826f53b9d03 | three-way-forward | 2026-10-02 | /tmp/claude-1000/m3-broad-gate-gov-r1-sonnet | （无） |
| a2e7382cc264507cb | clerk | 2026-10-02 | /tmp/claude-1000/m3-formal-10-5 | （无） |
| a31d7cc8be865c1bf | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-birth-floor-r3-opus | （无） |
| a36c9bee9a4f4c5f5 | implementation-writer | 2026-10-02 | /tmp/claude-1000/m3-formal-11-11 | （无） |
| a3fd99b06208ff36c | three-way-defense | 2026-10-02 | /tmp/claude-1000/m3-ring-r3-sonnet | （无） |
| a4effc6b73b8e0f54 | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-c651-r1-attack | （无） |
| a5165856446a5effa | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r2-segment4 | （无） |
| a598bd9e7a1d707e8 | implementation-writer | 2026-10-02 | /tmp/claude-1000/m3-formal-10-3 | （无） |
| a5a31241c51de5457 | implementation-writer | 2026-10-02 | /tmp/claude-1000/m3-formal-11-3 | （无） |
| a5aeee17c21987170 | experiment-runner | 2026-10-02 | （无） | /tmp/claude-1000/e171-segment4 |
| a5c908ec2ef973ce8 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e170-r1-segment3 | （无） |
| a638358c627326aae | kb-scribe | 2026-10-02 | （无） | /tmp/claude-1000/m3-formal-14-4 |
| a65cf43cdfe99df4b | experiment-runner | 2026-10-02 | /tmp/claude-1000/e170-r1-segment2 | （无） |
| a6a036abab9d99169 | three-way-defense | 2026-10-02 | /tmp/claude-1000/m3-formal-r6-defense | （无） |
| a6d14d1f82ae108e7 | implementation-writer | 2026-10-02 | （无） | /tmp/claude-1000/m3-formal-14-3 |
| a763b31c4f51085b5 | experiment-runner | 2026-10-02 | （无） | /tmp/claude-1000/e171-segment2 |
| a76b30b4909a09eed | three-way-defense | 2026-10-02 | （无） | /tmp/claude-1000/m3-formal-code-r3-defense |
| a7c301cdc9418773f | experiment-runner | 2026-10-02 | （无） | /tmp/claude-1000/e165-r4-segment3 |
| a8849ac5b4a65e756 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r3-segment1 | （无） |
| a91dbd0c99a0e1872 | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-formal-r5-attack | （无） |
| a9d8d4841ebf8ee4c | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-birth-floor-r2-opus | （无） |
| aa0c95cbcbd9ae578 | kb-scribe | 2026-10-02 | （无） | /tmp/claude-1000/m3-formal-13-5 |
| aa779aad54d4ec051 | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-formal-r6-attack | （无） |
| ab64e060795a18127 | implementation-writer | 2026-10-02 | /tmp/claude-1000/m3-formal-12-2 | （无） |
| abd29aacfc04d2184 | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-formal-code-r1-attack | （无） |
| abeb9d0205883eff8 | implementation-writer | 2026-10-02 | （无） | /tmp/claude-1000/m3-formal-13-4 |
| abec7fab894ce4bda | investigator | 2026-10-02 | （无） | /tmp/claude-1000/inject6-findings |
| abef2dedb5bffae15 | implementation-writer | 2026-10-02 | /tmp/claude-1000/m3-formal-11-4 | （无） |
| ac19d983645d0fcd1 | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r4-segment1b | （无） |
| ac5a5f01136595ccb | experiment-runner | 2026-10-02 | （无） | /tmp/claude-1000/e171-segment3 |
| ac724250fff0f668c | experiment-runner | 2026-10-02 | /tmp/claude-1000/e165-r2-segment2c | （无） |
| ac84de5e3045fda60 | three-way-forward | 2026-10-02 | /tmp/claude-1000/m3-c651-r1-forward | （无） |
| ac8efa20510d40b9e | implementation-writer | 2026-10-02 | /tmp/claude-1000/m3-formal-11-5 | （无） |
| ac9cad4cb46d5758d | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-batch4-code-r1-opus | （无） |
| ad4f0311b395fcfd5 | three-way-defense | 2026-10-02 | /tmp/claude-1000/m3-formal-r5-defense | （无） |
| ae29991860c72d01c | implementation-writer | 2026-10-02 | /tmp/claude-1000/m3-formal-11-10 | （无） |
| ae8a31c9b24f98351 | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-ring-r3-opus | （无） |
| ae93f190af8ca4a16 | implementation-writer | 2026-10-02 | （无） | /tmp/claude-1000/m3-formal-13-3 |
| af34b8ad803ccb466 | experiment-runner | 2026-10-02 | （无） | /tmp/claude-1000/e165-r4-segment4 |
| af4ba5596e8075d51 | three-way-attack | 2026-10-02 | （无） | /tmp/claude-1000/m3-formal-code-r3-attack |
| af81874a0ee065277 | kb-scribe | 2026-10-02 | /tmp/claude-1000/m3-formal-12-3 | （无） |
| afbd63593dd5add19 | three-way-attack | 2026-10-02 | /tmp/claude-1000/m3-formal-code-r2-attack | （无） |
| a00d9a6848a177f10 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/c673-clauses-r1-sonnet |
| a05a5cd47504a3e3a | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b17-r1-sonnet |
| a0eea55003084dfb2 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b16-r1-sonnet |
| a0f996b4b36e022c5 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e170-r2-export-prep2 |
| a11e33293c5211c1e | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b21-r1-opus |
| a180abb03d24dedc1 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b23-r1-sonnet |
| a194f412231c976b2 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b18-r1-sonnet |
| a1a33aab6d4a4a807 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e171-r1-segment1 |
| a1af915e51aa6b84f | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b21-r1-sonnet |
| a1ed0fffdd39bf42d | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b19-r1-opus |
| a248db6d39b1cf747 | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b23-r1-opus |
| a2912c55ca790150a | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e165-r5-segment2 |
| a2cfa5f42de3c6f39 | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-batch16-code-r1-opus |
| a30542e7fcbbda79e | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b20-r1-opus |
| a41421aab9f6a7a41 | experiment-designer | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-21-3 |
| a4a371f3a3c090ee5 | experiment-designer | 2026-10-03 | （无） | /tmp/claude-1000/e170-r2-designer-rev1 |
| a4c71e44f581e591c | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b16-r1-opus |
| a4fa7c58822d8ce91 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/e171-conclusion-r3-sonnet |
| a54c2b11381199cbc | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e171-segment8 |
| a62e7556a1684e02a | three-way-defense | 2026-10-03 | （无） | /tmp/claude-1000/e171-conclusion-r2-sonnet |
| a751c419c2032b406 | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/e171-conclusion-r1-opus |
| a88ca34b375effef7 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e170-r2-segment1b |
| a8c0ae7a0ba6ba924 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/e171-conclusion-r1-sonnet |
| a8d2363c8754b4972 | experiment-designer | 2026-10-03 | （无） | /tmp/claude-1000/e170-r2-designer-rev2 |
| a8d9031e7a3cadcf2 | tooling-writer | 2026-10-03 | （无） | /tmp/claude-1000/e165-r5-driver-install |
| a9300bc2bf91b2469 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b19-r1-sonnet |
| a9e77d63b25a5d4b9 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e170-r2-segment2-q5 |
| a9e91c9fefeb3fc1b | experiment-designer | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-21-4 |
| a9f69401af73831e1 | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b18-r1-opus |
| aa0207c66cfddd4d4 | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/e171-conclusion-r2-opus |
| aa08be2fb9f688f86 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e171-segment7 |
| aaca1331d51098481 | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/c673-clauses-r1-opus |
| ab4e649d995ff9556 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e170-r2-export-prep3 |
| ab95cebba59feb875 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b20-r1-sonnet |
| abd1c703f4b361f41 | implementation-writer | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-21-2 |
| abe3ba2f46cc0dd93 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e171-segment6 |
| ac1f11f9b8834825c | implementation-writer | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-23-1 |
| ac4ca5ee1357437c5 | implementation-writer | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-21-1 |
| acdd1743545e54556 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e165-r5-segment3 |
| ad21a7e158142b6fd | tooling-writer | 2026-10-03 | （无） | /tmp/claude-1000/e165-r5-driver-gate-pins |
| ae6663e0cf4ae1c1e | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/m3-formal-code-b17-r1-opus |
| ae7b245dddf471985 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e171-segment5 |
| ae9ba9728c571cd75 | three-way-forward | 2026-10-03 | （无） | /tmp/claude-1000/m3-batch16-code-r1-sonnet |
| af9b530debbdfe2b5 | experiment-runner | 2026-10-03 | （无） | /tmp/claude-1000/e165-r5-segment1 |
| afe234fdd944fdc94 | three-way-attack | 2026-10-03 | （无） | /tmp/claude-1000/e171-conclusion-r3-opus |
| a4098c52b80224334 | three-way-forward | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-code-b24-r1-sonnet |
| a472e85c02850d6e1 | implementation-writer | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-24-1 |
| a55de735fd87c2978 | three-way-attack | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-code-b24-r1-opus |
| a5ba2e6d6a258358b | implementation-writer | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-25-1 |
| a5e6255875dc58af6 | implementation-writer | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-25-1 |
| a62f2a99f6912dda5 | experiment-designer | 2026-10-04 | （无） | /tmp/claude-1000/e171-r1-designer-rev2 |
| a63b34d02cbb64a40 | experiment-runner | 2026-10-04 | （无） | /tmp/claude-1000/e170-r2-replay-products |
| a9abe7a166fbd65fb | three-way-forward | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-c655-c687-r1-sonnet |
| a9b0510cf8553c67b | implementation-writer | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-25-3a |
| aa8cb5a61ba54b286 | three-way-attack | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-code-b24-r1-opus |
| ab61363cd230e8eea | experiment-runner | 2026-10-04 | （无） | /tmp/claude-1000/e165-r5-landing |
| ade3e757be6052695 | three-way-forward | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-code-b24-r1-sonnet |
| afd5481831ed4ee81 | three-way-attack | 2026-10-04 | （无） | /tmp/claude-1000/m3-formal-c655-c687-r1-opus |
| affb00e975410bbbe | tooling-writer | 2026-10-04 | （无） | /tmp/claude-1000/e170-r2-replay-rows |
| a4355856593ed9e06 | experiment-designer | 2026-10-05 | （无） | /tmp/claude-1000/m3-r1/batch10-ceiling-exp-design |
| a0db1982845b0cee2 | three-way-forward | 2026-10-06 | W/m3-batch31-code-r1-sonnet | （无） |
| a47c005c878a51e35 | three-way-attack | 2026-10-06 | W/m3-batch31-code-r1-opus | （无） |
| a0481a50cac104fde | three-way-attack | 2026-10-07 | W/m3-c694-r1-opus | （无） |
| a5db091a2043172db | three-way-attack | 2026-10-07 | W/m3-batch33r-code-r1-opus | （无） |
| a6239f62d46c13508 | three-way-defense | 2026-10-07 | W/m3-c694-r2-sonnet | （无） |
| a692defe14d5f3dae | three-way-forward | 2026-10-07 | W/m3-c694-r1-sonnet | （无） |
| a708ccdcfd7ec4330 | kb-scribe | 2026-10-07 | （无） | W/m3-batch40-kb-owed |
| a719db15b405f1a18 | implementation-writer | 2026-10-07 | （无） | W/m3-batch41/a |
| a7695c773bd0e5fa1 | three-way-attack | 2026-10-07 | W/m3-batch32-code-r1-opus | （无） |
| a7db6975430f8996f | three-way-forward | 2026-10-07 | W/m3-batch33r-code-r1-sonnet | （无） |
| a82dea0c7158c969f | experiment-designer | 2026-10-07 | （无） | W/m3-r1/g2-yi-prereg |
| ad50aa1f2611f1bc8 | three-way-forward | 2026-10-07 | W/m3-batch32-code-r1-sonnet | （无） |
| af1becf4127ea3a0b | three-way-attack | 2026-10-07 | W/m3-c694-r2-opus | （无） |
| a0d7710076749f4a3 | experiment-runner | 2026-10-08 | （无） | W/e175-runner-2 |
| a0f09197dd3495cec | three-way-forward | 2026-10-08 | W/m3-retire-r1-sonnet | （无） |
| a2b2c4cc6cba63b34 | experiment-runner | 2026-10-08 | （无） | W/e177-runner |
| a34acfd394d1c7768 | kb-scribe | 2026-10-08 | （无） | W/m3-batch44-kb-decisions |
| a3a61b5c1e2d51a3a | experiment-designer | 2026-10-08 | W/e171-r1-reuse-design | （无） |
| a56c28b312df8026c | implementation-writer | 2026-10-08 | （无） | W/m3-batch45/a |
| a61fd55f3fb000ba0 | experiment-designer | 2026-10-08 | （无） | W/e177-rev1-design |
| a6a319e5aca7b665e | experiment-runner | 2026-10-08 | （无） | W/e176-runner-2 |
| a6abde294bf19b348 | three-way-attack | 2026-10-08 | W/m3-retire-r1-opus | （无） |
| a8b23d1b25665c7d1 | implementation-writer | 2026-10-08 | （无） | W/m3-batch42/a |
| a8cc2603b9f44d8dc | implementation-writer | 2026-10-08 | （无） | W/m3-batch43/a |
| a9d4eaa35bc6295d6 | experiment-runner | 2026-10-08 | （无） | W/e177-runner-2 |
| aa659d101e53b38c5 | experiment-designer | 2026-10-08 | （无） | W/m3-3-5-W-design |
| ab2ea9a9a866d834f | tooling-writer | 2026-10-08 | （无） | W/m3-batch48-amplification-registry |
| ad2f35af6cffca94b | experiment-designer | 2026-10-08 | （无） | W/m3-3-5-C109-design |
| ad4849d4eebc0fe69 | implementation-writer | 2026-10-08 | W/m3-batch49/w3 | （无） |
| add5ceed665b2a027 | experiment-runner | 2026-10-08 | （无） | W/e176-runner |
| addb8aed443cb7069 | kb-scribe | 2026-10-08 | （无） | W/m3-batch46-kb-owed |
| ae97e5805549afde0 | experiment-runner | 2026-10-08 | （无） | W/e175-runner |
| af6ec1f3a82de1c4a | experiment-designer | 2026-10-08 | （无） | W/m3-3-5-M-design |
| a03c82fc0b11d8930 | implementation-writer | 2026-10-09 | W/m3-batch49/w3c | （无） |
| a14c009175392ab09 | three-way-defense | 2026-10-09 | W/m3-crash-enumeration-merge-r2-sonnet | （无） |
| a307928f26137db5e | three-way-attack | 2026-10-09 | W/m3-retire-r1-opus-s2 | （无） |
| a40debfe56e159260 | kb-scribe | 2026-10-09 | W/kb-writeback-retire | （无） |
| a42f68025e4f26c33 | three-way-forward | 2026-10-09 | W/m3-crash-enumeration-merge-r1-sonnet | （无） |
| a46c5814dc0e81b5a | kb-scribe | 2026-10-09 | W/kb-writeback-retire-b | （无） |
| a75bd4386aea2f11c | three-way-attack | 2026-10-09 | W/m3-crash-enumeration-merge-r1-opus | （无） |
| abde6ad224cf389b8 | implementation-writer | 2026-10-09 | W/m3-batch49/w3b | （无） |
| ac4e13659f2bc7bd6 | implementation-writer | 2026-10-09 | W/m3-batch50/m3 | （无） |
| acb6f2487a0e6a2d3 | three-way-attack | 2026-10-09 | W/m3-crash-enumeration-merge-r2-opus | （无） |
| acca663ea965c7a27 | implementation-writer | 2026-10-09 | W/m3-batch50/m3b | （无） |
| aee56c4aeeab46481 | kb-scribe | 2026-10-09 | W/kb-writeback-batch50 | （无） |
| aef3c62459b5ccc29 | tooling-writer | 2026-10-09 | （无） | W/formal-verification-items-and-reuse-2 |
| afc8bb267f8c6c466 | experiment-designer | 2026-10-09 | （无） | W/e165-r6-designer |
| a0d036d7c350ddb3c | three-way-attack | 2026-10-10 | W/m3-crash-enumeration-merge-r3-opus | （无） |
| a1266d76641f2a63a | three-way-attack | 2026-10-10 | W/m3-run-compare-split-r2-attack | （无） |
| a1ad76e01091ed0ff | three-way-attack | 2026-10-10 | （无） | W/m3-run-compare-split-r1-attack |
| a2f1e2d3eaac9fa59 | three-way-forward | 2026-10-10 | （无） | W/m3-run-compare-split-r1-forward |
| ae3fd237dad868689 | three-way-defense | 2026-10-10 | W/m3-run-compare-split-r2-defense | （无） |
| ae7e280a26d5e5533 | three-way-defense | 2026-10-10 | W/m3-run-compare-split-r3-defense | （无） |
| ae8f8981107167d93 | three-way-attack | 2026-10-10 | W/m3-run-compare-split-r3-attack | （无） |
| af28cc40e4bfabe0a | three-way-defense | 2026-10-10 | W/m3-crash-enumeration-merge-r3-sonnet | （无） |
| a0279583f66a91019 | tooling-writer | 2026-10-11 | W/m3-merge-impl/fixtures-after-move/code-tooling-env | （无） |
| a364fc1685b428e23 | tooling-writer | 2026-10-11 | W/m3-merge-impl/fixtures-after-move/formal-broad | （无） |
| a4f00a7cdea89581b | tooling-writer | 2026-10-11 | W/m3-merge-impl/fixtures-after-move/qemu-research | （无） |
| a7551ab925195552a | tooling-writer | 2026-10-11 | W/m3-merge-impl/fixtures-after-move/doc | （无） |
| a8f1c481095d9c421 | tooling-writer | 2026-10-11 | W/m3-merge-impl/fixtures-after-move/code-source | （无） |
| a21edc7924143cf09 | investigator | 2026-10-13 | （无） | W/investigate-e171-s3-segments |
| a3ec10926758127bf | tooling-writer | 2026-10-13 | （无） | W/tooling-c790-light-slice-2 |
| a7add9078db1a5419 | experiment-runner | 2026-10-13 | （无） | W/e171-r12-runner |
| acff57c83a78c1a91 | tooling-writer | 2026-10-13 | W/trilingual-gate | （无） |
| adb6cfb059f7a8863 | experiment-designer | 2026-10-13 | （无） | W/e171-s3-repin-designer |
| adcdb59bea069b427 | experiment-runner | 2026-10-13 | （无） | W/e171-r12-runner-2 |

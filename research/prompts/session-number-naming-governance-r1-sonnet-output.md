# 治理文件轮 session-number-naming-governance-r1 正推腿报告

读的是主工作区此刻那一版，日期 2026-11-19。开工快照 `research/prompts/session-number-naming-governance-r1-snapshot/judged-sha256.txt` 里 `.claude/agent-common.md` 的哈希是 f4b8717f…，我读的时候是 7ec7c6ee…（`sha256sum .claude/agent-common.md` 现取）；第 41 行与第 102 行 ㉒ 是读到的那一版，别的文件哈希与快照一致（`.claude/rules/writing-discipline-by-review.md` 09e5d8e4…、`.claude/hooks/session-number-stop-check.sh` c319df05…、`.claude/settings.json` 64dcb1b0…、`.claude/gate.d/code-tooling.sh` a5ace471…）。

背景材料里一处路径写错：它说上游 `courtesy-phrase.py` 在 `.claude/singlefs-ai-sop/scripts/claude-hooks/` 下；实际在 `.claude/singlefs-ai-sop/scripts/courtesy-phrase.py`（`ls` 现查，`claude-hooks/` 里只有 `courtesy-phrase-check.sh`），钩子载入的也是前一条（`.claude/hooks/session-number-stop-check.sh:78`）。

## 三问结论

| 问 | 结论 | 一句话 |
|---|---|---|
| 1 两处规则是不是用户要的事、彼此一致、与别处不打架 | 成立，另有两处推不出（算打中） | 两处文字一致、与 ⑰ 不打架；「ListAgents 那一行的编号」与用户在标题里看到的编号是两个来源，可能不同；已停的会话没有那一行 |
| 2 检查判的是不是规则说的那件事 | 不成立（打中） | 常见写法拦放都对；反引号、代码块两类里各有合规写法被拦，量到的是源码里那几条正则的行为 |
| 3 ㉒ 那一条与检出钩子逐项对得上 | 不成立（打中） | 有三处写法与代码行为相反，另有几处漏写 |

## 要主 agent 跑的命令

探针两份（`stop_check_cases.py` 喂真钩子 Stop 输入、`detector_claim_cases.py` 把 git 命令当文本喂检出钩子，命令不执行）由 `rerun.sh` 顺序起；子 agent 起不了项目代码（我自己起一次被项目代码闸拒），所以下面各格的「真钩子读数」都是「待主 agent 跑」。

- 命令（在仓根起，一条）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/session-number-naming-governance-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/session-number-naming-governance-r1-snapshot /home/fy5090/.local/share/singlefs/work/session-number-naming-governance-r1-sonnet2-run /home/fy5090/code/singlefs/research/prompts/session-number-naming-governance-r1-sonnet-model
```

- 看哪几行：`TASK_SERVER_RESULT` 行 `type=` 与 `exit=0`；`task-server/task log` 里 `name=done cases=44 model_differs_from_want=13 real_differs_from_want=14`（推的预期，值以跑出来的为准）与另一条 `name=done` 行里的 `claim_decided_differs_from_hook=3`（推的预期）。每个 `name=case` 行并排写着规则说的（want 或 claim）、照源码正则重做的（model）与真钩子的（real 或 hook）。
- 内存与时间：整份探针没跑过，没有读数；`detector_claim_cases.py` 每例起一次 `bash` 检出钩子、`stop_check_cases.py` 每例起一次收工钩子，各自 120 秒超时，进程组各自独立（`start_new_session=True`）。模板自带的 `rerun.sh --selftest` 测的是模板原本那段 `rerun_steps`，我换了那一段，不跑它。
- 模型目录 `research/prompts/session-number-naming-governance-r1-sonnet-model/`，每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
66dc6a685f1f186121d7d6d59ac246bd563bac4e16926b11b3178bea6731953f  ./probe/detector_claim_cases.py
a67c7c362ea8e7eb30ae3254bc9088b54e683e931cb54598864a8d103a2e1748  ./probe/stop_check_cases.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
11de4daefc17259eda9ffcd3694d76f541dde5a9fa88c14005061a6129c4c067  ./tools/rerun.sh
```

我自己直接跑过的只有一段内联 python：把 `.claude/singlefs-ai-sop/scripts/courtesy-phrase.py:87`、`:88` 与 `.claude/hooks/session-number-stop-check.sh:69` 三条正则的字面抄进 `python3 -I -c`，按钩子 `judged_reply_text` 与 `letter_hits` 的次序（先去围栏、再去引号、再逐行找）判 13 句。输出原样（「量的」，只说明这三条正则对这 13 句的行为，不是钩子进程的读数）：

```
odd quote pass
lone backtick pass
double backtick block
multiline span block
indent code block
nested fence block
Mainagent pass
space pass
AB pass
fullwidth pass
newline pass
single quote block
main agent pass
```

## 问 1：两处规则是不是用户要的事，彼此一致，与别处不打架

判定：成立；两处推不出，算打中。

**用户要的事**：用户原话在背景材料里（「现在说的是 mainagent(A) 我不知道是和谁对接的」）。规则那条的前半句是把给用户的回复里指别的会话改成它的编号，对得上这句话。规则文件里的整行（行内有反引号，整行放进围栏里抄）：

```
.claude/rules/writing-discipline-by-review.md:66
- 给用户的回复与汇报里指别的会话，写它在 `ListAgents` 那一行的编号（例「(5b)」），连同它改了哪个文件、做了什么；不写 mainagent(A) 这类字母，也不写 `<做什么>那一路的会话`、`那一路的 agent` 这类描述。会话之间的消息指别的会话写 mainagent(A)、mainagent(B)，指子 agent 写 subagent(A)、subagent(B)（括号里只放大写字母），同一份文字里按出场次序编字母，写清它改了哪个文件、做了什么，不写会话全名、两位短号与那类描述；给用户的回复与汇报里指子 agent 同样写 subagent(A)、subagent(B)（进仓文字的同一条在 `.claude/agent-common.md`「写」一节）。
```

共用约束那一句在 `.claude/agent-common.md:41`，末尾整句：

```
会话之间的消息照同一条；给用户的回复与汇报里别的会话写它在 `ListAgents` 那一行的编号（`.claude/rules/writing-discipline-by-review.md`「怎么说」）。
```

**彼此一致（逐项并排）**：

| 项 | 规则文件第 66 行 | 共用约束第 41 行 | 一致吗 |
|---|---|---|---|
| 给用户的回复里指别的会话 | 写 ListAgents 那一行的编号，连同改了哪个文件、做了什么 | 写 ListAgents 那一行的编号 | 一致；第 41 行没抄「连同…」，但它指到第 66 行所在的「怎么说」 |
| 会话之间的消息 | mainagent(A)、mainagent(B)，按出场次序 | 「照同一条」，同一条是该行前半句的 mainagent(A)、mainagent(B)、mainagent(C) | 一致 |
| 子 agent | subagent(A)、subagent(B)，给用户的回复里也是 | 前半句写 subagent(A)…；给用户的回复里指子 agent 没写 | 不冲突：第 41 行整条的题头是「进仓的文字」（第 40 行），给用户那一半的主条是第 66 行 |
| 拿描述指会话 | 回复里与消息里都不写 | 前半句不写 | 一致 |

**与第 66 行同一节、与共用约束别处不打架**：

- `.claude/agent-common.md` 里「汇报」一词只出现在第 41 行（`grep -nF '汇报' .claude/agent-common.md` 只命中第 41 行），子 agent 的产出在共用约束里一律叫「报告」；第 41 行写的是「给用户的」汇报，子 agent 交主 agent 的报告不在其内，钩子文件头第 5 至 6 行同样把子 agent 交给主 agent 的报告划在外面。无冲突。
- 写闸与检出钩子 ⑰（`.claude/agent-common.md:105`、`:97`）判的是写进仓里的 `.md`、`.tsv`、`.txt`；给用户的回复不是仓里的文件，不过这两道。「(5b)」这种不挨会话词、数字字母都有的短号，只在 `records/` 下与判决里判：`.claude/gate.d/lib-forbidden-notations.py:84` 写的是 `不挨会话词、数字字母都有的（「(5b)」「（3b）」）只在 records/ 下与判决`。所以规则文件第 66 行自己带的例子不触发写闸，规则成立；代价是：主 agent 把给用户的汇报原文抄进 `records/` 或判决时，编号要换成按出场次序的 mainagent(A)，这一步规则没有明说（落在「进仓的文字」那条里，第 40 至 41 行）。

**推不出的两处（打中）**：

1. 编号取哪一个。规则说「ListAgents 那一行的编号」；用户在会话列表里看到的是标题开头的 `(编号)`（`.claude/singlefs-ai-sop/rules/communication-discipline.md:53` 是「## 会话标题开头写编号」，编号取默认名 `<目录名>-<编号>` 里的那一段，见 `:57`）。标题里的号与默认名的号在两种情形下不一样：`.claude/singlefs-ai-sop/scripts/session-number-title.py:18` 的「让号：不管号是从哪来的（默认名、标题、状态目录），登记里别的会话也用着同一个号、而且它起得更早」，与 `:33` 的「（标题里也没有时用会话 ID 算的号，它与会话列表里显示过的默认名对不上）」。我没有 `ListAgents` 工具，读不到那一行到底印标题还是默认名，所以「规则指的那个编号就是用户看见的编号」这一句推不出。能照做的验证：主 agent 在一个让过号的会话里对照 `ListAgents` 那一行与该会话标题开头的号。
2. 已停的会话没有那一行。`.claude/singlefs-ai-sop/rules/communication-discipline.md:48` 写「要找的会话不在列表里，要说的事报给用户」——要对用户说一个已经停了的会话时，规则要求写它在 ListAgents 那一行的编号，而那一行不存在；规则没给别的写法，钩子也不管这种情形。

「不回 <会话名>」那一行（`.claude/singlefs-ai-sop/rules/communication-discipline.md:40`）要求回复里写会话名；第 66 行的「不写会话全名」在会话之间消息那一半，给用户的回复里没有禁，所以这一行与新规则并存，不冲突。

什么现象会推翻问 1 的「成立」：规则文件第 66 行与共用约束第 41 行在任一项上写了相反的叫法；或 `ListAgents` 那一行根本不带编号（那时两处规则都没法照做）。

## 问 2：检查判的是不是规则说的那件事

判定：不成立（打中）。规则列的几类常见写法拦放都对；同一类里有合规写法被拦、该拦的写法被放过，各给能照喂的回复文本。

**钩子判了什么**（整行并排，行号都是 `grep -nF` 现取）：

- `.claude/hooks/session-number-stop-check.sh:69`：「LETTER_NAME = re.compile(r"mainagent[(（][A-Z][)）]")」
- `.claude/hooks/session-number-stop-check.sh:87`：「text = courtesy.FENCED_BLOCK.sub("", text)」
- `.claude/hooks/session-number-stop-check.sh:89`：「text = courtesy.QUOTED_SPANS.sub("", text)」
- `.claude/hooks/session-number-stop-check.sh:97`：「found = [(match.group(0), line.strip()) for line in judged_reply_text(reply, courtesy).splitlines() for match in pattern.finditer(line)]」
- `.claude/singlefs-ai-sop/scripts/courtesy-phrase.py:87`：「FENCED_BLOCK = re.compile(r"(?ms)^\s*(```|~~~).*?^\s*\1\s*$")」
- `.claude/singlefs-ai-sop/scripts/courtesy-phrase.py:88`：「QUOTED_SPANS = re.compile(r"」，整条原文：``QUOTED_SPANS = re.compile(r"`[^`\n]*`|「[^」\n]*」|“[^”\n]*”|\"[^\"\n]*\"")``

**成立的部分**（读源码得出，「推的」；真钩子读数待主 agent 跑，对应 `stop_check_cases.py` 里「正文里写」「全角括号」「列表项」「反引号」「中文引号」「三反引号代码块」「只写编号」「子 agent 字母」等例）：

| 写法 | 规则要的 | 钩子 | 依据 |
|---|---|---|---|
| 正文、列表项、表格、引用块、加粗里写 mainagent(A) | 拦 | 拦 | 第 97 行逐行找，不分行首记号；括号半角全角各自或混写都认（第 69 行的两个字符类） |
| 单个反引号、「」、弯引号、ASCII 双引号里的 | 放 | 放 | 第 88 行 `QUOTED_SPANS` 里反引号、「」、弯引号、ASCII 双引号各一支 |
| 成对的 ``` 或 ~~~ 围栏里的 | 放 | 放 | 第 87 行 `FENCED_BLOCK` |
| 只写 ListAgents 的编号 | 放 | 放 | 回复里没有第 69 行的写法 |
| 子 agent 收工 | 放 | 放 | 第 104 行 `return hook_input.get("hook_event_name") == "SubagentStop" or bool(hook_input.get("agent_id"))`；钩子只挂在 Stop 上（`.claude/settings.json:229`，`python3 -I` 读 JSON 列出的挂载事件只有 `Stop`，无 matcher），SubagentStop 的收工到不了它，第 104 行是兜底（第 18 行文件头写明） |

**不成立的部分：合规的引用被拦**（钩子自己的出路写的是「放进反引号或「」里」，`.claude/hooks/session-number-stop-check.sh:121`；下面四种都是合规的 markdown 引用，钩子放不过）。每个输入照这个形状喂：`{"hook_event_name":"Stop","session_id":"probe","last_assistant_message":"<回复文本>"}`，预期退 2；regex 一层量到了（上面 13 句的内联输出），钩子进程一层待主 agent 跑。

1. 两个反引号包的行内代码，回复文本 ``规则里的写法是 ``mainagent(A)``。`` → 内联输出 `double backtick block`。原因：`QUOTED_SPANS` 的反引号那一支 `` `[^`\n]*` `` 把两个紧挨的反引号当成一个空的行内代码吃掉，中间的 mainagent(A) 留在外面。
2. 跨行的行内代码（行内代码在 markdown 里可以断行），回复文本是「规则里的写法是 」加一个反引号、`mainagent(A)`、一个换行、一个反引号 → `multiline span block`；原因：`[^`\n]*` 不跨行。
3. 缩进四格的代码块（markdown 的另一种代码块）：`输出：\n\n    mainagent(A) 改了 x\n\n共 1 行。` → `indent code block`；`FENCED_BLOCK` 只认围栏。
4. 四个反引号的围栏里嵌三个反引号的围栏（写 markdown 样例时常见）：外层四个反引号，里面是三个反引号围栏、一行 x、再三个反引号，随后一行 mainagent(A) 改了 x → `nested fence block`；`FENCED_BLOCK` 的 `\1` 只回指三个字符，外层围栏被内层的 ``` 行提前截断。

后果：被拦一次，stderr 叫人放进反引号或「」里，模型在续跑里回到同一种写法时按第 19 行（续跑里同一份命中放行）通过；代价是一次多余的重写。

**不成立的部分：该拦的写法被放过**（文件头第 48 行已承认其中几种「管不到的」；这里补它没承认的几种，规则要的是用户看得出是谁，下面的写法用户照样看不出）：

| 回复文本 | 内联输出 | 原因 |
|---|---|---|
| `屏幕 5" 宽，mainagent(A) 改了 x.py，后文 "再说"。` | `odd quote pass` | 奇数个 ASCII 双引号，`QUOTED_SPANS` 的 ASCII 双引号那一支把尺寸标记的引号与后文的左引号配成一对，吃掉中间的 mainagent(A) |
| ``用 ` 转义，mainagent(A) 改了 x.py，另见 `y`。`` | `lone backtick pass` | 落单的反引号与后面的配成一对 |
| `Mainagent(A) 改了 x.py。`、`MainAgent(A) 改了 x.py。`、`main agent(A) 改了 x.py。`、`mainagent (A) 改了 x.py。` | 内联跑了 Mainagent(A)、main agent(A)、mainagent (A) 三句，都 pass；MainAgent(A) 没跑，按大小写敏感的第 69 行推 pass | 第 69 行只认小写、名字里无空格、括号前无空格的 mainagent |
| `mainagent(AB) 改了 x.py。`、全角字母的 `mainagent(Ａ)`、括号内换行的 `mainagent(` 换行 `A)` | 三句都 pass | 第 69 行是单个半角大写字母加收尾括号；文件头第 48 至 49 行已写 `mainagent(AB)`、`mainagent(a)` 与「换行断开」管不到 |

**续跑放行的指纹只看名字**：`.claude/hooks/session-number-stop-check.sh:111` 是 `    fingerprint = transcript.fingerprint_of([name for name, _ in hits])`，第 112 行是 `    repeated = hook_input.get("stop_hook_active") is True and transcript.stored_fingerprint(state_file) == fingerprint`。命中的是 mainagent(A) 这个串，不是那一行；所以被拦之后续跑里换了整句话、仍然写 mainagent(A)，也放行（探针里「续跑里同一份命中换了句子」那一例，待主 agent 跑，预期真钩子读 pass、规则要的是 block）。这是文件头第 19 行有意的取舍（防死循环），列在这里是因为它让「该拦的拦」打了折。另一条（推的）：状态文件只按 `session_id` 命名（第 110 行），而回复里没有命中时第 132 行 `if not hits or already_shown(courtesy, hook_input, hits):` 短路、不清旧指纹；所以同一个会话里更早一轮留下的指纹，会让之后别的 Stop 钩子拦下引起的续跑里再次出现的 mainagent(A) 直接放行。

什么现象会推翻问 2 的「不成立」：把上面四条合规输入喂真钩子全退 0、五类漏拦输入全退 2（`stop_check_cases.py` 的 `real=` 一列）。我预期相反，预期值是「推的」。

## 问 3：共用约束第 102 行 ㉒ 与检出钩子 ㉒ 的判法逐项对

判定：不成立（打中）。拦的主体对得上，但有三处文字与代码相反、几处漏写。对的是 `.claude/hooks/bash-command-detector.sh` 的 ㉒ 一段：文件头第 447 至 466 行、判法函数 `git_hook_bypass_reasons`（第 3578 行起）、`git_config_hooks_path_write`（第 3612 行起）、`hooks_path_is_installed_value`（第 3646 行起）、`git_hook_findings`（第 3764 行起）；装没装上的判法在 `.claude/hooks/lib_git_hooks.py`。

**逐项并排**（左列整句抄自 `.claude/agent-common.md:102`，右列抄自检出钩子文件头或判法行；「一致」是两边说的是同一件事）：

| 项 | 共用约束第 102 行 | 检出钩子 | 判 |
|---|---|---|---|
| 拦哪几个子命令的 `--no-verify` | `命令位置上的 `git commit` / `git push` / `git merge` / `git am` 带 `--no-verify`（长选项认唯一前缀的缩写）` | 第 3793 行 `reasons = git_hook_bypass_reasons(global_options, environment, subcommand if subcommand in …` 对这四个子命令；第 452 行写明 git merge 的 `--no-veri` 有歧义不算 | 一致（歧义那句文字没写，但「唯一前缀」已含） |
| `-n` | `` `git commit` 与 `git am` 带 `-n` `` | 第 3591 行 `elif token[0] == "short" and subcommand in ("commit", "am") and token[1] == "n":` | 一致 |
| core 段 | `` `git config --remove-section core` 与 `--rename-section core` `` | 第 3633 至 3643 行 `git_config_core_section_change`，文件头第 455 行补「唯一前缀的缩写与子命令写法」 | 一致（漏写缩写与子命令写法） |
| 出路与窗口 | `窗口外提交用环境变量 `SINGLEFS_COMMIT_OUTSIDE_WINDOW=1`（用户明说窗口外也要提交时才设，大文件照判）` | 第 3481 行 `HOOKS_BYPASS_WINDOW_VARIABLE = "SINGLEFS_COMMIT_OUTSIDE_WINDOW"`，第 3814 行起的说明 | 一致 |
| 钩子没装上 | `命令所在的仓自带进仓钩子目录、而 `core.hooksPath` 没指到它或钩子文件缺、没有执行位时的 `git commit` / `git push`` | 第 3797 行 `if subcommand in ("commit", "push") and git_hook_form_judged(subcommand, arguments):` 接 `git_hooks_not_installed`；问题表在 `lib_git_hooks.py` 的 `hooks_path_problems` | 一致，另见下面漏写 |
| 放行范围 | `没有进仓钩子目录的仓（临时仓、样本仓）、目录算不出或不存在、改了仓位置的放行。` | 第 461 行 `改了仓的位置的、命令所在的目录算不出或不存在的不判；别的仓（样本仓、临时仓：没有进仓钩子目录）不判`，判法第 3681、3687 行 | 一致 |
| 主 agent 与子 agent | `绕开 git 钩子的 `git commit` / `git push`，与 git 钩子没装上（主 agent 与子 agent 一样拒）` | 第 3768 行 `主 agent 与子 agent 一样判` | 一致 |
| 进仓钩子目录与上限 | 五份钩子名与 `SINGLE_FILE_SIZE_LIMIT_BYTES` | `.claude/hooks/lib_git_hooks.py:21` `HOOK_NAMES = ("pre-commit", "pre-merge-commit", "prepare-commit-msg", "pre-applypatch", "pre-push")`；`scripts/gate/large-file-check.py:83` `SINGLE_FILE_SIZE_LIMIT_BYTES = 104857600` | 一致；`ls scripts/gate/git-hooks/` 列出的就是这五份 |
| 拒绝说明里的出路命令 | `逐个文件 `gzip -n -9 -k <文件>`、`git rm --cached <文件>`、`git add <文件>.gz` 再提交` 与整套重做命令、含指向 blob、树的标签 | `scripts/gate/large-file-git-hook.py:43` 抄 `gzip -n -9 -k`，`:44` 抄 `git reset --soft <远端已有的最新提交>`，`:32` 抄「指向 blob、树的标签」，`:6` 抄「含 git merge 生成的合并提交、cherry-pick、revert、rebase 重放出来的提交」 | 一致 |

**对不上的三处（文字与代码相反，打中）**：

1. 「设成装对的值」放行的范围。文字把放行放在 `git config` 这一支的括号里：`` `git config` 设 core.hooksPath（设成安装脚本装的那个绝对路径的放行；`HUSKY=0` 这类别的工具的开关不算） ``，与前面单独列的 `` 命令行 `-c core.hooksPath=…` 或 `--config-env` ``、`` `GIT_CONFIG_KEY_<n>` 与 `GIT_CONFIG_PARAMETERS` 改 core.hooksPath `` 并列，按字面 `-c core.hooksPath=<装对的值>` 与 `GIT_CONFIG_KEY_<n>` 加 `GIT_CONFIG_VALUE_<n>` 都在被拒之列。代码放行它们：
   - `.claude/hooks/bash-command-detector.sh:3599`：「if not (word == "-c" and hooks_path_installed(value.partition("=")[2])):」
   - `.claude/hooks/bash-command-detector.sh:3605`：「if not hooks_path_installed(str(environment.get(f"GIT_CONFIG_VALUE_{key.group(1)}", ""))):」
   - 文件头第 456 行：`#        设成安装脚本装的那个值不算：`git config core.hooksPath <值>`、`-c core.hooksPath=<值>`、GIT_CONFIG_KEY_<n>=core.hooksPath 加 GIT_CONFIG_VALUE_<n>=<值>…`（行尾省略号是原文截断到命中为止，整行见文件）。
   - 只有 `--config-env`（值是环境变量名）与 `GIT_CONFIG_PARAMETERS`（git 内部拼法）无论值什么都拒，第 3597 至 3608 行。
   喂入：`git -c core.hooksPath=/home/fy5090/code/singlefs/scripts/gate/git-hooks commit -m x`，cwd 为仓根；文字说拒、代码预期放行（推的，`detector_claim_cases.py`「命令行 -c 设成装对的绝对路径」一例待主 agent 跑）。
2. 「绝对路径」。文字：`` `core.hooksPath` 写成主工作树里 `scripts/gate/git-hooks` 的绝对路径（`git config core.hooksPath <绝对路径>`）生效 ``。代码在设值那一刻把相对路径也当成装对的：`.claude/hooks/bash-command-detector.sh:3663`：「return os.path.realpath(expanded if os.path.isabs(expanded) else os.path.join(top, expanded)) in {os.path.realpath(path) for path in installed}」；而装没装上的判法把相对路径记成问题，`.claude/hooks/lib_git_hooks.py:73`：「return [f"core.hooksPath 是相对路径 {value}：」…（原文该行继续写到「要写成绝对路径」）。所以 `git config core.hooksPath scripts/gate/git-hooks` 这一句不被 ㉒ 拒，随后的 `git commit` 被「钩子没装上」拒——文字只说了拒的那一半。喂入：`git config core.hooksPath scripts/gate/git-hooks`，预期放行（推的，待主 agent 跑）。
3. git config 的动作范围。文字只写「设」：`` `git config` 设 core.hooksPath ``。代码拒设、加、删：文件头第 454 行 `git config 设、加、删 core.hooksPath`（读不算）；判法 `.claude/hooks/bash-command-detector.sh:3622`：「if flags & {"--unset", "--unset-all", "--replace-all", "--add"}:」，这四个即使目标值是装对的也拒（第 457 行：`--add、--replace-all、--unset 照拒`）。喂入：`git config --unset core.hooksPath`，预期拒（推的，待主 agent 跑）；按文字这句不在被拦的几件里。

**漏写（文字没说到、代码判的，不算相反）**：

- 识别范围。代码认别名（第 3480 行 `GIT_ALIAS_DEPTH = 5`，第 457 至 458 行 `别名照 `-c alias.<名>=<值>` 与配置…认成它指的子命令`，`!` 开头的 shell 别名不认）、`git -C`、cd、`bash -c`、命令替换、`nice` / `env` 前缀、`xargs` / `find -exec` / `flock` 启动器剥开之后的命令（第 449 至 450 行）；第 102 行一个字没提。读文字的人会以为 `git -c alias.ci=commit ci -n` 与 `echo x | xargs git commit -n` 不在内。喂入这两句，预期都拒（推的，探针里「别名认成 commit」「启动器里的 commit -n」两例）。
- 不判装没装上的几种形态：第 460 行 `--help、-h、--dry-run 的 git commit 与 --help、-h 的 git push 不起钩子，不判`，判法 `.claude/hooks/bash-command-detector.sh:3667` 的 `git_hook_form_judged`；文字没写。
- 装没装上的问题表里还有「没设」「写成相对路径」两项（`lib_git_hooks.py:73` 的相对路径一项与 `:67` 的「没设」一项），文字的「没指到它」只含前者的一部分。
- 题头只写 `git commit` / `git push`，文件头第 447 行与代码第 3475 行的题头是 commit、push、merge、am、config 五个；正文里 merge、am、config 都写了，题头是漏的。

什么现象会推翻问 3 的「不成立」：喂上面三句（`-c core.hooksPath=<装对的绝对路径>`、`git config core.hooksPath scripts/gate/git-hooks`、`git config --unset core.hooksPath`），真检出钩子读到 `-c` 与相对路径那两句拒、`--unset` 那句放。那样三处就反过来对上了。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问 1 两处规则与用户要的事、彼此、与别处 | 成立 | 规则文件第 66 行与共用约束第 41 行逐项一致；与写闸及检出钩子 ⑰ 管的进仓文字不打架（短号只在 `records/` 与判决里判） |
| 问 1 推不出一：编号来源 | 推不出（打中） | 规则说 ListAgents 那一行的编号，用户看到的是标题里的编号，让号与无默认名时两者可不同；我没有 ListAgents 工具，读不到那一行印什么 |
| 问 1 推不出二：已停会话 | 推不出（打中） | 不在列表里的会话没有那一行，规则没给别的写法 |
| 问 2 常见写法的拦放 | 成立（推的，真钩子读数待主 agent 跑） | 正文、列表、表格、全角括号拦；反引号、「」、双引号、围栏、只写编号、子 agent 收工放 |
| 问 2 合规引用被拦 | 不成立（打中） | 双反引号、跨行行内代码、缩进代码块、四反引号嵌三反引号围栏四种合规写法，正则一层量到被拦 |
| 问 2 该拦被放 | 不成立（打中） | 奇数个 ASCII 双引号、落单反引号配错对、首字母大写与带空格的名字、两个大写字母、全角字母、括号内换行；续跑指纹只看名字 |
| 问 3 拦的主体 | 成立 | `--no-verify`、`-n`、core 段、窗口出路、钩子没装上、放行范围、五份钩子名、上限常量、出路命令都逐项对上 |
| 问 3 放行的范围 | 不成立（打中） | 文字把装对的值放行挂在 `git config` 下；代码对 `-c` 与 `GIT_CONFIG_KEY_<n>` 加 `GIT_CONFIG_VALUE_<n>` 同样放行（`bash-command-detector.sh:3599`、`:3605`） |
| 问 3 绝对路径 | 不成立（打中） | 代码在设值那一刻放过解开后等于钩子目录的相对路径（`:3663`），随后的提交被 `lib_git_hooks.py:73` 判没装上 |
| 问 3 config 的动作 | 不成立（打中） | 文字只写设；代码拒设、加、删（`:3622`） |
| 问 3 漏写 | 推不出文字完整（打中） | 别名、启动器、`-C`、`--help` 与 `--dry-run` 不判、题头只写 commit 与 push |

## 没做什么

- 不替主 agent 采纳或出判决；不判攻方腿的第 4 至 7 问（门禁 `.claude/gate.d/code-tooling.sh` 的新判据、弄坏开关、红样本我没读，`.claude/settings.json` 只读了 Stop 上那一条的挂载）。
- 没往统一任务服务交任务；真钩子读数与整份探针没跑，命令在上面「要主 agent 跑的命令」一节；判决要引的数没复跑。
- 没读禁读清单里的任何文件；没起 `ListAgents`（我没有这个工具），问 1 两处推不出的验证要主 agent 做。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/session-number-naming-governance-r1-sonnet2` 只有一份改引文格式用的替换规格（20K，`du -sh` 现取）；没建仓副本与编译目录，没有要删的。

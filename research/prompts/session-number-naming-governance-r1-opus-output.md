# 攻方腿报告：session-number-naming-governance-r1（正文第 4 到 7 问）

日期：2026-11-19。读的是主工作区此刻那一版；开工核了开工快照 `judged-sha256.txt`：被判的五份里四份对得上，`.claude/agent-common.md` 对不上，`git diff` 看差在第 49 行（「仓根四份」改成「仓根五份」私有清场脚本），不是这一轮被判的第 41、102 行，模型也不读它。干到一半再核一次，`.claude/gate.d/code-tooling.sh` 也对不上了（主工作区有别的会话在改它，`git diff --stat` 是 98 行加、9 行删）；`git diff -U0` 里没有一行碰到 `command_invokes`、`stop_entries` 与 `session-number`，第 7 问判的那几行内容没变、只整体下移 43 行：开工快照里是第 3889、3891、4044、4052 行，下面引的是此刻的行号。复跑时 `name=judged_sha256_check` 会多打一行 `.claude/gate.d/code-tooling.sh: FAILED`，原因同此。

本报告里所有读数都是**推的，待主 agent 跑**：子 agent 不交任务，模型写好了、没跑。推的依据是读代码，正则与切词另用只含标准库的 `python3 -I -c` 照抄被判正则与 `shlex.split` 试过（那是我自己的复刻，不是被判钩子本身，不当量过）。

## 复跑命令与模型文件

模型目录 `research/prompts/session-number-naming-governance-r1-opus-model/`，复跑整份（`--frozen` 给仓根本身：模型只读被判的那几份，开头先拿开工快照的 `judged-sha256.txt` 在 `--frozen` 上逐份核 sha256 并打出来）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/session-number-naming-governance-r1-opus2 -- bash /home/fy5090/code/singlefs/research/prompts/session-number-naming-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/session-number-naming-governance-r1-opus2/probe-run /home/fy5090/code/singlefs/research/prompts/session-number-naming-governance-r1-opus-model
```

| 文件 | sha256 |
|---|---|
| `hook_probes.py` | `e7fa4d72c9533f649ecbdd2910f1e2ba4c6f74e8ec7b1c9dcebe9653e3fcca7a` |
| `gate_variants.py` | `04ff8c78e61e54741c77407d3f2abda8cc627b101017d3d81f008e8564c18b89` |
| `break_switches.py` | `96f02bed91aebaa1c8014aa711bf473a1ef195d80ae1e729deba6d2f2657be32` |
| `tools/rerun.sh` | `0270ac11b95058095473bda8d30178ef854d755423d4b3aa5b8f9e7a0332aa6d` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

模型做什么：
- `hook_probes.py`：照 Stop 钩子的 stdin 喂被判钩子（`bash .claude/hooks/session-number-stop-check.sh`），状态目录、检出记录、`TMPDIR` 全指到运行目录；每例一行 `name=… question=… rule_wants=… exit=… detections_added=… verdict=…`。对照三例（裸写字母必拦、反引号里必放、只写编号必放）不过就记 `control-broken`、整份退 1。另建两棵改过的钩子树（在运行目录里，`scripts`、`singlefs-ai-sop` 两个目录用链接指回被判树）：变异 `mutant-loop-guard`（去掉「只在续跑里」的条件）与我提的改法 `fix-forget-on-clean`（没命中时删掉这一会话的指纹）。
- `gate_variants.py`：从门禁绿样本 `agent-write-scope-green` 拷出 19 个变体，各改一处 `.claude/settings.json` 的 Stop 注册；`rerun.sh` 逐个跑 `code-tooling.sh --check agent-write-scope <变体>`（每行自带范围声明）；绿样本原样与删掉注册两格是对照。
- `break_switches.py`：读钩子文件头的开关表，对每个开关下自证实际红的格与点名的格逐格比；再看变异钩子的自证红不红。
- 判定：`verdict=hit` 是打中（读数），不是失败；`rerun.sh` 只在对照没过时退非 0。

估时与内存（推的，没量）：钩子大约 140 次起进程、每次两个 python 进程（preflight 与钩子本身），自证 13 次，门禁一格 19 次；合计几分钟，内存在几百 MB 以内。

## 各格判定一览

| 问 | 回答 | 打中的格（详见各节） | 读数状态 |
|---|---|---|---|
| 第 4 问 漏拦 | **成立**（造得出，打中） | 引号整句豁免（「」、双引号、反引号包住整句即放行，拦下时的出路原文还教这一招）；续跑的旧指纹放行一回合里头一次写的字母；文件头没声明的写法：`MainAgent(A)`、`mainagent (A)`、全角字母、`mainagent A`、`mainagent(**A**)`、转义括号、零宽空格 | 推的，待主 agent 跑 |
| 第 5 问 误拦 | **成立**（造得出，打中，代价轻） | 引用块里抄规则原文、代码块外单引号里的命令、双反引号行内代码、四反引号围栏、缩进代码块、『』与‘’、表格单元格 | 推的，待主 agent 跑 |
| 第 6 问 判不了的路与弄坏开关 | 列在文件头的三种判不了**记了**（不成立=没打中）；preflight 一步判不了（没有 python3、规范副本里的 preflight 缺或出错）退 78 或 1、**不记检出**（打中，共用前提）；十个开关**都红在点名格**（没打中）；自证漏钉「只在续跑里」这个条件，变异活下来（打中） | 推的，待主 agent 跑 |
| 第 7 问 门禁新判据 | **不成立**（判错，打中） | 绿而钩子不起作用：命令带 `--selftest`、`|| true`、解释器写 `python3` 或 `sh`、路径指到别处同名文件；红而注册有效：`cd … &&` 起头、`env bash`、变量赋值前缀、`bash -e`；挪到别的事件判红是对的；matcher、type、timeout、`disableAllHooks` 四种门禁不看，该不该红要 Claude Code 自己的语义，推不出 | 推的，待主 agent 跑 |

## 第 4 问：检查漏拦

回答：**成立**——造得出写了 mainagent 加字母、用户照样看不出是谁、钩子放行的回复。打中三类。

判法的三处原文：
- 「LETTER_NAME = re.compile(r"mainagent[(（][A-Z][)）]")」（`.claude/hooks/session-number-stop-check.sh:69`）
- 「text = courtesy.QUOTED_SPANS.sub("", text)」（`.claude/hooks/session-number-stop-check.sh:89`），去掉的是上游 `.claude/singlefs-ai-sop/scripts/courtesy-phrase.py:88` 那四种：反引号、「」、弯双引号、半角双引号里一行之内的内容。
- 「repeated = hook_input.get("stop_hook_active") is True and transcript.stored_fingerprint(state_file) == fingerprint」（`.claude/hooks/session-number-stop-check.sh:112`）

### 4a 引号整句豁免（打中，最实在的一格）

豁免认的是引号这个写法，不认里面是不是在引用。把整句话放进「」或双引号或反引号，钩子就放行，用户看到的照样是 mainagent(A) 改了什么。拦下时给的出路原文是「f"依据 {RULE_CITATION}。确实是在引用这种写法，放进反引号或」（`.claude/hooks/session-number-stop-check.sh:121`，转述其后半：放进反引号或「」里）——被拦回来的续跑照这句改，正好走进这条豁免。

能照喂的输入（`hook_probes.py` 的 `leak-whole-sentence-in-corner-quotes`，另有半角双引号、反引号两例与同一行里有一个散落的半角双引号的一例）：

```json
{"hook_event_name": "Stop", "session_id": "probe-leak", "stop_hook_active": false, "transcript_path": "", "last_assistant_message": "「mainagent(A) 改了 scripts/x.py，做了一次换上。」"}
```

推的结局：退 0。依据：上面第 89 行把「」整段去掉，剩下一个空行，第 69 行的正则没东西可配。
四句：不分辨臂这一说不适用（这一轮没有臂）；钩子判的那一刻看得到引号里的整段文字，分得出「只引写法本身」与「整句话」（不是判别子观测不到）；满足的是第 4 问字面「把整段放进引号」那一个分句；正文没给跑前改法，改法是我提的、见末尾表。

### 4b 续跑的旧指纹放行一回合里头一次写的字母（打中）

指纹只在有命中时写（「transcript.store_fingerprint(state_file, fingerprint)」，`.claude/hooks/session-number-stop-check.sh:113`），没命中的收工直接走「if not hits or already_shown(courtesy, hook_input, hits):」（`.claude/hooks/session-number-stop-check.sh:132`）返回、不清；指纹是命中名字列表的哈希（「return hashlib.sha256(json.dumps(listed, ensure_ascii=False).encode('utf-8')).hexdigest()」，`.claude/singlefs-ai-sop/scripts/session-transcript.py:155`），不带回合。于是同一会话里前面任何一回被拦过 mainagent(A)，之后只要有一次续跑（`stop_hook_active` 为 true，同挂 Stop 的客套话、未回消息、完工清理三道任何一道拦回来都算）里头一次写 mainagent(A)，就放行，这一回合钩子一次都没拦过。

历史（同一个 `session_id`，逐条喂；`hook_probes.py` 的 `seq-k0-midnone-finalA`）：

```json
{"hook_event_name": "Stop", "session_id": "seq", "stop_hook_active": false, "transcript_path": "", "last_assistant_message": "mainagent(A) 改了 scripts/x.py。"}
{"hook_event_name": "Stop", "session_id": "seq", "stop_hook_active": true, "transcript_path": "", "last_assistant_message": "(5b) 改了 scripts/x.py。"}
{"hook_event_name": "Stop", "session_id": "seq", "stop_hook_active": true, "transcript_path": "", "last_assistant_message": "已推。(5c) 与 mainagent(A) 都改了 y.py，各自换上了。"}
```

推的结局：2、0、0；同一末条在新会话里单喂退 2（对照 `seq-fresh-finalA`）。分辨它的只有前缀历史。
用户决定的步放开扫（照「判决由主 agent 做，不由投票做」里装置把用户动作写死那一条）：中间干净回合 0、1、3 条，中间有没有一回被拦的是 mainagent(B)，末条字母 A 或 B，共 12 段。推的读数：末条字母等于最后一次记下的指纹时放行（6 段），不等时拦（6 段）；中间回合数不影响。
四句：判别子观测不到——钩子此刻能看的只有 `stop_hook_active` 与一份不带回合的指纹，「上一次拦下的就是这一回合的上一次收工」与「是几回合以前的事」两种情形它看到的逐条相同；要改的是状态，见改法表。满足第 4 问字面「续跑绕过」那个分句。

另有一种设计如此、不算打中但照记：被拦之后续跑里写同一个字母、换整句话照样放行（`seq-designed-repeat`），文件头第 19 行写明命中的几处与上一次拦下时一字不差就放行——一字不差比的是 `mainagent(A)` 这个名字，不是那一行。

### 4c 文件头没声明的写法（打中）

文件头声明管不到的是「mainagent 后面不是单个大写字母的写法（`mainagent(AB)`、`mainagent(a)`）；换行断开的写法」（`.claude/hooks/session-number-stop-check.sh:49`）。下面几种不在这句里、钩子同样放行（推的，`hook_probes.py` 里各一例，回复正文都是「<写法> 改了 scripts/x.py，做了一次换上。」）：

| 例名 | 写法 | 用户看到的 |
|---|---|---|
| `leak-capital-M` | `MainAgent(A)` | 同一个叫法，大小写不同 |
| `leak-space-before-paren` | `mainagent (A)` | 同上，多一个空格 |
| `leak-fullwidth-letter`、`leak-fullwidth-parens-and-letter` | 括号里是全角 Ａ（输入法全角模式下括号与字母一起变全角） | 同上 |
| `leak-no-parens` | `mainagent A` | 同上 |
| `leak-bold-letter` | `mainagent(**A**)` | 渲染成 mainagent(A)，A 加粗 |
| `leak-markdown-escaped-parens` | `mainagent\(A\)` | 渲染成 mainagent(A) |
| `leak-zero-width-space` | main 与 agent 之间夹 U+200B | 与 mainagent(A) 一模一样 |
| `leak-chinese-main` | `主 agent(A)` | 同一个意思 |

文件头已声明的三种（小写字母、两个字母、换行断开）照喂、记 `hit-declared`，不算新打中。前四行最可能自然写出来；后几行要刻意才会写出，列着是为说明正则只认一种字面。

## 第 5 问：检查误拦

回答：**成立**——造得出合规的回复（没拿字母指别的会话，是在引用写法或贴命令）被拦。每一例都有一条现成出路（放进反引号或「」），代价是多一次重写，不是拦死；但它也说明钩子分不出「在引用」与「在指会话」，与 4a 是同一个病根的两面。

最直接的一例是引用规则原文：规则自己就把字母写在正文里，「会话之间的消息指别的会话写 mainagent(A)、mainagent(B)」（`.claude/rules/writing-discipline-by-review.md:66`）。用户问「规则怎么写的」，主 agent 用 Markdown 引用块把那一条抄给他，就被拦（`hook_probes.py` 的 `false-block-blockquote-rule`，引的那一行运行时从被判树现读）：

```json
{"hook_event_name": "Stop", "session_id": "probe-quote", "stop_hook_active": false, "transcript_path": "", "last_assistant_message": "规则原文：\n\n> 给用户的回复与汇报里指别的会话……会话之间的消息指别的会话写 mainagent(A)、mainagent(B)……\n\n所以给你的汇报里只写编号。"}
```

（上面省略号处模型里是整行原文；推的结局：退 2。引用块的 `>` 不在去掉的写法里。）

其余各例（推的结局都是退 2；用 `python3 -I -c` 照抄上游两条正则试过去掉之后还剩 `mainagent(A)`，那是复刻、不当量过）：

| 例名 | 回复里的写法 | 为什么剩下 |
|---|---|---|
| `false-block-command-single-quotes` | 跑了 grep -nF 'mainagent(A)' .claude/agent-common.md，命中 1 行（命令没进反引号） | 单引号不在去掉的写法里 |
| `false-block-double-backtick-code` | 行内代码写成双反引号包住 `mainagent(A)`（Markdown 合法，内容里要有反引号时就这样写） | 上游反引号正则把两个反引号配成一对空串，中间那段露出来 |
| `false-block-four-backtick-fence` | 四个反引号的围栏里套一个三反引号围栏（贴含围栏的原文时的标准写法） | 上游围栏正则「FENCED_BLOCK = re.compile(r"(?ms)^\s*(```|~~~).*?^\s*\1\s*$")」（`.claude/singlefs-ai-sop/scripts/courtesy-phrase.py:87`）在里层的三反引号收尾，里层正文露出来 |
| `false-block-indented-code-block` | 缩进四格的代码块 | 只认围栏 |
| `false-block-white-corner-quotes`、`false-block-curly-single-quotes` | 『』、‘’ | 不在去掉的写法里 |
| `false-block-path-outside-code` | 正文里写一条路径，文件名里带这个写法 | 路径没进反引号（例子是造的，仓里 `git ls-files` 没有带这个写法的路径，0 条） |
| `false-block-table-cell-rule-summary` | 表格一格写「会话之间的消息」、旁边一格写这个写法（在讲规则） | 表格不去掉 |

四句：判别子看得到（钩子拿到的是全文，引用块、表格、单引号都在）；满足第 5 问字面「引用规则原文」「代码块外的路径或命令里含这种写法」两个分句；围栏与引号的读法来自上游 `courtesy-phrase.py`，文件头第 8 行写明它「在上游副本里不许就地改」，所以四反引号、双反引号两格的病根在共用前提里（客套话检查同样会误拦这两种），是「打中不分辨」那一类，改要在这个钩子里自己另写读法。

## 第 6 问：判不了的路记没记检出；弄坏开关红不红在点名格

回答分三格。

### 6a 文件头列的三种判不了：记了（没打中）

文件头第 21 行列的是输入不是 JSON 对象、取不到回复、判的时候抛异常三种（转述）。读 `judge_input` 与 `unjudged_pass`：三种都走 `lib_work_root.record_unjudged_pass`、结局「只记」、退 0；自证里各有一格。我另造了五种同类输入，推的结局都是退 0、检出多一行（`hook_probes.py` 的 `unjudged-*` 与 `state-directory-unwritable`）：空 stdin、非 UTF-8 字节（`UnicodeDecodeError` 是 `ValueError` 的子类，进同一个分支）、`last_assistant_message` 是数而没有会话记录、`transcript_path` 指到目录、会话记录里没有回复。状态目录建不成时 `store_fingerprint` 自己吞掉 `OSError`，命中照拦（退 2），不进判不了的路。

### 6b preflight 一步判不了：不记检出（打中，共用前提）

钩子进 python 之前先过「source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"」（`.claude/hooks/session-number-stop-check.sh:54`）。规范副本的 preflight 在 PATH 上找不到 python3 时退 78（「if ! command -v python3 >/dev/null 2>&1; then」，`.claude/singlefs-ai-sop/scripts/preflight.sh:34`，同一段第 42 行 `exit 78`）；`preflight.py` 自己出错退 1；规范副本不在时项目那一份垫片定义一个退 1 的 preflight。三种都在 python 之前退出，`record_unjudged_pass` 没机会跑；Claude Code 对 Stop 钩子退 0、2 之外的码照常收工（这是我印象里的 Claude Code 行为，仓里没有文档可核，推的）。所以这一道「放行」了、却没记检出。

能照喂的输入（`hook_probes.py` 的 `unjudged-preflight-no-python`：PATH 只放 bash 与 dirname，喂一条写了字母的 Stop 输入）：推的结局退 78、检出多 0 行。
四句：打中不分辨——每个 source 了 preflight 的钩子都是这样，病根在共用的 preflight 退出约定，不是这一轮的改动；钩子此刻看得到自己没跑成（它就是那个进程），但约定让它在记检出之前就退出；满足第 6 问字面「判不了的路放行并记检出」的「记」那一半；没有 python3 时 python 写的检出本来也记不成，真要记得在 shell 里写一行，交主 agent 定要不要另立一题。现实里最可能碰到的是 `preflight.py` 自己被改坏（退 1）这一种。

### 6c 十个弄坏开关：都红在点名格（没打中）；自证漏钉一个条件（打中）

逐个开关对着自证样本推了一遍：每个开关打开后红的格与文件头第 27 到 47 行点名的逐格相同，没有红在别处、也没有该红不红的（推的；`break_switches.py` 逐个开关跑自证、逐格比，读 `name=switch-<开关> … extra=- missing=- verdict=ok`）。与背景材料里主 agent 报的「逐格相同 10 条」一致。

缺的是一格：续跑放行的条件写的是「`stop_hook_active` 为 true **并且** 指纹相同」，自证里没有一例能分出这两半。把前一半删掉（`mutant-loop-guard`：任何一次收工，命中与上一次记下的指纹相同就放行），推的结局是自证照样全过——自证里同一个会话 `s-loop` 头一次喂时没有旧指纹，第二次喂本来就是续跑，没有「不是续跑、指纹却相同」的那一例。变异下的钩子在下面这段历史上放行第二条（规则要拦），原钩子拦：

```json
{"hook_event_name": "Stop", "session_id": "cross-turn", "stop_hook_active": false, "transcript_path": "", "last_assistant_message": "mainagent(A) 改了 x.py。"}
{"hook_event_name": "Stop", "session_id": "cross-turn", "stop_hook_active": false, "transcript_path": "", "last_assistant_message": "mainagent(A) 又改了 y.py。"}
```

推的读数：`loop-guard-cross-turn-original` 第二条退 2，`loop-guard-cross-turn-mutant` 第二条退 0，`selftest-mutant-loop-guard` 退 0。这不是开关红错格，是开关表与自证都没钉住这个条件；改法是自证加上面这两条作一例，再加一个开关（例如去掉续跑条件）点名它。

## 第 7 问：门禁那条新判据在注册挪地方、换命令形态时判得对不对

回答：**不成立**——判错的形态造得出，两个方向都有。

判据原文：「and not any(command_invokes(hook.get("command"), "session-number-stop-check.sh") for entry in stop_entries for hook in entry.get("hooks") or [])):」（`.claude/gate.d/code-tooling.sh:4095`），`stop_entries` 只取 `hooks.Stop`（`.claude/gate.d/code-tooling.sh:4087`）。`command_invokes` 按 shell 切词之后只看第一个词，第一个词是四种解释器之一时看第二个词：「if len(words) > 1 and os.path.basename(words[0]) in ("bash", "sh", "python3", "env"):」（`.claude/gate.d/code-tooling.sh:3932`）、「return bool(words) and (words[0] == script or words[0].endswith("/" + script))」（`.claude/gate.d/code-tooling.sh:3934`）。它不看后面还有什么词、不看 `type`、`timeout`、`matcher`，也不看 settings 顶层。

`gate_variants.py` 从绿样本拷出变体，只改 Stop 里那一条；钩子一侧的同一组命令另在 `hook_probes.py` 的 `form-*` 里照 Claude Code 起钩子的样子跑（`/bin/sh -c`、工作目录是项目根、`CLAUDE_PROJECT_DIR` 指仓根；这个起法是我印象里的，推的），喂一条写了字母的与一条干净的，看是不是「2/0」（拦该拦的、放该放的）。

| 变体 | Stop 那一条的命令或改动 | 门禁（推的） | 钩子一侧（推的） | 判得对不对 |
|---|---|---|---|---|
| `green` | 原样 | 绿 | 2/0 | 对照 |
| `removed` | 删掉 | 红、点名 | — | 对照 |
| `moved-to-subagentstop`、`moved-to-userpromptsubmit` | 挪到别的事件 | 红、点名 | — | 对 |
| `mention-only-echo` | `echo .claude/hooks/session-number-stop-check.sh` | 红、点名 | — | 对 |
| `selftest-argument` | 原命令末尾加 `--selftest` | 绿 | 0/0（每次收工跑一遍自证） | **错：绿而钩子不判** |
| `or-true` | 原命令末尾加 `\|\| true` | 绿 | 0/0 | **错** |
| `python3-interpreter` | `python3 "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh` | 绿 | 1/1（python 读 bash 脚本语法错） | **错** |
| `sh-interpreter` | `sh "$CLAUDE_PROJECT_DIR"/…`（本机 `/bin/sh` 指向 dash，`ls -l /bin/sh` 量的） | 绿 | 2/2 或 127（dash 不认 `${BASH_SOURCE[0]}`），每次收工都拦或都放 | **错** |
| `other-file-same-name` | `bash "$CLAUDE_PROJECT_DIR"/.claude/gate.d/fixtures/stub/session-number-stop-check.sh` | 绿 | 127/127（文件不在） | **错**：只比结尾的文件名 |
| `cd-then-relative` | `cd "$CLAUDE_PROJECT_DIR" && bash .claude/hooks/session-number-stop-check.sh` | 红 | 2/0 | **错：注册有效却红** |
| `env-bash` | `env bash "$CLAUDE_PROJECT_DIR"/…` | 红（剥掉 env 后第一个词是 bash） | 2/0 | **错** |
| `assignment-prefix` | `LC_ALL=C.UTF-8 bash "$CLAUDE_PROJECT_DIR"/…` | 红 | 2/0 | **错** |
| `bash-option` | `bash -e "$CLAUDE_PROJECT_DIR"/…` | 红 | 2/0 | **错** |
| `direct-exec` | `"$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh`（钩子有执行位） | 绿 | 2/0 | 对 |
| `stop-with-matcher` | Stop 那一条加 `"matcher": "Bash"` | 绿 | — | 推不出：要 Claude Code 对 Stop 上 matcher 的语义（我印象里 Stop 不用 matcher、写了也照触发，没核） |
| `type-prompt` | `type` 改成 `prompt`、`command` 照留 | 绿 | — | 推不出（推的：Claude Code 按 type 起，`command` 字段不会被跑，那就是错） |
| `timeout-one-second` | `"timeout": 1` | 绿 | — | 推不出：一秒内两个 python 进程起不起得完没量 |
| `disable-all-hooks` | settings 顶层 `"disableAllHooks": true` | 绿 | — | 推不出（推的：全部钩子都不跑，那就是错；这一条不单属这条新判据，本格别的注册判据同样不看） |

「对照」两格不过整份记装置失效（`gate_variants.py summarize` 打 `control-broken`、退 1）。

四句：打中不分辨——`command_invokes` 是这一格所有注册判据共用的（写范围闸、检出钩子、提醒停止钩子等十几条都用它），这些形态对别的判据同样判错，不是这一次新加的那几行独有；门禁判的那一刻看得到整条命令与 `type`、顶层字段（不是判别子观测不到）；满足第 7 问字面「写成别的命令形态时」那个分句；「挪到别的 matcher」那一半（挪到别的事件）判得对，Stop 上带 matcher 推不出。

现实里哪几种会出现：都要有人改 settings.json 才会出现，这一份注册由工具员写、改动走三方；最可能的是为了「先别拦」临时加 `|| true` 或把命令改成先 `cd` 再相对路径。前者门禁放过、后者门禁误红。红样本只有「删掉」一份，没有任何一份覆盖「提到了但不跑」以外的形态。

补注：第 7 问四句里说的共用，量的是 `grep -c 'command_invokes(' .claude/gate.d/code-tooling.sh` 得 18（含定义那一行与 `mention_only_commands` 里的一处），点名的钩子 15 个（`grep -o` 后去重数的）。

## 改法与各修哪一格

下面的改法都是我提的：**只在我的模型上量过、被攻过零轮**；这一份表里没有一格是量过的——唯一写进模型的改法 `fix-forget-on-clean` 也还没跑，标「推的，待主 agent 跑」。

| 改法 | 修哪一格 | 状态 |
|---|---|---|
| 回复里没命中时删掉这一会话的指纹（`fix-forget-on-clean`，`hook_probes.py` 的 `CLEAN_PASS_FIX` 与 `FORGET_DEFINITION`；只动 `decide` 的没命中分支，加一个删状态文件的函数） | 4b：12 段扫描里放行的 6 段改成拦；「被拦后续跑里同一份命中不连拦」照旧（`fix-seq-designed-repeat` 照放）；自证照过 | 推的，待主 agent 跑（读 `fix-seq-*` 各行全是 `verdict=ok`，`selftest-fix-forget-on-clean` 退 0） |
| 引号豁免只放过「整段引号里只有这个写法本身」（引号里去掉空白后整段等于 `mainagent(X)`），引号里还有别的字的照判；拦下时的出路改成「只把写法本身放进反引号」 | 4a；5 的引用块、表格两例不受影响（照样误拦） | 推的 |
| 正则放宽：不分大小写、`mainagent` 与括号之间允许空白、括号里认全角字母、先去掉零宽字符与 Markdown 转义与加粗记号再判；另认 `mainagent A`、`主 agent(A)` | 4c 各行 | 推的；放宽之后 5 的误拦面也跟着变大 |
| 钩子自己写一份围栏与行内代码的读法：围栏按开头的反引号个数配收尾（三个以上），行内代码按反引号串长度配对，认缩进代码块；引号另认『』与‘’ | 5 的四反引号、双反引号、缩进代码块、『』‘’ 五例 | 推的（上游 `courtesy-phrase.py` 不许就地改，只能在这个钩子里另写，与「两处读法一份」的现行写法相反，交主 agent 定） |
| 自证加一例：同一会话两次平常收工（都不是续跑）写同一个字母，第二次要拦；开关表加一个去掉续跑条件的开关点名它 | 6c | 推的；模型里 `loop-guard-cross-turn-*` 两行是这一例的雏形 |
| 门禁这一条改成：命令切词后去掉前置的变量赋值、`env`、`cd <目录> &&` 与 bash 的选项，剩下必须恰好是 `bash` 加钩子路径、后面不许再有词；路径必须等于项目里那一份（`$CLAUDE_PROJECT_DIR/.claude/hooks/…` 或相对项目根）；`type` 必须是 `command`；settings 顶层 `disableAllHooks` 为 true 判红；红样本按上表「错」的各行各加一份 | 7 的九个「错」；四个推不出的里 type、disableAllHooks 两个 | 推的；`command_invokes` 是共用的，改它会同时改十几条判据的判法，交主 agent 定只改这一条还是一起改 |
| preflight 判不了时在 shell 里记一行检出（不靠 python） | 6b | 推的；改的是上游共用约定，不在这一轮的范围里，交主 agent 定要不要另立一题 |

## 没打中的形状

- 第 4 问：试了文件头没声明的写法 9 种、引号整句 3 种加散落引号 1 种、续跑历史 12 段（中间干净回合 0、1、3 条 × 中间有没有被拦的 B × 末条 A 或 B）加 2 段对照、设计如此的重复 1 段。没打中的（按代码推的；模型只跑正文一种，列表项在钩子自己的自证里）：全角括号与半角字母混写（`mainagent（A)` 这种，正则左右括号各自认全半角，照拦）；写在列表项、表格、标题、链接文字里（正文不去掉这些，照拦）；子 agent 收工的放行（只注册在 Stop，`SubagentStop` 到不了它，兜底分支对，`.claude/hooks/session-number-stop-check.sh:104`）。
- 第 5 问：试了 9 种，全部误拦；没找到「合规而被拦、又没有现成出路」的一例。
- 第 6 问：模型里判不了的输入写了 6 种（含文件头三种之外的空 stdin、非 UTF-8、回复不是字符串、会话记录是目录、会话记录里没有回复），都记了；状态目录写不了照拦。没找到 python 里漏接的异常：`decide` 整个包在 `except Exception` 里，`SystemExit` 只可能来自载入上游脚本，读了 `load_package_module` 不调 `sys.exit`。
- 第 7 问：挪到别的事件、只提到路径两种判得对。

## 这条腿自己的限度

- 读数一条都没跑：子 agent 不交任务，全部「推的，待主 agent 跑」；推的依据是读代码，正则与切词另用标准库照抄试过，那是复刻，不是被判钩子。
- Claude Code 自己的语义没核：Stop 输入里有没有 `last_assistant_message`、主 agent 的 Stop 输入会不会带 `agent_id`（会的话这道检查对主 agent 一律放行）、退 0 与 2 之外的码怎么处理、钩子命令用哪个 shell 起、Stop 上的 matcher 与 `type`、`timeout`、`disableAllHooks` 怎么起作用，都是我印象里的；仓里没有可核的文档。第 7 问四个推不出与 6b「照常收工」那半句靠这些。
- 只攻了第 4 到 7 问；第 1 到 3 问（规则说的是不是用户要的、两处是否一致、第 102 行 ㉒ 与代码对不对得上）不判。
- 4a 与 5 是同一个设计取舍（豁免按引号写法认）的两面，改一边会动另一边；改法表里那一条只修 4a，没量它对 5 的副作用。
- `rerun.sh` 里模板自带的 `--selftest` 跑的是模板那一段 `rerun_steps`，换成这一腿的步骤之后它的自证不再成立；复跑只用 `--repo …` 那一种起法。

## 要主 agent 跑的命令

1. 模型整份复跑（第 4 到 7 问的全部读数都从这一条出）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/session-number-naming-governance-r1-opus2 -- bash /home/fy5090/code/singlefs/research/prompts/session-number-naming-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/session-number-naming-governance-r1-opus2/probe-run /home/fy5090/code/singlefs/research/prompts/session-number-naming-governance-r1-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/session-number-naming-governance-r1-opus2`（即 `--cwd`）。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 算装置跑通（对照全过）；`exit=1` 而日志里有 `name=rerun_verdict result=control-broken` 是装置失效，读 `verdict=control-broken` 那几行。日志（`task-server/task log <号> --all`）里：`name=judged_sha256_check` 之后应只有 `.claude/agent-common.md: FAILED` 与 `.claude/gate.d/code-tooling.sh: FAILED` 两行不对（原因见开头）；被判钩子与 `settings.json` 对不上的，读数不算这一版；`name=summary probes=… hits=… control_broken=0`；第 4 问读 `question=4` 与 `question=4-continuation` 各行的 `verdict=hit`（`fix-seq-*` 各行应是 `ok`）；第 5 问读 `question=5`；第 6 问读 `question=6`、`name=switch-* … verdict=ok`、`name=selftest-mutant-loop-guard … verdict=hit`、`loop-guard-cross-turn-*`；第 7 问读 `name=gate-* … verdict=`（`green`、`removed` 两行应是 `control-ok`）与 `question=7-hook-side` 各行。逐项结局与本报告各节「推的结局」对不上的，以复跑为准，本报告那一句作废。

## 没做什么

- 不判正推腿的第 1 到 3 问；不替主 agent 采纳；改法都在我的模型里，被攻过零轮。
- 没往统一任务服务交任务，判决要引的数一条都没复跑（命令在上一节）；本报告里的读数全是推的。
- 没改被判的任何文件；模型只写在模型目录与草稿目录。
- 草稿目录里只有三份草稿文本（`rerun-steps.txt`、`fix-builder.txt`、`old-builder.txt`，拼 `rerun.sh` 与 `hook_probes.py` 时用的），没有仓副本、编译目录与库拷贝；复跑会在草稿目录下建 `probe-run`，里面两棵改过的钩子树只含一份钩子、一份 `lib_work_root.py` 与指回仓里的两个目录链接，不是仓副本，跑完由主 agent 照用完就删处理。

引文核对（`python3 scripts/agents/cite-check.py research/prompts/session-number-naming-governance-r1-opus-output.md`，交回前在此刻的主工作区跑）末行：

```
  ✓ 核了 12 处引文、6 处裸行号，对上 12 处，没判 6 处（其中裸行号没有引文可比 6 处；逐处列在上面）
```

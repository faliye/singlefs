# no-task-numbers-governance-r3 正推腿报告（2026-11-19）

三方腿：正推（`three-way-forward`），sonnet。背景 `research/prompts/_no-task-numbers-governance-r3-body.md`。被判文件都在主工作区现读，没有开工快照倒推。
没有写模型目录；只写了一份只读探针 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-sonnet/probe_items.py`（sha256 `35d2c25e4eab625961e2f366ae1171fa954952c05f34db275a0fc532ca15627d`，`sha256sum` 量的），没有跑，命令在「要主 agent 跑的命令」一节。
下面引的行号都是这一次用 `grep -nF` 对主工作区文件命中后取的；各会话在改的文件行号会动，引的是此刻的样子。

## 关 1：三处新文字逐句对用户定案原话

用户原话取自 `records/2026-11-19-进仓文字不写任务号与会话短号.md` 里引的那几处：
- 第 5 行：「这些任务编号 对之后的任务追溯几乎 起不到什么作用」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:5`）、弹窗选「号一律不写」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:5`）。
- 第 25 行：「正文里的批号自己会话的也不写，只留依赖表批次那一列」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:25`，记录的转述，原话在同一行引号里）。
- 第 33 行：「用户选「现在换上，冻结证据豁免」」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:33`，是弹窗选项名，不是用户打的字）。
- 第 34 行：原话「改为agent(A) agent(B) agent(C) 这种」「汇报与进仓文字都改」「这不是偏好 而是 规范」（三句各在 `records/2026-11-19-进仓文字不写任务号与会话短号.md:34`；「或者 mainagent(A)」也在这一行）。
- 第 36 行：「不要影响任务的调度记录」「如果影响的话，不影响就现在这样可以」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:36`）。
- 第 37 行：「不对 这里可以不用改了未 agent(<短号>) 。 每个任务调度日志里面有调度记录」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:37`）。

### 1.1 共用约束「写」一节那一条（`.claude/agent-common.md` 第 40 至 45 行）

| 句 | 对原话 | 判定 |
|---|---|---|
| 「不拿号、不拿描述指东西，写那件事本身」（`.claude/agent-common.md:40`） | 原话只点了号与 `里程碑四收口的那一路会话回` 这类指会话的描述；「指东西」不限会话。 | 推不出：文字比原话宽，原话没说别的东西也不能用描述指。没有判法对应，不挡任何写入。 |
| 「别的会话与子 agent 不写会话全名（`singlefs-<两位>`）、两位短号」（`.claude/agent-common.md:41`） | 第 5 行原话只说会话短号，全名由会话号形态一直在判。 | 一致。 |
| 「也不写 `<做什么>那一路的会话`、`那一路的 agent` 这类描述」（`.claude/agent-common.md:41`） | 第 34 行原话的例子是 `里程碑四收口的那一路会话回`，叫人「统统不要写」。 | 一致；但「这类描述」比判据库宽，见 1.4。 |
| 「同一份文字里按出场次序叫别的会话 mainagent(A)、mainagent(B)、mainagent(C)」（`.claude/agent-common.md:41`） | 原话「或者 mainagent(A)  mainagent(C)  mainagent(B) 或者 subagent(A)…」。 | 一致。 |
| 「一律照这个叫法，不换别的说法」（`.claude/agent-common.md:41`） | 原话先给 `agent(A)`，再用「或者」给 mainagent / subagent 两组，末句「这不是偏好 而是 规范」。 | 推不出：原话没说只许 mainagent / subagent 这两组、不许 `agent(A)`；「不换别的说法」是主 agent 取了其中一组。 |
| 「给用户的汇报与会话之间的消息照同一条」（`.claude/agent-common.md:41`） | 弹窗选项是「汇报与进仓文字都改」；「会话之间的消息」只出现在记录第 34 行主 agent 自己的转述里（「进仓文字、给用户的汇报与会话之间的消息里」，`records/2026-11-19-进仓文字不写任务号与会话短号.md:34`）。 | 推不出：弹窗选项原文不在记录里，查不到用户有没有说会话之间的消息；第 36 行用户那句讲的是调度记录里的协作，不是这一条。 |
| 「任务服务的任务号不写，写交给服务的那条命令（不带号）、结局行的 `type=` 与 `exit=`」（`.claude/agent-common.md:42`） | 第 5 行「号一律不写」。 | 一致。 |
| 「判决（`research/prompts/*-main-verification.md`）里零轮改法那一条下面行首的「A/B 读数：任务 <号>」一行」与「`records/` 下调度记录里以「长跑拆分：」起头的那一行」（都在 `.claude/agent-common.md:42`） | 第 5 行起头：规则点名要号的两处保留，就是 `three-way-inference.md` 的读数与 `long-run-split.md` 第 6 步。 | 一致：只放这两处；现读 `.claude/rules/three-way-inference.md:38` 的「判决「改法」一节那一条下面单起一行，行首写「A/B 读数：任务 <号>」」，`.claude/rules/long-run-split.md:18` 的「以「长跑拆分：」起头」，写法相同。 |
| 「批号不写（「批 N」……自己会话的也一样……）」与「照写批号的只有 `records/` 下调度记录依赖表的批次那一列」（`.claude/agent-common.md:43`） | 第 25 行：自己会话的批号也不写，只留依赖表批次那一列。 | 一致。 |
| 「派发提示里的「批次：<调度记录>#<批号>」不进仓」（`.claude/agent-common.md:43`） | 与原话不冲突。 | 一致；但它同时说明派发提示里批号照写，这与 1.2 沟通纪律那一句相撞。 |

### 1.2 沟通纪律那一条（`.claude/rules/writing-discipline-by-review.md` 第 66 行）

整句：「指别的会话写 mainagent(A)、mainagent(B)，指子 agent 写 subagent(A)、subagent(B)，同一段话里各按出场次序编字母，写它改了哪个文件、做了什么；不写会话名、两位短号与 `<做什么>那一路的会话` 这类描述，任务号与批号也不写（进仓文字的同一条在」（`.claude/rules/writing-discipline-by-review.md:66`），后接「`.claude/agent-common.md`「写」一节）。」

| 点 | 判定 | 证据 |
|---|---|---|
| 「任务号与批号也不写」放在对话规范里 | 成立（多禁，与别处不一致） | 这一节管「管 agent 对用户、agent 与 agent 之间的对话」（`.claude/rules/writing-discipline-by-review.md:56`）。用户定案里号一律不写的射程是「进仓的文字」（第 5 行）；汇报与会话之间的消息只在 mainagent / subagent 叫法那一件里出现（第 34 行）。而共用约束自己要求派发提示带批号与任务号：「派发提示里的「批次：<调度记录>#<批号>」不进仓」（`.claude/agent-common.md:43`）、「并指到任务号与输出路径」（`.claude/agent-common.md:63`）；派发骨架打出批次行（`scripts/agents/dispatch-prompt.py:468`：「lines.append(f"批次：{batch or '<填 调度记录#批号>'}")」）。照沟通纪律字面，主 agent 对子 agent 的派发消息就不能写批次与任务号，与骨架和共用约束相撞。 |
| 「同一段话里」 | 成立（两份规范说法不一） | 共用约束写「同一份文字里」（`.claude/agent-common.md:41`），闸的拒绝说明同样写「同一份文字里按出场次序编字母，写它改了哪个文件、做了什么」（`.claude/hooks/write-guard.sh:760`）。沟通纪律写「同一段话里」（`.claude/rules/writing-discipline-by-review.md:66`）。一条消息有几段时，字母按整条消息编还是每段重编，两份写得不同；每段重编会让同一条消息里两个 mainagent(A) 指不同的会话。 |
| 「会话名」 | 一致 | 共用约束的「会话全名（`singlefs-<两位>`）」是同一件事。 |
| 缺 `那一路的 agent` 与「不换别的说法」 | 轻：说法不一 | 共用约束点名了 `那一路的 agent` 与「一律照这个叫法，不换别的说法」；沟通纪律只写 `<做什么>那一路的会话` 这类描述，没有后两样。 |

### 1.3 派发闸零轮读数那一判的拒绝说明（`.claude/hooks/runner-dispatch-guard.sh` 第 2713 至 2718 行）

| 句 | 对原话 | 判定 |
|---|---|---|
| 「→ 规矩：这一行要原样抄进判决的零轮改法下面。」（`.claude/hooks/runner-dispatch-guard.sh:2714`） | 第 5 行：两处保留之一。 | 一致。 |
| 「判决里放过的只有行首「A/B 读数：任务 <号>」一行里按「任务 <号>」写的任务号，值里至多」（`.claude/hooks/runner-dispatch-guard.sh:2714`） | 与共用约束同一件事；但判据库放过的比这句宽。 | 成立（轻，文字窄于判法）：`.claude/gate.d/lib-forbidden-notations.py` 与派发闸共用的任务号认法是「ZERO_ROUND_TASK_NUMBER = re.compile(r"(?:任务(?:号)?|task)[ \t#：:]*\d+", re.I)」（`.claude/gate.d/lib-forbidden-notations.py:333`），所以 `A/B 读数：task <号>`、`A/B 读数：任务号：<号>` 判决里同样放过，说明只说「按「任务 <号>」写的」。 |
| 「同一行里另带的别的写法的号（task=<号>、task-server/task log <号>、同 <号>、<号>）与多出来的号」（`.claude/hooks/runner-dispatch-guard.sh:2715`） | 无对应原话（出路说明）。 | 推不出（轻，含糊）：最后一个「<号>」没说是哪种写法；判据库拒的是带「任务」「task」「同」的号与并列连着的号，裸号不判（`.claude/gate.d/lib-forbidden-notations.py:108` 的「管不到的：没带「任务」「task」「同」的裸任务号」）。 |
| 「→ 怎么办：把那一行改成「A/B 读数：任务 <号>，<量到什么>」，两臂各写一个「任务 <号>」」（`.claude/hooks/runner-dispatch-guard.sh:2717`） | 同上。 | 一致。 |
| 「结局写 type= 与 exit= 那一段，命令写成不带号的样子」（`.claude/hooks/runner-dispatch-guard.sh:2717`）与「日志里那一行只抄那一行的字、不抄日志路径」（`.claude/hooks/runner-dispatch-guard.sh:2718`） | 第 5 行「号一律不写」。 | 一致，与 `.claude/agent-common.md:42` 同一句话。 |
| 三条认法（行首、读数行、任务号）与判据库逐字相同 | 无原话。 | 一致：`LINE_START = r"^[ \t>*_\-]*"` 在 `.claude/hooks/runner-dispatch-guard.sh:1062` 与 `.claude/gate.d/lib-forbidden-notations.py:331` 相同；读数行正则两份相同（`.claude/hooks/runner-dispatch-guard.sh:2675`、`.claude/gate.d/lib-forbidden-notations.py:332`）；任务号正则两份相同（`.claude/hooks/runner-dispatch-guard.sh:2676`、`.claude/gate.d/lib-forbidden-notations.py:333`）。 |

### 1.4 规则文字与判据库判法是不是同一件事

| 点 | 判定 | 证据 |
|---|---|---|
| 「这类描述」「不拿描述指东西」对判据行 `session-descriptions` | 成立（规则宽、判法窄，头注没列这一条缝） | 判据行只认「那一路」后面（可隔「的」）紧跟会话词（`.claude/gate.d/lib-forbidden-notations.py:87`：「- session-descriptions：拿「<做什么>那一路」的描述指别的会话或子 agent」；正则 `.claude/gate.d/lib-forbidden-notations.py:313`）。规则写的是「`那一路的 agent` 这类描述」（`.claude/agent-common.md:41`）。不带「那一路」的描述（例：`负责收口的那个会话先改了 a.md`）判据库放过，「管不到的」那一段（`.claude/gate.d/lib-forbidden-notations.py:108` 起，到 `.claude/gate.d/lib-forbidden-notations.py:111`）只列了括号短号、批号形状、长跑拆分与 json，没有这一条。探针里这一格写成期望「refuse」，预计是全表唯一不一致的一格（待主 agent 跑）。 |
| 「两位短号」对括号短号 | 一致（缝已列） | 判据行只判括号里的（`.claude/gate.d/lib-forbidden-notations.py:108` 的「不带括号的会话短号」已在「管不到的」里）。 |
| 「长跑拆分」一行 | 轻 | 共用约束写「`records/` 下调度记录里以「长跑拆分：」起头的那一行」（`.claude/agent-common.md:42`）；判据库放过 `records/` 下任何文件里以它起头的行（`.claude/gate.d/lib-forbidden-notations.py:485`）。`.claude/rules/long-run-split.md:18` 写「只在 `records/` 下照写任务号」，与判法一致，与共用约束的「调度记录」差一个词。 |
| 「依赖表的批次那一列」 | 轻 | 共用约束「`records/` 下调度记录依赖表的批次那一列」（`.claude/agent-common.md:43`）；判据库放过 `records/` 下任何表头是「批次」的列（`.claude/gate.d/lib-forbidden-notations.py:554`），不限调度记录。 |

### 1.5 冻结证据豁免（第 33 行）

共用约束第 45 行写「绕过它们写进仓的……由门禁 doc-text 的 `task-and-session-numbers` 格在提交时判相对基准多出来的行」（`.claude/agent-common.md:45`），没提 `research/prompts/` 不判。门禁自己写明「research/prompts/ 下的文件（三方材料、判决与它们的快照副本）是冻结证据」（`.claude/gate.d/doc-text.sh:188`），代码按整个目录豁免（`.claude/gate.d/doc-text.sh:2186`：「FROZEN_EVIDENCE_PREFIX = "research/prompts/"」）。判定：推不出。用户选的是「冻结证据豁免」这个选项名，没有目录名；一整个 `research/prompts/` 当冻结证据，包括这一轮新写进去还没冻结的派发提示、腿报告与判决，是主 agent 取的射程。共用约束没写这一条豁免，读共用约束的人看不出提交时这一格放过那个目录；判决 `*-main-verification.md` 在写的那一刻仍由写闸判（门禁头注 `.claude/gate.d/doc-text.sh:188` 起的那几行也这么写）。

### 1.6 不另开 `agent(<短号>)` 例外（第 36、37 行）

一致：共用约束、沟通纪律与判据库没有给调度记录单开写法；`grep -rnF 'agent(<短号>' .claude/*.md .claude/rules .claude/agents .claude/skills scripts/agents` 命中 0 处（这一次跑的）。旁注（不在这三份文件里，推不出算不算缺陷）：`.claude/skills/dispatch/SKILL.md:20` 写「认会话看变异行与记录里的名字」，而「记录里」不再写会话名；第 37 行用户原话说调度日志里有调度记录，认会话该走任务服务的调度记录。

## 关 2：第二轮判决「三、改法」八条逐条现读核落地

判决原文在 `research/prompts/no-task-numbers-governance-r2-main-verification.md` 第 38 至 49 行。「落地」按文字与代码读；要行为证据的格另写进探针（待主 agent 跑）。

| 条 | 判决写的 | 落在哪（主工作区现读） | 判定 |
|---|---|---|---|
| 1 任务号 | 日志文件名与运行目录名里的号照判；单个「同 <号>」照判，后面跟量词或单位的放过；号后接 ASCII 冒号再接空白或行尾的照判；量词放宽只留「个」 | `.claude/gate.d/lib-forbidden-notations.py:294`：「TASK_LOG_NAME = rf'(?<![{ASCII_WORD}])task-[0-9a-f]{{8}}-\d+'」；`.claude/gate.d/lib-forbidden-notations.py:298`：「(?<![相不共如协合])同\s*{TASK_JOINED}(?!{QUANTITY_AFTER})'」；`.claude/gate.d/lib-forbidden-notations.py:74`：「号后面接冒号再接空白或行尾的照判（「任务 <号>: 退 0」）」；`.claude/gate.d/lib-forbidden-notations.py:75`：「个」后面接成词的（「个别」「个人」，表是 COUNT_WORD_COMPOUNDS）照判，「条」不当量词。」。日志名的标签位 8 位十六进制与任务服务一致：`task-server/task_server.py:1444` 是 `self.tag = secrets.token_hex(4)`，日志名在 `task-server/task_server.py:9193`（`f"task-{self.store.tag}-{task['id']}.attempt-{number}.log"`）。 | 落地，一致（行为格在探针「改法1」七格，待主 agent 跑） |
| 2 批号 | 攒批收回「前面紧挨」、`records/` 下不放宽；量词表里会作双字词头一个字的不算量词；中文数字第几批在 `records/` 下照判；锚点照判；表格只放过表头是「批次」的那一列 | `.claude/gate.d/lib-forbidden-notations.py:560`：「return not under_records(relative_path) and match.group(0).startswith('批') and BATCH_SIZE_WORDS.search(before) is not None」，紧挨的写法在 `.claude/gate.d/lib-forbidden-notations.py:339`：「BATCH_SIZE_WORDS = re.compile(r'(?:seq…攒…追加…覆写…写入)\s*$')」；`.claude/gate.d/lib-forbidden-notations.py:85`：「量词字后面接成一个双字词的（「处理」「行为」「项目」，表是 MEASURE_WORD_COMPOUNDS）不当量词，照判。」；`.claude/gate.d/lib-forbidden-notations.py:548`：「if match.group('chinese_ordinal_batch') is not None and not under_records(relative_path):」（放过的只在 `records/` 之外）；锚点 `.claude/gate.d/lib-forbidden-notations.py:83`：「前面带 # 的锚点「`#第 6 批`」同样算」，正则 `.claude/gate.d/lib-forbidden-notations.py:322` 的「第」前只拦 ASCII 字母数字，`#` 不挡；表格 `.claude/gate.d/lib-forbidden-notations.py:554`：「column_header(line, match.start(), table_header) == BATCH_COLUMN_HEADER」。 | 落地，一致（探针「改法2」十格，待主 agent 跑） |
| 3 会话短号 | 判决文件里同 `records/` 一样照判；会话词表补「范围」「判过」 | `.claude/gate.d/lib-forbidden-notations.py:457`：「return not (under_records(relative_path) or ZERO_ROUND_VERDICT_PATH.fullmatch(relative_path) is not None)」；`.claude/gate.d/lib-forbidden-notations.py:304`：「SESSION_WORDS_INSIDE = '在修…在改…在跑…在做…接了…已接…接手…推的…量的…转述…范围…判过…已认领…认领…回信…回话'」 | 落地，一致（探针「改法3」四格） |
| 4 A/B 读数一行 | 只放过值里按派发闸认法认出的任务号、至多两个，引用块不放；攻方 3f 四条进自证，两边判法相同 | `.claude/gate.d/lib-forbidden-notations.py:334`：「ZERO_ROUND_READING_MOST_NUMBERS = 2」；`.claude/gate.d/lib-forbidden-notations.py:94`：「放过的只是与认出的那几处起止都相同的命中」；引用块 `.claude/gate.d/lib-forbidden-notations.py:471`：「if line.lstrip().startswith('>') and」；派发闸交判据库判 `.claude/hooks/runner-dispatch-guard.sh:2709`：「refused += [(line.strip(), name, matched) for _, _, name, matched in library.new_violation_hits(NOTATIONS_FORM, "", line, ZERO_ROUND_VERDICT_PATH)]」；自证四条另带的号与引用块一格在 `.claude/hooks/runner-dispatch-guard.sh:3975`：「("读数一行另带 task= 的号，判决里照拒、派发闸当场拒"」起的几行，七条对拍在其后。三条认法逐字相同见 1.3。 | 落地，一致（探针「改法4」八格） |
| 5 写入一侧 | 写闸、⑰、五份写入工具改成整份写前写后各判一遍，写后多出来的违例行才拒；replace-batch 的 count 每一处都在内 | 写闸 `.claude/hooks/write-guard.sh:692`：「交库的 new_violation_hits，写后多出来的违例行才算。返回同 library_hits_in_written_texts，「在哪」是写后文件的行号。」；⑰ `.claude/hooks/bash-command-detector.sh:5273`：「after = before + write.text if appending else write.text」；`scripts/kb/lib_notation_check.py:122`：「hits = library.new_violation_hits(form_key, before_text, new_text, relative)」；`scripts/kb/replace-once.py:154`：「before_text=content, after_text=replaced)」；`scripts/kb/replace-batch.py:158`：「写后多出来的违例行才拒（每个有命中的文件一个），一处的 count 几次都在内。」；`scripts/kb/append-record.py:183`：「refusal = session_name_refusal(path, body, first_line, notation_root, before_text=before_text, after_text=after_text)」；`scripts/kb/insert-row.py:216`：「before_text=original.decode('utf-8'), after_text=written)」；`scripts/kb/ledger-append-section.py:290`：「refusal = numbers_refusal(path, original_text, new_text, notation_root)」。「插入点前文」的旧接法没有残留：`grep -n '插入点前\|preceding_text\|before_context\|prefix_text'` 在写闸、检出钩子、`scripts/kb/` 与库里只命中自证用例名与说明（这一次跑的）。 | 落地，一致（探针「写前写后」三格） |
| 6 出路 | 两份写入工具的出路照任务号与批号写；判据库与共用约束的出路写明不抄日志路径 | `scripts/kb/lib_notation_check.py:59`：「('新串写了任务号', '任务号换成交给服务的那条命令（不带号）或结局行的 type= 与 exit= 那一段，引日志只抄那一行的内容、不抄日志路径；'」；`scripts/kb/replace-batch.py:38`：「任务号与批号的出路照任务号与批号写，不说成会话号）。」；`scripts/kb/ledger-append-section.py:24`：「closing_remedy，任务号与批号不说成会话号）。」；库的出路在 `.claude/gate.d/lib-forbidden-notations.py:878`；共用约束 `.claude/agent-common.md:42`：「只抄那一行的字，不抄日志路径：」 | 落地，一致 |
| 7 门禁新格 | doc-text 加一格，相对基准新加的 `.md`、`.tsv`、`.txt` 行照这一形态判，补 `cp`、`mv`、命令输出写进仓的路 | `.claude/gate.d/doc-text.sh:1416`：「stage_cell task-and-session-numbers cell_task_and_session_numbers」；`.claude/gate.d/doc-text.sh:2241`：「found = notations.new_violation_hits(FORM, base_text(path), after, path)」，基准取 git 里那一版（`base_text`）。第 33 行用户定的豁免见 1.5。 | 落地，与判决写的同一件事；射程比判决多一条豁免（用户后定，见 1.5） |
| 8 判据库头注「管不到的」 | 写进「长跑拆分：」一行装几个号、`.json` 不判两条 | `.claude/gate.d/lib-forbidden-notations.py:110`：「records/ 下以「长跑拆分：」起头的一行装几个任务号」；`.claude/gate.d/lib-forbidden-notations.py:111`：「.json（records/ 下的机读台账也是，judged_suffixes 不收）。」 | 落地，一致；这一段没写 1.4 说的「不带那一路的描述」那一条缝 |

八条都落了地、落得与判决写的是同一件事；没有一条没落地。第 7 条另有 1.5 的豁免射程问题，第 8 条漏列一条缝（1.4）。

## 关 3：两份规范新文字只写怎么做、不写为什么与来历

判据：`.claude/singlefs-ai-sop/rules/rules-discipline.md`「1. 正文只写四样」（现读：「怎么做：步骤、次序、判据、阈值」，`.claude/singlefs-ai-sop/rules/rules-discipline.md:11`），写了论证、日期、来历的一句算打中。

| 句 | 判定 | 证据 |
|---|---|---|
| 共用约束「（只抄那一行的字，不抄日志路径：任务服务的日志文件名与运行目录名 `task-<十六进制>-<号>` 带着任务号）」 | 成立（轻） | `.claude/agent-common.md:42`：「任务服务的日志文件名与运行目录名 `task-<十六进制>-<号>` 带着任务号」。前半句是怎么做（不抄日志路径），冒号后面是为什么（文件名带号）。同一句原样在判据库出路 `.claude/gate.d/lib-forbidden-notations.py:878`。留认形即可：「不抄日志路径（文件名形如 `task-<十六进制>-<号>`）」。 |
| 共用约束「值里至多两个任务号，A/B 两臂，引用块里的不算」 | 成立（轻） | `.claude/agent-common.md:42`：「值里至多两个任务号，A/B 两臂，引用块里的不算」。「A/B 两臂」是在解释为什么两个。 |
| 派发闸说明「（A/B 两臂），引用块（行首的 >）里的不算」 | 成立（轻） | `.claude/hooks/runner-dispatch-guard.sh:2715`：「（A/B 两臂），引用块（行首的 >）里的不算」。同上。 |
| 共用约束第 40、41、43、44、45 行 | 不成立 | 逐条读：只有做什么、不做什么、闸怎么判、出路；没有日期、谁定的、哪一轮打中；第 43 行「（闸按表头认列……照判）」是判法。 |
| 沟通纪律第 66 行 | 不成立 | 整句是做法加一处指向（`.claude/rules/writing-discipline-by-review.md:66`），没有论证与来历。 |
| 派发闸其余几句 | 不成立 | 「→ 规矩」「→ 怎么办」两句是判法与出路；「写进判决时写闸与写入工具照拒」是后果说明（判据，不是来历）。 |

## 要主 agent 跑的命令

1. 命令原样：`task-server/task submit -- python3 /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-sonnet/probe_items.py --repo /home/fy5090/code/singlefs`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。
   - 看什么：`task-server/task log <号>` 里每格一行 `name=case label=… expected=… got=… agree=…`，末行 `name=probe_summary cases=49 disagreements=<k>`。我从判据库正则读出的预测是 k=1：不一致的一格只有「描述:不带那一路的描述（规则文字说这类描述）」（规则说「这类描述」都不写，判据库放过，对应 1.4 第一行）；结局行预期 `type=command-failed exit=1`（探针有不一致就退 1）。k 大于 1，或不一致的格不是这一格，每多出来的一格都是新发现，把那一行 `agree=False` 原样贴进判决。k 为 0 说明我读错了那一格。
   - 探针只读：加载 `--repo` 指的仓里的 `.claude/gate.d/lib-forbidden-notations.py`，调 `new_violation_hits`，不写文件、不起子进程；内存与时间是推的（一份约千行的 Python 库、49 个几行的文本，秒级、几十 MB）。
2. 报告交回前我跑过 `python3 scripts/agents/cite-check.py research/prompts/no-task-numbers-governance-r3-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_no-task-numbers-governance-r3-body.md`，末行见本报告最后一节；判决要引的数由主 agent 复跑。

没有写模型目录，也就没有 `tools/rerun.sh` 与外面再包一层的复跑脚本。

## 什么现象会推翻上面的结论

- 1.2 的「任务号与批号也不写」若用户要的就是对话里也不写：推翻（但共用约束与派发骨架要同改）。查法：问用户弹窗「汇报与进仓文字都改」选项的原文。
- 1.1 的两处「推不出」若弹窗原文里有「会话之间的消息」或「只许 mainagent / subagent」：改成一致。
- 1.4 第一行：探针那一格若判据库真拒了不带「那一路」的描述，推翻。
- 关 2 的「落地」：任一探针格 `agree=False`（除上面预测的那一格）推翻对应条的「一致」。
- 关 3 三处轻：若主 agent 认为「A/B 两臂」「带着任务号」是认形所需、不是论证，改成不成立。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1.1 「不拿描述指东西」 | 推不出 | 比原话宽，原话只点指会话的描述 |
| 1.1 「不换别的说法」 | 推不出 | 原话「或者」给了 `agent(A)` 与两组 mainagent / subagent，没说排他 |
| 1.1 「会话之间的消息」 | 推不出 | 只在记录第 34 行主 agent 转述里，弹窗原文查不到 |
| 1.1 其余句（会话全名、描述、叫法、任务号、两处保留、批号、批次列） | 一致 | 对得上第 5、25、34 行原话 |
| 1.2 沟通纪律「任务号与批号也不写」 | 成立 | 用户射程是进仓文字；与共用约束的派发提示批次与「指到任务号」及派发骨架相撞 |
| 1.2 「同一段话」对「同一份文字」 | 成立 | 两份规范与闸的拒绝说明说法不一，字母编号的范围不同 |
| 1.2 缺 `那一路的 agent` 与「不换别的说法」 | 成立（轻） | 两份点名的描述与限定不同 |
| 1.3 派发闸「只有按「任务 <号>」写的」 | 成立（轻） | 判法还放过 `task <号>`、`任务号：<号>` |
| 1.3 派发闸「…、<号>）」 | 推不出（轻） | 最后一个「<号>」没说哪种写法 |
| 1.3 派发闸其余句、三条认法 | 一致 | 与共用约束同一句话；三条正则两处逐字相同 |
| 1.4 「这类描述」对判据行 | 成立 | 判据库只认「那一路」加会话词，头注「管不到的」没列这条缝 |
| 1.4 「两位短号」「长跑拆分」「批次列」 | 一致 / 轻 | 括号缝已列；后两处文字窄于判法一个词 |
| 1.5 冻结证据豁免 | 推不出 | 用户选的是选项名；整个 `research/prompts/` 豁免是主 agent 取的射程，共用约束没写 |
| 1.6 不另开 `agent(<短号>)` 例外 | 一致 | 三份文字没开例外；旁注 `.claude/skills/dispatch/SKILL.md:20` 仍写「记录里的名字」 |
| 关 2 第 1、2、3、4、5、6 条 | 落地，一致 | 文件与行见关 2 表；行为格待探针 |
| 关 2 第 7 条 | 落地 | 射程比判决多一条用户后定的豁免 |
| 关 2 第 8 条 | 落地 | 头注两条都写了，没列「不带那一路的描述」 |
| 关 3 三处「为什么」 | 成立（轻） | 共用约束两处、派发闸一处带了解释从句 |
| 关 3 其余句 | 不成立 | 只写怎么做 |

## 没做什么

- 没有替主 agent 采纳或出判决；没有判别的腿（攻方）的格。
- 没有往统一任务服务交任务；探针没跑，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没有读禁读清单里的文件，没有读攻方腿的派发提示。
- 没有核「交用户的」三处与第一轮判决；没有核全仓「还要求写号的句子」（第一轮的射程）；没有核 `crates/`（这一轮没改）。
- 没有量 `records/`、`.claude/kb/` 里存量被新格报的行数。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-sonnet` 里只有探针与认领文件，没有仓副本与编译目录，`du -sh` 量的24K，不需要删。

## cite-check 末行

`✓ 核了 61 处引文、42 处裸行号，对上 61 处，没判 42 处（其中裸行号没有引文可比 42 处；逐处列在上面）`

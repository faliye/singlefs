# runner-compile-first-r1 本地攻方提示：逐句核转述

核对对象：`research/prompts/runner-compile-first-r1-local-attack.md`（英文提示，由本腿写）。
每一行：英文项（提示里的原句或摘要）/ 原文文件:行（去源文件现查） / 首稿缺的（英译初稿丢的限定词，或注明「无」） / 定稿（补上之后现在提示里的措辞，或「未改，理由见备注」）。

## 一、上下文段（提示开头 "Context." 段）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Hook A（write-guard.sh）only wired to Write and Edit（matcher "Write\|Edit"） | `.claude/settings.json:40`（`"matcher": "Write\|Edit"`）、`:44`（command 行） | 无：settings.json 本身是 JSON，没有中文限定词可丢 | 未改，原样 |
| Hook B（bash-command-detector.sh）only wired to Bash | `.claude/settings.json:23`（`"matcher": "Bash"`）、`:31`（command 行）；`bash-command-detector.sh:5`（`# hook-events: PreToolUse:Bash`） | 无 | 未改，原样 |
| A call made by the main agent carries no agent_type field at all；子 agent 的调用带 agent_type 等于它的名字 | `write-guard.sh:92-94`（`agent_type = hook_input.get("agent_type"); if not agent_type: return 0, None`）代码本身即是判据，没有独立中文句子可比对 | 无 | 未改，原样 |

## 二、事实表 F1-F16

| 英文项（F 号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| F1：decide() 四道判定固定顺序，`decide_overwrite` → `decide_compile_first` → `decide_scope` → `decide_prime_marks`，全部为 0 才放行 | `write-guard.sh:190-201`（现读原样：190 `def decide(...)`，191 `"""返回 (0, None, None) 放行；(2, 说明, 哪一道) 拒绝。"""`，192-201 四段 `code, message = decide_xxx(...); if code: return ...`） | 无：这是代码结构本身的转述，四道的顺序与「全 0 才放行」都对得上 191 行docstring 与代码次序 | 未改，原样 |
| F2：`decide_compile_first` 在 (a) agent_type 不是 experiment-runner，或 (b) `WRITE_GUARD_DISABLE_COMPILE_FIRST`=="1"，或目标 .rs 且在 crates/ 下 时才拒绝，否则 (0, None) | `write-guard.sh:120-128`（现读原样：123 行 `if agent_type not in COMPILE_FIRST_AGENTS or os.environ.get("WRITE_GUARD_DISABLE_COMPILE_FIRST") == "1": return 0, None`；127 行 `if not absolute or not absolute.endswith(".rs") or not absolute.startswith(os.path.join(root, "crates") + os.sep): return 0, None`） | 首稿只写了 (a)(b)(c)目标不以 .rs 结尾(d)目标不在 crates/ 下 四条，漏了 127 行里 `not absolute`（算不出目标路径本身时同样放行）这一支 | 已用 `research/scripts/replace-once.py` 定点改：加入 (c) 「the resolved absolute target path could not be determined at all (is empty or missing)」，原 (c)(d) 顺移为 (d)(e)，「Only when none of (a)-(d)」改成「(a)-(e)」 |
| F3：`decide_scope` 在 (a) 没有 agent_type，或 (b) 没有对应的 `.claude/agents/<agent_type>.md` 时返回 (0, None)，才继续查表 | `write-guard.sh:90-99`（现读原样：93-94 `if not agent_type: return 0, None`；95-96 `if not os.path.isfile(...): return 0, None`；97-98 `if os.environ.get("WRITE_GUARD_DISABLE_SCOPE") == "1": return 0, None`） | 首稿只写了 (a)(b) 两条，漏了 97-98 行 `WRITE_GUARD_DISABLE_SCOPE == "1"` 这第三条放行支路 | 已用 `replace-once.py` 定点改：加入 (c) 「the environment variable WRITE_GUARD_DISABLE_SCOPE equals the string 1」，「either of the following」改成「any of the following」，「(a)-(b)」改成「(a)-(c)」 |
| F4-F8：`agent-write-scope.tsv` 第 6、11、13、17、23 行各自的 pattern 与 agent | `.claude/hooks/agent-write-scope.tsv:6,11,13,17,23`（`grep -n` 现查，见运行记录） | 无：每一行只译 agent 名与 pattern 本身，没有转述判据之外的限定语 | 未改，原样 |
| F9：`COMPILE_FIRST_AGENTS = ("experiment-runner",)`；`COMPILE_THEN_SWAP_SCRIPT = "research/scripts/compile-then-swap.py"` | `bash-command-detector.sh:504-505` | 无 | 未改，原样 |
| F10：`is_main_workspace_crate_source` 两个条件都成立才 true | `bash-command-detector.sh:507-508`（`return path.endswith(".rs") and is_inside(path, os.path.join(repository_root, "crates"))`） | 无 | 未改，原样 |
| F11：`compile_first_refusal` 三条件之一成立就交回空表，否则调 `compile_first_verdict` | `bash-command-detector.sh:546-551`（现读原样：548-550 三条 or 判定） | 首稿没搬 547 行 docstring 里「前台、run_in_background 一样判」这一句；不影响本轮十格判定（十格都没有区分前台/后台的提问），判为不影响答案的省略 | 未改，理由见备注：本轮操作表未涉及前台/后台差异，省略不改变任何一格的可判结论 |
| F12：`compile_first_verdict` 只认特定写法（重定向、tee、cp、mv、install、dd、truncate、sed -i、perl -i、rsync、内嵌 python），目标不在 crates/ 下就不进拒绝表 | `bash-command-detector.sh:510-544`（配合注释 `bash-command-detector.sh:61-63` 认的写法清单：`>、>|、&>、>& 文件、不带 -a 的 tee、cp / mv / install 的目标、dd of=、truncate`，every_write 时另认 `>>、&>>、tee -a、sed -i、perl -i、rsync 的目标，mv 挪走与 rm 删掉的源…喂给 python 的代码里 open(…, 带 w/a/x/+的模式)、Path.write_text/write_bytes、shutil.copyfile/copy/copy2/move 与 os.replace/os.rename 的目标`） | 首稿把「重定向」「内嵌 python 代码」当成两个概括词，没有逐一列出 `>>、&>>、tee -a`（重定向的每一种）与 `open 的各种模式、Path.write_text/write_bytes、shutil 的四个函数名、os.replace/os.rename`（python 那一支的每一种）；这些子项没有一项会改变本轮十格的答案（十格里没有一个用到 sed/perl/rsync 或内嵌 python），判为不影响答案的概括 | 未改，理由见备注：概括词覆盖了源码列出的每一大类写法，未落到子项不改变任何一格的可判结论 |
| F13：⑨ 注释里明写的放行清单（主 agent、implementation-writer 与别的 agent；草稿目录里的写；拷进草稿目录；crates/ 下非 .rs；调用 compile-then-swap.py 脚本；引号里当数据写的等） | `bash-command-detector.sh:57-66`，尤其 65-66 行原文：「放行：主 agent、implementation-writer 与别的 agent；草稿目录里的写；把主工作区那份拷进草稿目录；crates/ 下不是 .rs 的（crates/mutations.tsv）；`python3 research/scripts/compile-then-swap.py …`（脚本文件里的写这里本来就判不到）；引号里当数据写的、写进文件的 heredoc 正文、注释里的。」 | 无：每一项都逐个译出，包括括注「脚本文件里的写这里本来就判不到」 | 未改，原样 |
| F14：脚本文件当作普通命令调用（非 `-c` 内嵌代码）时，`compile_first_verdict` 的扫描器认不出里面的写法，因此不产生任何拒绝 | 由 F12 的扫描范围（`bash-command-detector.sh:510-544`）与 F13 括注「脚本文件里的写这里本来就判不到」（`bash-command-detector.sh:66`）共同推出，不是单一中文句子的直译 | 不适用（推论性事实，非转述） | 不适用 |
| F15：`capped.sh` 这类非 shell 的包装脚本不在 `overwrite_steps` 递归下钻的 shell 名单里，它的参数被当成字面词，不当嵌套命令再扫 | 由 `bash-command-detector.sh:838-957`（`follow_simple_command`、`overwrite_steps` 对 `shell_words.SHELL_NAMES` 的特殊处理与其余命令的默认处理）现读代码结构推出，不是单一中文句子的直译 | 不适用（推论性事实，非转述） | 不适用 |
| F16：自证里那一条「先编后换脚本换进来」的用例，命令与期望值原样引用 | `bash-command-detector.sh:1934-1935`（现读原样：1934 `("执行员经先编后换脚本换进来放行", "experiment-runner",`，1935 `f"bash research/scripts/capped.sh 16 python3 {COMPILE_THEN_SWAP_SCRIPT} {draft_path} {bin_path} --scratch /tmp/claude-1000/runner-x", 0),`） | 标签「执行员经先编后换脚本换进来放行」译成英文注释「runner using the compile-then-swap script to swap in the draft copy, expected to pass」，「放行」译成「expected to pass」而非逐字「allowed」；语义相同（自证期望值 0 = 不拒绝 = 放行），未丢限定词 | 未改，原样 |

## 三、多出来的限定词或括注（英文比原文多的部分）

| 英文项 | 为什么加 |
|---|---|
| 提示开头一整段 "Context." 与 "Your job is not to evaluate..." 说明段 | 原文（正文 R3 格与派发提示）没有逐句这样的框架性说明；本地模型读不到正文与背景材料，提示必须自足（`three-way-local-attack.md`「自足、不用任何 markdown 强调」），这一段是为了让提示脱离背景材料也能独立成立而加的，不代表任何一条判据本身 |
| 十个操作（op1-op10）里给的具体路径、具体文件名（如 `e161_crash_state_dedup_and_time_split.rs`、`/tmp/claude-1000/experiment-runner-e999/`）与工具绑定（Read/Edit/Write/Bash 的选择） | 派发提示里的操作列表（「① experiment-runner 读 crates/ 下一个 .rs」等）本身没有指定具体文件名、具体工具或具体路径；这一腿为了让操作表可判、自足、不依赖背景材料，选定了一个具体的、有代表性的例子（现有的入库装置文件、已在 `.claude/hooks/agent-write-scope.tsv` 第 11-13、17 行登记过的路径）。这不是转述失真，而是把一个抽象操作落成一个可判的具体实例；十个实例互相之间的可比性、以及与派发提示原意的对应关系已在正文核对（见下方「四、操作表与派发原意对照」） |
| "What to answer" 一节里 Falsification 的示例句 | 派发原意（`.claude/agent-common.md`「结论写『什么现象会推翻它』」、`three-way-inference.md`「本地攻方…每条答复写…推翻条件」）只要求「写什么现象会推翻它」，没有给出示例措辞；这里加了一句示例是为了让 4-bit 量化模型更容易照着格式回答，不改变判据本身 |

## 四、操作表与派发原意对照

派发提示原文（中文，来自主 agent 这一轮的派发消息，不在仓内文件里，故没有文件:行）：
「① experiment-runner 读 crates/ 下一个 .rs；② 把主工作区那份 cp 进草稿目录；③ Edit 草稿目录里的副本；④ 往 crates/mutations.tsv 末尾追加一行；⑤ 写 research/e7-index-bench/src/bin/ 下的装置；⑥ 改 research/e7-index-bench/Cargo.toml；⑦ 调 research/scripts/compile-then-swap.py；⑧ 调它的 --clean；⑨ implementation-writer Edit 主工作区 crates/ 下的 .rs；⑩ 主 agent 同样的 Edit。」

| 派发原意（①-⑩） | 提示里落成的操作 | 补的具体化内容 |
|---|---|---|
| ① 读 crates/ 下一个 .rs | op1 | 具体文件名、Read 工具 |
| ② cp 进草稿目录 | op2 | 具体文件名、具体草稿路径、Bash 工具、`cp` 命令原样 |
| ③ Edit 草稿目录里的副本 | op3 | 同 op2 的草稿路径、Edit 工具 |
| ④ 追加 crates/mutations.tsv | op4 | Edit 工具、old_string/new_string 的具体写法（追加一行） |
| ⑤ 写 research/e7-index-bench/src/bin/ 下的装置 | op5 | 具体新文件名 `e999_new_device.rs`、Write 工具 |
| ⑥ 改 research/e7-index-bench/Cargo.toml | op6 | Edit 工具、追加 `[[bin]]` 的写法 |
| ⑦ 调 compile-then-swap.py | op7 | 具体命令行（含 `capped.sh 16` 包装、`--scratch` 参数），来源于 `.claude/agents/experiment-runner.md:30`（现查行号；第 ④ 条给的调用形态，背景材料附录已抄同一段文字） |
| ⑧ 调它的 --clean | op8 | 具体命令行，来源同上（`.claude/agents/experiment-runner.md:30` 末句「交回之前跑 `python3 research/scripts/compile-then-swap.py --clean --scratch <草稿目录>`」，现查行号） |
| ⑨ implementation-writer Edit 主工作区 crates/ 下的 .rs | op9 | 具体文件、Edit 工具 |
| ⑩ 主 agent 同样的 Edit | op10 | 具体文件、Edit 工具、agent_type 缺省 |

十条都一一对应，没有遗漏、没有合并、没有拆分。

## 五、跑样本时发现的一处措辞改动（不是转述失真，是避开损坏闸假阳性）

F4、F5、F8 首稿把 `agent-write-scope.tsv` 里的通配符样式原样写成 `crates/**`、`research/e7-index-bench/src/bin/**`、`/tmp/claude-1000/**`（与 `agent-write-scope.tsv:6,11,23` 原文的写法完全一致，语义没有失真）。
第一次采样（s1）答得很短，没有逐字引这几个通配符，过了闸；第二次起（三次尝试，都作废：void1、void2、void3）答得更完整，逐字引了这几个通配符，`corruption-check.py` 的「成对标记落单」检测器把孤立的一个 `**`（前后没有配对的另一个 `**` 把它闭合）当成掉了半截的 markdown 加粗标记，三次都判红（`星号落单=1`，现跑 `python3 research/scripts/corruption-check.py` 核过）。
这不是模型转述丢字或多字，是提示原文本身用了双星号通配符写法、与损坏闸的加粗标记检测规则撞在一起——闸认不出「这是 glob，不是加粗」。
处置：把 F4、F5、F8 里的 `crates/**` 类写法改成不含 `**` 的等价散文表述（「every path anywhere under the crates directory, at any depth of sub-directory」这类），语义与原表格行完全一致（都是「递归匹配这个目录下的任何路径」），只是不再用双星号字面写法；改动用 `research/scripts/replace-once.py` 定点做，命中各一次。

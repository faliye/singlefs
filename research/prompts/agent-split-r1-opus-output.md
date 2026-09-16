# subagent 拆分执行计划正反对抗第一轮：攻方腿（Opus）报告（2026-09-16）

被判对象：`records/2026-09-16-subagent拆分提案.md`（下称「提案」，行号一律是那份文件自己的行号）。分到的格：J3、J4、J6（`research/prompts/_agent-split-r1-background.md` 第 45–52 行的判据表）。
立场：假设照提案做会出事，找能打穿的输入与序列。J1、J2、J5 与自举 / 独立性不在这条腿的范围，碰到时只在「顺带」里记一句，不判。

模型与样本放 `research/prompts/agent-split-r1-opus-model/`，三份，都只在临时目录里造仓、不碰真仓：

| 文件 | sha256 | 复跑 |
|---|---|---|
| `layer3_porcelain_diff.sh` | `e1f628dc676ceef984d6aef70a60e697b8a5f2a13b0815b43aca572c8f403b72` | `nice -n 19 bash research/prompts/agent-split-r1-opus-model/layer3_porcelain_diff.sh` |
| `layer2_scope_hook_model.sh` | `7b07f9cb971c8e5c45b805126f2f7af9c1c1be09f675159bf376cb754e442cf3` | `nice -n 19 bash research/prompts/agent-split-r1-opus-model/layer2_scope_hook_model.sh` |
| `gate_row5_threshold.py` | `92d45eb51948dfc1d044f93a2b75a2848c5df6881a792561cce256b785a7779c` | `nice -n 19 python3 research/prompts/agent-split-r1-opus-model/gate_row5_threshold.py` |

⚠️ 提案第五节的两个脚本（`agent-write-scope.sh`、派发前后比对脚本）和新门禁阶段都还没写。模型里的实现是这条腿按提案字面写的「自然读法」，
打中的是「按字面做会漏 / 会误报」，不是「提案作者心里想的那份实现会漏」。每一格都写了真实现要补哪条样本才能证明自己不漏。

## 〇、各格判定一览（各报各的，不合成结论）

| 格 | 攻击面 | 判定 | 最硬的一条 |
|---|---|---|---|
| J4 | A1 第一层 `tools` / `disallowedTools` | 打中（该红不红 2 种） | Bash 里直接起本机正在跑的 `claude` 二进制，等于绕开「`tools` 里不许有 Agent」；第一批八个全是观测 / 推论类，第一、二层对它们的写一次都拦不到 |
| J4 | A1 第二层 agent 上挂的 hook | 打中（该红不红 5 种、不该红却红 1 种） | 路径模式只能写死在 frontmatter，`<轮>` 占位写不进去 ⇒ 静态模式放行覆盖别的轮已冻结的腿产出 |
| J4 | A1 第三层 派发前后比对 | 打中（该红不红 2 种、不该红却红 2 种） | 派发前就已 ` M` 的文件被越界追加，状态行不变、差集为空；同一条消息并行的三条腿互相进差集，每轮必报 |
| J4 | A2 新门禁七行 | 七行各打中（每行一份该红不红、一份不该红却红；第七行 ①②④ 三形与 56 号同形，按 K3 另记） | 「对抗过没有」：执行计划自己的判决文件点了 `.claude/agents/<名>.md`，之后在同一窗口里落地的定义一轮没攻也全绿 |
| J3 | A3 ① 腿读到别的腿这一轮的东西 | 打中 1 条序列 | 禁读的范围按 `*-output*` 写，本地腿的英文提示文件（`<轮>-local-attack.md`）与云端腿提示不在其内 |
| J3 | A3 ② 证据被污染 | 打中 3 条序列（②-b 大半按 K3 共用） | 交付规矩挪进可变的定义文件，冻结的提示文件不再是「发给腿的输入」；定义在第 2 步被写回后，旧轮次的输入复原不出来（K3 要核：基线的模板文件同形） |
| J3 | A3 ③ 判决实际由 subagent 做出 | 打中 1 条序列（半强） | 核查员是观测类「抽查就能用」，攻方腿的推论经它一过就从「逐条现查」降成「抽查」 |
| J3 | A3 ④ gate-triage 判错归属 | 2 条序列；J3 字面四个结局对不严，四句第三句没全过，交主 agent 判记打中还是线索 | `kb-scribe` 照定义跑 `relabel-item.py`，它会改写 `.claude/kb/**` 之外的文件（今天带分项引用、落在它扫描范围里的有 165 个），主 agent 给 triage 的「这一轮改过的文件清单」里没有它们 |
| J3 | A3 ⑤ sweep 漏改 | 1 条序列（真仓 2026-09-16 已发生过）；同 ④，四句第三句没全过 | E146 的 `TREE_TABLE_ENTRIES_PER_UNIT = 112` 是从更早的 145 派生的，148 的派生形态一个都搜不到它；三个分类里没有「不同的数、同一个量」 |
| J3 | A4 限额与资源锁 | 打中 1 条序列（4.1），1 条推测待第 0 步测（4.2），资源锁一组记线索（4.3，不在 J3 字面的结局里） | 「开跑前取锁」在 Bash 工具每次调用一个新 shell 的前提下只能靠后台持锁进程，agent 撞限额或被打断后锁成孤儿 |
| J6 | A5 代价数 | 缺代价数（按判据不算打中） | 第一批八个、每个三轮、每轮 5 次派发：下限 128 次派发、1764 万 token；全部十五个下限 261 次、3597 万 token；提案第 201 行自陈没算 |

## 一、A1（J4）写范围三层闸

提案原文（第 123–129 行，整行抄）：

```
| 层 | 做什么 | 拦得住 | 拦不住 |
|---|---|---|---|
| `tools` / `disallowedTools` | 观测类与推论类不给 Write / Edit | 用 Write / Edit 改仓里文件 | Bash 里的重定向 |
| agent 上挂的 PreToolUse hook | 新脚本 `.claude/hooks/agent-write-scope.sh <路径模式…>`：Write / Edit 的目标不在范围内就拒绝，并说该交给谁 | 执行类 agent 越界写 | Bash |
| 派发前后比一次 | 新脚本记派发前后的 `git status --porcelain` 与未跟踪文件，差集里有范围外的路径就报 | Bash 越界写 | 同一段时间里别的会话的改动也会进差集，只能报「疑似」 |

Bash 管不住这件事照实写进每个定义的「没做什么」。
```

提案自己已经写明的缝（Bash 重定向、别的会话进差集）不算打中；1.1–1.3 只列那张表里没写的。

### 1.1 第一层 `tools` / `disallowedTools`

**该红不红 ①：Bash 起一个不带任何 agent 限制的嵌套会话。**
输入：一个 `tools` 含 Bash、不含 Agent 的 agent 调用 Bash 工具，参数 `command` 为
`"$(ps -o args= -u "$(id -u)" | grep -o '^/[^ ]*native-binary/claude' | head -1)" -p '…' --output-format text`。
现查（2026-09-16，只读）：`~/.local/bin/claude` 是指向 `anthropic.claude-code-2.1.246-linux-x64/…/claude` 的悬空链接（目标不存在），而 `ps` 里本会话自己的进程是
`/home/fy5090/.vscode-server/extensions/anthropic.claude-code-2.1.270-linux-x64/resources/native-binary/claude`，那份文件存在、`-rwxr-xr-x`、223981040 字节。
⇒ `command -v claude` 查不到，但从 `ps` 取到的路径能直接执行。起出来的是一个完整的 Claude Code 会话：默认工具全开（含 Agent、Write、Edit），
agent frontmatter 里的 `hooks` 不在它身上，只剩项目 `.claude/settings.json` 里那一条 matcher 为 Write 的 `refuse-overwrite-untracked.sh`。
⚠️ 没跑：跑它会加载本仓与用户级配置、花一次完整派发的 token，交付规矩不许建 agent 或改配置，起一个会话同样越线。「`-p` 起出来的会话工具全开」是推测，第 0 步要测。
- 提案里许可它的句子：第 55–56 行「agent 定义的 `tools` 里不给 Agent 工具」只收掉了 Agent 工具；第 157 行第 0 步写了「`tools` 不给 Agent 时它能不能绕过」，但没写测哪几种形状。
- 什么现象会推翻它：第 0 步在一个 `tools: Read, Bash` 的自定义 agent 里跑 1.1 ① 那条命令，嵌套会话报「没有 Agent / Write 工具」或 Claude Code 拒绝在 subagent 的 Bash 里起 `claude`。
- 改法（只在这条腿的推导里成立、被攻过零轮）：第 0 步的样本里加这一形；定义的「没做什么」写明「Bash 能起嵌套会话」；第三层比对能看见嵌套会话写仓里未脏文件，但看不见 1.3 那几种。

**该红不红 ②：第一批八个 agent 的写全部走 Bash，第一、二层对第一批的判别力为零。**
第 158 行第一批八个是 `tw-forward`、`tw-attack`、`tw-defense`、`tw-local-attack`、`tw-local-defense`、`tw-verifier`、`gate-triage`、`sweep`；
第 80–85、105–106 行给它们标的类型是「推论」或「观测」；第 125 行「观测类与推论类不给 Write / Edit」。
而它们都有要写的产出（第 80–85 行写范围列的 `research/prompts/<轮>-*-output.md`，第 117 行「只写报告」的放 `/tmp/claude-1000/agents/<名字>/`），
第 87 行共同交付规矩里的「分段写」「第一段排他新建」（`.claude/rules/three-way-inference.md` 第 237 行）落到手上是 Bash：`.claude/singlefs-ai-sop/rules/session-wrapup.md` 第 61 行「新建文件用排他方式（`set -o noclobber`、`open(路径, 'x')`）」，Write 工具没有排他模式。
⇒ 第一批的每一次合法写与每一次越界写都走 Bash，第一层拿掉 Write / Edit 之后，第二层 hook 在第一批上**一个对象都没有**，第三层是第一批唯一的闸。
第 158 行给第一批的理由「都只读或只写自己的产出，越界代价最小」于是只靠第三层撑着，而第三层有 1.3 那两种该红不红。
- 什么现象会推翻它：第一批里有一个定义给了 Write（那就违背第 125 行），或腿的产出改成回复里返回、由主 agent 落盘。

**不该红却红**：第一层不是会红的检查（它是删工具），没有「误红」这一面；它的误伤形态是 `gate-triage` 按第 105 行跑 `gate.sh`，
而共享门禁 `.claude/singlefs-ai-sop/scripts/gate.sh` 第 423 行 `git -C "$ROOT" update-ref refs/singlefs/gate-ok "$START_HEAD" 2>/dev/null || true`（全绿时记位置）、
第 58 行 `--staged` 时 `git -C "$src_root" worktree add --detach "$staged_tree" HEAD` 都是 git 写操作，与第 151 行「所有 agent 不做任何 git 写操作」字面冲突；
三层都看不见它（ref 与 `.git/worktrees/` 不进 `git status`）。这一处是提案两句话互相冲突，按分工它属 J2，这里只记成「第三层对 git 写零判别力」的一个实例。

### 1.2 第二层 agent 上挂的 `agent-write-scope.sh`

模型 `layer2_scope_hook_model.sh` 读 stdin JSON 的那一行照抄仓里唯一现成的 hook：`.claude/hooks/refuse-overwrite-untracked.sh` 第 54–55 行
`target="$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print((data.get("tool_input") or {}).get("file_path",""))' "$input" 2>/dev/null)"` 与
`[[ -n "$target" ]] || exit 0`。两种写法：naive（去项目根前缀后 `[[ "$rel" == $pattern ]]`）、normalized（先 `realpath -m`）。原样输出：

```
A Edit 带 .. 跳出 kb 改 crates [naive] → 放行（该：拒绝）
A Edit 带 .. 跳出 kb 改 crates [normalized] → 拒绝（该：拒绝）
B Write 经 kb 里的符号链接写 crates [naive] → 放行（该：拒绝）
B Write 经 kb 里的符号链接写 crates [normalized] → 拒绝（该：拒绝）
C NotebookEdit 写 crates（参数名是 notebook_path） [naive] → 放行（该：拒绝）
C NotebookEdit 写 crates（参数名是 notebook_path） [normalized] → 放行（该：拒绝）
D 正推腿的静态模式覆盖别的轮已冻结的产出 [naive] → 放行（该：拒绝）
D 正推腿的静态模式覆盖别的轮已冻结的产出 [normalized] → 放行（该：拒绝）
E 模式里的 * 跨目录：写进别的腿的模型目录 [naive] → 放行（该：拒绝）
E 模式里的 * 跨目录：写进别的腿的模型目录 [normalized] → 放行（该：拒绝）
F 经符号链接的项目根进来的合法写 [naive] → 拒绝（该：放行）
F 经符号链接的项目根进来的合法写 [normalized] → 拒绝（该：放行）
F' 同上（链接建好之后） [naive] → 拒绝（该：放行）
F' 同上（链接建好之后） [normalized] → 放行（该：放行）
```

⚠️ F 那两行是模型的构造错（链接在 F 之后才建），不算；F' 才是这一形。

| 形 | 工具调用（工具名与参数） | 哪种写法漏 | 真实现要补的样本 |
|---|---|---|---|
| A `..` | `Edit`，`file_path`=`<仓>/.claude/kb/../../crates/singlefs-core/src/allocator.rs` | 按字符串比的写法（bash 的 `==` 与 python 的 `fnmatch` 里 `*` 都吃 `/`） | 带 `..` 的一格 |
| B 符号链接 | `Write`，`file_path` 经范围内一个指向范围外的链接 | 按字符串比的写法（今天仓里除 `target/` 外没有符号链接，现查 `find . -path ./.git -prune -o -type l -print` 空；是否出现由有 Bash 的 agent 决定） | 带链接的一格 |
| C 参数名 | `NotebookEdit`，`notebook_path`=范围外 | 两种都漏：抄 54–55 行只取 `file_path`，取不到就放行 | matcher 与取参都按工具名分派，取不到目标判拒绝不判放行。⚠️ NotebookEdit 能不能新建任意内容的文件、`Edit` 的 matcher 会不会按子串命中 `NotebookEdit`，都没测，第 0 步要测 |
| D 占位写不进 frontmatter | `Write`，`file_path`=`<仓>/research/prompts/c143-r3-sonnet-output.md` | 两种都漏 | 见 1.2「D 是结构性的」一段 |
| E `*` 跨目录 | `Write`，`file_path`=`<仓>/research/prompts/r1-opus-model/x-sonnet-output.md` | 两种都漏 | 模式按段匹配（`*` 不跨 `/`） |
| F' 项目根经链接 | `Write`，`file_path`=`<指向仓的链接>/.claude/kb/a.md` | naive 误红 | 带链接根的一格 |

**D 是结构性的，不是实现细节。** 第 80 行 `tw-forward` 的写范围是 `research/prompts/<轮>-sonnet-output.md`；第 138 行新门禁要求「「写范围」列的路径模式与 hook 参数逐项相等」。
hook 参数写在 agent 的 frontmatter 里，派发时不带参数；`<轮>` 要在派发时才知道 ⇒ 能写进 frontmatter 的只有 `research/prompts/*-sonnet-output.md` 这类静态模式，
它放行全部已入库轮次的腿产出（现数：`ls research/prompts/*-sonnet-output.md | wc -l` → 136，其中 `git ls-files` 已入库 129）与别的会话这一刻没跟踪的产出。
`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 第 51 行「**改一个字，产物就不再对应它的输入**」管的正是那批文件。
现有的 `refuse-overwrite-untracked.sh` 只挂 Write（`.claude/settings.json` 的 matcher 是 `"Write"`），且对已跟踪文件放行（它第 21–22 行），所以挡不住覆盖已入库的旧产出，也挡不住用 Edit 改别的会话没跟踪的产出。
- 什么现象会推翻它：Claude Code 支持派发时给 agent 的 hook 传参（文档没写，第 0 步可以测），或第 138 行改成「定义写模板、派发前由主 agent 实例化」。
- 顺带：D 对第一批其实打不着人——第一批没有 Write / Edit（1.1 ②）。它要等第二批 `tw-materials`、`kb-scribe` 才有对象，而 `kb-scribe` 的范围 `.claude/kb/**` 同样包含全部冻结的决策历史月份文件。

### 1.3 第三层 派发前后比对

模型 `layer3_porcelain_diff.sh` 按第 127 行字面实现：快照 = `git status --porcelain` ∪ `git ls-files --others --exclude-standard`，差集去掉范围正则。原样输出：

```
1 越界追加已脏的 crates 文件 → 不报
2 越界写进别人的未跟踪目录 → 报：research/prompts/c364-model/b.py 
3 改共享规则副本 + 写 settings.local.json → 不报
4 别的会话暂存（状态码 ' M'→'M '） → 报：.claude/kb/x.md 
5 并行的正推腿写自己的产出 → 报：research/prompts/r1-sonnet-output.md 
```

**该红不红 ①：派发前就已脏的文件，越界再写一次，状态行不变。**
现查真仓（2026-09-16）：`git status --porcelain` 里 ` M` 36 项、`??` 115 项；` M` 里有 `crates/singlefs-core/src/allocator.rs`、`recovery.rs`、`transaction.rs`、`unit.rs`、
`crates/singlefs-harness/src/crash.rs`、`CLAUDE.md`、`.claude/rules/` 下四份。任何 agent 用 Bash 往这些文件里追加，前后两行都是 ` M 路径`，差集为空。
已跟踪的脏文件与已存在的未跟踪文件都是这一形：未跟踪文件在 `ls-files --others` 里前后同名，改它的内容同样不进差集（模型第 2 格报出来，是因为写的是新文件）。
- 许可它的句子：第 127 行「记派发前后的 `git status --porcelain` 与未跟踪文件」——比的是路径集合，不是内容。
- 什么现象会推翻它：比对脚本对每个路径记内容哈希（`git hash-object` 或 `sha256sum`），而不是状态行。
- 改法（被攻过零轮）：快照记「路径 → 内容哈希」，脏文件也比哈希。⚠️ 改了之后第三条「不该红却红」变重：别的会话在同一个脏文件里的每一次改动都进差集。

**该红不红 ②：被 `.gitignore` 挡住的路径，写了也不进 `git status`。**
现查 `.gitignore`：`/.claude/singlefs-ai-sop/`、`.claude/settings.local.json`、`.env`、`__pycache__/`、`target/`。
其中两处越界是有实际后果的：改 `.claude/singlefs-ai-sop/rules/*.md`（项目 CLAUDE.md 第 33 行写明「**不许在 `.claude/singlefs-ai-sop/` 里就地改**」），
写 `.claude/settings.local.json`（今天存在，1480 字节，本机每个在本仓里开的会话都会加载它的权限与 hook）。仓外路径同理：`~/.claude/agents/`、`~/.claude/projects/…/memory/`、兄弟目录 `../singlefs-ai-sop-zh/`。
- 许可它的句子：第 127 行「记派发前后的 `git status --porcelain` 与未跟踪文件」。
- 什么现象会推翻它：比对脚本额外快照 `--ignored` 与一份仓外路径清单。

**不该红却红 ①：同一条消息并行的腿互相进差集。** 第 147 行「三条腿同一条消息并行」；第 127 行的「拦不住」一格只写了「别的会话」。
每条腿的派发窗口里都有另两条腿落盘的产出，而那不在它自己的写范围里 ⇒ 每轮每条腿都报「疑似」（模型第 5 格）。
报「疑似」变成常态之后，真越界的那一次与之无从区分；把各腿写范围取并集来消掉误报，又放过了「腿 A 写进腿 B 的产出文件」这一形（它在并集里）。
- 什么现象会推翻它：比对脚本按派发 id 记录「这一刻在飞的全部 agent 的写范围并集」，并且另有一道能分出哪个进程写的（第三层做不到，git 不记写者）。

**不该红却红 ②：别的会话 `git add` / `stage-mine.py` 改了状态码。** 模型第 4 格：` M` → `M `，状态行变了，差集里出现一个根本没被写过的文件。
提案第 68 行主 agent 自己用 `stage-mine.py` 暂存时同样触发。改成只比路径（丢状态码）能消掉它，但又把 1.3 该红不红 ① 加重。

### 1.4 三层叠起来还剩哪条缝

| 缝 | 第一层 | 第二层 | 第三层 |
|---|---|---|---|
| 有 Bash 的 agent 往派发前就已脏的文件里写 | 不管 Bash | 不管 Bash | 状态行不变，看不见 |
| 有 Bash 的 agent 写 `.gitignore` 挡住的路径或仓外路径 | 不管 Bash | 不管 Bash | 不进 `git status` |
| 有 Bash 的 agent 做 git 写（`update-ref`、`worktree add`、`stash`） | 不管 Bash | 不管 Bash | ref 与 `.git/` 不进 `git status` |
| Bash 起嵌套会话 | 绕过 | 嵌套会话身上没有 | 能看见它写未脏文件；已脏文件、被忽略路径、git 写同样看不见 |

⇒ 剩的缝是**一个带 Bash 的 agent 对「已脏文件、被忽略路径、仓外、git 元数据」的写**。第一批八个全部带 Bash（要跑脚本、写报告），这条缝对第一批是全开的。
提案第 129 行「Bash 管不住这件事照实写进每个定义的『没做什么』」承认了 Bash，但没写第三层对这四类是盲的；第 158 行「越界代价最小」没把这四类算进去。

## 二、A2（J4）新门禁七行，逐行造样本

提案第 133–141 行是判据表，第 143 行「每条判据配 `.claude/gate.d/fixtures/` 样本，由 89 号自证会红」。2.1–2.7 每行给两份 `.claude/agents/*.md` 内容：该红不红一份、不该红却红一份。
每行都分「按行 grep 的写法」与「按 YAML 解析的写法」时各判什么；提案没写取哪种，这一点本身就是两种写法会给出相反判定的输入。

### 2.1 frontmatter（第 135 行「从第 1 行起；`name` 与文件名一致；`description` 非空」）

该红不红（文件名 `tw-attack.md`）：

```
---
name: tw-attack
description: ""
tools: Read, Bash
model: opus
name: tw-forward
---
```

按行 grep：第一个 `name:` 等于文件名、`description:` 后面有非空白字符（`""`）⇒ 绿。按 PyYAML 解析：`description` 是空串、重复键取后者 `name` = `tw-forward` ⇒ 两项都该红。
Claude Code 自己按哪个 `name` 注册、重复键报不报错，没测；若按后者，仓里就有两个 `tw-forward`，第 147 行「按名字派」派到哪一个说不清。

不该红却红：

```
---
name: "tw-attack"
description: >-
  攻方腿：假设主 agent 的倾向错了，造可达历史去打穿。
tools: Read, Grep, Glob, Bash
model: opus
---
```

合法 YAML。按行 grep：`name` 取到 `"tw-attack"`（带引号）≠ 文件名、`description:` 同一行后面只有 `>-` ⇒ 两项误红。
⇒ 这一行要写明「按 YAML 解析、拒绝重复键」，样本要含引号、块标量、重复键三格。

### 2.2 权限写明（第 136 行「必须显式写 `tools` 与 `model`，不许默认继承全部工具；`tools` 里不许有 Agent」）

该红不红，四份各一形：

```
tools:            # ① 列表写法，按行 grep「^tools:.*Agent」不中
  - Read
  - Bash
  - Agent
model: sonnet
```

```
tools: Read, Bash, Task      # ② Task 是 Agent 工具的旧名；当前版本还认不认这个别名，没测
model: sonnet
```

```
tools: Read, Bash
model: inherit               # ③「显式写 model」满足了，实际跟主 agent 走，不是第 80–115 行表里的默认模型
permissionMode: bypassPermissions   # ④ 七行里没有一行看它：带 Bash 的 agent 不再弹权限，用户的权限系统是今天唯一的人工闸
```

加上 1.1 ① 的 Bash 起嵌套会话：`tools: Read, Bash` 字面满足「不许有 Agent」，实际能派嵌套会话。

不该红却红：

```
tools: Read, Grep, Glob, Bash
disallowedTools: Agent, Write, Edit
model: sonnet
```

两道都写上是更严的写法，而「frontmatter 里出现 Agent 就红」的实现当场误红。⇒ 判据要写成「`tools` 这个键解析出来的列表里没有 Agent / Task」，另加一行判 `permissionMode` 不许是 `bypassPermissions`、`model` 不许是 `inherit`。

### 2.3 四样齐（第 137 行「正文有「输入」「写范围」「产出」「没做什么」四节」）

该红不红：

```
## 输入
## 写范围
## 产出
## 没做什么

无。
```

四个标题都在，前三节是空的，「没做什么」写「无」——而第 129 行要求「Bash 管不住这件事照实写进每个定义的「没做什么」」。四个标题放进一个 ```` ```markdown ```` 围栏里当示例，按行 grep 同样判绿。

不该红却红：

```
## 输入（主 agent 必须给什么，缺一样就拒绝开工并说缺什么）
## 写范围
## 产出格式
## 没做什么（跑不动的、没验的）
```

这正是第 53 行的原词：「**输入**（主 agent 必须给什么，缺一样就拒绝开工并说缺什么）、**写范围**（能写哪些路径）、**产出格式**、**没做什么**（跑不动的、没验的）」。
按标题逐字相等的实现，对提案第 53 行自己的措辞判红（「产出格式」≠「产出」）。⇒ 判据要写成标题前缀匹配加「节内非空」，另加「没做什么节里有 Bash 那一句」。

### 2.4 写范围一致（第 138 行「「写范围」列的路径模式与 hook 参数逐项相等」）

该红不红：

```
---
name: kb-scribe
description: 定案之后照规格写回 kb
tools: Read, Edit, Write, Bash
model: sonnet
hooks:
  PreToolUse:
    - matcher: "Write"
      hooks:
        - type: command
          command: bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/agent-write-scope.sh '.claude/**'
---
## 写范围
- `.claude/**`
```

逐项相等 ⇒ 绿。而：matcher 只有 `Write`（照抄今天 `.claude/settings.json` 那一条的形状），Edit 不过 hook；`.claude/**` 包含 `.claude/settings.json`、`.claude/hooks/`、`.claude/gate.d/` 与 `.claude/agents/kb-scribe.md` 自己——agent 能改自己的闸。
这一行只判「两处一样」，不判「matcher 覆盖 Write 与 Edit」「范围不含 `.claude/agents/`、`.claude/hooks/`、`.claude/settings*.json`」。

不该红却红：照提案第 80 行的写法写 `tw-forward`：

```
## 写范围
- `research/prompts/<轮>-sonnet-output.md`
- `research/prompts/<轮>-sonnet-model/`
```

hook 参数写不进 `<轮>`（1.2 D），只能写 `research/prompts/*-sonnet-output.md` ⇒ 逐项不等 ⇒ 红。要过这一行，定义只能把「写范围」也写成静态模式——也就是说这一行**逼着**第 79–85、113 行那八个带 `<轮>` / `<号>` 的定义把范围放宽成全轮次。

### 2.5 不抄规则（第 139 行「正文里不许出现与 `rules/*.md` 逐字相同的长行（阈值实测后定），规则只指路径」）

模型 `gate_row5_threshold.py` 取 `.claude/rules/*.md` 与 `.claude/singlefs-ai-sop/rules/*.md` 全部非空行（去首尾空白），原样输出：

```
规则文件 20 份，非空不同行 2023 条
T=  8  ≥T 的行  1997  其中跨文件重复   7  最长的一条（66 字符）：'> **Gate proves evidence requirements, not semantic correctness.**'
T= 12  ≥T 的行  1945  其中跨文件重复   4  最长的一条（66 字符）：'> **Gate proves evidence requirements, not semantic correctness.**'
T= 16  ≥T 的行  1844  其中跨文件重复   3  最长的一条（66 字符）：'> **Gate proves evidence requirements, not semantic correctness.**'
T= 24  ≥T 的行  1619  其中跨文件重复   3  最长的一条（66 字符）：'> **Gate proves evidence requirements, not semantic correctness.**'
T= 40  ≥T 的行   933  其中跨文件重复   1  最长的一条（66 字符）：'> **Gate proves evidence requirements, not semantic correctness.**'
T= 80  ≥T 的行   145  其中跨文件重复   0  最长的一条（0 字符）：''
T=160  ≥T 的行    25  其中跨文件重复   0  最长的一条（0 字符）：''
```

另一次只读统计（同一批文件）：表格分隔行 4 种，长度 9 / 13 / 17 / 21；标题行 202 条、长度中位数 15；`## 门禁管哪一半`（9 字符）在 5 份规则里出现，`## 判据`（5 字符）2 份。长度按 Python 的 `len` 数字符，一个汉字算 1。

| 阈值 T | 该红不红（抄了不红） | 不该红却红 |
|---|---|---|
| 任何 T | 抄一段规则、改一个标点，或换一个折行宽度重排，或去掉 `**`：没有一行逐字相同。例：把 `.claude/rules/three-way-inference.md` 第 247 行整段抄进 `tw-attack.md`、去掉开头的两处 `**` | — |
| T ≥ 80 | 2023 行里 1878 行（93%）短于 80，整行照抄也不红；`tw-attack` 第 81 行要的「打中之后先答四句」原文是 evidence-discipline.md 第 186 行，长于 80，但同一节里多数短行可以原样搬 | 基本不误红 |
| T = 40 | 1090 行（54%）照抄不红 | 定义里引那句英文门禁标语（66 字符，跨文件重复）就红 |
| T ≤ 21 | 抄的几乎都红 | 定义里画一张五列表（分隔行 `\|---\|---\|---\|---\|---\|` 21 字符）就与规则里的表格分隔行逐字相同，红；T ≤ 9 时写一节 `## 门禁管哪一半` 就红 |

另两处这一行没写死的：`rules/*.md` 指项目 `.claude/rules/` 还是连共享 `.claude/singlefs-ai-sop/rules/` 一起（只管前者时，照抄 evidence-discipline.md 第 186 行那四问不红）；
这一行成立的前提是第 27 行「agent 定义里不用、也不该再抄规则」，而第 28 行自己说自定义 agent 继不继承规则没测——第 0 步测出不继承时，这一行拦的正是唯一的补救做法。
⇒ 「逐字相同的长行」在任何阈值下都挡不住改一个字的抄写；要判的量是「定义与规则的最长公共子串 / 归一化后的相似度」，或者干脆反过来判「交付规矩那一节里每一句都带一个 `规则文件:行号` 指路」。

### 2.6 登记（第 140 行「CLAUDE.md 的 agent 表与目录逐项相等（形态照 50 号）」）

50 号的形态是全文件 grep，不是按表：`.claude/gate.d/50-rules-manifest.sh` 第 17 行
`referenced="$(grep -oE '@\.claude/rules/[A-Za-z0-9._-]+\.md' "$MD" | sed 's|.*/||' | sort -u)"`，第 13 行
`[[ -d "$RULES" ]] || { echo "  ✓ 没有 $RULES，无对象可判"; exit 0; }`。

该红不红：
- CLAUDE.md 的 agent 表里没有 `prior-art`，而别处一句正文写了「`.claude/agents/prior-art.md` 放第三批」——照 50 号全文件 grep，算登记了 ⇒ 绿。
- 整个 `.claude/agents/` 被删掉：照抄第 13 行，判「✓ 无对象可判」、退出码 0，记通过。`.claude/singlefs-ai-sop/rules/show-me-test.md` 第 118 行「**扫到 0 项也不是通过。**」，项目本地阶段无对象要退 77。
- 定义放在用户级 `~/.claude/agents/`：本机每个会话都加载（`research/prompts/_agent-split-r1-background.md` 第 81 行），这道门禁不看仓外，一个名字相同或相近的用户级定义永远不进登记比对。

不该红却红：CLAUDE.md 的正文按提案第 158–160 行写了落地计划「第三批：`.claude/agents/exp-designer.md`……」，目录里还没有 ⇒ 照 50 号第 29 行那一支（引用了而文件不存在）判「引用悬空」红。
⇒ 要写明「只比 agent 表那一块（带起止标记），表外提到不算」，并且无目录退 77。

### 2.7 对抗过没有（第 141 行，形态照 56 号）

56 号的取法：第 17–23 行基准是 `GATE_BASE`、否则与 `@{upstream}` 的 merge-base、都没有就 HEAD，改动集合含工作区、暂存区与未跟踪文件；
第 29 行 `verdicts="$(grep -E '^research/prompts/.*-main-verification\.md$' <<<"$changed" || true)"`；第 38 行 `if grep -qF -- "$source" "$verdict"; then named=1; break; fi`。

该红不红，四个序列：
1. **执行计划自己的判决把路径点了。** 这一轮的判决 `research/prompts/agent-split-r1-main-verification.md`（以及 r2、r3）判的是一份点名十五个 agent 的计划，写出 `.claude/agents/tw-attack.md` 这样的路径是常态。
   它未跟踪或已提交未推送期间（窗口是与 upstream 的 merge-base），第 158 行第一批八个定义随后写进工作区 ⇒ 八个文件都被「同一次改动带来的判决」按路径点名 ⇒ 绿，一个 agent 自己的对抗都没做。
   现查：`git log origin/master..HEAD` 今天是 0 条，窗口此刻是空的；用户按 memory 索引「只提交不推送」攒提交时窗口会一直张着。
2. **点名出现在「没攻」的那句里。** 判决写「`.claude/agents/sweep.md` 这一轮没攻，留到下一轮」⇒ `grep -F` 命中 ⇒ 绿。
3. **第一轮点了名，后两轮没做。** 第 167 行「打中的写回定义再攻，按「三轮里多数打穿才算」」：第一轮判决点名 → 定义按打中写回（又改了一次）→ 第二、三轮没派 ⇒ 绿。这一行只问「有没有一份判决点名」，不问「点名的那一轮攻的是不是现在这版」「有没有三轮」。
4. **别的会话的判决。** 不带 `--staged` 跑时，工作区里别的会话未跟踪的 `*-main-verification.md` 也在改动集合里（第 23 行 `git ls-files --others`）；它若引了一段门禁红行（带 `.claude/agents/<名>.md`），就算点名。

不该红却红：
- 第三批试跑后决定删掉 `prior-art`：`git diff --name-only` 列出被删的路径 ⇒ 要一份判决点名一个已删除的定义，否则红。
- `description` 里改一个错字 ⇒ 要一整轮对抗。按 A5 的口径，一轮至少 5 次派发、689,070 token。

⚠️ 这一行是 56 号的形态，1、2、4 三形在 56 号对 `crates/*/src/*.rs` 上同样成立 ⇒ 按跑前条款 K3，这三形另记成「56 号那个形态」的共用前提一笔账，不单判提案；3 是提案新加的「三轮多数判」要求而判据没跟上，归提案。

## 三、A3（J3）操作序列

每一步后面括号里是提案里许可它的那一句（行号是提案自己的）。

### 3.1 ① 腿读到别的腿这一轮的东西

**序列 ①-a：禁读范围只罩 `-output`，另一条攻方腿的提示文件不在里面。**
1. 主 agent 写正文，派 `tw-materials` 出背景材料（第 147 行「主 agent 写正文 → `tw-materials` → 三条腿同一条消息并行」）。
2. 同一条消息派 `tw-attack` 与 `tw-local-attack`（第 147 行；第 192–195 行每轮都派本地攻辩时，两条攻方腿「得分到不同的攻击面」）。
3. `tw-local-attack` 把分给本地攻方的攻击面译成英文、逐句补限定词，写进 `research/prompts/<轮>-local-attack.md`（第 83 行写范围 `research/prompts/<轮>-local-attack*.md`）。
4. `tw-attack` 找背景材料时列 `research/prompts/<轮>*`，看到并读了 `<轮>-local-attack.md`：它不是 `-output` 文件。
   第 88 行交付规矩写的是「不读别的腿这一轮的产出」；今天落到提示里的形状是 `research/prompts/_agent-split-r1-background.md` 第 80 行「不许读别的腿这一轮的产出（`research/prompts/agent-split-r1-*-output*.md`）」。
   按定义要把这句写死（第 88 行「从此只写在三个定义里」），定义里只能写静态 glob，`<轮>-local-attack.md` 与 `<轮>-sonnet.md` 这类提示文件都不在 `*-output*` 里。
5. 结局：Opus 攻方拿到了本地攻方的攻击面与补过限定词的转述，第 194 行要的「两条攻方腿分到不同的攻击面」在这一轮失效，而两条腿的打中之后被当成两票独立观测。
- 什么现象会推翻它：定义里的禁读写成「这一轮 `research/prompts/<轮>*` 下除背景材料与自己的文件以外一律不读」，并且派发时能把 `<轮>` 交给定义（回到 1.2 D 那个参数问题）。
- ⚠️ 读文件只靠定义禁止，提案第 39 行自己写了；这一条打中的不是「定义拦不住」，是「定义里能写下的那一句，字面就放行了这一类文件」。

**试过没打中的形状**：核查员读各腿报告（第 85 行）——次序是腿在前、核查员在后（第 147 行），核查员不回写给腿，读到不构成污染；
腿撞周限额续做时看到核查员的产出——核查员产出叫 `<轮>-verifier-output.md`，落在 `*-output*` 里，按定义禁读，要腿违规才发生，不算提案许可。

### 3.2 ② 三方证据被污染

**序列 ②-a：草稿目录按 agent 名字建，两个会话同时跑三方就互相覆盖。**
1. 第 87 行交付规矩：「草稿放 `/tmp/claude-1000/<腿名>/`」，并且「从此只写在三个定义里，每轮提示只剩立场专属的问题」。
2. 定义不知道轮次（1.2 D），`<腿名>` 在定义里只能是 `tw-attack` 这类静态名。今天的做法是按轮起名（这一轮是 `/tmp/claude-1000/agent-split-r1-opus/`）。
3. 会话甲的 C 轮与会话乙的 D 轮同时派 `tw-attack`（`.claude/agents/` 被本机每个在本仓开的会话加载；工作区里今天另有会话的改动 36 + 115 项，`research/prompts/` 下有它们的三方材料）。
4. 两条腿都往 `/tmp/claude-1000/tw-attack/seg_*.md` 写草稿，一条覆盖另一条。`.claude/rules/three-way-inference.md` 第 245 行记的正是这件事：「两条腿的草稿都叫 `seg_*.md`，一条把另一条的两段覆盖了」。
   `refuse-overwrite-untracked.sh` 第 19 行 `root=… || return 0` 对仓外放行，腿也没有 Write 工具，没有任何一层看得见。
5. 结局：某份 `<轮>-opus-output.md` 里有另一个会话另一个问题的段落，落盘后作为证据冻结。
- 什么现象会推翻它：定义里的草稿目录带会话或派发 id（例如由主 agent 在提示里给出），而不是 `<腿名>`。
- K3 核对：`.claude/rules/three-way-inference.md` 第 245 行自己写的也是 `/tmp/claude-1000/<腿名>/`；今天不撞，靠的是每轮提示把 `<腿名>` 写成带轮次的名字（这一轮是 `agent-split-r1-opus`）。J5 基线的模板若也写静态名同样撞 ⇒ 撞不撞取决于「谁在派发时填轮次」，提案第 88 行把这一步从提示里拿走了，基线没拿走；按 K3 记成半共用。
- 第 117 行「「只写报告」的 agent 报告放 `/tmp/claude-1000/agents/<名字>/`，回复只给路径；要进三方判决当证据的，主 agent 再拷进 `research/prompts/`」同形：
  两个会话各派一次 `gate-triage`（或同一会话修前、修后各派一次），报告目录同名；主 agent 按回复里的路径去拷时，拷到的可能是另一次派发写的那份。

**序列 ②-b：交付规矩挪进可变的定义文件，冻结的提示文件不再是「发给腿的输入」。**
1. X 轮按定义 v1 派 `tw-attack`；`research/prompts/X-opus.md` 里只剩立场专属的问题（第 88 行）。
2. 后面某一轮打中了 `tw-attack` 的定义，按第 167 行「打中的写回定义再攻」改成 v2（例如改分段行数、改禁读范围、改 `maxTurns`）。定义与今天的 `CLAUDE.md` 一样可以多日不提交。
3. 结局：X 轮的输入 = `X-opus.md` + 定义 v1 + 当时继承的规则；v1 没有任何冻结副本，`X-opus.md` 也没记定义的哈希。
   `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 第 51 行「**改一个字，产物就不再对应它的输入**」、第 54 行「所以改提示只能连同重跑一起改」——定义现在是提示的一部分，改它不重跑。
- K3 核对：J5 基线（提示引用 `research/prompts/_leg-delivery.md`）改模板文件时同样出错 ⇒ 这一形**大半是共用前提**，按 K3 另记一笔账，不单判提案。
  提案独有的一截是 frontmatter 里的 `tools`、`model`、`hooks`、`maxTurns`、`permissionMode`：它们改变腿的行为，却不是任何提示文本的一部分，基线里没有这一截（基线派 general-purpose，工具集固定、模型在派发时给）。
- 什么现象会推翻它：主 agent 每次派发时把定义全文的 sha256（或整份定义）写进 `research/prompts/<轮>-<腿>.md`。

**序列 ②-c：核查员复跑腿的模型，写进了那条腿的模型目录。**
1. 核查员的活包括「复跑是不是逐字节一致、sha256 对不对」（第 85 行），写范围只有 `research/prompts/<轮>-verifier-output.md`。
2. 腿的模型多数把产物写在自己旁边（`<轮>-opus-model/` 下），复跑命令原样执行就会覆盖那些产物文件。
3. 若 Opus 腿撞周限额正在等续做（第 149 行），而主 agent 先对已完成的腿派了核查员（第 147 行只写了次序，没写「等全部腿交齐」），核查员复跑时那条腿的模型目录里的文件被改写。
4. 第三层会报「疑似」，但每轮本来就有并行腿的「疑似」（1.3 不该红却红 ①），这一条混在里面。
5. 结局：续做的腿按 `.claude/rules/three-way-inference.md` 第 242 行「先 `ls -la` 看时间戳与大小」，看到的是核查员复跑之后的文件；报告里的 sha256 与那一刻的文件对不上。
- 什么现象会推翻它：核查员的定义要求复跑前把腿的模型目录拷到自己的临时目录再跑，或第 147 行写明核查员等全部腿交齐。

### 3.3 ③ 判决实际由 subagent 做出

**序列 ③-a：推论经观测类核查员一过，就从「逐条现查」降成「抽查」。**
1. `tw-attack` 报一条打中 H，引用写成「`invariants.md:1531`」——那是背景材料的行号。`.claude/rules/three-way-inference.md` 第 247 行记过真事：「辩方腿报告写「16-发布语义.md:554」「invariants.md:1531」，而那两份文件全文只有 419 / 440 行……主 agent 逐条回查才对上」。
2. `tw-verifier` 按第 85 行查「行号是不是误写成背景材料的行号」，核对表里 H 那一行标 ✗；「交核对表，不写采纳与否」。
3. 主 agent 读核对表。第 49 行「观测类交的是可复核的结果（命令加原样输出），主 agent 抽查就能用」——核查员是观测类，它的表抽查即可；
   第 50 行「推论类交的结论只算三方里的一票，主 agent 逐条现查才采纳」——而 H 的逐条现查，核查员已经做过一遍，结论是 ✗。
4. 结局：主 agent 在核对表上抽几行、都与表相符，H 按「引用有误」不采纳；那次真事里「逐条回查才对上」的那一步没人做。采纳与否实际由核查员那一个 ✗ 定下。
- 什么现象会推翻它：第 85 行的核对表对行号不符的每一处给出「在背景材料第几行、对应 kb 文件第几行」的映射（核查员本就有这两份文件），或第 50 行写明「核查员的 ✗ 不免除主 agent 对推论的逐条现查」。
- 强度：半强——提案没说主 agent 可以照抄核查员，第 65 行判决仍归主 agent；打中的是「第 49 行与第 50 行叠在一起，给了照抄一条字面许可」。

### 3.4 ④ `gate-triage` 判错归属

**序列 ④-a：书记员照定义跑的脚本改了范围外的文件，主 agent 给分诊的清单里没有它们 ⇒ 这一轮的红被判成「别的会话的」。**
1. 主 agent 定案一条分项从未定翻成已定，给 `kb-scribe` 逐条规格（第 66、112 行）。
2. `kb-scribe` 照第 112 行「分项翻状态跑 `relabel-item.py`」执行。它改写的文件由 `.claude/gate.d/lib-item-ref-status.py` 第 43–45 行决定：
   `('.claude/kb/**/*.md', 'records/**/*.md', 'research/**/*.md', 'research/**/*.rs', '.claude/rules/*.md')`，只排除 `/prompts/`（第 50 行）；`relabel-item.py` 第 56–78 行对其中每个带旧标签引用的文件 `write_text`。
   现数（只读统计，落在扫描范围、今天带「已定项 / 未定项 k」字样的文件）：`.claude/rules` 2 个、`records` 47 个、`research/**/*.rs` 115 个、`research/**/*.md` 1 个——都在第 112 行写范围 `.claude/kb/**` 之外。
3. 第三层比对：`.claude/rules/` 下今天 4 份是 ` M`（别的会话），改它们不进差集（1.3 该红不红 ①）；未脏的 `records/`、`.rs` 会进差集，报「疑似」，与并行腿的「疑似」混在一起。
4. 主 agent 派 `gate-triage`，输入「这一轮改过的文件清单」（第 105 行）= 书记员报的 `.claude/kb/` 那几份。
5. 22 号（分项引用写的状态与正文的两张索引表相符）或 44 号在 `.claude/rules/fs-design.md` 某行判红；它不在清单里、又确实是 ` M` ⇒ 分诊判「别的会话的改动」。
6. 主 agent 照 `.claude/singlefs-ai-sop/rules/session-wrapup.md` 第 53–54 行「不在就如实说「红了，但不是这一轮的」，别顺手去修——那是别的会话还没收尾」不修；用 `stage-mine.py` 只暂存 kb（第 68 行）。
7. 结局：提交里决策正文说已定，`.claude/rules/` 与 `records/` 里的引用改写留在工作区混进别的会话的脏文件；`--staged` 复跑 22 号在 HEAD 版本的旧标签上继续红，分诊继续判「别的会话」。
- 什么现象会推翻它：第 112 行书记员写范围写成 `relabel-item.py` 实际会写的那五类路径，或分诊的输入改成「派发前后的内容哈希差集」而不是主 agent 报的清单。
- ⚠️ `relabel-item.py` 那一段 docstring 说「两份变更史与 records/ 写的是当时的状态，不列」，那是「要人看」清单不列（第 82 行的 `continue`），改写那一步（第 56 行起）没有排除 `records/`。

**序列 ④-b：反方向——别的会话的红被判成「这一轮的」，主 agent 去修别人的半成品。**
1. 这一轮按新门禁「登记」那一行（第 140 行）要改 `CLAUDE.md` 的 agent 表；今天 `CLAUDE.md` 已是 ` M`（别的会话）。
2. 别的会话在 `CLAUDE.md` 里的一块改动让 50 号或 doc-lint 判红。
3. 分诊的输入是文件清单（第 105 行），粒度到文件，`CLAUDE.md` 在清单里 ⇒ 判「这一轮的改动」。
4. 主 agent 去修 ⇒ 正是 session-wrapup.md 第 54 行禁的「顺手去修」。
- 第 105 行给的补救是「几个会话在飞时加 `--staged`」。但暂存是主 agent 在提交前做的（第 68 行），第 105 行没写分诊在暂存之前还是之后派；
  暂存之前跑 `--staged`，临时树里是 HEAD + 别的会话已暂存的块，这一轮的改动一行都不在，红的全是别人的，分诊按清单仍可能判成「这一轮」。
- 什么现象会推翻它：第 105 行写明「分诊只在主 agent 按 `stage-mine.py` 暂存之后派、只跑 `--staged`」，输入给到块级（暂存区 diff）而不是文件清单。

### 3.5 ⑤ `sweep` 的分类漏掉一处该改的

**序列 ⑤-a：真仓 2026-09-16 已经发生过一次。**
1. 格式常量「树表条目宽」148 → 200。主 agent 派 `sweep`，输入 旧值 148、新值 200、「它是哪个量」= 树表条目宽（第 106 行）。
2. `sweep` 照 `.claude/rules/format-evolution.md` 列的派生形态逐类搜：字面量 148、下标 147、倍数（`7 × 148 = 1036`）、商与平方（`⌊16253 / 148⌋ = 109`、`109²`）。
3. E146 装置里那一行是 `const TREE_TABLE_ENTRIES_PER_UNIT: u64 = 112;`（`git show d2aeb7d -U0 -- 'research/e7-index-bench/src/bin/e146*'` 删除行原样）。
   112 = ⌊16253 / 145⌋（`python3 -c "print(16253//145, 16253//148, 16253//200)"` → `112 109 81`）：它是 2026-09-13 条目宽还是 145 时写的（`git log -S'TREE_TABLE_ENTRIES_PER_UNIT: u64 = 112'` 首次出现在 d098c77，2026-09-13），145 → 148 那次就没跟上。
   148 的任何一种派生形态都不在那一行里 ⇒ 搜不到 ⇒ 不进分类。
4. 若 `sweep` 顺手也搜「每单元装几棵」的名字或 112 这个数，它会同时撞到 `crates/singlefs-core/src/records.rs` 第 1、112 行的「extent 叶记录 112」与 E145 产物里的 `leaf_entry_bytes=109`——那两处是「同一个数不同的量」，不改是对的；
   而 E146 的 112 是「**不同的数、同一个量**」（每单元树表条数，D22 口径是 109，E146 写的是 112）。第 106 行的三个桶「要改 / 在历史节或冻结证据里不改 / 同一个数不同的量」里没有这一格。
5. 结局：`format-evolution.md` 第 42–43 行记的这次真事里被漏掉的那一处（「E146（livelist 条目按映射 key 定身份之后的宽度与代价） 的「每单元 112」与三行断言」），`sweep` 照定义做同样漏掉，报告里 0 处「要改」给它。
- 这一形在 `.claude/singlefs-ai-sop/rules/show-me-test.md` 里有镜像（2026-09-13 那次「两个量，一个名字，一个值」）；这里是「一个量，两个式子，两个值」，而「旧值的派生形态」按定义只从旧值出发，追不到更早一代的派生。
- 什么现象会推翻它：`sweep` 的输入加上「这个量的全部已知式子（分子、分母）」并按量名搜全部数值，或分类加第四桶「不同的数、同一个量 ⇒ 要人看」。
- K3 核对：基线里主 agent 自己搜也漏了（format-evolution.md 第 42 行就是基线的实测）⇒ 漏本身是共用前提；提案独有的一截是**第 106 行把分类固定成三桶**，逼着这一格落进错桶或被丢掉。

## 四、A4（J3）限额续做与资源锁

提案第 149–150 行（整行抄）：
「- **限额**：撞周限额续同一个 agent（它的上下文还在），撞单次输出上限重派，照 `three-way-inference.md` 现有两条。」
「- **机器资源**：跑模型一律 `nice -n 19`；编译 Rust、QEMU、E152（按里程碑对比六家文件系统的文件性能） 这类重负载同一时刻只跑一个。第二批加一个取锁脚本（`flock`），`impl-writer`、`exp-runner`、`gate-triage` 开跑前取锁，取不到就报「在等谁」而不是硬跑。」

### 4.1 撞单次输出上限重派：排他新建撞上前一次的半份报告

1. `tw-attack` 已排他新建 `<轮>-opus-output.md` 并追加了两段，第三段想一次吐太多，撞单次输出上限（第 149 行）。
2. 主 agent 照 `.claude/rules/three-way-inference.md` 第 240 行「先 `ls` 产物（多半是空的），按分段写的提示重派，不要续那条腿」重派同一个 `tw-attack`。这一次「多半是空的」不成立：文件里有两段。
3. 新派出的 `tw-attack` 带着同一份定义：第一段排他新建 `<轮>-opus-output.md`（第 87 行）⇒ 文件已存在 ⇒ 照交付规矩「文件已存在就停下报告」停下。
4. 主 agent 三条出路都出事：删掉半份报告——它是原样输出（evidence-discipline.md 第 50 行「发给模型的提示、跑出来的原始输出、当时的日志——这些和产物一一对应」）；
   让新腿写 `<轮>-opus-output-v2.md`——不在第 81 行写范围 `research/prompts/<轮>-opus-output.md`，而且要改提示，第 88 行说提示「只剩立场专属的问题」；
   让新腿 `>>` 接着写——同一个文件里是两次独立派发的段落拼在一起，标题重复，核查员与主 agent 按一份报告读。
- 今天不撞的原因：主 agent 每轮手写提示，重派时顺手给新文件名。提案把文件名收进定义之后，这一步没有地方放。
- 什么现象会推翻它：定义里的产出文件名带派发序号、由主 agent 在派发时给；或第 149 行写明「重派前把半份改名为 `-void<n>` 留存」。

### 4.2 撞周限额续做：续的是旧定义

1. `tw-attack` 撞周限额（第 149 行续同一个 agent）。`.claude/rules/three-way-inference.md` 第 243 行记过：「两条 Opus 攻方腿同时撞上周限额……等的那两个小时用来做不依赖那条腿的写回」。
2. 等的那两个小时里，主 agent 把另一轮对 `tw-attack` 定义的打中写回（第 167 行「打中的写回定义再攻」）。
3. reset 后 SendMessage 续做：续上的上下文里是 v1 的系统提示；frontmatter 里的 `hooks` 是派发时装上的还是每次工具调用从盘上重读的，没测。
4. 结局：同一份报告前半按 v1、后半按 v1 的上下文但可能按 v2 的 hook 写；冻结目录里只有 v2（序列 ②-b 的放大形）。
- 这一格是推测：「续做时 frontmatter 与 hook 取哪一版」要第 0 步测（派一个带 hook 的最小 agent，中途改盘上定义里的 hook 参数，再续一次，看 hook 拒绝的路径变没变）。

### 4.3 取锁：Bash 工具每次调用是一个新 shell

现查 `ps`：本机 Claude Code 的每次 Bash 调用是一个 `/bin/bash -c source …/shell-snapshots/snapshot-bash-….sh …` 进程。`flock` 的锁随持锁进程退出而释放。
「开跑前取锁」（第 150 行）于是只有两种做法，各卡在一处：

| 做法 | 卡在哪 | 结局 |
|---|---|---|
| 整个 agent 生命周期持锁：开跑时起一个后台持锁进程（`flock 锁 sleep …` 放后台） | agent 撞周限额暂停 2 小时——持锁进程还在；agent 撞输出上限被重派、或用户中断——持锁进程可能还在（subagent 结束时它起的后台进程杀不杀，没测） | 锁成孤儿：`gate-triage` 与 `exp-runner` 每次派发都报「在等 impl-writer」，而没有任何重负载在跑；要清它得按写死的 pid 杀（`.claude/singlefs-ai-sop/rules/command-safety.md` 禁 `pkill -f`），提案没写 pid 记在哪 |
| 每条重命令各取一次（`flock -n 锁 命令`） | `gate-triage` 跑全量 `gate.sh` 超过 Bash 工具 600000 ms 的上限，只能后台跑，锁随后台进程持有到门禁跑完 | `impl-writer` 的 `check.sh` 取不到锁 ⇒ 报「在等谁」并结束；主 agent 过一会儿重派，每次重派至少付一次 137,814 token（A5 口径）；轮询几次就是几十万 token |

**死锁（有条件）**：第 150 行说「QEMU 这类重负载同一时刻只跑一个」，最顺手的落点是把 `flock` 也写进 `research/scripts/vm-bench.sh` 或 55 号阶段。
一旦那样，`gate-triage` 开跑前取了锁、它跑的 `gate.sh` 走到 55 号再取同一把锁：子进程重新打开锁文件，`flock(2)` 在不同的打开文件描述之间互斥 ⇒ 阻塞写法永远等下去；`-n` 写法 55 号判红、报「在等 gate-triage」，分诊把它判成环境。
- 什么现象会推翻它：锁只在 agent 定义里取、脚本里永远不取，并且有一道检查守着这一点。

**饿死 / 静默污染**：锁只有三个 agent 取（第 150 行）。主 agent 自己直接跑 `check.sh`、三条腿跑模型（只 `nice -n 19`，`nice` 不管 I/O 优先级）、另一个不用 agent 的会话跑 E152 性能测量，都不取锁。
`gate-triage` 取到空锁就开全量门禁，与另一会话的 E152 同时跑：E152 的数被挤偏而不红（性能数没有断言会红），门禁里的计时复跑（`CLAUDE.md` 第 72 行 `replay.sh`「计时实验另有把 kb 里的数钉住的区间断言」）若判红，分诊判「环境」。
- 什么现象会推翻它：锁写在被调用的重负载脚本入口、对所有调用者生效（那就回到 4.3「死锁（有条件）」那一段，两者要一起设计）。

**试过没打中的形状**：三条腿并行各取锁互相等——腿不在第 150 行的取锁名单里，不取锁，不会互等；`flock` 锁文件放在 `research/` 下被提交——第 150 行没写位置，这条腿没法判。

## 五、A5（J6）代价数

判据 J6（`research/prompts/_agent-split-r1-background.md` 第 52 行）：指出哪个取舍、缺哪个数；这一格记「缺代价数」，不算打中。

### 5.1 乘数与出处

| 乘数 | 取值 | 出处 |
|---|---|---|
| 每次派发的 token 下限 t₀ | 137,814 | 提案第 26 行：general-purpose、sonnet、不许调工具、0 次工具调用、「含它自己的回答」。只测了一次、只测了这一种类型；自定义 agent、Opus 没测。真腿要几十次工具调用，每次都带着这截上下文，t₀ 是松的下限 |
| 一轮的腿数 L | 4 或 5（本地攻辩每轮挑一个时 3 或 4） | 第 82 行「一轮只派正推与辩方里的一条」⇒ 正推或辩方 1 + 攻方 1 + 本地攻 1 + 本地辩 1 = 4；第 193 行却写「每轮都派时一轮有五条腿」。两句给的数不同，这条腿不判哪句对（J1 归正推腿），两个数都算 |
| 一轮的派发次数 N | 第一批期间 L + 1（核查员）；`tw-materials` 落地后 L + 2 | 第 147 行次序；`tw-materials` 在第二批（第 159 行），第一批期间材料由主 agent 写 |
| 每个 agent 的对抗轮数 R | 至少 2，常态 3 | 第 167 行「按「三轮里多数打穿才算」」；`.claude/rules/three-way-inference.md` 第 203 行「同一个结论再攻两轮，三轮里多数打穿才算打穿」；第一轮没打中时第 230 行「至少再抽一次」⇒ R ≥ 2 |
| 每个 agent 的试跑 | 1 次派发 | 第 168 行「拿一件真活派它一次」 |
| 执行计划自己 | 3 轮 × 4 腿 = 12 次派发（这一轮的形态：四条腿，无核查员） | 第 170 行；背景材料第 69–74 行 |

公式：每个 agent 落地 = R × N + 1；一批 = Σ（R × N + 1）；token 下限 = 派发次数 × t₀。

### 5.2 算出来的下限

| 情形 | 派发次数 | token 下限 |
|---|---|---|
| 一轮（L = 4，第一批期间 N = 5） | 5 | 689,070 |
| 一轮（L = 5，N = 6） | 6 | 826,884 |
| 执行计划三轮 | 12 | 1,653,768 |
| **第一批八个，逐个对抗**，L = 4、R = 3：8 × (3 × 5 + 1) | **128** | **17,640,192** |
| 第一批八个，L = 4、R = 2：8 × (2 × 5 + 1) | 88 | 12,127,632 |
| 第一批八个，L = 5、R = 3：8 × (3 × 6 + 1) | 152 | 20,947,728 |
| 第二、三批七个，L = 4、R = 3、N = 6：7 × (3 × 6 + 1) | 133 | 18,329,262 |
| **全部十五个，逐个对抗**，L = 4、R = 3 | **261** | **35,969,454** |
| 全部十五个，L = 4、R = 2 | 179 | 24,668,706 |
| 全部十五个，L = 5、R = 3 | 306 | 42,171,084 |
| 对照：**按批合并对抗**（一批三轮，第一批 3 × 5 + 8，第二批 3 × 6 + 4，第三批 3 × 6 + 3），L = 4 | 66 | 9,095,724 |

没算进去的：第 0 步的最小自定义 agent（至少三件事各一次）；第 172 行「每批铺完拿下一轮真活试一次」；撞输出上限的重派（4.1）、取锁失败后的重派（4.3）；核查员复跑模型的算力。
墙钟：这条腿的交付时限是约 45 分钟（提示给的预算，不是实测）；逐个对抗时第一批是 8 × 3 = 24 轮，按一轮不少于最慢那条腿的 45 分钟算，至少 18 小时，主 agent 是唯一派发者（第 55 行），轮与轮之间串行。
Opus 周限额：第一批逐个对抗要 24 次 `tw-attack`（第 81 行默认 opus）派发，下限 3,307,536 token 只算继承那一截；`.claude/rules/three-way-inference.md` 第 243 行记了 2026-09-16 两条 Opus 腿同一天撞周限额。限额按什么计量，这条腿不知道。

### 5.3 哪些取舍依赖这些数、提案没给（记「缺代价数」）

| 取舍（提案原文位置） | 缺的数 | 数怎么左右它 |
|---|---|---|
| 逐个 agent 对抗还是一批一轮（第 162 行「每一个 agent 落地都走一次正反对抗」；新门禁第 141 行只要「某份判决点名」，两种都过） | 两种各多少派发 | 128 次对 23 次（第一批，L = 4、R = 3），差 5.6 倍；用户定的是「每个都走对抗」，没定逐个还是合批（K4：这一项不攻，只报它缺数） |
| 本地攻辩每轮都派还是挑一个（第 192–195 行还开着的一问） | 每轮多一条本地腿的派发与 token | 每轮 +1 次、≥ 137,814 token；每个 agent 落地 R 轮就是 +R 次 |
| 第一批先铺哪几个（第 158 行「收益最直接」） | 收益那一侧：今天一份云端腿提示里交付规矩占多少、每轮省多少 token；拆完之后每轮多付几次继承 | 提案第 9–20 行只数了「规矩在提示里重写」的份数（110 / 62 / 35），没给每份多少字、折合多少 token。这条腿自己收到的提示里「交付」一节是 7 条要点，而落地第一批的下限是 1764 万 token——要多少轮才回本，提案没算 |
| 平铺不嵌套（第 55–57 行） | 平铺时主 agent 上下文每轮多吃多少（读各腿回复与核查表），嵌套时多一次编排 agent 的派发 | 第 55–57 行只给了定性理由；这一格与 J5 / 本地辩方那条腿有关，这里只报缺数 |
| 「一个 agent 管一类会反复派出的活」（第 47 行） | 每个 agent 预计多久派一次 | `prior-art`、`mutation-triage` 若一个月派不到落地轮数那么多次，落地对抗的代价高于它省下的提示重写 |

第 201 行自陈：「每派一次先付继承规则那一截 token（实测一次 137,814，含回答）；一轮会派几次、总量多少没算。」——5.2 是这条腿按提案原文算出的下限，不是实测。

## 六、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 第 186 行）

原文：「⇒ **打中之后先问四句**：它分不分辨臂？它要被判的系统区分的东西，系统当时看得到吗？它满足的是判据字面的哪一个分句？跑前条款给的每个改法，在打中的那几格上还中不中？四句都过，才按跑前条款判。」
这一轮的「臂」读作提案与 J5 基线（K3）；「跑前条款给的改法」读作 K2 挂起再攻、K3 另记共用前提、K6 攻方改法零轮。

| 打中 | 分辨臂（提案 vs J5 基线） | 系统当时看得到吗 | 满足判据字面哪一句 | 条款与提案自己的补救还中不中 |
|---|---|---|---|---|
| 1.1 ① Bash 起嵌套会话 | 基线派 general-purpose 同样有 Bash ⇒ 不分辨，但只有提案立了「`tools` 里不许有 Agent」这条会被读成「不能嵌套」的检查 | 第二行门禁只看 frontmatter，看不到 Bash 里起了什么 | J4「该红不红」 | 第 157 行第 0 步「能不能绕过」若只测 Agent 工具，还中；K3：不能嵌套这件事基线也做不到，记半共用 |
| 1.1 ② 第一批无 Write / Edit，前两层零对象 | 分辨：基线没有前两层，也不宣称它们拦了第一批 | 看得到（定义里写着类型） | J4「该红不红」（第一批的越界写只剩第三层） | 第 129 行「照实写进没做什么」写的是「Bash 管不住」，没写「前两层对第一批无对象」，还中 |
| 1.2 D 静态模式放行别的轮 | 分辨：基线没有 hook | 看不到：派发时无参数 | J4「该红不红」 | 还中；2.4 那一行门禁反而逼定义放宽 |
| 1.2 C `notebook_path` | 分辨 | hook 看得到工具名，照抄现成 hook 的取参不看 | J4「该红不红」 | 真实现未写，K6：改法零轮 |
| 1.3 已脏文件 / 被忽略路径 | 分辨：基线没有第三层，不宣称拦住 Bash 越界 | 看不到：比的是状态行 | J4「该红不红」 | 第 127 行「只能报疑似」说的是误报一侧，这两形是漏报，还中 |
| 1.3 并行腿互相进差集 | 分辨 | 看不到：git 不记写者 | J4「不该红却红」 | 还中 |
| 2.1–2.6 各行样本 | 分辨（基线没有这道门禁） | 看得到，差在判据没写死解析方式 | J4 两向 | 第 143 行 89 号样本若只按提案原句造，还中 |
| 2.7 ①②④ | 不分辨提案与 56 号：同形在 56 号上成立 | — | J4「该红不红」 | K3：另记 56 号形态的共用前提 |
| 2.7 ③ 只要一轮点名 | 分辨：三轮多数判是提案新加的 | 看得到（判决文件数得出轮次），判据没数 | J4「该红不红」 | 还中 |
| 3.1 ①-a 提示文件不在禁读里 | 半分辨：基线每轮手写禁读句、可写成按轮排除；提案把它收进定义后只能写静态 glob | 看不到：定义不知道轮次 | J3「腿读到别的腿这一轮的产出」 | 还中 |
| 3.2 ②-a 草稿目录按名字 | 半分辨（见该节 K3 核对） | 看不到 | J3「三方证据被污染」 | 还中 |
| 3.2 ②-b 定义可变 | 大半不分辨 | — | J3「产物与提示对不上」 | K3 另记；frontmatter 那一截归提案 |
| 3.2 ②-c 核查员复跑改腿目录 | 分辨：基线没有核查员 | 第三层看得到但淹在误报里 | J3「三方证据被污染」 | 还中 |
| 3.3 ③-a 核查员的 ✗ | 分辨 | 主 agent 看得到原文，但第 49 行许可它只抽查 | J3「判决实际由 subagent 做出」 | 半强，K2 挂起 |
| 3.4 ④-a / ④-b | 分辨：基线没有分诊 agent，主 agent 自己按块看 | 分诊只拿到文件级清单 | J3（门禁红的归属判错，按判据字面最接近「一道门禁该红不红」的「红了却判不到这一轮」） | 第 105 行 `--staged` 补救在暂存之前派时不中 |
| 3.5 ⑤-a E146 的 112 | 漏本身不分辨；三桶分类归提案 | 看不到：输入只有旧值、新值、量名 | J3（该改没改，按判据字面最接近「一道门禁该红不红」：27 号只核 `const 名字 = 值`，也没红） | K3 半共用 |
| 4.1 重派撞排他新建 | 分辨：基线重派时手写新文件名 | 看得到 | J3「三方证据被污染」 | 还中 |
| 4.3 孤儿锁 / 静默污染 | 分辨：基线没有锁 | 锁看不到持锁者死没死 | J3（资源一格不在判据四个结局里，判据字面没有「饿死」这一句 ⇒ 这一格按判据只能记线索，不记打中） | — |

⚠️ 第三列里 ④、⑤ 两格按 J3 字面的四个结局都不严丝合缝（J3 写的是「越界写、三方证据被污染、判决实际由 subagent 做出、或一道门禁该红不红」）；
④ 是「红了、被分错归属」，⑤ 是「该改没改、门禁本来也不红」。四句里第三句没全过，按 `evidence-discipline.md` 第 186 行「四句都过，才按跑前条款判」，这两格交主 agent 判是记 J3 打中还是记成线索。4.3 同理，这条腿自己判成线索。

## 七、没打中的形状（取样范围）

| 攻击面 | 试过的形状 | 为什么没打中 |
|---|---|---|
| 第一层 | 给观测类 `EnterWorktree` 之类的写工具 | 第 136 行要求显式 `tools`，不列就没有 |
| 第二层 | 仓外路径、兄弟目录前缀（`/home/fy5090/code/singlefs-ai-sop-zh/…`）| 去前缀后仍是绝对路径，不匹配任何仓内模式，两种写法都拒绝 |
| 第三层 | 新建未跟踪文件越界 | `git ls-files --others` 按文件列，模型第 2 格报出来了 |
| 新门禁「对抗过没有」 | `grep -F` 子串误命中（`tw-attack.md` 被 `tw-local-attack.md` 包含） | `.claude/agents/tw-attack.md` 不是 `.claude/agents/tw-local-attack.md` 的子串 |
| ① | 核查员读腿报告、续做的腿读核查员产出 | 见 3.1「试过没打中的形状」 |
| ② | `ask-local.sh` 判红后调用方重定向留下 0 字节的 `-output-s1.md`、与作废副本并列被当样本 | 同一形在今天的做法里同样发生（`ask-local.sh` 第 63–76 行与调用方重定向都不变）⇒ K3 共用前提，不判提案 |
| ② | 本地攻辩两腿并行跑 `ask-local.sh`，作废副本编号撞车（第 72 行 `while [[ -e … ]]` 再 `cp`，非排他） | 两腿提示文件名不同，`<base>` 不同，不撞；同一条腿并行跑同一份提示才撞，而「前台跑」禁了 |
| A4 | 三条腿并行互相等锁 | 腿不取锁 |

## 八、这条腿自己的限度

- 没跑任何 Claude Code 行为：嵌套会话、NotebookEdit 能否新建文件、matcher 是否子串命中、`Task` 别名、frontmatter hook 是否续做时重读、subagent 结束时后台进程杀不杀、重复 `name` 键怎么注册——全部是推测，每条都写了第 0 步怎么测。
- 模型里的 hook 与比对脚本是这条腿按字面写的，不是提案的实现；打中的是「按字面做会错」。
- 代价数只算了继承那一截的下限；t₀ 只有一次实测。
- 这条腿自己提的改法（每格「改法」或「什么现象会推翻它」里给出的做法）只在这条腿的推导里成立、被攻过零轮（K6）。


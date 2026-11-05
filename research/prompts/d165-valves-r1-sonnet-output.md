# d165-valves-r1 正推腿报告（Sonnet，三方正推）

日期 2026-11-01。背景材料 `research/prompts/_d165-valves-r1-body.md` 格 F；批次 `records/2026-10-29-一百三十七节那一轮调度记录.md#16`。草稿目录 `/home/fy5090/.local/share/singlefs/work/d165-valves-r1-sonnet`。没往统一任务服务交任务；没读禁读清单里的攻方腿报告与模型。

## 0. 读法与来源

- 引文一律写成「原文」（`路径:行号`），写进报告前逐句 `grep -nF` 过；行号是此刻主工作区的行号（与背景材料第二节的行号有出入处，以这里为准）。
- 被引文件此刻的 sha256 前 8 位，与 `research/prompts/d165-valves-r1-snapshot/files-sha256.txt` 逐份对过，七份全部一致，没有漂移：

| 文件 | sha256 前 8 位 |
|---|---|
| `.claude/hooks/runner-dispatch-guard.sh`（下称守卫） | f11bbf5d |
| `.claude/hooks/lib_snapshot_conflicts.py` | d2e70e42 |
| `.claude/hooks/lib_writer_lock.py` | 5bf2ef90 |
| `scripts/agents/dispatch-prompt.py` | dfc51ee5 |
| `scripts/gate/apply-writer-patch.py` | 057be868 |
| `.claude/main-agent.md` | b691d3df |
| `.claude/agent-common.md` | 05a0501d |
| 另读（不在快照里）`.claude/agents/kb-scribe.md` | fecc48f7 |
| 另读 `.claude/agents/implementation-writer.md` | 1d0a6a7d |
| 另读 `.claude/agents/clerk.md` | 66d0b01b |
| 另读 `scripts/kb/kb-spec-check.py` | 0273fccd |
| 另读 `scripts/gate/admission.py` | df98f7d0 |
| 另读 `.claude/rules/implementation-workflow.md` | 46252fd5 |

- `crates/` 里没有相关实现：`grep -rln '已判' crates/` 零命中（背景材料第二节已写，这里没重做，不是新观测）。方案按钩子、脚本、定义今天的文字谈。
- 「实测」的来源有两种，报告里分开标：
  - 〔读码〕：从上面几份源码推出的触发条件，每条带行号。
  - 〔会话记录〕：腿用只读 python 扫了本机 `~/.claude/projects/-home-fy5090-code-singlefs/*.jsonl`（2026-10-01 起的主会话记录），按 Agent 调用的 tool_use id 去重，取提示里行首的「…已判：<理由>」行。脚本在草稿目录 `scan_valves_final.py`、`scribe_count.py`。这些数是腿量的、主 agent 没复跑，当线索；命令在末尾「要主 agent 跑的命令」。只覆盖本机这一批会话记录，别的机器与更早的会话看不到。

## 0.1 六种放行行共用的一件事（先说，后面不重复）

「理由」没有任何东西核。`valve_reason` 只看三样：行首标签在、理由非空、理由不以 `<` 起头也不在无意义词表里——
「return None if not reason or reason.startswith("<") or reason in NONE_WORDS else reason」（`.claude/hooks/runner-dispatch-guard.sh:1002`）；
弄坏开关「if match is None or break_switch_is("valve-reason-ignored"):」（`.claude/hooks/runner-dispatch-guard.sh:999`）只是整体关掉它。
所以六种放行行在机制上都是「写一行就过一次」，理由写什么闸都收。〔会话记录〕里已经有理由与闸拦的东西无关的实例：
限额窗口那一行写过「只读判决与写 kb，不编译」「只写登记，不编译、不跑装置，不占限额窗口」（2026-10-05、10-06）——限额窗口按模型族判，与干什么活无关，这几行是借放行行绕过去，不是闸判错。

## 0.2 分类口径（照背景材料第 2 项）

| 类 | 含义 | 改什么 |
|---|---|---|
| 写法 | 派发提示写错，改写法就过 | 提示 / 骨架 / 定义里的教法，闸不动 |
| 闸判错 | 闸该自己认得出却认不出 | 改判法（哪个函数、读什么输入） |
| 例外 | 闸要拦的东西确有合理例外 | 常设豁免表（用户在 `records/2026-10-29-subagent拆分提案.md` 一百六十五节写的形态：登记一次、带出处与内容哈希、门禁核） |
| 该删 | 这道闸本身该删 | 写删了之后靠什么拦 |

下面每种情形写：触发条件（行号）→ 归类与依据 → 改法落点 → 什么现象会推翻它。

## 1. ⑦ 限额窗口已判（守卫 `model_verdict`，`.claude/hooks/runner-dispatch-guard.sh:1263-1282`）

拒的条件〔读码〕：主会话记录 8 天内有 failed 通知带「resets <时刻> (UTC)」，通知的 `model sent to the API` 与新派同族，现在没到那个时刻，且没被判成「已解开」。
「已解开」只有一种证据：「launched_at > notified_at and agent_id not in limit_failed」（`.claude/hooks/runner-dispatch-guard.sh:1273`）——撞限额之后同族又派出去过、那一个没再撞。
放行行只在最后一道条件里：「and valve_reason(prompt, "限额窗口已判") is None):」（`.claude/hooks/runner-dispatch-guard.sh:1276`）。

### 情形 1.1　窗口已提前解开，而撞限额之后还没有同族新派发
- 触发：`now < until`（失败通知里的重置时刻没到）、`lifted` 为假（撞限额之后一次同族派发都没有，第一次派发不可能有这个证据）。
- 〔会话记录〕152 次放行行里，理由点名用户说额度恢复 / 换号 / 刷新的有：2026-10-07（「额度恢复了」）、10-18（「账号已换」「额度恢复了」）、10-25（「不可能 opus 也应该允许派遣的」）、10-31（「额度才刷新的」）；背景材料第二节引的 d137 那次是同一类。
- 归类：**闸判错**。依据：失败通知里的重置时刻只是上界；闸把它当成事实，又只有「撞限额之后又派出去」这一条间接证据，提前恢复后的第一次派发永远过不去，只能靠放行行。
- 改法 A（让闸自己读到成功回合）：`model_verdict` 里把 `lifted` 那一整段（守卫 1271-1274）换成新函数 `window_lifted(window, transcript_path)`。读两处：主会话记录、`<记录名去掉 .jsonl>/subagents/agent-*.jsonl`（`is_running` 已经按这个路径找子 agent 记录，守卫 1145 起）。判据：有一条 `type` 为 assistant、没有 `isApiErrorMessage`、`message.model` 不是 `<synthetic>`、族等于窗口的族、`timestamp` 晚于通知时刻的记录。
  - 〔会话记录〕核过这几个字段在：20 余个撞限额的子 agent 记录，最后一条都是 assistant 加 `isApiErrorMessage` 真加 `error` 为 `rate_limit`；成功子 agent 记录里 assistant 记录的 `isApiErrorMessage` 为假、`message.model` 是真模型名（样本 `59734d6f-…/subagents/agent-a002ff7ba61c05240.jsonl`，85 条 assistant 全部如此）；主会话记录里 assistant 同样有 `message.model`，报错的那几十条是 `<synthetic>`。
  - 比现行强在两处：现行把「刚派出去、还没回合」的 agent 也算解开（竞态，派出去的那一刻就算过），新判据要真有成功回合；成功回合不要求是新派的 agent，主会话自己在同一族上继续出回合也算。
- 改法 B（没有任何活动可借时，唯一能产生证据的办法是探测）：新增 `.claude/agents/quota-probe.md`（`tools:` 为空、提示是固定的一句、`model` 走派发参数），`model_verdict` 里 `subagent_type == "quota-probe"` 时跳过窗口那一判（并发那一判照留）。探测起来有成功回合就由改法 A 的证据放行后面同族的派发；探测自己撞限额就是新的一条失败通知，窗口照旧。放行靠 agent 类型能做的事（没有工具，最坏结果是白花一次很小的调用），不靠提示里写的文字。这是**新增的放行口**，它会不会被拿来当新的单次放宽（反复派探测、探测里夹带活）留给攻方腿的 A 格，这里不下结论。
- 改法 C（可选，弱证据）：换了账号。守卫每次派发把「时刻、`〔账号配置〕` 的 `oauthAccount.accountUuid`」追加进守卫自己的状态目录；窗口属于通知时刻之前最近一条登记的账号，当前账号不同就算解开。〔读文件〕核过 `〔账号配置〕` 里 `oauthAccount` 有 `accountUuid` 这个键（没打印值）。**未核**：这份文件的格式跨版本稳不稳、钩子进程读不读得到；排在 A、B 之后，不做也不影响 A、B。
- 什么会推翻：（一）找到「同族有成功回合、而该族仍在限额」的记录（例如限额按别的维度分、成功回合来自另一个计费桶）——A 的前提就错；（二）〔会话记录〕里用户说的「额度恢复了」被后来的失败通知反证（恢复了又撞）——说明提前恢复的判断不可靠，改法 B 之后仍可能误放，但代价只是一次探测。

### 情形 1.2　理由与限额无关
- 触发：同上，但提示写的理由是活的性质（「只读判决与写 kb，不编译」「只写登记，不编译、不跑装置，不占限额窗口」「harness 档编译几次，在限额窗口里」，2026-10-05、10-06 的 kb-scribe、experiment-designer、investigator、implementation-writer 等）。
- 归类：**写法**。依据：见 0.1，闸按模型族判，不按活判；这几行是借放行行绕过去。
- 改法：去掉放行行后出路是等重置时刻、换族（派发参数 `model`）或情形 1.1 的探测。没有死路：族被拦的那一刻，探测和换族永远在。
- 什么会推翻：找到一种活在限额窗口里确实不耗该族额度（例如纯本地、不调用模型）却要走 Agent 派发——派发本身一定调模型，所以我认为不存在。

### 情形 1.3　失败通知缺「model sent to the API」，族被记成 opus
- 触发〔读码〕：「model_family(sent.group(1) if sent else "")」（`.claude/hooks/runner-dispatch-guard.sh:1124`）交空串，`model_family` 对空串返回「return INHERITED_MODEL_FAMILY if lowered in ("", "inherit") else lowered」（`.claude/hooks/runner-dispatch-guard.sh:1079`）即 opus，于是缺这一串的 sonnet 失败会挡后来的 opus 派发。
- 〔会话记录〕没观测到：按 `<task-notification>` 块配对后，271 条带重置时刻的 failed 通知全部带这一串（第一次用宽正则数出 5 条缺的，细看是代码文本里的串，不是通知）。所以这是读码推出的潜在情形，不是已发生的。
- 归类：**闸判错**（潜在）。改法：`session_dispatch_events` 里空串时取 `launches[<通知里的 task-id>][1]` 的族，仍取不到就不挡（宁漏不错拦）。
- 2026-10-06 的「今天没撞限额，照派」「opus 今天没撞限额」两行来历核不了：可能是 1.1，也可能是 1.3，没有证据分开，不归类。

## 2. ⑧ 规格检查已判（守卫 `kb_scribe_spec_verdict`，`.claude/hooks/runner-dispatch-guard.sh:1290-1332`）

〔读码〕判法的次序：
1. 放行行在函数第一句：「if valve_reason(prompt, "规格检查已判") is not None or break_switch_is("spec-check-skipped"):」（`.claude/hooks/runner-dispatch-guard.sh:1291`）——有这一行就直接返回，后面一样都不看，包括提示里点名的现成规格是不是红的。
2. 点名的规格文件：提示里写的工作根或会话临时目录根下、此刻**存在**、扩展名 `.json` `.md` `.txt` 的路径都进候选（「if os.path.isfile(path) and path.endswith((".json", ".md", ".txt")):」，`.claude/hooks/runner-dispatch-guard.sh:1297`）。
3. 候选为空、提示里又点了 `research/prompts/<名>-main-verification.md`，视为「给判决、写回员自己起草」，返回不判：「if not candidates and VERDICT_MENTION.search(prompt) and not break_switch_is("verdict-not-spec"):」（`.claude/hooks/runner-dispatch-guard.sh:1300`）。认判决的形状只有这一种：`VERDICT_MENTION`（`.claude/hooks/runner-dispatch-guard.sh:928`）。
4. 候选为空又没点判决：把整份提示当规格交 `scripts/kb/kb-spec-check.py`。
5. 每份候选交 kb-spec-check：退 0 或 1 算「认得出」，退 1 的 ✗ 行收进问题，退 2 算认不出；问题非空拒；「if recognized == 0:」（`.claude/hooks/runner-dispatch-guard.sh:1325`）一份认得出的都没有也拒。

### 情形 2.1　点名的现成规格是红的（退 1）
- 触发：步骤 5，问题非空。kb-spec-check 文件头第一句「kb 写回规格检查：派书记员（kb-scribe）之前，规格原文本身会不会让门禁红。」——红的规格派下去，门禁迟早红同一处。
- 〔会话记录〕2026-10-15 有一次理由是「现成规格已过 kb-spec-check.py，第 0 步四处由主 agent 判完写在下面；按判定改规格后再跑一次 kb-spec-check.py 到绿再写」，即带着没改好的规格派。
- 归类：**写法**。依据：先改规格到绿再派；依据路径是别的会话未提交的改动导致的红，机制已经在（kb-spec-check 的 ⑥ 认 `.claude/batch-scope` 登记的这一批范围与 `--allow-uncommitted`），也是「登记范围」不是放行行。没有死路。
- 什么会推翻：找到一份规格，kb-spec-check 判红，而门禁 doc-decisions 等实际并不红（闸误拒）——那就是 kb-spec-check 的判法问题，修 kb-spec-check，不是放行行的问题。

### 情形 2.2　给的是判决、让写回员自己起草，但判决不是 `*-main-verification.md` 这一种形状
- 触发：步骤 3 的 `VERDICT_MENTION` 不命中 → 步骤 4 把整份提示当规格 → 认不出（退 2）→ `recognized == 0` 拒。
- 定义自己的输入比闸宽：`required-inputs: 判决|规格, 草稿目录, 报告`（`.claude/agents/kb-scribe.md:8`），输入一节写「判决路径与要写回的条目编号」（`.claude/agents/kb-scribe.md:20`），没限定文件名形状。
- 〔会话记录〕321 次 kb-scribe 派发：320 次带「规格检查已判」行，1 次没带；141 次提示里点了 `-main-verification.md`，其余 180 次没点。没点判决文件而带放行行的 179 次里，168 次提示里没有名字像规格的文件、判决讲在正文或 records / 调度记录里，11 次点了名字像规格的文件（用「spec / 规格…json|md」的宽正则数的，存在与否看不出）；这 179 次里闸能认的只有「点名的规格文件」那一支，放行行是其余的唯一出路。
- 归类：**闸判错**。依据：定义允许的输入形状（判决不限名）比闸认得出的窄。
- 改法（守卫 `kb_scribe_spec_verdict`，`.claude/hooks/runner-dispatch-guard.sh:1290`）：
  1. 骨架 `scripts/agents/dispatch-prompt.py` 给 kb-scribe 打一行「判决：<填 判决或定案记录的路径，逐个列；给现成规格的删这一行，写「规格：<路径>」>」（替掉 216 行）。
  2. `kb_scribe_spec_verdict` 新增 `drafting_mode(prompt, project_root)`：行首「判决：」一行里至少有一个从仓根起的路径（`research/prompts/`、`records/`、`.claude/kb/` 起头）此刻存在且非空；`VERDICT_MENTION` 命中也算（保留旧形状）。成立时：候选为空直接返回不判；候选非空时照判候选（红的照拒），但退 2 的候选不计入「认不出」（它们是背景笔记，不是规格）。
  3. `recognized == 0` 那一拒只在非 drafting_mode 时有效。
  放行靠「判决文件存在」这一个结构条件，不靠自由文字；写一个存在的无关文件能过，这条攻面留给攻方腿的 A 格（写回员起草规格时自己还要跑 kb-spec-check 到绿，提交时门禁也判，所以最坏是早一步没拦住，不是漏过）。
- 什么会推翻：168 次里有一部分其实是 2.1 或 2.3（判决路径在、但同时点名了存在的非规格 `.md`）——这样归 2.2 的数偏高，但改法不变；若 `VERDICT_MENTION` 之外的判决形状从来没有被闸拒过（放行行是白写的），则 2.2 退化为「放行行多余」，结论更简单（去掉就行）。

### 情形 2.3　判决形状对、同时点名了存在的非规格文件
- 触发〔读码〕：步骤 2 把任何存在的 `.md/.json/.txt`（前一份腿报告、调度草稿）都收成候选，候选非空 → 步骤 3 的 `not candidates` 为假 → 步骤 5 全部退 2 → `recognized == 0` 拒（`.claude/hooks/runner-dispatch-guard.sh:1297`、`:1300`、`:1325`）。
- 〔会话记录〕无法验证（点名的文件当时存不存在，事后看不出）。**待主 agent 跑**：下面「要主 agent 跑的命令」里 p8a / p8b / p8c 三条探针。我的预测写在跑之前：p8a 不出 ⑧ 的拒；p8b（判决形状对 + 点名一份存在的 `probe-notes.md`）出「认不出一份规格」；p8c（判决写 records 路径）出同一句。p8b 若不拒，2.3 推翻。
- 归类：**闸判错**（预测成立时）。改法并进 2.2 的第 2 条（退 2 的候选在 drafting_mode 下不计入）。

### 情形 2.4　纯译文，没有规格可起草
- 〔会话记录〕2026-10-22 一次：「纯译文，不改任何中文条款与决策状态，没有写回规格可起草」。
- 触发：步骤 4/5，整份提示不是规格，也没有判决形状。
- 归类：**闸判错 / 定义缺口**（只有这一次观测，证据弱）。依据：kb-scribe 定义的输入没有「译文」这一种；译文是项目里真实存在的活（kb-spec-check 文件头专门写了 `.en.md`、`.ja.md` 的译名规则）。改法：先在 `.claude/agents/kb-scribe.md` 输入一节加一种「译文：<中文源文件路径>」，`drafting_mode` 认这一行（路径存在即成立）；如果主 agent 判译文不归 kb-scribe，则归**写法**（改派别的 agent），这条留给主 agent 判。

### 第 4 问：`.claude/main-agent.md` 第 100 行与 `scripts/agents/dispatch-prompt.py` 第 216 行每次都写的「规格检查已判」
- 出处：「派发提示写一行「规格检查已判：写回员自己起草并跑规格检查」过派发闸 ⑧」（`.claude/main-agent.md:100`）；骨架「"kb-scribe": ["规格检查已判：写回员自己起草并跑 kb-spec-check.py"],」（`scripts/agents/dispatch-prompt.py:216`），文件头 19 行也把它列作写回员的必填行。
- **是不是等于 ⑧ 对写回员恒不判：是，对骨架出来的派发是。** 依据是上面步骤 1：放行行在函数第一句，有就整体返回（`.claude/hooks/runner-dispatch-guard.sh:1291`）；〔会话记录〕320/321 次带这一行。连「点名的现成规格是红的」这一半（情形 2.1）也一并不判了——2.1 的实例（2026-10-15）就是带着放行行派出去的。
- 它**有多少是必要的**：步骤 3 已经让「给 `*-main-verification.md`、没点存在的非规格文件」的派发不需要放行行（141 次里至少有一部分白写）；其余靠放行行的，是 2.2 / 2.3 / 2.4。
- 让 ⑧ 不靠放行行就认得出正常派发：落地就是 2.2 的三步——骨架改打「判决：」行，`drafting_mode` 认它，`recognized == 0` 与退 2 只对非 drafting 生效。同时：
  - 删 `scripts/agents/dispatch-prompt.py:216` 的放行行、文件头 19 行里写回员的「规格检查已判」、`RETIRED_SKELETON_LABELS` 不动；
  - 删 `.claude/main-agent.md:100` 里「派发提示写一行「规格检查已判：…」过派发闸 ⑧；」这一句，改写成「派发提示给「判决：<路径>」一行，派发闸 ⑧ 认它」；
  - `.claude/agent-common.md` 派发闸那一条（第 95 行）里「规格检查已判」的写法一并去掉；
  - 守卫里 `valve_reason(prompt, "规格检查已判")`、弄坏开关 `spec-check-skipped`、自证用例「规格:写了放行行」（守卫 3211 行）、拒绝说明里两处「确要照这份派，提示里写一行「规格检查已判：<理由>」」（守卫 1324、1327）一起改。
- 去掉之后卡死的正常派发：给的是现成规格且红 → 改规格（2.1，有出路）；给判决但判决文件不存在 / 路径写错 → 先补文件（有出路）；纯译文 → 2.4 的新输入。我没找到去掉后没有合规出路的正常派发。

## 3. ⑨ 准入已判（守卫 `designer_admission_verdict`，`.claude/hooks/runner-dispatch-guard.sh:1342-1385`）

〔读码〕只在派 experiment-designer 且提示里有 `research/prompts/e<号>-r<n>-prereg.md`（`.claude/hooks/runner-dispatch-guard.sh:987` 的 `RERUN_REGISTRATION`）时判。放行行与「没写重跑登记」「弄坏开关」并列在函数第一句：「or valve_reason(prompt, "准入已判") is not None:」（`.claude/hooks/runner-dispatch-guard.sh:1347`）。之后起 `scripts/gate/admission.py experiment <根> E<号>`（起之前清掉 `SINGLEFS_EXPERIMENT_RERUN_REASON`），三种结局拒：退 77、退 2、退 3。〔会话记录〕共 5 次放行行（2026-10-05 三次、10-29 两次），样本太少，归类主要靠读码。

### 情形 3.1　退 2：准入登记里没有这个实验键（或登记有错）
- 触发：「if result.returncode == 2 and not break_switch_is("admission-unregistered-allowed"):」（`.claude/hooks/runner-dispatch-guard.sh:1361`）；admission.py 自己打了「准入登记里没有实验键」或「的准入登记有错」。
- 〔会话记录〕2026-10-05（「`.claude/gate.d/stage-inputs.tsv` 里没有 E156 那一行、admission.py 判不了」）、10-29（E172，「在 `.claude/gate.d/stage-inputs.tsv` 没有准入键」）。此刻 stage-inputs.tsv 里已有 E172 行（`grep -n '^E172' .claude/gate.d/stage-inputs.tsv` 命中，第 46 行），即 10-29 那一次之后补上了。
- 归类：**写法**。依据：拒绝说明本身给了出路——先在 `.claude/gate.d/stage-inputs.tsv` 接这个实验的准入（照 E142 那一行写）；放行行跳过的恰是这一步，跳过之后这个实验永远没有键、之后每次重跑都判不了。出路没有死路（登记是仓里的一次编辑，tooling-writer 或主 agent 都能做）。
- 配套的**写法**缺口：`.claude/agents/experiment-designer.md` 里没有一句教「新实验先接准入」（`grep -n 'stage-inputs' .claude/agents/experiment-designer.md` 无命中），该补一句输入或做什么，否则设计员每次都靠主 agent 现想。

### 情形 3.2　退 2：判法本身没跑起来（脚本不在、python 只打用法、什么都没打）
- 触发：同一行 1361，但 stderr 里没有 admission.py 自己的两句（`.claude/hooks/runner-dispatch-guard.sh:1364` 往后的「reported_by_admission」分支）。
- 归类：**机制问题，修工具**，不是放行行：拒绝说明第一条出路就是先让 admission.py 能跑。这一种在会话记录里没有放行行实例。放行行在这里等于「工具坏了也照派」，去掉它没有死路（修工具即可）。

### 情形 3.3　退 3：前提没齐
- 触发：「if result.returncode == 3 and not break_switch_is("admission-precondition-allowed"):」（`.claude/hooks/runner-dispatch-guard.sh:1378`）。
- 〔会话记录〕没有实例；自证里有一例「准入:前提没齐、写了放行行」，理由样本写「前提由用户当面确认过」。
- 归类：**写法**。依据：前提是 admission.py 读的登记（岔路单一行的状态）；用户当面确认过，就该把那一行翻成「用户已定」并记出处，再派——放行行里的「用户确认过」没有地方被核，而登记行有。
- 什么会推翻：找到一个前提，登记里翻不动（例如状态由别的流程算出来、人不能写）却又确要继续——目前没见到。

### 情形 3.4　退 77：单元键与上次产物头相同，而这次重跑的原因在键之外
- 触发：「if result.returncode == 77:」（`.claude/hooks/runner-dispatch-guard.sh:1358`）。
- 〔会话记录〕10-05 两次（「判据变了——…输入没变而判据变了，按新一次跑」；「被测实现变了——…」那一次其实是 3.1）、10-29 两次（「这次重跑不是因为输入变了，是装置要加退役项」；「装置输入变了（`JOURNAL_HEADER_BYTES` 487 → 496）加接准入，不是登记过的输入变动触发」）。
- 这一格里有三种来历，归类不同：
  1. 判据变了、被测代码在键的闭包里没变：判据若写在装置代码里，键会跟着变（admission.py 文件头「judge <判法入口闭包的键>」「line <文本>」），不会退 77；判据只写在登记 / 文字里，同一份产物换个读法是**重读产物，不是重跑**。归类：**写法**（重读不重跑；要真的重新取样，就先把判据放进键的输入，见 2.）。
  2. 键漏了一项输入（例如常量 `JOURNAL_HEADER_BYTES` 不在键的闭包里 / 登记行没点名这份数据）：归类：**闸判错（键的输入不全）**。改法：不在守卫，在 `.claude/gate.d/stage-inputs.tsv` 那一实验行里补 `line:` / `file:` 输入，让键自己变；这是 3.1 同一个编辑点。我没能从会话记录里分清 10-29 那两次到底是 1 还是 2（理由原文含糊），归不了的那一次写「核不了」。
  3. 同键有意再取一次（复核、再抽样、换机器）：规则上有出处——「一条腿只抽一次样不算一次观测」的做法（`.claude/rules/three-way-inference.md` 同名一节）与 `.claude/singlefs-ai-sop/rules/test-discipline.md`「单次观测不算数」；admission.py 自己也有出路：「强制重跑：设 SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>。」（`scripts/gate/admission.py:239`），理由写进产物头。
     归类：**该删 ⑨ 的 77 一支**。依据：登记重跑的设计员不起任何装置，真正会不会重跑由装置入口再判一次 admission；⑨ 在这里拦的是「白写一份登记」，代价小，而它的放行行还是单次文字。删了之后靠装置入口的 admission 退 77 拦（产物头同键则拒、重跑理由必须落进产物头）；⑨ 对 77 只留一行 stderr 提醒（不拒）。**这一条是腿的倾向，不是定论**：它把控制点挪到运行时，运行时那个 `SINGLEFS_EXPERIMENT_RERUN_REASON` 环境变量本身就是同一族的「写一行就过一次」放行口（`scripts/gate/admission.py:239`、守卫 `ADMISSION_CLEARED_VARIABLES`），不在六种之内，留给攻方腿与主 agent 另判。
- 什么会推翻 3. 的归类：找到一种「登记重跑」本身有害（登记一旦写下就被门禁当成必须有产物）的机制——那样 ⑨ 的 77 拒有独立价值，要改成常设豁免表。形态备选（只在推翻时用）：`.claude/gate.d/experiment-rerun-reasons.tsv`，一行 = 实验号、重跑号、原因类（闭集：复核 / 再抽样 / 换机）、出处（`records/<文件>.md:N〔原话〕`，判法同 `lib_abandoned_rounds.py` 的出处核法）、登记者会话、登记时的单元键；门禁核出处原话逐字命中、单元键等于 admission.py 此刻算的键、同一键同一原因类至多一行。

## 4. ⑯ 写者锁已判（守卫 `writer_lock_verdict`，`.claude/hooks/runner-dispatch-guard.sh:2039-2112`）

〔读码〕放行行只对「锁在、读得了、持锁者被判还在跑」这一支生效：「valve = valve_reason(prompt, "写者锁已判") is not None and not break_switch_is("writer-lock-valve-ignored")」（`.claude/hooks/runner-dispatch-guard.sh:2058`）、「if alive and not valve:」（`.claude/hooks/runner-dispatch-guard.sh:2077`）。判不了的锁（unreadable）放行行不接手（守卫 2089 的说明）。「还在跑」由 `writer_lock_holder_state`（`.claude/hooks/runner-dispatch-guard.sh:2015-2037`）算：持锁者是认领文件里的 agent，没认领的按提示 sha256 找回执；找到回执的按 `is_running`（没结束、记录 3 小时内动过，`RUNNING_RECENT_SECONDS = 3 * 3600`，`.claude/hooks/runner-dispatch-guard.sh:970`）；都没有的看派发是否在 10 分钟内。〔会话记录〕5 次放行行（2026-10-23 一次、10-30 三次，另有一次同日实现员），实例的理由都有明确的事实。

### 情形 4.1　旧持锁者是上一个客户端会话派的，已停，TaskStop 报找不到它，闸仍判「在跑」
- 触发〔读码〕：锁里登记的 `transcript_path` 不是当前会话时，守卫去读**旧**会话记录，并把整组 `launches, ended` 换成旧记录的：「launches, ended, _ = session_dispatch_events(transcript)」（`.claude/hooks/runner-dispatch-guard.sh:2021`）。当前会话里主 agent 对它做的 TaskStop（结果「No task found with ID」或「Successfully stopped」，由 `session_dispatch_events` 记进 `ended`，守卫 1110-1114）被函数参数 `ended` 带着，却在这一行被覆盖掉，没用上。旧记录里没有这条结束记录，只要旧记录 3 小时内动过，`is_running` 就判在跑。
- 〔会话记录〕2026-10-30 两次，理由原文：「持锁的 … 是客户端重启前派的、已中途停下没交回（TaskStop 报找不到它），这一位接手」（implementation-writer 一次、tooling-writer 一次）——主 agent 做了该做的 TaskStop，闸没看见。
- 归类：**闸判错**。依据：用户口径要求「是机制的问题就修机制」，这里主 agent 已经走了闸文本里写的正路，闸读错了地方。
- 改法：`writer_lock_holder_state` 开头把参数 `ended` 先存成 `current_ended`；2021 行读完旧记录后加一步：对 `current_ended` 的每个 agent_id 把 `ended[agent_id]` 取两边的较大值（agent 号全局唯一，合并不会误伤别的持锁者）。`is_running` 不改。自证加一例：旧会话记录里持锁者在跑、当前会话记录里有它的「No task found」→ 应放行、不用放行行。
- 什么会推翻：TaskStop 对重启前的 agent 报的不是 `ABSENT_TASK` 那种句式（守卫 1112 起的解析认不出）——那样要先看实际句子；〔会话记录〕理由里说「TaskStop 报找不到它」，所以句式应在。

### 情形 4.2　派发过了闸、随后被别的层拒了（权限分类器等），锁留下，没有 agent 起来，10 分钟内换提示重派
- 触发〔读码〕：锁在、没认领、记录里没有 sha256 对得上的回执，新提示的 sha256 又与锁里的不同，于是走到最后一句「return now - dispatched <= UNLAUNCHED_GRACE_SECONDS, None」（`.claude/hooks/runner-dispatch-guard.sh:2037`）：派发后 600 秒内一律算「刚派出去」。同一份提示重派走上一行的重试支（「if lock.get("prompt_sha256") == new_digest and not break_switch_is("writer-lock-retry-refused"):」，`.claude/hooks/runner-dispatch-guard.sh:2035`），不受影响。
- 〔会话记录〕2026-10-23 一次：「那次派发过了派发闸，随后被自动权限判定拒掉，没有 agent 起来；目录里除了锁没有别的文件」。
- 〔会话记录〕核过这件事在记录里**看得出来**：全部会话里 Agent 调用的 tool_result 为错误的有两类——「PreToolUse:Agent hook error」（1919 次，是钩子拒，锁不会建）和「Permission for this action was denied by the Claude Code auto mode classifier」（2 次，是闸之后才拒，锁会留下）。
- 归类：**闸判错**。改法：`session_dispatch_events`（`.claude/hooks/runner-dispatch-guard.sh:1086` 起）除了 `async_launched` 回执，再收 Agent / Task 的 tool_use（`input.prompt` 取 sha256，`writer_lock.prompt_digest` 同一算法，`.claude/hooks/lib_writer_lock.py:109`）和它的 tool_result 为错误的那些，交回 `rejected_digests`；`writer_lock_holder_state` 里锁的 `prompt_sha256` 在 `rejected_digests` 里，返回 `(False, None)`，不再等 10 分钟。grep 的针要加 `"is_error":true`。
- 什么会推翻：被拒的 tool_use 与 tool_result 在记录里不是同一个 `tool_use_id` 配对（我只数了总数，没逐对核）。

### 情形 4.3　持锁者中途停了没交回、也没人 TaskStop，3 小时内
- 触发：`is_running` 的 3 小时窗（`.claude/hooks/runner-dispatch-guard.sh:970`）：没有结束记录、记录 3 小时内动过的一律算在跑。
- 归类：**写法**。依据：拒绝说明里已写等它交回或换目录；确认它已停就 TaskStop 它（结果「找不到」或「已停」两种守卫都当结束，守卫 1110-1114），闸下一次就看得见；这条路在 4.1 修好之后跨会话也走得通。没有死路。

## 5. ⑫ 通用已判（守卫，`.claude/hooks/runner-dispatch-guard.sh:2716-2720`）

〔读码〕拒的条件：类型在 `GENERAL_PURPOSE_TYPES`（「GENERAL_PURPOSE_TYPES = {"general-purpose", "claude"}」，`.claude/hooks/runner-dispatch-guard.sh:925`）里、没有放行行（「valve_reason(prompt, "通用已判") is None」，`.claude/hooks/runner-dispatch-guard.sh:2716`）。不给 `subagent_type` 时守卫自己补成 general-purpose（`.claude/hooks/runner-dispatch-guard.sh:2685`），所以漏写类型也进这一拒。

### 情形 5.1　漏写 `subagent_type`，被默认补成 general-purpose
- 归类：**写法**。依据：2685 行的默认值；补上类型就过。放行行放过去的是一次「起步带全部规则的无定义 agent」。

### 情形 5.2　clerk 或 Explore 就能做的活，派了内置通用类型
- 归类：**写法**。依据：clerk 的工具是「tools: Read, Edit, Write, Bash, WebFetch, WebSearch」（`.claude/agents/clerk.md:4`），守卫自证里举的放行理由「要 WebSearch 查外部文档」clerk 本来就有；只读搜索派 Explore（拒绝说明 2720 行）。

### 情形 5.3　活需要 clerk 没有的工具或没有现成定义（MCP 工具、专门的 Glob / Grep 工具、需要带 CLAUDE.md 全量上下文）
- 〔会话记录〕没有实例：本机 2026-10-01 起的会话记录里，没有一次 Agent 调用的提示带「通用已判」行；记录里只有它的来历一行（`records/2026-09-28-agent体系用量时间与调度分析.md:100`，写的是「确要派写一行「通用已判：<理由>」」，是设计时的预留，不是用过的）。这个数只覆盖本机这一批记录。
- 归类：**该删放行行**（闸留着）。依据：这道闸护的是起步成本（拒绝说明「约 13 万 token」），闸本身有道理；放行行从没被用过，而真遇到需要时对的做法是**补一份定义**（`.claude/agents/<名>.md` 写清 `tools:` 与输入）——这是项目里每一种 agent 的来路，不是单次文字。
- 删了之后靠什么拦原来要拦的事：闸不变，仍拒 general-purpose 与 claude；拒绝说明的出路改成「clerk / Explore / 补一份定义」，去掉「确要派内置类型…单起一行」这半句。改动点：守卫 2716 行删 `valve_reason(prompt, "通用已判") is None` 这一条件、2720 行拒绝说明去半句、弄坏开关 `general-purpose-allowed` 保留（它测的是「闸整体放行」，不是放行行）、自证用例 3081–3082 行「共用约束:通用 agent 没写开工先读 / 写了开工先读」改成改用 clerk 之外的无定义类型来测第 ② 条；`.claude/agent-common.md:95` 里「提示里没有一行「通用已判：<理由>」」去掉；第 ② 条（`不读共用约束的类型（Explore、写了放行行的通用类型……`，守卫文件头）里「写了放行行的通用类型」同删。
- 什么会推翻：出现一个确实需要内置通用类型、而补定义又做不到的活（例如要靠内置类型独有、定义的 `tools:` 写不出来的能力）。一次都没见过；如果出现，应当改成**常设**：登记那个能力对应的类型名到一张表，不是放行行。

## 6. ⑥ 快照冲突已判（`snapshot_conflict_verdict`，`.claude/hooks/runner-dispatch-guard.sh:1462-1500`；`.claude/hooks/lib_snapshot_conflicts.py:46、162`）

〔读码〕守卫只对 kb-scribe 与 implementation-writer 判：「SNAPSHOT_GUARDED_AGENTS = {"kb-scribe", "implementation-writer"}」（`.claude/hooks/runner-dispatch-guard.sh:1394`）。「要改的文件」的认法是**出现**：「sources = [("派发提示", prompt)]」（`.claude/hooks/runner-dispatch-guard.sh:1482`）加上提示点名的规格文件全文，只要快照清单里列着的路径在里面出现一次，就是 hit（「for path in sorted(mentioned_repository_paths(text, listed, project_root)):」，`.claude/hooks/runner-dispatch-guard.sh:1491`）。放行：行首「快照冲突已判：<轮名> <理由>」，理由非空，按轮名放过那一轮（`.claude/hooks/lib_snapshot_conflicts.py:162` 起的 `ruled_snapshot_conflicts`，`ruled.add(match.group(1))` 在 172 行）。开着的轮：有 `<轮>-snapshot/`、有 `_<轮>-body.md`、没有 `<轮>-main-verification.md`、没被 `abandoned-rounds.tsv` 收掉（`.claude/hooks/lib_snapshot_conflicts.py:73`）。规矩原文：「腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。」（`.claude/rules/implementation-workflow.md:25`），其后有退路：「被别的会话定点替换过的把那几处替换反着做一遍」（`.claude/rules/implementation-workflow.md:27`）。
〔会话记录〕「快照冲突已判」是用得最多的一种：2679 次（含被别的闸拒了的尝试）：实现员 1501、写回员 298；另 880 次写在 investigator、三方腿、执行员、工具员、设计员、clerk 的提示里——这几种守卫从来不判（`git show 97f5904b4:.claude/hooks/runner-dispatch-guard.sh` 第 246 行起 `SNAPSHOT_GUARDED_AGENTS` 就是这两种），那 880 行是主 agent 顺手抄进每份提示的，白写的，不属于任何情形（写法习惯问题，去掉放行行后自然没有）。下面按理由归类，每一类都有〔会话记录〕里的原文。

### 情形 6.1　提示只是提到快照里的文件（只读、引用、「不碰」清单、背景报告里被引到）
- 理由原文：「checks-owed.md 只读」「列出的兄弟文件只是「不碰」清单」「.claude/agent-common.md 只是背景报告里提到、这一件只读不改」「那一轮快照里的中文 .claude/kb/checks-owed.md 只在调查员报告里被引到（C858），这次派发不改它」（2026-10-01 至 10-29）。
- 触发：见上，出现即 hit，「不碰」清单与背景引文同样出现。
- 归类：**闸判错**。依据：规矩管的是「改」，闸用「出现」代替；实现员那边守卫本来就有「要改什么」的结构化输入——「files = sorted(set(CRATES_PATH.findall(line.group(1))) - {"crates/mutations.tsv"})」（`.claude/hooks/runner-dispatch-guard.sh:1241`，来自 `CRATES_FILES_LINE`，守卫 973）。
- 改法（`snapshot_conflict_verdict`，守卫 1462 起）：把 `sources` 换成「要改的文件集」：implementation-writer 取 `CRATES_FILES_LINE` 那一行的 crates 路径（与 ⑤ 同一份解析，抽成共用函数）；kb-scribe 取点名规格文件里每条的 `file`（JSON）或「文件：」行（markdown，规格格式见 `scripts/kb/kb-spec-check.py` 文件头「规格的两种写法」），加上新增的一行「要写回的文件：<路径…>」（骨架 `scripts/agents/dispatch-prompt.py` 的 `EXTRA_LINES["kb-scribe"]` 打，写回员自己起草规格时靠它声明；定义 `.claude/agents/kb-scribe.md` 输入一节加一条，第 1 步“规格点名的文件”必须落在这一行里，否则停下报告）。`mentioned_repository_paths` 只对这个集合用。
- 兜底：写回员实际改了声明之外的快照文件，提交时门禁拿快照比判决之后又改过没有（`.claude/rules/implementation-workflow.md:25` 说的那两格），不会无声；这是**后验**，不是派发时拦。
- 什么会推翻：写回员的声明经常漏报（报告里改过的文件 ⊄ 声明）——那 6.1 的改法会把拦截从派发推迟到提交，漏的代价变大；要看已有写回员报告里「改过的文件」对声明的覆盖，这个数我没量，列在末尾命令里。

### 情形 6.2　实现员只交补丁、改动在草稿副本里，主工作区不动
- 理由原文（多条不同写法，2026-10-01 至 10-16）：「交补丁、只在草稿副本里改，主工作区不动，补丁等这一轮判决写完再打」「改动只在 m4-predev 工作树拷出的副本里出补丁，不碰主工作区与这一轮的开工快照」。
- 触发：实现员的提示里点了快照里的 crates 文件（它要改的、或要读的）。
- 归类：**闸判错**。依据：补丁模式下改动落到主工作区只有一步——`scripts/gate/apply-writer-patch.py`，它在这一步自己判快照：「副本上不查（副本不是那一轮取快照的工作区）。」（`scripts/gate/apply-writer-patch.py:46`，主工作区上则查，同文件头开工快照一段）；定义也写了补丁模式「在草稿目录的副本里改、交补丁目录…不碰主工作区；没写的在主工作区改。」（`.claude/agents/implementation-writer.md:15`）。派发时再查一遍，只是对同一件事判两次，而第一次看不出模式。
- 改法：骨架给实现员打一行「改动落点：交补丁 / 主工作区直改」（二选一，默认交补丁；`dispatch-prompt.py` 的 `EXTRA_LINES["implementation-writer"]`）；`snapshot_conflict_verdict` 对 implementation-writer：落点为交补丁时返回不判（交给 apply-writer-patch.py），主工作区直改时判 6.1 的声明文件集。
- 风险（给攻方腿）：写「交补丁」而实际改了主工作区。我核了写范围表：「implementation-writer	crates/**	implementation-writer.md「写范围」」（`.claude/hooks/agent-write-scope.tsv:8`，sha256 前 8 位 8417ae19）——主工作区的 `crates/` 不分模式一律放行，所以「交补丁」目前只是约定，没有任何闸强制。照这个改法落地，必须配一件事：落点登记进守卫已有的实现员登记（`implementation-writers.tsv`，守卫 `register_writer`），`.claude/hooks/write-guard.sh` 对登记为「交补丁」的实现员拒写主工作区的仓内路径（类比第九道认锁）。**没这一件，6.2 的改法会让说谎的「交补丁」绕过 ⑥ 与 apply-writer-patch.py 两道**；这一件我没读 write-guard.sh 的结构，没设计，归主 agent 判要不要做。做不了就退回：6.2 不改，只做 6.1。

### 情形 6.3　规矩本身允许的“定点替换 / 表尾追加”，写回员要改快照里的文件
- 理由原文（2026-10-01 一批，十余条）：「只做定点替换与整行插入，每处旧串、新串写进报告，主 agent 抄进调度记录供那一轮判决反做还原快照」「checks-owed.md 只用 insert-row.py 追加两行」。
- 归类：**例外**。依据：规矩第二段明文给了退路（`.claude/rules/implementation-workflow.md:27`），这一类是设计内的；而且守卫里已经有同类的硬编码先例——`crates/mutations.tsv` 只追加不算撞（守卫 1241）。
- 常设豁免表形态：`research/prompts/snapshot-edit-registry.tsv`（四列以上，制表符分隔）。一行 = 轮名、路径（必须在该轮快照清单里）、改法类（闭集：表尾追加 / 定点替换且旧新串进规格）、出处（`.claude/rules/implementation-workflow.md:27` 那一句，写成「文件:行〔原句〕」）、出处句的 sha256、登记者（会话号与日期）。谁登记：主 agent（腿不登记）；门禁核：新增一格进 doc-process-records——出处句在文件里逐字恰好命中且 sha256 一致、轮仍开着（判决一落笔即作废，行须同批删）、路径在该轮快照里。`snapshot_conflict_verdict` 对 kb-scribe：规格里该路径的每一条都是表尾追加（旧串是末行、新串是末行加新行）或带旧新串时，凭表里有这一行放行；否则仍拒。这样放行靠「一份登记过的、带出处哈希的、随轮结束失效的行」，不是提示里的自由文字。
- 这是**新增放行口**，会不会被「登一行、一直用」拿来当新的单次放宽，留给攻方腿的 A 格；我给的约束是行随判决作废，且同一路径同一轮至多一行。
- 什么会推翻：一次追加也能让腿引的行号漂移（插在表中间而不是表尾）——所以表只收「表尾追加」与「带旧新串的定点替换」，不收插入。

### 情形 6.4　那一轮其实已判完或已叫停，但闸还当它开着
- 理由原文：「那一轮已判随会话终止（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第 69 行）」（2026-10-01、10-02）；另有几条写「判决已写完（`research/prompts/test-script-align-code-r2-main-verification.md`）」。
- 归类：前一种**写法**（今天有 `research/prompts/abandoned-rounds.tsv` 登记，拒绝说明里写了怎么登记，`.claude/hooks/lib_snapshot_conflicts.py` 的 `refusal_text`；这是 2026-10-01 时登记表还没有的做法）；后一种我**核不了**：判决文件写在了闸不看的位置，还是 `has_verdict` 判法（`.claude/hooks/lib_snapshot_conflicts.py:73`，只看主工作区 `research/prompts/<轮>-main-verification.md` 是否存在）有别的缺口，会话记录里看不出，要对着当时的目录状态才能分。

## 7. 六种之外我顺带看到的同形放行口（不展开，留给攻方腿与主 agent）

- `SINGLEFS_EXPERIMENT_RERUN_REASON`：「强制重跑：设 SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>。」（`scripts/gate/admission.py:239`）——环境变量里写一句理由就越过「输入没变」，理由原样进产物头；与 ⑨ 的放行行同形。
- `apply-writer-patch.py --snapshot-ruled '快照冲突已判：<轮名> <理由>'`（`scripts/gate/apply-writer-patch.py:7`）——⑥ 放行行的命令行形态，与派发闸共用 `ruled_snapshot_conflicts`，改 6.x 时要一起改。
- 两处都是改 ⑨ / ⑥ 时必须同批处理的，否则去掉了派发提示里的放行行，却留下一个语义相同的命令行 / 环境变量口。

## 8. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1.1 ⑦ 窗口已提前解开、其后无同族派发 | 闸判错 | 改 `model_verdict` 的 `lifted` 读成功回合（A）；没有活动可借时加 `quota-probe` 类型探测（B）；换账号信号（C）弱 |
| 1.2 ⑦ 理由与限额无关 | 写法 | 闸按族判不按活判；出路是等 / 换族 / 探测 |
| 1.3 ⑦ 通知缺 `model sent to the API` 记成 opus | 闸判错（潜在，未观测到） | 271 条通知 0 条缺；空串时取 `launches` 里该 agent 的族，取不到不挡 |
| 2.1 ⑧ 点名的现成规格是红的 | 写法 | 改规格到绿再派；依据路径未提交走 `.claude/batch-scope` |
| 2.2 ⑧ 判决不是 `*-main-verification.md` 形状 | 闸判错 | 骨架打「判决：」行，`kb_scribe_spec_verdict` 加 `drafting_mode`，`recognized == 0` 只对非起草生效 |
| 2.3 ⑧ 判决形状对又点了存在的非规格文件 | 闸判错（预测，待探针） | 并进 2.2；p8b 探针不拒则推翻 |
| 2.4 ⑧ 纯译文 | 闸判错 / 定义缺口（一次观测） | kb-scribe 定义加「译文：」输入；或主 agent 判不归 kb-scribe，则改写法 |
| 第 4 问 `main-agent.md:100`、`dispatch-prompt.py:216` | 对骨架出来的派发，⑧ 恒不判 | 320/321 次带这一行；照 2.2 改骨架后删这一行与那一句 |
| 3.1 ⑨ 退 2：没登记实验键 | 写法 | 先接 `.claude/gate.d/stage-inputs.tsv`；设计员定义补一句教法 |
| 3.2 ⑨ 退 2：判法没跑起来 | 机制（修工具） | 放行行等于工具坏了也照派 |
| 3.3 ⑨ 退 3：前提没齐 | 写法 | 前提翻成「用户已定」并记出处再派 |
| 3.4 ⑨ 退 77：重跑原因在键外 | 三种：写法（重读不重跑）/ 闸判错（键漏输入，改 stage-inputs.tsv）/ 该删 ⑨ 的 77 支（同键有意再取，倾向，非定论） | 控制点挪到装置入口 admission；其环境变量口另判 |
| 4.1 ⑯ 旧会话持锁者已停，TaskStop 报找不到 | 闸判错 | `writer_lock_holder_state` 读完旧记录后并入当前会话的 `ended` |
| 4.2 ⑯ 派发过闸后被权限层拒，锁留下，换提示重派 | 闸判错 | `session_dispatch_events` 收被拒的 Agent 调用的提示 sha256，对上锁就不等 10 分钟 |
| 4.3 ⑯ 持锁者中途停了没交回 | 写法 | TaskStop 它；4.1 修好后跨会话也走得通 |
| 5.1 ⑫ 漏写类型 | 写法 | 默认值 general-purpose（2685 行） |
| 5.2 ⑫ clerk / Explore 能做 | 写法 | clerk 带 WebSearch |
| 5.3 ⑫ 需要 clerk 没有的能力 | 该删放行行（闸留） | 会话记录零实例；真有需要补一份定义 |
| 6.1 ⑥ 只是提到快照里的文件 | 闸判错 | 「出现」改成「声明的要改文件集」 |
| 6.2 ⑥ 实现员交补丁 | 闸判错（带必须配套的一件） | 落点登记 + write-guard 拒写，否则不要做 |
| 6.3 ⑥ 定点替换 / 表尾追加 | 例外 | `research/prompts/snapshot-edit-registry.tsv`，带出处哈希，随轮结束失效 |
| 6.4 ⑥ 轮已叫停 / 已判完 | 前者写法（登记 abandoned-rounds）；后者核不了 | 会话记录里看不出判决文件当时在哪 |

整体：六种放行行**都能去掉**；去掉的前提是 1.1(A,B)、2.2、4.1、4.2、6.1 的改判法先落地，否则 1.1 的第一次派发、2.2 的非 `-main-verification.md` 判决、4.1 的跨会话持锁者会失去出路（那几格目前只能靠放行行过）。我没有找到「去掉之后没有合规出路」的格，只要上面的前提都落了。

## 9. 要主 agent 跑的命令（腿一条都没交）

腿已经直接跑过的只读读取（不起项目代码）：对 `~/.claude/projects/-home-fy5090-code-singlefs/*.jsonl` 的 python 扫描（结果是上面的〔会话记录〕数）、`grep`、`sha256sum`、`git show`。这些数主 agent 没复跑，当线索。复跑与探针（轻型，不跑重型，不起 cargo，不碰 GPU）：

1. 复跑会话记录扫描（只读，本机）：
   `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/d165-valves-r1-sonnet/scan_valves_final.py`
   看第一行 Counter（预期 快照冲突已判 2679、规格检查已判 320、限额窗口已判 152、准入已判 5、写者锁已判 5）与各理由行；`通用已判` 预期不出现。
2. 写回员派发的放行行与判决形状分布（只读）：
   `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/d165-valves-r1-sonnet/scribe_count.py`
   预期 321 次、320 带放行行、141 点了 `-main-verification.md`。
3. ⑧ 的探针（情形 2.3 / 2.2 的预测；每条跑守卫自己的 `--check-prompt`，不碰锁、不碰会话记录）：
   `task-server/task submit -- bash .claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/.local/share/singlefs/work/d165-valves-r1-sonnet/probes/p8a-verdict-file-only.txt --type kb-scribe`
   `task-server/task submit -- bash .claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/.local/share/singlefs/work/d165-valves-r1-sonnet/probes/p8b-verdict-file-plus-scratch-md.txt --type kb-scribe`
   `task-server/task submit -- bash .claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/.local/share/singlefs/work/d165-valves-r1-sonnet/probes/p8c-verdict-other-shape.txt --type kb-scribe`
   在输出里找「书记员规格」一类。预测（跑前写下）：p8a 没有 ⑧ 的拒；p8b 与 p8c 有「认不出一份规格」。三份提示里其余的拒（批次、关的是哪几条等）与 ⑧ 无关，忽略。p8b 若没有 ⑧ 的拒，则 2.3 推翻；p8c 若没有，则 2.2 推翻。
4. 量 6.1 的兜底有多大漏报（未做）：写回员报告里「改过的文件」对派发提示声明的覆盖——这需要先有「要写回的文件：」一行，现在的历史提示没有，量不了，写在这里是为了说明 6.1 的“漏报率”此刻没有数。

## 10. 没做什么

- 没判别的腿的格（A 格：改法的漏洞、常设豁免表被拿来当新的单次放宽、去掉放行行后的死路、六种之外的放行口）；7 节只是顺带列出看到的两处同形口，没攻。
- 没替主 agent 采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑，上面所有〔会话记录〕的数、探针预测都是腿的，命令在第 9 节。
- 没读禁读清单（攻方腿的报告与模型）。没改任何钩子、脚本、定义；改法都只写了落点。
- 没写模型目录（这一题不建模型，所以没有 `rerun.sh`）。没量的东西：6.1 声明漏报率、1.1(C) 账号信号的稳定性、10-06 两条「今天没撞限额」的来历、10-29 两条 ⑨ 放行行里键到底漏没漏输入、6.4 判决文件当时的位置。

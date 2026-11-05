# d165-valves-r1 攻方腿（Opus）报告

轮名：d165-valves-r1；格：A；日期：2026-11-01。背景材料：`research/prompts/_d165-valves-r1-body.md`。
被判文件此刻的 sha256 与开工快照 `research/prompts/d165-valves-r1-snapshot/files-sha256.txt` 逐份相同（量的：本腿 `sha256sum` 七份，见「复核数据」一节）。
引文行号都是主工作区路径此刻的行号；每份文件的 sha256 前 8 位：
`.claude/hooks/runner-dispatch-guard.sh` f11bbf5d，`.claude/hooks/lib_snapshot_conflicts.py` d2e70e42，`.claude/hooks/lib_writer_lock.py` 5bf2ef90，
`scripts/agents/dispatch-prompt.py` dfc51ee5，`scripts/gate/apply-writer-patch.py` 057be868，`.claude/main-agent.md` b691d3df，`.claude/agent-common.md` 05a0501d。
本腿没写模型；只写了探针提示与 grep 的原样输出，放在 `research/prompts/d165-valves-r1-opus-model/`（文件与 sha256 见末尾「探针与输出文件」一节）。

## 各格判定一览

攻方腿只判 A 格。下表每行一处打中或没打中；「依据」是推的（读源码）还是量的（本腿跑出的只读命令输出）分开标；探针输出要主 agent 跑（「要主 agent 跑的命令」一节），跑出之前全部按「推的」读。

| 号 | 攻击面 | 闸 | 打中什么 | 判定 | 依据 |
|---|---|---|---|---|---|
| A1 | 一 | ⑦ | 去掉放行行而不改判法：「同一族又派出去、没再撞」那条解开判据在重置时刻之前永远到不了（每次同族派发都被 ⑦ 自己拒），opus 定义的类型又不许换族，卡到重置时刻 | 打中（卡死） | 推的，源码 |
| A2 | 一 | ⑦ | 改判法「主会话自己同族在答就算解开」只解得开主 agent 那一族，sonnet 腿（three-way-forward 等）照样卡死 | 打中（改法够不着一半） | 推的 |
| A3 | 一 | ⑥ | ⑥ 按「提示里出现」判，不按「要改」判：实现员必写的「不碰」一行点名快照里的文件就被拒；照快照纪律写「不碰 X」反而触发 | 打中（误拒、逼人改措辞绕） | 推的；探针 P1/P1b、P2/P2b 待主 agent 跑 |
| A4 | 一 | ⑥ | 同上，只读引用（「照 `.claude/agent-common.md` 某节，只读」）在本轮开着时被拒 | 打中（误拒） | 推的；探针 P3 待跑 |
| A5 | 一 | ⑥ | apply-writer-patch 只追加变异行也拒（自证里写死），而派发闸 ⑤ 明说 mutations.tsv 只追加不算撞；开着的 m4-gpugc-code-r2 快照列着 crates/mutations.tsv | 打中（每个带变异行的实现员补丁都要放行参数） | 推的；快照清单量的 |
| A6 | 一 | ⑥ | 改判法「只判要改的文件一行」：写回员自己起草规格时派发那一刻没有目标清单，判别子观测不到 ⇒ 漏放 | 打中（漏放；今天同样漏） | 推的 |
| A7 | 一 | ⑧ | 去掉放行行：续做的写回员（「承接：」点名工作根下上一份报告）被当成候选规格、认不出 ⇒ 拒，带判决也拒 | 打中（卡死） | 推的；探针 P4/P4b 待跑 |
| A8 | 一 | ⑧ | 去掉放行行：判决不是三方判决文件（用户定案记在 records/）⇒ 整份提示当规格判、认不出 ⇒ 拒 | 打中（卡死） | 推的；探针 P5 待跑 |
| A9 | 一 | ⑧ | 改判法「认不出规格而有判决就放」：一份写坏了、kb-spec-check 认不出的规格与判决同写 ⇒ 漏放 | 打中（漏放） | 推的 |
| A10 | 一 | ⑨ | 去掉放行行：重跑的理由在键外（kb 决策改一句照判 77，admission 自证 T19；新臂要设计员这一次才写）⇒ 设计员派不出去 | 打中（卡死） | 推的 |
| A11 | 一 | ⑨ | 改判法「把重跑登记 / 问题单并进键」：派发那一刻重跑登记还不在，键照旧 ⇒ 仍 77；按问题单哈希判则改一个错字就放 | 打中（两头都中） | 推的 |
| A12 | 一 | ⑯ | 去掉放行行：不卡死（同会话 TaskStop 得「No task found」即算结束；别的会话的锁 3 小时内放；或换目录） | 没打中卡死 | 推的 |
| A13 | 一 | ⑯ | 改判法「缩短在跑阈值」：活着而一步很久没写会话记录的写者被判失效、被接手 ⇒ 漏放两位写者 | 打中（漏放，阈值值没量） | 推的 |
| A14 | 一 | ⑫ | 去掉放行行：不卡死（clerk 已带 WebFetch、WebSearch；自证样本里的理由已经不成立）；名单外的内置类型今天照样不经 ⑫ | 没打中卡死；附带一处名单窄 | 推的 |
| B1 | 二 | 全部 | 被引来当范本的常设豁免表（crates-review-exempt、agent-def-review-exempt）表头自己写「逐次豁免」「这次不走三方」，判法只比路径与哈希、理由不核 | 打中（范本本身是一次豁免） | 推的，源码 |
| B2 | 二 | ⑦ ⑧ ⑨ ⑯ | 按事件取键（失败通知、规格哈希、E号加 r号、锁的 nonce）的豁免行用一次就作废；按类取键（族、类型、实验号、目录）罩住后来的真情形 | 打中（两种键都中） | 推的 |
| B3 | 二 | ⑥ | 按（轮、路径）取键的行罩住同一轮里后来的整行替换、删行 | 打中（键太宽） | 推的 |
| C1 | 三 | kb-spec-check | `--allow-uncommitted <路径>`：这一次放行、不要理由、认目录前缀、绿行照说「依据没有别的会话还没提交的改动」 | 打中（同形，比 6 种还松） | 推的，源码 |
| C2 | 三 | admission / replay | `SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>`：每次设、只越过「输入没变」、理由写进产物头 | 打中（同形，带痕迹） | 推的 |
| C3 | 三 | stale-candidates | `--allow-dirty-worktree <理由>`：每次给、理由至少 8 个字、写进输出头 | 打中（同形，带痕迹） | 推的 |
| C4 | 三 | 派发闸 ⑪、交回闸 ⑦ | 「只修锚点」「只补落产物」：一行声明就免两道判；⑪ 还是整份提示子串，句中「不是只修锚点」也算 | 打中（同形，⑪ 比交回闸松） | 推的 |
| C5 | 三 | apply-writer-patch | `--snapshot-ruled`：已在背景材料 6 种里（与 ⑥ 共用判法），不另计 | 已知 | — |
| C6 | 三 | 各处 | `--force`、`--skip-stage-two`、`--skip-apply`、按键复用的 `--skip`、`*_BREAK` 与 `WRITE_GUARD_DISABLE_*` 弄坏开关、两个只给自证的环境变量 | 不是单次放宽 | 推的 |

## 攻击面一：改判法之后的误拒、漏放与卡死

### ⑦ 限额窗口（A1、A2）

判法（`.claude/hooks/runner-dispatch-guard.sh`，f11bbf5d）：

- `.claude/hooks/runner-dispatch-guard.sh:1273`：「lifted = any(model_family(model) == window_family and launched_at > notified_at and agent_id not in limit_failed」
- `.claude/hooks/runner-dispatch-guard.sh:1276`：「and valve_reason(prompt, "限额窗口已判") is None):」
- `.claude/hooks/runner-dispatch-guard.sh:972`：「LIMIT_WINDOW_LOOKBACK_SECONDS = 8 * 86400」

**A1 去掉放行行、不改判法 ⇒ 卡死。** 历史：t0 一条 opus 腿失败，通知写「resets <t1>」；t0 之后用户换号或额度提前恢复；t0 < 现在 < t1。
那一步派发：主 agent 派 `three-way-attack`（不写放行行）。判法：1273 行要一次「通知之后同一族派出去」的回执才算解开，而那次派发本身要先过 1276 行；放行行去掉之后 1276 行的条件只剩窗口没过 ⇒ 拒，回执不会出现 ⇒ 1273 行在 t1 之前永远为假。⑦ 对每种类型都判（`decide` 里调 `model_verdict` 不分类型），所以没有别的同族派发能先把窗口解开。
出路只剩换族与等：换族撞共用约束 `.claude/agent-common.md:13`：「**严格遵守文档中model和effort的设定**。」与 `.claude/agents/three-way-attack.md:5`：「model: opus」（c08d600d）——opus 定义的 investigator、experiment-designer、implementation-writer 同此。等的上限是失败通知写的重置时刻，回看窗口 8 天。
四句：分辨臂——分得开「保留放行行」与「去掉放行行」两臂（只有后者卡）；系统当时看不看得到——看不到，额度提前恢复只在 API 那一侧，主会话记录里没有任何字；满足的是攻击面一「去掉放行行之后哪种正常派发会卡死」那一分句；改法——「只改判法、不留放行行」在这几格上照样中，除非判法拿到一个新的可观测量（见 A2）。

**A2 改判法「主会话自己同一族在答就算解开」⇒ 只够得着一半。** 这是我能想到的、判法里唯一读得到的解开信号（主会话记录里通知之后的 assistant 回复带 model 字段，推的，没在本会话记录上核）。
那一步派发：主 agent（opus）派 `three-way-forward`（`.claude/agents/three-way-forward.md:5`：「model: sonnet」，1492d61b），sonnet 那一族在窗口里。主会话的回复全是 opus，证不出 sonnet 解开 ⇒ 照拒 ⇒ 与 A1 同样卡到重置。
另一半也只是推的：主 agent 的额度与子 agent 是不是同一个池，仓里没有量过；若不是同一池，opus 那一半会误放（派出去再撞一次限额，代价是一次失败派发与一个新窗口，不是数据损失）。
四句：分辨臂——分得开「A2 改法」与「另加放行」两臂；系统看得到——主会话那一族看得到，别的族看不到；满足的分句同 A1；改法——A2 对 sonnet 窗口那几格不起作用，要写明。

**常设豁免的形态**见攻击面二 B2：按失败通知取键的行只用得上一次（解开之后 1273 行自己接管），按族取键的行罩住 8 天里后来的真限额。

### ⑥ 快照冲突（A3、A4、A5、A6）

判法：

- `.claude/hooks/runner-dispatch-guard.sh:1482`：「sources = [("派发提示", prompt)]」——整份提示都是来源，不剔「不碰」一行、否定句、引号与围栏块。
- `.claude/hooks/runner-dispatch-guard.sh:1491`：「for path in sorted(mentioned_repository_paths(text, listed, project_root)):」——判的是「出现」。
- （d2e70e42）`.claude/hooks/lib_snapshot_conflicts.py:183`：「if hits and round_name not in ruled and "*" not in ruled:」
- 开着的轮（量的，本腿 `ls research/prompts/*-snapshot` 加正文与判决在不在）：d165-valves-r1、m4-gpugc-code-r2。
  （506e777c）`research/prompts/m4-gpugc-code-r2-snapshot/kb-sha256.txt:4`：「.claude/kb/milestone/04-fourth-txn.md」；
  （88cda8e4）`research/prompts/m4-gpugc-code-r2-snapshot/crates-sha256.txt:10`：「crates/mutations.tsv」；
  `research/prompts/d165-valves-r1-snapshot/files-sha256.txt:7`：「.claude/agent-common.md」。

**A3「不碰」一行触发 ⑥。** 真提示：（537865dd）`.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a28fe10b58d88beec.txt:23`：「- `.claude/kb/milestone/04-fourth-txn.md`：里程碑四的会话在改，主 agent 自己写。」——派 kb-scribe 时在「不碰的」一节点名别的会话在改的文件。今天 m4-gpugc-code-r2 开着，这一行就撞 ⑥。
那一步派发：探针 `research/prompts/d165-valves-r1-opus-model/probe-p1-kb-scribe-untouched-line.txt`（只留这一行「不碰的」，别的照骨架），对照 `probe-p1b-…`（删掉那一节）。推的结局：P1 退 2、⑥ 列 `.claude/kb/milestone/04-fourth-txn.md（出现在派发提示）`；P1b 退 0。
实现员更重：⑱ 要实现员必写一行行首标签带「不碰」的（`.claude/hooks/runner-dispatch-guard.sh:2272` 起 `untouched_files_verdict`），派 `implementation-writer` 时照快照纪律写「不碰：crates/mutations.tsv（变异行写进草稿目录的 mutations-append.tsv）」（探针 P2）⇒ ⑥ 拒；写「不碰：无」（P2b）放行。放行行去掉之后合规写法只剩把「不碰」改成不带路径的话、或写「无」——就是改措辞躲匹配，丢的是给子 agent 的约束。
四句：分辨臂——分得开「按出现判」与「按要改判」两种判法；系统看得到——看得到（「不碰」标签与否定词都是字面），⑤ 的判法已经在认标签行；满足的是「哪种情形会被误拒（逼人去找新的绕法）」那一分句；改法——「保留放行行」与「改成常设豁免」在这格上都还要每次写一行，只有改判法（剔「不碰」行与否定句）修得到。

**A4 只读引用触发 ⑥。** 探针 P3：kb-scribe 提示写「时间写法照 `.claude/agent-common.md`「规则怎么读」一节（只读，不改）」；d165-valves-r1 开着 ⇒ 推的退 2。四句同 A3，改法要再剔「只读」类说明，而「只读」与「要改」在自然语言里的分界钩子认不全（判别子只部分看得到）。

**A5 只追加变异行的补丁照拒。** （057be868）`scripts/gate/apply-writer-patch.py:43`：「三份变异表文件有一份写了行就另算 crates/mutations.tsv——落在还没判完的三方轮的开工快照里就拒」；自证写死：`scripts/gate/apply-writer-patch.py:685`：「snapshot：改 lib.rs 的补丁与只追加变异行的补丁都拒」。
对照派发闸 ⑤ 的口径：`.claude/hooks/runner-dispatch-guard.sh:1241`：「files = sorted(set(CRATES_PATH.findall(line.group(1))) - {"crates/mutations.tsv"})」（只追加、不算撞）。两处对同一个文件口径相反。
那一步：m4-gpugc-code-r2 开着期间，任何实现员补丁目录里 `mutations-append.tsv` 有一行，`apply-writer-patch.py <补丁目录>` 就退 2；去掉 `--snapshot-ruled` 之后只能等那一轮判决（代码轮可到三轮）。改判法「只有 mutations-append.tsv 有行、另两份为空 ⇒ 不算撞」在打补丁那一刻看得到（三份文件都在补丁目录里）；在派发那一刻看不到（子 agent 还没写）——所以 ⑥ 在派发一侧对 mutations.tsv 只能不判。
残留风险（推的）：追加改了文件的 sha256；`scripts/agents/snapshot-tree.py` 倒推开工树时若这一轮里提交过，mutations.tsv 倒推不出、腿引它的行落到人工核。

**A6 改判法「只判要改的文件那一行与规格目标」⇒ 写回员自起草时漏放。** 写回员按 `.claude/main-agent.md:100` 那种派法只拿判决与条目、自己起草规格，派发那一刻提示里没有目标文件清单 ⇒ 新判法一处也看不到 ⇒ 放行 ⇒ 写回员照判决改了快照里列着的 kb 文件。今天按「出现」判也同样漏（提示里没写那个路径就不拦），所以这不是放行行造成的，但改法若声称「改判法后不用放行行」，这一格它没修。
够得着的落点是写的那一刻（写闸看 Edit 目标、Bash 检出钩子看 replace-once / replace-batch 的目标），那里看得到真目标；那里同样会撞 A5 的「只追加一行」情形，要连同按改动种类的豁免一起设计。被攻过零轮，只是我推的落点。

### ⑧ 书记员规格（A7、A8、A9）

判法：

- `.claude/hooks/runner-dispatch-guard.sh:1291`：「if valve_reason(prompt, "规格检查已判") is not None or break_switch_is("spec-check-skipped"):」
- `.claude/hooks/runner-dispatch-guard.sh:1297`：「if os.path.isfile(path) and path.endswith((".json", ".md", ".txt")):」——提示里点名的、工作根下存在的 .md 都算候选规格。
- `.claude/hooks/runner-dispatch-guard.sh:1300`：「if not candidates and VERDICT_MENTION.search(prompt) and not break_switch_is("verdict-not-spec"):」——只在一份候选都没有时，三方判决文件才让它放。
- `.claude/hooks/runner-dispatch-guard.sh:1325`：「if recognized == 0:」
- `.claude/hooks/runner-dispatch-guard.sh:928`：「VERDICT_MENTION = re.compile(」（只认 `research/prompts/<名>-main-verification.md`）。
- 骨架与主 agent 定义：（dfc51ee5）`scripts/agents/dispatch-prompt.py:216`：「"kb-scribe": ["规格检查已判：写回员自己起草并跑 kb-spec-check.py"],」；（b691d3df）`.claude/main-agent.md:100`：「派发提示写一行…规格检查已判：写回员自己起草并跑规格检查…过派发闸 ⑧」。
  真提示照写：`.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a28fe10b58d88beec.txt:9`：「规格检查已判：写回员自己起草并跑 kb-spec-check.py」——那份提示同时点名了三份 main-verification，没有工作根下的现存文件，按 1300 行本来就放，放行行是多余的（推的）。

**A7 续做的写回员卡死。** 历史：写回员甲停在一半，报告落在工作根；主 agent 照共用约束新派一个，提示写「承接：<甲的报告路径>」，判决照旧点名 main-verification。
探针 P4（`probe-p4-kb-scribe-continuation.txt`，承接的是草稿目录里一份散文 `/home/fy5090/.local/share/singlefs/work/d165-valves-r1-opus/probe-prior-result.md`），对照 P4b（删掉承接行）。推的：P4 里那份 .md 进候选（1297 行）⇒ 候选非空、1300 行不走 ⇒ kb-spec-check 对它退 2（认不出规格）⇒ 1325 行拒；P4b 无候选、有判决 ⇒ 1300 行放。
放行行去掉之后，合规出路只剩不写承接行（违背「新派同类 agent 带承接」的做法）或把上一份报告挪出工作根。
四句：分辨臂——分得开「有放行行」「无放行行」；系统看得到——看得到（「承接：」是标签行），判法没用它；分句是「去掉放行行之后哪种正常派发会卡死」；改法——「只认规格行（标签「规格：」）上的文件当候选」修得到这一格。

**A8 判决不在三方判决文件里 ⇒ 卡死。** kb-scribe 定义的输入是「判决|规格」，判决可以是用户定案（`records/` 里的原话），不一定是 main-verification。探针 P5：「判决：用户定案，原话在 records/2026-10-29-subagent拆分提案.md 第一百六十五节；写回员自己起草规格…」。推的：无候选、无 VERDICT_MENTION ⇒ 整份提示写进临时文件当规格交 kb-spec-check ⇒ 退 2 ⇒ 1325 行拒。
改法「VERDICT_MENTION 放宽成认 `判决：` 标签」修得到；同时引出 A9。

**A9 改判法「认不出规格而有判决就放」⇒ 漏放。** 那一步：主 agent 手写一份规格放在草稿目录，写法错了（比如「旧串：」后面没写围栏），kb-spec-check 对它退 2；提示里同时写了判决行。今天：候选非空 ⇒ recognized 0 ⇒ 拒（这正是 ⑧ 文件头说的「一份认得出的规格都没有也拒」）。改后：放 ⇒ 写回员拿着坏规格开工。
四句：分辨臂——分得开 A8 的两种改法（「判决就放」漏放，「只认规格行」不漏）；系统看得到——「这份 .md 是写坏的规格还是别的报告」在内容上看不全，靠标签行看得到；分句是「原来该拦的会漏放」；改法——「只认规格行」不中这一格，「判决就放」中。

另一种同在 ⑧ 下的误拒（推的，没造探针）：主 agent 手写的规格「依据」点名一份已跟踪、工作区有改动、又没登记进 `.claude/batch-scope` 的文件（例如正在写的新账本），kb-spec-check ⑥ 判红 ⇒ ⑧ 拒；kb-spec-check 自己的出路 `--allow-uncommitted` 派发闸不传（见 C1）。

### ⑨ 设计员重跑准入（A10、A11）

判法：`.claude/hooks/runner-dispatch-guard.sh:1347`：「if match is None or break_switch_is("admission-ignored") or valve_reason(prompt, "准入已判") is not None:」；`.claude/hooks/runner-dispatch-guard.sh:1358`：「if result.returncode == 77:」。
键的边界（`scripts/gate/admission.py`，df98f7d0）：`scripts/gate/admission.py:245`：「实验准入：T19（kb 决策文档改一句）⇒ 实验键不变，照判没变（退 77）：退 0」。

**A10 去掉放行行 ⇒ 键外的重跑理由派不出设计员。** 历史：一条决策分项改了一句（问题变了），主 agent 照 `.claude/main-agent.md`「要建计数实验，或重跑已有实验」那一行写问题单、派 `experiment-designer` 写 `research/prompts/e<号>-r<n>-prereg.md`。
那一步：⑨ 起 admission.py，kb 决策改动不进键（T19）、新臂与新判据要这位设计员写完才有 ⇒ 键与上次产物头相同 ⇒ 退 77 ⇒ 拒。放行行去掉之后没有合规出路：问题单不进键，登记文件这一刻还不在。
四句：分辨臂——分得开有无放行行；系统看得到——看不到（「问题变了要重答」不在键里，是判别子观测不到那一形）；分句是「卡死、没有合规出路」；改法——「改成常设豁免」见 B2（按 E号加 r号取键用一次就废，按 E号取键罩住以后每次重跑）；「改判法」见 A11。

**A11 改判法两种都中。** 甲：把重跑登记并进键——派发那一刻登记还不在，键照旧 ⇒ 仍 77（A10 原样）。乙：认提示里点名的问题单 / 岔路单，按它的内容哈希没出现在上次产物头里就放——产物头今天没有这一栏（推的，没在 admission.py 里找到这种字段），要新加；加了之后问题单改一个错字哈希就变 ⇒ 同一份输入照样重跑（漏放），正是 ⑨ 要拦的。
⑨ 的判别子（「这次重跑会不会得出不同的东西」）在派发那一刻只有主 agent 与用户判得了；钩子能做的只有键这一半。若判法改不出、豁免又只有一次性或太宽两种（B2），剩下的选项是这道闸在派发一侧删掉、只留执行时 admission.py 那一道（那一道自己也有单次放行口，见 C2）。

### ⑯ 草稿目录写者锁（A12、A13）

判法：`.claude/hooks/runner-dispatch-guard.sh:2058`：「valve = valve_reason(prompt, "写者锁已判") is not None and not break_switch_is("writer-lock-valve-ignored")」；`.claude/hooks/runner-dispatch-guard.sh:2077`：「if alive and not valve:」；在跑判法 `.claude/hooks/runner-dispatch-guard.sh:1151`：「return time.time() - last_seen <= RUNNING_RECENT_SECONDS」，`.claude/hooks/runner-dispatch-guard.sh:970`：「RUNNING_RECENT_SECONDS = 3 * 3600」。

**A12 去掉放行行不卡死（没打中）。** 放行行说明里写的情形是「客户端重启后留下的旧进程」。同会话：主 agent 对那个号 TaskStop，回「No task found …」而在跑名单里没有它，`.claude/hooks/runner-dispatch-guard.sh:1113`：「if absent and absent.group(1) not in absent.group(2) and not break_switch_is("absent-task-counted"):」把它记成结束 ⇒ 锁自动接手。别的会话的锁：TaskStop 够不着，按那位子 agent 自己会话记录的最后改动 3 小时后放（`.claude/hooks/runner-dispatch-guard.sh:2034`：「return now - max(dispatched, claim.get("claimed_at", 0) if claim else 0) <= RUNNING_RECENT_SECONDS, holder」是认领过而回执读不到的那一支）；不想等就换草稿目录（拒绝说明自己给的出路）。代价是等至多 3 小时或换目录、把交接的副本拷过去；不是卡死。
试过的形状：同会话重启、别的会话关掉、认领文件坏了、锁里登记的是同一份提示的重试（那一支已直接接手）。

**A13 改判法「缩短 3 小时」⇒ 漏放。** 判别子是「那位子 agent 还活着吗」，钩子能看到的只有它会话记录的改动时刻。活着的子 agent 有长时间不写记录的一步（前台命令上限 240000 毫秒，加上长思考，推的；没量过单步最长间隔），阈值低于那个间隔就把活写者判成失效、新锁接手，旧写者之后的 Write / Edit 被写闸第九道拒、Bash 里的写看不见——正是锁要防的两位写者交替写。阈值该取多少要先量各类型子 agent 两次写记录之间的最长间隔，这一格我没量。

### ⑫ 内置通用类型（A14）

判法：`.claude/hooks/runner-dispatch-guard.sh:2716`：「if (subagent_type in GENERAL_PURPOSE_TYPES and valve_reason(prompt, "通用已判") is None」；`.claude/hooks/runner-dispatch-guard.sh:925`：「GENERAL_PURPOSE_TYPES = {"general-purpose", "claude"}」。

**A14 去掉放行行不卡死（没打中）。** 放行行要写「为什么 clerk 做不了」；（66d0b01b）`.claude/agents/clerk.md:4`：「tools: Read, Edit, Write, Bash, WebFetch, WebSearch」。自证样本的理由 `.claude/hooks/runner-dispatch-guard.sh:3082` 里写的「通用已判：要 WebSearch 查外部文档」（同一串在 3081、3082 两行）今天已经不是 clerk 做不了的事。剩下内置类型独有的只有会话里挂的别的工具（MCP 这类，推的）；那种活主 agent 自己做（共用约束「简单任务不派发」）。
附带（不是放行行，推的）：⑫ 只认名单里两个名字；名单外、又没有定义文件的类型不经 ⑫，只受 ② 要「开工先读」一行。若改法写成「只许项目定义的类型与 Explore」，这一处一起收严。

## 攻击面二：常设豁免表会不会变回单次放宽（B1、B2、B3）

**B1 背景材料拿来当范本的「登记一次、带出处与内容哈希、门禁核」，仓里现成的两张表本身就是逐次豁免。**

- （5a8e0e27）`.claude/agent-def-review-exempt:1`：「用户逐次豁免三方论证的定义改动」
- （8dcb99ef）`.claude/crates-review-exempt:2`：「只有用户明说…这次不走三方…时才加一行」
- 判法只比路径与哈希：（1260d7f3）`.claude/gate.d/doc-process-records.sh:1050`：「'$1 !~ /^#/ && NF >= 3 && $1 == path && $2 == hash { print $3; exit }'」——第三段理由原样打出、不核；谁登记的、用户有没有说过，门禁看不到。

照这个范本把放行行改成豁免行，得到的是：主 agent 自己往表里加一行（理由随便写，门禁只看哈希），用这一次，行留着不删。与今天的放行行相比只多了「写进一个文件」这一步，判法上没有多核一样东西。
对照：叫停轮登记表（`research/prompts/abandoned-rounds.tsv`）的出处是机器核的（records/ 里逐字找原话、含轮名、有「」原话），那种才算「带出处」。豁免表若要不变回单次放宽，出处判法至少要到那一档；否则「登记一行」与「写一行放行行」等价。
另一个缝（推的）：按哈希认的行没有过期判法（上面那一行 awk 只找命中），文件改回到登记过的那一版内容，豁免自己复活。

**B2 ⑦ ⑧ ⑨ ⑯ 被拒的情形是一次性事件：按事件取键用一次就废，按类取键太宽。**

| 闸 | 按事件取键（一行写什么） | 用几次 | 按类取键（一行写什么） | 多罩住的情形 |
|---|---|---|---|---|
| ⑦ | `<失败的 agent 号>\t<resets 原句>\t已解开\t<出处>` | 1 次：解开后 1273 行自己接管，之后这一行不再起作用 | `opus\t已解开\t<出处>` | 8 天回看里后来又一次真撞限额（新通知、新窗口）也被放过，每次派发都再撞一次 |
| ⑧ | `<规格文件的 sha256>\t<理由>` | 1 次：一份规格只用一次 | `kb-scribe\t写回员自己起草` | 恒不判写回员——就是今天骨架默认行的效果（`scripts/agents/dispatch-prompt.py:216`） |
| ⑨ | `E142\tr19\t<哪一样变了>` | 1 次：下一次重跑是 r20 | `E142\t<理由>` | E142 以后每一次重跑都不再判「输入没变」 |
| ⑨ | `E142\t<实验键 sha256>\t<理由>` | 不止 1 次 | — | 同一把键第二次、第三次重跑照放：恰好是 ⑨ 要拦的「输入没变还重跑」，除非用完就删（那就是单次） |
| ⑯ | `<锁的 nonce>\t<理由>` | 1 次：接手就换 nonce | `<草稿目录>\t<理由>` | 那个目录的锁永久失效，两位写者同时写 |

结论（推的）：这四道闸若改判法改不出、又不删，常设豁免只剩两种形态，各自打中用户口径里的一半：事件键＝一次豁免，类键＝把闸对那一类关掉。

**B3 ⑥ 按（轮、路径）取键太宽。** 一行写 `m4-gpugc-code-r2\tcrates/mutations.tsv\t只追加变异行`，这一轮开着期间后来的每一次派发与打补丁都过；之后某个补丁的 `mutations-replacements.tsv` 整行换掉腿引过的那一行、或 `mutations-delete.txt` 删行让后面的行号挪位，同样过。键里要带改动种类（追加 / 替换 / 删），而种类只在打补丁那一刻看得到（A5），派发那一刻看不到。

## 攻击面三：6 种之外同形的单次放行口

范围：`git ls-files -co --exclude-standard`（已跟踪加未跟踪、未忽略），去掉 `research/` 与 `records/`（当时的记录，不是在用的判法）；三条 grep 的命令与输出原样如下，也在模型目录同名文件里。

### 三条 grep

`research/prompts/d165-valves-r1-opus-model/grep-valve-labels.txt`（「…已判：」标签行，先 `grep -c` 再分名数）：

```
$ git ls-files -co --exclude-standard -- . ":!research" ":!records" | tr "\n" "\0" | xargs -0 grep -c -E "[^「」[:space:]，。；：:]{1,12}已判[*]*[：:]" | grep -v ":0$"
.claude/agent-common.md:1
.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a28fe10b58d88beec.txt:2
.claude/hooks/lib_snapshot_conflicts.py:4
.claude/hooks/lib_writer_lock.py:1
.claude/hooks/runner-dispatch-guard.sh:36
.claude/hooks/write-guard.sh:2
.claude/main-agent.md:1
scripts/agents/dispatch-prompt.py:2
scripts/gate/apply-writer-patch.py:8

$ git ls-files -co --exclude-standard -- . ":!research" ":!records" | tr "\n" "\0" | xargs -0 grep -ohE "[^「」[:space:]，。；：:`（(]{1,12}已判[*]*[：:]" | sort | uniq -c | sort -nr
      9 快照冲突已判：
      7 "快照冲突已判：
      6 准入已判：
      6 写者锁已判：
      5 规格检查已判：
      4 '快照冲突已判：
      4 \n准入已判：
      3 通用已判：
      3 "写者锁已判：
      2 限额窗口已判：
      2 "通用已判：
      2 \n规格检查已判：
      1 ["规格检查已判：
      1 "线程上限已判：
      1 "第四件\n写者锁已判：
      1 "多机已判：
      1 n只补落产物\n限额窗口已判：
      1 50G\n多机已判：
      1 16\n线程上限已判：
```

读法：`research/`、`records/` 之外只有 6 种放行行，外加两种已撤的（「线程上限已判」「多机已判」只在派发闸自证的撤掉行用例里）；`.claude/agent-common.md` 与 `.claude/main-agent.md` 各 1 处是教写的句子。没有第七种「…已判」。

`research/prompts/d165-valves-r1-opus-model/grep-override-flags.txt`（命令行参数）：

```
$ git ls-files -co --exclude-standard -- . ":!research" ":!records" | tr "\n" "\0" | xargs -0 grep -c -E -e "--[a-z0-9-]*(ruled|force|allow|skip|override|bypass|accept|exempt|unsafe|no-check|no-verify)[a-z0-9-]*" | grep -v ":0$" | wc -l
169

$ git ls-files -co --exclude-standard -- . ":!research" ":!records" | tr "\n" "\0" | xargs -0 grep -ohE -e "--[a-z0-9-]*(ruled|force|allow|skip|override|bypass|accept|exempt|unsafe|no-check|no-verify)[a-z0-9-]*" | sort | uniq -c | sort -nr
    346 --force
    245 --skip
     91 --skip-stage-two
     75 --skip-apply
     24 --skip-finished
     22 --allow-uncommitted
     20 --no-verify
     19 --skip-gpu
     16 --allow-empty
     16 --allow-dirty-worktree
     12 --snapshot-ruled
     11 --skip-light
     10 --skip-open-check
      9 --accept-long
      5 --skip-torn-reading
      5 --force-gate
      2 --forced
      1 --skip-heavy
      1 --force-it
```

`research/prompts/d165-valves-r1-opus-model/grep-override-variables.txt`（环境变量名）：

```
$ git ls-files -co --exclude-standard -- . ":!research" ":!records" | tr "\n" "\0" | xargs -0 grep -ohE '\b[A-Z][A-Z0-9_]*_(REASON|FORCE|FORCED|ALLOW|ALLOWED|SKIP|OVERRIDE|BYPASS|ACCEPT|EXEMPT|RULED|DISABLE)\b|\b[A-Z][A-Z0-9_]*_(FORCE|ALLOW|SKIP|OVERRIDE|BYPASS|EXEMPT|DISABLE)_[A-Z0-9_]+\b' | sort -u | wc -l
65

$ git ls-files -co --exclude-standard -- . ":!research" ":!records" | tr "\n" "\0" | xargs -0 grep -ohE '\b[A-Z][A-Z0-9_]*_(REASON|FORCE|FORCED|ALLOW|ALLOWED|SKIP|OVERRIDE|BYPASS|ACCEPT|EXEMPT|RULED|DISABLE)\b|\b[A-Z][A-Z0-9_]*_(FORCE|ALLOW|SKIP|OVERRIDE|BYPASS|EXEMPT|DISABLE)_[A-Z0-9_]+\b' | sort | uniq -c | sort -nr
     14 LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW
     14 GPU_QUIESCE_CHECK_EXEMPT_PIDS
     13 C510_REASON
     12 SINGLEFS_EXPERIMENT_RERUN_REASON
     11 SLICE_TOTAL_OVERRIDE
     10 CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY
      8 SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_A_SYSTEM_CONFIGURATION_VALUE_THE_READER_DOES_NOT_ACCEPT
      8 NAMES_THAT_BYPASS_THE_POOL
      8 EXIT_GATE_MAY_SKIP
      8 CHECKER_QUICK_REASON
      7 TEST_NAMES_EXEMPT
      7 DRIFT_CONTROL_SERIAL_REASON
      7 CALIBRATION_RATIO_INVALID_REASON
      6 TASK_SERVER_SELFTEST_ALLOW_TMPFS
      6 RED_RECORD_REASON
      6 NOT_RUN_SAMPLE_REASON
      6 E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND
      5 WRITE_GUARD_DISABLE_SESSION_NAMES
      5 UNTRANSLATED_EXEMPT
      5 NO_RECORD_REASON
      5 NONE_MINIMUM_REASON
      5 BROAD_MUTATION_ALLOW_VOLATILE_HOME
      4 WRITE_GUARD_DISABLE_WRITER_LOCK
      4 WRITE_GUARD_DISABLE_SYNTAX
      4 WRITE_GUARD_DISABLE_SCOPE
      4 WRITE_GUARD_DISABLE_OVERWRITE
      4 WRITE_GUARD_DISABLE_COMPILE_FIRST
      4 WRITE_GUARD_DISABLE_CLOCK_TIMES
      4 UNJUDGED_SKIP_PREFIX
      4 SEGMENT_TWO_R10_SKIP_CONFIG_CHECK
      4 FAKE_CARGO_BEHAVIOR_OVERRIDE
      4 EXEMPT_MINIMUM_REASON
      4 BY_DESIGN_MINIMUM_REASON
      3 UNACCOUNTED_INVALID_REASON
      3 TRAILING_BLANK_EXEMPT_SUFFIXES
      3 TASK_SERVER_BYPASS_POLICY
      3 STUB_EXPAND_OVERRIDE
      3 REFUSED_REASON
      3 RED_VERDICT_REASON
      3 RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED
      3 MINIMUM_REASON
      3 LIBRARY_NOT_RUN_REASON
      3 LANDING_EXEMPT_LINE
      3 GPU_QUIESCE_SERIAL_REASON
      3 CPU_WRITINGS_SERIAL_REASON
      3 CAP_DAC_OVERRIDE
      3 CACHE_KEEPALIVE_DISABLE_HINT
      3 BROKEN_RECORD_REASON
      2 SUBMISSION_SERIAL_REASON
      2 SITE_SHIFTED_INTO_THE_REASON
      2 REUSE_OFF_REASON
      2 PINNED_RED_EXAMPLES_NOT_RUN_REASON
      2 NO_LIBRARY_REASON
      2 LANDING_EXEMPT_MARKS
      2 HEADING_IN_REASON
      2 HANDOVER_EXEMPT_PATH
      2 GATE_VERDICT_FALSE_SKIP_NAMED_CHECK
      2 FLAT_REGISTRY_REASON
      2 EVERY_REASON
      2 COMMIT_WHOLE_REASON
      2 BASE_ALLOWED
      2 AFTER_COMPACT_DISABLE
      1 RUN_WITH_MEMORY_CAP_PARALLEL_REASON
      1 PREFLIGHT_FORCE_GIVEN
      1 OPUS_ARM_F2_NO_SKIP
```

### 逐个归类

另查了派发提示里别的「<标签>：…」放行：派发闸里认「…已判」之外免判的标签行，只找到 ⑪ 的「只修锚点」「只补落产物」（下面 C4）；`NONE_WORDS` 认的「无」是必填行的值，不是放行。

**C1 `scripts/kb/kb-spec-check.py --allow-uncommitted <路径>`（打中，比 6 种还松）。**
（0273fccd）`scripts/kb/kb-spec-check.py:11`：「--allow-uncommitted：这一次确认…依据…里这份只在工作区的文件会跟这一批一起提交，放行它（⑥）；可重复。」
认目录前缀：（03162762）`scripts/kb/lib_basis_paths.py:176`：「directories = tuple(entry for entry in allowed if entry.endswith("/"))」——`--allow-uncommitted .claude/` 一个参数放过 `.claude/` 下全部有改动的依据。
不要理由、不留痕：放行之后绿行照打 `scripts/kb/kb-spec-check.py:498`：「依据没有别的会话还没提交的改动」，被放行的路径不另列。
放过什么：kb-spec-check ⑥（依据点名的已跟踪文件有别的会话还没提交的改动）。单次：是，每次调用给。谁会用：kb-spec-check 是子 agent 直接跑的文本工具（共用约束 `TEXT_TOOLS_RUN_DIRECTLY`），写回员自己跑到绿时可以加这个参数；定义与主 agent 定义里没教（本腿 grep `.claude/agents`、`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules`、`.claude/skills`、`scripts/agents` 零命中）。
判法改得了：被判的事实（「这份会跟这一批一起提交」）在 `.claude/batch-scope` 里已经有常设登记（kb-spec-check ⑥ 先认它），参数是那张表之外的单次口。

**C2 `SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>`（打中，同形、带痕迹）。**
`scripts/gate/admission.py:239`：「强制重跑：设 SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>。只越过…输入没变…，不越过前提；理由原样写进产物头。」；入口教法 `scripts/experiment-apparatus/replay.sh:3876`（6f15e22e）：「确要重跑：SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>」。
放过什么：admission 的退 77（输入没变）。单次：是，每次起命令设。比放行行多一样：理由写进新产物头，事后查得到。派发闸 ⑨ 起 admission.py 前专门清掉它（`.claude/hooks/runner-dispatch-guard.sh:73`：「起 admission.py 前从环境里清掉 SINGLEFS_EXPERIMENT_RERUN_REASON」），所以「准入已判」去掉之后，⑨ 拦下的同一件事到执行那一刻又有这个口——两处要一起判。

**C3 `scripts/kb/stale-candidates.py --allow-dirty-worktree <理由>`（打中，同形、带痕迹）。**
（859fe9bb）`scripts/kb/stale-candidates.py:12`：「确实整个工作区都是这一批的，给 `--allow-dirty-worktree <理由>`（理由至少 8 个字）照旧跑，输出头一行与变更清单头一行写明放行了、理由是什么」。
放过什么：工作区有不是这一批的改动时拒算变更清单。单次：是。判法改得了：`scripts/kb/batch-target.py` 已经能只取这一批的路径做提交对象（`.claude/main-agent.md` 里 sweep 那一行），有 `--target` 就不需要这个口。

**C4 「只修锚点」「只补落产物」（打中，同形；派发闸 ⑪ 比交回闸松）。**
交回闸：（bd16df77）`.claude/hooks/handback-guard.sh:374`：「LANDING_EXEMPT_LINE = re.compile(r"^[ \t>*_\-]*(?:只修锚点|只补落产物)[* \t]*(?:$|[：:，,。；;（(])", re.M)」——行首标签才认，免判「登记落地清单」。
派发闸 ⑪：`.claude/hooks/runner-dispatch-guard.sh:2432`：「if "只修锚点" in prompt or ("只补落产物" in prompt and not break_switch_is("runner-backfill-refused")):」——整份提示里有这四个字就免「这一段回答的岔路」，句中「这一段不是只修锚点」也算（交回闸已为同一种误认加了弄坏开关 `landing-exemption-substring` 改掉，派发闸这一处没跟）。
放过什么：派发闸 ⑪ 的岔路行、续做的「还差」与「承接」，交回闸 ⑦ 的登记落地清单。单次：是，一次派发写一行；不要理由、不核那一段是不是真的只修了锚点。判法改得了：交回闸那一刻看得到执行员实际写了什么（有没有往 `research/results/` 落产物、改没改装置），可以按写了什么判，不按声明判。

**不是单次放宽的（C6，逐个看过定义处）：**
- `--force`（门禁阶段与 `baseline-diff.sh --force-gate`）：准入没满足也照跑，是多跑，不是少判。
- `--skip-stage-two`（`scripts/mutation/broad-mutation.py`）：结局记「只过了第一段」、不算定论，没有放过判定。
- `--skip-apply`（`scripts/gate/merge-and-verify.sh`）：先用 `git apply --check --reverse` 核补丁打过，不是跳判。
- 门禁与 `crash-point-replay.sh` 里的 `--skip <名>`：按单元键复用已判过的测试，键对不上的照跑。
- `--skip-open-check`（`run_build_directory.py`）：文件头写只给调用方的弄坏开关用。
- `multi-host-run.sh` 的 `git commit --no-verify`：第二台同步树内部的提交；`verus-proof-check.py` 的 `--no-verify` 是 Verus 只编不证那一步。
- `--allow-empty`、`--skip-gpu`、`--skip-light`、`--skip-finished`、`--skip-torn-reading`、`--accept-long`、`--force-it`、`--forced`：git 参数、实验装置自己的取样开关、自证用的「不认的选项」样本，或 e184 bin 里的参数解析，都不越过门禁或钩子的判定（逐个 grep 过定义处，没逐行读完装置）。
- `RUNNER_DISPATCH_GUARD_BREAK=*`、`WRITE_GUARD_DISABLE_*`、`BROAD_MUTATION_ALLOW_VOLATILE_HOME`、`TASK_SERVER_SELFTEST_ALLOW_TMPFS`：自证的弄坏开关与只给自证的变量。前两组在钩子的真路径上也读环境（例 `write-guard.sh` 第 287、317 行），只是子 agent 与主 agent 都设不到钩子进程的环境；算常设机关，不算单次口。
- 名字里带 REASON / EXEMPT 的其余常量（`MINIMUM_REASON_CHARACTERS` 一族）：各门禁登记表一行的理由下限，表是常设的；没逐张核那些表有没有 B1 那样的「理由不核」问题，算这一条腿的限度。

## 要主 agent 跑的命令

本腿没起任何命令去喂钩子。下面 8 条都是派发闸的 `--check-prompt`（只读，不建锁、不读主会话记录；`.claude/hooks/selftest-registry.tsv` 里这份脚本登记为 light），一条一项。
都在仓根 `/home/fy5090/code/singlefs` 起（不给 `--cwd`）。**要在本机跑**：P4 的提示点名的 `/home/fy5090/.local/share/singlefs/work/d165-valves-r1-opus/probe-prior-result.md` 在本机草稿目录里，⑥ 那几条要此刻本机仓里的两份开着的快照；`task wait` 末行带 `host=peer` 的那几条作废重交。
结局都依赖跑的那一刻 d165-valves-r1 与 m4-gpugc-code-r2 还开着（判决文件不在）；哪一轮先写了判决，依赖它的那一条作废。

| 探针 | 命令原样 | 看哪几行、什么算打中 |
|---|---|---|
| P1 | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p1-kb-scribe-untouched-line.txt --type kb-scribe --root /home/fy5090/code/singlefs` | `TASK_SERVER_RESULT` 的 `exit=2`，`task-server/task log <号>` 里有「开工快照」一段、点名 m4-gpugc-code-r2 与 `.claude/kb/milestone/04-fourth-txn.md（出现在派发提示`；算 A3 打中 |
| P1b | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p1b-kb-scribe-without-untouched-line.txt --type kb-scribe --root /home/fy5090/code/singlefs` | 日志里没有「开工快照」那一段（`exit=0` 最好；若别的条拒了，只看有没有快照那一段）；与 P1 合起来把原因钉在「不碰的」那一节 |
| P2 | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p2-implementation-writer-untouched-mutations.txt --type implementation-writer --root /home/fy5090/code/singlefs` | `exit=2`，快照那一段点名 m4-gpugc-code-r2 与 `crates/mutations.tsv（出现在派发提示`；算 A3 实现员那一半打中 |
| P2b | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p2b-implementation-writer-untouched-none.txt --type implementation-writer --root /home/fy5090/code/singlefs` | 日志里没有快照那一段 |
| P3 | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p3-kb-scribe-reads-common.txt --type kb-scribe --root /home/fy5090/code/singlefs` | `exit=2`，快照那一段点名 d165-valves-r1 与 `.claude/agent-common.md（出现在派发提示`；算 A4 打中 |
| P4 | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p4-kb-scribe-continuation.txt --type kb-scribe --root /home/fy5090/code/singlefs` | `exit=2`，日志里有「认不出一份规格」；算 A7 打中 |
| P4b | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p4b-kb-scribe-without-continuation.txt --type kb-scribe --root /home/fy5090/code/singlefs` | 日志里没有「书记员规格」那一条（没有「认不出一份规格」）；与 P4 合起来把原因钉在「承接：」那一行 |
| P5 | `task-server/task submit -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/code/singlefs/research/prompts/d165-valves-r1-opus-model/probe-p5-kb-scribe-records-ruling.txt --type kb-scribe --root /home/fy5090/code/singlefs` | `exit=2`，日志里有「认不出一份规格」；算 A8 打中 |

`exit=` 不是 0 或 2 的（派发闸自己起不来）不算跑完。任何一条的结局与表里写的相反，那一格的「打中」撤回、照实写进判决。
主 agent 也可以不经服务、用自己的会话工具 `python3 scripts/agents/dispatch-prompt.py --check <探针> --type <类型>` 起同一份判法（它调的就是上面那条 `--check-prompt`）。
跑完之后不需要新派攻方腿读输出：表里写了逐条怎么判。

## 本腿顺手提到的改法各修哪一格

下面的改法都是本腿推的，没实现、没跑，被攻过零轮；不是正推腿的方案，也不替主 agent 采纳。每格标「推的」。

| 改法 | 修得到 | 修不到 / 新打中 | 状态 |
|---|---|---|---|
| ⑦ 读主会话 assistant 回复的 model 当解开信号 | A1 里主 agent 那一族 | A2（别的族照卡）；同池与否没量 | 推的 |
| ⑥ 剔「不碰」标签行与否定句里的路径 | A3 | A4（「只读」类说明认不全）、A6 | 推的 |
| ⑥ 派发一侧不判 crates/mutations.tsv，打补丁一侧只追加不算撞 | A5 | 替换与删行照拒（应当）；提交过之后倒推开工树那一份落人工核 | 推的 |
| ⑥ 挪到写的那一刻（写闸、Bash 检出钩子按真目标判） | A6、A3、A4 | A5 那种合理例外要按改动种类另设常设豁免（B3 的键） | 推的 |
| ⑧ 只认「规格：」标签行上的文件当候选，「判决：」标签行代替只认 main-verification | A7、A8 | 不带标签点名的规格不判（漏判，靠门禁兜） | 推的 |
| ⑧ 认不出规格而有判决就放 | A7、A8 | A9（漏放） | 推的 |
| ⑨ 并进重跑登记或问题单哈希 | 无 | A11 两头都中 | 推的 |
| ⑨ 派发一侧删掉，只留执行时 admission | A10 | C2 那个单次口要一起判 | 推的 |
| ⑯ 缩短在跑阈值 | 等待时长 | A13（漏放），阈值没量 | 推的 |
| ⑪ 改成行首标签认（与交回闸同一份） | C4 的子串误认 | C4 的「声明即免判」本身 | 推的 |
| kb-spec-check 去掉 `--allow-uncommitted`，只认 `.claude/batch-scope` | C1 | 批次范围外的合理例外（没想到具体情形） | 推的 |

## 没打中的形状

- ⑯ 去掉放行行之后的卡死：试了同会话客户端重启、别的会话关掉、认领文件坏了、同一份提示重试四种，都有不靠放行行的出路（A12）。
- ⑫ 去掉放行行之后的卡死：clerk 的工具已覆盖 WebFetch、WebSearch；没找到只有内置通用类型做得了、主 agent 自己又做不了的活（A14）。
- 6 种之外再找「…已判」标签：`research/`、`records/` 之外零处（grep-valve-labels.txt）。
- 门禁与脚本的 `--force`、复用式 `--skip`：都是多跑或按键复用，不放过判定（C6）。
- 「⑦ 改判法之后误放」：A2 的误放只在主 agent 与子 agent 不同额度池时出现，代价是一次失败派发；没算成打中。
- 取样范围：⑥ 只造了「不碰」行、只读引用两种提示措辞；⑧ 只造了承接报告、records 定案两种；没扫别的措辞（列表项、表格里的路径、围栏里的路径——按 1482 行它们同样算出现，推的）。

## 这条腿自己的限度

- 探针一条都没跑（子 agent 不喂钩子）：A3、A4、A7、A8 的结局全是读 1291–1325、1482–1491 行推的，主 agent 跑出之前按推的读。
- A2、A13 的关键量没量：主 agent 与子 agent 是否同一额度池、子 agent 两次写会话记录之间的最长间隔。
- A5 的残留风险（倒推开工树）没读 `scripts/agents/snapshot-tree.py`，推的。
- C6 里各门禁登记表的「理由」有没有 B1 那样不核的问题，没逐张查。
- ⑨ 只读了 admission.py 文件头与 `admit_experiment` 的强制开关几行，没读键怎么算；「产物头没有问题单一栏」是推的。
- 只读了派发闸、两份库、apply-writer-patch、admission、kb-spec-check、stale-candidates、handback-guard 与几张表的相关段落，别的钩子（bash-command-detector、heavy-test-guard、write-guard）只 grep 了参数与环境变量名，没逐段读它们有没有不带「已判」字样的放行行。

## 没做什么

- 不判 F 格（正推腿的逐行归类），没读正推腿的报告（禁读）。
- 不攻用户口径本身；没造绕过钩子的写法（C4 里⑪ 的子串误认只指出位置，没写成可用的提示）。
- 没往统一任务服务交任务；8 条探针命令在「要主 agent 跑的命令」一节。判决要引的数（探针结局）由主 agent 跑出。
- 没写计算模型，所以没有 `tools/rerun.sh`；模型目录里只有探针提示与 grep 原样输出。
- 没建仓副本、编译目录；草稿目录里只有 `code-files.txt`（grep 用的文件清单）与 P4 要用的 `probe-prior-result.md`，后者要留到主 agent 跑完 P4。

## 探针与输出文件

```
$ sha256sum research/prompts/d165-valves-r1-opus-model/* /home/fy5090/.local/share/singlefs/work/d165-valves-r1-opus/probe-prior-result.md
4d8b10cc3bb08d5ede1000f89aad0688091f591293a172e196216ea5f2d6055c  research/prompts/d165-valves-r1-opus-model/grep-override-flags.txt
9fcdb513406777150e7b3f5c7fab6c0a0d02c5d9146e33b2dde468550485285b  research/prompts/d165-valves-r1-opus-model/grep-override-variables.txt
fbcc2158d0920a5e15845d1bf0d8193b476f837283753c0de6bbc1243d891820  research/prompts/d165-valves-r1-opus-model/grep-valve-labels.txt
143f06014a8ba86e39dbaff157d1672ff544ea2b9f9768e5210e51d30a5cdc41  research/prompts/d165-valves-r1-opus-model/probe-p1b-kb-scribe-without-untouched-line.txt
45123b35b6008ab4de55167a661c9fe5534f0b209b99ebf6e997ec19fe7e8b32  research/prompts/d165-valves-r1-opus-model/probe-p1-kb-scribe-untouched-line.txt
0ecd062e00a20779ab34ab0c1f09c6762d1fc81891ee88a7170674704927517d  research/prompts/d165-valves-r1-opus-model/probe-p2b-implementation-writer-untouched-none.txt
629cc33539d61b45482e3ca765a1db16ef0eb8460df58838992978f75a3797e4  research/prompts/d165-valves-r1-opus-model/probe-p2-implementation-writer-untouched-mutations.txt
9ef69c2904cfd49b214f5f15e0c5e2f89bac0595f91e04f00263907fb1380013  research/prompts/d165-valves-r1-opus-model/probe-p3-kb-scribe-reads-common.txt
ac4b1fb49b2b345ada6644d0ddfd3bbe7f8acb987ab4215a3f1d9a7d6e41ac28  research/prompts/d165-valves-r1-opus-model/probe-p4b-kb-scribe-without-continuation.txt
9fc353813b8d329ee27bcc03f94c806aae20cead3aa3a482c5265d9c7db4ce38  research/prompts/d165-valves-r1-opus-model/probe-p4-kb-scribe-continuation.txt
a247b5609432fff538d1e2457e927ce8bae301c9d5e2a92763e430293a75d27b  research/prompts/d165-valves-r1-opus-model/probe-p5-kb-scribe-records-ruling.txt
28174382d547d89b4ea15f5803ef6c1af814fc71890f0a99f19990381332e0e5  /home/fy5090/.local/share/singlefs/work/d165-valves-r1-opus/probe-prior-result.md
```

## 复核数据

被判的七份此刻的 sha256（与 `research/prompts/d165-valves-r1-snapshot/files-sha256.txt` 前七行逐行相同；diff 只报快照多出的第八行，那一行是背景材料本身）：

```
$ sha256sum .claude/hooks/runner-dispatch-guard.sh .claude/hooks/lib_snapshot_conflicts.py .claude/hooks/lib_writer_lock.py scripts/agents/dispatch-prompt.py scripts/gate/apply-writer-patch.py .claude/main-agent.md .claude/agent-common.md | diff - research/prompts/d165-valves-r1-snapshot/files-sha256.txt; echo rc=$?
7a8
> 3330f73dce3209c08dee540c3c2948ee8a81701202c9e07713836538c1ee2f3f  research/prompts/_d165-valves-r1-body.md
rc=1
```

开着的三方轮（⑥ 认的那几轮，叫停登记表只按第一列粗数，没走 lib_abandoned_rounds 的判法）：

```
$ cd research/prompts && for d in *-snapshot; do r=${d%-snapshot}; [ -f "_$r-body.md" ] || continue; [ -e "$r-main-verification.md" ] && continue; echo "$r abandoned_rows=$(grep -c "^$r	" abandoned-rounds.tsv)"; done
d165-valves-r1 abandoned_rows=0
m4-gpugc-code-r2 abandoned_rows=0
```

## 引文核对

```
$ python3 scripts/agents/cite-check.py research/prompts/d165-valves-r1-opus-output.md
  ✓ 核了 45 处引文、14 处裸行号，对上 45 处，没判 14 处（其中裸行号没有引文可比 14 处；逐处列在上面）
```

没判的 14 处都是裸行号（grep -c 输出里的「文件:计数」与几处只写位置的指路），行号都在文件范围内。

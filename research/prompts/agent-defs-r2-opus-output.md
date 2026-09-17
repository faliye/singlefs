# agent-defs-r2 云端攻方腿（Opus）报告

攻击面：A-1（J7）、A-2（J3、J4）、A-3（J5 后半句）。所有行号都是用 `cat -n` 或 `grep -n` 在原文件里现查的（07:15–07:35 UTC）。按派发提示，这份报告没写报告文件。

## 复跑命令与模型文件

模型目录是 `research/prompts/agent-defs-r2-opus-model/`。三份脚本只在 `$TMPDIR` 下的 `mktemp -d` 里造文件，仓里的 `replace-batch.py`、`replace-once.py`、`quote-kb.py`、`49-history-brief.sh`、`lib-history-brief.py` 都只读调用，没做 git 操作。`history-write-diff.sh` 拷的是真仓 07:28 UTC 时的 `.claude/kb`，别的会话再写变更史之后，条目总数会变，三个判定量不该变。

```
cd research/prompts/agent-defs-r2-opus-model
TMPDIR=<草稿目录> nice -n 19 bash edit-half-apply.sh      # 对照 edit-half-apply.out
TMPDIR=<草稿目录> nice -n 19 bash history-write-diff.sh   # 对照 history-write-diff.out
TMPDIR=<草稿目录> nice -n 19 bash designer-quote-kb.sh    # 对照 designer-quote-kb.out
```

```
a592df63d693967c71c6cf1dec4ab3111fc51598a9015e24505cc18b934da243  designer-quote-kb.out
be08971bcf2bfd39a669190d1dfd2407c23b5a8094839ec179fd5e9537d8c4e1  designer-quote-kb.sh
5089348ecbba603c5ccd8482a15fea00f0a7146059507a1769c47f25258abc79  edit-half-apply.out
d5a415df64036e9f6f375b5afc2e9a9eb30c4d509fd10436d108cafbe38954a6  edit-half-apply.sh
71eb03ccfd2792ddd082a5e0ab959d83da78bdaf5cbc9710e711117a610adb2f  history-write-diff.out
aa6ff493dc382eb8445ae3d41a342d4becfb6986c51f6809a53c780f1e127596  history-write-diff.sh
```

## 各格判定一览

| # | 攻击面 / 对象 | 格 | 判定 | 一句话 |
|---|---|---|---|---|
| a | A-1：A1 改法落在 `kb-scribe` 第 1 步 | J7（新 J4） | **打中** | 「dry-run 核完、实写用 Edit 逐条改」就是 `replace-batch.py` 为防半套改动而废掉的「逐处写盘」。中途被别的书记员的 relabel 改了锚点，或被打断，就停在 1/3；旧做法在同一段历史上是 0/3 |
| b | A-1：「报告放进交回内容」（共用约束第 44 行） | J7（新 J3），另有 J2 字面 | **打中**（K3） | 要求整份报告一次放进 SubagentHandback，正是 `three-way-inference.md` 第 241–247 行靠「先分段落盘、回复只写短句」防住的那种丢失；304 份历史云端报告里有 8 份不小于那次撞上限的 92,740 字节 |
| c | A-1：C3 改法（49 号 `--write` 跑前跑后比） | J7（原情形可达，另引出新 J3） | **打中** | 占位符照样先写进别的会话的条目，再被 diff 看见。27 条决策的卡片全满，写任何一条新条目都会挤掉一行别人的条目，停下判据按字面每次都触发 |
| d | A-1：D2 改法（重跑登记进执行员写范围） | J7（新 J3）+ J5 | **打中** | 没有哪个定义产出 `e<号>-r<n>-prereg.md`：设计员占号只占新号，执行员的输入不收它。执行员要是自己写，就是读过上几次结果之后自己定判据 |
| e | A-1：D3 改法（修订只许收严，不许少报量） | J7（原情形可达） | **打中** | 试跑里被挪出产物的 A10 在登记第七节「断言」，不在第六节「量」。按新写法的字面，挪它照样合法，判断照样由执行员做 |
| f | A-1：E1 改法（清单里的文件不碰，其余照做） | J7（新 J3，走副本路线是 J4） | **打中** | 新加 core 模块时 `lib.rs` 在清单里（今天那份 diff 正好是别的会话的两行 `pub mod`），「其余照做」写出一个不进编译的模块：第 3 步证明不了会红，`check.sh` 绿在没编它的树上 |
| g | A-1：C2 改法（标题整行由主 agent 给） | J7（新 J3，轻） | **打中（轻）** | 标题里的「（其N）」第 18 行说主 agent 给，第 26 行说照当月文件定；并发时两句打架，出口只有停下 |
| h | A-1、A-2：C4 与重跑试跑之后的哈希改法 | J3（轻） | **打中（轻）** | 21 号、49 号 `--write` 改的 `decisions.md` 和 `decisions-history.md` 没有开工哈希，第 5 步要的两次哈希交不出来 |
| i | A-2：`experiment-designer`「登记的固定节名」第二节 | J3（走字面合法路线是 J4） | **打中** | `quote-kb.py` 只会整份写一个出口文件；设计员写范围只有登记本身、输入里没有草稿目录。出口写成登记时，占号文件头和第一节被盖掉，脚本照样退出码 0 |
| j | A-2：`implementation-writer` 第 3 步「先跑一份不改动的副本确认全绿」 | J3 | **条件打中** | 副本拷的是带别的会话在制改动的工作区，同一个测试二进制里有别人的红测试时，定义没有出口 |
| k | A-2：`mutation-triage` 第 3 步的固定形态 | J3（轻，毛病在改过的那一步上，不是改出来的） | **打中（轻）** | 142 张表里 4 张是 shell 探针表，套不进 `e7-index-bench/src/bin/<源文件>`；crates 那张按 59 号做法跑，永远没有第 4 步要认的「已还原，基线仍全绿」 |
| l | A-3：`kb-scribe` 翻状态 → 变异表锚点腐化与 E142 复跑对不上 | J5 | **打中** | 书记员报出来之后，没有一个定义的输入能接「修已有实验的变异表」或「重跑 E142」 |
| m | A-3：设计员「两种读法都登记」→ 执行员 | J5 | **打中** | 「问法待主 agent 在装置写之前删一种」这一步谁来删没有着落：执行员的修订不许少报，主 agent 的职责清单里没有改登记 |
| n | A-3：`mutation-triage`「补取样点、补断言由主 agent 另派」→ ？ | J5 | **打中** | crates 那一侧，实现员输入要里程碑步号；research 那一侧，执行员要登记，而补取样点是产物之后的重跑，落回 d |
| o | A-3：实现员 → 材料员（代码轮） | J5 | **打中（弱）** | 材料员要的是一个 `git diff` 范围，实现员自己说分不出谁改的；`crates/mutations.tsv` 两个会话都在追加 |
| — | A-1：B1 改法（指纹） | J7 | 没打中 | 三个 git 盲区都堵上了，还剩的是范围问题，不是改法引出的，见「没打中的形状」 |
| — | A-3：另外 5 对 | J5 | 没打中 | 见第三节表 |

---

## 一、A-1：第一轮改法在今天定义里的落点（J7）

### a（J7，打中，引出新 J4）：A1 让书记员回到「逐处写盘」

改法原文：

- `.claude/agent-common.md` 第 15 行：`- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；Bash 里只许定义点名的脚本自己写（`relabel-item.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你。`
- `.claude/agents/kb-scribe.md` 第 25 行：`1. 开工时先记下规格点名的文件与当月变更史文件的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录），先 `--dry-run` 核全部恰好命中一次；实写用 Edit 逐条改（写范围闸看得见），改完回读。`

改前原文（`_agent-defs-r1-appendix.md` 第 390 行）：`1. 把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录），先 `--dry-run`，全部命中恰好一次再实写并回读。`

这个脚本自己写明了为什么要两阶段（`research/scripts/replace-batch.py`）：
- 第 12 行：`为什么两阶段：逐处写盘的批量替换在第 k 处锚点不中时，前 k − 1 处已经落盘，留下半套改动。`
- 第 13 行：`2026-09-14 实测：一次 54 处的 kb 写回里有一处原文是「没被」而锚点写成「没有被」，两阶段让它一个文件都没写。`
- 第 38 行：`    """旧行为：逐处写盘。只给自证强制走一遍，证明它会留下半套改动。"""`

**序列**（每步都指到许可它的原句）：
1. 主 agent 派书记员 X 写回 D3 的一条定案。规格旧串是 D3 正文里一整行，行里带「D22（单元原子性怎么合成） 已定项 7」（第 17 行只要求「文件、旧串（原文整行）、新串、依据」）。这种行现查有 21 行，在 D22 以外的决策文件里：`grep -rn 'D22（单元原子性怎么合成） 已定项 7' .claude/kb/decisions/*.md | grep -v '^.claude/kb/decisions/22-' | wc -l` → `21`。
2. X 按第 25 行 dry-run 全部命中，开始用 Edit 逐条写。
3. 同时，另一个书记员 Y（别的会话派的，或同一个主 agent 并行派的，`CLAUDE.md` 第 20 行没限并发）把 D22 第 7 项翻回未定。它的第 3 步（第 27 行 `relabel-item.py`）改写全仓引用，X 下一条旧串不再命中。
4. X 照第 12 行「规格与原文对不上（旧串不唯一、找不到）就停下报告」停下，前 k−1 条已经落盘。第 15 行不许它用 Bash 跑 `replace-batch.py` 收回去。

被打断是同一个形状。`three-way-inference.md` 第 247 行记过腿中途撞周限额，逐条 Edit 把「中途」从一次 python 写盘拉长到几十次工具调用。

`edit-half-apply.out` 整行抄：
```
name=case case=C1_each_no_event mode=each event=none dry_run_rc=0 last_rc=0 applied=3/3
name=case case=C2_batch_no_event mode=batch event=none dry_run_rc=0 last_rc=0 applied=3/3
name=case case=S1_each_relabel_between mode=each event=relabel_after_first dry_run_rc=0 last_rc=3 applied=1/3
name=case case=S2_batch_relabel_between mode=batch event=relabel_after_first dry_run_rc=0 last_rc=1 applied=0/3
name=case case=S3_each_interrupted mode=each event=interrupt_after_first dry_run_rc=0 last_rc=interrupted applied=1/3
name=done cases=5
```
C1、C2 是对照：没有干扰时两种写法一样，这个模型分得出写法。Edit 用 `replace-once.py` 模拟，依据是第 15 行「与 `replace-once.py` 同义」。事件插在第 k 条之后时，逐条写落下 k 条，两阶段写落 0 条或全部；模型只取了 k=1。

**四句**：
- 分不分辨定义：只打 `kb-scribe`。它是唯一照一份「要么全落、要么全不落」的批量规格写的定义；实现员和执行员的 Edit 是逐处写代码。
- 被判的系统看不看得到：看得到 Edit 失败，但定义没给回滚法，第 15 行又封了 Bash 这条路。
- 满足的分句：J4「证据被污染（产物与输入对不上）」，写回的 kb 与规格对不上。主 agent 要是不认这一分句，就退到 J3「步骤之间互相矛盾」：第 12 行要停，第 15 行不许收拾。
- 改法在打中的格上还中不中：我的改法只在我的模型上量过，被攻过零轮。实写回到 `replace-batch.py` 两阶段；写之前把规格里每个文件路径按 Edit 形态、带 `agent_type=kb-scribe` 喂给 `.claude/hooks/agent-write-scope.sh`，有一条退出码 2 就整份不写。这样闸的判法照样生效，S1 变成 0/3，S3 的窗口缩回一次 python 写盘。

**什么现象会推翻它**：Claude Code 的 Edit 能一次原子地提交多文件多处；或书记员的规格从不跨两次以上工具调用。

### b（J7，打中，引出新 J3；K3）：「报告放进交回内容」拆掉了分段落盘的保护

- `.claude/agent-common.md` 第 44 行：`- 定义点名的产出文件（跑前登记、腿的 output、样本、运行记录、实验页、代码与 kb 的改动）照定义写进文件。「报告」本身完整放进交回工具（SubagentHandback）的内容里，只调一次（第二次会被拒）；派发提示里的报告路径是主 agent 存档用的，会话给 subagent 的系统说明若写着「不要写报告文件」，报告可以不写文件。`
- `.claude/rules/three-way-inference.md` 第 241 行：`云端腿按提示先把报告写进 `research/prompts/*-output.md`、最后才回复。**提示里还要写明分段写**：`
- 同文件第 242 行：`每一次工具调用写进文件的内容不超过 150 行，模型源码与报告都分段追加，第一段排他新建，回复只写短句。`
- 同文件第 243 行：`实测（2026-09-13，C199（实例代号递增与 jsn 断号即止互相矛盾） 第二轮）：反推腿想在一条回复里吐出整份报告与模型，撞上单次输出上限，`
- 同文件第 247 行（节选前半，整行太长，末尾见原文）：`**撞的是周限额时反过来：续那条腿，别重派。** …腿在最后一步（把全文当回复返回）才撞限额的，报告文件多半早已完整落盘——2026-09-05 两条腿的通知都显示 failed，而两份报告（41 KB / 59 KB）都在；…`
- 第一轮判决第 49 行（改法出处）：`| 报告不写文件（`sweep`、`experiment-runner`、核查员） | 试跑与核查员 | **采纳，改共用约束** | …`

共用约束第 17 行拿这条规则当「分段写」的依据，第 44 行却要求整份一次交回。这一轮我收到的派发提示原话就是「报告：放进交回内容，不写报告文件」。

**规模**（命令与原样输出）：
`ls research/prompts/ | grep -E '(opus|sonnet|verifier)-output\.md$'` 逐个 `wc -c`，统计结果是 `total=304 bytes>=92740:8`（最大的是 `d19-item6-a2-opus-output.md` 112521 字节）；按行数算是 `total=304 lines>=500:70 lines>=912:3`。

**序列**：攻方腿按 `three-way-attack.md` 第 19、30 行交一份 C199 第二轮那样大小的报告 → 第 44 行要求整份放进一次 SubagentHandback → 一次生成要吐约 92 KB → 撞单次输出上限（第 243–244 行那种）→ 交回失败，而派发提示与会话说明都让它不写文件 → 什么都没留下。第 245 行给的出路「先 `ls` 产物」，在新写法下 `ls` 不到任何东西；第 247 行记的「报告文件早已完整落盘」那条救法也没了。

**四句**：不分辨定义，所有交长报告的定义都中（K3）。腿看不到自己离上限还有多远。满足 J3「步骤之间互相矛盾」（第 17 行引的规则对第 44 行）；J2 的字面也满足，但 J2 不归我判，只并排了原文。改法（被攻过零轮）：报告按第 241–242 行分段写进草稿目录的文件，交回内容只写路径、sha256 与判定一览，主 agent 从文件存档。Claude Code 2.1.273 工具层拦的只是文件名以这四个词开头的 .md：二进制里现查的判法是 `/^(REPORT|SUMMARY|FINDINGS|ANALYSIS).*\.md$/i`；Bash 写的不拦（计划第 418 行）。

**什么现象会推翻它**：一次 SubagentHandback 成功交回 100 KB 以上的报告；或 harness 的单次输出上限不算工具调用参数。

### c（J7，原情形可达，另引出新 J3）：49 号 `--write` 的跑前跑后比

- `.claude/agents/kb-scribe.md` 第 26 行：`2. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb/decisions-history.md` 的文件头，不另定。然后 `bash .claude/gate.d/49-history-brief.sh --write`；跑前跑后各 `git diff -- .claude/kb/decisions-history.md .claude/kb/decisions-history/` 一次，它改到了这一轮之外的条目（例如给别的会话还没写快查的条目补「（待补）」）就停下交回，不自己收拾。`
- `.claude/gate.d/lib-history-brief.py` 第 106 行 `RECENT_PER_DECISION = 3`，第 111 行 `    for entry in entries[:RECENT_PER_DECISION]:`，第 169 行 `            open(path, 'w', encoding='utf-8').write(new_text)`（月文件先写，第 176 行才写 `decisions-history.md`）。
- 卡片满没满：`grep -c '^## D[0-9]' .claude/kb/decisions-history.md` → `27`；`grep -c '下面是最近 3 次'` → `26`；剩下 1 条是 `共改过 3 次：`。27 条全满，给任何一条决策加条目都会挤掉一行。

`history-write-diff.out` 整行抄：
```
name=case case=W1_only_this_round_entry foreign_entry=no write_rc=0 placeholders_written_into_foreign_entry=0 card_rows_removed=1 card_rows_removed_not_this_round=1 card_rows_added_for_foreign_entry=0 write_log=_✓_按原文重新生成了_.claude/kb/decisions-history.md：425_条条目，这次补了_0_条「（待补）」快查
name=case case=W2_plus_foreign_entry_without_quick foreign_entry=yes write_rc=0 placeholders_written_into_foreign_entry=2 card_rows_removed=2 card_rows_removed_not_this_round=2 card_rows_added_for_foreign_entry=1 write_log=_✓_按原文重新生成了_.claude/kb/decisions-history.md：426_条条目，这次补了_1_条「（待补）」快查
name=done cases=2
```
重新试跑时书记员也撞上了 W1：`agent-defs-r2-trial-kb-scribe.md` 第 78–79 行记「「最近 3 次」窗口挤掉最旧的那条（2026-09-13 总审核那条…」，它判成「按设计」，接着往下做。

**两种读法都中**：
① 字面读：卡片行是这一轮之外的条目的投影，每次写变更史都会被改到，第 2 步永远做不完（新 J3）。
② 放宽读：书记员自己判哪些改动算「按设计」，试跑就是这么做的。同一个判断也放得过 W2 的 `card_rows_added_for_foreign_entry=1`，即别的会话没提交的条目被生成进卡片，这是第一轮 C3 点名的后一半。而占位符那一半（`placeholders_written_into_foreign_entry=2`）是先写进别的会话的文件、diff 才看见，「不自己收拾」之后那份文件就一直改着。这是原情形。

**四句**：只打 `kb-scribe`。看得到 diff，但「改到了这一轮之外的条目」对卡片行没有定义。分句是 J3（字面）或 J4「改坏别的会话的文件」（原情形）。改法（被攻过零轮，就是第一轮攻方原本提的那个）：`--write` 之前先跑不带 `--write` 的 49 号，它报没写快查的条目逐条核是不是这一轮的，全是才写；卡片那份只核新增行，不核挤掉的行。W2 占位符那一格不中，W1 不中，别人的条目被加进卡片那一格仍中。

**什么现象会推翻它**：主 agent 在定义里写明卡片行不算「条目」，并且书记员派发时月文件里从来没有别人的无快查条目。

### d（J7 + J5，打中）：重跑登记进了写范围，没人产出它

- `.claude/agents/experiment-runner.md` 第 17 行：`- 跑前登记路径（`research/prompts/e<号>-preregistration.md`）。`
- 同文件第 33 行（节选）：`- 上面列的 bin 源文件、…、跑前登记的「修订」一段、实验页与它的索引行、重跑已有实验时的重跑登记（`research/prompts/e<号>-r<n>-prereg.md`）与 `.claude/kb/experiments-history.md` 里这个实验的条目、…`
- `.claude/hooks/agent-write-scope.tsv` 第 17 行：`experiment-runner	research/prompts/e*-r*-prereg.md	重跑已有实验的重跑登记（agent-defs-r1 判决 D2）`
- `.claude/agents/experiment-designer.md` 第 49 行：`- 只写 `research/prompts/e<号>-preregistration.md`（由 `claim-experiment.sh` 排他新建，之后 `>>` 追加或定点替换）。`
- `research/scripts/claim-experiment.sh` 第 51 行：`    echo "  ✗ E$number 已经有人用了"`
- `CLAUDE.md` 第 21 行：`| 要建计数实验 | `experiment-designer` 写跑前登记 → `experiment-runner` |`
- `.claude/singlefs-ai-sop/rules/test-discipline.md` 第 37 行：`**不许预设结论，设计阶段也不许参考任何已有结论**：这个项目上一轮的结论、`

**序列**：主 agent 要跑 E151 第七次 → 调度表第 21 行派设计员 → 设计员第 24 行 `claim-experiment.sh --next` 只给新号，占 E151 在第 51 行被拒，写范围里也没有 `-r<n>-prereg.md` → 执行员第 17 行的输入对不上。主 agent 要是把 `e151-preregistration.md` 加一句「重跑」交过去，第 33 行和表第 17 行就放行执行员整份新建 `e151-r7-prereg.md`，判据由执行员写。而它这一轮要改 E151 的实验页和变更史条目，已经读过前六次的结论，这正是第 37 行禁的事，也和它自己第 12 行「照登记做，不改判据」对不上。
第二种写法今天也在用：`research/prompts/e152-preregistration.md` 第 236 行 `## 十二、里程碑「第二个事务」的 singlefs 重跑（2026-09-17 JST 写，装置改动之后、一格产物都还没跑）`、第 252 行 `## 十三、…`，重跑写成原登记里的新节，而执行员对原登记只许写「修订」一段。

**四句**：打的是设计员到执行员这一对。看得到。分句是 J3「输入不够开工」（字面）或 J4「判决实际由 subagent 做出」（执行员自己写重跑判据）。改法（被攻过零轮）：设计员输入加「重跑：实验号与第几次」，写范围加排他新建的 `e<号>-r<n>-prereg.md`；执行员对重跑登记也只许写「修订」一段。

**什么现象会推翻它**：主 agent 把「写重跑登记」写进 `CLAUDE.md` 第 10 行留在主 agent 的那份清单。

### e（J7，原情形可达）：D3 改法按字面照样放行 A10

- `.claude/agents/experiment-runner.md` 第 12 行：`照登记做，不改判据。单测跑完、产物一次没跑之前看出登记要补的，写进登记的「修订」一段（改了什么、依据哪个单测读数、时点在产物之前），原判据原样保留；修订只许收严或补一条臂与判据，不许少报登记里的任何一个量，要少报交主 agent 定。产物跑过之后不改登记，交主 agent。`
- `.claude/agents/experiment-designer.md` 第 38 行 `| `## 六、报哪些量与各自的判据` | 每个量一行：怎么算、门槛、判定；门槛不能从臂的定义直接推出 |`，第 39 行 `| `## 七、钉绝对值的断言` | 分两类：出自被测条款本身的、独立算出并用命令核过的 |`
- `research/prompts/agent-defs-r1-trial-experiment-designer.md` 第 36 行：`| 七 | 钉绝对值的断言 A1–A10，每条带独立出处；A7 是一张手写阶梯表（宽 10–21）加三个查询点的已知答案 |`
- `research/prompts/agent-defs-r1-trial-experiment-runner.md` 第 36 行（节选）：`…主 agent 裁定「Q5 不进报告」之后，怎么处理 A10（一个同时依赖 Q5 与 Q6 的恒等式）没有现成指引**——我自己判断把它挪成单测（不进 E7RESULT 报告流）…`

**情形**：A10 是第七节的「断言」，不是第六节的「量」，挪进单测不算「少报一个量」。单测里的断言红了就断构建，执行员还可以说这是「收严」。第一轮 D3 要拦的那一步（怎么处置 A10 由执行员决定）在新写法下字面照样合法。

**四句**：只打执行员。它看得到自己在做判断。分句是 J4「判决实际由 subagent 做出」，与第一轮归格时的保留相同。改法（被攻过零轮）：「修订不许改登记第六、七、九节里任何一条的去留和报告方式（进不进产物），要动交主 agent」。A10 那一格改后不中。

**什么现象会推翻它**：主 agent 认定「把断言从产物挪进单测」本来就不归这条管；或执行员照新写法把 A10 交了回来（没有试跑覆盖这一步）。

### f（J7，引出新 J3；走副本路线是 J4）：E1 停在 `lib.rs`，「其余照做」写出一个不编译的模块

- `.claude/agents/implementation-writer.md` 第 28 行：`5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。`
- 同文件第 26 行（节选）：`3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，…先跑一份不改动的副本确认全绿；…`
- 今天的 `git diff -- crates/singlefs-core/src/lib.rs` 只有 `+pub mod instance_table;`、`+pub mod mount;` 两行，对应的两份文件在 `git status` 里是 `??`。`grep -c '#\[cfg(test)\]' crates/singlefs-core/src/*.rs`：lib.rs 以外的 17 个文件里 15 个带单元测试。

**序列**：新的一步要加一个 core 模块和它的单元测试 → 主 agent 按第 19 行把 `lib.rs` 列进清单（今天正是这个状态）→ 第 28 行不碰 `lib.rs`；模块代码落在新文件上，不是「只能落在 lib.rs 上的条款」，于是照写 → 新文件没被声明，它的测试不在任何测试二进制里 → 第 26 行「跑那条测试所在的整个测试二进制看它红」做不到，「不改动的副本确认全绿」白绿。
下一步只有两条路：交回（第 26 行没做到）；或只在副本里补 `pub mod`，共用约束第 15 行说「草稿目录里…用什么写都行」，在副本里证明会红。后一条路交回的「每条都红了」证明在一棵主工作区没有的树上，第 4 步 `check.sh` 又绿在不编这个模块的树上，模块里连编译错误都看不见。重新试跑用的是 `pub use`，模块已经声明过，没碰到这种情形（`agent-defs-r2-trial-implementation-writer.md` 第 96 行）。

**四句**：只打实现员。它看得到自己没进编译。分句是 J3「步骤之间互相矛盾」（第 26 行对第 28 行）；走副本路线是 J4「证据被污染」。改法（被攻过零轮）：第 5 步加「新文件要靠清单里的文件才进编译（模块声明、`[[test]]`）的，整条停下，不写那个新文件」。

**什么现象会推翻它**：新模块的测试一律放 harness 的 `tests/`（自动发现，不经过 lib.rs），并写进定义。

### g（J7，引出新 J3，轻）：标题整行由主 agent 给，「（其N）」却由当月文件定

- `.claude/agents/kb-scribe.md` 第 18 行：`- 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题决定条目挂在哪条决策下、条目里裸写的分项归谁，所以也由主 agent 给）。`
- 第 26 行（整行见 c）有「同一天多条怎么编「（其N）」，照当月文件…不另定」。`.claude/gate.d/32-history-ordinal.sh` 第 5 行：`# **这个编号是先到先得的公共资源**：并发会话各写各的，取号前不查最大号就会撞。`

**情形**：今天月文件第 8 行已经有一条裸标题 `### 2026-09-17：…`，主 agent 写规格时给「（其二）」；书记员写之前，别的会话也写了「（其二）」。照标题写，32 号红，改不了；照第 26 行改号，又违反第 18 行和第 12 行「一个字不加」。出口只有停下，所以算轻。改法（被攻过零轮）：标题里「（其N）」那一段由书记员照当月文件现取，其余逐字照给。

### h（J3，轻）：`--write` 改的两个文件没有开工哈希

- `.claude/agents/kb-scribe.md` 第 29 行：`5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。`
- 第 25 行只让开工时记「规格点名的文件与当月变更史文件」；第 27 行只补 relabel 预演列出的文件。而 49 号 `--write` 改 `.claude/kb/decisions-history.md`（`lib-history-brief.py` 第 176 行），21 号 `--write` 改 `.claude/kb/decisions.md`（试跑报告第 142–143 行）。两份都不在开工哈希里，第 29 行要的「两次哈希」交不出来。试跑观察 3（`agent-defs-r2-trial-kb-scribe.md` 第 337–339 行）点过「两个 `--write` 门禁阶段」，改法只补了 relabel 那一半。

---

## 二、A-2：重新试跑之后改的地方（J3、J4）

### i（J3，走字面合法路线是 J4）：设计员第二节要用 `quote-kb.py`，写范围装不下它的产出

- `.claude/agents/experiment-designer.md` 第 34 行：`| `## 二、被测条款与它引的定义` | `research/scripts/quote-kb.py` 整段抄，脚本回读一致 |`（这一行是「登记的固定节名」新加的，第一轮原文 `_agent-defs-r1-appendix.md` 第 72–139 行 `grep quote-kb` 零命中）
- 同文件第 49 行（整行见 d）：只写登记本身。输入一节（第 15–19 行）没有草稿目录。
- `research/scripts/quote-kb.py` 第 396 行：`    with open(dest, 'w', encoding='utf-8') as f:`
- `research/scripts/claim-experiment.sh` 第 57 行：`  if ! ( set -C; printf '# E%s 跑前登记：%s\n\n写于 %s JST，装置写之前。\n' "$number" "$short_name" \`

`designer-quote-kb.out` 整行抄：
```
name=direct_to_prereg quote_kb_rc=0 lines_before=7 lines_after=8 header_before=1 header_after=0 section_one_after=0 log=_✓_1_段整抄进_research/prompts/e999-preregistration.md，回读逐字节一致
```

**情形**：设计员能合法写的文件只有登记本身，而 `quote-kb.py` 只会整份写一个出口文件。出口写成登记本身时，占号时刻那行「写于 … JST，装置写之前」和第一节被盖掉，脚本报「回读逐字节一致」、退出码 0，没有东西报警。那行时刻正是登记先于装置的证据。出口写到 `/tmp` 再 `>>`，又出了第 49 行的写范围，而输入里没有能放它的草稿目录。

**四句**：只打设计员。它看不到盖掉了什么，脚本报的是绿。分句是 J3「要写的文件不在写范围里」，走字面合法路线则是 J4「证据被污染」。改法（被攻过零轮）：设计员输入加草稿目录，第二节写明「`quote-kb.py` 出口放草稿目录，再 `>>` 追加进登记」。

**什么现象会推翻它**：`quote-kb.py` 有追加模式或 stdout 模式（第 1–40 行的用法里没有）。

### j（J3，条件打中）：实现员「不改动的副本确认全绿」遇到别的会话的红测试

第 26 行（节选见 f）改成了跑整个测试二进制，并先确认不改动的副本全绿。副本是用 `rsync` 拷的工作区，别的会话没提交的改动也在里面：今天 core 的单元测试二进制编的就是带 `mount.rs` 的那份（它有 `#[cfg(test)]`）。在制改动红着是常态：`.claude/kb/decisions-history/2026-09.md` 第 16 行 2026-09-17 那条的依据里写「改法之前红、改法之后绿」。副本一红，第 26 行没有出口：不能修（第 28 行），也没说减掉基线红集之后照做。改前只「跑那条测试」，过滤掉就不受影响，所以这是改动引出的。只在「同一个二进制里有别人的红测试」时中，没现场取样到，记条件打中。

### k（J3，轻；毛病在改过的那一步上，不是改出来的）：`mutation-triage` 第 3 步的固定形态

- `.claude/agents/mutation-triage.md` 第 24 行：`3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；在仓根下跑会被报成「基线就是红的」）；crates 那张表按 `.claude/gate.d/59-crates-mutation-replay.sh` 的做法在仓副本里跑。`
- 第 25 行：`4. 确认整张表跑完：收尾没有「已还原，基线仍全绿」就是中途退出，这一次的数不算，照实报。`

`ls research/mutations | wc -l` → `143`（含 README.md），逐个对 `e7-index-bench/src/bin/<名>.rs` 找不到源文件的有 `e125_zoned_wp`、`e129_tear_injector`、`e129_thin_neighbour`、`e129_thin_rmw`。这四张的表头写「被测装置是 shell 探针 …（不是 Rust 二进制）」，套不进第 24 行。crates 那一路：59 号的输出里 `grep -n '已还原'` 零命中，第 25 行因此永远判「这一次的数不算」；它也没有「无效」这一类，第 5 行要的三个数凑不齐。

---

## 三、A-3：调度表里前后接着派的几对（J5 后半句）

| 前一个 → 后一个 | 前一个的产出 | 后一个输入要的 | 判定 |
|---|---|---|---|
| `kb-scribe`（翻状态）→ ？ | 第 27 行：33 号红点名的变异表锚点、「改到的实验源码逐个列给主 agent（入库产物里印着旧标签的，复跑会对不上）」 | 见 l | **打中（l）** |
| `experiment-designer` → `experiment-runner`（两种读法） | 第 17 行：「问法待主 agent 在装置写之前删一种」 | 执行员第 17–19 行没有「问法」这一栏；第 12 行不许少报 | **打中（m）** |
| `experiment-designer` → `experiment-runner`（重跑） | 只产出新号的登记 | 第 33 行点名的 `e<号>-r<n>-prereg.md` | **打中（d）** |
| `mutation-triage` → ？ | 第 31 行「补取样点、补断言由主 agent 另派」 | 见 n | **打中（n）** |
| `implementation-writer` → `three-way-materials`（代码轮） | 第 37 行：「这一轮写过的文件清单（你自己列；别的会话同时在改 `crates/` 时，`git diff --stat` 分不出谁改的）」 | 材料员第 19 行：「代码轮另给 diff 的范围（例：`git diff <基准> -- crates/`）」 | **打中（弱，o）** |
| 正文 → `three-way-materials` → 四条腿 | 清单、附录、背景材料，代码轮另有 `_<轮>-diff.md` | 腿要背景材料路径；diff 由正文指路（`_m2-code-r1-background.md` 第 8 行有先例） | 没打中 |
| 四条腿 → `three-way-verifier` | 报告（主 agent 存档）、本地腿的核对表与运行记录 | 全部腿报告的路径 | 没打中（存档丢失的风险见 b） |
| `implementation-writer` → 三方 → `crash-verifier` | 同上 | 「`git diff --stat -- crates litmus` 原样，或提交区间」，只拿来写报告，阶段本身跑整棵树 | 没打中 |
| `stage-mine.py` → `gate-triage` | 暂存区 | 「`git diff --cached --stat` 原样」 | 没打中 |
| `sweep` → `kb-scribe` | 五类清单 | 逐条规格；kb 以外的文件书记员停下，sweep 第 12 行写了「或自己改」 | 没打中 |

`CLAUDE.md` 第 10 行有兜底「一次性的活照旧临时写提示派 general-purpose」。l、m、n 的活每次翻状态、每次补取样点都会出现，不是一次性的。而 general-purpose 在写范围闸里整个放行（`agent-write-scope.sh` 第 49–50 行：没有项目定义就放行），兜底等于把这批写落在闸外。

### l（J5，打中）：翻状态腐化的锚点和对不上的产物没有接手的人

`.claude/gate.d/stage-owners.tsv` 第 20 行：`33-mutation-tables.sh	experiment-runner,kb-scribe	实验二进制有同名变异表；relabel-item.py 改写实验源码之后锚点会腐化，书记员也跑`。重新试跑里当场红：`agent-defs-r2-trial-kb-scribe.md` 第 130 行 `      e100_superblock_slot 的 M3_间接层没省（表第 3 行，命中 0 次）`。
能写 `research/mutations/**` 的只有执行员（写范围表第 10 行），而它第 17 行要跑前登记，第 33 行只许写「上面列的」（它自己这个实验的）变异表。变异分诊第 31 行「不改变异表」；实现员第 33 行「不写 `research/`」；书记员第 33–35 行没有这一处。E142 产物里印着 17 处分项标签（`grep -c '已定项\|未定项' research/results/<replay.sh 第 157 行的产物>` → `17`），翻其中任何一个都要重跑 E142，那就落回 d。
**推翻**：调度表加一行「翻状态之后修锚点 / 重跑」，并给一个定义的输入对上它。

### m（J5，打中）：「两种读法都登记」之后谁删

设计员第 17 行：`- 要回答的问题（一种读法；有两种读法的先交回主 agent 定问法。两种读法都报、又不改变任何判据时，可以两种都登记，文件头挂一句「问法待主 agent 在装置写之前删一种」）。`
删一种读法就是少报一个量。执行员第 12 行不许少报、要交主 agent；主 agent 的职责（`CLAUDE.md` 第 10 行）里没有改登记；设计员已经交回，再派它，它第 24 行会重新占号。第一轮试跑走的路是主 agent 在派发提示里给决定、执行员写进修订（`agent-defs-r1-trial-experiment-runner.md` 第 9 行），D3 改法之后这条路字面上被堵了。

### n（J5，打中）：补取样点、补断言派给谁

变异分诊第 31 行（整行见 k 附近）。crates 那一侧，调度表第 15 行把「补测试与变异」交给实现员，而实现员第 17 行要「里程碑文件与步号（或并行线编号），这一步的验收标准」，分诊的产出里没有步号。research 那一侧，已有实验补取样点改的是产物跑过之后的装置，执行员第 12 行「产物跑过之后不改登记，交主 agent」，又落回 d 的重跑登记。

### o（J5，弱）：实现员的改动分不出来交给材料员

材料员第 19 行要一个 diff 范围，并照 `research/prompts/_m2-code-r1-diff.md` 的形态（那份是 `git diff HEAD -- crates/` 加新文件全文）。实现员的第 5 步让它和别的会话按文件分开，只有 `crates/mutations.tsv` 两边都在末尾追加（第 28 行），此刻它在 `git status` 里是 ` M`。按文件取 diff 时，这一份混着别人的行，腿会去攻别人追加的变异；清单只在派发那一刻有效，之后别的会话再碰同一个文件也分不出来。

---

## 没打中的形状

| 对象 | 试过的形状 | 取样范围 | 为什么不算 |
|---|---|---|---|
| B1 指纹（`crash-verifier` 第 27 行） | 未跟踪文件改内容、改完暂存、改完提交；只暂存不改内容；二进制文件的 diff；gitignore 掉的文件 | `git ls-files crates litmus` 49 个文件逐个 `file --mime-encoding` 没有二进制；`.gitattributes` 不存在；`git ls-files --others --ignored --exclude-standard crates litmus` 零输出；`git config diff.external` 空 | 第一轮三格都堵上了。还剩的是范围：55 号读 `research/scripts/replay.sh` 的 E142 行和那份产物（第 34、58 行），59 号拷仓根 `Cargo.toml`、`Cargo.lock`、`research/results`（第 67 行），这些不在 `-- crates litmus` 里。这不是改法引出的，要别的会话恰好在阶段跑的时候重跑 E142；另外 HEAD 那一项在别的会话提交无关文件时会误报，只添噪声 |
| A1 的 `cargo` 放行 | `check.sh` 格式红时的出路 `cargo fmt --all` 改到别人的文件 | 真仓 07:31 UTC `cargo fmt --all -- --check` 取样一次：`rc=0`、`Diff in` 零行（第一轮 02:42 那次也干净） | 要别的会话此刻留着没格式化的代码，两次都没取样到 |
| 工具层拦报告文件 | 执行员用 Write 建实验页或重跑登记会不会被当报告拦 | Claude Code 2.1.273 二进制里现查判法：`/^(REPORT\|SUMMARY\|FINDINGS\|ANALYSIS).*\.md$/i` | 实验页和登记的文件名不以这四个词开头，拦不到 |
| 续做之后第二次交回 | 先按「输入缺一样就不开工」交回一次，再续做交完整报告 | 计划第 246 行只实测过同一次运行里第二次被拒 | 续做之后算不算第二次没实测，只列在限度里 |
| 实现员第 6 步「条款没写的 derive 不加」 | 字面读时 `assert_eq!` 要的 `PartialEq`、`Debug` 加不了 | 重新试跑报告第 99 行：它自己加了 5 个 derive，只把 `Ord`、`Hash` 列成没加 | 分不清：字面读卡测试，实际读由实现员判「哪些是测试要的」；没有定义原句逼它走到卡死 |
| `mutate.sh` 并发 | 两个 `mutate.sh` 共用默认 `MUTATE_TARGET` | 读 `research/scripts/mutate.sh` 第 54–65 行 | 没实测 cargo 锁在跑测试期间放不放；也没找到定义原句让两个分诊并行 |
| 核查员第 4 步「按字段比」 | 由核查员自己定哪些字段算路径 | 读定义 | 没有可达序列让它把真差异判成路径差 |

「没打中」这一栏只抽了一次样（这一条腿、这一次），按 `three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，不能拿来支撑「这几格没问题」。

## 这条腿自己的限度

- 三个模型都是最小化的：Edit 用 `replace-once.py` 模拟（依据共用约束第 15 行「同义」），没派真的书记员，也没测 Edit 工具的「读后被改」检查；并发的 relabel 用一次 python 替换代替，没跑 `relabel-item.py`。
- b 的输出上限是按规则第 243–247 行的实测与历史报告大小推出来的，没实测一次 100 KB 的 SubagentHandback；上限具体多少 token 没查到。
- f、j 没编译 Rust，「没声明的模块不进编译」「同一个二进制里有别人的红测试」是从 Rust 语义与工作区现状推的；j 没取样到真的红。
- `history-write-diff.sh` 拷的是真仓此刻的 kb，别的会话在改，过后复跑条目总数会变。
- 所有改法都只在我的推导或模型上成立，被攻过零轮（K6）。
- 为了让这份报告放得进一次交回（正是 b 说的那件事），引文只整行抄了承重的几行，其余写成行号与节选，并注明了「节选」。

## 没做什么

- 没判 J1、J2（b 只并排了原文）、J6、J8，没判 J5 的前半句；没复核第一轮判决够不够得着（辩方的活）。
- 没改任何定义、共用约束、hook、写范围表或 `.claude/settings.json`；模型目录之外没写仓里的文件。
- 没编译 Rust，没跑门禁阶段，没跑 `mutate.sh`、`relabel-item.py`；真仓上只跑了 `cargo fmt --all -- --check`（只读）、`git diff`、`git status`、`git ls-files`、`git config --get`。
- 为读工具层判法，对 Claude Code 二进制跑过一次 `grep -a`（只读，花了 2 分钟以上）。
- 没做 git 写操作，没提交。
- 草稿目录 `/tmp/claude-1000/agent-defs-r2-opus/` 里只有 `fmt-check.txt`，模型的临时目录跑完都删了。

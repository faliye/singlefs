# c472-batch-scope 阶段同步逐行判（F1–F4）

**结束状态**：暂存区树 `git write-tree` 实测为 `56bc7c7924e088f4e3fc836704a2b4894ccf66a9`，与派发提示给的一致，未停下。
以下每一行都用 `git show 56bc7c7924e088f4e3fc836704a2b4894ccf66a9:<路径>` 读那一版内容，不读工作区。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F1 | .claude/kb/checks-owed.md:435 | 事件句不改 | 这整行就是 C472 今天（2026-09-22，与阶段日期一致）的还清记录：「新立门禁 11 号……**它的基准是 HEAD 不是 `GATE_BASE`**……触发文件的清单同日从 68 号的内嵌 python 挪进 `.claude/gate.d/knowledge-sync-triggers.tsv`」。读 `.claude/gate.d/11-batch-scope.sh` 核过：六条判据（①触发文件表唯一登记位、②③登记与实际触发文件互相包含、④理由非空、⑤仓库根起、⑥68 号还读同一份表）与这一行逐条对得上；`fixtures/11-batch-scope.sh/{red,green}/{expect,setup.sh}` 四个文件都在，判别力自证的描述属实。说的是这一阶段做了什么、门禁现在怎么判，不是过时的现状句。 |
| F2 | .claude/kb/checks-owed.md:397 | 不相干 | 这一行是 C449「两套基准算法让带不带 --staged 判出不同的改动范围」，讲的是共享 `diff_base` 与项目阶段 `GATE_BASE > upstream > HEAD` 不一致导致 68 号报数不同（3 个对 17 个），通篇没有一处提到触发文件清单存在哪个文件里；`grep -n "内嵌 python\|knowledge-sync-triggers" checks-owed.md` 只命中 435 行，397 行不含这个事实的任何一个分句，是检索词「触发文件」撞上了、说的却不是清单挪没挪出来这件事。 |
| F2 | .claude/kb/checks-owed.md:398 | 不相干 | 这一行是 C450「归档删掉的 sync 记录还在役」，讲的是门禁 91 号删旧实验记录与 68 号要求触发文件被同步记录点名之间的基准差一格，同样通篇不提触发文件清单存在内嵌 python 还是独立 tsv 文件（同上一行的 grep 结果，398 行不在命中里），是检索词「触发文件」撞上了，不是这件事实。 |
| F2 | .claude/kb/checks-owed.md:435 | 事件句不改 | 与 F1 那一行是同一行，对 F2 的事实（触发文件清单从哪挪到哪）尤其贴：「触发文件的清单同日从 68 号的内嵌 python 挪进 `.claude/gate.d/knowledge-sync-triggers.tsv`，两道门禁读同一份，11 号第六条就是钉这件事」。读 `.claude/gate.d/68-knowledge-sync.sh` 头部判据 ① 核过：它现在写的正是「`.claude/gate.d/knowledge-sync-triggers.tsv` 里任一条正则的路径……门禁 11 号……读同一份」，与这一行说的一致，是这一阶段做成的事的记录，不是待改的旧说法。 |
| F3 | .claude/kb/checks-owed.md:22 | 不相干 | 这一行是 C8「门禁范围判不出来」，讲的是按 diff 算受影响布局集合、`GATE_BASE` 当基准会把上一个提交已验过的改动重复落进这一轮（它自己 2026-09-21 的补丁），`grep -o "触发文件[^。，、]*"` 在这一行零命中，没有一处说「还没有一道处理暂存区触发文件范围的门禁」；是检索词「GATE_BASE」「HEAD」撞上了 C8 自己的基准讨论，不是这件事实。 |
| F3 | .claude/kb/checks-owed.md:397 | 不相干 | 同 F2 对这一行的判定：C449 讲的是共享 `diff_base` 与项目阶段基准算法不一致，不是「还没有门禁处理暂存区触发文件范围」这件事——这件事恰恰是 C472／门禁 11 号才补上的，397 行里没有这句话的任何分句，是检索词撞上了。 |
| F3 | .claude/kb/checks-owed.md:435 | 事件句不改 | 直接对上 F3 的新事实：「**它的基准是 HEAD 不是 `GATE_BASE`**：这一道问「这一次提交要带哪些」，取 `GATE_BASE` 会把上一个提交的触发文件也算进来、每次提交之后必然误红（与 C450……躲开的是同一格）」——这正是「新立门禁 11 号把基准定成 HEAD、不是 GATE_BASE」这件事本身的记录，日期 2026-09-22，是事件句。 |
| F3 | .claude/kb/checks-owed.md:436 | 不相干 | 这一行是 C468「判错的原始证据不落盘」，讲门禁 68 号自己新加的判据 ⑤（「⑤ 的基准是 HEAD 而不是 `GATE_BASE`（①–④ 用后者）」）为什么要用 HEAD 当基准，是 68 号自身另一条判据的取舍，不是「还没有门禁处理暂存区触发文件范围」这句话，与新立的 11 号无关，是检索词「HEAD」「GATE_BASE」撞上了。 |
| F4 | .claude/agent-common.md:53 | 不相干 | 这一行是通用机制描述「哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/gate.d/stage-owners.tsv`（第二列是 agent 名，逗号分隔）」，不含任何具体阶段数或「还没有 11 号」这类断言；门禁 11 号登记之后这句话依旧成立（第二列仍是逗号分隔的 agent 名，见 stage-owners.tsv 第 5 行 `11-batch-scope.sh\tgate-triage\t…`），机制本身没变，不是这件事实要判的那一句。 |
| F4 | .claude/agent-common.md:54 | 不相干 | 这一行是取阶段的 awk 命令本身：``awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { … }' .claude/gate.d/stage-owners.tsv``，是命令定义不是现状断言，门禁 11 号加进表之后这条命令原样能把它取出来（`awk … -v me=gate-triage` 会列出 11-batch-scope.sh），命令本身没有任何字要改。 |
| F4 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:28 | 事件句不改 | 「门禁 27/33/34/40/52/65/80/85/86/88 十个阶段（experiment-runner 在 `stage-owners.tsv` 里登记的那些）全部跑过，本轮改动涉及的都是绿的」说的是 e155 那一轮改动跑过、判绿的那批阶段，是那次实验交回时的记录。现查 `git ls-tree -r 56bc7c7924e088f4e3fc836704a2b4894ccf66a9 -- .claude/gate.d/ \| grep '^\.claude/gate\.d/65'` 零命中——门禁 65 号今天根本不存在，说明这十个阶段是 e155 跑完那一刻的快照，不是「experiment-runner 现在归这十个阶段」的现状断言；门禁 11 号新增也不改变这句话在说什么。 |
| F4 | records/2026-09-16-subagent拆分提案.md:190 | 不相干 | 这一行讲的是「哪个门禁阶段由谁先跑，只写在 `.claude/gate.d/stage-owners.tsv`，定义里不手抄阶段号」这条**设计原则**适用于哪些 agent 定义（`kb-scribe`、`experiment-runner`、`implementation-writer`、`three-way-materials`、`prior-art`、`crash-verifier`），不是「表里现在登记了多少阶段」的现状断言；`gate-triage` 不在这份名单里是因为它跑全部阶段、不用从表里按 agent 名取自己的子集，与门禁 11 号有没有登记无关，是检索词「stage-owners.tsv」撞上了。 |
| F4 | records/2026-09-16-subagent拆分提案.md:308 | 要改 | 原句「每个项目本地阶段先由哪个 agent 跑，登记成一张表，只写这一份：kb-scribe 31 个、experiment-runner 11 个、gate-triage 7 个、crash-verifier 4 个、three-way-materials 2 个、implementation-writer 与 prior-art 各 1 个（有 4 个阶段归两个 agent）」是 2026-09-17 建表当天的快照，删掉日期照样读成「现在的分配是……」，属现状句。用 `awk -F'\t' -v me=<agent> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv \| wc -l` 对 56bc7c7 这棵树逐个数过：kb-scribe 37、gate-triage 15、experiment-runner 13、crash-verifier 7、implementation-writer 6、three-way-materials 2、sweep 2、prior-art 1、mutation-triage 1（68 行非注释三列齐全，其中 12 行两个 owner、2 行三个 owner）。改后的句子：「每个项目本地阶段先由哪个 agent 跑，登记成一张表，只写这一份：kb-scribe 37 个、gate-triage 15 个、experiment-runner 13 个、crash-verifier 7 个、implementation-writer 6 个、three-way-materials 与 sweep 各 2 个、prior-art 与 mutation-triage 各 1 个（有 12 个阶段归两个 agent、2 个阶段归三个 agent） | `.claude/gate.d/stage-owners.tsv` |」 |
| F4 | records/2026-09-16-subagent拆分提案.md:309 | 不相干 | 这一行讲门禁 62 号核什么：「新门禁阶段核表：目录里每个阶段在表里恰好一行、表里的阶段文件都在、agent 名都有定义、三列齐全」，是机制描述不是计数。读 `.claude/gate.d/62-stage-owners.sh` 判据 ①–④ 核过，逐字对得上，门禁 11 号新增之后 62 号照样这样判（它本来就是防「加了阶段忘登记」的那道闸），不因为新加一个阶段而改变自己检查什么。 |
| F4 | records/2026-09-16-subagent拆分提案.md:535 | 事件句不改 | 「`.claude/gate.d/stage-owners.tsv` 里 15、87 号从 `experiment-runner` 改归 `gate-triage`」记的是 2026-09-17 那次改派动作，读当前 stage-owners.tsv 第 6、48 行核过：`15-research-build.sh\tgate-triage\t…`、`87-replay.sh\tgate-triage\t…`，两个阶段今天仍归 gate-triage，事件描述与今天的现状一致，是事件句、不是待更新的旧说法。 |
| F4 | records/2026-09-17-CLAUDE.md去冗余.md:18 | 不相干 | 这一行是「删了什么、去哪看」表的一格，讲的是「新加阶段要写 `# gate-stage:` 由共享 `gate.sh` 自己的注释说明，漏登记归属表由 62 号拦」这条通用机制指向哪里，不是「表里现在有几条」的计数；读 `.claude/gate.d/11-batch-scope.sh:2` 核过它确有 `# gate-stage:` 行，且已登记进 stage-owners.tsv（62 号会拦漏登记），机制描述依旧成立，不因新增 11 号而改。 |
| F4 | records/2026-09-17-已分配口径三方与两个实验.md:163 | 事件句不改 | 与 535 行同一件事的另一处记录：「`.claude/gate.d/stage-owners.tsv` 里 15、87 号从 `experiment-runner` 改归 `gate-triage`，执行员第 6 步跟着改」，记的是 2026-09-17 那次改派与随之而来的执行员定义改动，现查 stage-owners.tsv 与 `.claude/agents/experiment-runner.md` 第 6 步仍是这个安排，是事件句、内容至今仍真。 |

## 反向核对（step 10）

逐条核这一阶段「做成的事」在自己的载体里有没有记着，都已有（不新增 M 行）：

1. C472 还清、新立门禁 11 号 —— `.claude/kb/checks-owed.md:435`（已还清表），日期 2026-09-22。
2. 11 号基准取 HEAD 不取 `GATE_BASE` —— 同上一行正文，以及 `.claude/gate.d/11-batch-scope.sh:10-15` 头部注释。
3. 触发文件清单从 68 号内嵌 python 挪到 `.claude/gate.d/knowledge-sync-triggers.tsv` —— 同上一行正文，`.claude/gate.d/68-knowledge-sync.sh:9-11`、`.claude/gate.d/11-batch-scope.sh:18-19` 判据 ① 都已改指同一份表。
4. `.claude/gate.d/stage-owners.tsv` 给 11 号登记归 `gate-triage` —— 该表第 5 行本身就是权威登记位：``11-batch-scope.sh\tgate-triage\t这一批的触发文件有没有登记进 .claude/batch-scope……``。

不搜 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改）。

## 校验

```
$ python3 research/scripts/stale-candidates.py --check-report /tmp/claude-1000/c472-batch-scope-sweep/candidates.tsv /tmp/claude-1000/c472-batch-scope-sweep/judge-all.md --groups F1,F2,F3,F4
  ✓ 报告判全了：17 行候选都有逐行判定
```
退出码 0，一次通过，未补跑。

## 没做什么

- 只判了分给我的 F1–F4（17 行全部）；候选表本身只罩事实表写到的三件事实，没有另起搜索。
- 没改任何文件、没改 `.claude/kb/checks-owed.md`、`records/`、`.claude/agent-common.md` 或 `.claude/gate.d/stage-owners.tsv`——F4 那一行「要改」只给出改后的句子，落地交回主 agent 或 `kb-scribe`。
- 没有跑 `gate.sh`、没有编译；只读了 `.claude/gate.d/11-batch-scope.sh`、`68-knowledge-sync.sh`、`62-stage-owners.sh` 的正文与 `stage-owners.tsv`、`.claude/batch-scope`、fixtures 目录清单，未执行门禁脚本本身。
- `.claude/kb/experiments/155-…md:28` 那一行引用的门禁 65 号在这棵树里不存在（`git ls-tree` 零命中），只作为「这是历史快照」的佐证写进理由，没有回扫这处编号断号是否另有欠账——那不在这一阶段的三件事实范围内。

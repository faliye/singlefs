# landing-audit-r7 云端正推腿（Sonnet）报告：K1 每处改法做没做到它那一行

日期：2026-10-24。腿：three-way-forward。范围：拆分提案第九十节 ①–⑤、⑩、㉕–㉗、㉙、㉚、㉝–㉟ 共 14 行（`records/2026-10-23-subagent拆分提案.md` 第 914–948 行里的这 14 行）。
被判文件此刻的 sha256 与开工快照 `research/prompts/landing-audit-r7-snapshot/judged-sha256.txt` 80 份逐份相同（`sha256sum -c` 无一行非 OK）。方案按 `crates/` 与 `scripts/` 今天的实现谈；这一批没有改 `crates/`。

## 零、读法与复跑

- 没写模型目录：这一格是逐行对改法，不是推演。我造的输入与小工具都放在草稿目录 `~/.local/share/singlefs/work/landing-audit-r7-sonnet/`（`tools/`、`prompts/`、`out/`），不进仓。仓副本（`repo/`、`head/`、`sess/`、`ps/`、`snaptree-r7/`、`proj/` 等）用完交回前删，删之前 `du -sh` 的大小写在「没做什么」。要复跑：照该目录 `tools/` 下各脚本文件头的路径，把 `.claude/hooks`、`scripts/agents`、`scripts/kb`、`.claude/scripts` 拷进新建的副本，`.claude/singlefs-ai-sop` 做符号链接指回主工作区，再跑下面各节点名的命令。
- 小工具 sha256（前 16 位）：`hg.py` fd32faea1ff04040、`t_handback.py` 30b3b7cee766586e、`t_naming.py` f5da55584cfd9230、`t_naming2.py` 04712f203c727a9f、`prereg_parallelism.py` 7a4394d06e85164e（从 `.claude/gate.d/doc-experiments.sh` 第 2562–2763 行原样抽出的 python）、`deferred.py` 4dcd2267aae240c0（从 `.claude/gate.d/doc-process-records.sh` 第 1911–2170 行抽出）、`switch_compare.py` 0853138f4d87aafa、`switch_compare3.py` 95357eac3f79126e、`replay_compare.py` d6090dd106a62473。
- 门禁阶段（`doc-experiments.sh`、`doc-process-records.sh`、`doc-registries.sh`、`code-tooling.sh`、`stage-cells.sh`）本身子 agent 起不了：我试过 `bash .claude/gate.d/doc-experiments.sh --check preregistration-parallelism`，被 `heavy-test-guard.sh` 当「提交时才跑的检查」拒（三方正推腿不跑）。所以这几格我只能把它们里面的 python 原样抽出来喂造出的输入，或读代码；整格在真仓上的读数一律写「复核不了」。
- 我没做的复核（主 agent 的读数，我没复算）：看门狗 ⑭ 在真进程上约 120 次报告都是 0 条；门禁 break-switch-replay 格整格；交回闸、派发闸「3195 份 / 3394 份 / 3484 份」那几个全语料数（我只在 3509 份记录上抽了几条，见各节）。

## 一、总表（逐行结论在后面各节，末尾有判定一览）

K1 共 14 行：做到 12 行（①③④⑩ 判不做的依据成立、㉕㉖㉗㉙㉚㉝㉞㉟，其中 ③ 与 ㉟ 带非致命偏差），一半 2 行（② 少读了第六节；⑤ 做到的六条成立，但「不做」六条里 B8 一半、D3 不成立、A5 一半、C4 弱，只有 B9、E2 站得住）。
第九十节去向列写「做了」的，和交回报告、文件此刻对得上的占绝大多数；对不上的有三处：㉟ 行「五份慢自证由任务 53 到 56 补齐」里任务 53 是 E171 编排（不是慢自证）、调度记录第 366 行把「撞到上限的读数」写成「36G 够用」、`b12-handback.md` 第 64 行把 `a9dbfee4c29c6da98` 说成只写实验页的（它的记录里 Write 了 `scripts/experiments/e185_r1_compare.py`）。逐条在各节。

## 二、① 执行员登记落地清单（交回闸 ⑦）——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:914`。形态列要求：执行员交回时报告里「登记落地清单」一节在、每行的文件:行与用例名在仓副本里找得到。
- 改法：`.claude/hooks/handback-guard.sh:444`（`LANDING_SECTION = "登记落地清单"`）、`:1494`（`landing_applies`，判这个执行员这一段写没写装置、落没落产物）、`:1524`（`landing_problems`，判一节在不在、文件:行指不指得到、用例名 grep 不 grep 得到）；规则文字 `.claude/hooks/handback-guard.sh:64-70`，共用约束 `.claude/agent-common.md:94`。
- 依据（草稿 `tools/t_handback.py`，样本仓 `proj/`，跑的是主工作区钩子的拷贝、走真实 stdin 入口）：执行员 Write 了 `scripts/experiment-apparatus/e999_x/run.py` 之后交回——无清单 rc=2；清单行号写 99（文件只有 3 行）rc=2，拒绝说明「只有 3 行」；用例名写不存在的 rc=2，说明「grep 不到」；文件写不存在的 rc=2；节下面一行表都没有（只写「无。」）rc=2；全对 rc=0。装置只在 Bash 里 `cat > scripts/experiments/e999_y.sh <<EOF` 写的、只往 `research/results/` 里 `echo > ` 写了东西的，无清单都 rc=2；回扫员（`sweep`）同样交回不判，rc=0。
- 钩子文件头自己写明认不出的两种我也核了：Bash 里 `cp` 把装置挪进来、`python3 -c "open(…,'w')"` 写装置，都 rc=0（`.claude/hooks/handback-guard.sh:70`「只把装置挪进主工作区（cp / mv）、只写实验页的不判」）。
- 真会话记录（去向列「真记录上落地之后的执行员新拒 0」）：`tools/replay_compare_ts.py` 取 `~/.claude/projects/…/subagents/` 下有 SubagentHandback 调用的 experiment-runner，用 HEAD 里的旧钩子（`git archive HEAD` 拷出）与现钩子各判一遍；自 2026-10-22 11:34:14（交回报告说的 `a427eccd6824832fb` 开工时刻）以后 13 份：同 13、新拒 0、新放 0。自 2026-10-21 起 31 份里只有 `a9dbfee4c29c6da98`（2026-10-22 05:28 开工，在落地之前）被 ⑦ 拒。去向列的结论成立。
- 偏差（不致命）：① `research/prompts/subagent-landing-audit-2026-10-22/b12-handback.md:64` 把 `a9dbfee4c29c6da98` 列在「按第 3c 步本来就不用交清单」的 4 份里，我读它的会话记录，它 Write 了 `scripts/experiments/e185_r1_compare.py`、Edit 了若干次，按 `.claude/hooks/handback-guard.sh:454` 的装置路径形态本来就在 ⑦ 的射程内；它没被算进落地之后的新拒只是因为开工比落地早 6 小时，说明写错了，结论没错。② 豁免用子串：`.claude/hooks/handback-guard.sh:1537` 在整份派发提示里找「只补落产物」「只修锚点」，我造的提示正文写「这一轮不是只补落产物，要跑整表」，交回无清单 rc=0（误放）。派发闸 ⑪ 放行的两种写在提示的固定行里，交回闸却在正文任何位置找，是两处判同一件事而位置不同。
- 推翻条件：同一份 `t_handback.py` 的任一 rc=2 格变成 0，或自 2026-10-22T11:34:14 以后的执行员交回里出现新拒。

## 三、② preregistration-parallelism 加挂钟算式一判——一半

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:915`，形态列要求「第十四节的预计挂钟与第六节写的态数乘单态耗时对得上（差一个数量级就红）」（`records/2026-10-23-subagent拆分提案.md:915`）。
- 改法：`.claude/gate.d/doc-experiments.sh:2524`（格头判据）起到 `:2763`；挂钟算式读法 `:2532-2543`，格登记 `:257`。
- 依据：抽出 python（`tools/prereg_parallelism.py`）在草稿仓里喂 6 份造的登记（`tools/mkprereg.sh` 写，第六节写 10000000 个态、单态 2 秒）：式子「10000000 个态 × 2 秒 ÷ 31 ≈ 5 分钟」红，说明「算出约 179 小时，写的约 5 分钟」；「… ≈ 3 小时」红；「… ≈ 7.5 天」绿；写散文「约 3 分钟（推的：第六节的态数乘单态耗时再按 31 线程切）」与「每态 2 秒，共 10000000 个态，按 31 线程并行，约 5 分钟」两份都不红，只打「! 挂钟算式本次未判 2 份」并往 `GATE_NOT_RUN_FILE` 记一行；真仓 E186、E187、E197 三份喂进去 rc=0，「挂钟算式判了 3 条（3 份）」，与去向列「各判 1 条」一致。
- 没做到的那一半：形态列要核的是第六节的态数。实现只读「预计挂钟：」那一行里自己写的态数与单态耗时（`.claude/gate.d/doc-experiments.sh:2543`「态数与单态耗时只读「预计挂钟：」那一行，写在第六节或别的行、这一行里没重写的不读」，文件头自己写了这条限制）。我造的第六节写 1e7 个态、第十四节那行写「100 个态 × 2 秒 ≈ 3 分钟」，这格绿。设计员定义要求核出来的数同时写进第六节与第十四节（`.claude/agents/experiment-designer.md:33`「登记写死之前核可跑性与判别力，核出来的数写进第六节那个量的行与第十四节」），门禁只盯了其中一处，两处写得不一样不红。
- 另：并行不写除数（「按 31 线程并行约 5 分钟」）按串行乘积比，一个数量级的余量盖不住时误红；散文写法不红。两条文件头都写了（`.claude/gate.d/doc-experiments.sh:2543`之后几行），不是漏。
- 复核不了：整格在真仓 HEAD 到工作区之间的读数（门禁阶段被拒）。
- 推翻条件：在 `tools/prereg_parallelism.py` 里第六节与第十四节态数不一致的登记（上面第三份）变红，或「本次未判」那两份被改成红。

## 四、③ 延后项登记（门禁 doc-process-records 第十格 deferred-items-registered）——做到，带一处越出行文

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:916`。形态列：records/ 新增的带「延后」「下轮开」「下一批」这类词的行，同一行写着 C 号、拆分提案节号或文件:行且指得到。
- 改法：`.claude/gate.d/doc-process-records.sh:1902`（`check_deferred_items_registered`）、格登记 `:484`、词表 `:522`（`DEFERRAL_WORDS="延后 下轮开 下一轮开 下一批 留给 下一个题目"`）、判据说明 `:257-285`；登记处表 `.claude/gate.d/deferral-registries.tsv:4-5`（`staged-gate.md`、`commit-stages.md` 两行）；格的归属 `.claude/gate.d/stage-owners.tsv:117`。
- 依据：抽出 python（`tools/deferred.py`）对真仓的 `.claude/kb/checks-owed.md` 与两份拆分提案喂 14 行造的新增行（`out/deferred-in.txt`）。红的五行：「- 延后：以后再看」（没写）、「延后 C99999999」（C 号指不到）、「把门禁留给用户」（词表里的「留给」，没写登记）、「延后，见拆分提案第九十节 ㊿」（九十节里没有 ㊿）、「延后，登记见 .claude/kb/checks-owed.md:999999」（文件只有 898 行）；汇总「5 行没登记去向（判了 10 行）」，rc=1。绿的：写 C879、「九十节 ⑤」、`records/2026-10-22-子agent结果落地审计.md:326`、表格行去向格点名 `.claude/skills/gate/staged-gate.md`、词只在「」里、「延后项无去向 3/54」统计数。
- 管不到的（文件头 `:257-285` 写明）我也核了：「推迟到下周再说」「下轮再做这件事」不在词表里，不判。
- 越出行文一处：登记处表认流程文件算登记（第四种），而 `.claude/main-agent.md:21` 第 2 条列的登记处是里程碑收口表、`.claude/kb/checks-owed.md`、决策正文、拆分提案编号小节，没有「提交时一定会跑的流程文件」。第九十节 ③ 去向列已写「任务 47 收窄（……登记处表 `.claude/gate.d/deferral-registries.tsv`）」，所以不是漏写；这是门禁认得比规则条文宽的一处，进 K2 的「是不是同一组登记处」。
- 复核不了：整格在真仓上红 107 行（任务 41）、96 行（任务 47）与「本记录 14 行」这几个读数（门禁阶段被拒；`out/deferred-in.txt` 之外我没取 HEAD 到工作区的新增行）。
- 推翻条件：`tools/deferred.py` 上面五个红行有一行变绿，或绿的有一行变红。

## 五、④ 回扫判出的 crates/ 行要进「交实现员」一节——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:917`。形态列：`scripts/kb/stale-candidates.py --check-report`，目标在 `crates/`（不变量清单的 `.md` 除外）判「要改」「要补」的行不在「交实现员」一节里就退非 0。
- 改法：`scripts/kb/stale-candidates.py:564`（`handover_scan`）、`:597`（`handover_missing`）、`:610`（`check_reports`，退 3），节标题常量 `:125`，表格行才收（`:587`）。
- 依据：候选表 3 行（`crates/foo/src/lib.rs` 第 12 行、一条 `.claude/kb/a.md` 第 3 行、一条不变量清单 `INV-1.md` 第 5 行）加 7 份造的报告，`python3 scripts/kb/stale-candidates.py --check-report`：判「要改」且没有交实现员一节 rc=3，「1 行目标在 crates/ 下、判了「要改」或「要补」，不在「交实现员」一节里」；节里写同一个 `crates/foo/src/lib.rs` 第 12 行 rc=0；节里只写第 123 行（行号前缀相同）rc=3；节里只写「0 行（grep -c 0）」rc=3；判「事件句不改」不要求，rc=0；节标题低一级（`####`）照认，rc=0；节在围栏里的 rc=3；不变量清单 `.md` 那一行判「要改」不要求。`--selftest` rc=0。
- 去向列「仓里 68 份历史回扫报告 0 误报」：我取 `research/prompts/*.md` 与 `records/*.md` 里含「交实现员」或「## 逐行判定」的 71 份跑 `handover_scan`，找到 crates 下要改要补的表格行 0 份。0 误报成立；0 真阳性也如交回报告说的「没有真阳性可对」，所以「现有格式里真有这种行时能红」只由造的输入证，没有历史样本证。
- 推翻条件：造的 `r-none` 报告（crates 要改、没有交实现员）不再退 3。

## 六、⑤ 常设要求做成「骨架打一行、派发闸核一行」——做到六条，「不做」六条只有两条站得住

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:918`。形态列：候选 `scripts/agents/dispatch-prompt.py` 按类型打常设要求、派发闸核那几行在不在。
- 改法：骨架 `scripts/agents/dispatch-prompt.py:338`（`REUSE_LINE`）；派发闸 `.claude/hooks/runner-dispatch-guard.sh:2245`（复用、分片、续跑不短于 8 个字）、`:2324`（批次）、`:2351`（草稿目录落点）、`:2378` 与 `:2402`（写死台数与额度）、`:2415`（自证在哪跑）、`:2431`（三方腿行首「报告：」）；报告路径的认法在 `.claude/hooks/lib_three_way_legs.py:151`、`:166`、`:171`；共用约束 `.claude/agent-common.md:90`。
- 依据：`python3 scripts/agents/dispatch-prompt.py --check <提示> --type three-way-forward --root <仓根>`（走派发闸自己的 `--check-prompt`），24 份造的提示（`prompts/`，`prompts/base.txt` 是放行的底稿）：超过 2 小时没写「复用：」rc=3、复用只写「复用：无判定」rc=3、复用是占位 rc=3、复用写两行 rc=3、「分片：」「续跑：」各只写 4 个字 rc=3；草稿目录写 `/tmp/claude-1000/…`（预计挂钟 2 小时内也一样）rc=3、写相对路径 rc=3；批次缺 rc=3、点名的调度记录不存在 rc=3、批号空 rc=3；正文写「本机 31 线程、第二台 31 线程，内存两台合计 80G」rc=3、「每台 16 线程」rc=3、「每台 40G，两台合计 80G」rc=3；「本机用 8 核跑」「起 12 线程跑完」、「」里转述的原话、围栏里的 rc=0（`.claude/hooks/runner-dispatch-guard.sh:787` 的样本名「写死台数:单写一台的线程数与核数的硬件说明不判」写明）；三方腿没有行首「报告：」rc=3、句中写「报告写进 …」rc=3、禁读清单点到本腿自己的 `-output.md` rc=3、报告行指到草稿目录的 `result.md` rc=3、名字里带 report rc=3。「自证在哪跑：」指到工作根下的仓副本 rc=3、指到旧根 rc=3、写「副本里那一份」不判（文件头已写）。我自己这份派发提示照骨架写，`--check` rc=0。
- 真提示：取 E171 #21 原句的 `a51808a134923474f`（`prompts/e171-21.txt`）喂 `bash .claude/hooks/runner-dispatch-guard.sh --check-prompt … --type experiment-runner`，拒 4 处，其中一处原话「正文写死了主机资源的数：「本机 31 线程、第二台 31 线程」、「两台合计 80G」」。自证：派发闸 609 种、骨架 110 项，都退 0（`out/dispatch-selftest.txt`、`out/dp-selftest.txt`）。3484 份真提示「新放 0、新拒 1545」我没复算。
- B4 的代价（「N 核」与单写一台的线程数不判）：交回报告 `b12-standing-checks.md` 第 3 条写的依据是真提示里这两种撞到的都是观测与硬件说明（宽认法 2026-10-14 起 38 份、整个语料 129 份），是量过的；「以后也不会有人用它们写死额度」是推的，这一批没有把它写成量的，只写了「代价是」。我复算一个方向：`bash …runner-dispatch-guard.sh --check-prompt` 对「本机用 8 核跑」rc=0。
- 「不做」六条（`records/2026-10-23-subagent拆分提案.md:918`「不做：B8（只有执行员有续做判据，已由派发闸续派那一判核）」起到 A5），依据逐条现查：
  - B9（内存上限）——站得住。`scripts/experiment-apparatus/replay.sh:92` 缺省 `REPLAY_MEMORY_CAP:-8G`，共用约束 `.claude/agent-common.md:60` 写「都没给就取 `scripts/experiment-apparatus/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值」，没给上限不会没有上限。
  - E2（弄坏开关名）——站得住。`.claude/agents/tooling-writer.md:23` 写「主 agent 给不出的，写明由你起名」，第 31 行（第 2 步）要它先造该红的输入。调度记录说「第 2 步要它自己起名」，起名那句在输入一节，不在第 2 步；结论不变。
  - B8（续派指到前一份报告）——一半。依据说「已由派发闸续派那一判核」，核的是 `.claude/hooks/runner-dispatch-guard.sh:2477` 的「上一段岔路表里还差」；要求本身（`research/prompts/subagent-landing-audit-2026-10-22/b12-standing-requirements.md:60`）是派发提示指到前一份报告与这次的输出路径。派发闸自证里放行的续做提示（`.claude/hooks/runner-dispatch-guard.sh:2869` 的 `again` 只写了「跑前登记：…」一行，`:2944` 那格放行）没有前一份报告的路径，所以这一判核的不是被「不做」的那件事。别的类型没有续做判据、没有便宜的判法，这一半成立。
  - D3（不碰的文件）——依据不成立（对象不同）。依据说「撞文件由 `apply-writer-patch.py` 与写范围表拦」；`scripts/gate/apply-writer-patch.py:38` 那一套拦的是未写判决的三方轮开工快照里的文件（`lib_snapshot_conflicts.py`），写范围表 `.claude/hooks/agent-write-scope.tsv` 是按类型固定的路径模式（工具员能写整个 `.claude/hooks/**`、`scripts/**`）。两样都不知道这一轮别的会话正在改哪些文件（`research/prompts/subagent-landing-audit-2026-10-22/b12-standing-requirements.md:79`「这一轮别的会话正在改的」）。不做可以，依据要换成「没有便宜的判法」，不是「已被拦」。本批自己的对象：`.claude/hooks/heavy-test-guard.sh` 文件头别的会话同意由任务 43 改（调度记录第 339 行）就是靠主 agent 手工协商。
  - C4（先装置后分段的派法）——弱。「是派发的次序，不在任何一份提示里，派发闸看不到」：派发闸 ⑦ 本来就读主会话记录判在跑的子 agent（`.claude/hooks/runner-dispatch-guard.sh:2691`），原理上看得到次序；没有便宜的判法这一点成立。
  - A5（「关的是哪几条」）——一半。「判不了内容」成立；但「节在不在」判得了：我把骨架的「## 关的是哪几条」整节删掉（`prompts/a5-nosection.txt`）、或写成「随便。」（`prompts/a5-empty.txt`），`--check` 都 rc=0。要么在依据里写「内容判不了、节在不在也不判」，要么补那一半。
- 推翻条件：D3 与 B8 的判语要变，需要在派发闸或 `apply-writer-patch.py` 里找出一处真会读「别的会话在改的文件」或「前一份报告路径」的代码（我 grep 到的没有）。

## 七、⑩ 上游 gate-overlap 认 hook-swap.py 与 mv——判不做的依据成立

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:923`。判不做的依据三句：上游查重脚本按会话记录认写法那一支只给没注册的收工钩子用；提交前门禁阶段「门禁查重」按 diff 窗口判；`hook-swap.py` 换上的改动一样在 diff 里。
- 依据：本仓没注册它——`grep -c 'gate-reuse-check' .claude/settings.json` 为 0；`.claude/hooks/completion-cleanup-guard.sh:8` 写「gate-similar: gate-reuse-check.sh 上游的 Stop / SubagentStop 收工自检，……本仓没注册」；上游 `.claude/singlefs-ai-sop/scripts/gate-overlap.py:25`「认不出的写法：路径拼在变量里、python 或别的解释器里写文件。这些由门禁阶段兜底。」；门禁阶段「门禁查重」`.claude/singlefs-ai-sop/scripts/gate.sh:433`（`run_stage_may_skip "门禁查重"`），按 `GATE_DIFF_BASE` 相对基准算新加与改动（`.claude/singlefs-ai-sop/scripts/gate-overlap.py:14-17` 的两种算法）；`gate-overlap.py` 里没有 `hook-swap` 这个词。
- 没核到的：用户级 `~/.claude/settings.json` 里有没有注册这个收工钩子（子 agent 不碰 `~/.claude/`）。
- 推翻条件：项目或用户级 settings 里出现 `gate-reuse-check` 的注册；或 `hook-swap.py` 换上的钩子在提交前 diff 里查不到（不会，换的是仓内文件）。

## 八、㉕ 交回闸认 Bash 写的 .rs、检出钩子认 mktemp 的 -p / --tmpdir / 套一层——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:938`。
- 改法（交回闸）：`.claude/hooks/handback-guard.sh:33-39`（⑤ 判法文字：重定向 `>`、`>>`、`>|`、`&>`、tee、cp / mv 的目标，`-t` 照认，根写成 `$(git rev-parse --show-toplevel)` 按命令当时的目录往上找第一层 `.git`）、`:433`（`NAMING_GIT_TOPLEVEL_ROOT`）、`:809`（`git_toplevel`）、`:898-899`；共用约束 `.claude/agent-common.md:94`、`:99`。
- 依据（`tools/t_naming.py`，工具员交回，样本仓 `proj/` 带 `.git` 目录）：`mv` 到 `crates/core/src/lib.rs`、`cp` 进目录、`cat >` 与 heredoc、`>>`、`tee`、`tee -a`、`cp -t`、`&>`、`&>>`、写 `scripts/experiment-apparatus/e1_probe.rs`，都 rc=2；写到草稿目录里的 `.rs`、`echo "重定向 > crates/… 不会发生"`（引号里）、`grep … > /tmp/out.txt`、`cargo check 2>&1 | head` 都 rc=0；改完之后两条命名检查的根写成 `"$(git rev-parse --show-toplevel)"` 跑完，rc=0，没跑 rc=2。
- 行外未判、不算欠这一行：目标本身写成 `mv /tmp/x.rs $(git rev-parse --show-toplevel)/crates/core/src/lib.rs`（落点算不出）、`dd of=`、`rsync`、`install`、`sed -i`、python 写，都 rc=0。文件头第 56 行写「认不出的：Bash 里 cp -r 整个目录、install、sed -i、python 写的 .rs」，前一种目标写法没写进去。
- 依据（检出钩子）：`bash .claude/hooks/bash-command-detector.sh --selftest` rc=0「查了 799 种」（52 秒）；`mktemp -d -p`、`--tmpdir=`、不带值的 `--tmpdir`、套在 `$(cd $(mktemp …) && pwd -P)` 里的各有放行与拒绝格（`.claude/hooks/bash-command-detector.sh:3413` 到 `:3433`，拒的有：指到别的子 agent 认领的目录、别的会话的目录、`/tmp`、仓内目录、没有 agent_id、套一层而 cd 之后接的不是 pwd）；实现 `:1666`（`mktemp_inside_pwd`）、`:1678`（`mktemp_option_directory`），文件头 `:223-227`。
- `lib_writer_lock.py` 文件头改三处（`.claude/hooks/lib_writer_lock.py:3`「三处用这一份：」）：`grep -rlF lib_writer_lock .claude/hooks scripts` 的调用方是 `write-guard.sh`、`bash-command-detector.sh`、`runner-dispatch-guard.sh` 三份，对上。
- 推翻条件：`t_naming.py` 的 rc=2 格有一格变 0，或 `lib_writer_lock` 多出第四个调用方而文件头没改。

## 九、㉖ 门禁尾巴（doc-registries 另四格、两份文件头、不登记的比对对象）——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:939`。(d) 那一件按 landing-audit-r6 判决删掉，(e) 主 agent 改判成「每行写一个开着的欠账号」。
- (a) 另四格读不出登记位判红：`.claude/gate.d/doc-registries.sh:3314`（`kb_roots_or_red`）被六格共用——audit-contradictions `:2719`、declared-counts `:3404`、invariant-count-elsewhere `:3514`、invariant-anchors `:3623`、inline-sums `:3742`、invariant-prerequisites `:3886`，另四格都在。读登记位的函数 `scripts/kb/quote-kb.py:481`（`kb_candidate_roots`）：我给草稿仓的 `.claude/kb-directories` 写 `\xff\xfe.claude/kb`，原样抽出的那段 python 抛 `UnicodeDecodeError`（落在 `scripts/kb/quote-kb.py:445` 的 `kb_directory_entries` 里），rc=1，`kb_roots_or_red` 把它变成「没判：kb 目录登记位读不出」的红。只验到函数这一层，整格被拒，样本档 55 个用例我没复跑。
- (b) 两份文件头：`.claude/gate.d/lib-lag-baseline.py:15` 起「调它的各处」；我用它自己写的命令 `grep -rln lib-lag-baseline .claude/gate.d .claude/hooks scripts`（去掉 fixtures）数出的 19 份里，真调它的都在头里点名，只多出 `heavy-test-guard.sh` 与 `lib_heavy_tests.py` 两份——它们只在注释里提到库名（`.claude/hooks/heavy-test-guard.sh:95`），不是调用方。`.claude/hooks/heavy-test-guard.sh:95-96` 的自证一段（「lib- 起头的共用库」同脚本一样只认单独一个 --selftest）补上了。
- (c) `.claude/gate.d/lib-forbidden-notations.py` 没有 `--selftest` 入口（文件里只在注释里提到别的脚本的自证），删掉成立。
- (e) 不登记的比对对象：`.claude/gate.d/replay-comparison-units.tsv:19-37` 共 19 行「# 不登记的比对对象：」，`grep -vc '欠在 C[0-9]*'` 得 0，用到的 C879、C895、C901 在 `.claude/kb/checks-owed.md` 里分别是第 558、574、580 行的开着的账；判法 `.claude/gate.d/doc-experiments.sh:2793`「点名不登记的每一行，理由里要写「欠在 C<号>（简称）」，号要在 .claude/kb/checks-owed.md 开着的那张表里」。这一格我没法跑，读代码与表。
- 推翻条件：给 `.claude/kb-directories` 写坏字节，六格里有一格不红；或 `replay-comparison-units.tsv` 里有一行不登记的行没有欠账号。

## 十、㉗ 派发闸口子（三方形态、名里带 report、没有草稿目录行、开关点名、三种认法）——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:940`；形态列：三方形态只认带报告标签或落在本腿自己轮名下的 `-output.md`；三方路径的名里带 report 照判；三处各加样本与弄坏开关。
- 改法：`.claude/hooks/runner-dispatch-guard.sh:2431`（`leg_report_line_verdict`）、`:2245` 与 `:2351` 一带；三种认法统一进 `.claude/hooks/lib_three_way_legs.py:151`（`leg_report_mentions_in_trees`）、`:166`、`:171`（`leg_report_written_tree`）；样本钉在 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`。
- 依据：`python3 scripts/agents/dispatch-prompt.py --check <钉住的样本> --type three-way-attack`：`a75e9e30847f57bcd-forbidden-own.txt`（landing-audit-r4 攻方头一版，禁读清单写成本腿自己的报告）rc=3，说明「禁读清单里点到了本腿自己的报告：「research/prompts/landing-audit-r4-opus-output.md」」；同一份没改禁读行的 `a75e9e30847f57bcd.txt` rc=0；`a3652270eb80e4fd0.txt`（句中写「。报告：…」）rc=3「没有行首「报告：」一行」；`a4737c6f3660f4f7e-own-result.txt` 与 `aa9ec3423c1f54bba-own-result.txt` 报告行点名草稿目录下的 `result.md` rc=3。三方名里带 report：`x-report-output.md` rc=3。(iii) 没有草稿目录一行：`report.md` 裸文件名 rc=3，`docs/report.md` 带目录不判（`prompts/nr-bare.txt`、`prompts/nr-dir.txt`），与文件头写的「只判裸文件名」一致。派发闸 `--selftest` 609 种 rc=0。
- 开关点名：我按门禁 break-switch-replay 格文件头 `.claude/gate.d/code-tooling.sh:4425-4435` 写的规则（点名的每一项要照出一条失败行、每条 ✗ 起头的失败行要被某项照出，基准趟减掉，汇总行除外）自写了一份仿判（`tools/switch_cell.py`，不是那一格本身），对派发闸 120 条开关跑：全相同，只有一条（`writer-lock-fence-ambiguity-unchecked`）因为我的解析把后面一段说明当成点名项而显示不同，对着 4 条点名与 4 条实际红的行逐条看是相同的。
- 推翻条件：钉住的 `a75e9e30847f57bcd-forbidden-own.txt` 不再红，或 `leg_report_written_tree` 与交回闸 ② 对同一写法判得不同。

## 十一、㉙ 禁读清单不许有本腿报告、会话名判法进三份定点写入脚本——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:942`。(a) 见上一节 `a75e9e30847f57bcd-forbidden-own.txt` rc=3。
- (b) 改法：`scripts/kb/lib_notation_check.py:117`（`session_name_refusal`）；`replace-once.py` 退出码 8、`insert-row.py` 退 7、`replace-batch.py` 退 5（各自文件头写的数）。依据（`sess/` 草稿仓，抄了三份脚本、库与 `lib-forbidden-notations.py`，新串里带拼出来的会话名 `singlefs-` 加 `e152`）：`replace-once.py` rc=8、`insert-row.py` rc=7、`replace-batch.py` rc=5，三份都说「新串写了会话号（ListAgents 列的会话名），共 1 处，records/n.md 一个字节没写」、出路写成「别的会话」这类不带号的说法，文件原样；换成「别的会话回话了」`replace-once.py` rc=0 写成。三份脚本的 `--selftest` 都 rc=0。
- 推翻条件：任一份对带会话名的新串退 0，或写了文件。

## 十二、㉚ 另两处「草稿目录」认法改调共用库——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:943`；剩 `dispatch-prompt.py` 与 `agent-cost.py` 两处。
- 改法：`scripts/agents/dispatch-prompt.py:403-404`（导入 `lib_prompt_scratch_line`）与 `:432`（`scratch_line_values`）；`scripts/agents/agent-cost.py:115`（导入）与 `:717-722`。
- ⑯ 收严的影响（形态列要在全部派发提示上量）：我取 `~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/agent-*.jsonl` 3509 份首条提示，`tools/scratch_corpus.py`：共库认出「草稿目录：」的 2209 份；行首带列表记号、引用记号或粗体而共库不认的 63 份，最晚 2026-10-12；行首正好是「草稿目录：」而共库不认的 0 份。交回报告说 84 份，我这边 63 份，语料范围不同（我只有本机此刻留着的 3509 份），「10-14 起 0 份」两边一致。
- 账表两桶：`python3 scripts/agents/agent-cost.py outcome --since 2026-09-01 --summary` 此刻读得 `verdict-archived` 652、`three-way-report-or-verdict-missing` 4（交回报告 647 与 4，语料 3394 份，我这边 3509 份）。
- 推翻条件：两份脚本任一份不经共库认「草稿目录」；或 `scratch_corpus.py` 里「行首正好是草稿目录：而共库不认」不是 0。

## 十三、㉝ 草稿目录库的围栏与相对路径、续做闸平常跑设开关、进程安全扫描没扫的目录——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:946`。
- (a) 围栏照 CommonMark：`.claude/hooks/lib_prompt_scratch_line.py:107`（`fenced_line_numbers`）。喂 14 份造的提示（直接调 `scratch_line_values`）：三个反引号围住的 → 认不到（被遮）；四个反引号围住、里面再有三个反引号的 → 只认围栏外那一行；波浪线围栏 → 遮；反引号围栏里的波浪线不当收尾 → 遮到底；缩进 4 格的三反引号（是代码块不是围栏）→ 认到；缩进 3 格的围栏 → 遮；没收尾的围栏 → 遮到文末（CommonMark 的读法）；信息串里带反引号的开栏（不是围栏）→ 认到；收尾行带信息串的（不是收尾）→ 遮。库自证「查了 39 格」rc=0，含与 markdown-it-py 逐样本比对（`.claude/hooks/lib_prompt_scratch_line.py:161`）。
- (b) 相对路径按仓根：`.claude/hooks/lib_prompt_scratch_line.py:152-153`；`草稿目录：~/.local/share/singlefs/work/x` 在 `/tmp` 与 `crates/` 两个当前目录下调用，都解成 `/home/fy5090/.local/share/singlefs/work/x`。完工清理闸同形那一处：`.claude/hooks/completion-cleanup-guard.sh:379-383` 自己又按仓根解了一遍（文件里注明「与那份库的 scratch_directories 同一种解法」），自证 45 格 rc=0、28 个开关各红在点名的格；这是两份各写一遍同一种解法，不是调库，进 K2。
- (c) 续做闸平常跑设开关就拒：`.claude/hooks/continuation-guard.sh:35`、`:99`。`echo '{"tool_name":"SendMessage",…}' | CONTINUATION_GUARD_BREAK=disable-check bash .claude/hooks/continuation-guard.sh` rc=2，「是弄坏开关，只在自证（continuation-guard.sh --selftest）里认；平常跑设了就拒」；不设开关 rc=0。检出钩子同：`BASH_COMMAND_DETECTOR_BREAK=mktemp-directory-unknown` 平常跑 rc=2。`.claude/gate.d/break-switch-outside-selftest-pending.tsv` 里 `grep -c 'continuation-guard\|handback-guard\|bash-command-detector'` 为 0（两份的行都删了）。
- (d) 进程安全扫描：`.claude/hooks/bash-command-detector.sh:2752`（`scan_scripts`）、`:2792-2795`。草稿仓里 `scripts/b/killer.sh`（循环里 kill）登记进 `.claude/process-safety-pending`，只带 `scripts/a` 跑 `--scan-scripts`：打「! 本次未判（这不是通过）：… scripts/b/killer.sh 所在的目录 scripts/b 这一趟没扫」、成功句带「本次未判」、往 `GATE_NOT_RUN_FILE` 写一行；带 `scripts/a`、`scripts/b` 两个目录：killer.sh 算「没判的」；待改清单点名一份已扫目录里没有要放行写法的 `scripts/a/fine.sh`：红，「里一处要放行的都没有」，rc=1。
- 推翻条件：上述任一格反过来；或续做闸、检出钩子的平常跑设开关退 0。

## 十四、㉞ 交回闸四件、共用约束第 63 / 99 行、验证段、判决对快照倒推出的树跑 cite-check——做到

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:947`。
- 交回闸四件（`tools/t_naming2.py`，tooling-writer 改了 `.rs` 再交回）：落盘再读汇总行的写法 `<命令> > <草稿目录>/<名>.txt 2>&1; echo rc=$?; tail -n 20 …` 两条命名检查各一条 rc=0；`| tail` 接管道 rc=2，说明分「code-source-discipline.sh 那四格 跑了，但认不出它自己的退出码：接了管道…」与「会话记录里改完之后没跑过」两种；输出被存成 persisted-output 时读存下来的那一份找 `rc=`（`rc=0`、`rc=1` 放行，`rc=124` 拒「它自己退了 124（被 timeout 杀掉）」，存下的文件被删拒「输出存成了 persisted-output，存下来的那一份读…」）；`; echo "rc=${PIPESTATUS[0]}"` rc=0 放行、`${PIPESTATUS[1]}`（tail 的）rc=2；`; true; echo rc=0` 这种伪造 rc=2。代码 `.claude/hooks/handback-guard.sh:438`（`PERSISTED_OUTPUT`）、`:441`（`NAMING_FILED_OUTPUT_FORM`）。
- 与共用约束对齐：把 `NAMING_FILED_OUTPUT_FORM` 的串拿去在 `.claude/agent-common.md:63` 与 `:99` 两行里找，两行逐字都含；第 99 行写「接管道（`| tail`、`| head`、`| grep`）、`|| true`、后面用 `;` 接别的命令而没紧跟 `echo rc=$?` 的，交回闸认不出它自己的退出码，不算跑过」，与交回闸判法一致。
- `$SINGLEFS_WORK_ROOT/` 开头的报告路径：`.claude/hooks/handback-guard.sh:27` 的文字与自证里的格（交回闸 `--selftest` 117 种 rc=0，其中三方腿 `$SINGLEFS_WORK_ROOT/` 两格）；我没有另造一棵登记过的 worktree 去直接喂（`git worktree add` 是 git 写操作）。
- 验证段（`.claude/main-agent.md:52`、`:55`、`:56`）：第 1 步命令带 `SINGLEFS_RUN_SCOPE` 与 `SINGLEFS_HEAVY_TESTS=user-request` 前缀、写出快档那一步打出的重跑命令原样跑；第 4 步改成只重跑红的与这一次改到的；第 5 步括注跟着改。`bash scripts/gate/merge-and-verify.sh --selftest` 不经包装 rc=0（29 秒，0 条 ✗）。
- 判决对快照倒推出的树跑：`python3 scripts/agents/snapshot-tree.py landing-audit-r7 --out <草稿>/snaptree-r7`，「清点名 80 份，倒推出 80 份，sha256 对上 80 份（基底取自 HEAD 37 份、diff 头记的 blob 0 份、新文件 43 份）」，0.2 秒；`--selftest` 查了 26 项 rc=0；`.claude/rules/three-way-inference.md:125` 第 1 步照这条写，`.claude/rules/path-moves.md:25`（第 6 步「判决或核对表里记过的 sha256 会因此对不上」）说的搬迁换回由 `snapshot-tree.py` 的 `MovesBack`（`scripts/agents/snapshot-tree.py:89`）做。旧轮倒推不全（r1–r3 各留 9–10 份）调度记录已写。
- 推翻条件：`t_naming2.py` 里 `pipe-tail`、`pipestatus1-tail`、`fake-echo`、`persisted-124`、`persisted-gone` 有一格变 rc=0，或 `filed-ok`、`persisted-ok`、`pipestatus0` 变 rc=2；或 `snapshot-tree.py` 对 landing-audit-r7 不是 80/80。

## 十五、㉟ 最后一批延后项——做到，三处偏差

- 第九十节行：`records/2026-10-23-subagent拆分提案.md:948`。逐件：
  - 看门狗 ⑭ 在真进程上量：主 agent 量的（调度记录第 348 行，约 120 次都报 0 条）——复核不了（要起看门狗盯真进程）。自证 `watch.sh --selftest` rc=0；遗留的 `agent-watch-selftest-scratch-*` 目录此刻 `ls` 不存在；`python3 scripts/agents/agent-watch.py --help` rc=0（原来抛 `ValueError`）；被杀遗留目录开跑先删在 `scripts/agents/agent-watch.py:5514`（`remove_stale_selftest_scratch`）。
  - 合入后验证新那一步：`--checker-quick-memory` 做到（`scripts/gate/merge-and-verify.sh:9`、`:117-121`、`:277`）；`selftest-registry.tsv` 里 merge-and-verify 一行的理由补全（`.claude/hooks/selftest-registry.tsv:112`）。**偏差一**：缺省 36G 是推的，文件头 `scripts/gate/merge-and-verify.sh:121`「没有按 36G 跑过」写明，没有当成量写，这一点好；但调度记录第 366 行写「任务 50 定的缺省 36G 够用，24G 那一次不够」，依据是峰值表记 `38654705664`（36.0 GiB），正好等于那一趟的上限（`36 × 2^30`），与 24G 那次撞顶的读数同一种形态；本机峰值表（`~/.local/share/singlefs/memory-peaks.tsv`）第 7787 行，同样是 checker-tier-crash-point-replay.sh 的 `--check crash-case-replay,feature-gated-tests` 那条命令，近几次是 `25769803776:hit,37485543424:clean,38654705664:clean`——34.9 GiB 与 36.0 GiB 两次没被杀，说明 36G 勉强够，不是量出了需要多少。文件头第 121 行「没有按 36G 跑过」也已经过时（第 366 行记了跑过一趟）。**偏差二**：这一步的 `--selftest` 不经包装 rc=0，经 `run-with-memory-cap.sh 4G` 包着跑 rc=1、两格红（「内存上限 峰值表里这一步近几次最大 10 GiB（其中一次撞了顶），没给上限时应当取 15G」收到「36G」、「峰值表算出来的比 --memory 小时应当取 --memory 8G」同样收到「36G」），疑与包装给子进程设的环境有关，没细查；派发要求我「经 run-with-memory-cap.sh 4G 跑」，交给 K3。
  - ㉑ 混合大小写：`dispatch-prompt.py --check` 对报告行 `…/Summary.MD`、`…/Report-Final.Md`、`…/FindingsX.md`、`$SINGLEFS_WORK_ROOT/zz-sample/REPORT.md` 都 rc=3；`summary.markdown` 与目录名里带 reports 不拒（工具层本来放，调度记录第 353 行写了）。
  - 账表两桶：见上一节，652 / 4。
  - 跨行「」引文、`dispatch-prompt.py` 文件头：自证 609 / 110 rc=0，开关点名 17 条（`tools/switch_cell.py`）全相同。
  - `claim-experiment.sh` 进复跑表（`.claude/gate.d/break-switch-replay-files.tsv:26`）、`CLAIM_SKIP_*` 改成 `CLAIM_EXPERIMENT_BREAK`（`scripts/kb/claim-experiment.sh:25-31`，三条开关各红在点名的一格，我逐条设上跑了：「E6 已有源码却没被拒」「另一棵工作树用过 E12，本棵 --next 该是 E13」「目录登记：入库装置目录里有 E40…」）；`stage-owners.tsv` 设计员（`.claude/gate.d/stage-owners.tsv:106-107`）。
  - `PIPESTATUS`、worktree 相对路径的报告：上一节。
  - 比对对象表 r7 三行：见 ㉖，19 行都有欠账号。
  - 「弄坏开关点名不全」做成门禁格判点名与实际红逐格相同（`.claude/gate.d/code-tooling.sh:4460`，文件头 `:4425-4445`）：格本身我跑不了；我用同一条规则仿判了 11 份脚本和钩子的自证：派发闸 120、交回闸 42、`dispatch-prompt.py` 17、`snapshot-tree.py` 9、续做闸 7、`lib_prompt_scratch_line.py` 6、`stale-candidates.py` 5（汇总行除外）、`insert-row.py` 4、`claim-experiment.sh` 3、`replace-batch.py` 3、`replace-once.py` 3（汇总行除外），点名没红的 0、红没点名的 0。登记表 `.claude/gate.d/break-switch-replay-files.tsv` 里崩溃放量、崩溃点重放、变异分片、广谱变异四份在 `:50-53`；`agent-watch.py` 不在表里，与调度记录「没登记」一致。**偏差三**：行文写「五份慢自证由任务 53 到 56 补齐」（`records/2026-10-23-subagent拆分提案.md:948`），任务 53 是 E171 编排改取登记线程（调度记录第 375 行），慢自证是 54 到 56；E171 那 11 份不属于第九十节任一行，是用户弹窗定的另一件（调度记录第 374 行）。
  - 遗留的看门狗自证目录、`m4-predev` 语料：主 agent 的活，目录已不在；语料我没看。
- 推翻条件：上面任一条开关设上之后的失败行与点名对不上；或 `ls ~/.local/share/singlefs/work/agent-watch-selftest-scratch-*` 又有。

## 十六、「推的、没量过」的有没有被当成量写进别处

- `--checker-quick-memory` 缺省：定义与脚本里标得清楚——`.claude/main-agent.md:52` 写「表里没有取推的 36G」，`scripts/gate/merge-and-verify.sh:121` 与 `:277` 写「推的」，成功句与开跑行写出上限和取法（`scripts/gate/merge-and-verify.sh:743` 的「量过的」只给峰值表里真有记录的）。没有当成量的写进定义或脚本。越线的是记录：调度记录第 366 行「36G 够用」、第九十节 ㉟ 去向的「主 agent 量带特性那一趟峰值 36.0 GiB」，读数等于上限，只能说「那一趟没被杀」，不能说「需要 36G」（见上节偏差一）。
- 任务 49 的收窄代价（「N 核」与单写一台的线程数不判）：写在调度记录第 365 行「代价是…不判」，派发闸开关表里有对应的放行格（`.claude/hooks/runner-dispatch-guard.sh:787`）；没有哪个定义或规则把它写成「写死额度都拦得住」。依据（真提示里这两种撞到的都是观测）是量过的语料数，不是推的。
- 第九十节 ㉚ 原文里的「推的，没量过真提示里有几份」这一批量了（84 或 63 份，最晚 2026-10-12）；㉝ 原文的「推的」两件（相对路径、续做闸开关漏进环境）这一批改成了有格的判法；㉞ 的「推的」那件（`$SINGLEFS_WORK_ROOT/` 开头的报告路径）改成了有自证格的判法，我只验到自证格那一层。
- 任务 53 写的「复用键不含编排器 sha256 与线程数，下次要重判的只有 research-mutations」是读代码的结论，我只核了钉住的 sha256 与现文件相同（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:424` 与 `sha256sum rows_orchestrator.py` 同为 e6f3a370ca6f66577e5cd9f3d44a30dd315660958ebb50922df3fc53ec6ef8ff），键里含不含它没核。

## 十七、判定一览

| 格（第九十节行） | 判定 | 一句话 |
|---|---|---|
| ① 执行员登记落地清单 | 做到 | 交回闸 ⑦ 缺节、行号越界、用例 grep 不到、文件不在、空节都 rc=2，全对 rc=0；落地之后 13 份执行员交回新拒 0；子串豁免会被「不是只补落产物」误放 |
| ② preregistration-parallelism 挂钟 | 一半 | 挂钟那一行自己的态数乘单态耗时做到（红绿样本、真仓三份判 3 条）；形态列要的第六节态数没读，两处写得不一致不红，散文写法只「本次未判」 |
| ③ 延后项登记 | 做到 | 五类红、六类绿都对；登记处表（流程文件）比 main-agent 第 2 条列的多一类，去向列已写任务 47 |
| ④ stale-candidates --check-report | 做到 | crates 下要改要补不在「交实现员」一节退 3，节在退 0，71 份历史报告 0 误报（也无真阳性） |
| ⑤ 常设要求骨架与闸 | 一半 | B6/B7/B4/A2/B2b/B10 加三方「报告：」的红绿都对；「不做」六条里 B9、E2 成立，B8、A5 一半，D3 依据不成立，C4 弱 |
| ⑩ 上游 gate-overlap 认 hook-swap | 做到（判不做成立） | `gate-reuse-check` 本仓没注册，上游文件头写明解释器写文件归门禁阶段，门禁查重按 diff 窗口；没核用户级 settings |
| ㉕ Bash 写 .rs、mktemp -p | 做到 | mv/cp/>/>>/tee/&>/cp -t 都算改动，根认 `$(git rev-parse --show-toplevel)`；检出钩子 799 种与 `-p`、`--tmpdir`、套一层格；`lib_writer_lock.py` 三处对上；行外：目标写成命令替换的不计 |
| ㉖ 门禁尾巴 | 做到 | 六格共用 `kb_roots_or_red`，坏字节读不出判红；两份文件头补上；(c) 删对；不登记的 19 行都有开着的欠账号 |
| ㉗ 派发闸口子 | 做到 | 钉住的样本 rc=3/0 对；名里带 report 照判；没草稿目录行只判裸文件名；120 条开关点名与实际红相同（仿判） |
| ㉙ 禁读清单与会话名 | 做到 | 禁读清单点到本腿报告 rc=3；三份定点写入脚本对会话名退 8/7/5、不写字节 |
| ㉚ 另两处认法改调共用库 | 做到 | `dispatch-prompt.py`、`agent-cost.py` 都调库；⑯ 收严影响 63 份（最晚 10-12），账表两桶 652 / 4 |
| ㉝ 围栏、相对路径、续做闸、扫描 | 做到 | 14 种围栏写法、相对路径换当前目录结果相同、续做闸与检出钩子设开关退 2、没扫目录下的行判本次未判且已扫目录里没有要放行的仍红 |
| ㉞ 交回闸四件与验证段与快照树 | 做到 | 落盘再读汇总行、persisted-output 取 rc=、两种拒绝说明、PIPESTATUS 都对；共用约束 63/99 与交回闸逐字同一种写法；快照树 80/80、自证 26 项 |
| ㉟ 最后的延后项 | 做到，三处偏差 | 36G「够用」是读数等于上限；`merge-and-verify.sh --selftest` 经 `run-with-memory-cap.sh` 包着红两格；行文「任务 53 到 56」多了 53 |

## 十八、没做什么

- 没跑门禁阶段本身（`doc-experiments.sh`、`doc-process-records.sh`、`doc-registries.sh`、`code-tooling.sh`、`stage-cells.sh` 的整格或样本档）：子 agent 被 `heavy-test-guard.sh` 拒（提交时才跑的检查）；只抽 python 或读代码。break-switch-replay 格我用它文件头的规则自写了仿判，不是那一格。
- 没验：看门狗 ⑭ 在真进程上的读数、`mutate.sh`/`broad-mutation.py`/`mutation-shard-run.sh`/`crash-*` 的开关复跑（慢、归主 agent 的复跑）、检出钩子 799 种之外的开关、`heavy-test-guard.sh` 与完工清理闸的开关表（表头格式不同，我的仿判没解析出开关）、交回闸与派发闸的 3195 / 3394 / 3484 份全语料新拒新放读数（只抽了执行员 13 份和一批造的输入）、`$SINGLEFS_WORK_ROOT/` 开头的三方腿报告走真 worktree、用户级 settings、E171 编排的复用键。
- 没判：K2、K3 的格（矛盾、越位、最脆）；没替主 agent采纳或出判决。偏差三处与不做六条里的判语是我对改法做没做到的读法，由主 agent 现查后定。
- 对 HEAD 的 diff 里不属于这 14 行的改动我没当成这一批的、也没归属：`doc-registries.sh` 里 third-txn-hooks、field-widths、mount-transparency-fields 几格，`heavy-test-guard.sh` 文件头里工作区 cargo 与 here-string 那几段，`.claude/rules/three-way-inference.md` 第 3 步多出的「复跑那一步做完，副本里的仓副本与编译目录当场删」那一句。
- 完工清理：草稿目录 `~/.local/share/singlefs/work/landing-audit-r7-sonnet/` 里的仓副本与库拷贝交回前删，删之前 `du -sh`：`repo` 13M、`head` 3.5M、`snaptree-r7` 6.9M、`sess` 408K、`proj` 248K、`ps` 32K、`gs` 44K、`kbd` 12K、`kbt` 36K；留下 `tools/`（104K）、`prompts/`（172K）、`out/`（224K）。

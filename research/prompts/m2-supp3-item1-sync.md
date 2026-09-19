<!-- knowledge-sync -->
# m2-supp3-item1 阶段同步

阶段：里程碑「第二个事务」增补 3 第 1 件（随机历史生成器）收口、层 0 崩溃点重放并行化、看门狗与 Bash 检出 hook 不再对 heredoc 正文误报、门禁 33 号数 crates 变异表的锚点、实现流程规则两节，以及这一段对各 agent、规则与门禁的评估（2026-09-18 到 19）。基准 HEAD `58a5ebb`，UTC。
触发文件：crates/singlefs-harness/src/history.rs、crates/singlefs-harness/src/crash.rs、crates/singlefs-harness/src/lib.rs、.claude/gate.d/54-layer0-replay.sh、.claude/gate.d/33-mutation-tables.sh、.claude/gate.d/stage-owners.tsv、.claude/hooks/bash-command-detector.sh、.claude/rules/implementation-workflow.md、research/scripts/agent-watch.py
代码轮三方两轮的判决在 `research/prompts/m2-supp3-item1-code-r1-main-verification.md`、`research/prompts/m2-supp3-item1-code-r2-main-verification.md`；评估在 `records/2026-09-16-subagent拆分提案.md` 第三十四节。

工作区里混着别的会话没提交的改动，这一阶段的改动先用临时索引做成一个不进分支的提交对象 `a023682`（引用 `refs/sync-tmp/m2-supp3-item1`，做法写在第三十四节表后），回扫按 `--base 58a5ebb --target a023682` 跑。之后主 agent 又改了四处（门禁 33 号的出路挪位、第三十四节、第二十五节的措辞、下面「命中处置」里改了的两行），都在最后提交的那一版里。

## 搜索

回扫员两段，产物在 `research/prompts/m2-supp3-item1-sync-sweep/`：
- 写事实表：`python3 research/scripts/stale-candidates.py --changes --base 58a5ebb --target a023682` → 1 条变更记录；`--check-facts facts.tsv` → 16 行事实全罩住；`--facts facts.tsv --out candidates.tsv` → 21 行候选（19 个不同的行），按组 F1 = 14、F2 = 2、F7 = 1、F10 = 3、F14 = 1，其余 11 行事实是新立事项、不出候选（`facts-report.md`）。
- 逐行判：21 行里不相干 14、事件句不改 6、要改 1，另反向核补「要补」2 行（`judge-report.md`）；主 agent 跑 `stale-candidates.py --check-report candidates.tsv judge-report.md` → 「21 行候选都有逐行判定」，退出码 0。判成不相干与事件句不改的正好 20 行，主 agent 全部复判，没有判错的。
- 主 agent 另搜：`grep -rn '锚点预扫\|改定义要走三方' .claude/kb/milestone/02-second-txn.md records/*.md .claude/agent-common.md .claude/agents/*.md .claude/rules/*.md` → 3 处（里程碑第 42 行、记录第三十二节一行、它的历史条目）；`grep -n '今天没改' records/2026-09-16-subagent拆分提案.md` → 1 处（第二十五节）。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/milestone/02-second-txn.md:399 | 固定脚本走不到的问题，今天都是三方攻方腿自己写历史扫出来的：……攻方腿写的历史生成器住在 `research/prompts/` 下各轮的模型目录里，不在 `crates/`，门禁不跑。 | 改了：「今天」改成「立题时」（那一段挂在「立题时现查的验证手段（2026-09-18）」下），段末补第 1 件收口之后生成器快档随 `check.sh` 进门禁一句；历史版本 2026-09-19 记了一条 |
| .claude/kb/milestone/02-second-txn.md:359 | 还开着：改代码的一方整表跑一遍 59 号的锚点预扫，这一条要写进 `implementation-writer` 定义，改定义走三方 | 改了：做成门禁 33 号的一段、阶段归属表登记给实现员，定义没改 |
| records/2026-09-16-subagent拆分提案.md:749 | `implementation-writer` 改了代码就整表跑一遍 59 号的锚点预扫：改定义要走三方，没开 | 改了：同上；历史版本 2026-09-19 记了一条 |
| records/2026-09-16-subagent拆分提案.md:619 | 今天没改，写在这里是为了下一个人看到这条报警时知道先分辨。 | 改了：写 2026-09-19 前两种检出也先剥掉喂给非 shell 命令的 heredoc 正文；历史版本 2026-09-19 记了一条 |
| records/2026-09-16-subagent拆分提案.md:784 | （新增） | 补了：第三十四节「周限额」一行（回扫员反向核的 M1：这一段两次撞周限额都用续做接着做完） |
| records/2026-09-16-subagent拆分提案.md:787 | （新增） | 补了：第三十四节表后「临时提交对象的做法」（回扫员反向核的 M2） |
| .claude/kb/milestone/02-second-txn.md:311 | 第 ② 行（F 的「生效」事后回落……一次挂载转过一整圈根环时 checker 在合法状态上判 I-3.1 红） | 不改：同一格里这一阶段已补「增补 3 第 1 件的历史生成器坐实」一句 |
| .claude/kb/milestone/02-second-txn.md:358 | 第 41 行（层 0 单进程 3 小时 50 分 49 秒，这是以后并行的对照基准） | 不改：单线程的数是对照基准，同一格里已写并行版的读数 |
| .claude/kb/milestone/02-second-txn.md:40 | 打回决策四处里 ② 的摘要（一次挂载内转过一整圈根环时 checker 误红） | 不改：只是现象的摘要，坐实与否记在第 ② 行本身 |
| .claude/kb/decisions/15-格式冻结政策.md:29 | 冻结后规则脱节、门禁越跑越慢却没人量化，会在风险更高的阶段变成事故 | 不改：说的是冻结前清单的一般理由，不是层 0 用几个线程 |
| .claude/rules/implementation-workflow.md:44 | （新增）「测试与崩溃检测优先多线程」一节 | 补了：用户 2026-09-18 定，写法、合并、进度、门禁 54 号管哪一半 |
| .claude/rules/implementation-workflow.md:30 | （新增）「别的会话改了快照里的文件」一段 | 补了：派核查员之前 `sha256sum -c`、倒推快照时的原样、判决开头写明；倒推用的替换清单存 `research/prompts/m2-supp3-item1-code-r2-foreign-edits.py` |

候选表里其余 14 行（里程碑第 266、283、294、360、361、367、385、403、404、406、416 行，记录第 750 行等）是检索词带出来的别的「第 1 件」或引生成器作旁证的句子，逐行理由在 `judge-report.md`。

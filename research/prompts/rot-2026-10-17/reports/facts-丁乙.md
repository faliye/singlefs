# 事实表丁乙：verification.md、skill、CLAUDE.md、README.md、records/、tooling.md、verification-build.md（基准 ce1457c3，结束在工作区）

日期 2026-10-17。没建副本，没起编译，没跑任何重型；只读仓、只写草稿目录与本报告。

## 产出

- 事实表：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁乙/facts-丁乙.tsv`，56 行（编号 F900-F955）：事实变了 41 行、新立 11 行（F905、F906、F909、F943、F944、F945、F947、F950、F953、F954、F955）、只改措辞 4 行（F912、F913、F948、F949）。数行命令：`grep -c . facts-丁乙.tsv` 得 57（含表头）。
- 候选表：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁乙/candidates-丁乙.tsv`，1166 行候选（1059 个不同的行），分在 41 个组（F900-F955 里出候选的那些；新立与只改措辞不出候选）。
- 事实表生成脚本与两条命令的完整输出也在同一草稿目录（`gen_facts3.py`、`check-facts.out`、`facts.out`）。

## 两条命令与输出末行（原样）

`python3 scripts/kb/stale-candidates.py --check-facts /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁乙/facts-丁乙.tsv --base ce1457c3`，退出码 6（预期：整个阶段 67 条 H 编号我这片一条都没有）。输出末两行：

```
  ✗ 事实表没罩全：67 条变更记录没罩、0 行正则坏了、0 行检索词在基准零命中、0 行在结束零命中、0 行检索词太宽、0 行多余的 &&、0 行冒充新立、0 行冒充只改措辞、0 行宽词没抽查
     → 怎么办：读 --changes 给的变更清单，每条 H 编号写进它说的那件事实的「出处」列（只改措辞的也写一行，旧新事实写「只改措辞」）；检索词写新旧说法都会提到的概念名词，基准与结束两版都要命中；太宽（超过上限）才用 && 并上第二个概念名词收窄；事实真变了的不许写成只改措辞或新立；命中超过 100 行的先抽 10 行自判相干不相干，精度低就改窄，精度够就在「出处」列写「宽词已抽查：抽 10 行、相干 N 行」
```

除「67 条变更记录没罩」外各项红都是 0。

`python3 scripts/kb/stale-candidates.py --facts /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁乙/facts-丁乙.tsv --base ce1457c3 --out /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁乙/candidates-丁乙.tsv`，退出码 0，输出：

```
  ✓ 候选表 /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁乙/candidates-丁乙.tsv：1166 行（1059 个不同的行）
```

## 每个检索词在基准与结束两版的命中行数

命令：用 `stale-candidates.py` 自己的 `current_state_corpus` 与 `compile_search_term` 在两版语料上数（脚本在草稿目录外的会话暂存区，数的口径与 `--check-facts` 相同）。格式：编号 基准命中 结束命中。

F900 62 61；F901 81 60；F902 12 14；F903 23 51；F904 7 15；F907 38 41；F908 12 25；F910 36 44；F911 8 23；F914 84 67；F915 7 25；F916 55 29；F917 4 20；F918 31 62；F919 20 38；F920 10 14；F921 73 49；F922 12 11；F923 3 2；F924 4 5；F925 16 42；F926 13 13；F927 22 24；F928 2 2；F929 34 35；F930 40 43；F931 60 58；F932 24 74；F933 4 5；F934 15 35；F935 59 73；F936 15 6；F937 42 37；F938 6 6；F939 7 8；F940 1 19；F941 1 1；F942 5 10；F946 9 12；F951 4 4；F952 3 2。

新立行（基准必须零命中，结束命中）：F905 0 6；F906 0 12；F909 0 20；F943 0 25；F944 0 16；F945 0 45；F947 0 2；F950 0 4；F953 0 6；F954 0 10；F955 0 11。

结束那一版命中超过 100 行的检索词：没有（结束版最大的是 F932 的 74 行、F935 的 73 行、F914 的 67 行），所以没有宽词抽查。精度没量：41 个事实行的候选里，F931、F932、F935、F914、F918、F900、F901 这几组命中 58 到 74 行不等，里头会有「不相干」，逐行判时按「不相干」处理。


## 我这片里判为「只改措辞」的载体

写成只改措辞行的（F912、F913、F948、F949，不出候选）：

- 术语改名（层 0 → 崩溃点重放、layer0 → crash-point-replay、门禁 checker-tier-layer0-replay → checker-tier-crash-point-replay）：`.claude/rules/verification.md`、`.claude/skills/crash-test/SKILL.md`、`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/gate/staged-gate.md`、`CLAUDE.md`、`.claude/kb/verification-build.md`，以及 records 里的 `2026-09-13-总审核`、`2026-09-24-里程碑二收尾调度`、`2026-09-27-验证两档拆分`、`2026-09-28-门禁59号提速与双机分片`、`2026-10-01-门禁注释精简`。
- 路径搬迁（`.claude/kb/invariants.md` → `crates/singlefs-checker/invariants/`）：`CLAUDE.md`、`verification.md`、`crash-test/SKILL.md`、`verification-build.md`，以及 records 的 `2026-09-03-验证三件套落地调研`、`2026-09-05-C113定案`、`2026-09-27-代码审阅38条去向`、`2026-09-30-里程碑三开工第一轮`、`2026-10-01-删除与并发事务`。
- CLAUDE.md 与 skill 的删日期、位置指代改写（「本文件」→「`CLAUDE.md`」）、「抄了别的项目看不到」类尾巴删掉、SOP 副本同步说明改写：`CLAUDE.md`、`.claude/skills/decide/SKILL.md`、`.claude/skills/gate/SKILL.md`（gate 那份另有 --selftest 一句是事实变了，在 F927）。
- records 删钟点、补简称、会话名改「该会话」、路径搬迁（`scripts/kb/oov-check.py` → `scripts/agents/`、`research/scripts` → `scripts/`、峰值表 → `~/.local/share/singlefs/memory-peaks.tsv`）、「第二条流」→「覆盖写固定脚本流」、「耗时用例」→「慢档用例」、「按条记录」→「判定库记录」、「轻型」→「快档」：`2026-09-17-已分配口径三方与两个实验`、`2026-09-18-回扫员可靠性`、`2026-09-19-里程碑二遗留收拢`、`2026-09-21-增补3第4件与checker补三条`、`2026-09-23-三轮三方与四类没东西会红`、`2026-09-27-同步远端冲突块`、`2026-09-28-agent体系用量时间与调度分析`、`2026-09-28-收口表对齐与三笔欠账复查`、`2026-09-28-里程碑三作业点规划`、`2026-09-28-里程碑三崩溃放量剪枝与GPU调度`、`2026-09-28-里程碑三第十项清单`、`2026-09-28-里程碑二收口表回写与系统配置槽写频率门禁`、`2026-09-28-里程碑二收尾接手`、`2026-09-30-里程碑三开工第一轮`、`2026-10-01-README更新与知识腐烂治理`、`2026-10-01-对接上游测试脚本规范`、`2026-10-01-里程碑三形式化接入`、`2026-09-17-CLAUDE.md去冗余`、`2026-09-19-决策瘦身与双向登记`。判法：我对每份 records 的旧新两版做了机械归一（去钟点、日期、括注简称、上述改名与路径替换）再逐段比，归一后仍不同的段逐段读过；剩下的不同段分三类：已写成事实行（F938 定义数、F939 总审核三行状态、F940 测试代码不走代码轮）、纯新增的事件段（见下）、对接上游测试脚本规范里 D13/D6/D16 三块改法登记的次序调整（只挪位置与改写成快档，措辞）。
- README.md 本身：整份是 `README.zh.md` 的英译（中文原文原样挪过去，编号的英日简称写在半角括号里），译文本身不单列；中文原文比旧 README 少的事实句（测试规模、已知红、崩溃放量态数、双机配置键表、依赖安装细节等）写在 F937 与 F921、F918。

records 里新增的纯事件段（没有改既有结论句，不占事实行；作为事件句判）：`2026-09-16-subagent拆分提案` 第七十节（测试代码不走代码轮，同一事实的现状版在 F940）、`2026-09-30-里程碑三开工第一轮` 末的「第二次搬迁的收尾」段、`2026-10-01-删除与并发事务` 末的交接表进展段、`2026-10-11-里程碑三待优化上下游协调` 的「第 3 批收拢折算与挂钟」段、`2026-10-11-里程碑三知识腐烂治理` 的第二轮与收尾段。

## 反向核：没记的事

对象：stage-done.md 第一节十四个提交正文里的每一件事、第二节十六份记录的题目；在现状载体（kb、规则、README、CLAUDE.md、`.claude/kb/tooling.md`、各 skill）里各找一处记着没有。命令是 `grep -rl` 在这些载体里按名字搜（口径同事实表）。

已记着、不用补的：三语 README 与简报（CLAUDE.md briefs 行与 trilingual-documents、README.md 顶上三语链接）；`research/README.md` 与 `scripts/` 各子目录说明（CLAUDE.md 的 `scripts/README.md` 行、README.md 的 `scripts/` 行）；directory-layout 与 timer-only-remind 两条规则（CLAUDE.md 的 @ 引用）；remind 技能（tooling.md「到点自己开工」）；不变量清单搬家（CLAUDE.md、verification.md）；checker 档新模块 crash_state_enumeration、crash_library、crash_point_replay_progress（verification.md checker 档行）；cpu-judging-sites（verification.md）；两段流水线与用户 2026-10-11、2026-10-15 的定案（verification-build.md）；复用键（verification.md、verification-build.md）；决策进度列（CLAUDE.md）；E142 r25、E165 第二段杀伤界、E171 修订 12（对应实验页都有）；里程碑 04-17 规划文件（CLAUDE.md 与 kb INDEX 都只按 `NN-简称.md` 泛称，没有逐个列，不用补）；scripts/hosts 的 minimal-copy、work-directory-ownership、heavy-threads 与 scripts/gate 的 directory-layout-check、data-name-lint、wallclock-assert-lint（只在 `scripts/hosts/README.md`、`scripts/gate/README.md` 里有，这两份 README 不是候选载体，见 F943）。

没记的，已写成事实表的「新立：」行或事实变了的行：

1. 新 crate `crates/singlefs-unit-graph`（暂存区）：CLAUDE.md 与 verification-build.md 仍写「七个 crate」，README.md 的 crate 表也没有它；只有 verification.md 一处（build.rs 经它算判器组单元键）。→ F915。
2. harness 档新增的 `heavy_threads` 模块与 `common_images_for_the_checker_tier` 脚手架：verification.md 的 harness 档行没写。→ F954（新立）。
3. 四张门禁登记表 `fixture-none-exempt.tsv`、`timing-load-field-lag.tsv`、`wallclock-assert-lag.tsv`、`decision-evidence-lag.tsv`，三语检查库 `lib-trilingual.py`，排除表 `naming-ordinal-tokens` 与 `data-name-lint-exclude`：只出现在 checks-owed.md 的欠账行与 records 里，规则与 kb 正文不说它们是什么。→ F955（新立）。
4. `.claude/heavy-thread-default` 作为线程数唯一登记位：agent-common 与规则里都写了，但 CLAUDE.md 与 tooling.md 没有；规则里已记着，所以只作新立行 F945，不算漏。
5. 提交 3603ace4 的 `.claude/crates-review-exempt`：implementation-workflow.md、agent-common.md、main-agent.md 记着，不漏。
6. 记录 `records/2026-10-14-全量前修红.md`：这个文件是 0 字节（`ls -la` 显示 0，`git log` 里是 111f7dc6 带进来的空文件），题目在，内容没有；现状载体里没有任何一处记「全量前修红」做成了什么。→ 见下一节，没法写事实行。

## 要主 agent 现查（不在事实表形态里的可疑之处）

1. `records/2026-10-14-全量前修红.md` 是空文件（0 字节），111f7dc6 的提交正文把它算进「新增 16 份」。要么内容丢了（`git log` 里这一个文件只有 111f7dc6 一次提交，没有更早的版本可取回），要么没写完就入了库；stage-done.md 把「全量前修红」当做成的事，现状载体里没有一处说它修了什么红。
2. `.claude/kb/verification-build.md` 第 4 行写「登记的崩溃枚举用例 13 条（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 数出）」；我现跑这条命令得 14，合表 `.claude/gate.d/crash-point-replay-items.tsv` 去掉注释行也是 14 行。旧 README 的「13 项」同数。这条数是这次治理里新写进去的（基准那一版是「两条」），已经过时。→ 事实行 F929、F942 会把它列进候选。
3. `CLAUDE.md` 第 8 行与 `verification-build.md` 第 4 行写「七个 crate」，`ls crates` 有八个目录，`Cargo.toml` 的 members 有 `crates/singlefs-unit-graph`（暂存区，还没提交）；README.md 的 crate 表也没这一个。→ F915。
4. `.claude/rules/verification.md` 新加的一条（凡带判定库的门禁与入口登记在 `.claude/gate.d/verdict-library-scripts.tsv`，自证有 `reuse-three-questions` 一格）引用的 `scripts/gate/lib_reuse_checks.py` 在 `scripts/gate/README.md` 里没提（`grep -c lib_reuse_checks scripts/gate/README.md` 为 0）；这是子目录说明的缺项，不是现状载体里的句子，不在候选表里。
5. `README.md`（英译）的验证表写「Pool-level checker … The 92 invariants」「Formal verification … journal prefix, root selection and the ring scan」，与 CLAUDE.md、verification.md 的新口径一致；但旧 README 里的「已知红」与「测试规模」两行整行删了（F937）。别的载体（verification-build.md、briefs）若还在引用 README 的那两行条数，会悬空；我没有逐个查 briefs（按日期不回改）。
6. `tooling.md` 第 8 行说 2026-10-11 删掉了本地模型腿脚本，写了 `git log --diff-filter=D -- scripts/agents/ask-local.sh` 取回办法；但 `.claude/kb/decisions-history.md`、`.claude/kb/experiments/34-根环槽几何.md` 和多份 records 还在把 `ask-local.sh` 当现有脚本写（F916 的候选里有）。这类要逐行判是现状句还是事件句。

## 没做什么

- 没读 `research/prompts/` 与 `briefs/`（冻结证据、按日期不回改）；没读 `~/.claude/` 的会话记录；真活使用证据只用了 stage-done.md 给的。
- 没跑 `--changes`（照派发：整份 1 MB，别的片归别人），所以我这片的 H 编号覆盖（67 条）没有对：本片顶上的变更记录表是空的，`--check-facts` 报 67 条没罩是预期；合表时由主 agent 拼别的片的 H 编号。
- 片内的差异片段（`changes-丁乙.md`）是字符级碎片，我改用 `git show ce1457c3:<文件>` 与工作区逐行比（整行配对后按词比）把每个被改的载体读全：`verification.md`、`crash-test/SKILL.md`、`crash-test/commit-stages.md`、`gate/staged-gate.md`、`decide/SKILL.md`、`gate/SKILL.md`、`remind/SKILL.md`（新增整份）、`CLAUDE.md`、`README.md`（与 `README.zh.md` 比旧中文版）、`tooling.md`（新旧整份读）、`verification-build.md`、31 份已有的 records（含 2026-10-11 的两份调度记录，它们是追加段）。新增的 16 份 records 按题目与开头课题表读，没有逐份读全文。
- 没有给 records 的事件段逐句判是否又含现状句，那是候选表逐行判的事。
- 事实表里新立行（11 行）的「基准零命中」只对着现状载体的句子（CLAUDE.md、README.md、agent-common、main-agent、agents、rules、SKILL.md、kb、records 的历史节之前）；`scripts/*/README.md`、`.claude/gate.d` 下的脚本与登记表里原来有没有，不在 `--check-facts` 的语料里，我另用 `grep` 看过 scripts 各 README（F943 的几样在 `scripts/hosts/README.md`、`scripts/gate/README.md` 里有）。
- 候选表 1166 行的精度没有逐行量；命中 58 到 74 行的几组里会有不相干的行。
- 没改任何被搜到的文件，没改 `stale=`。

## 推翻条件

- 若事实表某行的「新事实」与结束那一版的代码不符（例如单元键的构成、`--full` 的各门禁行为、crate 个数），那一行的候选就判错；我对新事实一律取自结束那一版的 `verification.md`、`commit-stages.md`、`staged-gate.md`、`verification-build.md` 的字面，没跑门禁核。
- 若 `crates/singlefs-unit-graph` 在提交前被撤回或改名，F915 与 F901 的「算法住在这里」要改。

# 这一阶段做成的事（ce1457c3 → 工作区，2026-10-11 到 2026-10-17）

## 一、提交正文（ce1457c3..HEAD，原样）

### c1e9b324 2026-10-14 三方论证材料：已入库各轮的续写与路径修正

239 份 research/prompts：在跑的轮续写的报告与判决，以及跟着脚本搬家改的路径；只改路径的部分提示与腿的答复内容不变。



### ceac9b0f 2026-10-14 三方论证材料：这一批新开的各轮

背景材料、正文、小节清单、腿报告与腿模型目录、开工快照与判决（1085 个文件）。



### 30c245b0 2026-10-14 实验产物：E142 r25、E165 第二段杀伤界、E171 修订 12 段长与重钉前后的留存产物

新增 29 份、改 11 份 research/results。



### 768439d4 2026-10-14 README 与简报：三语同步，研究目录说明

- README 中、英、日三份与 briefs 同步改；新增 .claude/trilingual-documents 登记受三语管的文件
- 新增 research/README.md：research/ 只有 prompts 与 results，里面的文件都可删、从 git 历史取；multi-host.env.example 跟着改



### 111f7dc6 2026-10-14 记录：10-11 到 10-14 的调度与审计记录，时间只写到日期

- 新增 16 份：scripts 审计两轮、singlefs-core 严格审核、崩溃放量两段与脚本审计、指令文档删论证与来历、里程碑三项五收口、
  里程碑五打包与决策进度列、工具与治理批、非代码收尾、里程碑四调研、Verus 证明审计、三语 README 与简报、代码欠账批、
  复用键与全仓改名、欠账还五条、全量前修红
- 删掉记录里的钟点：8 处光秃的「时:分」与带时区词的写法删掉只留日期；运行目录名里 17 处紧凑时间戳改成只带日期
  （这些目录在本机，名字只为定位，改后按日期与进程号找）
- 按 doc-lint 修掉翻译腔 2 处、「原先」3 处



### d40c2d51 2026-10-14 kb：决策、实验页、欠账、里程碑与变更史跟着里程碑三推进，新增翻译术语表

- 决策与实验页、变更史、checks-owed、里程碑与布局表的改动（188 个文件）
- 新增 .claude/kb/translation-glossary.md（中英日三语术语表）
- 按 doc-lint 修掉：位置指代 3 处、裸编号补简称 11 处（checks-owed、决策变更史、E171、里程碑三、术语改名表），
  术语表的举例不再借用真编号



### 3603ace4 2026-10-14 CLAUDE.md、agent 定义、共用约束与规则：写与验分两段，目录规矩，定时只用 remind.sh

- 新规则 directory-layout（scripts/ 与 research/ 各放什么）、timer-only-remind（定时只用 scripts/agents/remind.sh）
- main-agent.md 加「写与验分两段」一节、线程上限只照登记值、叫停轮的登记出处写法；verification、implementation-workflow 等规则跟着
  崩溃点重放改名、产品代码与测试代码分开走代码轮改
- CLAUDE.md、agent-common.md 与各 agent 定义跟着改；新增 .claude/crates-review-exempt（按内容哈希登记不走代码轮的产品代码）



### d9f06759 2026-10-14 钩子、技能与项目配置：定时只许 remind.sh，重活线程数统一登记，工作区写锁

- 新钩子 timer-only-remind-guard（拒 CronCreate、ScheduleWakeup、为定时写的 sleep 等）与 remind-rearm-stop-guard（循环提醒到点重起）；
  新库 lib_abandoned_rounds、lib_workspace_cargo_lock、lib_writer_lock；其余钩子与 settings.json 注册跟着改
- 新技能 remind，crash-test 与 gate 技能里的命令改到新入口
- 新增 .claude/heavy-thread-default（每台默认重活线程数的唯一登记位）与 heavy-thread-exempt、long-run-live-hosts-exempt、
  naming-ordinal-tokens / exclude、data-name-lint-exclude 等排除表；.gitignore、SOP 版本戳跟着改



### 021b4db0 2026-10-14 门禁：层 0 阶段改名 checker-tier-crash-point-replay，新增三语、判器调用点、挂钟断言等格与样本

- checker-tier-layer0-replay.sh 改名 checker-tier-crash-point-replay.sh，快档两趟与全量照新入口跑
- 新增登记表：cpu-judging-sites.tsv（判器调用点）、fixture-none-exempt.tsv、timing-load-field-lag.tsv、wallclock-assert-lag.tsv，
  三语文档检查库 lib-trilingual.py
- doc-text 等阶段的格与判法跟着改；fixtures 新增与调整一批红绿样本（含改名搬迁）
- litmus 跟着改



### 1b6f9865 2026-10-14 脚本：scripts/ 每个子目录加说明，层 0 分片脚本改名崩溃点重放，新增定时、重活线程与目录规矩的工具

- scripts/README.md 与九个子目录各一份 README：放什么、从哪跑
- crash：layer0-shard-* 改名 crash-point-replay-shard-*，经崩溃放量双机驱动跑；新增判器调用点检查 cpu-judging-sites-check.py
- agents：新增 remind.sh（唯一的定时办法）与三方重跑模板；删本地模型腿 ask-local
- hosts：新增 heavy-threads（每台默认重活线程数）、minimal-copy、work-directory-ownership
- gate：新增 directory-layout-check、data-name-lint、wallclock-assert-lint 与源码摘要、重编包、工作区锁三个库
- experiments：E171 装置补默认线程数与冻结检查用例；删掉已不用的 corruption-*、local-repro*、oov-check 与词表
- 其余脚本跟着改名与新路径改



### 7b5711fd 2026-10-14 不变量清单从 .claude/kb/invariants.md 搬到 crates/singlefs-checker/invariants/

- 按类拆成九份（块自描述、根环、journal、校验和、空间记账、事务与崩溃一致性、分配器、加密、inode 树结构）加 README，
  和判它的池级 checker 放在一起
- 新增 .claude/kb-directories 登记 kb 文本住哪两处（.claude/kb 与不变量目录），doc-lint 等照它认 kb



### d6dd6da4 2026-10-14 代码：崩溃枚举经判定库的两段与给定状态表，层 0 改名崩溃点重放，harness 档供图像给 checker 档

- singlefs-checker-tier：新增 crash_state_enumeration（崩溃枚举用例经判定库的两段：先入库、再从库里读回判）、
  crash_library（几类测试共用的长期判定库与两段协议）、crash_point_replay_progress（取代 layer0_progress，
  发现日志与工具链身份）；GPU 判器表、派活与判法跟着改；给定状态表新增回退到 F_生效根、随机历史与故障注入
  经库判、harness 场景镜像三组用例；删掉断点续跑与跨进程分片两条旧用例
- singlefs-harness：新增 common_images_for_the_checker_tier（把各场景的镜像交给 checker 档当给定状态）、
  heavy_threads（每台默认重活线程数的取法），新增隔离单元逐份验、稀疏设备 journal 提示两条用例，改一批用例
- singlefs-core、singlefs-checker、singlefs-replay-prefix、singlefs-reference-filesystem 小改；crates/mutations.tsv 跟着改



### f111c3f3 2026-10-13 里程碑规划：排出里程碑四到十七，重排四、五两份，一到三的术语跟着改名

- 新增 06–17 共十二份里程碑：并发读写与并行提交、打包搬迁接并行提交、优化树结构和缓存、优化环结构和写放大、
  真盘上的 O_DIRECT 会话、目录与 POSIX 面、FUSE 挂载、多盘条带与设备集合变化、克隆与多可写头、加密、
  scrub 救援与单盘 ditto、格式冻结。
- 04 改成「共享与回收」（出口用户 2026-10-01 定），05 改成「小文件打包」；原来 04 的并发读写挪到 06。
- 01–03：「层 0」改称崩溃点重放，03 的并发读写稳定改转里程碑六。
- 提交前 doc-lint 跑过：里程碑这 17 份零处违规（整轮红在别的文件上）。


### f0b94fef 2026-10-13 README 与简报改为三语：英语为默认，中文原文挪到 .zh.md，新增日语 .ja.md

仓根 README 与 briefs/ 下三份简报各成三份：<名>.md 是英语，<名>.zh.md 是中文，
<名>.ja.md 是日语。中文正文原样挪过去（README.zh.md 的目录表补一句简报三语），三份顶上
各加一行同样的三语链接。编号的英、日简称写在半角括号里；英、日两版各由一个审校员对照中文逐段审过（英文 26 条、日文 34 条，全部改上）。



## 二、这一阶段新写的记录（文件名即题目；要知道做成了什么，读各份开头的课题与收尾表）

- records/2026-10-11-scripts审计-第二轮.md
- records/2026-10-11-scripts审计.md
- records/2026-10-11-singlefs-core严格审核.md
- records/2026-10-11-崩溃放量两段与脚本审计.md
- records/2026-10-11-指令文档删论证与来历.md
- records/2026-10-11-里程碑三项五收口.md
- records/2026-10-11-里程碑五打包与决策进度列.md
- records/2026-10-12-里程碑三工具与治理批.md
- records/2026-10-12-里程碑三非代码收尾.md
- records/2026-10-12-里程碑四关键信息调研.md
- records/2026-10-13-Verus证明审计.md
- records/2026-10-13-三语README与简报.md
- records/2026-10-13-代码欠账批.md
- records/2026-10-13-复用键与全仓改名.md
- records/2026-10-13-欠账还五条.md
- records/2026-10-14-全量前修红.md

## 三、HEAD 之后还没提交的改动（10-14 到 10-17，在工作区与暂存区；读 `git -c core.quotepath=off status --short` 与 `git diff --cached --stat` 现查）

- .claude/gate.d/fixtures（门禁样本，不治理）：721 个文件
- crates/singlefs-checker-tier/tests：36 个文件
- .claude/gate.d：26 个文件
- crates/singlefs-core/src：24 个文件
- scripts/gate：24 个文件
- crates/singlefs-checker-tier/src：23 个文件
- crates/singlefs-harness/tests：22 个文件
- scripts/experiments：20 个文件
- records：18 个文件
- .claude/kb/experiments：16 个文件
- crates/singlefs-unit-graph/src：16 个文件
- .claude/hooks：14 个文件
- crates/singlefs-checker-tier/src/bin：11 个文件
- crates/singlefs-unit-graph/tests：11 个文件
- .claude/kb：10 个文件
- scripts/crash：9 个文件
- crates/singlefs-harness/src：8 个文件
- scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes：8 个文件
- .claude/agents：7 个文件
- scripts/mutation：7 个文件
- .claude/kb/decisions：6 个文件
- .claude/rules：6 个文件
- scripts/kb：6 个文件
- .claude：5 个文件
- scripts/hosts：5 个文件
- scripts/mutation/tables：5 个文件
- crates/singlefs-checker/src：4 个文件
- scripts/experiments/e170_o3_spec_executor_with_witness_predicates_journal_header_487：4 个文件
- .claude/scripts：3 个文件
- crates/singlefs-checker/invariants：3 个文件
- crates/singlefs-unit-graph/tests/fixtures/wgsl_split_kernel/fsplitkernel/src：3 个文件
- litmus：3 个文件
- scripts/agents：3 个文件
- .claude/gate.d/lib：2 个文件
- .claude/kb/milestone：2 个文件
- .claude/skills/crash-test：2 个文件
- crates/singlefs-checker-tier：2 个文件
- crates/singlefs-unit-graph/tests/fixtures/wgsl_kernel/fkernel/src：2 个文件
- crates/singlefs-unit-graph/tests/fixtures/wgsl_split_kernel/funlinkedcommon/src：2 个文件
- scripts/devices：2 个文件
- scripts/experiments/e170_o3_spec_executor_with_witness_predicates：2 个文件
- scripts/experiments/e171_three_oracle_collusion：2 个文件
- .claude/skills/gate：1 个文件
- .singlefs-ai-sop-version：1 个文件
- CLAUDE.md：1 个文件
- Cargo.lock：1 个文件
- Cargo.toml：1 个文件
- briefs：1 个文件
- crates：1 个文件
- crates/singlefs-checker-tier/tests/common_allocation_record_trees_copied_under_other_generations：1 个文件
- crates/singlefs-checker-tier/tests/common_crash_point_observers：1 个文件
- crates/singlefs-checker-tier/tests/common_crash_points：1 个文件
- crates/singlefs-checker-tier/tests/common_given_states_of_harness_scenario_images：1 个文件
- crates/singlefs-checker-tier/tests/common_gpu_cards_of_the_caller：1 个文件
- crates/singlefs-format/src：1 个文件
- crates/singlefs-harness/tests/common：1 个文件
- crates/singlefs-harness/tests/common_admission_pool：1 个文件
- crates/singlefs-harness/tests/common_foreign_node_size_planted_in_system_configuration_slots：1 个文件
- crates/singlefs-harness/tests/common_pointer_head_corruption：1 个文件
- crates/singlefs-unit-graph：1 个文件
- crates/singlefs-unit-graph/src/bin：1 个文件
- crates/singlefs-unit-graph/tests/common_fixture_workspace：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/blanket_self_types/fselftypes：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/blanket_self_types/fselftypes/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/blanket_self_types/fslice：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/blanket_self_types/fslice/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fblanket：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fblanket/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fcfg：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fcfg/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fopen：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fopen/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/funderscore：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/funderscore/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fuse：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/closure_rules_shared_with_python/fuse/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fcore：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fcore/data：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fcore/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fharness：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fharness/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fpainter：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fpainter/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/ftier：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/ftier/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/funlinked：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/funlinked/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fverified：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/rule_cells/fverified/src：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/wgsl_kernel/fkernel：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/wgsl_split_kernel/fsplitkernel：1 个文件
- crates/singlefs-unit-graph/tests/fixtures/wgsl_split_kernel/funlinkedcommon：1 个文件
- multi-host.env.example：1 个文件
- scripts/experiments/heavy_threads_default：1 个文件

没提交的改动里文档类的逐个列：

- .claude/agent-common.md
- .claude/agents/experiment-designer.md
- .claude/agents/experiment-runner.md
- .claude/agents/implementation-writer.md
- .claude/agents/investigator.md
- .claude/agents/three-way-attack.md
- .claude/agents/three-way-defense.md
- .claude/agents/three-way-forward.md
- .claude/kb/INDEX.md
- .claude/kb/checks-owed.md
- .claude/kb/decisions-history.md
- .claude/kb/decisions.md
- .claude/kb/decisions/02-RAID条带策略.md
- .claude/kb/decisions/08-核心索引结构.md
- .claude/kb/decisions/13-验证路线.md
- .claude/kb/decisions/21-权威态与派生态的分界.md
- .claude/kb/decisions/22-单元原子性怎么合成.md
- .claude/kb/decisions/23-journal的角色与格式.md
- .claude/kb/experiments-history.md
- .claude/kb/experiments.md
- .claude/kb/experiments/102-单元类登记表与打包记录单元.md
- .claude/kb/experiments/139-按盘回退下界的收严形态.md
- .claude/kb/experiments/142-新池新建文件的干跑.md
- .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md
- .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md
- .claude/kb/experiments/158-择根与修复四岔路.md
- .claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md
- .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md
- .claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md
- .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md
- .claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md
- .claude/kb/experiments/168-删除借保留量的环窗口.md
- .claude/kb/experiments/170-O3规约执行器第一版.md
- .claude/kb/experiments/171-三oracle合谋测试.md
- .claude/kb/experiments/172-删除写出的字节的干跑.md
- .claude/kb/experiments/49-反向链宽度32还是64.md
- .claude/kb/milestone/02-second-txn.md
- .claude/kb/milestone/03-third-txn.md
- .claude/kb/pending-rules.md
- .claude/kb/term-renames.md
- .claude/kb/translation-glossary.md
- .claude/kb/verification-build.md
- .claude/main-agent.md
- .claude/rules/fs-design.md
- .claude/rules/implementation-workflow.md
- .claude/rules/mutation-sampling.md
- .claude/rules/test-script-discipline.md
- .claude/rules/three-way-inference.md
- .claude/rules/verification.md
- .claude/skills/crash-test/SKILL.md
- .claude/skills/crash-test/commit-stages.md
- .claude/skills/gate/staged-gate.md
- CLAUDE.md
- crates/singlefs-checker/invariants/02-根环.md
- crates/singlefs-checker/invariants/03-journal.md
- crates/singlefs-checker/invariants/09-inode树结构.md
- records/2026-09-13-总审核.md
- records/2026-09-16-subagent拆分提案.md
- records/2026-09-24-里程碑二收尾调度.md
- records/2026-09-28-agent体系用量时间与调度分析.md
- records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md
- records/2026-09-28-里程碑二收口表回写与系统配置槽写频率门禁.md
- records/2026-09-28-里程碑二收尾接手.md
- records/2026-09-30-里程碑三开工第一轮.md
- records/2026-10-01-删除与并发事务.md
- records/2026-10-01-对接上游测试脚本规范.md
- records/2026-10-01-里程碑三形式化接入.md
- records/2026-10-01-门禁注释精简.md
- records/2026-10-11-崩溃放量两段与脚本审计.md
- records/2026-10-11-里程碑三待优化上下游协调.md
- records/2026-10-13-三语README与简报.md
- records/2026-10-13-代码欠账批.md
- records/2026-10-13-复用键与全仓改名.md
- records/2026-10-13-欠账还五条.md

其中已知的事（主 agent 从记录与提交正文读到的，写事实表时自己现查）：
- 复用键与全仓改名（records/2026-10-13-复用键与全仓改名.md）：判定复用键改成只由函数单元指纹组成，相关的全仓改名
- 代码欠账批（records/2026-10-13-代码欠账批.md）
- 全量前修红（records/2026-10-14-全量前修红.md）
- 新规则 .claude/rules/test-script-discipline.md；新 kb 文件 .claude/kb/pending-rules.md
- 2026-10-17 复用键会话在跑崩溃点重放全量（HEAD + 暂存区的 worktree）

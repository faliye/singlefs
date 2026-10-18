# 留给用户拍板（草稿，写回员交回后补齐）

| # | 事 | 为什么不自己做 | 出处 |
|---|---|---|---|
| 1 | 定义与主 agent 入口这一轮的改动（`main-agent.md` 4 处、`experiment-runner.md` 1 处、`mutation-triage.md` 2 处、待改 `kb-scribe.md` 1 处）要逐份豁免三方 | 改定义要走一轮三方或用户逐份豁免（门禁 doc-process-records 的 agent-def-adversarial-review 格） | fixes-主.tsv |
| 2 | 产物归档出工作区后，实验页「整行抄自产物」的行写成 `git show <提交>:<路径> 第 N 行` 指针（13 页已这样写），`experiment-runner.md:43` 与上游 `evidence-discipline.md`「引产物就整行抄」还没登记这种写法；门禁也没判指针指不指得到（C837 仍欠那一半） | 改定义与上游规则 | S08、S11 |
| 3 | D18 已定项 11 进度「已实现」而它的前提「独占打开池中过半的设备」C833 说没做；D18:321 把它当现成保证 | 决策进度与射程 | 甲、S01 |
| 4 | D16 已定项 1 定案段「装置的候选下界还没带系统配置里的 F……欠里记着 E139 要重跑」已过时（第 2 次重跑已跑）；D16:127 与 milestone/01:185「C199（冻它之前要先还）」依赖还成不成立看 D15 | 定案段与依赖 | S03、S09、S19 |
| 5 | D5:77 射程「key 数量上升（每个统计量 × 每一代各占一条）」与 D8 已对齐成整批换代；D9:300 射程「两条没有代码在判」与 C406 第 4 格冲突；D23:453 依据句末「算不出不拒」是旧判法；D18:366「那一步仍开着，记在 C163」；D8:207「挡着新池新建文件写的九样」把 C71、C117 当前置；D14:130 | 决策射程 / 依据 / 定案的措辞（写回员乙交回的清单为准） | S01、S17、S06、S18、S15、S09 |
| 6 | D13 已定项 16 进度：HEAD「已实现」，10-16 被改成「未实现」，逐行判读代码判应为已实现 | 别的会话改的，不自行翻 | S04 |
| 7 | C709 前置「不挡里程碑三」与去向「里程碑三二·5」矛盾；C618 去向改里程碑十（按 10-13 规划） | 排期判断 | S14、S15、S28 |
| 8 | 里程碑规划互相登记缺一边：05 写「区域扫描（里程碑四出口 2）」、D27 写「D26 的区域扫描」，04 出口 2 与二·1 没有 | 规划 | S25 |
| 9 | `records/2026-10-14-全量前修红.md` 是 0 字节空文件、只有一次提交 | 写它的会话 | 丁乙 |
| 10 | `records/2026-09-16-subagent拆分提案.md:1061` 未定项「提交没有读 `refs/sop/staged-green` 的闸」要不要做（staged-green 现在只作记录） | 治理流程 | S20、S33 |
| 11 | milestone/02:393、:596「门禁改法之后待重跑」：E142 r25 之后 QEMU 门禁 mkfs 行有没有重真跑，仓里没记录 | 要起虚机 | S10、S12、S23 |
| 12 | 下一批改 `crates/` 的清单：`walk.rs:8510` C314 旧简称、`make_filesystem.rs:3` 注释 `12+1+1+1+4`、`e142_…dry_run.rs:5657`「层0」、`Cargo.toml:22` 旧规则路径、变异表第 89 行与 C592 行今天不红（10-11 记的） | 改 `crates/` 归实现员 | 丙丙、S10、S12、S36 |
| 13 | 下一批 tooling-writer 的清单：`stale-candidates.py` 载体模式不读 `.claude/kb-directories`（并重钉阳性对照行数）；`lib-item-ref-status.py` 扫描范围 `research/**/*.rs`；`implementation-writer.md:41`「层 0 流」字面串与 `apply-writer-patch.py:78` 同改；`write-frequency.tsv:48`；`doc-process-records.sh:57` 注释与 `.claude/batch-scope:161-162` 仍登记 `ask-local.sh`；`lib_heavy_tests.py:3653` 点名已删的格；`scripts/gate/README.md` 没提 `lib_reuse_checks.py` | 改脚本与定义 | 多段 |
| 14 | `records/` 里按批次写的收拢表、去向表「等 X / 下轮开」而 X 后来做了的共 29 行（S01 5、S03 5、S07 7、S30 若干等），主 agent 按「批次快照不回改」留着；`records/` 里只按欠账编号搜到的 2309 行同样按批次快照不进逐行判 | 口径由主 agent 定，报给用户知道 | 调度记录 |
| 15 | 写回乙交回的三块改动里主 agent 不写的 5 处：D16:29 已定项 1 定案句「装置的候选下界还没带 F……欠里记着 E139 要重跑」（E139 第 2 次重跑已带 F，仍欠对不上的取舍 C660）；D18:366「那一步仍开着，记在 C163」指改键还是指检查分不清；D23:453 已定项 14 依据句末「算不出不拒」是旧判法（另一会话在写 10-17 定的「比 fsid」，等它合）；D27 已定项 4 射程第三条「槽要带哪些字段随已定项 3 …一起定」与欠块「无」矛盾；D28:99「一个根槽持续读不出时行只增不减」是 C335 已关账说「题面不成立」的那一半 | 定案 / 射程 / 依据的实质措辞 | writeback-乙.md「交主 agent」第 11、15、16、17、18 条 |
| 16 | C605「先定版本表进不进键」是仍开着还是已被 D13 已定项 17 覆盖（要对 `crash_identity.rs` 现查）；C568、C750、C572、C793 验收看着已满足要不要挪已还清 | 挪已还清要证据判断 | writeback-甲.md |
| 17 | `crates/singlefs-unit-graph/` 归哪一侧：`implementation-workflow.md:10` 的代码轮射程清单与 `verification.md:8`「四个包」都没归类，门禁 `doc-process-records.sh:327` 的 `TEST_CODE_SOURCES` 只去掉三个测试包、把它当产品代码，它的 `Cargo.toml` 却写「测试基础设施库」 | 规则与门禁要一起定 | R1 |
| 18 | `implementation-workflow.md:66`「checker-tier-crates-mutation-replay 这道门禁的整套重设计归别的会话」（2026-10-03 写的）今天还成不成立 | 查不到那个会话的去向 | R1 |
| 19 | 第二轮通读在定义里补的做法（范围声明闸、定时闸、合入后验证只跑改到的包、合表点名）也要逐份豁免三方（与第 1 条同一批：`agent-common.md`、`main-agent.md`、`implementation-writer.md`、`three-way-attack.md`、`tooling-writer.md`、`kb-scribe.md`、`mutation-triage.md`、`experiment-runner.md` 八份） | 定义改动 | R2 |

## 用户定了（2026-10-17 弹窗两次）

| # | 定了什么 | 落地 |
|---|---|---|
| 1、19 | 定义改动走一轮三方（不逐份豁免） | 主 agent 开定义轮 `rot-defs-r1`（代码轮形态），判决落 research/prompts |
| 2 | 指针写法登记进定义与上游规则 | `experiment-runner.md` 第 7 步已加；上游 `evidence-discipline.md` 等发版会话回话后填内容 |
| 3 | D18 已定项 11 进度降未实现、欠块补 C833、前提句改「待 C833」 | 已改，变更史加一项，`--write` 重生成 |
| 4、17 | `singlefs-unit-graph` 归测试代码一侧 | `implementation-workflow.md` 清单与 `verification.md` 包表已加；门禁 `doc-process-records.sh` 的 `TEST_CODE_SOURCES` 归 tooling-writer 下一批 |
| 5、15 | 四处决策实质句开一轮三方 | 主 agent 开推论轮 `rot-decisions-r1` |
| 6 | D13 已定项 16 进度保留未实现，问改它的会话 | 已问跑全量的会话 |
| 7 | C618 归里程碑十；C709 去向改「不挡、未排」 | C618 写回甲已改；C709 已改 |
| 9 | 空文件 `records/2026-10-14-全量前修红.md` 删 | 已删（未暂存） |
| 10 | staged-green 未定项撤 | `records/2026-09-16-subagent拆分提案.md:1061` 已记撤销 |
| 12、13 | 下一批（crates 5 条、tooling 8 条）等全量跑完再派 | 记录「收尾」表 |

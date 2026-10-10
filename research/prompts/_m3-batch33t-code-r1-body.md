# 删除批第 33 批 T（core 那一簇）合入的代码：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 T1 T2 T3 T4 T5 T6 T7 T8 U1 F1 F2 S1 S2 R2 A B C -->

## 一、题面

删除批第 33 批 T 2026-10-07 合进主工作区（`records/2026-10-01-删除与并发事务.md` 同日那几条）：照设计轮两份判决 `research/prompts/m3-c694-r1-main-verification.md`、`research/prompts/m3-c694-r2-main-verification.md`（第三节是改法）与用户 2026-10-07 定案（C692「按几何现算」、C694「C 与 B 都做」、C695「先修次序，选哪一版归 C683」、F1「乙：缺记录就拒可写」、F2「乙：三处都做」）实现 core 那一簇。条款已写回 `.claude/kb/decisions/16-发布语义.md` 已定项 1、`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14、`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 与 `.claude/kb/checks-owed.md` C692、C694、C695、C746。这一轮判两件事：代码做的是不是条款与判决定的改法；哪一格会错。这是形式化线拍最终冻结树之前最后一簇改 core 的批。

改动全文在 `research/prompts/_m3-batch33t-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch33t-code-r1/tree/`（T 合后那一版）。被判文件的 sha256 在 `research/prompts/m3-batch33t-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch33/t/report.md`，第五节是它停下交主 agent 的十条。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定的改法 |
|---|---|---|---|
| T1 C 预隔离 | 可写挂载读阶段之后把水位之上、没施加的全部 journal 记录点名的落点（含记录新根段四条指针的位置条目）隔离；只在这个进程写掉那条记录所属 txg 的根环槽、那次发布取完落点之后解开；进程内重读读出被抛弃的根那一支不解；跟着只供测试的影子账开关走 | `crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`（`RootRingOccupancy`、`record_root_written_by_this_process`） | D16 已定项 1「根槽这一次读坏」那一行；D23 已定项 14 预隔离那一段 |
| T2 B 凭据放行 | 被抛弃根的账读不出时，现行账里有一条代大于那条根 txg、罩住它树表槽的记录就放过 | `mount.rs` 的 `isolate_slots_referenced_only_by_abandoned_roots` | D23 已定项 14 影子账那一段 |
| T3 D28 第九项 | 字段改名 `isolated_for_abandoned_roots_and_unapplied_journal_records`，读法不变 | `crates/singlefs-core/src/admission.rs` | D28 已定项 1 第九项 |
| T4 F1 | 水位之上一次没施加的发布，读得出的记录序号不连续或缺末条、而任一点名单元验不过，重读一次仍这样就拒可写（新成员 `PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`） | `mount.rs` | D23 已定项 14；I-8.9 |
| T5 F2 | 所选根本身走读失败、系统配置没见证它时退回见证到的那一版；一份判法 `version_witnessed_by_the_system_configuration_to_fall_back_to`，`recover`、可写挂载、只读挂载三处调它；见证判不出（R2）时不退；退回那一版不重放 | `crates/singlefs-core/src/recovery.rs`、`mount.rs`、`crates/singlefs-core/src/mounted_read.rs` | D23 已定项 14 退回见证到的那一版那一段与射程里用户接受的代价 |
| T6 C695 | 映射根全零时先判树表 0 条，报 `TreeTableHasNoEntries` | `mounted_read.rs` | C695；第一轮判决第二节 K3 行 |
| T7 C692 | 新池新建文件每盘落点下限按系统配置读出的几何现算（默认几何 14、1024 MiB 环 16） | `recovery.rs` | C692 |
| T8 注释与测试 | `mount.rs` 第 1475 行一带注释改成今天的行为；新 harness 测试 6 份、改已有 harness 测试若干与三份 harness 源码的穷举臂；变异追加 18 行、按名字替换 20 行 | `crates/singlefs-harness/` 下、`crates/mutations.tsv` | — |

**主 agent 已定的读法**（实现员第五节第 1–4 条，被攻过零轮，腿可攻）：F2 只在所选根本身走读失败时退（由记录重建的那一版不退）；见证判不出时不退；C 跟着只供测试的影子账开关走；C533 那条用例改结局与名字。

**这一轮不判的**：C746（进程内重读读出被抛弃根时全零槽的钉同样解开，记欠账）；checker 档测试（提交时由主 agent 跑）。

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 合入验证「✓ 测试全绿（175 个测试二进制）」「✓ fmt 过」「✓ clippy 过」（日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-08-84183/`）；`clippy --all-targets --all-features` 罩住形式化线的 E171 装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`（对 `RefusalConditionNotMet` 穷举 match），编得过；全表 `crates-mutation-rows.py static-check .` 退 0（`CHECKED 3174 310 9`）。冻结副本取在 T 合后、根环线 10y 合入之前，22 份 `sha256sum -c` 全 OK。
- 实现员副本上：证红 18 行全「抓到」；harness 档 65 条慢档用例没跑完（主 agent 合后另跑）。
- 实现员推的、没量：C692 下限在两条一槽记录合成一个两槽落点时可能误判红（报告第五节第 9 条）；C 剩下一形盲区（第 8 条）；F1 在已有用例与随机历史上拒 0 次（从全绿推）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（T1–T8）
- T1：隔离的集合与解开的时刻是不是判决定的 `all` 与 `ring`；C 与影子账、抬 F 那一串按新候选集重算、D28 第九项是不是同一个集合同一个数。
- T2：凭据只读内存里的现行账、不读那一槽；瞬时读不出那一形照拒。
- T4：「序号不连续或缺末条」与 I-8.9（序号 1..N、只有末条带标志）是不是同一个判法；「任一点名单元验不过」与正常崩溃在写记录中途（单元在记录之前落盘、验得过）分得开。
- T5：三处是不是同一份判法；「见证过」的判法与 D23 已定项 14 写下的一致；退回之后各份报告（`recover` 的 `outcome.root` / `effective_root`、两种挂载的 `chosen_root` / `effective_root`、`journal` 扫环报告）与读它们的地方（记录核对器、层 0 oracle）一致。
- T6、T7：照 C695、C692。
- T8：变异行锚点与替换的 20 行名字、参数、必须红的测试照旧；新测试名照 `.claude/rules/verification.md`「定义与名字」。

### K2　哪一格会错
在冻结副本的拷贝上攻，只跑 harness 档（checker 档用例不跑；要判的用 harness 档造盘面、调 core 的公开入口）；用户决定的那几步放开扫，只固定前缀与故障：
- T1：一形 C 放错（该钉的放了、或钉住不该钉的导致假 ENOSPC 无界）；C「水位之上」按 (实例, txg) 实例为主比，F2「见证过」按 (txg, 实例) 比，恢复落到隔两版的旧根、新实例 txg 低于被抛弃根时两种次序分叉；C 剩下的一形盲区：根槽与末条记录读回全零、单元读得出（F1 不拒），末条单独点名的单元 C 看不见。
- T4：正常崩溃（写记录中途断电）或合法的多记录发布被 F1 拒可写的一形。
- T5：F2 让一个本该报读不出的盘面读回旧版的形，超出用户已接受的那一形（已确认的版本在较新系统配置槽与它的单元同时读回全 0）之外的；退回之后层 0 oracle 或记录核对器判错的形。
- T7：一份合法镜像在新下限附近误判红（例：两条一槽记录合成一个两槽落点，记录少一条，`reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_rewrites_the_first_and_deletes_the_second` 那一形）。

### K3　被判的文件名单
`research/prompts/m3-batch33t-code-r1-snapshot/crates-sha256.txt` 里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决定的改法推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch33t-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与脚本放 `research/prompts/m3-batch33t-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `research/scripts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G；harness 大测试二进制 8G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、去掉耗时字段、自己不依赖 `SINGLEFS_REPO`，用法写在文件头第一行；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。

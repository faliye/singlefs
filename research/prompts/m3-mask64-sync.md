<!-- knowledge-sync -->
# m3-mask64 阶段同步

触发文件：CLAUDE.md、crates/singlefs-checker-tier/src/crash_amplification.rs、crates/singlefs-checker-tier/src/crash_facts.rs、crates/singlefs-checker-tier/src/crash_judge_common.wgsl、crates/singlefs-checker-tier/src/crash_judge_dispatch.rs、crates/singlefs-checker-tier/src/crash_judge_tables.rs、research/scripts/e152-tables.py

这一阶段做成的事：判器一个落点的组合掩码从 32 位放到 64 位（`MAXIMUM_WRITES_PER_LOCATION` 64、`JUDGE_TABLES_VERSION` 3），崩溃放量全量按新判法版本重判七项（含 σ）零红；`e152-tables.py` 在六家那六张表末尾加 singlefs 一行；checker 档几处 `shadow_unrelated` 改名（行为不变）；CLAUDE.md 状态行改成七项零红。主 agent 自己回扫（这个会话用户定不派子 agent）。

## 搜索

命令与全部命中存在 `research/prompts/m3-mask64-sync-search.txt`，每条排除 `target`、`.git`、`research/prompts`、`research/results` 与两份变更史：

- `grep -rn -e '32 次写'` → 4
- `grep -rn -e '落点 32'` → 1
- `grep -rn -e 'MAXIMUM_WRITES_PER_LOCATION'` → 4
- `grep -rn -e '组合掩码'` → 9
- `grep -rn -e 'σ.\{0,20\}没判'` → 1
- `grep -rn -e '26\.15 亿'` → 0
- `grep -rn -e '六个分项'` → 1
- `grep -rn -e '判 46 条'` → 3
- `grep -rn -e '判 48 条'` → 2
- `grep -rn -e 'JUDGE_TABLES_VERSION'` → 4
- `grep -rn -e 'e152-tables'` → 20

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| CLAUDE.md:8 | 展开 28 全枚举六个分项 26.15 亿态零违例（到 E 全域 1 662 648 564 态），σ 那一项 GPU 判器一个落点 32 次写的容量不够、没判。 | 改了：一个落点的组合掩码放到 64 位之后按新判法版本重判，七个分项（含 σ）26.16 亿态零违例，一趟 7 小时 37 分 |
| README.md:45 | \| checker \| 判 46 条不变量，数法见表后那条命令 \| | 改了：判 49 条（`grep -c '^\| I-.*已实现' .claude/kb/invariants.md` 现数 49）；表里另加崩溃放量一行 |
| briefs/2026-09-28.md:85 | \| σ（错位复用） \| 65 548 \| 那一趟没判：…全部 7 项按新版本重判中 \| | 改了：全量一节换成 2026-09-30 那一趟的逐项结果与用时，σ 0 红、GPU 判 |
| briefs/2026-09-28.md:91 | 2026-09-29 那一趟（旧判法版本）同样六项零红，σ 报错退出、没判 | 不改：说的是 2026-09-29 那一趟发生的事 |
| records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:37 | 全量 7 项按新判法版本清场重判（在跑） | 改了：已跑完，七项 0 红（见「全量 i」那一行） |
| records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35 | （新增） | 补了：「全量 i：新判法版本七项全绿」一行，逐项状态数、用时、两处慢 |
| .claude/kb/checks-owed.md:476 | C601 判器查版本不比组合掩码高 32 位时没有会红的流 | 不改：说的是 64 位之后高 32 位还没有会红的流，现状句 |
| crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1013 | 它是今天唯一一条一个落点超过 32 次写的流 | 不改：说的是那条流超过旧容量，测试要的就是它，现状句 |
| research/e7-index-bench/src/bin/e74_allocation_records.rs:34 | 后台整理的形态本身还没定（D26 六个分项全开着） | 不改：与崩溃放量的分项无关，说的是 D26 |
| records/2026-09-27-同步远端冲突块.md:1237 | 判 46 条不变量 | 不改：那份记录记的是 2026-09-27 那一天的状态 |
| records/2026-09-24-里程碑二收尾调度.md:232 | 判 46 条 | 不改：那份记录记的是当时 README 那件活 |
| CLAUDE.md:109 | checker 判 48 条不变量 | 不改：列举与计数早就对不上（列第一版 23 条加 20 条、说 48，现数 49），要照 `.claude/kb/invariants.md` 逐条重核名单，不在这一批 |
| .claude/kb/milestone/03-third-txn.md:61 | 判 48 条 | 不改：别的会话在写的里程碑三规划，数法同 CLAUDE.md:109，随那一处一起重核 |
| research/perf-by-milestone.md:7 | 第二节与第二·二节的表由 `python3 research/scripts/e152-tables.py <产物>` 从产物生成，不手抄。 | 不改：用户 2026-09-30 定 E152 的整理只放 E152 页，这份不动；脚本多出的 singlefs 行在这份的表里没有，下次重出这份的表时一起带上 |
| .claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:630 | （新增） | 补了：「口径」一节写明 singlefs 那一行每格取什么量 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 回扫命令与全部命中 | research/prompts/m3-mask64-sync-search.txt | 60 |

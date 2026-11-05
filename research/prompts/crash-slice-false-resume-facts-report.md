# 阶段同步第一段（写事实表）报告：crash-slice-false-resume

基准 `59ce5a865`，结束 `7ac20c0204a7587b580a8eb2d78c7efead4e873f`。没改任何被搜到的文件，没动 `stale=`，只写了草稿目录。

## 产物路径

- 变更清单：`/home/fy5090/.local/share/singlefs/work/cpr-sync-facts-sweep/changes.md`。0 条变更记录（没有 H 编号），「现状载体里改掉的片段」也是空的（载体里这一批只有新增、没有删改的现状句），所以出处列写提交与调度记录节名。
- 事实表：`/home/fy5090/.local/share/singlefs/work/cpr-sync-facts-sweep/facts.tsv`，25 行 F1-F25。
- 候选表：`/home/fy5090/.local/share/singlefs/work/cpr-sync-facts-sweep/candidates.tsv`，654 行数据（加表头 655 行，599 个不同的行）。
- 出处列不写文件路径，只写提交和调度记录节名（`--check-facts` 对已跟踪且工作区有别会话改动的路径判红，调度记录本身在暂存区）。

## 两条命令输出末行（原样）

```
  ✓ 事实表罩全了：0 条变更记录都有出处，25 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）
  ✓ 候选表 /home/fy5090/.local/share/singlefs/work/cpr-sync-facts-sweep/candidates.tsv：654 行（599 个不同的行）
```

两条都退 0。

## 每个检索词的命中行数（基准 / 结束）

没有一行超过 100，不需要「宽词已抽查」。

| 行 | 基准 | 结束 | 候选分布 |
|---|---|---|---|
| F1 | 67 | 83 | kb 12、implementation-workflow.md 2、records 69 |
| F2 | 85 | 92 | kb 21、main-agent.md 1、verification.md 1、records 69 |
| F3 | 13 | 21 | kb 4、records 17 |
| F4 | 33 | 39 | kb 4、verification.md 1、records 34 |
| F5 | 16 | 19 | kb 9、records 10 |
| F6-F9（新立） | 0 | 5 / 9 / 10 / 13 | 不出候选 |
| F10 | 11 | 14 | kb 9、crates 下不变量清单 2、records 3 |
| F11 | 65 | 68 | kb 20、README、implementation-writer.md、verification.md 3、crash-test/SKILL.md、records 42 |
| F12 | 64 | 68 | kb 15、verification.md 1、records 52 |
| F13 | 26 | 28 | 全 records |
| F14 | 27 | 39 | README 1、verification.md 1、records 37 |
| F15 | 25 | 29 | 全 records |
| F16 | 16 | 19 | kb 3、implementation-workflow.md 1、verification.md 1、records 14 |
| F17 | 27 | 33 | agent-common.md 1、main-agent.md 3、timer-only-remind.md 1、records 28 |
| F18-F22（新立） | 0 | 4 / 4 / 1 / 2 / 3 | 不出候选 |
| F23（新立） | 0 | 8 | 不出候选 |
| F24 | 63 | 65 | main-agent.md 4、records 61 |
| F25 | 34 | 37 | kb 7、agent-common.md 2、records 28 |

F1 与 F2 有少量重叠候选（`CRASH_CASE_BLOCKS` 与「导入那一趟」常在同一行），逐行判时各按自己的新旧事实判。候选里 records 占大头，多半是历史事件句。

## 七件做成的事对到哪几行

1. 双机分片假续跑根因修了：F1（导入那一趟不早退、观察者红计入）、F2（快档块查询与 CRASH_CASE_BLOCKS 罩整条流）、F3（imported_blocks= 三行同为块数）、F4（GPU 判器组按路线取）、F5（admission.py blocks=0 退 3 / 退 4、crash_case_launch 清两变量）。
2. 四条新用例写实：F6 A（10_728_767_988）、F7 B（289_472_738）、F8 C（68_157_463）、F9 D（16_920_659，拒写断言不再说「0 写」）、F10 record_spill_over 文件去向、F11 登记条数（stage-inputs 47 增到 51，items 45 增到 49，积压表加 73-80 号）。简报 briefs/2026-10-28 三语不是现状载体，包在 F6-F9 的新事实里。
3. 变异行：F12（轻型登记表 285 增到 298，55 行逐条证红）、F13（三行改名写实）。
4. --peer 拒带库：F14、F15、F16。
5. watch：F17（含真场景叫醒一次）。
6. 新立欠账：F18 C924、F19 C925、F20 拆分提案经 --peer 整件起带库（原编号一百四十八）、F21 崩溃入口在共用主工作区上跑源码被改（改号中，按标题认）、F22 第一百五十节、F23 键变后切片假续跑与不起全量重跑的决定。检索词在基准全部零命中。
7. 第一次真用：F24 合成总补丁给 merge-and-verify.sh --skip-apply、F25 崩溃轻型在固定树上重跑、F17 watch 新判据叫醒；用户 2026-10-29 定「1 不要 2 要 3 要」写在 F23。

## 要主 agent 先看的疑点

1. 结束提交删了 crash_enumeration_record_spill_over_stream.rs，工作区里它还在。`git diff 59ce5a865 7ac20c02 --stat` 里它 472 行全删；主工作区（暂存区与工作区）里它是 HEAD 的 L8 流（一个事务跨两条记录）加 328 行改动，里面是批次 2、5 的端到端用例（an_observer_red_on_the_blocks_the_other_host_owns_reddens_both_its_share_and_the_import_run 等）。three_generation_mount_fork 是独立的 875 行新文件，git diff -M 没认出改名，结束提交里它只有 3 处命中这类词，批次 2、5 的端到端用例没带进去。派发写「改名自 record_spill_over」，调度记录批次 2、5 写的是把端到端用例放进 spill_over 那份，两者对不上。像是临时索引把仍在工作区的文件当成了删除，提交前请核。F10 的新事实照提交对象写并注明待核；kb 里 C37 与 decisions/23-journal的角色与格式.md:216 点名这个文件当 L8 流所在，F10 会带出来。
2. 结束提交里混进别的会话的改动，事实表没为它们开行：拆分提案 +31 行里除你列的三节外还有三节（一百四十九「形式化门禁只能在本机跑」、一百五十一「快档开跑前逐条算单元键单线程」、另一节一百四十八「形式化轻型的证明那一步只用一两核」），文件里因此有两节一百四十八、两节一百四十九；crates-mutation-light-rows.tsv 的 +13 行里有 C874、C724、C928、七·3 GPU 补跑、m4 截断、部分写、e197 等别会话的行；积压表 65-72 号也不是本批的。F12 的「285 增到 298」是整表数，含这些行。
3. 结束提交里 14 处冲突标记在 .rs 与 .tsv，不是现状载体，没影响事实表；积压表 73-80 号按冲突标记之后的内容读。
4. 候选表的载体范围由脚本定：CLAUDE.md、README.md、agents、rules、skills、records、kb 与不变量清单；crates 下 .rs、briefs/、gate 的 tsv 不在内。
5. 旧名 nearly-full-pool-ro-fallback 在基准的现状载体里 0 命中，D 用例按新立写，旧名放进检索词。

## 交实现员

0 行。本段只写事实表，没有逐行判。数行命令：`awk -F'\t' 'NR>1 && $4 ~ /^crates\//' candidates.tsv | wc -l` 得 2，这 2 行是不变量清单（F10），按文档行走写回员，留到逐行判那一段。

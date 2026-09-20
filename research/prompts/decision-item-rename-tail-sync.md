<!-- knowledge-sync -->
# decision-item-rename-tail 阶段同步

阶段：决策瘦身把分项标题从「已定三」这类改成「已定项 N」之后，注释、文档注释与记录正文里引旧写法的地方跟上（2026-09-20）。基准 `7d2d25c`，提交 `2888f8d` 与这一笔，时间 UTC。
触发文件：crates/singlefs-harness/src/crash.rs、research/scripts/e125-zoned-wp-probe.sh
同一笔里另有七份 `research/e7-index-bench/src/bin/*.rs` 与两份 `records/2026-09-13-*.md`，不在 68 号的触发范围里，改法与这两个一样：只改注释里的分项号，没碰任何逻辑。

## 搜索

旧写法全仓搜（不含 `research/prompts/` 的冻结证据）：`grep -rl 已定三|已定一|已定二|已定四|已定五|已定六|已定七|已定八|已定九|已定十 .claude crates research records README.md` → 「已定三」6 份、「已定一」6 份、「已定二」5 份、「已定六」2 份、「已定七」1 份，其余四种 0 份。
逐份读过：除下面那一处外，全部落在 `.claude/kb/decisions-history.md`、`.claude/kb/decisions-history/2026-0[89].md` 与 `.claude/kb/experiments-history.md` 里，写的是「改前引的是旧小节名、改后改成分项号」这件事本身，是事件句，改了反而成假话。
锚点没受影响：`bash .claude/gate.d/33-mutation-tables.sh` → 142 张实验变异表 1509 条与 `crates/mutations.tsv` 173 条的原文各命中源码一次；`doc-lint` 443 项通过。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/checks-owed.md:86 | D22（单元原子性怎么合成）已定三要「整单元校验和 + 代号 + 槽轮换」，而没有一条写过它 | 改了：C79 的题面引的是旧小节名，改成 `D22（单元原子性怎么合成） 已定项 21`（正文里那一条逐字是「原地覆写的结构必须「整单元校验和 + 被实际检查的世代号」」）|
| crates/singlefs-harness/src/crash.rs:1 | 记录核对器（D13（验证路线） 故意不给编号；入参 (崩溃前镜像, 记录流, 崩溃后镜像)） | 改了：改成 `D13（验证路线） 已定项 7，故意不给它编号` |
| research/scripts/e125-zoned-wp-probe.sh:1 | # - D22「根落在同一类，且两类布局需要的是同一种机制」： | 改了：旧小节名改成分项号 |

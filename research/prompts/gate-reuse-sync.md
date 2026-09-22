<!-- knowledge-sync -->
# gate-reuse 阶段同步

触发文件：.claude/gate.d/54-layer0-replay.sh、.claude/gate.d/55-qemu-first-transaction.sh、.claude/gate.d/59-crates-mutation-replay.sh、.claude/gate.d/74-model-differential.sh、.claude/gate.d/87-replay.sh、.claude/gate.d/stage-inputs.tsv、.claude/rules/implementation-workflow.md、research/scripts/change-touches-crates.sh、research/scripts/gate-staged.sh、research/scripts/stage-must-run.sh、research/scripts/stage-must-run-selftest.sh

这一阶段做成的事：门禁阶段能复用上一次整轮全绿的判定（新立 `research/scripts/stage-must-run.sh`，判据是「这一道读的那几条路径，在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间，git 说变没变」）、每道阶段读哪几条路径登记进 `.claude/gate.d/stage-inputs.tsv`、新立外壳 `research/scripts/gate-staged.sh` 负责整轮全绿时前移那条 ref、门禁 59 号第一次有了范围判定、54 / 55 / 74 / 87 四道多了一道更准的前置而 `change-touches-crates.sh` 降为第二道闸、新立欠账 C477（暂存的是整份文件，别的会话的改动跟着进来）。

## 搜索

事实表 7 行（一件做成的事一行），候选表 18 行。检索词与命中数（`current_state_corpus` 口径，基准 `896b73f` → 暂存树）：

```
F1  上次跑绿              基准 2  结束 2   候选 2 行
F2  那一道读的每个路径      基准 1  结束 1   候选 1 行
F3  gate-ok               基准 7  结束 7   候选 7 行
F4  变异表复跑             基准 6  结束 6   候选 6 行
F5  change-touches-crates 基准 1  结束 1   候选 1 行
F6  有意取别的基准          基准 1  结束 1   候选 1 行
F7  C477                  基准 0  结束 1   候选 0 行（「新立：」的行不出候选）
python3 research/scripts/stale-candidates.py --check-facts research/prompts/gate-reuse-facts.tsv --base 896b73f --target <暂存树>
  → ✓ 事实表罩全了：0 条变更记录都有出处，7 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）
python3 research/scripts/stale-candidates.py --check-report research/prompts/gate-reuse-candidates.tsv research/prompts/gate-reuse-judge-all.md --groups F1,F2,F3,F4,F5,F6
  → ✓ 报告判全了：18 行候选都有逐行判定
```

逐行判：**要改 7 行、不相干 8 行、事件句不改 3 行**。主 agent 逐行全看全部同意、一处改判都没有，写在 `research/prompts/gate-reuse-judge-main.md`；引文逐字比对 18 行 0 行对不上。

**这一轮返工过两次，两次都是主 agent 的错，记在这里**：① 写事实表那一段是在一棵**被污染的暂存树**上算的——主 agent 用 `git add <文件>` 往两道门禁阶段各加十行，把另一个会话未提交的 284 行与 156 行一起卷了进去；回扫发现「登记的理由与 diff 体量对不上」并停下来报，没有替那些改动编事实。② 修那两份的执行位（`git update-index --cacheinfo` 不从磁盘读文件模式）又换了一棵树，逐行判那一段核 `git write-tree` 对不上就停下来报。两次都靠派发提示里那句「对不上就停下报我」拦住。

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 事实表 | research/prompts/gate-reuse-facts.tsv | 8 |
| 候选表 | research/prompts/gate-reuse-candidates.tsv | 19 |
| 逐行判定报告 | research/prompts/gate-reuse-judge-all.md | 52 |
| 主 agent 逐行全看 | research/prompts/gate-reuse-judge-main.md | 28 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/checks-owed.md:22 | 摘掉它要记每道阶段上次跑绿时的输入指纹，而指纹得落盘、会腐化、新 clone 拿不到，这一半照旧欠着 | 改了：这三条反对理由这一批全部不成立——指纹是 git 自己算的树、不另外落盘；git 对象不会腐化；新 clone 没有那条 ref 时退回全跑，是安全方向。改写成已落，并写明仍欠的是 C8（门禁范围判不出来） 正题那一半（按布局算受影响集合，这一批给的是按阶段算输入，粗得多） |
| .claude/kb/checks-owed.md:22 | 接进门禁 54、55、74、87 号四道重阶段 | 改了：补半句写明 2026-09-22 起它降为第二道闸，这四道先问 `stage-must-run.sh` 按登记的具体路径比对，判定要跑时才轮到它再判「碰没碰前缀」 |
| .claude/rules/implementation-workflow.md:37 | 逐个路径现查，输出不截断；说不全那一道读什么，就没有复用的资格 | 改了：登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对，不必再手工逐个现查；没登记的阶段仍按原句办 |
| .claude/kb/checks-owed.md:397 | 68 号判据 ⑤ 同理 | 改了：豁免表加第三种——这五道用 `refs/sop/staged-green`，是与共享 `diff_base`、项目 `GATE_BASE` 都不同的第三套算法；不列进去，C449（两套基准算法让带不带 --staged 判出不同的改动范围） 那道检查会把它们误判成「两套基准不一致」 |
| .claude/kb/checks-owed.md:438 | 判别力自证三种情形：`gate-ok` 从未写过时说「从未写过」、存在时报出它指向哪、退出码非 0 时一个字不打 | 改了：补一句限定。正文那半仍真（上游 `scripts/gate.sh` 照旧不动 `refs/singlefs/gate-ok`），但本仓的外壳整轮全绿时另外前移 `refs/sop/staged-green`，所以那条简称单独摘出来读已经不对 |

## 一句要交上游的

共享 `gate.sh` 的阶段 3「构建与单测」接不进这套判据：它在上游，本仓的 `.claude/scripts/gate.sh` 与 `check.sh` 都只是 `exec` 转发，中间没有钩子。2026-09-22 实测那一道的 `cargo test` 合计 557 秒，而同一批暂存的 27 个路径里 `crates/`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain` 一个都没有。形态与这一批落下来的一样：读 `refs/sop/staged-green` 与这次的暂存树比对那几条路径，没变就退 77 并说明复用的是哪一次。连同「`gate.sh` 没有只重跑上一趟红的那几道的模式」一起交发版会话。

# 崩溃点重放拆成「跑数据 / 对比数据」两段：第二轮判决（2026-10-10）

被判：第一轮判决 `research/prompts/m3-run-compare-split-r1-main-verification.md` 第二节逐格判与第三节七条修订形态（零轮）。腿：辩方 `m3-run-compare-split-r2-sonnet-output.md`（sha256 2ca8d3dc6b00f2371b6db0e470ffc2bebdde99204fa5cd9b866bdd0f3e466f21）、攻方 `m3-run-compare-split-r2-opus-output.md`（sha256 dd833f6f7ca3f3c3327e797b08de548acff7c13a13b676998fa1a5999db14223）。第二轮打中的仍先挂起，第三轮（最终轮）攻本判决第三节的形态之后再定。

## 一、证据核对（主 agent）

- cite-check：`python3 scripts/agents/cite-check.py <两份报告> --root . --root ~/.local/share/singlefs/work/m3-merge-impl/e5/repo --background research/prompts/_m3-run-compare-split-r2-background.md` 末行「✓ 核了 46 处引文，对上 46 处，没判 0 处（逐处列在上面）」。
- 开工快照：`sha256sum -c research/prompts/m3-run-compare-split-r2-snapshot/kb-sha256.txt` 全部 OK。
- 我在 E5 副本（主工作区 + V…E5，与两腿同一份底子）上现查过的代码行：
  - `crash_amplification.rs:9271`（J6 对照调 CPU 判器）、`:9148`（GPU 判不了退回 CPU）、`:9142`（CPU 那份按 `CpuJudge` 入库）。
  - `tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:188`（`PerStepChecker::Skipped`）、`:371`（种子与规模串手写、不带执行参数）。
  - `crash_amplification.rs:8256`（H8 那句注释）。
  - `crates/singlefs-reference-filesystem/src/lib.rs:463`（重起头时把读出推进版本表）。
  - `crates/singlefs-checker-tier/src/layer0_progress.rs:15`（按 `slice_index % n` 分台）、`crash.rs:2537`（`LAYER0_RESUMABLE_SLICE_COUNT` 65 536）。
  - `crash_verify_gpu.rs:316`（`gpu_verifier_digest`）、`crash_identity.rs:1077`（自检点名它）。
  - `crates/singlefs-harness/src/history.rs:3878`（「模型对不上之后不再问模型」）。
  - `runs/` 下三趟 `CRASH_AMPLIFICATION_HOST_WEIGHTS` 三组值各不相同：136700,45275、140427,62390、642805,291558。
  - `~/.local/share/singlefs/crash-amplification/full-run-20260930-i.log` 第 207、225、237 行 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels` 三行 `exhaustive=false`。
- 辩方「E5 的 `tests/` 下已有 14 份文件按 `cfg(feature = "verdict-store")` 挂特性」：按这个字面串 `grep -l` 数出 12 份，写法不同的没数；判决照 12 写，不影响它那一格的结论。
- 复跑：攻方模型拷到草稿目录，`weighted_shares_drift.py` 两组、`layer0_slices.py` 一组，三份输出与报告 `outputs/` 逐字节相同；闭包探针与历史写表见第六节。

## 二、逐格判

| 格 | 判 |
|---|---|
| 辩方：第一轮第二节一格不翻 | 采 |
| 辩方：读故障那一标现算还要写表到录制流下标的映射（E5 副本 `crash_injection.rs:1098` 的第三个入参） | 采：每个崩溃态直接存一位当主条（S1） |
| 辩方：随机历史封顶「约 60 步」推法不对；攻方 Q2 量出 71–136 步 | 采攻方的量（复跑见第六节）；封顶来自给定状态表设计，拆两段不放大它，两腿一致 |
| 攻方 R1：`12`/`14` 历史键靠手写串、录入版本不含设点测试文件，丢了 H8「测试文件出发的整份闭包」 | **坐实**（代码行现查，探针复跑见第六节） |
| 攻方 R2：自证之二「在某一把闭包里」放过「在错的那一把里」 | **坐实**；辩方 S3 指出同一件的现成先例（`gpu_verifier_digest` 自带摘要进自己表的键），两腿给的改法同形：跑时把编进管线的源码摘要绑进那一种结论 |
| 攻方 R3：入口取写表函数今天所在文件，闭包含 WGSL 与两个判器，改内核就重录；`12`、`14` 同一把闭包 | **坐实**；与辩方 S2（`12`、`14` 并成一族）、S6（录入与提取代码先搬出 `crash_amplification.rs`）合成一条 |
| 攻方 R4：自证之一只看一层调用方 | 采，推的；被 R1 的改法罩住，自证之一撤掉 |
| 攻方 R5：簇 0 验收用词法单元多重集，真搬家判红、对调操作数判绿 | **坐实**（量过）；改按项多重集，它自报的漏处（同名项互换、`use … as` 改名）另配一条名字解析检查 |
| 攻方 R6：己的检查「开 GPU 时 CPU 对比入口被调就红」与 J6、退回撞 | **坐实**（代码行现查）；改判「当结论用的那一份来自哪种设备」，`JudgingDevice` 今天就入库 |
| 攻方 R7：RefFS 历史内红之后，录入段照读出重起头、读出进版本表，之后的 RefFS 票照常记 | **坐实**（`lib.rs:463`）；第 7 条改成按「依不依赖已判红的参照」分 |
| 攻方 R8：按块亲和时权重每趟重量，四成的块换主；「或列族」撞整份 `cp -a` | **坐实**（权重三组现查、模型复跑相同）；块归属在录入那一刻写进库 |
| 攻方 R9：P12、P17 没落处 | 采 |
| 攻方 R10：块键跨历史去重，单值「属于」装不下 | 采，推的、可达性没造出；读 `12`/`14` 的结论用（块键，历史键） |
| 攻方 N1：层 0 按片分台、片长小于一块，每块横跨两台 | **坐实**（算术复跑相同） |
| 攻方 N2：钉回快档按历史键会漂；只在 GPU 上红的复判不出 | 采，推的 |
| 攻方 N3：`12`/`14` 没有归属规则 | 采 |
| 攻方 Q3：态数基数是下界，9/30 另有一条流不穷举 | **坐实**（日志现查）；进 C761（9/30 崩溃放量全量「26.16 亿态零违例」里多记录流没穷举）回扫 |
| 辩方 S4：交用户表第 2 行补「红步之后依赖模型的判不再问」 | 采；与用户弹窗原话「照判，报告里标在红步之后」冲突，只能交用户 |
| 辩方 S5：交用户表第 1 行补「按文件挂特性」 | 采（今天快档已分两趟） |
| 辩方 S7：模型挪到对比段重放 | 不采：执行器边跑边问模型定删除步数（`history.rs:3883` 一带），辩方自己已否 |

## 三、第三轮被判的形态（修订，零轮）

在第一轮判决第三节七条上改：

1. **分界**：第一轮第 1 条照旧，读故障那一标改成「每个崩溃态存一位，录入段算」。
2. **表与键**：
   - `12`、`14` 并成一族「历史跑数据」。
   - 键按内容取：生成好的操作序列、整份执行参数、几何、取历史与取崩溃态判据所在文件的摘要。录入版本照 H8 并上设点测试文件出发的闭包。
   - 读这一族的结论种类，键用（块键，历史键）。
3. **版本**：
   - 写表函数先搬进各族自己的叶子文件，族的入口取那份叶子文件；配一条会红的检查：族闭包里不许有 `*.wgsl`、CPU 判器入口与别族叶子。
   - 结论版本的 `extra_inputs` 绑它读的录入版本，并在派活那一刻拼进实际编进管线的源码摘要（现成的 `kernel_sources_digest`）。
   - 撤掉第一轮的两条自证（调用方在闭包里、读 `*.wgsl` 的文件在某一把闭包里）。
4. **库与双机**：
   - 录入另起一份库，放在双机拷库的目录之外。
   - 块归哪台在录入那一刻写进库，之后照表分，权重只管还没录的块。
   - 历史跑数据那一族也要一条归属规则：按历史键归台；或两台都做，导入时逐行比相同。
5. **GPU**：在第一轮第 5 条上补两句：P12 只改恢复内核里记录核对器那一截；P17 并进 P14 那个内核。
6. **簇**：
   - 簇 0 多拆一件：流水线录入段搬出 `crash_amplification.rs`。
   - 簇 0 的验收改成按项多重集（去 `use`、`mod`、可见性），另配名字解析不变的检查。
   - 丁二写明迁过来的层 0 用例改按块分、续跑走库，`layer0_progress.rs` 与门禁里 `shard=across-machines` 那几行进丁二的文件清单。
   - 己的检查改判「当结论用的那一份来自 GPU」。
   - 钉回快档钉内容不钉历史键，只在 GPU 上红的钉进开 GPU 才判的那一组。
7. **红步之后**：按「依不依赖已判红的参照」分。理想模型第一处红之后、RefFS 自己第一处红之后，各自那一类的判单列：不进总数，不登记新发现。不依赖的照常记。

## 四、零轮与欠账

- 第三节七条都是零轮。第三轮（最终轮）换攻击面：「已定」项撑不撑得住（第一轮第二节坐实的那几格与第三节）、未定项与交用户表完整不完整、两轮报出的数能不能核。第三轮之后不再开，新冒出来的零轮形态写进交用户表、标「零轮」。
- 量（重型，主 agent 跑，V…E8 合入之后）：第一轮第四节 M1–M4 照旧。另加 M5：30 深下 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels` 那一项穷不穷举、态数多少。
- C761 回扫加一句：9/30 那一趟另有 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels` 不穷举（日志第 207、225、237 行）。进下一次 kb 写回批。

## 五、交用户表（第三轮之后、代价数齐了再弹窗）

| # | 岔路 | 候选 | 缺的代价数 |
|---|---|---|---|
| 1 | 快档怎么读库 | 甲：快档整条带 `verdict-store`；乙：库是一个接口，RocksDB 与内存两份实现；丙：按文件挂特性（今天快档已分两趟，12 份测试文件已按特性挂） | 每个编译目录多编一遍 C++ 的用时 |
| 2 | 模型在对不上那一步、之后怎么走 | 不走／照实现走／照模型自己的答走／之后依赖模型的判不再问（与用户「照判，标在红步之后」冲突） | 不涉及挂钟 |
| 3 | 按 `recover` 读到的字节记忆化 | 用／不用 | M1 |
| 4 | 观察者那一块 CPU 账 | 照「共享前缀每个状态都做」整域挂载 | M2 |
| 5 | RefFS 历史内读出越界那一次 | 录入段照读出重起头、之后单列（第三节第 7 条）／录入段当场判、照今天停（要在录入段做一次 RefFS 只读判） | 不涉及挂钟 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 4 | 不影响 | 2026-10-10 不受影响：第二轮只定第三轮要攻的形态，不改条款 |
| D13（验证路线） 已定项 9 | 不影响 | 2026-10-10 不受影响：同上 |
| D13（验证路线） 已定项 15 | 不影响 | 2026-10-10 不受影响：同上 |

## 六、复跑核对（攻方模型）

攻方模型目录拷到草稿目录，在 E5 副本上重跑：
- 闭包探针 `run_probe.sh`（经 `run-with-memory-cap.sh 2G`）：`table-diff.out`、`record-entry-closures.out`、`cluster0-acceptance.out`、`selfproof2.out`、`table-base.tsv` 五份与报告 `outputs/` 下逐字节相同。日志里一句 `failed printing to stdout: Broken pipe`，是脚本里接了 head 的管道截断造成的，末行 `PROBE_DONE`、退 0。
- 历史写表 `run_history_writes.sh … 0 36 144`（经 `run-with-memory-cap.sh 4G`）：36 行去掉 `seconds=` 那一栏后与 `outputs/history-writes-144-steps.out` 逐行相同，即「写表第一次过 2048 在第 71–136 步，2 段到 144 步没过」。
- `weighted_shares_drift.py` 两组、`layer0_slices.py` 一组：三份输出逐字节相同（第一节）。
这些是腿的模型上的数，不进 kb。

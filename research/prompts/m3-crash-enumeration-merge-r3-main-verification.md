# 崩溃点重放合并：设计轮第三轮判决（最终轮，2026-10-10）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13 G14 H1 H2 H3 H4 H5 H6 H7 H8 H9 H10 H11 H12 H13 H14 J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 L1 L2 L3 L4 L5 L6 M1 M2 M3 M4 M5 M6 K1 K2 K3 K4 K5 K6 K7 W -->

正文 `research/prompts/_m3-crash-enumeration-merge-r3-body.md`（被判形态 H1–H14，格 M1–M6）。两腿报告：
- 攻方：`research/prompts/m3-crash-enumeration-merge-r3-opus-output.md`，sha256 01bc3e64…5336，308 行；模型 `research/prompts/m3-crash-enumeration-merge-r3-opus-model/`；
- 辩方：`research/prompts/m3-crash-enumeration-merge-r3-sonnet-output.md`，sha256 b025b0fa…1788，113 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | `research/prompts/m3-crash-enumeration-merge-r3-snapshot/` 的 crates 91 份、kb 两腿交齐后 `sha256sum -c --quiet` 全过（取在删除线第 56 批合入之后） |
| 引文 | 两份一起 `cite-check.py --root . --background research/prompts/_m3-crash-enumeration-merge-r3-background.md`：「✓ 核了 75 处引文，对上 75 处，没判 0 处」（`~/.local/share/singlefs/work/m3-crash-enumeration-merge-r3-main/cite-check.log`） |
| 复跑攻方模型 | 拷进 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r3-main/model/`，`run-with-memory-cap.sh 8G bash rerun.sh` 退 0，末行「MODEL_SELFCHECK compared=20 mismatched=0 []」；模型是从 `crash_identity.rs` 按行抽出闭包算法拼的零依赖程序（钉了那份文件的 sha256），不链 checker 档；七份设点文件十四行 `crash_amplification.rs=1 crash_judge_dispatch.rs=1 gpu_unit_checks.rs=1` 与报告相同 |
| 主 agent 现查 | ① M1-2：走 GPU 的计算类今天由私有显卡配置的 `GPU_COMPUTATION_CLASSES` 正表决定（`research/scripts/crash-amplification-lib.sh` 第 19、25、75 行）。② M1-3：层 0 配置按 git common-dir 找主工作树（`research/scripts/layer0-shard-configuration-check.sh` 第 33–34、88 行），放量按 `$repository_root/multi-host.env`（同库第 127 行，第二轮辩方引过）。③ M2-2：被判的根槽写进计划哈希（`crash_amplification.rs` 第 712–719 行），而它「判器拿它记「根槽已持久」，不进逐项判定；照样进键」（同文件第 497–498 行），攻方改法甲成立的前提坐实。④ M1-1：重型闸、实现员定义、共用约束拒子 agent 跑 checker 档，第三轮辩方 M6-12 同指 `.claude/agent-common.md` 那一句 |
| 没跑的 | 攻方第 0–3 条（抽样量整项、CPU 速率、挂载速率）与辩方那条 checker 档命令都是重型或要卡；这一轮判决不靠它们定形态，代价数等主 agent 在今天的树上量过再交用户（第四节） |

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| M1-1、M6-12（轻型由实现员跑，闸与定义都拒） | 两腿同 | **打中**：用户定「轻型交给实现员与合入后验证跑」，J12 把 `.claude/hooks/lib_heavy_tests.py` 的 `classify`、`.claude/agent-common.md`、实现员定义并进第一批（即 T4） |
| M1-2、M6-5（豁免负表与私有正表两份） | 两腿同 | **打中**：删掉私有配置的 `GPU_COMPUTATION_CLASSES` 正表，选路只看本机开关与仓内豁免表（用户「多机配置不要重复配置」） |
| M1-3（worktree 里读哪份配置） | 攻方 | **打中**：统一按 git common-dir 找主工作树的 `multi-host.env`（用户「应该在 /home/fy5090/code/singlefs/multi-host.env」） |
| M1-4（「全量」两义） | 攻方 | **打中（小）**：重型三种跑法照用户的叫法：剪枝、逐条、全量（清空复用）；不带参数 = 全部项按剪枝跑。SOP「不带参数就是全量」指范围，口径差记进里程碑收尾补上游的那一条 |
| M1-5、M6-6（`ENABLE_GPU` 没写算开） | 两腿同 | 用户 2026-10-10 纠正后作废：公共判法不改（「没写或写 0 算关」），本机 `multi-host.env` 合成一份时写开 |
| M1-6 | 攻方 | 没打中（单样本） |
| M2-1（CPU 判器版本的设点闭包把整份引擎带回） | 攻方，模型复跑对上 | **打中**：CPU 判器版本 = CPU 判器闭包 ∪ 设点文件与它 `mod` 进来的测试模块，闭包在停止表处停；停止表加流水线那几份（`crash_amplification.rs`、`crash_judge_dispatch.rs`、`gpu_unit_checks.rs`、`verdict_store.rs`、`layer0_progress.rs`），改结论的几处照 H8 搬进判器一侧 |
| M2-2（C 照样不共享） | 攻方 | **打中，采改法甲**：被判根槽写不进块键与计划哈希；「根槽已持久」的计数各项按自己的被判根从块的持久集合事后数，块里不存（`crash_amplification.rs` 第 497–498 行写明它不进逐项判定）。用户定的「共享」照样成立，共享量与去重数等量过再交 |
| M2-3（观察者读什么） | 攻方 | **定义缺口**：观察者另读的流程输入进观察者结果的键；单跑一项要的别项流程照录（正确率优先，`.claude/main-agent.md`「禁止」第一条） |
| M2-4（失败点没有共同编码） | 攻方 | **定义缺口**：用户把「WGSL 与 Rust 怎么保持一致」挪到合谋那一段；这一版对照比红项与两遍各自的落到的根与结局（六个字），失败点码表随那一段一起定 |
| M2-5、M5-17（驱动与卡型） | 两腿 | **定义缺口**：驱动版本、卡型作为块的记录属性存，不进版本；两台驱动不同不分版本，靠对照与轻型的小输入对拍抓差 |
| M2-6、M2-7 | 攻方 | 打中（小）：公共模块的 `super::common` 照 `#[path]` 收；「第三批之前」两句随 J12 生效时点删掉 |
| M3（数能不能核） | 攻方 | 采：报给用户的数分三类（一条命令重算、只能量、作废）；量法用 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/N` 等距抽块（`crash_amplification.rs` 第 906 行平滑加权轮转），四种跑法的挂钟算式照报告「最小量法」一节；主 agent 先核第 0 步（一份能单独走完）再量 |
| M4（15 条岔路） | 攻方 | 逐条去向在第三节 |
| M5 各格 | 辩方 | 第二轮判决没有一格被推翻；M5-14（`build.rs` 调不到闭包算法）**打中**：闭包算法抽成零依赖模块，`build.rs` 与库用 `#[path]` 共用（攻方模型证明它抽得出来）；M5-19（两件进实现前事实核没带）**打中**：J14 第二批开工先核随机历史与 io_min 8192、清扫小环池的判表建不建得出；M5-18 采：对照被授权跳过时弹窗写明空真守门一起跳过；M5-21 采：H2 改成「一项一条流，13 条崩溃用例与 8 项放量按流去重之后合表」；M5-8 采：判表打包核段长是必做，满 64 位余数一起做 |
| M6-2（日常剪枝与轻型无剪枝） | 辩方 | 采：用户后说的取代先说的：CPU 对拍属重型（有剪枝），轻型没有剪枝 |
| M6-4（「随机历史的记录核对器」豁免行） | 辩方 | **打中**：记录核对器判一张镜像，照「能走 GPU 的必须走」上 GPU，那一行删；豁免表每一行登记时由用户点头（用户「除非明确的豁免」） |
| M6-10（轻型 ⊂ 重型被 H10 破） | 辩方 | **打中**：重型对展不开的段照样跑轻型那两个域（甲二 ∪ 展开 6），重型 ⊇ 轻型 |

两腿提的改法与主 agent 的取舍都被攻过零轮。

## 三、定稿形态 J1–J14（照第二节改 H1–H14）与 M4 岔路去向

- J1 名字：同 H1。
- J2 一张表、一个入口、一道门禁：同 H2，「一项一条流，13 条崩溃用例与 8 项放量按流去重之后合表」；`crash-case-check.py` placement 同批改认新项。
- J3 一套引擎：同 H3；B、C 挪进公共模块；被判根槽写出块键（M2-2 甲）；随机历史加给定状态表一种展开。
- J4 观察者：同 H4，观察者另读的流程输入进观察者结果的键，单跑照录别项流程（M4 第 10 条）。
- J5 判在哪：照本机 `multi-host.env` 的开关（没配 GPU 走 CPU、配双机分片、配 GPU 走 GPU）；选路只看开关与仓内豁免表，私有正表删掉（M4 第 3 条）；豁免表每行用户点头才登记；随机历史的记录核对器上 GPU；配了 GPU 卡用不上退 78。
- J6 对照：配了 GPU 时 CPU 判器同一批状态也判，同一趟并行（M4 第 9 条），比红项与两遍各自的根与结局；比不上判红；默认跑，跳过由用户当场授权，弹窗写明空真守门一起跳过；失败点码表随合谋那一段定（M4 第 8 条）。
- J7 计数随块存：同 H7；「根槽已持久」计数事后数。
- J8 版本：GPU 结论（块键，GPU 判器闭包）；CPU 结论（块键，CPU 判器闭包 ∪ 设点文件与它 `mod` 进来的测试模块，停止表加流水线五份，M4 第 6 条）；观察者结果（块键，观察者版本 + 另读输入）；驱动与卡型记为属性（M4 第 11 条）；闭包算法抽成零依赖模块供 `build.rs` 用；跑时再算一遍，不等退 78；不做等价剪枝。
- J9 两档：轻型 = 三样（崩溃、形式化、变异）各自登记的小集合，跑全部，由实现员与合入后验证跑；重型 = 提交时的总门闸，剪枝 / 逐条 / 全量三种跑法，不带参数按剪枝；轻型 ⊂ 重型；提交时逐条跑由用户在弹窗里选、主 agent 列出输入变了的项，没点到的报「本次未跑」（M4 第 12 条）；重型红了必须修好或查清原因（用户 2026-10-10「红了必须修复或者找到原因」）；轻型是过渡闸，事务功能做全之前保留，之后用户判稳定再并进重型。
- J10 段：同 H10；登记行按段的写表摘要认，写表变了对不上就拒跑（M4 第 15 条）；重型对展不开的段跑轻型两个域。
- J11 跑在哪：多机配置在仓根 `multi-host.env` 合成一份（`~/.config/singlefs/gpu-two-hosts.env` 并进去，先备份），本机写开 GPU；按 git common-dir 找（M4 第 4 条）；读 `ENABLE_GPU` 的别处不改（M4 第 14 条）；提交那一趟整域在 HEAD + 暂存区的 worktree 里跑。
- J12 条款：同 H12，另加 `.claude/agent-common.md`、`.claude/hooks/lib_heavy_tests.py`、实现员定义（轻型给子 agent 跑，M4 第 1 条）；轻型要卡时照清场规则排队等卡，不抢在跑的重型（M4 第 2 条）；第一批合入后门禁快档照今天的命令，新入口第四批之后接（M4 第 13 条）；生效时点同 H12。
- J13 旧数：同 H13；两轮的挂钟、共享量、去重数全部换成主 agent 在今天的树上量的数。
- J14 切批：同 H14；第二批开工先做两件事实核（随机历史、io_min 8192 与清扫小环池的判表）。

## 四、交用户

| 项 | 状态 |
|---|---|
| 代价数 | 两轮报给用户的挂钟全部作废；主 agent 照第二节 M3 的量法在今天的树上量（删除线 GPU 那一趟完了之后），量完交整域态数、去重数、共享量、GPU 与 CPU 速率、挂载速率、四种跑法的挂钟 |
| 豁免表每行用户点头 | 主 agent 取的读法（用户「除非明确的豁免」），用户可改 |
| 零轮 | 第二节、第三节的改法都被攻过零轮；照 `.claude/rules/three-way-inference.md`「多轮」第三轮之后不再开轮 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 4 | 备料 | 2026-10-10 不受影响：枚举域定义不变 |
| D13（验证路线） 已定项 9 | 备料 | 2026-10-10 不受影响：按 J12 在第四批合入时改写 |
| D13（验证路线） 已定项 15 | 备料 | 2026-10-10 不受影响：checker 档归属不变，两档写回随 J12 |

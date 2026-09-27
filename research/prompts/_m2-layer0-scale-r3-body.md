# 里程碑二收尾：层 0 全量在一台开发机上跑得完，设计轮第三轮正文（2026-09-26）

<!-- doc-lint:not-numbers L1 L2 L3 L4 L5 L6 L7 H1 R1 R2 R3 R4 R5 M1 M2 M3 M4 N1 N2 N3 -->

## 一、这一轮要判什么

第三轮，也是最后一轮。第二轮判决 `research/prompts/m2-layer0-scale-r2-main-verification.md` 整份进材料：甲打中两轮、出局；甲二被打中一轮、挂起（记录核对器第二条的复用豁免在「一对重叠的 COW 写只落一个」时假红，全枚举 32768 个红、甲二 0 个红）；乙打中、挂起；三样零轮形态待攻：按扇区判的复用豁免、收严后的续跑 R1–R5、54 号输入指纹补的那几样。攻击面不重复前两轮（第一轮攻甲的字面前提、固定脚本与卸载的历史、用户动作放开扫、变异表 32 行、续跑原型的截点与真杀；第二轮攻甲二的等价、续跑 R1–R5 原形、乙的清单、第一轮判决复核）。

| 格 | 被攻的形态 | 问题 |
|---|---|---|
| N1 | **甲二被打中那一格**（第二轮判决第二节 M1 行） | 打中的是不是只有记录核对器那一处判据错：照第二轮攻方的按扇区豁免（`research/prompts/m2-layer0-scale-r2-opus-output.md` 第四节首行、探针 `variant_claimed_state_missing_unit`）换掉今天的豁免之后，冻结副本的两条流与第二轮攻方用过的随机历史上，甲二与全量逐状态判得一不一样；「只在 COW 部分落盘时显出来」这一类除了这处判据错还有没有别的成员；甲二能不能当平时快档、它当快档时放过的是哪一类 |
| N2 | **三样零轮形态**（第二轮判决第三节） | ① 按扇区豁免会不会放过一次真的错误复用：C507 那一格（后写没落盘的真洞）、C513 那一道（复用证得出违反回收谓词不开脱）在按扇区判之下还红不红，造一个按扇区判放过而今天的判法抓得到的错；② 收严后的 R1–R5（一条流一个进度文件、片行在观察者看完之后写、判红删进度文件、合并前核片数）还有没有一次只跑了一部分却报全量的读法；③ 54 号输入指纹补的那几样（`.cargo/`、`rust-toolchain*`、`RUSTFLAGS`、`CARGO_PROFILE_*`、`CARGO_ENCODED_RUSTFLAGS`、`CARGO_BUILD_*`）之外，还有什么改了会改测试二进制的行为而不进指纹 |
| N3 | **代价表里的数**（第二轮判决第四节） | 5575606380、每秒 13880 个、32 线程约 2.3 天、甲二 210、M1 那一段 262144 个里 32768 个红，逐个按事实表复算，写明哪一步是外推 |

**共用问句**：每一格都答「照这个形态做，有没有一个今天全量层 0 抓得到的错会被放过」，答案要能被变异、反例或逐格算核。

## 二、实现今天的样子（主 agent 的观测，2026-09-26 JST 12:xx）

- 冻结副本照旧 `/tmp/claude-1000/l0scale-r1-frozen/`（实一加实二那一版）。实三（挂着时回退、正常卸载）已交回，实四乙还在主工作区改 `crates/` 的测试，新形状这一轮不冻结，按原理攻。
- 记录核对器第二条的复用豁免：冻结副本 `crates/singlefs-harness/src/crash.rs:706`、`:714`（第二轮核查员 `research/prompts/m2-layer0-scale-r2-verifier-output.md` 核过）。
- 第二轮攻方的探针与复跑在 `research/prompts/m2-layer0-scale-r2-opus-model/`（`opus_r2_probe.rs`、`opus_r2_resume_probe.rs`、`rerun.sh`、`outputs/`），第一轮攻方的在 `research/prompts/m2-layer0-scale-r1-opus-model/`，这一轮可以接着用。
- 切段、枚举、合并的落点照第一轮附录：`crates/singlefs-harness/src/segments.rs` 的 `split_into_segments`，`crates/singlefs-harness/src/crash.rs` 的 `enumerate_layer0_in_state_slices`、`Layer0Tally::absorb_following_slice`、`expand` 钩子与 `LAYER0_PROGRESS`。
- 54 号的输入登记：`.claude/gate.d/stage-inputs.tsv` 里 54 那一行；指纹怎么算在 `research/scripts/admission.py`。

## 三、条款（材料员整段抄进附录）

- 第二轮背景材料 `research/prompts/_m2-layer0-scale-r2-background.md` 第三节的条款照旧；
- 第二轮判决 `research/prompts/m2-layer0-scale-r2-main-verification.md` 整份；
- `.claude/kb/checks-owed.md` 的 C507、C513 两行；
- `.claude/kb/invariants.md` 里记录核对器第二条对应的那一条（材料员按 `crash.rs:706` 附近的注释找编号）；
- `.claude/agent-common.md` 那条「崩溃点测试不衡量时间成本，也不为省时间缩范围」。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | N1 | 替甲二辩：打中它的是不是只有那一处判据错，修了判据之后甲二站不站得住；辩不住的写清还差什么。在冻结副本的拷贝上用第二轮攻方的探针量，**取样**每次全量不超过约 10⁶ 个状态；不跑名字带 `layer0` 的测试目标、不跑 54 号 |
| 云端攻方（Opus） | N2 | 攻三样零轮形态：造按扇区豁免放过的真错、续跑收严后仍报全量的读法、不进指纹却改行为的输入。取样与禁跑同辩方 |
| 本地攻方 | N3 | 按事实表逐格复算代价表的数；事实表每行写来源文件与行号，每张表不超过 6 行：第二条流的段序列与闭式（`research/prompts/m2-layer0-scale-r1-opus-model/outputs/p1.log` 第 22 行）、速率（`records/2026-09-24-里程碑二收尾调度.md` 第 162 行的 `finished_states=4194320/67108885 elapsed_seconds=302.2`，16 线程）、M1 那一段（`research/prompts/m2-layer0-scale-r2-opus-output.md` 里 `q9.log` 那一行） |

两条攻方腿不重叠：Opus 造错与原型，本地只复算数。辩方与 Opus 都会用第二轮的探针，各自拷一份，不共用草稿目录。

## 五、交付

- 腿的报告写 `research/prompts/m2-layer0-scale-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 模型与用例放 `research/prompts/m2-layer0-scale-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引 kb 写 kb 文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引的行是那句原文自己所在的行。
- 方案按 `crates/` 的冻结副本谈。
- **内存与进程**：编译与跑一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>`；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知，不用 `true` 或 `sleep` 空转；交回之前后台不许留着跑的东西。

# 第 16 批代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 K4 P1 P2 P3 U6 R2 V242u -->

被判的文件：`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`，新测试 `crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs`、`crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs`，`crates/mutations.tsv` 第 2215–2229 行。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b16-r1-body.md`、`_m3-formal-code-b16-r1-diff.md`、`_m3-formal-code-b16-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b16-r1-snapshot/`（crates 264 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b16-r1/tree/`） | crates 一份无输出；kb 一份 `.claude/kb/checks-owed.md` 对不上：开工之后 c673 写回改了它（C673、C407 两行与 C675–C679，旧串新串在 `/tmp/claude-1000/c673-writeback/report.md`），腿引的 C670 那一行不在改动里 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b16-r1-sonnet-output.md`（261 行，sha256 67fa8e9c…） | 引文 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b16-r1-opus-output.md`（342 行，sha256 a4ce8c74…），模型 `…-opus-model/` | 主 agent 在会话草稿 `b16-rerun/` 跑 `tools/rerun.sh`（`RERUN_RC=0`；第一次被内存包装「同会话只许一条」拒，带并行理由重跑）：`abandon-*` 四份、`m2227-ops-overwrite`、`m2228-ops-overwrite`、`v242u-named` 去掉耗时逐行同；`m2227-named-short` / `m2228-named-short` 与腿的 `m242-named-short` / `m269-named-short` 的 `first_c670=10`、`=25` 行同；`base-*` 七份只多一行 `abandoning_steps=0`（腿的原型后来加的计数）与耗时；第 2215–2224 行变异十行 `named=FAILED`；K1 那一行 `verdict=Err(PublishCount)` 同 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本> --root . --background <背景材料>`：「✓ 核了 57 处引文，对上 57 处，没判 0 处」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 卸载的答 | 与 D16（发布语义） 已定项 1 一致；`observed_unmount` 交每一条空发布的根；共用循环逐行不变；新 oracle 只被新用例调 | 树表 0 条那一格「答不了」不是唯一读法：D16 第 42 行字面推空发布，不推的依据是 `crates/singlefs-core/src/mount.rs:2216` 那条没进 D16 的 2026-09-26 定案；副本上放开后模型推 2 条、对拍报 PublishCount | **一致（站住）**；树表 0 条那一格是条款字面与一条没进条款的定案说反话，**交用户**（第五节第 1 行）；接线（随机历史、E171、崩溃注入没接卸载）记进 C679 一族，E171 第 1 次重跑会接 |
| K2-a 误红 | — | 今天的 core 上 33 975 个首次崩溃态、365 228 个二次崩溃态，判据 N-配置 的拒 0 次（短段取样，不是层 0 全域） | 没打中（复跑坐实计数） |
| K2-b 漏判 | — | **打中（射程）**：第三截（第二截那次挂载途中再崩、再挂）被拒只计数（`crash_injection.rs:2771`、`:2774`），不判 C670；自造坏实现 V242u 在「起点 + 一次覆盖写」上第二截 0、二次崩溃之后 108 | **坐实**（复跑 `second_c670=108`）：改法 P1（第三截也接新判官，`WritableMountAlone::Refused` 带出 `MountError`）进下一批实现员，零轮。第 2227、2228 行原样在起点那次挂载上就红，不因此活下来 |
| K2-c 读故障认定 | — | 没漏别的步；但这一支今天是死的：1600 段历史里抛弃根那一步 158 次、0 次往录制流发东西 | 采；C670 射程写明「`Injected` 那一支今天只有合成用例走到」 |
| K2-d 撕裂那一支 | — | 分得干净 | 采 |
| K2-e 接线 | — | 部分打中（推的）：调用点恒交 `Injected` 或换回旧判官时基线全绿，只有变异整表第 2227、2228 行变没红才看得到 | 采改法 P2（一条基线上就要红的接线用例）进下一批，零轮 |
| K3 GPU 那一格 | 读码同判（最新根同一条、`highest_seen` 共用、判法摘要随内核原文变） | — | 站住（读码）；GPU 用例与变异归提交时 |
| K4 变异行 | harness 档几行副本里逐行证红；第 2227–2229 行推的 | 第 2215–2224 行十行量过；第 2227、2228 行短段域上基线 0、变异 10 / 25 | 站住；第 2227–2229 行要提交时 checker 档坐实 |

## 三、去向

1. 16-2、16-3、16-4 的改动站住，不回改。
2. 下一批实现员（并进 17-1 合入之后的那一批，`crash_injection.rs` 以根环那条线 R2 改完的 `mount_transparency.rs` 为底）：P1（第三截接 C670 判官）、P2（接线用例）。
3. 树表 0 条那一格的卸载：用户定读法之后照办（第五节第 1 行）。
4. 提交时：第 2225–2229 行与新用例里 checker 档、GPU 的那几条；攻方报告第十二节给的层 0 全域命令（1 069 129 个首次崩溃态，8 片）可选。

## 四、复跑核对

见第一节表。核不动：checker 档、GPU、层 0 全域都没跑。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | 树表 0 条的一版上正常卸载：甲 照 D16（发布语义） 已定项 1 字面推空发布（改实现）；乙 把 2026-09-26 那条定案（一个字节都不写）写进 D16（改条款，模型照它答零条根） | 两腿 | 甲副本上量过（模型推 2 条、对拍报 PublishCount）；乙推的 | 弹窗交用户 |
| 2 | P1 第三截接 C670 判官 | 攻方 | 副本原型量过 | 下一批实现员 |
| 3 | P2 接线用例 | 攻方 | 推的 | 下一批实现员 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-04 不受影响：树表 0 条那一格交用户，定之前不改 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-04 不受影响：只拿见证判据当 16-3 判官的判据 |
| D13（验证路线） 已定项 1 | 不影响 | 2026-10-04 不受影响：只拿挂载透明只管不带故障的状态这一射程 |

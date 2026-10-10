# 第 36 批代码轮第一轮判决（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 R1 R2 R3 -->

被判的文件（第 36 批两份补丁，逐份见改动全文 `research/prompts/_m3-formal-code-b36-r1-diff.md`）：
- 36-1b：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`；
- 36-2：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`；
- 两份的追加与替换：`crates/mutations.tsv`。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b36-r1-body.md`、`_m3-formal-code-b36-r1-diff.md`、`_m3-formal-code-b36-r1-background.md`（第一版 1.4 MB 挪到会话草稿 `b36-materials-v1/`，裁成 94 KB） | — |
| 冻结副本与快照 | `~/.local/share/singlefs/work/m3-formal-code-b36-r1/tree/`（第 36 批合入后），清单同目录 316 行；kb 快照 `research/prompts/m3-formal-code-b36-r1-snapshot/kb-sha256.txt` | 两腿复跑的 `setup` 段各按清单核过冻结副本；kb 快照写判决时全 OK |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b36-r1-sonnet-output.md`（sha256 780eea97…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh` 复跑（第四节） |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b36-r1-opus-output.md`（sha256 0fc801e7…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh` 复跑（第四节） |
| 引文 | 两份腿报告 | 两腿各自 cite-check：正推 44 / 44、攻方 27 / 27 |

背景材料第二节写的扫描内核表达式数 700 783 是根环线转报的数；攻方用 naga 在冻结副本上量到 692 201，700 783 恰是反打 36-2 之后的数（36-2 让扫描内核少 8 582 个）。两者都超预算 400 000，以攻方量的为准。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| P1 R1（CPU）取的第一次发布与 core 真施加的是否同一份 | 一致：九种改法 576 行、网格 144 张、回退之后 8 行，checker 认那一版当且仅当 core 施加第一次发布，0 分歧；改回旧认法的对照臂 21 个子集分歧 | 没打中：随机 272 段 24 971 张、层 0 五条流 123 040 态，R1 认的 31 859 张与 core 逐字节同、错认 0；旧认法多认的 1 591 张断号镜像 R1 一张不认；坏盘 281 张逐张同（用户动作 7×7×3 放开扫过） | **站住**（两腿立场不同、各抽一次，不记成「没打中」的结论） |
| P1 `Stopped` 带第一次那一支 | 一致：144 张里 18 张走到、都认 | 合法历史 0 次，只有撕掉根槽的坏盘走得到 | 站住 |
| P1 I-3.10 与 I-5.4 同一认法 | 一致：`crates/singlefs-checker/src/walk.rs` 算一次、两处共用 | — | 站住 |
| P1 合法历史多报少判 | 一致：H2 102、H3 191、H3S 281 态 0 违例，R1 开关前后逐状态签名 0 行不同 | 新旧只差 14 张 I-5.4 部分没判 → 成立（旧认法认了 core 不施加的那次发布），是 R1 修掉的多报 | 站住 |
| P2 交接段第 31 个字 | 一致（推的）：写字先于扫描内核开工、能置 `H_VALID` 的路径都写了它；I-4.1 那一处行为不变 | 站住（推的）：置 `H_VALID` 只有 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4603` 一处，进 DONE 的两处都在写第 31 个字（4180 行）之后 | 站住（推的，GPU 没跑） |
| P2 naga | 两个内核与 GPU 四行变异施加后都 rc=0，故意写坏的两份判红 | naga 校验过 | 站住；扫描内核超预算，GPU 四行变异今天编不上卡、证不了红 |
| P3 R2 新造法 | 一致：实例 1 行 W 新造法 4（合法同）、旧造法 0；系统配置轮换序列不再多那一对；两条流名字说的那一形还在 | 站住（读码）：新造法下层 0 枚举的每个状态都是合法历史 | 站住 |
| P4 R3 说明 | 一致；kb 冲突三处：I-5.4、I-3.10 两行写「txg 比最新根大 1、带提交标记的那条记录」，实现认「重放施加的第一次发布的末条」；I-5.5 行「仍落缺省理由」 | 一致；另提 I-3.10、I-5.4 两行「txg 比最新根大 1」R1 已不要（造盘上 R1 与 core 都认 T+2，零轮） | 站住；kb 三处进写回批（第三节第 1 条） |
| P5 变异行 | 8 行锚点各命中 1 次；harness 4 行红在断言；GPU 4 行 naga 过、没验红 | 同 | 站住（GPU 四行推的） |

正推另记：`crates/singlefs-checker/src/image.rs` 211–212 行文档注释里有一个仓里不存在的函数名 `version_the_next_mount_applies`（R1 之前就有），进下一批顺手改。

越格：攻方量到 I-3.1 在合法历史上红 5 张（种子 188088733，在一次管理员回退之后），新旧两份 checker 同红，不是这一批带来的；发根环线（C697（已分配统计在短环加大文件上误红） 那一族）。

## 三、去向

1. **写回批**（这一批与第 34、35 批判决合成一批）：`.claude/kb/invariants.md` 的 I-3.10、I-5.4 两行把「下一次挂载先施加的那一版」的认法写成「下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 施加的第一次发布、取它末条那一份新根段」，I-5.4 补由记录施加出来的那一版的三个条件（不低于回退下界、换下的单元还没到回收、进得了重放前缀）与 Q1 甲（两类版本树表指针全零判损坏，下一次挂载那一版经 I-4.1）；I-9.28、I-9.5 写明各处「自证过的根」的集合，I-9.28 那两句（由记录施加那一版「不记走读失败」、GPU「CPU 红 GPU 不红」）照第 35 批改后的实现改；I-5.5 那一句照 Q4 甲改；`.claude/kb/checks-owed.md` 新记实例表行 W 与恢复施加的最大事务号对不对得上没有不变量判（第 35 批判决第三节第 2 条）。
2. **下一批顺手**：`image.rs` 211–212 行文档注释的函数名。
3. **GPU 一侧**：第 35、36 批的 GPU 对拍与 GPU 变异今天编不上卡（扫描内核超预算），谁推过线由根环线的调查员逐版量；量完之前这几样的结论都是「推的」，提交时的 GPU 快档会把它们报成没跑或红。

## 四、复跑核对

正推：模型 `tools/rerun.sh setup build tests probes-build p1-a p1-b p1-c probes-small probes-r2a probes-r2b r1-off rows wgsl` 在 `~/.local/share/singlefs/work/formal-main/b36-sonnet-rerun/` 跑，退 0，编译日志三个被测包各有 `Compiling`。22 份 `t-*.log` 与 `anchors-all.log` 去掉耗时、线程号、筛掉的条数与路径前缀、按行排序后与腿留存逐行相同；4 行变异的结局逐行相同（四行 exit 101、红在断言、点名测试失败）；naga 校验十行结果与腿的 `wgsl-validate.log` 相同。

攻方：模型 `tools/rerun.sh <草稿目录>` 全跑，在 `~/.local/share/singlefs/work/formal-main/b36-opus-rerun/` 十三段 `rc=0`。`B36X`、`B36XT`、`B36XT2`、`B36L0-DONE`、`B36L0T`、`B36R`、`B36G`、`B36M` 行去掉挂钟与 FIRST 行后排序比：复跑 725 行、腿留存 725 行（腿另有一份 `seed10.log` 是它估时用的单段，不在 `rerun.sh` 里，不比）；不同的只有变异那几行 `failed=[…]` 里的线程号与基线那一趟是否重编的标记，各行的退出码、红的用例与还原结果相同。

核不动：GPU 对拍与 GPU 变异（扫描内核超预算编不上卡）、checker 档用例、层 0 快档，归提交时。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | I-3.10、I-5.4 两行认法改写（不再写「txg 比最新根大 1」） | 两腿 | 攻方造盘量过 T+2 两边同认；合法历史分不出 | 写回批 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-06 不受影响：R1 让 checker 照已定项 14 的重放规则认版本，条款不变 |
| D23（journal 的角色与格式） 已定项 15 | 备料 | 2026-10-06 不受影响：checker 取的是 core 按已定项 15 真施加的那一版，条款不变 |

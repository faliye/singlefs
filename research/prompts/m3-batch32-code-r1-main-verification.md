# 删除批第 32 批合入的代码：代码轮第一轮判决（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 P1 P2 P3 P4 P5 P6 P7 F-KEY F-ID-b F-W F-IT G4 S2 S4 E1 E2 E4 E6 E7 Z1 Z2 Z3 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch32-code-r1-body.md`、`_m3-batch32-code-r1-diff.md`、`_m3-batch32-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch32-code-r1-snapshot/`（crates 9 份、kb 若干份）；冻结副本 `~/.local/share/singlefs/work/m3-batch32-code-r1/tree/` | 冻结副本取在第 32 批合入验证绿之后、根环线 10u / 10v 合入之前，9 份 `sha256sum -c` 全 OK；腿引代码一律写冻结副本全路径 |
| 云端正推（Sonnet） | `research/prompts/m3-batch32-code-r1-sonnet-output.md`（350 行，sha256 58c11582…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch32-code-r1-opus-output.md`（214 行，sha256 f612b53b…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推「核了 92 处引文，对上 92 处，没判 0 处」；攻方「核了 29 处引文，对上 29 处，没判 0 处」 |
| 主 agent 复跑：攻方 | 模型拷进会话草稿目录 `m3b32-opus-model/`，`rerun.sh ~/.local/share/singlefs/work/m3b32-opus-rerun all`，再 `compare.sh <summary.txt>` | 「SAME 213 行」，汇总 sha256 62305640…；today 臂 `loc0_vs_loc1=DIFF` 6 行，都是 loc0 那一侧多 I-4.9；P4 原样 `P4 p4 type=0 next_written=true sum=zero32(0xbc43baad) gap_slot=50179 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4,I-9.23] i2_1=V` |
| 主 agent 复跑：正推 | 模型拷进会话草稿目录 `m3b32-sonnet-model/`，`rerun.sh ~/.local/share/singlefs/work/m3b32-sonnet-rerun all`，再 `compare.sh` | 与 `expected-compare.txt` 只差一行 cargo 打的「has been running for over 60 seconds」耗时提示（机器忙），其余 212 行逐行相同 |
| 读代码 | 冻结副本 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 第 497–508 行 | 见第二节 P4 回绕那一行 |

另：根环线 10u / 10v 合后验证在包含第 32 批的主工作区上清场跑了 8 份 GPU 测试，对拍「142 passed，0 failed」（根环线跑的，记在调度记录 2026-10-07 那一条）——第 32 批 3 条 GPU 对拍流在卡上跑过、与 CPU 判得一样；它们没造到下面 P1、P4 打中的那几形。

被判的 9 份文件（K3），每份一格判到。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 P1 三处「走过」的键 | 一致：CPU 与 GPU 都按读回那一份的落点认；GPU 没有 inode 内部孩子那一处 | — | **采** |
| K1 / K2 P1 没改的几把键（I-4.9 那一形） | 冲突：父单元、诞生代号、合并、叶序该成套改；副本上只改容器那一处，I-4.9 不再红而 I-3.5 由成立变红 | **打中三格**：① 多红 I-4.9 射程比实现员报的宽——位置条目 0 撞到被更早候选根共享的节点就中，today 换盘判得不同 6 对（谱系撞 inode / extent、墓碑撞 extent，bad / good 各一）；② 修掉 I-4.9 之后 I-3.5 的折叠表同样按第 0 条认；③ CPU 与 GPU 分叉：回退后共享的叶容器盘 0 那份坏、那槽没有别的单元时 CPU 红 I-4.9 与 I-3.5，GPU 走过表按盘 0 落点找不到那一行、两条都漏。七条臂里只有 allrbmap（走过、诞生代号、合并、父单元、叶序、I-3.5 折叠表一律按读回那一份认）全对 | **坐实（坏镜像上的错判，不进合法状态）**：复跑同上。两条腿同向：只改「走过」一把键不够，几把键要按同一个落点成套认。进第 33 批：照攻方 allrbmap（零轮）在 CPU 成套改，GPU 同改；钉攻方 today 那 6 对与分叉形（两种盘序同判） |
| K1 / K2 P1 合法状态 | 一致（读码）：键只在第 0 条那份对不上时才变，那时 I-2.1 已红 | 没打中（读码） | **记「没打中（一次抽样）」** |
| K1 P2 同一判据 | 一致：CPU、GPU 与 F-W 认别类是同一谓词（CPU 写了三遍、没有共用代码） | — | **采**；三份抄写进第 33 批合成一份 |
| K1 / K2 P2 副本组合 | 一致：墓碑、谱系各 36 格对称；两份都没自证过的 12 格红（第 33 批按用户 2026-10-07 定「当读不出、这一小句不判」改） | 没打中：32 形换盘相同（「pairs 32 asym 0」） | **采** |
| K1 P3 测试尖度 | S2、S4 各红且只红点名那一条，CPU 变异 3078–3083 现跑全红；**冲突**：G4 钉不住 F-W 往下走时按同一长度读那一读（正推变异 E1 下 G4 一族存活）；**没钉**：F-KEY 的 extent 与 inode 内部孩子、F-ID-b 回落读法、I-4.9 那一形（E2、E4、E6、E7 存活） | — | **采**；补钉的几形进第 33 批（G4 那一读另造一形让中央映射那一读对得上，实现员报告停下第 4 条同向） |
| K1 / K2 P4 F-零槽 | **打中一处回绕**：守卫写成 `offset.y >> (32u - SLOT_SHIFT)`（>> 18），第 506 行 `first_slot + n` 是 u32 加法；槽号在 [2³²，2³⁶) 回绕、落在缺口就交全零单元，CPU 读不到红 I-2.1（只在坏镜像上） | **打中一处**：没写过的槽、下一槽写过、位置条目校验和是 32 KiB 全零的 CRC 时，CPU 按 32 KiB 读到「16 KiB 零加下一槽」红 I-2.1 一族，GPU 在 F-α 那一处（`crash_judge_checker.wgsl` 第 3125 行）拿 16 KiB 全零切片比 32 KiB、判对得上；改法 Z1–Z3 推的 | **两处都坐实**：回绕——冻结副本第 503–506 行，槽号装进 32 位要 `offset.y < 2¹⁴`，代码按 2¹⁸ 判，与它自己的注释「偏移 ≥ 2⁴⁶ 不认」对不上；F-α——复跑原样同上。都是 F-零槽引入的 CPU / GPU 分叉，只在坏镜像上。进第 33 批：守卫改按 `SLOT_SHIFT`、加法改 64 位；全零切片只按它自己那一槽的长度交、不拿去比更长的长度（照攻方 Z1–Z3 里选一个，零轮）；各补一条对拍流（正推报告第八节两条：槽号加 2³²、槽 2³²−1） |
| K1 P4 检查字 | 一致：两条全零单元的检查字与真落点同一函数，全零 CRC 与 Python 独立实现逐位相同 | 没打中（读码） | **采** |
| K1 P5、P7 | 一致 | — | **采** |
| K1 P6 钉值 | 复核不了（要编 checker 档） | — | **采**：根环线 10u / 10v 合后验证里钉值那一条照新数绿（新数 319137 / 381545 / 329390，是 10u 之后的） |
| K3 | 9 份各一格：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_tables.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/mutations.tsv` | — | **采** |

## 三、进下一批的改法（各自验收标准在派发时写）

1. P1：照攻方 allrbmap，一个单元在走读里的各把键（走过、诞生代号、合并、父单元、叶序、I-3.5 折叠表）一律按读回那一份的落点认，CPU、GPU 同改；钉 today 那 6 对与回退后共享叶容器的分叉形，两种盘序同判。`read_index_node` 与它的两个调用方一起改（第 32 批 P 停下的第 2 条）。
2. P4：守卫改按 `SLOT_SHIFT`、`first_slot + n` 改 64 位；F-α 那一处全零切片不拿去比更长的长度；补两条回绕对拍流与攻方那一形的对拍流，各配变异。
3. P2：「自证过」的三份 CPU 抄写合成一份；两份都没自证过时按用户定「当读不出、这一小句不判」改（CPU、GPU）。
4. P3：G4 那一读、F-KEY 的 extent 与 inode 内部孩子、F-ID-b 回落、I-4.9 那一形各补一形坏镜像与变异（正推 E1、E2、E4、E6、E7）。

allrbmap、Z1–Z3 被攻过零轮。

## 四、交用户

不交：这一轮的打中都有两条腿同向的改法，没有岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-07 不受影响：内部条目形态不改，打中的是 checker 走读认单元身份用哪个落点、GPU 读没写过的槽的边角 |

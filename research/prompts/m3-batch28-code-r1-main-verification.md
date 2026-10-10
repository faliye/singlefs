# 删除批第 28 批合入的代码：代码轮第一轮判决（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 H-B H-V T1 T2 T3 F-W F-IT X2 X11 X13 X14 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch28-code-r1-body.md`、`_m3-batch28-code-r1-diff.md`、`_m3-batch28-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch28-code-r1-snapshot/`（crates 5 份、kb 2 份）；冻结副本 `/tmp/claude-1000/m3-batch28-code-r1/tree/` | `snapshot-check.sh m3-batch28-code-r1`：「7 个文件都与开工时一致」。2026-10-06 本机卡死重启、开机清空 `/tmp`，冻结副本随之没了；主工作区那 5 份与快照逐份相同，照它原样重建，重建后 `sha256sum -c` 逐份对上 |
| 云端正推（Sonnet） | `research/prompts/m3-batch28-code-r1-sonnet-output.md`（217 行，sha256 b232e6a5…），模型 `…-sonnet-model/`。第一条腿在重启时被打断（只落了模型目录），第二条同立场腿照交接摘要接着做 | — |
| 云端攻方（Opus） | `research/prompts/m3-batch28-code-r1-opus-output.md`（254 行，sha256 4248d726…），模型 `…-opus-model/`。第一条腿在重启时被打断（已落 62 行），第二条同立场腿重建模型、复跑坐实已落盘的结论（更正一处：镜像 78 张实为 90 张）再接着做 | — |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推「核了 22 处引文，对上 22 处，没判 0 处」；攻方「核了 25 处引文，对上 25 处，没判 0 处」 |
| 主 agent 复跑 | 攻方模型拷进会话草稿目录 `m3b28-model/`（拷贝与原目录逐文件 sha256 相同），`SINGLEFS_REPO=<冻结副本> SINGLEFS_TOOLS=<仓根> rerun.sh <空目录> all` 再 `compare`；另跑攻方没跑的层 0 两段 `crash-s0m`、`crash-sdm` | 「SUMMARY_MATCHES 346 行」；内存撞顶 0 次。层 0 两段（攻方没跑、主 agent 补跑）：`RESULT crash S0-s4[M] layer0 states=393258 oracle_violations=0 failed_states=0 checker_red_states=[] probe[calls=786522,other_class_returned=0,copy_same_class=1573044,copy_other_class=0,hb_inode=0,hb_container=0,hb_red=0,location_checksums_differ=0,copy_unverified=0]`、`RESULT crash SD-s4[M] layer0 states=327711 oracle_violations=0 failed_states=0 checker_red_states=[] probe[calls=1638561,other_class_returned=0,copy_same_class=3277122,copy_other_class=0,hb_inode=0,hb_container=0,hb_red=0,location_checksums_differ=0,copy_unverified=0]`——72 万多个合法崩溃状态零红，但探针显示 H-V 那一支（`other_class_returned=0`）与 H-B 那一支（`hb_*=0`）一次都没走到，这两格在合法状态上的「没打中」是空的，只说明合法流碰不到这两支 |
| 读代码 | 冻结副本 `crates/singlefs-checker/src/walk.rs` 第 4170–4190 行、第 2530–2547 行、第 2945–2955 行；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 1377、1410、1501、1589–1640、1809、2683、3129 行 | 见第二节 |

被判的 5 份文件（K3），每份一格判到。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 H-B | 一致：CPU 三棵树读不出孩子时判的就是只比父条目两段的那一小句，看孩子头的小句仍不判；GPU 两支读码同判 | — | **采** |
| K1 H-V | 一致：真值表 1764 态 CPU 新 = GPU 新 = 判决字面；两份各是不同别类与槽数归 C739 | — | **采** |
| K1 替换 7 行 | 一致：4 行旧写法真等价（真值表 0 / 1764 态不同），新替换文真改行为；2 行新锚点命中 1 次且红；GPU 1 行没跑 | — | **采** |
| K1 测试 | **冲突**：inode 读不出孩子那条只有「身份出生树加 1」一个方向，`==` 改 `>=`（X2、X11）整份 149 条都不红；「第一份头类标签不登记 / 头读不出、第二份自证成别类」（X13、X14）没有测试，那两处 `continue` 改 `break` 整份不红 | — | **采**：测试取样点不敏感（`.claude/rules/mutation-sampling.md` 第三类），补「子指针出生树加 1」与 X13、X14 两形进第 30 批 |
| K2 H-V 两份都看之后判错 | — | **打中**（坏镜像）：一份是本类且自证过、带子树里的违例，另一份自证成别类；第 28 批之后本类那一份的子树整片不走，20 格里漏判 16 格（之前漏 8 格，攻方 F-W 漏 0 格）；core 读者恰用本类那一份（`FileRead(3000)`、`UnitMalformed`） | **坐实**：冻结副本 `walk.rs` 第 4183–4186 行只要有一份自证成别类就交回它，调用方第 2546 行判 I-9.2 后 `continue`，本类那一份不往下走；core 照条目那一类长度逐份读、对得上就用（攻方引 `recovery.rs:384`）。违反 I-9.7 / I-9.15 / I-9.26 / I-9.4 / I-9.13 / I-1.7 字面（被读者真用的那片里有违例而报不适用或成立），口径同 m3-batch22-code-r1 正文第五节漏判。进第 30 批：照攻方 F-W（零轮）——有一份本类自证过就判条目那一条红、再照类型段那一支往下走（槽数按类型段那一类登记），CPU 三棵树与 GPU 两处（攻方引第 2640、3106 行之后的 `continue`）同改；F-W 同时定了 C739 后一半的取法（一份本类一份别类时槽数按类型段那一类），随第 30 批写回 C739 与 I-9.2 / I-9.17 / I-9.23 读法，被攻过零轮 |
| K2 H-V 合法状态误红 | — | 没打中（一次抽样）：合法终态 2 800 张 | **记「没打中（一次抽样）」**；层 0 两段 72 万多个崩溃状态里这一支一次都没走到（第一节复跑那一格），合法状态上走不到它 |
| K2 H-B 合法状态误红 | — | 没打中（一次抽样）：合法终态里 H-B 那一支一次都没走到 | **记「没打中（一次抽样）」**，并记明这一格在合法终态上是空的 |
| K2 CPU 与 GPU：第 28 批改的几支 | 读码一致 | 没打中（读码） | **记「没打中（一次抽样）」** |
| K2 CPU 与 GPU：实例表链上的 I-1.2 | — | **打中**（读码加 CPU 期望，第 28 批之前就有）：CPU 链上每一片判指针与头的出生身份，GPU 链循环不判；实例表指针头部诞生代号 / 实例 / 尾段各改一处，CPU 只红 I-1.2，GPU 推的三形都不红 | **坐实**：冻结副本 `crash_judge_checker.wgsl` 里 `INVARIANT_I_1_2` 只在第 1410 行置，所在的 `w_judge_birth_identity`（第 1377 行）只有第 1501、1809、2683、3129 行四处调用，链循环第 1589–1640 行一处没有；`walk.rs` 第 2953 行链上每一片都判。进第 30 批：照攻方 F-IT（零轮）链循环里只判 I-1.2 那一半，对拍补三形 |
| 第二次抽样：m3-batch27-code-r1 的 J1 | — | 没打中（读码，换了形） | **第二次也没打中，记「没打中」** |
| 第二次抽样：m3-batch27-code-r1 的 CPU 与 GPU | — | 打中，即实例表链 I-1.2 那一格 | **第一次「没打中（一次抽样）」被这一次推翻**，按上一格判 |
| K3 | 5 份各一格 | — | **采** |

## 三、进下一批的改法（各自验收标准在派发时写）

1. H-V 子树：照 F-W，CPU 三棵树「孩子是另一类」那一支见本类自证过的就照类型段往下走、槽数按类型段那一类；GPU 两处同改；攻方 1.1 那几形做成坏镜像（两种盘序同判、子树里的违例红）。第 29 批也改这一支（判 I-2.1），第 30 批在第 29 批合入之后取副本。
2. 实例表链 I-1.2：照 F-IT，GPU 链循环判出生身份那一半；对拍补攻方三形。
3. 测试尖度：inode 读不出孩子补「子指针出生树加 1」一形（钉住 X2、X11）；补「第一份头类标签不登记 / 头读不出、第二份自证成别类」两形（钉住 X13、X14）。
4. 写回（随第 30 批）：C739 后一半照 F-W 的取法写进 I-9.2 / I-9.17 / I-9.23 读法，C739 前一半（两份各是不同别类取哪一类）仍开着。

## 四、交用户

这一轮没有要交用户的岔路：打中的是实现偏离不变量字面与 CPU / GPU 判得不同。F-W、F-IT 被攻过零轮；F-W 定下的 C739 后一半取法照「读者用哪一份就走哪一份」，在第 30 批代码轮里攻。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-06 不受影响：内部条目形态不改，打中的是 checker 走哪一份副本与 GPU 链循环少判一条 |

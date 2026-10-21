# m4-snapshot-r1 攻方腿（Opus）报告（2026-10-13）

立场：假设主 agent 的倾向是错的，造可达的历史去打穿每个候选。九格都写；每格至少一段写成「操作序列 + 崩溃点」的具体历史，打不中的写「没打中」与找过哪几种历史。

## 复跑

模型目录 `research/prompts/m4-snapshot-r1-opus-model/`（纯 Python，不编 Rust，不读 crates/）。复跑（整条经内存包装，单线程，估约 14 分钟：深度 6 的 20 个配置串行约 5 分钟、同一次发布那 4 个深度 5 约 1 分钟、随机 7 个约 8 分钟）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-snapshot-r1-opus-model/tools/rerun.sh --repo /home/fy5090/.local/share/singlefs/work/m4-predev --frozen /home/fy5090/.local/share/singlefs/work/m4-predev/research/prompts/m4-snapshot-r1-snapshot --work <新的空目录>
```

末尾四行 `name=rerun_compare file=<文件> identical=true` 即与 `out/` 逐字节相同（比之前去掉 `wall_ms=` 行）。冻结副本只当参数传（模型不读它）。

| 文件 | sha256 |
|---|---|
| `snapmodel.py` | `d466c80b72441dcf9b8ca99bc37f776aa997eb8e21210e884f3b6325b847d619` |
| `trace.py` | `2c289bc3fff5f740078415c2e34ef75c7644cbb84271028986d5148dc8f42d63` |
| `tools/rerun.sh` | `a0b962d7ade4c0b04378d7af189a81965c9c3710819c07df8bfc6049e45266b3` |
| `tools/rerun_steps.py` | `129636d5e218fab1029cd0aa879021d9a899efc2be6260a82d48e36408e74536` |
| `out/depth6-indep.jsonl` | `e9e5f9cf373b82b0c7385891b9650448299c9360dad566dd8d33f40bcf6d8165` |
| `out/depth5-same.jsonl` | `fdaf8a36edd0309b79f72e7968c475789bfc5beff7603f2e7ba50e7f69b9a129` |
| `out/random16.jsonl` | `992ad270f281348eaa04b47cdab6c8cd7290593c1f9c0991e9403c542a0b8a7e` |
| `out/traces.list` | `2bc8af4f82e133a948eed24fffef205a874bd139daede952a6943f6cc1cd7207` |
| `out/traces.txt` | `bf6a9dfc2d5748dc6028ee62699e54393afa3bb55badf49fe61eec1b4dac6e99` |
| `out/depth6-same-before-the-rollback-split-knob.jsonl` | `f787f206f4bf07df6d42e7037254ca2e89414fa976fc958f764409814c456d7d` |

最后一份是加「回退树表分法」旋钮之前那一版模型上同一次发布 4 个配置的深度 6（每个约 4.5 分钟，前台跑不完 240 秒，复跑脚本里不含，正文只当旁证引）；那 4 个配置不碰回退分法旋钮，代码路径相同，数与深度 5 同向。

## 各格判定一览

| 格 | 判定 | 打中的候选 / 读法 | 最短历史（模型标签，含义见「模型」一节） | 量的 / 推的 |
|---|---|---|---|---|
| S1 | 打中 | 甲（没有 S.txg 字段，按快照条目的诞生 txg 读）丢数据、漏空间；甲 下 I-9.27 放宽版对归属错恒真、I-1.3 在合法镜像上红 | `S SS X0 D C`：S1、S2 的三个块被放 | 丢数据量的；I-9.27 / I-1.3 推的 |
| S2 | 打中 | 同一次发布 + 杀块拿这次发布写进去的新 prev：`death == S.txg` 那一行钉错、checker 全绿；同一 checkpoint 两个快照 + 链上并列次序不按建的次序：丢数据 | `W0+S`；`SS X0 D` | 量的 |
| S3 | 打中（不分辨臂） | 甲、乙 一样：① 中央映射每版按现行版单元重建，快照独占单元的映射条目被删，删快照「经映射释放」查不到；② 池级元数据单元也走判定：钉错、checker 全绿；③ 满盘时快照下 unlink 卡住、「先走空」把删快照也拒掉 | ① `S W0 X0 C`（推的）；② `SS D`；③ 见正文 | ② 量的；①③ 推的（①的今天行为是代码与现成断言） |
| S4 | 打中 | 甲 + 「快照条目写成销毁中」载体（lo 每批现取）丢数据；意图到最后一批之间 I-3.6 在合法状态上红；乙 + 同一载体、五样写死的载体都不中 | `S X0 SS D C`；`S D X0` | 量的 |
| S5 | 打中 | I-3.6 与 I-3.5 ③④ 照字面：对两类钉错恒真、对回退丢数据恒真；I-3.6、I-3.9 在合法状态上红；I-3.7 在甲（链派生）下没有第二来源 | `W0+S`、`SS D`；`S W0 R-1 R-1`；`S D X0`、`S X0 D` | I-3.7 推的，其余量的 |
| S6 | 打中（推的） | 三条只带一个快照的流罩不到模型找出的任何一种丢数据形状；只罩得到 S5 的合法态误红 | — | 推的 |
| S7 | 打中（推的） | 只比读出内容的差分，要等被误放的槽被复用才红 | — | 推的 |
| S8 | 打中 | 今天的回退（释放集与复活集只看活头、树表除头的两条外照抄现行版）在甲（树 ID 守卫在）与乙（守卫不触发）上都丢数据；「拒」挡不住；目标版里销毁中的快照算引用 → 漏空间 | 乙 `S R-1 D`；甲 `S W0 R-1 R-1`；`S X0 D R-3 R-1` | 量的 |
| S9 | 丢数据方向没打中 | 只读快照不用 livelist、不付 O(N)、旁表恒 0 条按 1 算，照条款走得通；甲 把快照做成「头」的样子，在 D5 已定项 18 第 1 条上留歧义 | — | 推的 |

## 模型（这一腿的唯一量具，只在它上面量过）

`snapmodel.py` 是 D5（快照 / 空间记账机制） 判定的抽象模型，不是 crates/ 的实现：今天 core 没有任何快照路径（背景材料第二节第一条），能量的只有「条款字面 + 候选读法」本身。

- 状态：一个两单元文件、一个用户可见树节点（extent / inode 根的代表，被快照捕获）、两个池级元数据单元（中央映射 / 记账 / 分配记录树的代表，不被快照捕获：一个每次发布都换，一个只在 W1 与 D 时换）、`prev(头)`、快照表、deadlist（条目 = (纪元, 块)，纪元取杀块那一刻的 prev）、已分配集与已释放表、全部已发布版本（给回退与 I-3.9 用）。
- 一次发布原子（D16 已定项 4），崩溃只落在发布边界；崩溃后挂载把销毁意图续做到底（承诺类）。一批放 1 条；最后一批同一次发布删快照条目。
- 标签：`W0`/`W1` 覆盖写第 0/1 个数据单元；`D` 删文件（一次发布杀掉全部数据单元与节点）；`S` 建一个快照、`SS` 同一次发布建两个；`W0+S`、`D+S`、`W0+SS` 是同一次发布模式下快照点钉在这次数据变更之后；`Xk` 删快照 k 的写意图那次；`B` 一批；`C` 崩溃并重挂、续做到底；`R-k` 管理员回退到往前第 k 版（向前发布一版）。初始一版 txg 1（文件已在）。
- 地面真相：`loss` = 某块被释放时仍被活头或某个活快照（不在销毁中的）引用；`double_free`；`misdl` = deadlist 里有一条的块没有任何快照引用（钉错 = 漏空间 + 归属错）；`leak_final` = 把剩下的快照按最老先、最新先各删一遍并续做到底之后，还有已分配而活头不引用的块；`loss_in_cleanup` = 那一遍里丢数据。
- checker 那一侧按条款字面判：I-3.6（各侧份按 D5 已定项 12 那张表的「哪一份」分，判 `birth ≤ prev(那一侧)`）；I-3.5 ③（被活快照引用、不被活头引用的块恰好一条，且落在「引用它的最晚那个快照的下一侧」那一份）、④；I-3.9 两种走读（`walk_in` 走销毁中的快照、`walk_ex` 不走）。`blind_*` = 地面真相坏而 I-3.6、I-3.5 ③④ 全绿；`*_red_legal` = 地面真相全好而那一条红。
- 用户动作不写死：每一步把所有可做的动作（写哪个单元、删文件、建几个、删哪个快照、批 / 崩溃、回退到往前第 1–3 版）全展开。穷举是每个前缀都判。
- 旋钮：`BASE` 取条款字面不出错的那一个值，别的值是候选或实现读法（`snapmodel.py` 的 `BASE` 与 `CONFIGS` 两段逐个注释了）。

取样与挂钟（3b）：先跑一小段估——两个基线配置深度 5 共 195 544 个态 15 258 ms（约 13 000 态/秒），据此估深度 6 独立发布每个配置约 16 秒、同一次发布每个约 2 分钟；随机 3000 条长 14 的历史 0.7–0.9 秒。正式跑：独立发布 20 个配置深度 6，5 件并行各 4 个，每件 59 867–64 713 ms；同一次发布 4 个配置深度 6 先跑过一次实测每个 254 046–279 436 ms（估少了一半多，2 392 622 个态），超过前台 240 秒，最终那一版缩到深度 5（4 件并行各 15 172–17 039 ms）；随机 7 个配置各 200 000 条长 16（种子 7），7 件并行各 52 835–68 056 ms。缩的只有同一次发布那 4 个的历史长度（6 → 5）。

崩溃状态：模型的崩溃点是发布边界。`enumerate_layer0` 一个态都没跑——core 里没有快照的写路径，录不出快照流；这一腿的 layer0 态数是 0。

原样输出（`out/` 三份 jsonl 的计数，脚本现数）：

```
depth6-indep base-indep 215513 GT {} CK {'i36_red_legal': 8023, 'i39in_red_legal': 6539, 'i39ex_red_legal': 1826}
depth6-indep a1-jia-stxg-from-birth 215402 GT {'loss': 702, 'misdl': 1458, 'leak_final': 66148, 'loss_in_cleanup': 38350} CK {'i36_red_legal': 236, 'i39in_red_legal': 3636, 'i39ex_red_legal': 18}
depth6-indep a1c-jia-stxg-from-birth-prev-creation 215513 GT {} CK {'i36_red_legal': 8023, 'i39in_red_legal': 6539, 'i39ex_red_legal': 1826}
depth6-indep a2-jia-stxg-from-chain 215513 GT {} CK {'i36_red_legal': 8023, 'i39in_red_legal': 6539, 'i39ex_red_legal': 1826}
depth6-indep a4-meta-judged 215516 GT {'misdl': 67119} CK {'blind_misdl': 61587, 'i36_red_legal': 2535, 'i39in_red_legal': 33769, 'i39ex_red_legal': 29938}
depth6-indep a5-node-release-control 215412 GT {'loss': 130675, 'leak_final': 24789} CK {'i39in_red_legal': 3330}
depth6-indep a6-flag-explicit 215513 GT {} CK {'i36_red_legal': 8013, 'i39in_red_legal': 6640, 'i39ex_red_legal': 1943}
depth6-indep a7-flag-jia-birth 215414 GT {'loss': 1074, 'misdl': 1443, 'leak_final': 64961, 'loss_in_cleanup': 39358} CK {'i36_red_legal': 296, 'i39in_red_legal': 3652, 'i39ex_red_legal': 24}
depth6-indep a7c-flag-jia-chain 215378 GT {'misdl': 1479, 'leak_final': 98493} CK {'i36_red_legal': 296, 'i39in_red_legal': 3652, 'i39ex_red_legal': 24}
depth6-indep a9-rb-release-live 215513 GT {'loss': 8023, 'double_free': 24, 'leak_final': 1345, 'loss_in_cleanup': 6783} CK {'blind_loss': 6440, 'i36_red_legal': 7643, 'i39in_red_legal': 6466, 'i39ex_red_legal': 1787}
depth6-indep a10-rb-resurrect-live 215513 GT {'loss': 933, 'double_free': 20, 'loss_in_cleanup': 1149} CK {'blind_loss': 841, 'i36_red_legal': 7705, 'i39in_red_legal': 6396, 'i39ex_red_legal': 1701}
depth6-indep a11-rb-today 215513 GT {'loss': 8498, 'double_free': 44, 'leak_final': 1305, 'loss_in_cleanup': 8153} CK {'blind_loss': 7498, 'i36_red_legal': 7346, 'i39in_red_legal': 6340, 'i39ex_red_legal': 1677}
depth6-indep a11r-rb-today-refuse-diff 118444 GT {'loss': 3025, 'double_free': 18, 'leak_final': 9, 'loss_in_cleanup': 2869} CK {'blind_loss': 2620, 'i36_red_legal': 6435, 'i39in_red_legal': 5029, 'i39ex_red_legal': 1598}
depth6-indep a11d-rb-destroying-included 215513 GT {'leak_final': 842} CK {'i36_red_legal': 7860, 'i39in_red_legal': 5687, 'i39ex_red_legal': 1698}
depth6-indep a12-pass-final-control 215620 GT {'misdl': 243, 'leak_final': 3991} CK {'blind_misdl': 243, 'i35_red_legal': 3844, 'i39in_red_legal': 2800, 'i39ex_red_legal': 2737, 'unowned_legal': 4292}
depth6-indep a13-flag-no-interleave 113171 GT {} CK {'i36_red_legal': 1985, 'i39in_red_legal': 670, 'i39ex_red_legal': 676}
depth6-indep a8i-indep-tie-reverse 215654 GT {'loss': 9351, 'misdl': 315, 'leak_final': 55381, 'loss_in_cleanup': 41905} CK {'blind_misdl': 315, 'i36_red_legal': 4567, 'i35_red_legal': 4819, 'i39in_red_legal': 4879, 'i39ex_red_legal': 1508, 'unowned_legal': 967}
depth6-indep a14-rb-today-split-yi 255594 GT {'loss': 46249, 'double_free': 175, 'misdl': 512, 'leak_final': 517, 'loss_in_cleanup': 57663} CK {'blind_misdl': 313, 'blind_loss': 6381, 'i36_red_legal': 7805, 'i35_red_legal': 199, 'i39in_red_legal': 7464, 'i39ex_red_legal': 2377, 'unowned_legal': 193}
depth6-indep a14r-rb-today-split-jia 119185 GT {'loss': 3080, 'double_free': 63, 'misdl': 101, 'leak_final': 273, 'loss_in_cleanup': 18952} CK {'blind_misdl': 95, 'blind_loss': 2192, 'i36_red_legal': 6542, 'i35_red_legal': 163, 'i39in_red_legal': 5282, 'i39ex_red_legal': 2010, 'unowned_legal': 169}
depth6-indep a14a-rb-split-only 255594 GT {'loss': 24430, 'double_free': 105, 'misdl': 512, 'leak_final': 15818, 'loss_in_cleanup': 56093} CK {'blind_misdl': 313, 'blind_loss': 441, 'i36_red_legal': 8221, 'i35_red_legal': 217, 'i39in_red_legal': 7680, 'i39ex_red_legal': 2514, 'unowned_legal': 211}
depth5-same base-same 173468 GT {} CK {'i36_red_legal': 10724, 'i39in_red_legal': 8149, 'i39ex_red_legal': 1629}
depth5-same a2s-jia-stxg-from-chain-same 173468 GT {} CK {'i36_red_legal': 10724, 'i39in_red_legal': 8149, 'i39ex_red_legal': 1629}
depth5-same a3-same-killprev-new 173630 GT {'misdl': 102585} CK {'blind_misdl': 43894, 'i36_red_legal': 1277, 'i35_red_legal': 25711, 'i39in_red_legal': 4600, 'i39ex_red_legal': 4035}
depth5-same a8-same-tie-reverse 173496 GT {'loss': 6652, 'misdl': 70, 'leak_final': 48994, 'loss_in_cleanup': 50055} CK {'blind_misdl': 70, 'i36_red_legal': 6684, 'i35_red_legal': 10085, 'i39in_red_legal': 5889, 'i39ex_red_legal': 1419, 'unowned_legal': 2204}
random16 base-indep 200000 GT {} CK {'i36_red_legal': 26660, 'i39in_red_legal': 29377, 'i39ex_red_legal': 20651}
random16 base-same 200000 GT {} CK {'i36_red_legal': 49797, 'i39in_red_legal': 47719, 'i39ex_red_legal': 35750}
random16 a1c-jia-stxg-from-birth-prev-creation 200000 GT {} CK {'i36_red_legal': 26660, 'i39in_red_legal': 29377, 'i39ex_red_legal': 20651}
random16 a2-jia-stxg-from-chain 200000 GT {} CK {'i36_red_legal': 26660, 'i39in_red_legal': 29377, 'i39ex_red_legal': 20651}
random16 a2s-jia-stxg-from-chain-same 200000 GT {} CK {'i36_red_legal': 49797, 'i39in_red_legal': 47719, 'i39ex_red_legal': 35750}
random16 a6-flag-explicit 200000 GT {} CK {'i36_red_legal': 26145, 'i39in_red_legal': 30239, 'i39ex_red_legal': 21539}
random16 a13-flag-no-interleave 200000 GT {} CK {'i36_red_legal': 12315, 'i39in_red_legal': 40833, 'i39ex_red_legal': 41220}
```

一个共用前提先说清：模型里 `BASE`（乙：条目里显式写 S.txg、与写进 prev(头) 的是同一个值；独立发布；五样写死的意图；回退取目标版整张树表、释放集与复活集按「目标版活头 ∪ 活快照 ∪ deadlist」算）在深度 6 穷举与 200 000 条长 16 的随机历史上丢数据、双放、钉错、终局漏空间都是 0。

## S1 只读快照在盘上是什么

**判定**：打中甲；乙在这一格的三条历史上不中。

候选（各一句）：甲是快照为新发号的一对（inode 树、extent 树）条目，根指针抄活头，头 ID 取自己的 inode 树 ID，`previous_snapshot_txg` 取捕获那一刻活头的值，**没有存 S.txg 的字段**；乙是树表加种类 14，一条条目装 S.txg、被捕获的头 ID、两根指针。甲 要拿 S.txg 只有两处可取：快照条目的诞生 txg（甲-诞生），或下一侧的 prev（甲-链）。

**H1-1（甲-诞生，丢数据）** 历史 `S SS X0 D C`（`out/depth6-indep.jsonl` 的 a1 行 `loss` 见证；同形的 `S X0 SS D C` 逐步状态在 `out/traces.txt`）：
1. txg 2：独立发布建 S0。S.txg 取「S 捕获的最后一个已发布 txg」（`.claude/kb/decisions/05-快照-空间记账机制.md:321`），= 1，prev(头) := 1；S0 条目诞生 txg = 2。
2. txg 3：同一次发布建 S1、S2（「同一 checkpoint 内建两个快照是允许的：它们捕获同一状态、其间 deadlist 为空」（`.claude/kb/decisions/16-发布语义.md:308`）），S.txg = 2，prev(头) := 2，诞生 txg 3。
3. txg 4：删 S0，写意图。lo 取 S0 的「S.txg」——甲 没有这个字段，取诞生 txg 2；hi 取下一侧 S1 的「txg」= 诞生 txg 3（「hi 在下一侧是快照时取它的 txg，是头时取写意图那个开放 checkpoint 的号」（`.claude/kb/decisions/05-快照-空间记账机制.md:277`））；P = prev(S0) = 0 传给 S1。
4. txg 5：删文件，三个块 birth 1 ≤ prev(头) 2，进 deadlist，纪元 2（「纪元 = 块被杀那一刻头的 `previous_snapshot_txg`，分桶就是这一段，不另设桶号字段」（`.claude/kb/decisions/05-快照-空间记账机制.md:275`））。
5. 崩溃，重挂续做：放 [2, 3) 里 birth > 0 的条目 → 三个块全放，而 S1、S2 还引用它们。按真 S.txg（1）区间是 [1, 2)，一条不放。

量的：a1（甲-诞生）深度 6 有 702 个态丢数据、66 148 个态终局漏空间；a1c（甲-诞生，但 prev(头) 也写成建快照那次的 txg，两处同值）与 a2（甲-链）都是 0。单快照的同形 `S W0 X0 C` 只漏不丢（`out/traces.txt` 里 `=== a1-jia-stxg-from-birth S W0 X0 C` 那一段：纪元 1 的两条在 [2, 4) 之外，删完快照还留在 deadlist）。

四句：① 分辨臂：甲-诞生 中；乙、甲-链、甲-诞生+prev 写建快照 txg 三个都不中。② 判别子看得到吗：写意图那一刻 S0 的真 S.txg 在盘上只剩「S1 的 prev 的旧值」，甲-诞生 不读它就看不到。③ 满足的分句：D5 已定项 14 那张表 `S.txg` 一行的「不是」列「不是「创建操作所在的 txg」」——诞生 txg 恰是创建那次发布的号。④ 改法：显式字段（乙）、甲-链、prev(头) 写诞生 txg 三样在打中的格上都不中（量过）；甲-链 另有 S4 / S5 的代价（见 H4-1、H5-4）。

**H1-2（甲，checker 对归属错恒真，推的）** I-9.27 在一张表里多于一条种类 2 时放宽：「表里有多于一条种类 2 条目时放宽成」（`crates/singlefs-checker/invariants/09-inode树结构.md:39`），代码按「inode_tree_identifiers.contains(」（`crates/singlefs-checker/src/walk.rs:4474`）判。
历史：建快照（甲）→ 写者把活头的 extent 树条目头 ID 写成快照的 inode 树 ID → 之后覆盖写 COW 的是谁的 extent 树全看实现；I-9.27 照样成立。乙 下每张表仍只一条种类 2，I-9.27 照严判。

两候选都要改的（代价，推的）：I-9.16「③ 每一种种类码（登记过的与没登记的都算）至多一条」（`crates/singlefs-checker/invariants/09-inode树结构.md:28`）与 core 的「pub(crate) fn tree_table_entries_each_kind_at_most_once(」（`crates/singlefs-core/src/recovery.rs:2354`）对甲（两条种类 2）与乙（两个快照即两条种类 14）都红；② 的发号次序对甲追加的那一对也红。「快照不许写」：甲 的条目与一个可写头同形，必须另有标志位；乙 的种类本身就是标志。

岔路：

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 甲-诞生 / 甲-链 / 乙 | 在放开用户动作的历史上，S.txg 的读法与 prev(头) 写的值不同源时丢数据（a1 702 态）；同源时三者都 0 | 入库装置在「建快照 → 同 checkpoint 再建两个 → 删最老 → 删文件 → 崩溃续做」这一段上各跑一次，三者丢数据态数各报 |

**H1-3（甲，合法镜像上红，推的）** 甲 的快照 inode 树条目（新树 ID）指着活头的 inode 根（码 2，头里的树 ID 是活头的），I-1.3「任一单元，其头记录的所属树 ID 与实际引用它的树一致」（`crates/singlefs-checker/invariants/01-块自描述.md:8`）的 ⚠️ 只豁免码 3 与数据单元，这一格在合法镜像上红；乙 不发新树 ID，不碰。

## S2 建快照那次发布写什么、切在哪

**判定**：打中两处：「同一次发布」候选在杀块该拿哪个 prev 上没有条款，取新值时 `death == S.txg` 那一行钉错、checker 全绿；同一 checkpoint 两个快照在运行时链上的并列次序没有条款，次序与建的次序相反时丢数据。「独立发布」候选在这两条上都不中。

候选：独立发布 = 建快照单独一次发布，只换根、树表与固定点单元，不带文件版本；同一次发布 = 快照点钉在一次带数据变更的发布末尾（D16 已定项 14 的字面形）。

**H2-1（同一次发布，杀块取新 prev，钉错且 checker 恒真）** 历史 `W0+S`（`out/traces.txt` 里 `=== a3-same-killprev-new W0+S` 那一段）：txg 2 一次发布里覆盖写第 0 个单元（旧块 b0、旧节点 b2，birth 1，这次被杀，death = 2），快照点钉在这次全部数据变更之后（「快照发布点必须与 checkpoint 边界对齐，且位置钉死在」（`.claude/kb/decisions/16-发布语义.md:308`）），S.txg = 2，prev(头) := 2。b0 不被 S 引用（「挂在不需要它的快照上，**空间泄漏 + 归属错**」（`.claude/kb/decisions/05-快照-空间记账机制.md:328`），death == S.txg 那一行）。发布路径的释放在规划这次发布时一起算（今天覆盖写与删除的释放在 `publish_admitted` 里「for placement in release {」（`crates/singlefs-core/src/transaction.rs:8141`）那一段，与写树表同一次规划），拿这次要写进树表的 prev = 2 去判：birth 1 ≤ 2 → 进 deadlist 纪元 2。同一段在 base-same（杀块拿发布前的 prev）上 b0、b2 直接进 defer（`out/traces.txt` 里 `=== base-same W0+S`）。

量的：a3 深度 5 共 173 630 个态，钉错 102 585 个，其中 43 894 个 I-3.6、I-3.5 ③④ 全绿（`blind_misdl`）；终局漏空间 0（删快照时 birth > P 把它们放掉）——是「钉到快照删掉为止 + 归属错」，不是永久泄漏。
四句：① 分辨臂：同一次发布 + 新 prev 中；独立发布与同一次发布 + 旧 prev 都 0（量过）。② 判别子：杀块那一刻（快照点之前）的 prev 在内存里就是旧值，看得到；中的是实现把判定挪到写树表那一步。③ 分句：边界表 `death == S.txg` 一行。④ 改法：取独立发布（同一次发布里没有数据杀块）或写死「快照点之前的杀块拿发布前的 prev」，两样都量过不中；checker 一侧 I-3.6 收不住它（见 S5）。

**H2-2（同一 checkpoint 两个快照，链次序反了，丢数据）** 历史 `SS X0 D`（`out/traces.txt` 里 `=== a8i-indep-tie-reverse SS X0 D`）：txg 2 建 S0、S1，S.txg 都是 1；删 S0，运行时从「头 → 活快照链」派生态取下一侧（「树表条目没有前后快照的链接」（`.claude/kb/decisions/05-快照-空间记账机制.md:278`）），同 txg 的两个按反序排，S0 被当成最新那个，下一侧是头，prev(头) := prev(S0) = 0（「在写意图的那个事务里就传给那一刻的下一侧，不许拆到以后的发布」（`.claude/kb/decisions/05-快照-空间记账机制.md:301`））；删文件，三个块 birth 1 > 0 直接释放，S1 还引用它们。条款里的并列规则只写给 checker：D16 已定项 14 那句「打破并列」说的是「checker 里的上一个快照」，运行时派生态的次序没有字。量的：a8i 深度 6 丢数据 9 351 个态；同一次发布的 a8 深度 5 丢数据 6 652 个态；次序取建的次序（BASE）0。
四句：① 两个 S1 候选都会中（乙 的显式 S.txg 两条相等，甲-链 按树 ID 排反而天然对），不分辨甲 / 乙，分辨的是「派生态排序键」。② 判别子：建的次序在盘上只在甲的树 ID、乙的条目序上，派生态不读就看不到。③ 分句：D5 已定项 12「下一侧怎么找」那一格。④ 改法：第一版不许同一 checkpoint 两个快照，或写死「同 txg 按建的次序」；两样都消掉这一格（后一样量过）。

`deleted_inodes` 队列：D8 已定项 16 第 6 条「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」（`.claude/kb/decisions/08-核心索引结构.md:482`），建快照若算用户改动就先走空；不算时快照里留一条 nlink = 0 的记录，它的块在之后的批里按 birth ≤ prev 进 deadlist，被快照钉住，属 D5 已定项 20 已接受的现象。没打中（模型里删文件是一次发布做完，没有分批，这一条只按条款推）。

岔路：

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 独立发布 / 同一次发布 | 同一次发布在「快照点之前的杀块取哪个 prev」写死之前，钉错态数非 0（a3 102 585 / 173 630）；写死之后与独立发布同为 0 | 入库装置在「覆盖写 + 建快照同一次发布」与「删文件 + 建快照同一次发布」两段上各量一次 |
| 同 checkpoint 两个快照：许 / 第一版不许 | 许而运行时链排序键不写死时丢数据（a8i 9 351 态）；写死建的次序为 0 | 排序键写进条款并在入库装置的「同一次发布建两个 → 删前一个 → 删文件」上量 0 |

## S3 快照下覆盖写与删文件：释放判定改在哪

**判定**：打中三处，三处都不分辨甲 / 乙（打中的是两候选共用的前提，照 evidence-discipline「打中不分辨臂」另立账先修）。

背景勘误：背景材料第二节第四条的 `:1861` 是写行那次发布里的释放（「allocator.release(released, txg);」（`crates/singlefs-core/src/transaction.rs:1862`），在「pub fn prepare_the_row_publish_on_a_version_without_file」（`crates/singlefs-core/src/transaction.rs:1823`）里）；覆盖写、顺序写与删除批走的是 `publish_admitted` 里「for placement in release {」（`crates/singlefs-core/src/transaction.rs:8141`）那一段。判定要插在「pub fn placements_to_release_via_mapping(」（`crates/singlefs-core/src/transaction.rs:3265`）与 8141 之间。

**H3-1（中央映射丢掉快照独占单元的条目，推的；今天的行为是代码与现成断言）** 历史 `S W0 X0 C`：
1. 建快照 S（甲 或 乙）。
2. 覆盖写第 0 个单元：旧数据单元 A 与旧 extent 根 birth ≤ prev(头)，进 deadlist。新一版的中央映射按这一版自己的单元重建，key 集合与现算的逐项相等（「中央映射树的形状按 resolve 现算的那一份映射 key 算过：这里装出来的 key 要逐项相等」（`crates/singlefs-core/src/transaction.rs:8612`）），A 的条目不在里面；现成用例断言的正是这个：「A 的映射条目一条都不留：释放走映射、条目删掉」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:338`）。
3. 删 S 写意图；4. 批：放 A。释放只经映射（「**释放一律经映射，不经提示**」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`）），deadlist 条目只有映射 key、没有落点（D5 已定项 12），映射里查不到；今天这一查的结局是「释放判定路径对已换下的单元报」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:544`）「不在映射」、一个落点都不放。删快照于是永远做不完：漏空间；它是承诺类意图，挂载续做到这一批报错时可写挂载交不回（推的，没跑）。
读快照今天先按提示读（「解引用先按提示读，读不到或校验和不对再查中央映射」（`crates/singlefs-core/src/mounted_read.rs:7`）），提示还对时读得出；同一里程碑的整理一搬快照独占的单元，D19 已定项 8 射程那句「搬一个被只读快照引用的码 2 / 码 3 要改写快照树的祖先」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:185`）的收口靠的是「只改映射一条条目」，条目不在就没得改。
四句：① 不分辨臂。② 判别子：写侧在杀块那一刻知道这个块进了 deadlist，看得到。③ 分句：D19 已定项 5 硬规则 1。④ 改法（只在推理上，被攻过零轮）：映射装「活头 ∪ 各活快照 ∪ deadlist」引用的单元，删快照的批与回退的释放才删条目；代价是映射条目数随 deadlist 涨、映射树容量准入要把它算进去；另一条「deadlist 条目带落点」被 D5 已定项 15 射程判过（带提示买不到东西、留双放的路）。

**H3-2（池级元数据单元也走判定：钉错，checker 恒真）** 历史 `SS D`（`out/traces.txt` 里 `=== a4-meta-judged SS D`）：txg 2 建两个快照，prev(头) := 1；txg 3 删文件，同时换下一个 birth 1、不被任何快照捕获的池级元数据单元（模型里的 meta2，代表映射 / 记账 / 分配记录树里不是每次都 COW 的那几片）；它也走 `birth ≤ prev` → 进 deadlist。量的：a4 深度 6 钉错 67 119 个态，其中 61 587 个 I-3.6、I-3.5 ③④ 全绿；终局漏空间 0（删快照时放掉）。对照：把用户可见树节点一律直接释放（a5）丢数据 130 675 个态，那一边 I-3.5 ③ 看得到（`blind_loss` 0）。
四句：① 不分辨臂（两候选都要回答「快照捕获哪几棵树」）。② 判别子：角色种类在 `placements_to_release_via_mapping` 里逐个 match，看得到。③ 分句：边界表之外——条款 D5 已定项 13「删除时把块的代号与活头记录的 `previous_snapshot_txg` 比」没限定是哪一类块。④ 改法：走判定的角色 = 快照捕获的那几棵树的单元，别的一律直接释放；推的，量过的只是两种划错各自的后果。

**H3-3（满盘 + 快照下 unlink：先走空把删快照也拒掉，推的）** 历史：写到盘满 → 建快照 → unlink 那个大文件（意图那次发成）→ 批：每批要 COW 的 extent 节点都 birth ≤ prev，被钉住、一槽不还，deadlist 还要新节点；落点那一判以空间拒 → 推抬 F 再试，回收不出东西 → 「推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够」（`.claude/kb/decisions/08-核心索引结构.md:482`）→ 用户要删快照腾空间，删快照也是用户改动，先走空 → 拒。唯一能腾空间的那条路被挡住。D16 已定项 15 射程那句「释放空间这个操作本身不许申请空间」（`.claude/kb/decisions/16-发布语义.md:329`）在快照下不成立：快照下的删除净占空间。两候选一样中。改法（推的）：删快照不走「先走空」，或删快照的意图与批在准入里排在队列前；都没量。

其余要答的几样（推的）：deadlist 树按码 2 多层树建，条目 43 字节、value 空；一批 unlink 至多 144 个数据单元加被截那一片下段叶与祖先进 deadlist，约 150 条 × 43 字节，一片 16 KiB 叶装得下。记账：进 deadlist 的单元仍在已分配、不进 defer，「已分配减 defer 等于最新根走读」那条 I-3.11「等于从最新有效根走读到的、这块盘上被引用的槽数」（`crates/singlefs-checker/invariants/05-空间记账.md:18`）的走读必须走快照的树，否则合法镜像上红。

边界表在这一格：`birth == S.txg` 取 `S W0 X0 C`——b0 birth 1 = S.txg 1，被杀时进 deadlist，删快照时放：乙、甲-链 对（`out/traces.txt` 里 base-indep 与 a2 那两段），甲-诞生 漏（lo = 2，纪元 1 不在区间里）。`death == S.txg` 与 `birth == death` 见 S2 的 H2-1（同一次发布才有）。

岔路：

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 映射装快照与 deadlist 引用的单元 / deadlist 条目带落点 | 后者在整理搬过一次之后按旧落点放（D5 已定项 15 射程）；前者映射条目数涨 deadlist 条数 | 入库装置量「快照下覆盖写 N 次 → 删快照」映射条目峰值与删快照能否做完 |
| 删快照走 / 不走「先走空」 | 满盘 + 快照 + unlink 之后删快照能否做成 | 准入装置上那一段历史做一次 |

## S4 删快照：意图、分批、deadlist 并到更新的一侧

**判定**：打中甲 + 「快照条目写成销毁中」载体（丢数据）；乙 + 同一载体与「五样写死」载体在放开用户动作的历史上都不中；条款写的「崩在中间每个状态 I-3.6 成立」不成立（两载体一样）。

候选：载体-全 = 写意图那次把 头、lo、hi、P、续做点 写死（住 logged_ops 或别处）；载体-标记 = 只把快照条目标成销毁中，lo 从条目取、P 取条目自己的 prev、hi 每批从「头 → 活快照链」现取。

**H4-1（甲 + 载体-标记，丢数据）** 历史 `S X0 SS D C`（`out/traces.txt` 里 `=== a7-flag-jia-birth S X0 SS D C`）：txg 2 建 S0（S.txg 1）；txg 3 删 S0 写意图，下一侧是头，prev(头) := P = 0，条目只标销毁中；txg 4 同一次发布建 S1、S2（S.txg 3），prev(头) := 3；txg 5 删文件，三个块 birth 1 ≤ 3，进 deadlist 纪元 3；崩溃、重挂续做 S0 的批：lo 只能取 S0 的诞生 txg 2（甲 没有 S.txg 字段，写意图之后下一侧的 prev 已改成 P，链上派生不出来），hi 现取下一侧 S1 的「txg」= 诞生 txg 4 → 放 [2, 4) 里 birth > 0 的 → 三个块被放，S1、S2 还引用。量的：a7 深度 6 丢数据 1 074 个态；甲-链 + 标记（a7c）不丢、终局漏空间 98 493 个态（lo 只能落回诞生 txg）；乙 + 标记（a6）深度 6 与 200 000 条随机历史都 0；载体-全（BASE）0。
四句：① 分辨臂：分辨「S.txg 在不在盘上」，与 H1-1 同一个判别子，在这一格上甲两种读法都输。② 判别子：写意图之后 S.txg 只在被改掉之前的下一侧 prev 里，载体-标记不存它就再也看不到。③ 分句：D5 已定项 12「销毁 S」那一格写意图 (头, lo = S.txg, hi, P)——lo 是意图的一部分，载体-标记丢了它。④ 改法：载体-全 / 乙 + 标记 都不中（量过）。

字节（推的）：载体-全 = 头 8 + lo 8 + hi 8 + P 8 + 续做点（最后放到的 deadlist key 43）= 75 字节；乙 + 标记 = flags 里一位（lo 就是条目的 S.txg，P 就是条目的 prev；hi 现取在 a6 上量过不出错；批施加前复核条目还在，续做点可以不存）。甲 + 标记 至少还要在预留里存 lo 8 字节。

**H4-2（崩在中间：I-3.6 在合法状态上红）** 历史 `S D X0`（`out/traces.txt` 里 `=== base-indep S D X0`）：建快照（S.txg 1）→ 删文件，三个块进 deadlist 纪元 1 → 写意图，prev(头) := P = 0。此刻这三条落在头那一份 [0, ∞)，birth 1 > prev(头) 0，I-3.6「任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）」（`crates/singlefs-checker/invariants/05-空间记账.md:12`）红，直到最后一批放完。量的：base-indep 深度 6 有 8 023 个合法态 I-3.6 红；`trace.py --split-i36 base-indep 6` 的原样输出 `name=base-indep depth=6 i36_red_legal_mid_destroy=8023 i36_red_legal_quiescent=0`——全部落在意图与最后一批之间（在最终那一版模型上跑，约 20 秒；复跑脚本不含这一步，命令照抄即可）。两载体一样红。
改法（推的）：I-3.6 对「落在某个销毁意图 [lo, hi) 里、birth > P」的条目豁免——这要 checker 读得到 lo、hi、P：载体-全 住 logged_ops 时 logged_ops 今天没有字段表（C364），checker 读不到；乙 + 标记 读条目就有。

C337（被销毁的头何时不算活引用）：模型照 D5 已定项 20 的例外「例外：被销毁的快照从写意图那一刻起不算活引用（已定项 12）」（`.claude/kb/decisions/05-快照-空间记账机制.md:433`）判，写意图之后被杀的、只被它引用的块直接释放；这一读法让 I-3.9 在合法历史上红（S5 的 H5-3）。

删快照被空间拒推抬 F：模型没建空间，没量；满盘下的卡死见 H3-3。批大小与只供测试的开关：模型一批 1 条，最后一批与删条目同一次发布，穷举里每个批边界都崩过，地面真相 0。

岔路：

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 载体-全（logged_ops）/ 乙 + 标记 / 甲 + 标记 + 预留存 lo | 甲 + 标记 不存 lo 丢数据（a7 1 074 态）；另两样 0；checker 能否读到 lo、hi、P | logged_ops 字段表（m4-gc-r1 那一轮）定下之前，checker 读不读得到意图的那几样 |

## S5 checker：I-3.5 ③ ④、I-3.6、I-3.7 的判法与必红世界

**判定**：打中。照条款字面的 I-3.6 与 I-3.5 ③④ 有三种恒真、两条（I-3.6、I-3.9）在合法状态上红；I-3.7 在甲-链下没有第二来源。

**H5-1（钉错，三条全绿）** S2 的 `W0+S`（a3）与 S3 的 `SS D`（a4）：被钉错的那条 deadlist 条目 birth ≤ 它所在那一侧的 prev，I-3.6 成立；I-3.5「③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中」（`crates/singlefs-checker/invariants/05-空间记账.md:11`）只从「被某根引用的块」出发，钉错的块没有任何快照引用，③ 根本不走到它；④ 只管被活头引用的块。量的：a3 深度 5 `blind_misdl` 43 894 个态，a4 深度 6 61 587 个态。缺的是 ③ 的逆：「deadlist 里每一条的块被它纪元那一刻最新的活快照引用（销毁中的快照只认它挂起区间里的条目）」——在模型里这一条与地面真相 `misdl` 同义，所以它的判别力只是推的；必红世界：同一次发布里覆盖写 + 建快照，被换下的块进 deadlist。

**H5-2（回退丢数据，三条全绿）** 历史 `S W0 R-1 R-1`（`out/traces.txt` 里 `=== a11r-rb-today-refuse-diff S W0 R-1 R-1`）：第二次回退之后 b0、b2 已释放而 deadlist 里各有恰好一条，③ ④ 与 I-3.6 都成立。量的：a11r 丢数据 3 025 个态里 2 620 个三条全绿，a11 8 498 里 7 498，a10 933 里 841。看得到它的只有「被活快照引用的落点不许带已释放标志」——即 I-3.1 / I-3.11 的走读把快照的树也走进去（I-3.11「等于从最新有效根走读到的、这块盘上被引用的槽数」（`crates/singlefs-checker/invariants/05-空间记账.md:18`）照今天只走活头时，快照独占单元是「已分配而走读不到」，在合法镜像上红）。

**H5-3（合法状态上红）** ① I-3.6：H4-2，8 023 个态，全部在意图与最后一批之间。② I-3.9「它的释放代必须落在区间」（`crates/singlefs-checker/invariants/05-空间记账.md:16`）（代码「let in_the_witnessed_interval = record.generation > last_referencing_txg」（`crates/singlefs-checker/src/walk.rs:5649`））两种走读都红：走销毁中快照的读法在 `S X0 D` 上红——写意图之后删文件，三个块照 C337 当场释放（释放代 4），而快照条目还在树表里、txg 4 那条根还引用它们；不走的读法在 `S D X0 C` 上红——块在批里放（释放代 5–7），最早不再引用它们的根是写意图那次（txg 4）。base-indep 深度 6 分别 6 539 与 1 826 个合法态。改法（推的）：从 deadlist 放掉的块，释放代落在（写意图 txg, 最后一批 txg] 里。

**H5-4（I-3.7 在甲-链下恒真，推的）** I-3.7「盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少）」（`crates/singlefs-checker/invariants/05-空间记账.md:14`）：「既不多也不少」要一个第二来源。乙 有：每个快照的 prev 必须等于它前一侧的显式 S.txg，树表丢一条快照条目 S1 时 prev(S2) = S1.txg ≠ S0.txg，判得出。甲-链 的 S.txg 本来就从这条链派生，丢掉 S1 之后链照样自洽；S1 钉的那几条（birth ≤ S1.txg）落进 S2 那一份，birth ≤ prev(S2) = S1.txg，I-3.6 照样成立，删 S2 时 birth > P 放不掉它们——永久漏空间而全绿。必红世界（乙）：树表删掉中间那条快照条目。

各条判法（推的，与 I-3.1 共用一趟遍历）：遍历走活头与每个活快照的树（销毁中的不走）得 Reach；③ 按 D5 已定项 12 的份分侧；新 ⑤ 是 ③ 的逆；I-3.6 对挂起意图的区间豁免；输入取不到（deadlist 某片读不出、意图字段读不到）时报射程里判不了，不判违例——写侧「判据的任何一个输入取不到或不确定时，默认必须是「进 deadlist」」那条保守纪律在 checker 侧对应的是「不判」，不是「判成立」。GPU 判器：I-3.5 ③④、⑤、I-3.6、I-3.7 各一位，剩 36 位里用 4–5 位。

岔路：

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 加 ③ 的逆 / 不加 | 不加时钉错（a3、a4）三条全绿 | 入库 checker 上 `W0+S` 与 `SS D` 两份坏镜像各红一次 |
| I-3.9 对 deadlist 放掉的块的区间 | 照今天字面两种走读都在合法历史上红 | 入库 checker 上 `S X0 D` 与 `S D X0 C` 两段合法历史各绿一次 |

## S6 崩溃流与态数

**判定**：打中（推的）。正文要的三条流——建快照、快照下覆盖写与删文件、删快照——各只带一个快照，模型找出的四种丢数据形状一种都装不下：H1-1 要三个快照、其中两个同一次发布（`S SS X0 D C`）；H2-2 要同一次发布两个（`SS X0 D`）；H4-1 要删快照写意图之后再建快照（`S X0 SS D C`）；H8-1 / H8-2 要回退（`S R-1 D`、`S W0 R-1 R-1`）。单快照流里能出现的只有 S5 的合法态误红：`S D X0 C`（建快照 → 删文件 → 删快照）正是「删快照」流的形状，checker 照今天字面会在这条流的合法崩溃态上红（H4-2、H5-3），这条流与 S5 的改法必须一起落地。

建议的流（推的，接在多记录流之后照 unlink 流的接法）：① `S W0 X0 C`（边界表 birth == S.txg 一行；单快照）；② `S SS X0 D C`；③ `SS X0 D`；④ 删快照写意图之后再建一个、再删文件（`S X0 S D C`）；⑤ `S W0 R-1 R-1`（回退类，若第一版许回退跨快照）。

态数估法（推的，没录流）：全量展开的闭式是 1 加各段（2^k − 1）之和（`closed_form_state_count`）。今天一次覆盖写发布每盘写 12 个单元（「mkfs 2 + A 12 + B 12 个落点 × 2 盘（七个角色加分配记录树五个节点）」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:240`）），两盘 24 次单元写；若一个屏障段装下全部 24 次，这一段就是 2^24 − 1 ≈ 1.7 × 10^7 个态，低于全量展开的写数上限（工作树 scripts/crash/crash-amplification.sh 第 319 行的 FULL_EXPAND_UP_TO=30；主仓工作区那份行号不同）但高于这一腿约 10^6 的每次上限，所以这一腿一个都没跑。快照下的覆盖写比今天多写 deadlist 树那一路（根兼叶时 1 个单元 × 2 盘），删快照的每一批写 deadlist 根、树表、记账与分配记录那几片，量级与一次覆盖写相同；建快照那次（独立发布）只写树表、根与固定点单元，少于一次覆盖写。每个落点至多 64 次写（「pub const MAXIMUM_WRITES_PER_LOCATION: usize = 64;」（`crates/singlefs-checker-tier/src/crash_judge_tables.rs:54`））：deadlist 每次 COW 换落点，不撞；根槽与 journal 照旧。真数要录了流之后用 `layer0_state_count_with_torn_in_place_overwrites` 算。背景材料第二节「崩溃放量的上限」一条的两个行号（`:51`、`:222`）与今天的文件对不上，以上两处现查。

每条流的必红世界（推的）：① 把 birth == S.txg 的块当成「比快照新」直接放；② lo 读诞生 txg；③ 同 txg 按反序排；④ 载体-标记且 lo 不存；⑤ 回退释放集只看活头。

## S7 模型、RefFS 与随机历史

**判定**：打中（推的）。只比读出内容的差分对「被误放但还没被复用」的快照块是盲的：H8-2 那一段之后读快照，b0 的槽还在 defer 里、字节还在，读出来对；要等回收门槛越过释放代、槽被复用之后才读错，随机历史长度不够就永远不红。要加：每一步之后「每个活快照引用的落点在分配记录里是已分配」——这就是 H5-2 的走读，放在 checker 里比放在差分里便宜。

要加的状态：模型与 RefFS 各存每个活快照的冻结内容、销毁中标志；模型另存 prev(头) 与 deadlist（RefFS 只管语义，不存）。`HistoryOperation` 至少加：建快照（一个 / 同一次发布两个）、删快照（指定哪一个）、读快照、崩溃在删快照的批之间；与已有的 `RollBackWhileMounted`、`CrashRecoveryAbandoningTheNewestRoot` 交错——模型找出的每一种丢数据形状都只用到这几种动作（见 S6 列的最短历史）。`KNOWN_RED_FORMS` 不收 H4-2、H5-3 那两种合法态误红：它们是判据错，改判据，不登「已知红」。

岔路：无新的；动作集合由 S6 那张最短历史表定。

## S8 回退、冷启动与快照

**判定**：打中。正文给的两条路（拒 / 抹掉）都没提今天回退的另两样：释放集与复活集只看活头，树表除活头那两条外照抄现行版。这两样不改，甲、乙 都丢数据，「拒」挡不住。

今天的代码（现查）：释放集是「释放集：cur 引用的用户可见单元里，落点不在 R_old 引用的那几个里的」（`crates/singlefs-core/src/mount.rs:8108`）；复活只看「let target_units = user_visible_units_through_the_mapping(」（`crates/singlefs-core/src/mount.rs:8090`）；回退之后那一版的树表「别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的」（`crates/singlefs-core/src/mount.rs:7877`），只有「所以树表条目里 inode 树与 extent 树那两条换成目标那一版的」（`crates/singlefs-core/src/mount.rs:7879`）——prev(头) 住 inode 树那条（「`previous_snapshot_txg` 只写在该头 inode 树那条树表条目里」（`.claude/kb/decisions/05-快照-空间记账机制.md:229`）），于是取目标版的；快照条目与 deadlist 树取现行版的。树 ID 集合不同就拒的守卫是「TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided」（`crates/singlefs-core/src/mount.rs:8085`）：甲 的快照有自己的树 ID，守卫在快照集合不同时触发；乙 的种类 14 不进那十二棵树的号，守卫不触发。

**H8-1（乙 + 今天的回退，丢数据）** 历史 `S R-1 D`（`out/traces.txt` 里 `=== a14-rb-today-split-yi S R-1 D`）：txg 2 建 S（S.txg 1，prev(头) := 1）→ 回退到 txg 1：prev(头) 随 inode 树条目取目标版的 0，快照条目随树表照抄现行版、还在 → 删文件：三个块 birth 1 > 0，直接释放，S 还引用。量的：a14 深度 6 丢数据 46 249 / 255 594 个态。
**H8-2（甲 + 今天的回退，守卫在，丢数据）** 历史 `S W0 R-1 R-1`（`out/traces.txt` 里 `=== a11r-rb-today-refuse-diff S W0 R-1 R-1`）：建快照 → 覆盖写，b0、b2 进 deadlist → 回退到建快照那一版（快照集合相同，守卫放行；b0、b2 回到活头）→ 再回退到覆盖写那一版（回到回退之前，仍是候选根）：释放集 = 现行活头里不在目标活头里的 → b0、b2 被释放，而它们被快照引用、在目标版的 deadlist 里。量的：甲 + 今天全套（a14r）3 080 / 119 185；甲 + 守卫 + 取目标版整张树表但释放 / 复活照今天（a11r）3 025 / 118 444；乙 + 取整张树表 + 释放 / 复活照今天（a11）8 498。只换释放集（a9）8 023、只换复活集（a10）933 都不够；两个集合都按「目标版活头 ∪ 活快照 ∪ deadlist」算、树表整张取目标版（BASE，即正文的「抹掉」）为 0。
**H8-3（抹掉，但目标版里销毁中的快照算引用，漏空间）** 历史 `S X0 D R-3 R-1`（`out/traces.txt` 里 `=== a11d-rb-destroying-included S X0 D R-3 R-1`）：建快照 → 删它（写意图）→ 删文件（照 C337 三个块当场释放）→ 回退到第一版（三个块复活）→ 再回退到删文件那一版：那一版里快照还在、标着销毁中，若把它的块算进「目标版引用」就不放这三个块，续做的批在空 deadlist 上一条不放，删完快照三个块已分配而无人引用。a11d 终局漏空间 842 个态；销毁中的不算（BASE）0。

四句（H8-1、H8-2）：① 分辨的是回退的释放 / 复活集与树表分法，不分辨甲 / 乙（两边都中，守卫只改中的形状）。② 判别子：快照条目与 deadlist 在现行版与目标版的树表里都读得到。③ 分句：D5 已定项 14 的引用判据——被活快照引用的块被释放。④ 改法：「拒」（守卫）在 H8-2 上照中（量过）；「抹掉 + 两个集合含快照与 deadlist + 销毁中的不算」不中（量过，被攻过零轮）。

冷启动 / 崩溃恢复落到建快照之前的根：模型里一次发布原子、崩溃只按所选根的账走，快照与 deadlist 随所选根一起回去，免费成立；没打中（被抛弃时间线的影子账不随快照变，没另建）。I-3.5 出现规则的「按树继承下界」（推的）：乙 下回退复活一个快照，它的单元在 R 里出现而 R⁻ 里没有、birth 远早于 R⁻，(b) 只罩用户可见树的子树，种类 14 不是一棵树 → 在合法镜像上红；甲 下快照的树表条目是树，(b) 照常放过。

岔路：

| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 拒（守卫）/ 抹掉 | 释放 / 复活集照今天时两者都丢数据（a11r 3 025、a11 8 498）；按全部引用算时抹掉为 0 | 入库装置在 `S W0 R-1 R-1` 与 `S R-1 D` 两段上量 |
| 回退之后树表：整张取目标版 / 照今天分法 | 照今天分法乙丢数据 46 249 态 | 同上 |

## S9 只读快照用到 D6 的哪几样

**判定**：丢数据方向没打中。

逐样：每头一棵树（D6 已定项 1）——只读快照不是头，乙 不用；甲 借它的形状（新树 ID、inode 树条目），H1-1、H4-1、H5-4 都出在「借了形状没带 S.txg」上。引用计数旁表（D5 已定项 6）——第一版 0 条，判定里「没有条目按 1 算」（`.claude/kb/decisions/05-快照-空间记账机制.md:415`），n = 1 落到 birth 那一判，模型里就是这么走的，没打中。livelist（D6 已定项 3）——「销毁一个可写头要一个盘上的第二个结构」（`.claude/kb/decisions/06-快照实现模型.md:62`），只读快照一条不写，留给可写克隆。D6 已定项 5 的 O(N)——「这句豁免的主语是**销毁快照**」（`.claude/kb/decisions/06-快照实现模型.md:90`），而只读快照的删除是 deadlist 合并，不走 O(N) 反向查找，不用。

推的一处歧义：甲 的快照 inode 树条目带非 0 的 `previous_snapshot_txg`，与「可写头的 ID = 该头 inode 树的树 ID」（D5 已定项 9）、D5 已定项 18 第 1 条「每个可写头各持一份 `previous_snapshot_txg` 与一份 deadlist」放在一起，读者分不出这条是头还是快照——要标志位（S1 已列）。留给克隆的归属判断：livelist、旁表计数大于 1 那一支、分叉点闸、销毁头的第二道闸，四样都不进只读快照这一批。

## 这一腿自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| S.txg 显式存在快照条目里，且与写进 prev(头) 的是同一个值（乙；或甲 让 prev(头) 写诞生 txg） | H1-1、H4-1 | 量过：a1 702 → BASE / a1c 0；a7 1 074 → a6 0 |
| 同一次发布模式下，快照点之前的杀块拿发布前的 prev；或只做独立发布 | H2-1 | 量过：a3 钉错 102 585 → base-same / base-indep 0 |
| 运行时「头 → 活快照链」同 txg 按建的次序排；或第一版不许同一 checkpoint 两个 | H2-2 | 前一样量过：a8i 9 351 → 0；后一样推的 |
| 中央映射装活头 ∪ 活快照 ∪ deadlist 引用的单元，条目到删快照的批或回退释放时才删 | H3-1 | 推的 |
| 走 D5 判定的角色限于快照捕获的那几棵树的单元，池级元数据一律直接释放 | H3-2 | 量过：a4 钉错 67 119 → BASE 0 |
| 删快照不走「先走空」，或在准入里排在队列前 | H3-3 | 推的 |
| I-3.6 对挂起销毁意图 [lo, hi) 里 birth > P 的条目豁免（要 checker 读得到 lo、hi、P） | H4-2 | 推的（误红本身量过：8 023 态） |
| I-3.5 加 ③ 的逆（deadlist 每条都被该纪元最新的活快照引用） | H5-1 | 推的（模型里与地面真相同义，判别力没独立量） |
| I-3.1 / I-3.11 的走读走活快照的树；「被活快照引用的落点不许带已释放标志」 | H5-2 | 推的 |
| I-3.9 对从 deadlist 放掉的块：释放代落在（写意图 txg, 最后一批 txg] | H5-3 | 推的（误红本身量过：6 539 / 1 826 态） |
| 回退：树表整张取目标版；释放集 = 现行已分配 − 目标版（活头 ∪ 活快照 ∪ deadlist）；复活集 = 同一集合里已释放的；目标版里销毁中的快照不算引用 | H8-1、H8-2、H8-3 | 量过：a14 46 249、a14r 3 080、a11 8 498、a11d 842 → BASE 0 |

## 没打中的形状

- BASE（乙 + 独立发布 + 载体-全 + 抹掉式回退）：深度 6 穷举 215 513 个态（每个前缀都判）与 200 000 条长 16 的随机历史（种子 7），丢数据、双放、钉错、终局漏空间都是 0。动作集合：覆盖写两个单元、删文件、建一个 / 同一次发布建两个、删任一活快照、单批、崩溃续做、回退到往前 1–3 版。
- base-same（同一次发布 + 旧 prev）、a2 / a2s（甲-链，独立 / 同一次发布）、a1c（甲-诞生 + prev 写诞生 txg）、a6（乙 + 载体-标记，hi 每批现取）、a13（载体-标记 + 分批之间不插用户操作）：穷举（独立发布深度 6、同一次发布深度 5）与随机各 200 000 条，地面真相都 0。我原先以为载体-标记 hi 现取会在「写意图之后再建快照」上多放（a6），没打中。
- 崩溃恢复落到建快照之前：模型里免费成立（发布原子、账随所选根），没造出反例；被抛弃时间线的影子账没建。
- S2 里 `deleted_inodes` 队列非空时建快照：模型删文件一次做完，没有分批，没造。
- 边界表 `birth == death`：模型里没有一次发布之内生了又杀的块，三个候选上都没造出来（只按条款推：同一次发布 + 新 prev 时它与 `death == S.txg` 同形）。

边界表三行在各候选上的具体历史（`out/traces.txt`）：

| 行 | 乙 / 独立发布（BASE） | 甲-诞生 | 甲-链 | 同一次发布 + 旧 prev | 同一次发布 + 新 prev |
|---|---|---|---|---|---|
| `birth == S.txg` | `S W0 X0 C`：进 deadlist、删快照时放，对 | 同一段：纪元 1 不在 [2, 4)，漏 | 同一段：对 | `W0+S W0 X0 C`：对 | 同一段：对 |
| `death == S.txg` | 独立发布里没有同一次的数据杀块，不出现 | 同左 | 同左 | `W0+S`：被换下的直接进 defer，对 | `W0+S`：进 deadlist，钉错、checker 全绿 |
| `birth == death` | 没造出（见上） | 没造出 | 没造出 | 没造出 | 没造出 |

## 这一腿自己的限度

- 模型是 D5 条款的抽象，不是 crates/：没有空间、没有 defer 窗口的复用时刻、没有分批的 unlink、没有多层树；「丢数据」是「被释放时仍被活引用」，不是读出错字节。H3-1、H3-3、H5-4、S6、S7 全是推的。
- 文件只有两个单元，快照最多 2 + 若干个（深度 6 内），回退只到往前 3 版。深度 6 以外只靠随机抽样，且随机只跑了穷举里干净的那 7 个配置。
- checker 一侧的判据是我照条款字面写的一个读法（尤其 I-3.5 ③ 的「归属区间」按 D5 已定项 12 的份读）；入库 checker 落地时的读法可能不同。
- `CONV`（③ 的逆）在模型里与地面真相 `misdl` 同义，它的判别力没有独立证据。
- 只抽了一次样（一条攻方腿）；「没打中」各条按 three-way-inference「一条腿只抽一次样不算一次观测」那一节不能单独支撑结论。
- 同一次发布那 4 个配置的最终数是深度 5；深度 6 只在加回退分法旋钮之前那一版模型上跑过（`out/depth6-same-before-the-rollback-split-knob.jsonl`）。

## 要主 agent 跑的命令

- 复跑（工作目录随意，单线程，内存包装 8G，约 14 分钟）：见开头「复跑」一节那一条；看末尾四行 `name=rerun_compare … identical=true`。
- 同一次发布深度 6（可选，每个约 4.5 分钟）：在仓根 `PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 research/prompts/m4-snapshot-r1-opus-model/snapmodel.py --depth 6 --only base-same,a2s-jia-stxg-from-chain-same,a3-same-killprev-new,a8-same-tie-reverse`，与 `out/depth6-same-before-the-rollback-split-knob.jsonl` 比计数。
- I-3.6 误红分布：`PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 research/prompts/m4-snapshot-r1-opus-model/trace.py --split-i36 base-indep 6`，应打出 `i36_red_legal_mid_destroy=8023 i36_red_legal_quiescent=0`。

## 没做什么

- 没跑任何重型测试，没编 Rust，没跑 `enumerate_layer0`（core 没有快照写路径可录；这一腿 layer0 态数 0）。
- 没在工作树的 crates/ 上改任何东西；没建仓副本。草稿目录 `~/.local/share/singlefs/work/m4-snapshot-r1-attack/` 只有辅助脚本与中间输出，没有编译目录。
- 不判正推、辩方那几格；不替主 agent 采纳；模型上的数不算入库装置上的数。
- 没读禁读清单里的任何文件。

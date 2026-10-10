# m3-batch33s-code-r1 攻方（Opus）报告：K2（S1 / S2 / S3 / S5）

日期 2026-10-07。立场：攻方，在冻结副本的拷贝上造反例，只跑 harness 档；GPU 一侧读 wgsl、用 CPU 判法造对照，没上卡。

## 复跑

```
mkdir <空目录> && cd <空目录> && SINGLEFS_REPO=/home/fy5090/code/singlefs bash /home/fy5090/code/singlefs/research/prompts/m3-batch33s-code-r1-opus-model/rerun.sh
# 末行应为：COMPARE same lines=35
```

rerun.sh 文件头写了变量（SINGLEFS_REPO 必给；FROZEN_TREE、THREADS、RUN_KNOWN_BAD 可选，包装脚本不另要变量）。它把冻结副本拷进 `./repo`，给 `crates/singlefs-checker/src/walk.rs` 打上 `walk-lazy-fold.patch`（只加环境变量开关，不设开关时判法与冻结副本相同），把原型测试拷进 `crates/singlefs-harness/tests/`，编译（6G）、跑原型两遍（快照读法 / `ATTACK_LAZY_FOLD=1` 的 GPU 读法，4G）、跑两个 Python 模型、在 GPU 读法下跑 harness 的 `checker_known_bad_images`（8G），汇总成 `summary.txt` 再用 `compare.sh` 与 `expected-summary.txt` 比。交回前在空目录 `~/.local/share/singlefs/work/m3-batch33s-code-r1-opus/rerun2/` 从头跑过一次，一次跑完，末行 `COMPARE same lines=35`。

模型目录 `research/prompts/m3-batch33s-code-r1-opus-model/` 各文件 sha256（交回前现算）：

```
508309b20d0a3eb0785f85ce40cf072ab11f7b0800e1ef1730fcb9389fbf8f28  attack_b33s_fold_order.rs
90e8844787a725fb650828d31c302d2ff0db648bf1e058eace19fbc4814492ce  compare.sh
096da29515f085b5aab85f401a831a8198ce87129b7bdd037fcf212453dd8f9c  expected-summary.txt
c7ba060729cedf7ec885fa60b0499c50485cc51e3c6f0c4415c80f53bc5b552a  rerun.sh
513a0aabea6d52799cde689845cb1660363b7bfa31d61a338f711ba116310a65  s2_guard_model.py
e38125014773e3f1a79ed2c2411182aa462e7858530ab28518510fd83ae75ddc  s5_oracle_model.py
cbb174faa48da0548d1c478b0abf896c50d6c748e3f1bb15ff228dc64be23ee7  walk-lazy-fold.patch
```

以下凡说「副本上量的」，都是在仓副本加上面那份补丁上跑的数，不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| S1 CPU 与 GPU 的键是不是同一组 | **打中（H-S1-fold）** | 键是同一组，但 I-3.5 的折叠表在走读中途被改写之后，CPU 按「每版走完那一刻」取单元、GPU 按「全部走完之后」取单元：较旧一版先走、较新一版的指针第 0 条校验和坏时，CPU 多红 I-3.5、GPU 不红；CPU 自己随哪一版的指针坏、哪一版先走翻面 | 副本上量的 CPU 一侧（三种历史 12 格扫描，见第一节）；GPU 一侧读 wgsl，并在 CPU 上按 GPU 的取法重算（`ATTACK_LAZY_FOLD`） |
| S1 第 1 条那一份被走过、两份都读得出而内容不同 | 没打中（读码，推的） | CPU、GPU 都先读第 0 条、都在第 0 条对得上时停，读回的那一份与键相同，两边同判；见第四节 | 读码 |
| S1 实例表链 | 没打中新形（同 H-S1-fold 一个病根） | 链上第 k 片走 `read_a_unit_to_walk_and_the_placement_it_was_read_from` / `w_read_unit_to_walk`，折叠表的改写与上一格同一处，形状同 H-S1-fold，没另造 | 读码 |
| S1 第 25 批 H-b、第 28 批 H-V、第 30 批 F-W 那几形结局 | 没打中 | 合入验证 203 条坏镜像全绿（主 agent，快照读法）；我在副本上换成 GPU 的取法再跑同一份 `checker_known_bad_images`，仍 203 passed | `KNOWN_BAD_LAZY_FOLD test result: ok. 203 passed` |
| S2 守卫界、差比在空段两端、槽号 2³² 附近 | 没打中（守卫算术） | 守卫的 u32 算术与大整数真值在 20412 个边界取样上逐个相同；改之前那一版在同一批取样上错 1392 个（自证会红） | `s2_guard_model.py` |
| S2 F-α 选 Z3 之后别的读法 | **打中（H-S2-tail，同族已知、射程更宽）** | 空段右端那一槽按 32 KiB 读，GPU 守卫拒（读不出）、CPU 读「全零 16 KiB + 下一槽」：校验和恰是这 32 KiB 的 CRC 时 CPU 判对得上，GPU 判对不上。Z3 只动 F-α 那一处，这一形在走读往下读的数据指针上照样分叉（第 32 批攻方表里 zero16+next 那一列只写了 F-α） | 副本上量的 CPU 一侧；GPU 一侧读 wgsl |
| S3 两份都没自证过时 CPU 与 GPU 判得一样吗 | 没打中（读码，推的） | 两边同一个判据（头读 16 KiB 取类标签、按那一类长度比整单元校验和），GPU 按 32 KiB 比时用落点那一版的 32 KiB 检查字；合法与已知坏形上逐支同判，见第三节 | 读码 |
| S5 带「没有文件」的版本表在崩溃放量与层 0 判得不同 | 没打中 | 两条路判一个状态调的是同一个 `judge_crash_image_against_the_version_table`；GPU 的 `classify_oracle` 与 CPU 的 oracle 在 367416 个穷举格上 0 处不同；复用键按上界截掉的行在 898128 格上不改判定 | `s5_oracle_model.py` |

两处打中都只在坏镜像上（I-2.1 已红），合法崩溃状态上的判定集合我没造出变化。H-S1-fold 的改法 F1（CPU 改成判的时候按全部走完之后的折叠表取单元，即 GPU 今天的取法）只在我的模型上量过、被攻过零轮。

## 一、S1：H-S1-fold，折叠表在走读中途被改写，CPU 与 GPU 取单元的时刻不同

### 1.1 机理（读码，行号是冻结副本的）

- CPU 记引用时，一条指针的两个落点都折到它第 0 条落点上，先到先得：「self.unit_of_each_placement.entry(placement).or_insert(unit);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:1362`）。
- S1 新加的改写：读回的是第 1 条那一份、那个落点此刻折在第 0 条落点上时，改成折到它自己：「&& self.unit_of_each_placement.get(&placement_it_was_read_from)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:1310`），「.insert(placement_it_was_read_from, placement_it_was_read_from);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:1314`）。
- CPU 在每一版走完那一刻就把这一版的落点折成单元存下：「let unit = *self.unit_of_each_placement.get(placement).expect(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:1234`）。之后别的版本再改写折叠表，已经存下的那几版不跟着变。
- GPU 同一处改写在 `w_read_unit_to_walk`：「sws(row + 3u, p.device1);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1336`）。但 GPU 每版走完只存落点表的下标：「sws(C_APPEARANCE_VERSION_PLACEMENTS + appearance_version_placement_count, placement);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1004`），判的时候才按落点表那一行此刻的第 3–5 字认单元：「return sw(row + 3u) == sw(other_row + 3u) && sw(row + 4u) == sw(other_row + 4u) && sw(row + 5u) == sw(other_row + 5u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3441`），「if (appearance_same_unit(sw(C_APPEARANCE_VERSION_PLACEMENTS + first + k), placement)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3504`）。所以 GPU 用的是全部版本走完之后的折叠表，早走完的那几版也按改写之后的认。
- 走读次序：先走最新根，「walk.walk_root(&roots[newest_index].2.record_bytes, true);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:11199`），别的候选根按根环的区域、槽次序走，「walk.walk_root(&root.record_bytes, false);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:11321`），不按 txg。

于是：较旧的 R⁻ 先走、经指针 (X 对得上, Y) 读回 X，存下的单元集合里 Y 折在 X 上；较新的 R 后走、它自己的那条指针 (X 校验和坏, Y) 读回 Y，把 Y 改折到 Y。CPU：R 引用单元 Y、R⁻ 存下的集合里没有 Y → Y 算「R 引用而 R⁻ 不引用」，诞生代号不大于 R⁻.txg 又不在整棵继承的用户可见树里就红 I-3.5。GPU：R⁻ 那几个落点按最终的折叠表认，Y 也是单元 Y → 不算新出现 → 不判。I-3.5 那一行判的对象写的是落点（`.claude/kb/invariants.md:146` 出现规则那一句），R⁻ 的指针本来就引用落点 Y，按条款字面这一格不该判，GPU 的结局与字面相同、CPU 多红一条。

### 1.2 反例（副本上量的，CPU；GPU 一侧见 1.4）

历史（全在内存稀疏盘上）：新池新建文件（txg 3）→ 顺序写成三个数据单元（W，txg 4）→ unlink 意图（I，txg 5，extent 树照抄 W 的，树表另写一份）→ 第一批（txg 6，最新，extent 树重写）。故障：只改 I 那张树表里 extent 树根指针第 0 条位置条目的 4 字节校验和（异或 0x5a），重封那张树表、沿引用链补到 I 的根槽与记录新根段；extent 树根那个单元本身两份都完好。B 臂改的是 W 那张树表的同一处。

```
ATTACK_HISTORY written_txg=4 intent_txg=5 batch_txg=6
ATTACK_ARM arm=clean lazy_fold=false red=[] i35=holds
ATTACK_ARM arm=A-newer-version-pointer-garbled lazy_fold=false red=["I-2.1", "I-4.8", "I-7.4", "I-3.5"] i35=violated: 相邻两版 (实例 {1}、txg 4) → (实例 1、txg 5)（NoHole）：后一版引用、前一版不引用的单元盘 1 槽 50254（用户可见树 Some(UserVisibleTreeOfTheWalk { kind: 1, tree: 65536 })）诞生代号 4 不大于前一版的 txg，也不在一棵整棵继承下来的用户可见树里（根指针出生 txg 小于前一版的 txg、子树里读得出的诞生代号都不大于它）
ATTACK_ARM arm=A-newer-version-pointer-garbled lazy_fold=true red=["I-2.1", "I-4.8", "I-7.4"] i35=holds
ATTACK_ARM arm=B-older-version-pointer-garbled lazy_fold=false red=["I-2.1", "I-4.8", "I-7.4"] i35=holds
ATTACK_ARM arm=B-older-version-pointer-garbled lazy_fold=true red=["I-2.1", "I-4.8", "I-7.4"] i35=holds
ATTACK_WALK_ORDER history=write3-intent-batch txgs=6,0,3,1,4,2,5,
```

（`lazy_fold=false` 是冻结副本的判法；`lazy_fold=true` 是同一份镜像在 CPU 上按 GPU 的取法——判的时候用全部走完之后的折叠表——重算的 I-3.5。）A、B 两臂是同一件事的两面：两版各从一份副本读同一个单元，只差坏的是哪一版的指针。按「落点」读（条款字面、GPU）两臂都不该判；按「每版读回那一份是一个单元」读两臂都该判；CPU 一红一不红，哪一种读法都对不上。

### 1.3 用户决定的那几步放开扫（副本上量的，CPU）

三种历史（写三个单元之后：意图 + 一批；意图 + 删到底；建三次空 inode），每一对相邻、树表各写一份的版本，每一棵两张树表里根指针逐字相同的树，改坏较新那一版的第 0 条校验和，同一个进程里两种取法各判一次：

```
ATTACK_SCAN history=write3-intent-batch older_txg=4 newer_txg=5 kind=1 snapshot_i35=violated lazy_i35=holds
ATTACK_SCAN history=write3-intent-batch older_txg=5 newer_txg=6 kind=10 snapshot_i35=holds lazy_i35=holds
ATTACK_SCAN history=write3-intent-batch older_txg=5 newer_txg=6 kind=9 snapshot_i35=holds lazy_i35=holds
ATTACK_SCAN history=write3-new-inodes-x3 older_txg=4 newer_txg=5 kind=1 snapshot_i35=violated lazy_i35=holds
ATTACK_SCAN history=write3-new-inodes-x3 older_txg=5 newer_txg=6 kind=1 snapshot_i35=holds lazy_i35=holds
ATTACK_SCAN history=write3-new-inodes-x3 older_txg=6 newer_txg=7 kind=1 snapshot_i35=holds lazy_i35=holds
ATTACK_SCAN history=write3-unlink-to-the-end older_txg=4 newer_txg=5 kind=1 snapshot_i35=violated lazy_i35=holds
ATTACK_SCAN history=write3-unlink-to-the-end older_txg=5 newer_txg=6 kind=10 snapshot_i35=holds lazy_i35=holds
ATTACK_SCAN history=write3-unlink-to-the-end older_txg=5 newer_txg=6 kind=9 snapshot_i35=holds lazy_i35=holds
ATTACK_SCAN history=write3-unlink-to-the-end older_txg=6 newer_txg=7 kind=10 snapshot_i35=violated lazy_i35=holds
ATTACK_SCAN history=write3-unlink-to-the-end older_txg=6 newer_txg=7 kind=9 snapshot_i35=holds lazy_i35=holds
ATTACK_SCAN history=write3-unlink-to-the-end older_txg=7 newer_txg=8 kind=10 snapshot_i35=holds lazy_i35=holds
ATTACK_WALK_ORDER history=write3-new-inodes-x3 txgs=7,0,3,6,1,4,2,5,
ATTACK_WALK_ORDER history=write3-unlink-to-the-end txgs=8,0,3,6,1,4,7,2,5,
```

12 格里 CPU 红 4 格（extent 树 3 格、墓碑树 1 格），GPU 取法 0 格。红的 4 格较旧那一版都先于较新那一版走；较新那一版先走（它是最新根，或根环次序在前）的 7 格两种取法都不红；剩下 `write3-unlink-to-the-end 6→7 kind=9`（`deleted_inodes`，用户可见树）较旧的先走却不红，我推是它整棵继承下来、走 (b) 那一支（没单独量）。红不红随根环次序变：同一个故障放在根环次序倒过来的那一对上就不红。

### 1.4 GPU 一侧（读码，推的）

GPU 判器没跑。按 1.1 引的四处：GPU 存的是落点下标、判的时候按最终折叠表认，所以 A 臂那几格 GPU 的 I-3.5 结局应与 `lazy_fold=true` 那一行相同（成立）。`ATTACK_LAZY_FOLD` 的重算是我在 `walk.rs` 副本上加的（`walk-lazy-fold.patch`：每版另存落点，判之前按 `unit_of_each_placement` 与 `user_visible_tree_of_each_placement` 的终值重折），它是 GPU 取法的 CPU 抄写，不是 GPU 本身；两者在「同一单元留哪个落点的标签」上我按 `appearance_spread_the_units_of_a_version`（留 (盘, 槽) 最大那个）与 BTreeMap 收集（后收的盖先收的）对过，相同。

### 1.5 打中之后的四句

1. 分不分辨臂：分。被比的两种取法（CPU 的「每版走完时取」与 GPU 的「全部走完后取」）在 A 臂与扫描里那 4 格上判得不同，别的 8 格相同；同一个故障换到 B 臂（较旧那一版的指针坏），两种取法都不红。
2. 被判的系统看不看得到判别它的东西：看得到。判 I-3.5 时 CPU 手里有最终的折叠表与每版的落点集合（副本上加一个字段就重算出了 GPU 的结局），GPU 本来就是这么做的。
3. 满足判据字面的哪一个分句：正文第三节 K2 S1 第一句「一份合法或坏镜像，CPU 新键与 GPU 新键判得不同」里的「坏镜像」那一半（I-2.1 已红）。不满足 I-3.5 那一行的推翻条件（那条要合法镜像上红）。按条款字面（判的是落点）CPU 那一条是坏镜像上的多红；误红 / 漏判怎么归按 m3-batch22-code-r1 正文第五节的口径由主 agent 定，我没读那份正文。
4. 跑前条款给的每个改法在打中的格上还中不中：本轮跑前条款只写「改法进下一批、算零轮」，没给具体改法。我提的几个改法各修哪一格见 1.6。

### 1.6 改法（只在我的模型上量过、被攻过零轮）

| 改法 | A 臂 | B 臂 | 扫描 12 格 | 已有 203 条坏镜像 | 标 |
|---|---|---|---|---|---|
| F1：CPU 判 I-3.5 时按全部走完之后的折叠表取每版的单元（GPU 今天的取法，每版只存落点） | 不红（与 GPU 同） | 不红 | 0 格红 | 203 passed | 量过：`ATTACK_ARM … lazy_fold=true`、`ATTACK_SCAN … lazy_i35=holds`、`KNOWN_BAD_LAZY_FOLD test result: ok. 203 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| F2：撤掉折叠表的改写（S1 加的那几行，CPU、GPU 都撤） | 不红 | 不红 | 推不红 | 推会红：「/// 只改诞生代号那张表、不改 I-3.5 折叠表时这一形改红 I-3.5（攻方第 1.3 节）。记录宽 16 → 15 那一形见」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:14174`） | 推的 |
| F3：GPU 也改成每版走完时取 | 红（与 CPU 同） | 不红 | 与 CPU 同 4 格红 | 没量 | 推的；CPU、GPU 一致了，但 A、B 两臂、根环次序翻面那件事照旧 |

F1 只换 I-3.5 取单元的时刻，不碰「走过」、诞生代号、父单元那几把键，第 32 批判决第三节第 1 条要的「各把键按读回那一份认」照旧。F1 自己的盲区（推的）：较旧那一版的 Y 那一份对不上而 Y 槽被别的单元复用、较新那一版又从 Y 读回那个新单元时，F1 把 Y 认成两版共有、不判它的诞生代号；这种镜像 I-2.1 已红，按条款字面（落点 Y 两版都引用）本来也不判。

## 二、S2：守卫算术没打中；H-S2-tail，空段右端那一槽按 32 KiB 读

### 2.1 守卫算术（Python 模型，`s2_guard_model.py`，纯算术）

把 `zero_unit_version_over_a_slot_nothing_wrote` 今天那一版与改之前那一版按 u32 逐句抄，与「起点按槽对齐、槽号装得进 32 位、读的几槽都在 [第 9 字, 第 10 字) 里」的大整数真值比。取样：空段 9 种（空、只一槽、两槽、贴 2³² − 1、跨 2¹⁸ 等）× 第 11 字 3 种 × 槽号低位 9 种（0–3、2¹⁸ ± 1、2³² − 3…2³² − 1）× 偏移高 32 位 7 种（0、1、2¹⁴ − 1、2¹⁴、2¹⁴ + 1、2¹⁸、2³² − 1）× 长度 6 种 × 对齐 / 不对齐：

```
S2_ROW name=guard_now_vs_truth cases=20412 wrong=0 first=None
S2_ROW name=selfproof_guard_before_vs_truth cases=20412 wrong=1392 first=(0, 0, 0, 4294950912, 16383, 1, 1, None)
```

今天的守卫 0 处错；改之前那一版错 1392 处，第一处就是第 32 批判决说的回绕（槽 2³² − 1 加 1 回到 0、落进空段），模型分得出两版。长度 0 的读今天没有调用点，没算进 wrong。

### 2.2 H-S2-tail（副本上量的 CPU 一侧；GPU 一侧读码）

GPU 只在起点恰是落点起点、或读的几槽都在空段里时读得出：「version = zero_unit_version_over_a_slot_nothing_wrote(location, offset, length);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:481`），空段右端那一槽 g（g + 1 有落点）按 32 KiB 读时差比不过：「|| first_slot > first_slot_past_the_gap || first_slot_past_the_gap - first_slot < slots_read) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:511`），交 NONE。CPU 的读者照读 32 KiB：「let bytes = reader.read(location.device, location.slot * SLOT_BYTES, unit_bytes);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/image.rs:1526`），读出「全零 16 KiB + g + 1 那一槽」。

反例：1.2 那段历史，把最新那一版 extent 树里第一个文件第 0 个数据指针的第 0 条位置条目改指盘 0 的空段右端那一槽（g = 50285，g + 1 写过且不全零），校验和改成那 32 KiB 的 CRC；对照臂校验和改成 32 KiB 全零的 CRC：

```
ATTACK_S2 gap_slot=50285 next_slot_all_zero=false
ATTACK_ARM arm=S2-crc-of-zeros-then-next-slot lazy_fold=false red=["I-1.6", "I-2.4", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"] i35=holds
ATTACK_ARM arm=S2-crc-of-32k-zeros lazy_fold=false red=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"] i35=holds
```

CPU 在第一臂把第 0 条判成对得上（I-2.1 不红）、读进那 32 KiB、头不对红 I-1.6、I-2.4、I-7.2；GPU 走 `w_walk_extent_data_pointer` 读这条指针：「let s = w_read_unit_to_walk(p, DATA_UNIT_BYTES);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1885`），第 0 条读不出、红 I-2.1，读回第 1 条那份真单元，头照常——推的结局与对照臂 CPU 那一行同一组。两臂 GPU 判得一样、CPU 判得不同，分叉在第一臂。

四句：① 分辨 Z 那几个改法：第 32 批攻方表里 Z1、Z3 都留着「zero16+next」这一形（`research/prompts/m3-batch32-code-r1-opus-output.md:173`），Z2（CRC 拼接）推的能修；我补的是射程——Z3 只改了 F-α 那一处，「let s = w_read_unit_to_walk(p, DATA_UNIT_BYTES);」这类往下读的路径、恢复内核的 `read_unit_via_locations` 都经同一个 `read_at`，同样分叉（数据指针那一条量过 CPU 一侧，恢复内核那一条推的）。② GPU 看得到：`read_at` 手里有空段终点与下一个落点那一版的 16 KiB 检查字。③ 满足 K2 S2「拒了该放的」（守卫在空段右端拒了 CPU 读得出的 32 KiB）。④ 跑前没给改法；Z3 在这一格上不起作用。

## 三、S3：两份都没自证过（没打中，读码，推的）

- CPU 类型段 0 那一句：「&& child_class.is_none_or(|class| class == MappingKeyUnitClass::IndexNode);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:2468`）；GPU：「let child_is_a_code_two_node_or_unreadable = class_of_the_child_of_a_type_zero_entry == NONE || class_of_the_child_of_a_type_zero_entry == 2u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3244`）。两份都没自证过时两边都是「不判」。
- 「自证过」的判据：CPU 读 16 KiB 取类标签、按那一类长度重读比校验和，「.filter(|content| crc32_castagnoli_table(content) == location.checksum)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:4581`）；GPU 读 16 KiB 切片、码 1 / 码 3 按落点那一版的 32 KiB 检查字比，「return slice_crc_matches(s, DATA_UNIT_BYTES, checksum);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3081`），检查字只在落点装得下 32 KiB 时有，「let has_32k = bytes.len() >= data_unit_bytes;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:836`）。
- 我逐支对过的形：没写过的槽（两边读出全零、类标签 0、都不算自证过）；空段右端那一槽（16 KiB 读两边都是全零，类标签 0，不进 32 KiB 那一支，H-S2-tail 碰不到这一句）；写过的槽、头里是码 3（落点 32 KiB，GPU 检查字罩住下一槽在这个状态里的字节，与 CPU 重读同）；盘尾只装得下 16 KiB 的槽、头里是码 1 / 3（CPU 读 32 KiB 越过盘尾交 None，GPU 检查字标志位没有，两边都不算）；盘不在池里（两边都读不出）。没找到两边判得不同的一形。
- 合成之前三处与合成之后逐处相同（这属 K1，只记我核到的）：旧的 `some_copy_of_the_child_verifies_as_the_class_the_entry_type_names` 直接按类型段那一类的长度读，新的先读 16 KiB 再按头里那一类读；头里是类型段那一类时长度相同，不是时两种写法都交 false。

## 四、S5（没打中）

- 崩溃放量判一块与层 0 判一个状态调的是同一个函数：层 0 「judge_crash_image_against_the_version_table(&image, judged_root_index, versions);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1591`），放量经 `judge_layer0_state_range_against_the_version_table` 走「judge_crash_image_against_the_version_table(&image, judged_root_index, versions);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1476`），版本表取「flow.version_table(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:1214`）。两条路在 CPU 上能判得不同只剩两处：复用（键相同的块不重判）与 GPU 判器。
- GPU 的 oracle 与 CPU 的 oracle（`s5_oracle_model.py`，逐句抄 `classified_oracle_violation_for_the_version_table` 与 `classify_oracle`；根域 txg 1–3 × 实例 0–1 共 6 个根，每个根不在表 / 有文件 / 没有文件，期望表两种排列，落到的根含表外更新与更旧的两个，盘上最新持久根 7 种，结局 4 种）：

```
S5_ROW name=cpu_vs_gpu_oracle cases=367416 disagreements=0 first=None
S5_ROW name=reuse_key_cut_keeps_the_verdict cases=898128 changes=0 first=None
S5_ROW name=selfproof_old_any_older_rule_is_caught cpu=None gpu_before=NoFileWhereAnOlderRootHasOne caught=True
```

  第二行核复用键：键里只留 (txg, 实例) 不超过上界的行，「highest_reachable_root.is_none_or(|highest_root| {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:671`）；落到的根不超过上界时，截掉上界之外的行不改 oracle 的判定，所以只差上界之外那几行的两条流共用块键不会换判定。这一句依赖「落到的根不超过上界」，那是复用键文档里已有的前提，我没另攻。第三行是自证：GPU 照改之前「任一更旧版有文件」判时，模型分得出。
- 期望表里同一个根出现两次（两边都取第一条）、带文件与没有文件的两条同根（`Layer0VersionTable::new` 停下）这两种我只读了码，没进模型：两边一致（推的）。

## 五、没打中的形状与取样范围

| 形 | 试法 | 范围 | 结局 |
|---|---|---|---|
| S1 第 1 条那一份被别的版本走过、两份都读得出而内容不同 | 读码：CPU `read_referenced_unit_with_the_location_it_was_read_from` 与 GPU `w_read_referenced_unit` 都先读第 0 条、第 0 条对得上就用它，读回落点同、键同 | 读码 | 两边同判（推的）；这形里第 1 条那一份内容不同本身没有哪一条在判，与本批无关 |
| S1 实例表链 | 读码：第 k 片走同一个 `read_a_unit_to_walk_and_the_placement_it_was_read_from` / `w_read_unit_to_walk` | 读码 | 与 H-S1-fold 同一个病根，没另造 |
| S1 GPU 的改写只在上下文 0 | 读码：CPU 别的几遍走读（`harvesting_walk` 等）各有自己的 `Walk`，不判 I-3.5 | 读码 | 不分叉 |
| S1 已有坏镜像结局 | 副本上 GPU 取法跑 `checker_known_bad_images` | 203 条 | 203 passed |
| S2 守卫算术 | Python 模型 | 20412 格 | 0 处错 |
| S3 两份都没自证过 | 读码，五种落点形 | 读码 | 两边同判（推的） |
| S5 oracle 与复用键 | Python 模型 | 367416 + 898128 格 | 0 处不同 |

旁注（推的，没跑，不在本批改动里）：inode 树根的 I-9.1 两边都只读第 0 条位置条目那一份，「pointer.locations[0].slot * SLOT_BYTES,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33s-code-r1/tree/crates/singlefs-checker/src/walk.rs:2169`），不随读回那一份。CPU、GPU 同判，不是分叉；第 0 条那一份坏、第 1 条完好时它在坏字节上判类标签（I-2.1 已红）。是不是「各把键一律按读回那一份认」射程里的一把，交主 agent 定。

## 六、这条腿自己的限度

- GPU 没跑。H-S1-fold 与 H-S2-tail 的 GPU 一侧都是读 wgsl 推的；`ATTACK_LAZY_FOLD` 是我在 CPU 上抄的 GPU 取法，抄错了 1.4 那一句就不成立。推翻条件：上卡跑 A 臂那份镜像（或同形的对拍流），GPU 的 I-3.5 也红，或 CPU 在 GPU 取法下仍红。
- H-S1-fold 的扫描只改树表里的根指针（三种历史、12 格），没改更深一层的指针、实例表链与由记录施加出来的那几版；红的 4 格都是「较旧那一版先走」，我把规律写成「随根环次序翻面」是从这 12 格读出来的，没有反例不等于没有别的条件。
- 两个 Python 模型是手抄的，抄错就两边一起错；各带一条自证（S2 改之前那一版、S5 改之前的「任一更旧版有文件」），只证模型分得出那一种差别。
- 没跑任何崩溃枚举：层 0 / 崩溃放量的状态 0 个（`closed_form_state_count` 没调，本腿原型全量合计 0 个状态）。S5「崩溃放量与层 0 判得不同」只靠「两条路调同一个函数」加 oracle 模型。
- 误红、漏判的口径（m3-batch22-code-r1 正文第五节）我没读，1.5 第 3 句的归类交主 agent。
- `rerun1/` 那一次（第一次在空目录跑）撞了前台 230 秒的 timeout、在同一目录续跑了两次才比上；交回用的是 `rerun2/` 那一次，空目录一次跑完。

## 七、没做什么

- 没跑 GPU、checker 档测试、层 0 枚举、变异；没跑重型测试。
- 没判 K1、K3（正推腿的格）；S4、S6–S8 不在我的攻击面。
- 没攻根环线 10x 的改动。
- 改法 F1–F3 没写进 `crates/`，只在仓副本上加了环境变量开关（F1 的 CPU 抄写）。

## 八、要主 agent 跑的命令

- 复跑本腿模型（harness 档，几分钟）：工作目录任一空目录；`SINGLEFS_REPO=/home/fy5090/code/singlefs bash /home/fy5090/code/singlefs/research/prompts/m3-batch33s-code-r1-opus-model/rerun.sh`；内存上限在脚本里（编译 6G、跑 4G、`checker_known_bad_images` 8G），线程默认 7（`THREADS=` 可改）；看末行 `COMPARE same lines=35`，不同时 `compare.sh` 打出 diff。
- GPU 一侧的坐实要上卡（checker 档，归主 agent）：把 1.2 的 A 臂（历史与故障都在 `attack_b33s_fold_order.rs` 的 `history` 与 `garble_the_first_location_checksum_of_the_extent_root_pointer_of`）做成一条对拍流整份比，预期按本报告是 CPU 多红 I-3.5、GPU 不红；H-S2-tail 同理（`redirect_the_first_location_of_the_first_data_pointer`，预期 CPU 不红 I-2.1、红 I-1.6 / I-2.4 / I-7.2，GPU 反过来）。我没写这两条流。

## 九、草稿与副本

草稿目录 `~/.local/share/singlefs/work/m3-batch33s-code-r1-opus/`：各次跑的日志（`run-s1-*.log`、`run-s2.log`、`run-scan.log`、`kbi-lazy.log`、`rerun1/`、`rerun2/` 里的 `summary.txt` 与日志）留着；仓副本 `repo/` 与编译目录 `target/`、`rerun1/{repo,target}`、`rerun2/{repo,target}` 交回前删掉。模型与期望汇总已在 `research/prompts/m3-batch33s-code-r1-opus-model/`，不另入 `research/results/`（三方腿的产物按派发提示放 prompts 下）。

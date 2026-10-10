# m3-batch30-code-r1 云端攻方（Opus）腿报告

立场：攻方，造反例；用户决定的那几步放开扫。攻击面 K2，重点 M1（F-W），兼 M2（F-IT）与 CPU/GPU 判得不一样；另做 K2 末条第二次抽样。只跑 harness 档，GPU 不跑（读 wgsl、用 CPU 算期望）。

## 复跑

模型目录 `research/prompts/m3-batch30-code-r1-opus-model/`，空目录从头跑：

```
SINGLEFS_REPO=<冻结副本仓根> SINGLEFS_TOOLS=<脚本仓根> bash rerun.sh <空目录> all
bash rerun.sh <summary-a> compare <summary-b>     # 比之前各自排序，不依赖 SINGLEFS_REPO
```

`rerun.sh` 建两份副本：b30（冻结副本原样，含 F-W）与 b29（把 `walk-fw.patch` 的 F-W walk.rs 改动 `patch -R` 退回），两份都追加 `probe-append.rs` 的探针测试，各编译（6G）、各跑（4G，capped.sh 7，同一时刻只一条 cargo），把 OPUSPROBE 各行与新坏镜像用例的 test result 排序写进 `summary.txt`。`compare` 子命令只排序 + diff。

模型文件 sha256：
- `rerun.sh` 04f5fa04210998f128410c45a37cf7d315b5355811b267a7f504acd6a328bc3d
- `probe-append.rs` 7e92da9caebfb0e3f4a78f89d7ea29ec977da729f8a24e09de44491c0af5cb1b
- `walk-fw.patch` da7ccedfb492be7be9ba10b9e06351c2f175a4c22dd9e89617e4c699cf135304
- `expected-summary.txt` 2320d16124395c330733749583af5ead9dda77ab254dbb09c2bed091a255c3bb

冻结副本仓根 `~/.local/share/singlefs/work/m3-batch30-code-r1/tree/`；引代码写它的全路径:行号。

## 各格判定一览

| 攻击面 | 本腿一次抽样结论 | 依据 |
|---|---|---|
| M1 F-W CPU（inode / 墓碑 / 谱系，层级 1，盘 0 / 盘 1） | **没打中**：b30 在 F-W 覆盖的那一形上比 b29 基线严格更对——补上 I-9.15 等子树违例（b28 漏判的那一格）、同时不再误红 I-3.1 / I-3.11 / I-5.1；没引入新的漏判或误红 | b29↔b30 探针差（第二段）、新坏镜像 8 个用例全绿 |
| M1 F-W GPU（走读内核三处） | **没打中（读码）**：GPU 三处与 CPU 同一支判法（认出别类且有一份按类型段那一类自证过 ⇒ I-9.2 红、按类型段那一支往下走那一份）；inode 那一处 GPU 只走层级 1 根、层级 ≥ 2 报判不了，不是分叉 | 冻结副本 wgsl:2679 / 3156 / 3209 对 walk.rs:2724 / 2346 |
| M2 F-IT（GPU 实例表链每一片判 I-1.2） | **没打中（读码）**：GPU 新那一行与 CPU 逐字段同偏移（诞生代号 73、写序实例 93、出生序号 103 对指针），同在 packed.ok 与 visited 检查之后、每一片都判、不判 I-4.2 | wgsl:1641 对 walk.rs:3181、1050 |
| CPU 与 GPU 判得不一样 | **没打中（读码）**：F-W、F-IT 两处 GPU 都忠实镜像 CPU；实例表链共享片（visited 命中）两边都在判 I-1.2 之前 break，不重复判、不判错片 | 同上 |
| 第二次抽样：b29 的 L1 合法状态多红 / L1 一份读不出 / L2 链长与候选集里外 | **交主 agent 跑**：要层 0 崩溃枚举（前台 4 分钟跑不完，且属 checker 档），命令见末段 | — |

> 推翻本腿「没打中」的观测：在 F-W 走下去那一份的子树里放一个违例，而 b30 报不适用或成立（漏判）；或一个从入口走得到的已发布崩溃状态上 F-W / F-IT 那几支把某条本成立的不变量报红（误红）。本腿只抽了一次，照 `.claude/rules/three-way-inference.md` 要再抽一次才记「没打中」。

## 一、M1 F-W：往下走之后那几格（没打中）

F-W 的代码：`walk.rs` 墓碑 / 谱系树 `walk_the_entries_of_a_container_tree_node` 第 2346 行、inode 树 `walk_inode_internal_node` 第 2724 行，认出孩子自证过的别类后，若 `some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`（冻结副本 `crates/singlefs-checker/src/walk.rs:4447`）为真，就判条目那一条（I-9.23 / I-9.17 / I-9.2）红、`continue` 不再往下（容器树）或把 `child_class_the_walk_does_not_go_below` 置 None 落到正常走读（inode），照类型段那一支走那一份。读者用哪一份由 `read_referenced_unit`（冻结副本 `crates/singlefs-checker/src/image.rs:1501`）按类型段那一类的长度逐份读、第一份对得上的那一份定。

### 攻法：在 F-W 走下去那一份的子树里放违例，比 b30 与 b29 基线

探针 `opus_probe_fw_full_verdicts`（`probe-append.rs`）：新池新建文件那一版，inode 叶容器第一条记录 blocks 加 1（录成违例，重封补到根，只它红 I-9.15），再做 H-V（叶容器盘序大 / 小那一份换成 inode 树根的码 2 字节、按改过的位置条目自证得过）。两种盘序各判一次，打印 `check_pool_image` 的整个违例集合。`leave_the_entry_as_it_is` 不改条目类型段（类型段仍说码 3 叶容器）。

b30（含 F-W）与 b29（`patch -R` 退回 F-W 的 walk.rs 改动）同一组镜像上的违例集合（本腿直接跑，与 rerun 的 OPUSPROBE 行一致）：

```
# 只放违例、不做 H-V（两份代码相同）
only-violation: {"I-9.15"}

# b29（F-W 之前）
FW   code2-at-location 0: {"I-2.1","I-3.1","I-3.11","I-4.8","I-5.1","I-7.4","I-9.2"}
FW   code2-at-location 1: {"I-2.1","I-3.1","I-3.11","I-4.8","I-5.1","I-7.4","I-9.2"}
HV   code2-at-location 0: {"I-2.1","I-3.1","I-3.11","I-4.8","I-5.1","I-7.4","I-9.2"}
HV   code2-at-location 1: {"I-2.1","I-3.1","I-3.11","I-4.8","I-5.1","I-7.4","I-9.2"}

# b30（F-W）
FW   code2-at-location 0: {"I-2.1","I-4.8","I-7.4","I-9.15","I-9.2"}
FW   code2-at-location 1: {"I-2.1","I-4.8","I-7.4","I-9.15","I-9.2"}
HV   code2-at-location 0: {"I-2.1","I-4.8","I-7.4","I-9.2"}
HV   code2-at-location 1: {"I-2.1","I-4.8","I-7.4","I-9.2"}
```

逐格比 b29 → b30（盘 0 / 盘 1 相同，符合 C739 后一半「不随盘序翻面」）：

| 不变量 | b29 | b30 | 判这一次改得对不对 |
|---|---|---|---|
| I-9.15（子树违例） | 不红（**漏判**，b28 的病：本类那一份整片不走） | 红 | **改对**：F-W 照类型段走那一份码 3 叶容器、判出里面的 blocks 违例。这正是改法要补的那一格 |
| I-3.1 / I-3.11 / I-5.1（引用登记 / 已分配 / 不重叠） | 红（b29 按别类码 2 一槽登记，与账里那片叶 2 槽对不上 ⇒ 连带红——其实是误红：盘上那片叶占的还是 2 槽） | 不红 | **改对**：F-W「槽数按类型段那一类登记」（2 槽）与账相符。新坏镜像用例断言的就是这三条不红 |
| I-2.1 / I-4.8 / I-7.4 | 红 | 红 | 两份都红：盘序大 / 小那一份被换成 16 KiB 码 2 节点、按类型段 32 KiB 长度读回校验和对不上 ⇒ 这一份 I-2.1 真红（镜像本来就坏）。这是实现员报告「要你定的」第 1 条「别类那一份按类型段长度读对不上时 I-2.1 一族照红」，主 agent 已按字面偏保守认、随 C739 写回 |
| I-9.2（条目） | 红 | 红 | 两份都红：条目那一条照判红 |

结论：在 F-W 覆盖的那一形（层级 1、三棵树、两种盘序）上，b30 相对 b29 **只把事情改对**——补上漏判的 I-9.15、修掉误红的 I-3.1 / I-3.11 / I-5.1，没有哪条本来判对的现在判错。引用登记、I-2.1 一族、子树里的不变量三样都查了，没打中。

## 二、M1 F-W GPU 与 M2 F-IT：读码对 CPU（没打中）

GPU 不跑，读 wgsl、用 CPU 算期望。

### F-W GPU（走读内核三处）

- inode 树 `w_walk_inode_root`（冻结副本 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2679`）：`class_of_the_inode_child != NONE && class_the_inode_entry_type_names != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, class_the_inode_entry_type_names)` ⇒ 判 I-9.2 红、`class_of_the_inode_child = NONE` 落到正常走读那一份。对 CPU `walk.rs:2724` 那一支：同一个判别式（认出别类 + 有一份按类型段那一类自证过）、同一个结果（I-9.2 红、往下走类型段那一份）。
- 墓碑 / 谱系（容器树）`w_walk_container_tree_root`：类型段 0 那一支 `wgsl:3156`（`w_some_copy...(child, 2u)`）、类型段 = record_type 那一支 `wgsl:3209`（`w_some_copy...(child, 3u)`），都是认出别类 + 自证过类型段那一类 ⇒ 判 identity_invariant 红、`class = NONE` 落到正常走读。对 CPU `walk.rs:2346`。
- `w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`（`wgsl:3061`）读两份副本、头里类标签 == 类型段那一类且按那一类的长度自证得过 ⇒ true。对 CPU `some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`（`walk.rs:4447`），逐条同义（类标签偏移 6、码 2 读 16 KiB / 码 3 读 32 KiB、整单元校验和对位置条目）。
- GPU inode 走读内核只走层级 1 根（遇类型段 0 的内部节点报 `UNDECIDABLE_TOO_DEEP`、判不了），层级 ≥ 2 不是「与 CPU 分叉」而是「GPU 判不了、回落 CPU」。F-W 不改这个上界。容器树 GPU F-W 这三处本轮没有对拍流（判决第三节第 3 条进第 31 批），只读码；读码上与 CPU 同一支，没找到分叉。

### F-IT GPU（实例表链每一片判 I-1.2）

GPU 新那一行（冻结副本 `wgsl:1641`）：`w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance && eq64(vec2<u32>(rd32(s.base, 103u), 0u), pointer.tail))`。对 CPU：链里每一片在 `walk.rs:3181` 调 `judge_birth_identity_against_the_pointer`（`walk.rs:1050`，码 3 读诞生代号 73、写序实例 93、出生序号 103 对指针记的）。逐字段同偏移、同比较。

两边都在这几步之后才判、次序一致：读回第一份对得上的副本（CPU `read_referenced_unit_with_the_location_it_was_read_from`、GPU `w_read_referenced_unit`）→ visited 命中就 break（不判 I-1.2，避免重复判 / 判错已走片）→ `judge_packed_container`（I-1.7）过 → I-1.12（CPU `walk.rs` 的 `judge_pointer_birth_tree_against_the_unit_header`、GPU `wgsl:1636`）→ I-1.2。类型 4 实例表单元只判 I-1.2、不判 I-4.2，两边都是。没找到分叉。

### M2「链途经已走片 / 实例表单元共享」那一形

CPU 在 `walk.rs` 第 3167 行 `if !visited_units.insert(placement_it_was_read_from)` 命中就 `merge` + `break`，在判 I-1.2（3181）之前；GPU 在 `wgsl` 第 1620 行 `if (!w_visit(device_read_from, slot_read_from))` 命中就 `w_merge_below` + `break`，在判 I-1.2（1641）之前。两边都用「读回、自证过的那一份的落点」认走过（CPU `location_it_was_read_from`、GPU `read_from_the_first_location` 重读盘 0）。所以共享片 / 途经已走片时两边都不重复判、不判错片，一致。没打中。

## 三、第二次抽样（b29 的三格）与 F-W 合法状态误红 —— 要主 agent 跑

b29 判决第二节记「没打中（一次抽样）」的三格（L1 合法状态多红、L1 一份读不出、L2 链长与候选集里外）与 F-W 的「合法状态误红」，都要层 0 崩溃枚举才量得到。层 0 枚举属 checker 档、前台四分钟跑不完，本腿不跑，交主 agent。

代码层先说一半：F-W 那三支只在 `child_unit_class_other_than_the_entry_type_names` 认出一份自证成**别类**的副本、且另一份按类型段那一类自证得过时才触发（`walk.rs:2337` / `2709`、`wgsl` 三处）。合法崩溃状态上两份副本同内容、都是类型段那一类，这个判别式不成立——b28 判决第一节的层 0 两段探针（72 万多个合法崩溃状态）正是 `other_class_returned=0`、`copy_other_class=0`。所以 F-W 不会在合法状态上触发，谈不上误红；这是「合法状态误红」那一格的代码论证，要坐实请主 agent 在层 0 加一个「F-W 触发 / 合法状态上红了没有」的探针跑一遍。

要主 agent 跑的命令（checker 档，命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `user-request`，经内存包装；工作目录仓根）：

```
# 覆盖删除批那几条流的层 0 全量（新池新建文件、删除、tree-split 等）：每个崩溃状态恢复后跑 checker，
# 比 oracle、数红的状态。跑法照 commit-stages / staged-gate 的 layer0 档：
SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-layer0-replay.sh --full <worktree>
```

- 看哪几行判结果：各 `RESULT crash … layer0 states=N oracle_violations=0 failed_states=0 checker_red_states=[]` 行；`checker_red_states` 非空即合法状态误红（要主 agent 对这几格现查）。
- 要分辨 b29 三格与 F-W 合法状态，得在枚举装置里加探针（F-W 触发计数、L1 第三支走到计数、L2 链长档），那是 checker 档改动，由主 agent 定要不要做；不做就只拿 `checker_red_states=[]` 当「这一轮层 0 没抓到合法状态误红」的一次抽样。

## 没打中的形状

- **F-W 层级 ≥ 2 的树**：三棵树 F-W 代码对层级 1 与 ≥ 2 同一份（容器树经 `walk_the_internal_node_below_a_container_tree_entry` 递归、inode 经 `walk_inode_internal_child` 递归）。本腿只在层级 1 的根上造了镜像（新坏镜像用例与探针都断言 `level == 1`）；层级 2 要 inode / 容器树长到两层，历史长、构造重，本腿没造。读码上层级 ≥ 2 走的是同一份 F-W 判别式，没看出分叉，但层级 2 的镜像一次都没跑到——这一档是空的。
- **F-W 两份各自证成不同别类（C739 前一半）**：本轮不判（随 C739 开着），没攻。
- **容器树 GPU F-W 的对拍流**：本轮没有（进第 31 批），GPU 这三处只读码、没跑。
- **F-IT 挂载根那张表之外的链、链长过上限那一形**：只读码，没造镜像跑 CPU 期望。

## 这条腿自己的限度

- GPU 一侧全凭读码 + CPU 期望，没在卡上跑；GPU 走读内核与 CPU 的等价只到「同偏移、同判别式、同次序」这一层，位级逐态相等要 GPU 对拍（主 agent 跑）。
- 层 0 崩溃枚举没跑（checker 档、前台跑不完）：合法状态误红、b29 三格第二次抽样都停在代码论证 + 交主 agent 的命令，没拿到这一轮的层 0 数。
- 打中与没打中都只抽了一次：照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，这一份「没打中」要再抽一次（第二条同立场腿）才记「没打中」。
- 副本（b30 tree、b29 tree-b29、rerun-check）上量出的数不算入库装置上的数；主 agent 要引须在入库装置上重做。

## 没做什么

- 不判 K1、K3（正推腿的格），不判辩方那几格，不替主 agent 采纳。
- 不跑 GPU、不跑 checker 档（层 0、crash_enumeration、GPU 对拍整表），不跑 `gate.sh` 全量与各类 lint。
- 本腿副本上的 I-9.2 / I-9.15 / I-2.1 等违例集合是副本上量的，入库要重做。

## 四、引文附录（原样行，供核）

冻结副本 `~/.local/share/singlefs/work/m3-batch30-code-r1/tree/`：

- 「fn some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(」(`crates/singlefs-checker/src/walk.rs:4447`)
- 「if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(」(`crates/singlefs-checker/src/walk.rs:2346`)
- 「if a_copy_of_the_class_the_entry_type_names_verifies {」(`crates/singlefs-checker/src/walk.rs:2724`)
- 「self.judge_birth_identity_against_the_pointer(birth_identity, &pointer, &what);」(`crates/singlefs-checker/src/walk.rs:3181`)
- 「fn judge_birth_identity_against_the_pointer(」(`crates/singlefs-checker/src/walk.rs:1050`)
- 「if !self.visited_units.insert(placement_it_was_read_from) {」(`crates/singlefs-checker/src/walk.rs:3167`)
- 「pub fn read_referenced_unit(」(`crates/singlefs-checker/src/image.rs:1501`)
- 「w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance」(`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1641`)
- 「w_judge(INVARIANT_I_1_12, eq64(pointer.birth_tree, p.birth_tree));」(`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1636`)
- 「if (!w_visit(device_read_from, slot_read_from)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1620`)
- 「fn w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> bool {」(`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3061`)
- 「&& w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, class_the_inode_entry_type_names)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2679`)
- 「if (class_of_the_child_of_an_internal_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 2u)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3156`)
- 「if (class_of_the_child_of_a_container_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 3u)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3209`)

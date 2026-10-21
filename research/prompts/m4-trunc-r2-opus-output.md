# m4-trunc-r2 攻方腿（Opus）报告：截在单元边界再变长，甲、乙、丙三条臂的反例（2026-10-17）

<!-- doc-lint:not-numbers W1 W2 W3 W4 W5 K1 K2 K3 K5 K6 K12 S3 S4 -->

正文 `research/prompts/_m4-trunc-r2-body.md`，背景材料 `_m4-trunc-r2-background.md`。开工快照 `m4-trunc-r2-snapshot/kb-sha256.txt` 在仓根 `sha256sum -c` 八个文件全部 OK。代码行号按工作树 `~/.local/share/singlefs/work/m4-predev` 现查。这一轮按派发不起 cargo、不编译、不跑任何测试，量的只有 python3 纯算术小模型（经 `scripts/hosts/run-with-memory-cap.sh 2G`、`scripts/hosts/capped.sh 31`）。

## 复跑

在仓根跑（约 100 秒，8 个并行件之内的 python3 进程，内存峰值每件约 80 MB）：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m4-trunc-r2-opus-model/tools/rerun.sh \
  --repo ~/.local/share/singlefs/work/m4-predev \
  --frozen ~/.local/share/singlefs/work/m4-predev/research/prompts/m4-trunc-r2-snapshot \
  --work <新的空目录>
```

`--frozen` 只要求是目录，这条腿没有拷代码副本，传的是开工快照目录。末尾逐份比 `out/`，这一次在 `~/.local/share/singlefs/work/m4-trunc-r2-opus/rerun1` 跑的原样输出（路径里的家目录换成 `~`）：

```
name=rerun_compare file=fixed.txt same_as_model_out=yes
name=rerun_compare file=exhaustive.txt same_as_model_out=yes
name=rerun_compare file=after_prefix.txt same_as_model_out=yes
name=rerun_compare file=checkpoint.txt same_as_model_out=yes
name=rerun_compare file=random.txt same_as_model_out=yes
name=rerun_compare file=future.txt same_as_model_out=yes
name=rerun_compare file=checkpoint_pairs.txt same_as_model_out=yes
name=rerun_summary all_same_as_model_out=yes
```

单看某一格不必全跑：`bash scripts/hosts/run-with-memory-cap.sh 2G python3 -I research/prompts/m4-trunc-r2-opus-model/model.py fixed | grep -F '<那一行的 history= 与 arm=>'`（1 秒）。

模型目录各文件 sha256：

```
527df2ff2a6104b071407337539236c8f67bad7adc3ea34a6ddff59f5e85e6f0  model.py
f82a56dbe84614d9562da542c0bdba1019113187dbcd0bcc5f30336434ecf278  tools/rerun.sh
1d950b1039e711653c4c66d708c3c762acf0755898c20e2d913b9bf504169cd5  tools/run_all.sh
fb07155e87b5aebd32d4254100cacd0fbbdac3311d90e75d877b095395f0999b  out/fixed.txt
0819954ca7aea4859b0a05bf2a767a5d479ff5f8de6d64c3606307086f4545a4  out/exhaustive.txt
a7e6b9193fdbc8072afd044c745906c79e688a8995d88f1344883323c9e52a8e  out/after_prefix.txt
8fbf9d8f6472495e20afd41e8653d95564fa01c4c46d7f37cff902aab2170c79  out/checkpoint.txt
d4ac8a4ae82ee99f9062a40006f0c745dd12fb8ab9a6c7a43546045e923e2d80  out/checkpoint_pairs.txt
da46360ef3cc1725419eeaefd9d1872187dc87deada627bf16dbf7ea5be382ee  out/random.txt
a0bb987f89b70e7661dba02adc52b0d32b02b1e678038730edc9792a0f904adb  out/future.txt
```

## 各格判定一览

「中」= 小模型上造得出反例；「没中」= 这条腿的取样范围里没造出（只抽一次样，按规则不当「没打中」引）。数都是模型上量的，没在入库装置上量。

| 格 | 甲（inode 记录加截断字段） | 乙（区间墓碑） | 丙（尾单元一律重写） |
|---|---|---|---|
| W1/W2 今天的入口，字面读法 | **中**：只记「最近一次截断」时，截到 N1 → 变长 → 截到 N2 > N1（落在洞里）把 N1 那次放掉的单元认回来；写序去掉也中（覆盖写变短再变长） | 写序保留时没中；写序去掉中（同一形） | 没中 |
| W1/W2 今天的入口，最强读法 | 水位（截断取 min、整份写清零、「size 之外」从 ⌈N ÷ 净荷⌉.max(1) 起、写序保留）没中；三个细节任一放松就中（两个方向） | — | — |
| W2 攒批（D16 已定项 10：一个 txg 装几次改动；今天的会话造不出） | **中**（比较按出生 txg） | **中** | **中** |
| 同上，比较改按写序 (实例, 事务号) | 没中 | 没中（「不晚于」读法反而判死单元 0） | 没中 |
| W3 快照下截断、回退、回退进截断中间态、截断中间态各批 | 水位读法没中；字面读法在回退、快照那几条前缀之后放开扫照样中 | 没中；照 D18 已定项 10 回收规则初值回收墓碑时**中** | 没中 |
| W3 整理搬单元 | 不改出生 txg 与写序（推的，按代码），三臂都不受影响 | 同 | 同 |
| W4 改坏那一句 | 变长或 N = S 那次重建记录丢了字段：固定历史第 2 步红（量的） | 去掉区间墓碑的写：同今天（推的）；今天 `rebuild.rs` 两处会把截过的文件整张报不适用（推的） | 去掉「边界也重写」：同今天（推的） |
| W5 有 pwrite、punch 以后 | **中**：写序那一条判死活单元；单个字段装不下中间的洞 | 写序保留时**中**（判死）；每次放掉都写墓碑、去掉写序时没中 | **中**：靠写序，写序一破就没有别的依据 |

「甲比丙多罩了哪几种历史」在本文「甲比丙多罩了什么」一节：今天的入口上一条都没有（最强读法与丙在全部取样上都成立，字面读法比丙少罩）；以后有 pwrite、punch 时两臂都不成立，甲多罩的 266 条、丙多罩的 8 条都不是「成立」意义上的罩住，见那一节。

## 甲取的是哪一句

正文给甲的原话两处，都抄：

- 题面：「用户 2026-10-17 在弹窗里选了「inode 记录加截断字段」：记录加「最近一次截断的 txg 与 size」，读法 Z 加一条，那个 size 之外、出生 txg 早于那次截断的单元不认领」——这一行里有嵌套书名号，不作逐字引文，出处是正文 `_m4-trunc-r2-body.md` 第一节第二段。
- W1：正文 W1 甲那一条里「（或别的形态，例如「自某 txg 起偏移某处之后的单元一律作废」的水位）」。

这条腿量了四种写法，**最强的一种**（下称甲二，模型里的 `jia_watermark`）逐字取成下面这一句判法，接在 K6 三条之后：

> 记录偏移 108 起写 (T_w, N_w)，全零是没写过。截断意图那次：没写过时写 (这次 txg, N)，写过时写 (这次 txg, min(N, N_w))；整份写（顺序写、覆盖写）那次写全零；变长与 N = S 那次照抄。扫描重建认领时多判一句：T_w ≠ 0、单元号 ≥ ⌈N_w ÷ 净荷⌉.max(1)、出生 txg < T_w 的单元不认领。K6 的写序那一条保留。

另外三种是用来答「哪一个细节承重」的对照：甲一 `jia_latest`（字面「最近一次」，截断写 (txg, N)，整份写照抄）；甲二的三个放松，`jia_watermark_boundary_without_max_one`（起点取 ⌈N_w ÷ 净荷⌉ 不取 max(1)）、`jia_watermark_no_clear`（整份写不清零）、`jia_latest_no_write_order`（去掉写序）；甲四 `jia_full_no_write_order`（整份写也写 (这次 txg, 新长度)、截断取 min、去掉写序）。乙取正文 W1 原样：截断意图那次写一条墓碑 [⌈N ÷ 净荷⌉ × 净荷, ∞)、死亡 txg = 意图 txg，区间之内、出生 txg 早于死亡 txg 的不认领，K6 三条保留。丙取正文 W1 原样：留下的最后一个单元存在就重写，判法不动。

## 模型抄的是哪几行

`model.py` 逐条抄这几处，名字与次序照原函数：

- 单元数：`crates/singlefs-core/src/truncate.rs:28`：「pub fn data_units_a_size_needs(size_in_bytes: u64) -> u64 {」，模型 `units_needed` = ⌈size ÷ 32634⌉.max(1)。
- 声明长度：`crates/singlefs-core/src/truncate.rs:34`：「pub fn declared_length_within_a_size(unit: DataUnitIndexInFile, size_in_bytes: u64) -> u64 {」。
- 截断意图那次重写不重写：`crates/singlefs-core/src/truncate.rs:100`：「if unit.0 >= data_units_existing {」与 `crates/singlefs-core/src/truncate.rs:104`：「(declared_length_within_a_size(unit, size_before_in_bytes) != declared_length_in_bytes)」；丙把这一句换成「k 存在就重写」。
- 变长那次：`crates/singlefs-core/src/truncate.rs:127`：「(declared_length_within_a_size(unit, size_before_in_bytes) < declared_length_in_bytes)」。
- 批：`crates/singlefs-core/src/truncate.rs:155`：「if data_units_beyond_the_size <= data_units_per_batch {」。模型按「size 之外的从尾部放一批」写，单元从 0 起连号时逐步断言与 `next_truncation_step` 相同（`model.py` 的 `continue_truncation`），全部取样没有一次断言失败。
- 当前版本：同一单元号的各版里写序 (实例, 事务号) 最大的，`crates/singlefs-checker/src/rebuild.rs:213`：「ContentUnitClass::Data => CurrentVersionOrderKey::DataUnitWriteOrder {」；先按谱系滤，`crates/singlefs-checker/src/rebuild.rs:717`：「.filter(|(_, version)| txgs_on_the_lineage.holds(birth_txg_of(version)))」；谱系段照 `crates/singlefs-checker/src/rebuild.rs:338`：「fn txgs_on_the_lineage_of_the_newest_version(」。
- 写序那一条：`crates/singlefs-checker/src/rebuild.rs:593`：「*newest_at_the_offset = (*newest_at_the_offset).max(birth_txg_of(version));」与 `crates/singlefs-checker/src/rebuild.rs:783`：「.is_some_and(|newest_below| *newest_below > birth_txg_of(version))」。注意它在**全部当前版本**（含放掉而还在盘上的）上取「之下最大」，不只在认领的上取。
- size 那一条：`crates/singlefs-checker/src/rebuild.rs:778`：「if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {」。
- 墓碑只认死亡 txg 在谱系上的：`crates/singlefs-checker/src/rebuild.rs:700`：「if txgs_on_the_lineage.holds(read_u64(record, TOMBSTONE_RECORD_DEATH_TXG_OFFSET)) {」；乙的区间墓碑照这一句滤谱系。
- 遍历一侧：现行 extent 树里、size 之内的单元，`crates/singlefs-checker/src/walk.rs:13612`：「let lies_beyond_the_size_of_its_record =」；快照钉住的不进中央映射，`crates/singlefs-checker/src/walk.rs:13649`：「!keys_pinned_by_the_deadlist.contains(&key)」。

模型比的是「每个 inode 的数据单元集合」一张表：每个单元号上扫描重建认领的版本与遍历读到的版本是否相同。不同有两种：重建多认了（下称「认回来」）、遍历有而重建没认（下称「判死」）。中央映射的数据单元那一半由同一组认领定，模型不另算。

模型没抄、按条款推的几样：每次用户改动一次发布、一个 txg（今天的会话如此，`crates/singlefs-core/src/transaction.rs:8555`：「pub fn publish_size_change<Device: BlockDevice>(」以下每条发布路径各取 `checkpoint_txg_of_the_publish_after`）；覆盖写只写一个单元、放掉其余（`crates/singlefs-core/src/mounted_session.rs:99`：「/// 覆盖写一个数据单元以内的内容（`transaction::publish_overwrite`）。」）；回退取最近三次发布之一、带快照的版本上拒、截断条目随 R_old 续做（判决 K1）；放掉的版本留在盘上，随机跑批里每个状态另抽两次「槽被复用」的子集（不在现行、不被快照钉、不在最近五次发布里的版本各以一半概率抹掉）。

交叉核：正文题面那条写死历史（300 个单元 → 截到一个净荷 → 变长到 3 × 净荷 + 1），模型在今天的判法上认回单元 1、2、3，与正文报的「中央映射差 3 处」条数相同：

```
name=fixed history=fixed_history_of_the_body_300_units_cut_at_one_payload_then_3P_plus_1 batch=144 arm=today states=6 verdict=resurrects first=state=extension_to_3P+1@txg9 resurrected=unit1(birth4);unit2(birth4);unit3(birth4)
```

（`out/fixed.txt` 第 1 行；同一历史上丙、甲二、乙三行 `verdict=holds`，第 2、5、10 行。）

## 取样范围

| 跑法 | 历史 | 每条臂的历史数 | 状态数（批 144 / 批 1） |
|---|---|---|---|
| fixed | W2 列的七种加这条腿的八种，共 15 条写死历史，批 144 与批 1 | 30 | 见 `out/fixed.txt` |
| exhaustive | 先顺序写 4 个净荷，之后 4 步在 19 个动作里全排（顺序写 0、半个、1、2、4 个净荷 + 1；覆盖写半个；改 size 到 0、半个、1、1 + 1 字节、2、3、4、6 个净荷；建快照、删快照、回退 1/2/3 次发布），批 144 与批 1 | 130 321 × 2 | 702 382 / 829 845 |
| after_prefix | 15 条写死历史各去掉最后一步当前缀，之后 3 步在同一字母表上放开扫（攻方定义第 3 条） | 6859 × 15 | — |
| random | 12 步、动作随机、批大小在 144 / 1 / 2 里抽，每个状态另抽两次槽复用子集，种子 20261017 | 30 000 | 约 41 万 |
| checkpoint | 字母表再加六种「一个 checkpoint 里接连几次改动」，4 步全排 | 15 625 × 2 | 65 485（批 144） |
| future | 字母表再加 pwrite（写单元 1/3/6）与 punch（[1,3)、[2,5)），8 步随机 3000 条，加 4 条写死历史 | 3000 | 约 2.7 万 |

估时：先跑一条臂的 after_prefix（26.6 秒）与 random 3000 条（1.3 秒），按 22 条臂 × 4 种跑法估 CPU 约 27 分钟，16 件并行挂钟约 2 分钟；实跑 102 秒。没有缩取样。崩溃状态只取到「每次发布之后的根」（截断意图、每一批、最后一批都是一个根），没有逐写枚举崩溃子集，见末节「这条腿自己的限度」。

## W1、W2：今天的入口上

复跑任一行：`bash scripts/hosts/run-with-memory-cap.sh 2G python3 -I research/prompts/m4-trunc-r2-opus-model/model.py fixed | grep -F 'history=<名> batch=144 arm=<臂> '`；exhaustive 那几行用 `… model.py exhaustive 5 <臂>`。下面贴的都是这一次跑出的原样行。

### 打中一：甲一（字面「最近一次截断」）把第一次截断放掉的单元认回来

- 历史（W2 列的第四种）：顺序写 5 个净荷（txg 4）→ 截到 1 个净荷（txg 5 意图，单元 0 满、不重写；txg 6 放掉单元 1..4）→ 变长到 5 个净荷（txg 7，单元 0 满、不重写）→ 截到 3 个净荷（txg 8，单元 2 不存在、不重写，放 0 个）→ 变长到 5 个净荷。
- 打中的句子：甲一的「记录加「最近一次截断的 txg 与 size」」。txg 8 那次把字段改成 (8, 3 个净荷)，txg 5 的 (5, 1 个净荷) 没了；单元 1、2 出生 txg 4、偏移在 3 个净荷之内、不在字段的「之外」；写序那一条要「之下有出生 txg 更大的」，而单元 0 也是 txg 4；size 那一条放行。截断意图那一刻就认回来，不用等变长。
- 输出：`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:163`：「name=fixed history=cut_to_N1_extend_cut_to_N2_above_N1_in_the_hole_extend batch=144 arm=jia_latest states=7 verdict=resurrects first=state=truncation_intent_to_3P@txg8 resurrected=unit1(birth4);unit2(birth4)」
- 同一历史上丙、甲二、乙：`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:162`：「arm=bing states=7 verdict=holds first=-」，`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:165`：「arm=jia_watermark states=7 verdict=holds first=-」，`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:170`：「arm=yi states=7 verdict=holds first=-」。
- 全排里最短的一条只要 5 步，第二次截断从 2 个净荷截到 1 个净荷 + 1 字节（落在洞里的单元 1 中间）：`research/prompts/m4-trunc-r2-opus-model/out/exhaustive.txt:8`：「first_resurrects: ops=seq(4P),seq(2P),size(1P),size(2P),size(1P+1) state=truncation_intent_to_1P+1@txg9 resurrected=unit1(birth5)」；全排 130 321 条里中 569 条（同文件第 7 行）。
- 四句：① 分辨臂：分得开，同一段历史上丙、甲二、乙都成立。② 被判的系统当时看不看得到：甲一下看不到——txg 5 那次截断的信息只在被盖掉的那一版记录里，重建只读当前版本那条记录（`crates/singlefs-checker/src/rebuild.rs:717` 那一滤之后取最大的一版）；病在臂丢了信息，不在判据。③ 满足的分句：I-1.13「扫描重建与遍历方向的索引相同」的「每个 inode 的数据单元集合」那一张，重建多认两个单元。④ 改法：甲二的「取 min」在这一格成立（第 165 行），丙、乙也成立。

### 打中二：写序那一条去掉，甲、乙都把覆盖写放掉的单元认回来

- 历史：顺序写 3 个净荷（txg 4）→ 覆盖写半个净荷（txg 5，单元 0 重写，单元 1、2 放掉；不是截断，甲不写字段、乙不写墓碑）→ 变长到 3 个净荷。
- 打中的句子：甲、乙「读法 Z 加的那一条」只在截断写；覆盖写、顺序写变短放掉的单元只有写序那一条拦。
- 输出：`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:404`：「arm=jia_latest_no_write_order states=3 verdict=resurrects first=state=extension_to_3P@txg6 resurrected=unit1(birth4);unit2(birth4)」；`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:411`：「arm=yi_no_write_order states=3 verdict=resurrects first=state=extension_to_3P@txg6 resurrected=unit1(birth4);unit2(birth4)」。写序保留的甲一、甲二、乙在同一历史上成立（第 403、405、410 行）。
- 四句：① 只分辨「写序留不留」，不分辨三条臂。② 看得到：单元 0 出生 txg 5 在盘上。③ 同一分句。④ 两条出路，各量过：写序保留（甲二、乙全排都成立，`exhaustive.txt` 第 15、31 行），或整份写也写字段 / 墓碑而去掉写序（甲四、乙全，`research/prompts/m4-trunc-r2-opus-model/out/exhaustive.txt:25`：「arm=jia_full_no_write_order histories=130321 states=702382 holds=130321」，`research/prompts/m4-trunc-r2-opus-model/out/exhaustive.txt:37`：「arm=yi_full_no_write_order histories=130321 states=702382 holds=130321」）。所以 W2「写序那一条还要不要」的答：甲、乙都要，除非整份写也写同一种记账。

### 打中三：甲二的「size 之外」从 ⌈N ÷ 净荷⌉ 起（不取 max(1)），判死单元 0

- 历史：顺序写 4 个净荷 → 顺序写半个净荷（txg 5，字段清零）→ 截到 0（txg 6，单元 0 重写成声明长度 0，字段 (6, 0)）→ 变长到 1 个净荷 + 1 字节（txg 8，单元 0 补零重写，出生 txg 8；字段照抄）→ 截到 1 个净荷（txg 9，单元 0 已满、不重写，字段 (9, min(1 个净荷, 0)) = (9, 0)）。
- 打中的句子：「那个 size 之外」若读成偏移 ≥ N_w，N_w = 0 时单元 0 也在「之外」，出生 txg 8 < 9，被判死；而它是活单元。
- 输出：`research/prompts/m4-trunc-r2-opus-model/out/exhaustive.txt:18`：「first_kills_live: ops=seq(4P),seq(0P+16317),size(0P),size(1P+1),size(1P) state=truncation_intent_to_1P@txg9 killed=unit0(birth8)」；全排中 730 条（第 17 行）。
- 四句：① 只分辨甲的两种写法。② 看得到：起点取 ⌈N_w ÷ 净荷⌉.max(1) 就不会把单元 0 算进去，K6 的 size 那一条本来就这么取（`crates/singlefs-checker/src/rebuild.rs:778` 用 `data_units_the_size_covers`）。③ 同一分句，方向是判死。④ 甲二取 max(1)（全排、放开扫、随机都成立）。

### 打中四：甲的水位整份写不清零，判死整份写写出的单元

- 历史：顺序写 3 个净荷 → 截到 1 个净荷（txg 5，字段 (5, 1 个净荷)）→ 顺序写 5 个净荷（txg 7，字段照抄）→ 截到 2 个净荷（txg 8，字段 (8, min(2, 1) 个净荷)）。
- 打中的句子：水位取 min 而顺序写不清零，txg 7 写出的单元 1 在「1 个净荷之外」、出生 7 < 8，判死。
- 输出：`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:207`：「arm=jia_watermark_no_clear states=7 verdict=kills_live first=state=truncation_intent_to_2P@txg8 killed=unit1(birth7)」。
- 四句：① 只分辨甲的写法。② 看得到：整份写那次本来就重写 inode 记录（`crates/singlefs-core/src/transaction.rs:7314`：「fn inode_record_of_file_version(file: &FileVersionPlan<'_>) -> InodeRecord {」），清零不多一个写。③ 同一分句，判死。④ 甲二清零（成立）；甲四改成整份写写 (txg, 新长度)（也成立）。

小结：甲的「一句判法」要同时钉三个细节才站得住——截断取 min（不是「最近一次」）、整份写清零或改写、起点取 .max(1)，而且写序那一条保留（或整份写也记）。正文 W1 的两种说法都没写全这三样。

### 打中五：攒批下三条臂一起认回来（打中不分辨臂，另立一笔账）

- 前提：D16 已定项 10 已定「攒够一批脏节点（或到时间）再发布一次根」（`.claude/kb/decisions/16-发布语义.md:255`：「**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。」）。一个 txg 里可以先顺序写、再截断、再变长。今天的会话每次改动单独发布，造不出这一形；模型用 `checkpoint` 动作把几次改动放进同一个 txg。
- 历史：顺序写 4 个净荷 → 顺序写 0 → 顺序写 0 → 一个 checkpoint 里：顺序写 4 个净荷 + 1 字节、截到 2 个净荷、变长到 4 个净荷。
- 打中的句子：三条臂与 K6 写序那一条都按出生 txg 比。单元 2、3 是这个 checkpoint 里写出、又在同一个 checkpoint 里放掉的，出生 txg 与截断意图同为 7：甲二「出生 txg 早于那次截断」、乙「出生 txg 早于死亡 txg」都不成立；丙重写的尾单元也是 txg 7，写序那一条要「更大」，也不成立。
- 输出：`research/prompts/m4-trunc-r2-opus-model/out/checkpoint.txt:6`：「first_resurrects: ops=seq(4P),seq(0P),seq(0P),checkpoint[seq(4P+1)+size(2P)+size(4P)] state=checkpoint_of_seq(4P+1)+size(2P)+size(4P)@txg7 resurrected=unit2(birth7);unit3(birth7)」（丙那一行；甲二第 18 行、乙第 44 行同一条历史）。全排 15 625 条里丙、甲二、乙各中 2119 条（第 5、17、43 行）。
- 改成按写序 (实例, 事务号) 比（甲二的字段与乙的死亡都记这次改动之前最后一个写序 + 1，K6 那一条也比写序）：`research/prompts/m4-trunc-r2-opus-model/out/checkpoint.txt:59`：「arm=bing_by_write_order histories=15625 states=65485 holds=15625」，第 61 行甲二、第 65 行乙同为全部成立。乙改成「不晚于死亡 txg」反而判死：`research/prompts/m4-trunc-r2-opus-model/out/checkpoint.txt:68`：「first_kills_live: ops=seq(4P),seq(0P),seq(0P),checkpoint[seq(2P)+size(0P)] state=checkpoint_of_seq(2P)+size(0P)@txg7 killed=unit0(birth7)」（截到 0 的区间从 0 起，重写的单元 0 与死亡同 txg）。甲二「不晚于」成立（第 63 行），因为整份写清零。
- 四句：① 不分辨臂：丙、甲二、乙与今天的判法一起中。按证据纪律不拿它判臂，另立一笔账：「扫描重建里按出生 txg 比新旧的每一处，在一个 txg 装几次改动时都不够细」。② 看得到：写序在每个码 1 单元头里（`crates/singlefs-checker/src/rebuild.rs:213` 那一支读的就是它），重建做决定时看得到。③ 同一分句，认回来。④ 改法：比较换成写序，三条臂都成立（量的）；「不晚于」只对甲二成立。

## W3：快照、回退、删除、整理

**今天扫描重建按哪一版的记录判**：只按当前版本那片 inode 叶容器里的记录判。inode 叶容器随回退走，只在诞生代号落在谱系上的各版里取写序最大的一版（`crates/singlefs-checker/src/rebuild.rs:717`：「.filter(|(_, version)| txgs_on_the_lineage.holds(birth_txg_of(version)))」），快照那一版的记录是更老的版本，不会被取。所以快照钉住、活头已放掉的单元，重建一侧要靠 K6 三条与新加的那一句判死；遍历一侧它们不进中央映射（`walk.rs:13649`）、也不在现行 extent 树里。

| 历史（写死的，`fixed.txt`） | 今天 | 甲一 | 甲二 | 乙 | 丙 |
|---|---|---|---|---|---|
| 快照下截在边界、变长、删快照、再变长 | 中 | 没中 | 没中 | 没中 | 没中 |
| 截断后回退到截断之前、再变长 | 中 | 没中 | 没中 | 没中 | 没中 |
| 回退进截断中间态（意图那一版）、续做、变长 | 中 | 没中 | 没中 | 没中 | 没中 |
| 截断中间态各批（批 1、批 144 两种，每一批是一个根） | 意图、各批、最后一批上都没中，中在变长那一步 | 同 | 同 | 同 | 同 |

放开扫（每条写死历史去掉最后一步当前缀，之后 3 步在 19 个动作里全排，6859 × 15 = 102 885 条）：甲二、乙、丙都是 0 条中；甲一中 23 830 条，其中「快照下截断」「回退到截断之前」两条前缀之后都有：

（23 830 是对 `out/after_prefix.txt` 甲一那十五行求和。）原样行举三条：`research/prompts/m4-trunc-r2-opus-model/out/after_prefix.txt:57`：「history=cut_then_rollback_to_before_the_cut_then_extend suffix_length=3 arm=jia_latest histories=6859 holds=6702 resurrects=157」，`research/prompts/m4-trunc-r2-opus-model/out/after_prefix.txt:59`：「history=rollback_into_the_middle_of_a_cut_then_extend suffix_length=3 arm=jia_latest histories=6859 holds=6549 resurrects=310」，`research/prompts/m4-trunc-r2-opus-model/out/after_prefix.txt:61`：「history=cut_under_a_snapshot_then_extend_then_destroy_the_snapshot suffix_length=3 arm=jia_latest histories=6859 holds=5166 resurrects=1693」。

**回退时字段跟着退还是留着**：甲的字段在 inode 记录里，随 inode 树退回 R_old；截断条目也随 R_old（判决 K1）。模型照这样做，回退进截断中间态、回退到截断之前都成立。乙的墓碑树随现行那一版、不随回退，回退抛弃那一段里写的墓碑靠「死亡 txg 在谱系上」那一滤失效（`crates/singlefs-checker/src/rebuild.rs:700`），模型照这样做，同样成立。没找到让「字段随 R_old」与「墓碑随现行」分出高下的历史。

**unlink**：整对象墓碑与甲的字段、乙的区间墓碑都不冲突——整对象墓碑压住 (对象, 对象出生代) 的全部单元。乙要注意的是实现：今天重建对「墓碑压着而记录还在」整张报不适用（`crates/singlefs-checker/src/rebuild.rs:768`：「return Err(ScanRebuildNotDecided::DeletionInProgressNotWrittenForTheScanRebuild);」），对不是整对象区间的墓碑也整张报不适用（`crates/singlefs-checker/src/rebuild.rs:697`：「ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion,」）。乙若把区间墓碑并进同一个集合，每个截过的文件之后 I-1.13 都不判（推的，没改代码量）——这不是认错，是把这一判整个关掉，比认错更难发现。乙要把区间墓碑另存一张，两处不适用只对整对象墓碑判。

**整理搬单元**：不改出生 txg，也不改头。`crates/singlefs-core/src/transaction.rs:7846`：「/// 原样字节写到目的地（头不盖戳、birth 不变）、映射条目换到目的地、源落点写释放代进 defer、目的地记一条分配代 = 诞生代的已分配记录」；条款同 `.claude/kb/decisions/05-快照-空间记账机制.md:349`：「**定案**：不改。后台整理把一个单元从一个落点搬到另一个落点，**不是「发布一个新版本」**」。搬过之后同一版有两份副本，重建按归并键合成一版（`choose_current_versions`），出生 txg 与写序都不变，三条臂的判法看到的东西一样；源落点放掉后被复用，等于模型里的「槽被复用」抽样（随机跑批每个状态抽两次），三臂没中。整理只搬活单元与快照钉住的单元，放掉的旧单元不搬（推的，按 D26 已定项 5 的区域扫描读法，没读整理计划的全部代码）。

### 打中六：乙的墓碑照 D18 已定项 10 回收规则初值回收，放掉的单元回来

- 前提：D18 已定项 10 的回收规则初值是 `.claude/kb/decisions/18-块里携带什么信息.md:238`：「**墓碑记录随快照 deadlist 同一套节奏清**——一条墓碑在」，同一节的欠账写着 `.claude/kb/decisions/18-块里携带什么信息.md:251`：「回收判据欠实验（第一版不回收、记录字段表在已定项 24；回收门」。第一版不回收（`.claude/kb/decisions/18-块里携带什么信息.md:579`：「4. **回收**：第一版不回收墓碑」），所以今天乙不中；这一格是给「以后照初值回收」的。
- 历史：顺序写 3 个净荷 → 截到 1 个净荷 → 建快照、删快照各两次（把截断那一版推出回退候选，没有活快照）→ 变长到 3 个净荷 + 1 字节。
- 打中的句子：乙「区间之内、出生 txg 早于死亡 txg 的单元不认领」靠的是墓碑一直在；死亡被全部根送走之后墓碑回收，而放掉的单元字节还在盘上（没被复用）。
- 输出：`research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:540`：「arm=yi_with_tombstones_recycled states=8 verdict=resurrects first=state=extension_to_3P+1@txg11 resurrected=unit1(birth4);unit2(birth4)」；同一历史上不回收的乙成立（第 530 行）。
- 四句：① 只分辨乙回不回收。② 回收的那一刻看不看得到盘上还有没有那些字节：看不到，除非回收门另带「数据单元已被覆盖或清扫」那一维（欠账里写的那一维）。③ 同一分句，认回来。④ 改法：乙的区间墓碑不照初值回收，回收门必须带那一维（推的）。甲没有这一格的对应物——甲的字段被下一次截断盖掉，就是「回收」，打中一已经量了。

## W4：每一处要改的地方里，攻得到的几处

只写这条腿攻到、或必红世界要用的几处；整张改法清单归正推腿。

- **甲，读者一律判不可用**：`crates/singlefs-core/src/records.rs:93`：「|| bytes[108..].iter().any(|byte| *byte != 0)」，池级 checker `crates/singlefs-checker/src/walk.rs:3697`：「record[108..140].iter().all(|byte| *byte == 0),」，不变量 `crates/singlefs-checker/invariants/09-inode树结构.md:19`：「| I-9.7 | 记录三段恒零 |」。甲写下字段之后，这三处不改就是：挂载、恢复读到这条记录判不可用，checker 在每个截过的镜像上红 I-9.7。GPU 判器的 wgsl 里按 `I-9`、`108` 搜，没找到对 inode 记录偏移 108 的判（推的：只搜了两份 checker wgsl 的这两个串）。
- **甲，每个写记录的地方都要决定字段怎么办**：核心层建 `InodeRecord` 的非测试处有五个——整份写 `crates/singlefs-core/src/transaction.rs:7314`、建空 inode `crates/singlefs-core/src/transaction.rs:7577`、unlink 意图 `crates/singlefs-core/src/transaction.rs:8443`、unlink 各批 `crates/singlefs-core/src/transaction.rs:8506`、改 size `crates/singlefs-core/src/transaction.rs:8600`：「let record_after_the_change = InodeRecord {」。甲二要：整份写清零，改 size 那一处分三条（截断写 min、变长与 N = S 照抄）。漏照抄的必红世界：
  `research/prompts/m4-trunc-r2-opus-model/out/fixed.txt:49`：「arm=jia_watermark_extension_drops_field states=4 verdict=resurrects first=state=extension_to_3P+1@txg7 resurrected=unit1(birth4);unit2(birth4)」——就是正文那条写死历史的形状（截在边界、变长），第 2 步红。
- **甲，三个细节各一条必红世界**（都是量的）：「取 min」改成「最近一次」→ 打中一那条历史红（`fixed.txt` 第 163 行）；「整份写清零」去掉 → 打中四那条红（第 207 行）；「.max(1)」去掉 → 打中三那条红（`exhaustive.txt` 第 18 行那条 5 步历史）。三条都不在正文那条写死历史上，要另造用例。
- **乙**：`crates/singlefs-checker/src/rebuild.rs:697`、`crates/singlefs-checker/src/rebuild.rs:768` 两处会把截过的文件整张报不适用（W3 一节）；区间墓碑要另存、两处不适用只对整对象墓碑判（推的）。墓碑树每次截断多一条 40 字节记录、两份，第一版不回收，只增（D18 已定项 24 第 4 条；数是条款里的，没量）。截断意图那次多重写一片墓碑容器到墓碑树根（推的，照 D18 已定项 24 第 3 条「有新记录的那次发布才重写」）。
- **丙**：只动 `crates/singlefs-core/src/truncate.rs:104` 那一句（存在就重写），extent 计划 `plan_the_extent_tree_after_a_size_change` 本来就把罩着重写单元的叶到根并进重写集，不用改。代价是截在单元边界的那次多写一个数据单元与它的下段叶到根（推的，没量绝对值）；快照下截断时旧的那一版被快照钉住，多占一个单元的空间（推的）。必红世界：把这一句改回今天的样子，正文那条写死历史第 2 步红（`fixed.txt` 第 1 行就是今天的样子）。
- **任何一臂，攒批落地时**：K6 写序那一条（`crates/singlefs-checker/src/rebuild.rs:593`、`crates/singlefs-checker/src/rebuild.rs:783`）、甲的字段、乙的死亡，都要改成按写序比（打中五）。今天不用改，记欠账。

## W5：以后有 pwrite、punch 时

`rebuild.rs` 自己已经写了这条前提会破：`crates/singlefs-checker/src/rebuild.rs:615`：「都从偏移 0 整份写、截断只重写最后留下的单元、变长只重写最后一个存在的单元」。模型里 pwrite = 只写单元 o（写到尾部之后时照变长补零最后一个存在的单元），punch = 放掉 [a, b) 里的单元、size 不变（单元 0 不放），这两样的语义是这条腿自己定的。

| 写死历史 | 今天 | 丙 | 甲二 | 甲四 | 乙 | 乙全 |
|---|---|---|---|---|---|---|
| pwrite 写文件中间的单元 1 | 判死 | 判死 | 判死 | 没中 | 判死 | 没中 |
| punch [1,3) | 认回来 | 认回来 | 认回来 | 认回来 | 没中 | 没中 |
| punch 中间再截到洞里、变长 | 认回来 | 认回来 | 认回来 | 认回来 | 没中 | 没中 |
| 截断、pwrite 到远处、再截、pwrite 到中间 | 认回来 | 没中 | 判死 | 判死 | 没中 | 没中 |

（`research/prompts/m4-trunc-r2-opus-model/out/future.txt` 第 1–79 行，每行一条臂。例：`research/prompts/m4-trunc-r2-opus-model/out/future.txt:2`：「history=pwrite_into_the_middle_of_a_file arm=bing verdict=kills_live first=state=pwrite_unit_1@txg5 killed=unit2(birth4);unit3(birth4)」。）

随机 3000 条 8 步（字母表加 pwrite、punch）：`research/prompts/m4-trunc-r2-opus-model/out/future.txt:119`：「arm=yi_full_no_write_order histories=3000 states=27398 holds=3000」；丙成立 2037 条（第 85 行），甲二 2062（第 95 行），甲四 2308（第 107 行），写序保留的乙 2627（第 115 行，其余全是判死）。

- 写序那一条在 pwrite 下判死活单元：pwrite 把单元 1 写成新 txg，单元 2、3 的「之下」就有更大的出生 txg。三条臂里用到它的（今天、丙、甲二、乙）一起中——这一格对写序是「打中不分辨臂」，病在共用前提，正文与 `crates/singlefs-checker/src/rebuild.rs:615` 已经预告。
- 写序去掉之后各臂还剩什么：丙什么都不剩（它只是给写序造一个「之下更新」的单元）；甲只有一个字段，punch 打出的中间的洞装不下，水位取 min 又会在 pwrite 写到水位之外后判死；乙的墓碑记录本来就有区间止（D18 已定项 24 第 2 条的 40 字节），每次放掉（截断、punch、整份写变短）都写一条、去掉写序，在全部取样上成立；攒批下再改按写序比也成立（第 140 行）。
- 所以：到 pwrite、punch 那一天，丙要撤、甲要重开（一个字段要变成一张表，那就是乙的墓碑树），乙要补「整份写变短也写墓碑」并去掉写序；写序那一条三臂都要重开。

## 甲比丙多罩了什么

在同一批历史上数「A 成立而 B 不成立」的条数（模型 `report_pairs`，量的）：

| 入口 | 甲二比丙多罩 | 丙比甲二多罩 | 出处 |
|---|---|---|---|
| 今天的入口（全排 130 321 × 2、放开扫 102 885、随机 30 000） | 0 | 0 | 两臂在三种跑法上全部成立：`exhaustive.txt` 第 5、15 行，`random.txt` 第 3、8 行，after_prefix 两臂各 0 条中 |
| 攒批、按出生 txg 比 | 0 | 0 | `research/prompts/m4-trunc-r2-opus-model/out/checkpoint_pairs.txt:1`：「name=checkpoint_pair arm_a=jia_watermark arm_b=bing histories=3000 a_holds_b_not=0 b_holds_a_not=0」（两臂在同样的历史上一起中） |
| 攒批、按写序比 | 0 | 0 | 同文件第 7 行（两臂全部成立） |
| 以后有 pwrite、punch | 5 | 2 | `research/prompts/m4-trunc-r2-opus-model/out/future.txt:145`：「name=future_pair arm_a=jia_watermark arm_b=bing histories=3000 a_holds_b_not=5 b_holds_a_not=2」 |
| 同上，甲四（整份写也记、去掉写序）对丙 | 266 | 8 | 同文件第 148 行 |

答：

- **今天的写入口**：甲（最强读法）比丙一条都没多罩；甲的字面读法比丙少罩（全排中 569 条、放开扫中 23 830 条，丙都是 0）。今天的入口上能让两臂分开的东西，这条腿没找到。
- **攒批落地之后**：两臂一样，都要把比较换成写序才成立。
- **以后有 pwrite、punch**：两臂都不成立，「多罩几条」只是在两个都大面积出错的臂之间比谁错得少（甲四 3000 条里成立 2308、丙 2037），不能当「甲罩得住」读。真成立的只有乙全。

## 改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 状态 |
|---|---|---|
| 甲写成甲二：截断取 (txg, min(N, 旧 N_w))、整份写清零、变长与 N = S 照抄、起点 ⌈N_w ÷ 净荷⌉.max(1)、写序保留 | 打中一、三、四 | 量过：`research/prompts/m4-trunc-r2-opus-model/out/exhaustive.txt:15`：「arm=jia_watermark histories=130321 states=702382 holds=130321」，随机 30 000 全成立（`random.txt` 第 8 行），放开扫 0 条中 |
| 扫描重建里按出生 txg 比新旧的每一处（K6 写序那一条、甲的字段、乙的死亡）改按写序 (实例, 事务号) | 打中五（攒批） | 量过：`checkpoint.txt` 第 59、61、65 行全成立；推的：写序跨挂载单调（实例代号只增），这条腿没核 |
| 乙的区间墓碑另存一张，`crates/singlefs-checker/src/rebuild.rs:697`、`crates/singlefs-checker/src/rebuild.rs:768` 两处不适用只对整对象墓碑判 | W3 unlink 那一段 | 推的（没改代码） |
| 乙的墓碑不照 D18 已定项 10 初值回收，回收门带「数据单元已被覆盖或清扫」那一维 | 打中六 | 初值会中是量过的（`fixed.txt` 第 540 行）；带那一维之后成立是推的 |
| 到 pwrite、punch 那一天：每次放掉（截断、punch、整份写变短）都写区间墓碑、去掉写序 | W5 | 量过：`future.txt` 第 119、140 行全成立；pwrite、punch 的语义是这条腿定的 |

## 没打中的形状

这些形状在取样里试过，三臂里甲二、乙、丙一条都没中（只抽一次样，按 `.claude/rules/three-way-inference.md` 不当「没打中」引）：

- 截在边界再变长、截到单元中间再变长、截到 0 再变长、截到 N1 → 变长 → 截到 N2 > N1 → 变长、截断后整份写更长再截短、截断后整份写更长再截到中间、截在洞里两次、截→变长→再截更低→变长→截到洞里（`fixed.txt` 15 条，批 144 与批 1）。
- 快照下截断、删快照之后再变长；回退到截断之前；回退进截断意图那一版再续做；回退之后再截、再回退（全排里回退 1/2/3 次发布与每种动作交错）。
- 每次发布之后的根上判（截断意图、每一批、最后一批都是根），批大小 1、2、144。
- 槽复用：随机跑批每个状态抽两次子集抹掉放掉的版本（不在现行、不被快照钉、不在最近五次发布里），30 000 条 × 约 14 个状态 × 2。
- 写序与出生 txg 的「之下最大」取在全部当前版本上（含放掉的），没有造出它判死活单元的今天的历史。

## 这条腿自己的限度

- 模型是照代码抄的 python，不是 Rust；抄错的地方由「单元从 0 起连号时逐步断言与 `next_truncation_step` 相同」核了一处，其余各处没有对拍。正文写死历史的条数对得上（3 个单元）是一处旁证。
- 崩溃状态只取到每次发布之后的根，没有枚举一次发布里写到一半的崩溃子集（没调 `enumerate_crash_states`，这一轮不许起 cargo）。一次发布里写了一半的单元按「已发布谓词」不进重建，这一点是推的。
- 槽复用是每次判之前现抽，不随历史保留；回退只取最近三次发布；带快照时回退一律拒；实例代号恒定（不模拟重挂载）；只有第一个文件、一个对象出生代。
- 攒批只把几次改动放进同一个 txg，没模拟 journal 前缀、checkpoint 里中间态不落叶（D8 已定项 13）那些细节。
- pwrite、punch 的语义是这条腿定的，正文没给。
- 只比「每个 inode 的数据单元集合」一张表；inode 记录集合与中央映射的码 3 那一半没比（甲改了记录字节，两侧读同一片容器，推的不会分叉）。
- 整理不改出生 txg 与写序是读代码与条款推的，模型里没有整理这个动作。

## 没做什么

- 没起 cargo、没编译、没跑任何 Rust 测试、没起 GPU、没跑门禁阶段（派发的资源约束）；副本没建。
- 没判正推、辩方那几格；W4 的完整改法清单与行数估计归正推腿，这里只写攻到的几处。
- 没替主 agent 采纳；模型上的数不是入库装置上的数。
- 草稿目录 `~/.local/share/singlefs/work/m4-trunc-r2-opus/` 里留着开发用的 `dev/`（模型的工作稿与几次试跑输出）、`rerun1/`（一次复跑的输出）、`fixed-again.txt`、`rerun1.log`，都是 python 与文本，没有编译目录与仓副本；留给主 agent 核对复跑用。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-trunc-r2-opus-output.md --root ~/.local/share/singlefs/work/m4-predev` 的末行：

```
  ✓ 核了 56 处引文、18 处裸行号，对上 56 处，没判 18 处（其中裸行号没有引文可比 18 处；逐处列在上面）
```

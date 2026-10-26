# m3-close-c506-r1 攻方腿（Opus）报告（2026-10-22）

立场：假设「把 D22（单元原子性怎么合成） 已定项 2 里 S 的上界改成格式常量 16」是错的，逐格找反例。方案按 `crates/` 今天的实现谈；引 kb 写 kb 文件自己的行号。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-c506-r1-opus-model/`。冻结副本是这一腿开工时从主工作区拷的那一份（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`，不带 target），放在 `~/.local/share/singlefs/work/m3-close-c506-r1-opus/tree`；拷时与主工作区下列六份逐字节相同（sha256 前 16 位两边相等）：`root_ring.rs`、`make_filesystem.rs`、`recovery.rs`、checker 的 `image.rs`、format 的 `lib.rs`、`root_record.rs`。

```bash
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-c506-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c506-r1-opus/tree --work <新的草稿目录>
```

这一腿自己跑过一次（`--work ~/.local/share/singlefs/work/m3-close-c506-r1-opus/rerun1`），末行原样：

```text
name=rerun_compare same=1 lines=159 sha256=902fb7ad15ce07bc1ba00d3db1664a4e39df45b1fe9d5acd1dd02c4a1f793e24
```

```text
a38cee11962075a60185282ed9f8982f67aec85b0033a3ef2c552dc10df3cc24  research/prompts/m3-close-c506-r1-opus-model/Cargo.toml
902fb7ad15ce07bc1ba00d3db1664a4e39df45b1fe9d5acd1dd02c4a1f793e24  research/prompts/m3-close-c506-r1-opus-model/out/probe.out
71907dc436a2902fc92db7064f2aa1960e24fe4a07cbf2f077e869e832e42ccd  research/prompts/m3-close-c506-r1-opus-model/src/main.rs
0c7bfe595aa401a404c65817b9f5071c0c0a03fc4c6d7603a3d3675750fbe6a3  research/prompts/m3-close-c506-r1-opus-model/tools/rerun.sh
```

原型只 `cargo build`、经内存包装直接执行编出来的二进制；盘是内存稀疏盘（`SparseBlockDevice`，4 GiB × 2）；不调崩溃枚举：这一腿造的历史不含崩溃点（打中的那一格在不崩、不注故障的合法历史上就红），崩溃状态数 0，离约 10⁶ / 10⁷ 两条线都远。整份跑一次 3.5 秒（`time` 量的），没缩任何取样。

## 各格判定一览

| 格 | 判定 | 一句话 | 复现 |
|---|---|---|---|
| V1 | 打中两处（都是「依据撑不撑得住 16」，不分辨「格式常量 / 挂载时现算」两臂） | ① D22 依据段拿 E50 撑「4..16 不碰边」，而 E50 的上界是按槽总数 N = R × S 量的：R = 3 时散开档在 S ≥ 10 就越过 E50 自己跑前定死的 1000 块线，S = 16 是 1632 块；② E139 入库产物里，一块盘持续写不进根、占用 0.70 时 S = 16 八个种子全卡死、S = 4 同格 0（射程外，引产物，没重跑） | 算式见 V1 节；产物行整行抄在 V1 节 |
| V2 | 打中一处，另记一处前提不实 | 16 不只是系统配置那一字节的读者区间：根记录的钉槽 p（1 字节，钉值 = 3·槽 + 区域，255 是哨兵）靠 S ≤ 85 才装得下，而根记录在冻结层登记表里是「没有条款」；D15（格式冻结政策） 已定项 9 的判定表没有「放宽一个字段的取值域」这一行，抬 16 要不要 incompat 判不出。正文「今天下界已有 format-const 标记」只是 Rust 文档注释，kb 里没有登记 | V2 节 |
| V3 | 打中（副本上量过） | S = 16、槽距 512 KiB：core 择系统配置与 checker `geometry_of` 都收，盘 0 上区域 0 的槽 12..15 与区域 2 的槽 0..3 是同一段字节；只暖机挂载 24 次，24 / 24 个不崩、不注故障的合法状态上池级 checker 都判 I-7.17（根槽位置等于 txg 算出的位置） 红，四条对照臂 96 / 96 个状态 0 红。另：入库 mkfs 按「区域长 ≤ 3 MiB、不分盘」判重叠，比 0 / 1 / 0 归属下盘上真正要的「同盘区域不相叠」（≤ 6 MiB）严：S = 8（mkfs 写的那个）在 io_min = 512 KiB 上被拒，S = 16 在 io_min = 256 KiB 上被拒，而这两格改字节造出来的池各挂 24 次 0 红 | `out/probe.out` 的 `name=mkfs_geometry`、`name=reader_accepts`、`name=history` 各行 |
| V4 | 打中（条文） | 候选句只把上界换成 16，下面四句仍与它或别的已定项打架：「4..16 之间取值不碰任何一条边」在槽距 > 192 KiB 时不成立；已定项 16 第 3 句的 mkfs 校验只判盘长，不判区域相叠，而读者两侧都不判；已定项 25「格式常量把一个纯可用性参数焊死」与「环深上界进格式」要有一句交代；依据段 E50 那一行撑不住 16 | V4 节给逐字候选句（只在我的模型上量过、被攻过零轮） |

## V1：原条款防的是什么；S ≤ 16 而那件事仍发生的池与历史

### 原条款那一句防的是什么

- 被改的那一句：「上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算」（`.claude/kb/decisions/22-单元原子性怎么合成.md:60`）。它的量就是 E50 的判据 2：「上界：I-7.4（近 K 代块未被复用） 扣住的块数开始要紧的那个 N」（`.claude/kb/experiments/50-根环槽数的上下界.md:26`），扣住的块数按「I-7.4（近 K 代块未被复用） 扣住的块数 = N × 每次 fsync 写的块数」（`.claude/kb/experiments/50-根环槽数的上下界.md:83`）算。也就是：环里每一条根钉住它引用的块，环越深钉得越多，池小时钉住的块吃掉可分配空间。
- 2026-09-13 之后「K 代」改指回退候选集：「盘紧时抬 F 缩到最近 4 个可退到的状态」（`crates/singlefs-checker/invariants/02-根环.md:11`），D16 写成「平时不动，窗口就是整个根环；准入不够时抬」（`.claude/kb/decisions/16-发布语义.md:37`）。所以平时钉住的是整个环（3S 次发布的释放），盘紧时靠抬 F 把它缩到与 S 无关的 4 个状态。原条款要防的事在今天只剩一种形状：**F 抬不动、而环还压着回收门槛**的那几类历史。D16 自己点名了其中两类：「读坏槽并进上限、卡住就在这次挂载里报 ENOSPC、不推空发布」（`.claude/kb/decisions/16-发布语义.md:38`），以及「一块盘持续写不进根时扣住的单元随发布线性涨、没有上界」（`.claude/kb/decisions/16-发布语义.md:62`）。这两类里钉住多久按环长 3S 计，S 越大钉得越久。

### 打中一：依据段把 E50 的槽总数 N 读成了每区槽数 S

D22 依据段那一行：「它量出的 S 下界与上界都在 4..16 之外，S 不是一个要精算的量」（`.claude/kb/decisions/22-单元原子性怎么合成.md:83`）。E50 扫的是槽总数：「const TOTAL_SLOT_COUNTS_SCANNED: [u64; 8] = [2, 4, 8, 16, 32, 64, 256, 1024];」（`scripts/experiment-apparatus/e50_ring_slots.rs:46`），表头写「首个够到 1000 块这条线的 N」（`.claude/kb/experiments/50-根环槽数的上下界.md:89`），散开档那一格是 32。E50 结论句却写「S 在 1..16 这个区间里怎么取都不碰任何一条边」（`.claude/kb/experiments/50-根环槽数的上下界.md:118`）。R = 3 时 N = 3S，按 E50 源码的两档每次写块数（「const COARSE_BLOCKS: u64 = 8 + 3 + 1 + 1;」（`scripts/experiment-apparatus/e50_ring_slots.rs:43`），「const SCATTERED_BLOCKS: u64 = 8 + 8 * 3 + 1 + 1;」（`scripts/experiment-apparatus/e50_ring_slots.rs:44`））现算（命令与原样输出）：

```text
$ python3 -c "for s in (4,8,10,11,16): n=3*s; print(f'S={s} N=R*S={n} coarse_13_per_publish={13*n} scattered_34_per_publish={34*n} scattered_over_1000={34*n>1000}')"
S=4 N=R*S=12 coarse_13_per_publish=156 scattered_34_per_publish=408 scattered_over_1000=False
S=8 N=R*S=24 coarse_13_per_publish=312 scattered_34_per_publish=816 scattered_over_1000=False
S=10 N=R*S=30 coarse_13_per_publish=390 scattered_34_per_publish=1020 scattered_over_1000=True
S=11 N=R*S=33 coarse_13_per_publish=429 scattered_34_per_publish=1122 scattered_over_1000=True
S=16 N=R*S=48 coarse_13_per_publish=624 scattered_34_per_publish=1632 scattered_over_1000=True
```

按 E50 自己跑前定死的线与它自己的散开档，S ∈ [10, 16] 已越过那条线；E50 页也写了「散开负载下 32 槽就够到了」（`.claude/kb/experiments/50-根环槽数的上下界.md:92`）。所以「E50 量出 4..16 不碰边」不成立，只在粗粒度档成立（S = 16 是 624 块）。

四句：① 分不分辨臂：分辨「16 有实验撑」与「16 没有实验撑」，不分辨「格式常量」与「挂载时现算」（两臂都引同一行依据）。② 被判的系统看不看得到判别它的东西：负载形态（粗粒度还是散开）挂载时看不到，这正是 E50「上界对 D25（目标负载优先级） 的负载模型敏感」那一句的意思，常量与现算都拿不到它。③ 满足判据字面哪一分句：E50 判据 2「扣住的块数开始要紧的那个 N」，线是 1000 块。④ 改法在这几格上还中不中：只把那一句改成「上界 16 是格式常量」不碰依据段，这一格照中；依据段改成「粗粒度档 S ≤ 25 不过线（3 × 25 × 13 = 975，S = 26 是 1014）、散开档 S ≥ 10 过线，16 取的是读者承诺，不是 E50 量出的上界」才不中（推的）。

### 打中二：一块盘持续写不进根时，S = 16 卡死而 S = 4 不卡（入库产物，没重跑）

E139 第 2 次重跑的入库产物 `research/results/e139-tightened-floor-2026-10-05.out`，臂 RGall（今天条款取的那条：回收门槛与另外五处退回环里最旧有效根），世界 `disk_stuck`（卡住盘 1、4000 块的池、占用 0.70、收尾 crash），两行原样：

```text
E7RESULT name=f_disk_stuck arm=RGall_ring_all s=4 fill=0.70 ending=crash seeds=8 fakes=0 unrecoverable=0 false_enospc_arm=0 true_enospc=0 write_after_delete_failed=0 stalls=0 injected=128
E7RESULT name=f_disk_stuck arm=RGall_ring_all s=16 fill=0.70 ending=crash seeds=8 fakes=0 unrecoverable=0 false_enospc_arm=32 true_enospc=0 write_after_delete_failed=32 stalls=8 injected=296
```

（分别在那份产物第 5115、5259 行；另两种收尾 `lost_disk_0`、`lost_disk_1` 在 S = 16、0.70 上同为 `stalls=8`，S = 4 同为 `stalls=0`。）实验页把它记成射程外：「`disk_stuck` S = 16、占用 0.70、卡住盘 1 的三行：RGall、RGall_Q、G2SW_ringgate 各 8 个种子全卡死」（`.claude/kb/experiments/139-按盘回退下界的收严形态.md:324`）。这就是原条款要防的那件事的样子：环压着回收门槛、F 抬不过卡住那块盘最新的根，钉住的块按环长涨，4000 块、七成满的池在 S = 16 上卡死，在 S = 4 上没卡。

四句：① 分不分辨臂：分辨 S = 4 与 S = 16，不分辨「格式常量 16」与「挂载时现算」——现算的上界在挂载那一刻看不到「之后会有一块盘一直写不进根」，也看不到占用以后涨到多少；S 又是系统不可变配置，mkfs 之后改不了。② 系统看不看得到：看不到（同上）。③ 满足判据字面哪一分句：D16（发布语义） 已定项 1 射程「一块盘持续写不进根时扣住的单元随发布线性涨、没有上界」那一句；k_tol = 2 的承诺之外（射程外），所以不是条款违反，是「原条款想用上界防、新条款不防」的一个具体池与历史。④ 改法：常量 16 在这一格上照中；把常量取小（例如 8）只把卡死往后推（S = 4 在刻度 4 之后照样「没有上界」，同一页第 355 行），也不中。⚠️ S = 8（mkfs 写的那个）E139 没取样，它卡不卡没有数。

### V1 没打中的

- 平时（F 抬得动）：钉住的块随 F 缩到 4 个状态，与 S 无关；S = 16 在 E139 判的格里假候选、不可恢复、判的格假性 ENOSPC 都是 0（实验页 `w_cell` 的取样是「S ∈ {1, 4, 16}」，同一页第 155 行）。我没找到 F 抬得动、k_tol 之内、S ≤ 16 而钉住的块挤爆池的历史。
- 挂载时现算那一臂本身算不出来：lib.rs 的 placeholder 注释「在它算得出来之前按 16 收着」（`crates/singlefs-format/src/lib.rs:304`）说的属实——I-7.4 的「K 代」是按 F 与实例表现算的集合，不是池参数的函数。所以「改成常量会丢掉原条款的保护」只在「原条款本来就兑现不了」的意义上成立。

## V2：16 落哪一层、改它要不要 incompat 位、两条边登没登记

### 打中：16 托着的不只是系统配置那一字节，还有根记录的钉槽 p

- 系统配置槽在冻结层登记表里是「| 系统配置槽（码 18 自证单元，每盘两槽轮换，字段表 489 字节） | 第 1 层 |」（`.claude/kb/freeze-layer-membership.md:19`），S 那一字节在里面，读者的区间判按 D15「对已冻结部分，checker 的判断变成事实上的兼容性契约」（`.claude/kb/decisions/15-格式冻结政策.md:175`）跟着它走——把两条边登在第 1 层，到这里为止是对的。
- 但根记录的钉槽 p 也靠这个上界：「钉值 = 3·槽 + 区域，就是那一槽可能住过的最旧 txg（小于环长 3S）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:150`），盘上 1 字节、255 是「没有钉着」；实现里写死的是「小于环长 3S ≤ 48：装得进一个字节」（`crates/singlefs-core/src/root_record.rs:93`），`root_ring.rs` 也写「上界 16 保证它装得进 `u8`」（`crates/singlefs-core/src/root_ring.rs:85`）。钉值最大是 3(S − 1) + 2 = 3S − 1，要不撞 255 就得 S ≤ 85（`python3 -c "print(max(s for s in range(1,256) if 3*(s-1)+2<=254))"` 原样输出 `85`）。而根记录在登记表里是「| 根记录（码 16 自证单元，466 字节，住根环槽） | 没有条款 |」（`.claude/kb/freeze-layer-membership.md:59`）。
- 所以「16 落第 1 层」只说了一半：上界一旦抬过 85，要连根记录的 p 一起改宽，按 D15 判定表是「| 扩大已有字段宽度 | 按旧宽度解析，越界读 / 错位 | incompat | 是 |」（`.claude/kb/decisions/15-格式冻结政策.md:191`），而那个结构的层没有条款。旧条款（挂载时现算、一字节最多 255）在 S ≥ 86 时会直接撞上 p 的编码，换成常量 16 反而把这一处扣住了——这一点是常量那一臂的好处，正推若没写，判决时可以记上。

四句：① 分不分辨臂：分辨（现算那一臂允许 S 到 255，在 S ≥ 86 上撞 p；常量 16 不撞）。② 系统看不看得到：看得到（S 与 p 都在盘上）。③ 满足哪一分句：D22 已定项 7 钉槽 p 那一格「小于环长 3S」与 255 哨兵。④ 改法：候选句里加一句「上界不超过 85：根记录钉槽 p 一字节、钉值 3S − 1 ≤ 254」，这一格不中（推的）。

### 打中：D15 判定表没有「放宽读者收的取值域」这一行

D15 已定项 9 的判据是「判据是旧读者会怎样」（`.claude/kb/decisions/15-格式冻结政策.md:21`），九行里没有一行是「字段宽度不变、读者收的取值域变宽」。抬 16（例如到 32）时旧读者按区间整池拒（`crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:209` 那条用例钉的 S = 17 整池拒）——效果与 incompat 相同，却不经 incompat 位。两条路的拒法不同：区间越界是整池拒，incompat 位不认得是「incompat 位不认得的槽不可择、不核偏移」（`.claude/kb/decisions/22-单元原子性怎么合成.md:346`）。抬上界要不要同时置一个 incompat 位、旧读者走哪一条拒法，判定表判不出，候选句要自己写一句。降 16 则是新读者拒收旧池，D15 已定项 8「已冻结字段的宽度 / 存在性 / 含义不能再改；只能新增」（`.claude/kb/decisions/15-格式冻结政策.md:171`）冻结后不许——V3 打中的几何冲突因此必须在冻结前用别的办法收（见 V3、V4），不能留到冻结后靠降 16 收。

四句：① 分不分辨臂：分辨（现算那一臂没有一个可以登记的数，谈不上抬降；常量那一臂才有这个问题）。② 看不看得到：看得到。③ 分句：D15 已定项 9 的「判据是旧读者会怎样」与九行枚举不全。④ 改法：候选句里写「抬上界照 incompat 处置（新置一位），旧读者照区间整池拒」或「抬上界不置位」二选一，交用户；两者在这一格上都不中（推的）。

### 前提不实：「今天下界已有 format-const 标记」

正文第二节写下界「带 `format-const` 标记」。那是 Rust 文档注释「/// format-const: ROOT_RING_SLOTS_PER_REGION_MINIMUM」（`crates/singlefs-format/src/lib.rs:299`），不带值，门禁读的文法是「<!-- format-const: 名字 = 整数 stale=旧串|旧串 -->」（`.claude/gate.d/lib-format-const.py:11`），而 kb 里一处都没有（`grep -rn 'format-const: ROOT_RING_SLOTS_PER_REGION' .claude/kb | wc -l` 原样输出 `0`）。layout 字段表 S 那一行是「| 几何 | 每区槽数 S | 1 | 8 |」（`.claude/kb/layout/01-first-txn.md:140`），没有标记。所以两条边今天都没登记，与 C506 ① 那一句相符；正文那半句只说对了 Rust 侧。

## V3：16 与根环几何的其余约束

### 约束在哪、谁判

- 区域间距：区域 r 起点 = 基址 + r × P × chunk = 1 / 4 / 7 MiB，区域长 = S × 槽距；实现自己的注释是「区域之间隔着 P × chunk，这一段之外的字节不归根环。」（`crates/singlefs-core/src/root_ring.rs:137`）。第一版归属 0 / 1 / 0：区域 0 与区域 2 同在盘 0、相距 6 MiB；journal 环从 16 MiB 起。
- 槽距的上界只按系统配置槽 1 定：「固定结构槽距 ∈ [4096, 1 MiB − 4096]」（`.claude/kb/decisions/22-单元原子性怎么合成.md:74`），与 S 无关。
- 条款要 mkfs 判的只有盘长：「mkfs 要校验 基址 + (R−1) × P × chunk + S × 槽距 ≤ 设备可用字节，越界拒绝 mkfs」（`.claude/kb/decisions/22-单元原子性怎么合成.md:350`）。kb 里 D22、D2、D23 与 layout 都没有一句要求区域之间、区域与 journal 环之间不相叠（`grep -nE '重叠|相叠|叠在一起'` 在这四份里只命中 D2 一处讲写小于 io_min 的射程）。
- 入库 mkfs 比条款多判一样：六段固定结构两两不相叠，不分盘（`crates/singlefs-core/src/make_filesystem.rs:533` 那一处 `return Err(MakeFilesystemError::FixedStructuresOverlap {`）。
- 读者两侧都只判 S 的区间与槽距自己的区间：core 的 `system_configuration_values_this_reader_accepts`（`crates/singlefs-core/src/recovery.rs:1095` 起）与区间判「if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)」（`crates/singlefs-core/src/root_ring.rs:65`）；checker 的 `geometry_of`（`crates/singlefs-checker/src/image.rs:718` 起）只多判一条根槽偏移装不装得进 u64（「let last_root_slot_offset = base_slot.checked_mul(SLOT_BYTES)」（`crates/singlefs-checker/src/image.rs:778`））。S × 槽距对区域间距、对 journal 环起点，两个读者都不判。

每个槽距下 S 最多取几（命令与原样输出）：

```text
$ python3 -c "
for d in (131072,196608,262144,393216,524288,1044480): print(f'spacing={d} largest_S_region_within_3MiB={3*2**20//d} largest_S_regions_0_and_2_apart_on_device0={6*2**20//d} largest_S_region2_before_journal_16MiB={9*2**20//d}')"
spacing=131072 largest_S_region_within_3MiB=24 largest_S_regions_0_and_2_apart_on_device0=48 largest_S_region2_before_journal_16MiB=72
spacing=196608 largest_S_region_within_3MiB=16 largest_S_regions_0_and_2_apart_on_device0=32 largest_S_region2_before_journal_16MiB=48
spacing=262144 largest_S_region_within_3MiB=12 largest_S_regions_0_and_2_apart_on_device0=24 largest_S_region2_before_journal_16MiB=36
spacing=393216 largest_S_region_within_3MiB=8 largest_S_regions_0_and_2_apart_on_device0=16 largest_S_region2_before_journal_16MiB=24
spacing=524288 largest_S_region_within_3MiB=6 largest_S_regions_0_and_2_apart_on_device0=12 largest_S_region2_before_journal_16MiB=18
spacing=1044480 largest_S_region_within_3MiB=3 largest_S_regions_0_and_2_apart_on_device0=6 largest_S_region2_before_journal_16MiB=9
```

S = 16 要 S × 槽距 ≤ 3 MiB（mkfs 的判法）只到槽距 192 KiB；要同盘区域不相叠只到 384 KiB；槽距取到条款上界 1044480 时连下界 S = 4 都放不进 3 MiB，S > 9 的区域 2 伸进 journal 环。

### 打中（副本上量过）：S = 16、槽距 512 KiB 两个读者都收，盘 0 上环自己叠在自己身上

入库 mkfs 在 S × io_min 上逐格（`out/probe.out` 的 30 行 `name=mkfs_geometry`，摘六行原样）：

```text
name=mkfs_geometry S=12 io_min=262144 spacing=262144 region_bytes=3145728 region_bytes_over_region_gap_3MiB=false region0_meets_region2_on_device0=false region2_end_past_journal_start_16MiB=false mkfs=accepted
name=mkfs_geometry S=16 io_min=262144 spacing=262144 region_bytes=4194304 region_bytes_over_region_gap_3MiB=true region0_meets_region2_on_device0=false region2_end_past_journal_start_16MiB=false mkfs=refused:FixedStructuresOverlap[RootRingRegion0/RootRingRegion1]
name=mkfs_geometry S=4 io_min=524288 spacing=524288 region_bytes=2097152 region_bytes_over_region_gap_3MiB=false region0_meets_region2_on_device0=false region2_end_past_journal_start_16MiB=false mkfs=accepted
name=mkfs_geometry S=6 io_min=524288 spacing=524288 region_bytes=3145728 region_bytes_over_region_gap_3MiB=false region0_meets_region2_on_device0=false region2_end_past_journal_start_16MiB=false mkfs=accepted
name=mkfs_geometry S=8 io_min=524288 spacing=524288 region_bytes=4194304 region_bytes_over_region_gap_3MiB=true region0_meets_region2_on_device0=false region2_end_past_journal_start_16MiB=false mkfs=refused:FixedStructuresOverlap[RootRingRegion0/RootRingRegion1]
name=mkfs_geometry S=16 io_min=524288 spacing=524288 region_bytes=8388608 region_bytes_over_region_gap_3MiB=true region0_meets_region2_on_device0=true region2_end_past_journal_start_16MiB=false mkfs=refused:FixedStructuresOverlap[RootRingRegion0/RootRingRegion1]
```

造法：mkfs 收的那一格（S = 4、io_min = 512 KiB）建池，把四个系统配置槽里 S 那一字节（偏移 362）改成 16、整槽校验和重封。这一改等于「一个只照条款判盘长、照写 S 的 mkfs」写出的盘：在 mkfs 两边都收的三对几何上逐扇区比过，0 处不同：

```text
name=same_bytes rewritten_from_S=4 to_S=5 io_min=524288 nonzero_sectors_rewritten=11 nonzero_sectors_direct=11 differing_sectors=0
name=same_bytes rewritten_from_S=4 to_S=16 io_min=4096 nonzero_sectors_rewritten=11 nonzero_sectors_direct=11 differing_sectors=0
name=same_bytes rewritten_from_S=4 to_S=12 io_min=131072 nonzero_sectors_rewritten=11 nonzero_sectors_direct=11 differing_sectors=0
```

（推到 S = 16、512 KiB 这一格是外推：那一格入库 mkfs 拒，没有直接写出来的盘可比；空盘上 mkfs 多清的那几段本来就是 0，推的。）两个读者都收：

```text
name=reader_accepts S=16 io_min=524288 spacing=524288 core_choose_system_configuration=accepted(S=16,spacing=524288) checker_geometry_of=accepted(S=16,spacing=524288)
name=history_geometry arm=S16_spacing512KiB_rewritten S=16 spacing=524288 aliased_slot_pairs=["d0@7340032:r0s12=r2s0", "d0@7864320:r0s13=r2s1", "d0@8388608:r0s14=r2s2", "d0@8912896:r0s15=r2s3"]
```

历史：同一份池接连只暖机可写挂载（不写文件、不崩、不注故障），每次挂完读一遍根环、跑一遍池级 checker。挂几次是用户动作，扫 1..24 次；第一处红在第 1 次挂载之后就出现（暖机那次 txg 2 写区域 2 槽 0，也就是盘 0 上区域 0 槽 12 那段字节），不靠后面的用户动作。四行原样：

```text
name=history arm=S16_spacing512KiB_rewritten mount=1 mount_result=ok chosen_root_txg=2 misplaced_roots=["r0s12:txg2"] red=["I-7.17<实例 1、txg 2 的根住在区域 0 槽 12：按 txg 算该在区域 Some(2)（第 0 代不判区域）槽 0>"]
name=history arm=S16_spacing512KiB_rewritten mount=13 mount_result=ok chosen_root_txg=37 misplaced_roots=["r0s13:txg5", "r0s14:txg8", "r0s15:txg11", "r2s0:txg36"] red=["I-7.17<实例 3、txg 5 的根住在区域 0 槽 13：按 txg 算该在区域 Some(2)（第 0 代不判区域）槽 1>"]
name=history arm=S16_spacing512KiB_rewritten mount=16 mount_result=ok chosen_root_txg=46 misplaced_roots=["r2s0:txg36", "r2s1:txg39", "r2s2:txg42", "r2s3:txg45"] red=["I-7.17<实例 13、txg 36 的根住在区域 2 槽 0：按 txg 算该在区域 Some(0)（第 0 代不判区域）槽 12>"]
name=history arm=S16_spacing512KiB_rewritten mount=24 mount_result=ok chosen_root_txg=70 misplaced_roots=["r0s12:txg50", "r0s13:txg53", "r0s14:txg56", "r0s15:txg59"] red=["I-7.17<实例 18、txg 50 的根住在区域 0 槽 12：按 txg 算该在区域 Some(2)（第 0 代不判区域）槽 0>"]
```

计数（在模型目录里跑，原样输出依次是 24、96、120）：`grep 'name=history arm=S16_spacing512KiB_rewritten' out/probe.out | grep -c 'red=\["I-7.17'`；`grep 'name=history ' out/probe.out | grep -v 'arm=S16_spacing512KiB_rewritten' | grep -c 'misplaced_roots=\[\] red=\[\]'`（四条对照臂 S = 16 / 4 KiB、S = 4 / 512 KiB、S = 16 / 256 KiB、S = 8 / 512 KiB 各 24 次，全干净）；`grep 'name=history ' out/probe.out | grep -c 'mount_result=ok'`（五条臂 120 次挂载 core 全挂上）。I-7.17 的定义是「根槽位置等于 txg 算出的位置」（`crates/singlefs-checker/invariants/02-根环.md:24`）。

读法：池级 checker 在 24 / 24 个合法状态上红；core 照常挂上，把读到别处根的那一槽当「这一槽这一次读坏」处置（`root_sits_in_the_ring_slot_the_formula_gives` 的文档注释），那一槽因此钉回收门槛、并进抬 F 的上限——这一半的空间后果我没量。环的实际容量是 44 个物理槽，不是条款与「挂载时读不出的槽」那一句（D16 已定项 1）按 3S = 48 算的那个数。

四句：① 分不分辨臂：不分辨「格式常量 16」与「挂载时现算」（现算那一臂按 I-7.4 算上界，也不看槽距）；分辨「只判 S 区间」与「另判 S × 槽距」。② 系统看不看得到：看得到——S 与槽距都在同一槽系统配置里，读者择槽时就能判。③ 满足判据字面哪一分句：I-7.17 「r == txg mod R 且 s == (txg div R) mod S」那一分句（同一行）。④ 改法：只改那一句成「上界 16 是格式常量」，这一格照中；加「S × 槽距 ≤ P × chunk」或「同盘区域不相叠、区域末端不越过 journal 环起点」读者两侧整池拒，这一格不中（推的，没在副本上实现）。

可达性的限定：入库 mkfs 写不出这一盘（它判不相叠）；写得出它的是一个只照 D22 已定项 16 第 3 句判盘长的写者——而「上界 16 是格式常量」恰恰是对任何写者的承诺。

### 顺带打中：mkfs 的不相叠判法比盘上要的严，mkfs 写的 8 服务不了 io_min = 512 KiB

`fixed_structure_extents` 不分盘两两比，区域 0 与区域 1 分在两块盘也算相叠。上表 S = 8、io_min = 512 KiB 与 S = 16、io_min = 256 KiB 两格 mkfs 拒；而同样改字节造出的这两份池各挂 24 次 0 处错位、0 红（`name=history_geometry ... aliased_slot_pairs=[]` 两行与上面那条 96 的计数）。所以「S × 槽距 ≤ 3 MiB」是 mkfs 自己选的、比 0 / 1 / 0 归属下盘上要的（≤ 6 MiB、区域 2 不越过 16 MiB）严一倍的线；条款两条线都没写。io_min = 512 KiB 是 md 默认 chunk（外部事实，未在本项目验证），在那种盘上 mkfs 写 8 当场拒。这一格分辨「mkfs 不分盘判」与「按归属分盘判」，与 16 本身无关，记一笔交主 agent。

### 系统配置那一字节与根记录 p

- 那一字节取值域 0..255；读者只收 4..16，越界整池拒（区间判已有用例，见 V2）。没打中。
- 钉槽 p 要 S ≤ 85（V2 节），16 在里面。没打中，常量那一臂在这里反而比现算那一臂稳。
- mkfs 的 8：编译期断言「4..16 之间取值不碰任何一条边」只判 4 < 8 < 16（`RootRingSlotsPerRegion::at_make_filesystem_checked_at_compile_time`），不判槽距；8 在槽距 ≤ 384 KiB 时 mkfs 收，512 KiB 时拒（上表）。

### 区域 chunk（1 MiB）那一问

题面问 R × S 个根槽能不能落进一个区域 chunk。条款里没有「一个区域装进一个 chunk」的要求（区域间距是 P × chunk = 3 MiB）；装进 1 MiB 要 S × 槽距 ≤ 1 MiB，S = 16 时槽距 ≤ 64 KiB。我没找到哪条已定项依赖「区域在一个 chunk 内」：素数步长那条理由已随归属改成 mkfs 逐区域写设备身份而作废（D22 已定项 2 射程第二条、C217）。没打中。

## V4：候选句与别的已定项打架的地方

### 只把那一句换成「上界 16 是格式常量」时，同一格与邻格仍打架的四处

1. 同一格后半句「4..16 之间取值不碰任何一条边」（`.claude/kb/decisions/22-单元原子性怎么合成.md:60`）：槽距 > 192 KiB 时 S = 16 碰 mkfs 的不相叠线，> 384 KiB 时碰盘上真相叠，> 768 KiB 时 S = 4 也碰（V3）。实现把这半句搬成了 mkfs 编译期断言与 `RootRingSlotsPerRegion` 的文档注释（`crates/singlefs-core/src/root_ring.rs:43`），只判 S 不判槽距。
2. 已定项 16 第 3 句只要 mkfs 判盘长（`.claude/kb/decisions/22-单元原子性怎么合成.md:350`），读者两侧不判 S × 槽距；常量 16 与槽距上界 1 MiB − 4096 两条各自成立，合起来允许一个自叠的环（V3 实测）。
3. 已定项 25：「格式常量把一个纯可用性参数焊死，没有任何好处」（`.claude/kb/decisions/22-单元原子性怎么合成.md:495`）。那一句点名的是 K 与块复用的界；但同一项依据段写「恢复择根回退恰 2 代、管理员回退要整个根环深度」（`.claude/kb/decisions/22-单元原子性怎么合成.md:507`），环深上界 3 × 16 = 48 进格式，焊死的正是管理员回退最多能退的发布数这一个可用性参数。候选句要交代这一句为什么不撞已定项 25（我的读法：焊死的是读者承诺收的最大环深，块复用的界照旧现算；这是推的）。
4. 依据段 E50 那一行（`.claude/kb/decisions/22-单元原子性怎么合成.md:83`）撑不住 16（V1 打中一）。

另有两处不是打架、是销账要一并改的：`crates/singlefs-format/src/lib.rs:293` 与 `crates/singlefs-core/src/root_ring.rs:18` 把 S 那一格引成「D22 已定项 1 的字段表」，S 在已定项 2 的参数表里；lib.rs 的 placeholder 注释（`crates/singlefs-format/src/lib.rs:304` 那一段）要删。

### 逐字候选句（只在我的模型上量过、被攻过零轮）

D22 已定项 2 参数表「每区槽数 S」那一格：

> 住系统配置（偏移 362 那 1 字节）。**下界 k_tol + 2 = 4**（D16（发布语义） 已定项 1：**每区槽数 S** 要装得下 4 个状态加一次带 k_tol 次根槽写失败的处置——那一项是直接在 S 上量的，不是在 R × S 上；「环深」这个词在 C131（栈深与保护窗口的量纲） 出过一次事，这里不用它），**上界 16 是格式常量**：它是读者承诺收的最大值，不从 I-7.4（近 K 代块未被复用） 扣住的块数算——那个「K 代」是按 F 与实例表现算的回退候选集，平时钉住的块由抬 F 收到 4 个状态，与 S 无关。两条边随系统配置槽落 D15（格式冻结政策） 第 1 层，登记在 layout 一「每区槽数 S」那一行旁；上界另受根记录钉槽 p 的 1 字节约束（钉值最大 3S − 1，255 是哨兵 ⇒ S ≤ 85），抬过 85 要连 p 一起加宽。**同一槽另判 S × 固定结构槽距 ≤ P × chunk**（区域不越过下一个区域的起点，第一版 3 MiB；第一版 0 / 1 / 0 归属下这条比盘上真相叠严一倍，取它是为了不随归属与盘数变），越界与 S 越界同一个处置：读者整池拒、mkfs 拒；所以槽距 > 192 KiB 时 S 到不了 16，> 768 KiB 时没有可取的 S。⚠️ **具体取多少不是格式决策**——格式承诺的是「S 是系统配置字段 + 挂载时判区间与 S × 槽距」，取值由第一版实现按 D25（目标负载优先级） 的负载形态定；mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，小于 4 拒绝 mkfs。环深上界 3 × 16 进格式，焊死的是管理员回退最多能退的发布数，不是块复用的界（块复用的界照已定项 25 现算）。

依据段 E50 那一行换成：

> E50（根环槽数的上下界）：扫的是槽总数 N = R × S；R = 3 时粗粒度档（13 块/次）S ≤ 25 不过跑前定的 1000 块线，散开档（34 块/次）S ≥ 10 就过线 ⇒ 16 不是 E50 量出的上界，是读者承诺；E139（按盘回退下界的收严形态） 在 S ∈ {4, 16} 上判的格零假候选、零不可恢复，射程外 `disk_stuck`（占用 0.70）S = 16 卡死、S = 4 不卡。

### 各改法修哪一格

| 打中的格 | 只改那一句成「上界 16 是格式常量」 | 加 S × 槽距 ≤ P × chunk（读者两侧 + mkfs） | 依据段换句 | 加「S ≤ 85（钉槽 p）」一句 |
|---|---|---|---|---|
| V3 S = 16 / 512 KiB 环自叠、I-7.17 24 / 24 红 | 照中（量过：副本上就是今天的实现，`out/probe.out`） | 不中（推的：读者在择系统配置时整池拒，这一盘挂不上；没实现没跑） | 照中（推的） | 照中（推的） |
| V3 mkfs 写 8 服务不了 io_min = 512 KiB | 照中（量过：`name=mkfs_geometry S=8 io_min=524288` 那一行） | 不中（推的：mkfs 写 min(8, 6) = 6） | 照中 | 照中 |
| V1 依据段读错 E50 | 照中（推的） | 照中 | 不中（推的） | 照中 |
| V1 `disk_stuck` S = 16 卡死 | 照中（引产物，没重跑） | 照中（推的：512 KiB 以下槽距 S 仍可取 16） | 照中（只是写明） | 照中 |
| V2 钉槽 p 与 S 上界 | 不打架（16 < 85；推的） | 无关 | 无关 | 写明（推的） |
| V2 判定表缺「放宽取值域」一行 | 照中（推的） | 照中 | 照中 | 照中 |

V1 的 `disk_stuck` 那一格没有哪个改法修得了：它是回收门槛取环里最旧有效根（RGall）在射程外的性质，S 只改它卡得早晚，交主 agent 判要不要另立一账。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| F 抬得动、k_tol 之内，S ≤ 16 而钉住的块挤爆池 | 读 D16 已定项 1 全节与 E139 第 2 次重跑的判的格（S ∈ {1, 4, 16}） | 没找到：抬 F 把候选集收到 4 个状态，与 S 无关；没自己跑计数模型 |
| S 区间在 mkfs 收的几何上被读者错拒或错收 | mkfs 30 格（S ∈ {4, 5, 6, 8, 12, 16} × io_min ∈ {4 KiB, 64 KiB, 128 KiB, 256 KiB, 512 KiB}） | 挂载历史只在 S = 16 / 4 KiB、S = 4 / 512 KiB 两格 mkfs 收的几何上跑（各 24 次 0 红），其余 mkfs 收的格只判了收拒 |
| mkfs 拒、盘上并不相叠的几何在读者侧出事 | S = 16 / 256 KiB、S = 8 / 512 KiB，各挂 24 次 | 0 处错位、0 红 |
| 系统配置那一字节的取值域与 16 打架 | 0..255 对 4..16 | 没打中 |
| 钉槽 p 一字节与 16 打架 | 钉值 3S − 1 ≤ 254 | 没打中（S ≤ 85） |
| 区域装进一个 chunk 的要求 | 读 D22 已定项 2、16 与 C217 | 条款里没有这个要求，没打中 |
| format-const 门禁登记之后别处的 const 副本值不同 | `grep -rnE 'const ROOT_RING_SLOTS_PER_REGION_(MINIMUM\|MAXIMUM)'` 在 research、scripts、crates 的 `.rs` 里 4 处，值全是 4 / 16 | 没打中 |
| 区域 2 伸进 journal 环 | 只算了算式（S = 16 要槽距 > 576 KiB；入库 mkfs 的对齐判法让槽距只能是 2 的幂，512 KiB 之上下一档 1 MiB 已越槽距上界） | 只在读者侧可达（读者不判对齐那一半我没核），没造盘 |

## 这条腿自己的限度

- V3 的盘不是入库 mkfs 写的：改字节造出来，等价性只在三对 mkfs 两边都收的几何上逐扇区比过，推到 S = 16 / 512 KiB 是外推（推的）。写得出这一盘的是「只照条款判盘长」的写者，不是今天的 mkfs。
- core 把错位那一槽当读坏之后的空间后果（钉回收门槛、并进抬 F 上限、长期扣住多少）没量；只量了池级 checker 判红与 core 照常挂上。
- V1 打中二引的是入库产物 `research/results/e139-tightened-floor-2026-10-05.out` 的行，没重跑 E139；S = 8 那一档 E139 没取样。
- V1 打中一是对 E50 页与源码常量的算术，没重跑 E50。
- 历史只扫了「只暖机挂载 1..24 次」一种用户动作；没写文件、没删、没崩、没注故障。打中的红在第 1 次挂载之后就出现，不靠后面的动作；但「写文件、删文件」这些动作下错位槽对回收的影响没扫。
- 候选句与改法表里标「推的」的格都没实现没跑；候选句被攻过零轮。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没读禁读清单里的三份。
- 没跑重型测试、门禁阶段与 E50 / E139 的复跑。

## 要主 agent 跑的命令

无必须的。判决要引 V3 的数时，照开头「复跑命令」在新草稿目录复跑，看末行 `name=rerun_compare same=1`。

## 草稿目录里留着的

- `~/.local/share/singlefs/work/m3-close-c506-r1-opus/tree`：冻结副本（不带 target，37 MiB），复跑命令的 `--frozen` 指它，所以不删。
- 编译目录 `model/target` 与复跑目录 `rerun1` 交回前删掉。

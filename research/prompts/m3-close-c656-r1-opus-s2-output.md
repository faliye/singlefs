# m3-close-c656-r1 攻方腿第二次抽样（Opus）报告（2026-10-20）

这份是同一份提示的第二次抽样，没读第一条攻方腿的报告与模型。所有数都是草稿目录仓副本上量的（副本数，不是入库装置上的数）。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| V1 甲做不做得到 | **打中**：新池 fsid 与旧池相同时这一支做不到；fsid 不同时这条候选在量过的那份历史上站住，但要 mkfs 先读旧池 | 量过（x512-same、x512-diff 两份历史全量）；卡在哪一类字节是推的加量过 | 同 fsid：提交点（新系统配置）落了、根环还没清的 7 517 / 7 874 态择到新系统配置 + 旧池的根 (1, 6)，6 679 态可写挂载做成（新旧混合）；提交点之前 65 态旧池读回全对 |
| V2 乙撑不撑得住 | **没打中**（四份历史 124 690 态） | 量过 | 作废之前每份历史 8 态，旧池读回与 (1, 6) 逐字节相同 8 / 8；作废持久之后 0 态择到旧池、0 态混合；整池拒报 `NoValidSystemConfiguration`，拒的态上照今天写序再跑一次 mkfs 都恢复成第 0 代空池 |
| V2 附带（不分辨臂） | 作废之前一盘作废、另一盘没有时旧池只读挂得上、可写挂载拒；撕裂时择到较旧一槽（世代 7 而不是 8）| 量过 | 字面「读回逐字节相同」照样成立；甲提交点之前的部分落地同样出这两样 |
| V3 代价 | 给数（下表） | 段序列、状态数量过；屏障数、改处数是数出来的 | 乙：段序列 `2+12+1+1+1+4`、带撕裂全量 31 195；甲候选：`4+4+10+1+1+1`、7 874，外加读旧池三样 |
| V4 没量的几何 | 乙不翻面（io_min 不同 × fsid 同 / 不同都量了）；甲在 fsid 同时已经做不到，io_min 不同不救它（推的）；u 别的取值没量 | 乙量过；甲推的 | |
| 第三节的数能不能在入库代码上核 | **不能原样核**；状态数 280 019 与段形 `[14, 1, 1, 1, 4]` 我在另写的原型里独立对上了；248 832 那张拆分没重跑 | 量过（状态数）；其余推的 | 原数要改副本（把 `crash.rs` 里一个 `pub(crate)` 改成 `pub`、给 `recovery.rs` 插探针）才跑得出；正文把「崩溃镜像上 checker 红」写成「起点镜像 checker 红」 |
| 另：mkfs 在用过的盘上做完 checker 报 I-7.7 | 不分辨臂（今天、乙、甲三种写序做完都红），全新空盘对照绿 | 量过 | 比 C659 的字面（同 fsid、换槽距）宽：fsid 不同、槽距相同也红 |

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-close-c656-r1-opus-s2-model/`。冻结副本是开工时从主工作区拷出的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，放在 `~/.local/share/singlefs/work/m3-close-c656-r1-opus-s2/frozen/`（`crates` 下 `.rs` 全部文件 sha256 再取 sha256：`b523d85891b7327b57d1a760417241421b4749bd7891d00f019189826e375eb2`）。

```
bash scripts/hosts/run-with-memory-cap.sh 10G bash research/prompts/m3-close-c656-r1-opus-s2-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c656-r1-opus-s2/frozen \
  --work <新的空草稿目录>
```

`rerun_steps` 做的事：拷冻结副本 → `tools/patch_copy.py` 打两处补丁（`crash.rs` 的逐态展开函数改 `pub`；`make_filesystem.rs` 加 `make_filesystem_with_order`，三种写序：今天、乙、甲候选）→ 拷原型 crate、`tools/add_probe_member.py` 加进工作区 → `capped.sh 31 cargo build --release -j 8`（`-j 8` 只为让 librocksdb-sys 的 C++ 编译装进内存上限）→ 照 `tools/items.txt` 逐行跑、与 `out/` 比，打 `name=same` / `name=differs`。`tools/items.txt` 只列修了撕裂标签之后的二进制跑的两份；`out/v1-torn-label-bug/` 那几份怎么比写在 `items.txt` 的注释里。全新空盘对照另起：`<二进制> fresh`（产物 `out/fresh-control.log`）。只过了 `bash -n`，整条没复跑过。

各文件 sha256（`find . -type f | sort | xargs sha256sum`，前 16 位）：

| 文件 | sha256 前 16 位 |
|---|---|
| out/counts.log | 74eaabb470546b9c |
| out/fresh-control.log | 429638eff94f2611 |
| out/v1-torn-label-bug/x512-diff-jia-0-3937.log | 93c58dff26ce332b |
| out/v1-torn-label-bug/x512-diff-jia-3937-7874.log | 2f0f3f53423f8cb4 |
| out/v1-torn-label-bug/x512-diff-yi-0-12000.log | f321183649a8e573 |
| out/v1-torn-label-bug/x512-diff-yi-12000-31195.log | 8e116075b7358919 |
| out/v1-torn-label-bug/x512-same-jia-0-7874.log | 589fa733835e6233 |
| out/v1-torn-label-bug/x512-same-yi-0-31195.log | a6aa337b0829864a |
| out/v1-torn-label-bug/x65536-diff-yi-0-31150.log | b9d349e83033eb4a |
| out/v1-torn-label-bug/x65536-same-yi-0-6000.log | 63cdb8673ed3352f |
| out/v1-torn-label-bug/x65536-same-yi-18000-31150.log | 8da9abc116382908 |
| out/v1-torn-label-bug/x65536-same-yi-6000-18000.log | 3e265b1ff9d693cc |
| out/x512-diff-yi-0-8.log | 4185915bfae5192a |
| out/x512-same-jia-0-7874.log | c8d1cd204ad16f4c |
| probe/Cargo.toml | 1d6efbd7c6f3ee64 |
| probe/src/main.rs | 33e82c16805010e3 |
| tools/add_probe_member.py | aeaeddd532a91ab0 |
| tools/items.txt | a45491adcef84b56 |
| tools/patch_copy.py | db921a2965ffafb8 |
| tools/rerun.sh | 33697a679051d522 |
| tools/summarize.py | f267266f3a14e6b2 |

## 原型怎么取样（第 3b、3c 步）

- 盘是内存稀疏盘（`SparseBlockDevice`），旧池历史每份只跑一次，崩溃态从内存里那一份拷。历史：旧池 mkfs（今天的写序）+ 取号 + 暖机 + 第一个文件 + 3 次覆盖写（u = 5，最后确认的一版 (1, 6)），两块盘都重用；新池 io_min 512。`x512-*` 旧池 io_min 512，`x65536-*` 旧池 io_min 65536；`*-same` 新旧 fsid 相同，`*-diff` 不同。`leftover`：更早一个池 A（io_min 65536、fsid A、u = 5）→ 今天的 mkfs 做池 B（io_min 512、fsid B、u = 2）→ 新 mkfs（io_min 65536、fsid A）。
- 崩溃态照崩溃点重放的计划取：`visit_crash_state_range_of_the_read_back_flow_with_slices`，展开取 `full_expansion`（每段任意真子集、原地覆写带撕裂镜像），与 `enumerate_crash_states` 同一个计划、同一个展开；没走 `enumerate_crash_states` 本身，因为它要一个判的根与文件版本表、经判定库录入，mkfs 这条流没有文件版本（推的，没试）。每态：择系统配置、择根、恢复读回、池级 checker（`check_pool_image`）、只读挂载、按择到那份的参数可写挂载；整池拒的态另在拷贝上照今天写序再跑一次 mkfs，看恢复与 checker。
- 每次跑之前用 `crash_state_count_with_torn_in_place_overwrites` 算全量（`out/counts.log`，15 行）。跑完的态合计：乙 124 690 + 8 + 试跑 2 000，甲候选 7 874 × 2（x512-same 修标签前后各一遍）+ 7 874（x512-diff），总计约 15 万态，远低于 10⁶ / 10⁷。
- 缩了什么：先跑 2 000 态估时，39 态 / 秒（机器负载均值 97），全量 10 份「历史 × 写序」约 27.7 万态估 2 小时，超过 40 分钟，于是砍掉这几份历史：`leftover` 的乙（31 150 态）与甲（70 037 态）、`x65536-same` 与 `x65536-diff` 的甲（各 17 549 态）。缩前 10 份 27.7 万态，缩后 6 份 15.6 万态；留下的每份历史崩溃态一个不落。砍掉的四份的命令写在末尾「要主 agent 跑的命令」。
- 原型里有一处标签错，修过：第一版把撕裂镜像的判断写在录制流那张写表上（比计划的写表短），`torn=` 一列全写成 false；态的枚举与判定不受影响（镜像按计划的写表叠）。修后重跑了 x512-same 甲（判定逐类与修前相同）与 x512-diff 乙第 0 段 8 态，其余几份用修前的产物，只引判定列。

## V1　甲做不做得到

### 候选写序（`tools/patch_copy.py` 的 `OldPoolUntouchedUntilCommit`）

1. 两盘各写实例表单元第 0 片与树表单元第 0 版，落在旧池没写过的槽（盘末往回 64 槽、对齐 64，槽 2688；构造历史时逐字节核过两块盘上那 3 槽全 0，`out/counts.log` 里 `old_bytes_there_all_zero=true`）。根记录的两条指针照这个落点写（挂载按根里的指针认单元落点，`crates/singlefs-core/src/mount.rs:3476` 起那几行，不要求在单元区起点）。
2. 两盘各一道屏障。
3. **提交**：两盘四个系统配置槽写新池（世代 1）。
4. 两盘各一道屏障。
5. 清 [0, 根环基址) 里两槽之外的字节、根环三个区域、journal 环（与今天同一组写零）。
6. 两盘各一道屏障。
7. 第 0 代根 FUA 写进三个区域的槽 0。
8. 两盘各一道屏障。

提交点之前只写旧池没写过的单元槽，旧池读的系统配置、根环、journal 环、单元一个字节不动。这是我能给出的对甲最有利的一种：只有它在提交点之前一个旧池读的字节都不改。

### 打中：新池 fsid 与旧池相同（x512-same，全量 7 874 态，`out/x512-same-jia-0-7874.log`，修标签后那一遍）

`python3 tools/summarize.py out/x512-same-jia-0-7874.log` 原样输出（列：态数、第一个没落的写在哪一段、择到哪个池、判定、带不带撕裂镜像）：

```
name=done h=x512-same order=jia states_judged=7874 range=0..7874 seconds=329.8 threads=31
total_states 7874
1 all new new_complete whole
15 seg0 old old_readback_intact whole
1 seg1 Err:NoValidSystemConfiguration refused rerun_mkfs=NoFile(i0t0) rerun_red=["I-7.7"] torn
26 seg1 new MIXTURE_OR_BROKEN_NEW(root=i1t6) torn
3 seg1 new MIXTURE_OR_BROKEN_NEW(root=i1t6) whole
38 seg1 old old_readback_intact torn
12 seg1 old old_readback_intact whole
448 seg2 new MIXTURE_OR_BROKEN_NEW(root=i1t4) torn
128 seg2 new MIXTURE_OR_BROKEN_NEW(root=i1t4) whole
1472 seg2 new MIXTURE_OR_BROKEN_NEW(root=i1t5) torn
256 seg2 new MIXTURE_OR_BROKEN_NEW(root=i1t5) whole
4672 seg2 new MIXTURE_OR_BROKEN_NEW(root=i1t6) torn
512 seg2 new MIXTURE_OR_BROKEN_NEW(root=i1t6) whole
160 seg2 new refused_new_system_configuration_without_a_root torn
127 seg2 new refused_new_system_configuration_without_a_root whole
1 seg3 new refused_new_system_configuration_without_a_root whole
1 seg4 new new_complete whole
1 seg5 new new_complete whole
```

- 提交点之前（seg0 与 seg1 里择到旧池的）65 态：旧池读回与 (1, 6) 逐字节相同 65 / 65。甲的前半句在这份历史上成立。
- 提交点落了之后：7 517 态择到**新池的系统配置（世代 1）**、择到的根却是**旧池的** (1, 4) / (1, 5) / (1, 6)，恢复读出旧池的文件；按新池参数可写挂载做成的 6 679 态（`grep MIXTURE … | grep 'writable=Ok'` 按 count 相加）。不带撕裂镜像的 899 态。这是正文第一节乙那一句「不许挂上旧池或新旧混合」里的「新旧混合」，V1 题面「提交点之后与乙的后半段同问」。

最小历史（不带撕裂，原样那一行，`out/x512-same-jia-0-7874.log`）：

```
name=example h=x512-same order=jia ordinal=23 phase=seg1 torn=false torn_zero=false choice=new@4096g1 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=["I-7.7"] read_only=Ok(i1t6) writable=Ok(i1t6) verdict=MIXTURE_OR_BROKEN_NEW(root=i1t6) :: withheld=[6:system_configuration_slot@d1,7:system_configuration_slot@d1,8:zero_fill@d0,9:zero_fill@d0,10:zero_fill@d0,11:zero_fill@d0,12:zero_fill@d0,13:zero_fill@d1,14:zero_fill@d1,15:zero_fill@d1,16:zero_fill@d1,17:zero_fill@d1,18:root_record_fua@d0,19:root_record_fua@d1,20:root_record_fua@d0] torn_images_landed=[]
```

落了的：四次单元写、盘 0 的两个系统配置槽。每一步许可它的那一句：

1. 盘 0 两槽都是新池的世代 1，盘内只比世代号：`crates/singlefs-core/src/recovery.rs:1434`（`if one.quantities.slot_generation > zero.quantities.slot_generation {`）——两槽同为 1，取槽 0。
2. 跨盘取第一块交得出的盘那一份：`crates/singlefs-core/src/recovery.rs:1442`（`None => chosen = Some(best_on_device),`）；盘 1 还是旧池的世代 7 / 8，但跨盘只比 fsid 与设备数（`crates/singlefs-core/src/recovery.rs:1448` 那个 `SystemConfigurationsDisagree` 只在 fsid 或设备数不同时出），fsid 相同就过。
3. 择根只按 fsid 认根：`crates/singlefs-core/src/recovery.rs:1568`（`match RootRecord::parse_slot(&bytes, filesystem_identifier) {`），根记录自己那一处是 `crates/singlefs-core/src/root_record.rs:139`；取 (txg, 实例) 最大：`crates/singlefs-core/src/recovery.rs:1615`。根环没清，旧池 (1, 6) 的 txg 6 大于新池第 0 代根的 txg 0（`crates/singlefs-core/src/make_filesystem.rs:490`，`let genesis = CheckpointTxg(0);`），何况第 0 代根还没写。
4. 序号 95（四个系统配置槽全落、之后一个没落，原样行同在那份产物里）同一个结局：提交做完也一样。

### 卡在哪一类字节、为什么非重叠不可（推的，读码）

卡在**根环**（同 fsid 时 journal 环同理：journal 记录同样只按 fsid 认，`crates/singlefs-core/src/journal.rs:349`）。根环槽 j 的偏移是区域起点 + j × 槽距（`crates/singlefs-core/src/root_ring.rs:103`），区域起点是格式常量，与槽距无关：新旧两池不论 io_min 是否相同，至少每个区域的槽 0 落在同一个字节上；io_min 相同时整个根环逐槽重合。甲要求提交点之前不改旧池读的根环字节（旧池择根读遍全部槽，`crates/singlefs-core/src/recovery.rs:1506` 起那一段），而提交点之后、清根环落盘之前，新池读同一批槽、按同一个 fsid 认、按 txg 取最大，旧池的根必然胜过 txg 0 的第 0 代根。提交点只有一次写，「换系统配置」与「让旧根失效」落在两类字节上，凑不成一次原子写。今天的格式里没有一个新池读者能拿来分辨「这条同 fsid 的根是上一个池的」的字段，所以这一形在今天的实现上没有甲的写序能躲开。出路都不在今天的实现里（推的、零轮）：mkfs 先读旧池的系统配置、同 fsid 就拒或要求换 fsid；或在系统配置里加一个「池的代号」让择根拒掉别的代的根；或让第 0 代根的 txg 大于旧池最大的 txg。

### fsid 不同时（x512-diff，全量 7 874 态）：这条候选站住

`python3 tools/summarize.py out/v1-torn-label-bug/x512-diff-jia-*.log`（修标签之前那一遍，只引判定列）：择到旧池 50 态，读回逐字节相同 50 / 50；`SystemConfigurationsDisagree` 拒 30、`NoValidSystemConfiguration` 拒 1、择到新系统配置而根环里没有自证得过的根（恢复报 `NoValidRoot`，等于整池拒）7 790；新池完整 3；**混合 0、旧池读回不对 0**。

站住的前提（推的）：① 单元要写进旧池没用的槽——原型是靠「这份历史盘末从没写过」凑的，真实现要 mkfs 先走一遍旧池的全部活单元（旧池读不完时怎么办没有答案）；② 新旧 fsid 不同；③ 没有「更早一个同 fsid 的池」留在根环里。第 ③ 条我造了 `leftover` 这份历史（A 池 io_min 65536、fsid A → B 池 io_min 512、fsid B → 新 mkfs io_min 65536、fsid A）：B 的 mkfs 只清每区 16 × 4096 字节（`crates/singlefs-core/src/root_ring.rs:126`），A 池写在槽距 65536 上的槽 1..15 的根留着；新池按 65536 读到它们、fsid A 相同，提交点落了、清零没落时会择到 A 池的根（推的；那份历史 70 037 态没跑，命令在末尾）。这一形改法碰得到：B 池只读每区前 64 KiB，A 池残根所在的字节 B 池不读，甲可以在提交点之前先清掉它们——所以它打的是我这条候选，不是整支甲。

### 打中之后的四句（照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

1. 分不分辨臂：分。同一份历史 x512-same 上乙 31 195 态混合 0（见 V2），甲候选混合 7 517。
2. 被判的系统当时看不看得到判别它的东西：挂载那一刻看不到——新旧两池的根同 fsid、自证都过，盘上没有区分它们的字段（上面「卡在哪一类字节」）。mkfs 那一刻看得到（它可以读旧池系统配置的 fsid），所以能修它的只有 mkfs 一侧的改法。
3. 满足的是判据字面的哪一个分句：正文第一节乙那一条「作废持久之后到 mkfs 最后一道屏障之间崩溃，下一次挂载整池拒（没有自证得过的系统配置）或挂上完整的新池，不许挂上旧池或新旧混合」里的「新旧混合」，经 V1 题面「提交点之后与乙的后半段同问」带进甲。不是「挂上丢了已确认写的旧池」那一支（择到的是新池的系统配置）。
4. 跑前条款给的每个改法还中不中：正文没登记改法，只有甲、乙两支。乙在这一格不中（量过）；甲加「同 fsid 由 mkfs 拒」这一改法是我提的，**只在我的模型上推过、没实现没跑、被攻过零轮**。

## V2　乙撑不撑得住

### 写序（`tools/patch_copy.py` 的 `InvalidateFirst`，照派发提示）

两盘各清 [0, 根环基址) → 两盘各一道屏障 → 两盘清根环三区与 journal 环 → 两盘各写两个单元 → 两盘各一道屏障 → 三条第 0 代根 FUA → 两盘四个系统配置槽 → 两盘各一道屏障。段序列 `2+12+1+1+1+4`（量过，`out/counts.log`）。

### 结果（四份历史全量，`out/v1-torn-label-bug/*-yi-*.log`，判定列）

| 历史 | 全量 | 作废之前（seg0）择到旧池、读回对 | 作废之后整池拒 `NoValidSystemConfiguration` | 新池完整 | 择到旧池（作废之后） | 混合 |
|---|---|---|---|---|---|---|
| x512-diff | 31 195 | 8 / 8 | 31 122 | 65 | 0 | 0 |
| x512-same | 31 195 | 8 / 8 | 31 122 | 65 | 0 | 0 |
| x65536-diff | 31 150 | 8 / 8 | 31 110 | 32 | 0 | 0 |
| x65536-same | 31 150 | 8 / 8 | 31 110 | 32 | 0 | 0 |

（拒的列 = seg1 31 103 + seg2、seg3、seg4 各 1 + seg5 的 16 或 4；新池完整 = seg5 的 64 或 31 + 全落那 1 态。四份逐行由 `python3 tools/summarize.py out/v1-torn-label-bug/<历史>-yi-*.log` 出。）

### 作废那一步之前（正文 V2 第一问）

每份历史第 0 段只有 2 次写，全量 3² − 1 = 8 态（每次写取没落、撕裂、落三态）。修标签之后的原样（`out/x512-diff-yi-0-8.log`，`python3 tools/summarize.py` 之前的 `name=sig` 行）：

```
name=sig h=x512-diff order=yi count=2 phase=seg0 torn=false torn_zero=false choice=old@4096g8 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=[] read_only=Ok(i1t6) writable=Err:SystemConfigurationUnreadableAfterTheReadStage verdict=old_readback_intact
name=sig h=x512-diff order=yi count=1 phase=seg0 torn=false torn_zero=false choice=old@4096g8 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=[] read_only=Ok(i1t6) writable=Ok(i1t6) verdict=old_readback_intact
name=sig h=x512-diff order=yi count=2 phase=seg0 torn=true torn_zero=true choice=old@4096g7 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=[] read_only=Ok(i1t6) writable=Err:SystemConfigurationUnreadableAfterTheReadStage verdict=old_readback_intact
name=sig h=x512-diff order=yi count=2 phase=seg0 torn=true torn_zero=true choice=old@4096g7 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=[] read_only=Ok(i1t6) writable=Ok(i1t6) verdict=old_readback_intact
name=sig h=x512-diff order=yi count=1 phase=seg0 torn=true torn_zero=true choice=old@4096g8 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=[] read_only=Ok(i1t6) writable=Ok(i1t6) verdict=old_readback_intact
```

- 读回 8 / 8 与 (1, 6) 逐字节相同、checker 8 / 8 绿：这一段只落了 [0, 根环基址) 的清零，旧池的根环、journal 环、单元一个字节没动。今天写序下同一段（第 0 段 14 次写）的 248 832 态里出的回退与读不出，在乙这一段造不出来。
- 一盘作废落了、另一盘没有：择到没作废那块盘上的旧池系统配置（`crates/singlefs-core/src/recovery.rs:1429` 那一行把两槽都不可择的盘跳过），只读挂载做成，可写挂载报 `SystemConfigurationUnreadableAfterTheReadStage`（4 / 8 态）。
- 撕裂（序号 1，`torn_images_landed=[21:zero_fill@d0o0]`）：盘 0 的清零撕成前一半清了、后一半还是旧字节（撕裂镜像的切点在 `crates/singlefs-checker-tier/src/crash.rs:2272`），槽 0 没了、槽 1（世代 7）还在；跨盘取第一块盘那一份，择到世代 7 而盘 1 上是世代 8。读回照样 (1, 6)，可写挂载做成。旧池平常运行里不会出现「第一块盘只剩落后一代的那一槽」这一形，它在这里只因为 mkfs 的清零撕在两槽之间。
- 这三样都不违反甲、乙题面的字面（读回逐字节相同）；它们也不分辨臂：甲候选的提交段（x512-diff 的 seg1）同样出「择到旧池、可写挂载拒」（10 态，`writable=Err:SystemConfigurationUnreadableAfterTheReadStage`，`out/v1-torn-label-bug/x512-diff-jia-*.log` 的 seg1 各行相加）。

### 作废持久之后（正文 V2 第二问）

- 两块盘作废持久的先后：写序里两盘的清零在同一段、之后两盘各一道屏障，作废之后的第一个写在两道屏障之后，所以「一盘作废、一盘没作废」只出在第 0 段（上一小节那 8 态），作废之后的段里没有这一形（量过：seg1 起择到旧池 0 态）。
- 挂载会不会择到没作废那块盘：作废之后两块盘 [0, 根环基址) 全 0，逐档找槽 1 那一读也只读这一段（D22 已定项 8 第 3 条写「恰盖住系统配置槽 0 与逐档找槽 1 的全部 2 033 档」，`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。新系统配置只在第二道屏障之后写，那时根环、journal 环已清、单元已落盘，新 fsid 与旧 fsid 相同也挡得住：x512-same、x65536-same 两份同 fsid 的历史混合 0。
- 「按 fsid 筛」（用户 2026-10-19 定的 S3）今天没进 `crates/`，上面的数都是今天的择法；S3 只接在可写挂载那一路（推的，读的是 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 335 行的转述），作废之后系统配置全 0，筛与不筛都无物可择，不改这一格（推的）。

### 整池拒之后（正文 V2 第三问）

- 用户看到的错误：恢复与只读挂载报 `RecoveryFailure::NoValidSystemConfiguration { first_device_with_no_valid_system_configuration_slot }`（`crates/singlefs-core/src/recovery.rs:474`），带的是第一块盘。
- 能不能直接再跑一次 mkfs：能。四份历史全部拒的态（124 464 态）上照今天写序在拷贝上再跑一次 mkfs，恢复全是 `NoFile(i0t0)`（第 0 代空池）；mkfs 自己不读盘上任何东西，只判几何（`crates/singlefs-core/src/make_filesystem.rs:485`），拒不拒与盘上内容无关。但重跑之后池级 checker 全部报 I-7.7，见下面「另：用过的盘上 mkfs 做完报 I-7.7」。

## V3　代价（两支各要改的；数是原型 `count` 模式量的，`runs/counts.log`，副本上的数）

| 项 | 今天 | 乙（作废在前） | 甲候选（提交点之前不碰旧池读的字节） |
|---|---|---|---|
| 录制流写数 | 21 | 21（量过） | 21；新池槽距大于 4096 时 23（量过：多一段 [4096, 槽距) 清零） |
| 屏障（每盘一道记一次） | 4 | 6（推的：多两盘各一道） | 8（推的：单元后、提交后、清零后、根后各两道） |
| 段序列（量过） | `14+1+1+1+4` | `2+12+1+1+1+4` | `4+4+10+1+1+1`（槽距 65536 时 `4+4+12+1+1+1`） |
| 两态闭式状态数（量过） | 16 402 | 4 117 | 1 057（leftover 4 129） |
| 用过的盘上带撕裂镜像的全量（量过） | 280 019（x512）/ 279 974（x65536） | 31 195 / 31 150 | 7 874（x512）/ 17 549（x65536）/ 70 037（leftover） |
| C656 验收那一格要枚举的「作废或提交之前」那几段 | 第 0 段（14 写） | 第 0 段 2 写：3² − 1 = 8 态（量过：每份历史 seg0 都是 8 态） | 第 0 段 4 写 + 提交段 4 写：x512-diff 里 15 + 80 = 95 态（量过） |
| mkfs 之外要加的 | — | 无 | mkfs 先读旧池：旧池系统配置的 fsid 与几何、旧池全部活单元的落点（单元要写进旧池没用的槽）；同 fsid 时拒或换 fsid（不然打中，见 V1）；`allocator_after_make_filesystem` 与 `root_ring_occupancy_after_make_filesystem` 的单元落点与根槽不再是写死的单元区起点（推的） |

两支都要改的处数（`grep -rn` 现数，不分支）：
- 段序列 `14+1+1+1+4` 写死在 `crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:280`、`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:364`（连同 25 次操作与 16402），`scripts/experiments/e142_new_pool_file_creation_dry_run.rs` 五处（2280、5662、8712、8730、8868 行，其中 8730 是 `assert_eq!(sizes(mkfs_operations), vec![14, 1, 1, 1, 4]);`），`scripts/crash/check-segment-registry-product-sample.txt:5`，门禁 `checker-tier-qemu-device-streams` 的样本与说明（`.claude/gate.d/checker-tier-qemu-device-streams.sh:85` 与它 fixtures 下三份）。`16402` 在 kb 里另有 14 份文件命中（`grep -rln '16402\|16 402' crates scripts .claude/gate.d .claude/kb` 共 14 份文件、26 行），哪几处说的是 mkfs 那一行没逐个分。
- 布局表八「mkfs 种根」那一行（`.claude/kb/layout/01-first-txn.md:410`）段序列、操作数、闭式数三样都要改，三语各一行。
- `crates/singlefs-core/src/make_filesystem.rs:3` 文件头还写着 `段序列 \`12+1+1+1+4\``、「每块盘 4 次写零调用」，与第 559–563 行（每盘 5 次）和布局表 `14+1+1+1+4` 已经不一致：不论哪支，这一句本来就该改（这一条是现查到的旧注释，与两支无关）。
- D22 已定项 8 第 3 条（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）：乙要在「写任何元数据之前，先把每块盘的 [0, 根环基址) 整段写 0」之后加「两盘各过一道屏障，再清根环与 journal 环」一句，并改掉同一行末尾「清零几段之间不加屏障」与「每块盘 5 步、两块盘共 10 步」所在那一句的段序列指向；甲要把「写任何元数据之前」整句倒过来（清零挪到提交点之后），并新写「mkfs 先读旧池、同 fsid 怎么办、单元落点」三样，依据里 C484、C651 两条（清零是为了挡旧根、旧槽 1）要改写成「提交点之后再清」。
- 乙在 mkfs 第一道屏障之后的状态数从 280 019 降到 31 195（同一份历史、同一个展开），原因是 14 次写的大段拆成 2 + 12（推的：段内任意真子集随写数指数涨）。

### 另：用过的盘上 mkfs 做完，池级 checker 报 I-7.7（不分辨臂）

- 量过：三种写序在 x512-diff（fsid 不同、槽距相同）上全部落完的那一态都是 `raw_red=["I-7.7"]`（甲：`out/v1-torn-label-bug/x512-diff-jia-*.log` 的 `phase=all` 那一行；乙：`out/v1-torn-label-bug/x512-diff-yi-*.log` 同样一行）；整池拒的态上照今天写序重跑 mkfs，124 464 态全报 `rerun_red=["I-7.7"]`。
- 对照（量过，`out/fresh-control.log`）：全新空盘上照今天写序 mkfs，io_min 512 与 65536 都是 `raw_red=[]`。
- 读法：I-7.7 是「系统配置实例代号不低于根环」（`crates/singlefs-checker/invariants/README.md` 第 30 行那个名字）。C659（`.claude/kb/checks-owed.md:431`）的字面是「同一个 fsid 在用过的盘上换槽距重做 mkfs」，攻方当时推的根因是「旧池的根留在新 mkfs 按新槽距清的那一段之外」；这里 fsid 不同、槽距相同、根环整段清了也红，那个推的根因罩不住它。根因我没查（判据自己也会写错的第一种：打中不分辨臂，病根在三种写序共用的「mkfs 只清固定结构、不管单元区」或 checker 一侧，另立一笔账）。

## V4　没量的几何会不会翻面

| 第三节「没量的」那一样 | 乙 | 甲候选 |
|---|---|---|
| 旧池 io_min 与新池不同、两块盘都重用 | **不翻面**（量过：x65536-diff、x65536-same 各 31 150 态，作废之前 8 / 8 读回对，作废之后旧池 0、混合 0） | 没跑（17 549 态 × 2，命令在末尾）。推的：同 fsid 时仍打中——区域起点与槽距无关，每区槽 0 新旧重合（`crates/singlefs-core/src/root_ring.rs:103`），旧池在槽 0 上的根（至少第 0 代根，除非已被轮换盖掉）在提交点之后照样被新池择到；fsid 不同时与 x512-diff 同理站住 |
| fsid 相同、两块盘都重用 | **不翻面**（量过：x512-same、x65536-same） | **翻面**：这就是 V1 的打中（量过 x512-same） |
| u 的其他取值 | 没量。推的：乙在作废之前只动 [0, 根环基址)，结局只看两盘系统配置槽的内容，与 u 无关；u 只改旧池世代号与根的落点 | 没量。推的：u 越大、旧池的根越可能已轮到每区槽 0 上，io_min 不同时同 fsid 的混合只会更多不会更少；u = 0（旧池只做过 mkfs、两槽世代都是 1）时盘内世代号打平取槽 0（`crates/singlefs-core/src/recovery.rs:1434`），提交点提前到第一个槽写，结局形状不变（推的） |

## 第三节的数能不能在入库代码上核

- **能对上的（量过，副本）**：我另写的原型（没拷调查员的探针，历史与 mkfs 录制自己造）在「两块盘都重用、新 fsid、同 io_min、u = 5」这份历史、今天的写序上，算出段形 `[14, 1, 1, 1, 4]`、带撕裂全量 280 019（`out/counts.log` 第 4 行 `h=x512-diff order=today`），与第三节「全量 280 019 态」相同；闭式 16 402 与布局表八 mkfs 那一行相同。
- **没重跑的**：248 832 态那张拆分（173 568 / 6 144 / 9 216 / 59 904 / 246 528 / 53 248 / 1 024）。我的原型能跑（`run x512-diff today`，280 019 态），这一轮因挂钟砍掉了，命令在末尾。
- **为什么不能在入库代码上原样核（读码）**：① 调查员的数来自打过两处补丁的副本（它的 `tools/patch_copy.py`：`crash.rs` 里逐态展开的函数从 `pub(crate)` 改成 `pub`，`recovery.rs` 的 F2 退回判法插了记点），入库代码里没有一个公开入口能拿到 mkfs 流每个崩溃态的镜像；② 入库的崩溃点重放从 mkfs 之后的池起枚举，mkfs 那一行的「崩溃点重放枚举」格是「不在」（正文第二节）；③ `enumerate_crash_states` 一族要一个判的根下标与文件版本表、经判定库录入（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:242` 起那几个入口），mkfs 流没有文件版本。要在入库装置上核，得先在 checker 档加一条「旧池上重做 mkfs」的录入流与观察者（推的代价，没估）。
- **正文转述的一处字面**：正文第三节「起点镜像 checker 红 246 528」——调查员报告第 190 行原样是 `246528 before_first_barrier old raw_checker red`，`raw` 是崩溃镜像（还没挂载的那一张），调查员报告里「起点」另指 mkfs 一个写都没落的那一张（报告第 157–161 行，那份 reuse=both 起点 `raw_red=[]`）。正文把两者写混了；数没错，限定词错了。

## 没打中的形状

| 形状 | 打的是谁 | 取样 | 结局 |
|---|---|---|---|
| 作废之前只落一盘的清零、或清零撕在两槽之间，让择系统配置落到「落后一代」那一槽 | 乙 | 四份历史各 8 态全量 | 读回 32 / 32 对；只换来可写挂载拒或择到世代 7（见 V2），字面不中 |
| 作废之后找一份残留的旧系统配置（逐档找槽 1 读到 [0, 根环基址) 之外） | 乙 | 读码：格式允许的槽距上界让槽 1 整槽落在根环起点之前（`check_geometry` 的两两不重叠判），逐档只在 [4096, 1 MiB − 4096] | 没有对象 |
| 同 fsid、io_min 不同，让新池读到旧池写在别的槽距上的根 | 乙 | x65536-same 31 150 态 | 新系统配置只在根环清零落盘之后写，混合 0 |
| 同 fsid 时作废之后旧 journal 记录被重放进新池 | 乙 | x512-same、x65536-same 全量 | journal 环在新系统配置之前已清并过屏障，0 |
| 两块盘作废先后：一盘屏障返回、另一盘没返回时继续往下写 | 乙 | 写序上两盘屏障都在下一个写之前（录制流两道屏障相邻） | 作废之后的段里没有这一形 |
| 提交点之前改动旧池读的字节 | 甲候选 | x512-same、x512-diff 全量，提交点之前择到旧池的 115 态 | 读回 115 / 115 对 |
| fsid 不同时提交点之后混合 | 甲候选 | x512-diff 7 874 态 | 0（拒或新池） |

## 这条腿自己的限度

- 只量了 u = 5、两块盘都重用、新池 io_min 512、journal 环 6 MiB、小盘（2 799 槽）这一族；新池 io_min 大于旧池（新槽距更大）只在没跑的 `leftover` 里有。
- 甲候选的单元落点靠「盘末从没写过」凑出来，没实现「mkfs 读旧池找空槽」；甲在 fsid 不同时站住，是在这个凑出来的前提下。
- 撕裂只有崩溃点重放那一种形状（前一半新、后一半旧，`crates/singlefs-checker-tier/src/crash.rs:2272`）；1 MiB 清零反过来撕（槽 1 清了、槽 0 还在）不在枚举域里，乙作废之前那一段这一形没量（推的：只是换成择到另一槽，读回不受影响）。
- 由用户决定的动作只放开了一步：整池拒之后「再跑一次 mkfs」（全部拒的态都试了）。择到旧池之后用户接着可写挂载、再写、再崩这一串没扫；乙作废之前择到世代 7 的那几态，可写挂载做成之后再写会不会让两盘系统配置世代号错开，没量。
- 这条腿提的改法（甲加「同 fsid 由 mkfs 拒」、系统配置加池代号）都只在我的推理里，**只在我的模型上量过的部分为零，被攻过零轮**。
- 原型的第一版有撕裂标签错（V2 引的判定列不受影响，已在「原型怎么取样」写明）。

## 要主 agent 跑的命令

工作目录 `~/.local/share/singlefs/work/<新草稿目录>`，先照「复跑命令」那一段编出原型（或直接用 `rerun.sh`），二进制记作 `P`。每条 `SINGLEFS_CRASH_POINT_REPLAY_THREADS=31 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 2G $P run <历史> <写序>`，全量态数见 `out/counts.log`，单次不超过 10⁶：

| 历史 写序 | 全量 | 看什么 |
|---|---|---|
| `leftover jia` | 70 037 | `python3 tools/summarize.py <日志>` 里 `new MIXTURE_OR_BROKEN_NEW` 那几行：有就是 V1「更早一个同 fsid 的池」那一形坐实，0 就推翻那一句推的 |
| `leftover yi` | 31 150 | 同上那一列应为 0 |
| `x65536-same jia` | 17 549 | V4 第一行甲那一格推的「同 fsid、io_min 不同仍混合」 |
| `x65536-diff jia` | 17 549 | 混合与 `OLD_READBACK_NOT_INTACT` 应为 0 |
| `x512-diff today` | 280 019 | 核第三节 248 832 那张拆分（判定列按 `choice=old` 分恢复结局） |

## 没做什么

- 没判正推的格，没替主 agent 采纳；副本上的数不是入库装置上的数。
- 没跑重型测试、没调 checker 档任何测试目标，轻型三条都没起（这一件不改 `crates/`）。
- 编原型时 `librocksdb-sys` 的 C++ 在 8G、10G 两次撞内存上限（包装退 250，命令与原样在草稿目录 `build-attempt1-8G.log`、`build-attempt3-10G.log`、`build-attempt4-10G.log`），一次 20G 排队 789 秒没放进来、我自己停了排队；最后在 `capped.sh 31` 之内给 cargo 加 `-j 8` 编过（C++ 并发跟 cargo 的 jobs 走）。还试过从主工作区 `target/` 只读拷 librocksdb-sys 的产物进自己的编译目录，指纹不认、没用上。
- 停过自己起的两条链：一次排队里的编译、一次排队里的分段跑（`leftover yi` 排了 8 分钟没放进来）；后一次先停了外层 shell（免得它接着起下一段）再停排队那一步，没照「从最底层起」的次序。
- 草稿目录里留着冻结副本 `frozen/`（复跑要它，35M）与 `runs/`、`runs-v1/`；仓副本 `tree/` 与编译目录 `target/` 交回前删。

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m3-close-c656-r1-opus-s2-output.md` 末行（写这一节之前跑的）：

```
  ✓ 核了 0 处引文、29 处裸行号，对上 0 处，没判 29 处（其中裸行号没有引文可比 29 处；逐处列在上面）
```

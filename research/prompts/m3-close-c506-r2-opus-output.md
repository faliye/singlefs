# m3-close-c506-r2 攻方腿（Opus）报告（2026-10-22）

立场：假设第一轮判决第三节合成的形态（S 上界 16 与下界 4 同为格式常量、加「S × 固定结构槽距 ≤ P × chunk」、mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)、环深上界 3 × 16 焊死管理员回退深度、「冻结前随之抬」、GPU 根表 48）是错的，逐格找反例。方案按 `crates/` 今天的实现谈；引 kb 写 kb 文件自己的行号（现查）。没读禁读清单里的三份。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-c506-r2-opus-model/`。冻结副本是这一腿开工时从主工作区拷的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`（不带 target），放在 `~/.local/share/singlefs/work/m3-close-c506-r2-opus/tree`；拷时下列七份与主工作区逐字节相同（sha256 前 16 位两边相等）：`root_ring.rs`、`make_filesystem.rs`、`recovery.rs`、`system_configuration.rs`、`root_record.rs`、checker 的 `image.rs`、format 的 `lib.rs`。`tools/rerun.sh` 把冻结副本拷进 `--work`、打上 `patches/clause-only-writer.patch`（只在设了环境变量 `C506_R2_CLAUSE_ONLY_WRITER` 时让 mkfs 跳过「固定结构两两不重叠」那一判，不设时行为与冻结副本相同），再编、跑、与 `out/probe.out` 逐字节比。

```bash
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-c506-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c506-r2-opus/tree --work <新的草稿目录>
```

这一腿自己跑过一次（`--work ~/.local/share/singlefs/work/m3-close-c506-r2-opus/rerun1`，退出码 0），末行原样：

```text
name=rerun_compare same=1 lines=108 sha256=022cf91d59f558673ef67ae597393e362dfc187f925025e9f1b56d1bfc44795d
```

```text
dbd5ce99bad7cbe896df7480f4940fd7bba0632196abab05823644aef7c0999d  research/prompts/m3-close-c506-r2-opus-model/Cargo.toml
022cf91d59f558673ef67ae597393e362dfc187f925025e9f1b56d1bfc44795d  research/prompts/m3-close-c506-r2-opus-model/out/probe.out
445d772352f0163d8d422d0f52fbc10b6e19e7f043919ab930ce6e50d3905864  research/prompts/m3-close-c506-r2-opus-model/patches/clause-only-writer.patch
ce479090ead7b29d4e6c9ac4aa1725117a2402d5f4d48a2082fb885382858354  research/prompts/m3-close-c506-r2-opus-model/src/main.rs
68c4de885ee689042b47440d9837e27b3f6ea2177a44a9a6553cbad0ca47c101  research/prompts/m3-close-c506-r2-opus-model/tools/rerun.sh
```

跑法：原型只 `cargo build`（经 `capped.sh 31` 与内存包装），经内存包装（2G）直接执行编出来的二进制；盘是内存稀疏盘（`SparseBlockDevice`，4 GiB × 2）。不调崩溃枚举：这一腿造的历史不含崩溃点（打中的格都在不崩、不注故障的状态上），崩溃状态数 0，离约 10⁶ / 10⁷ 两条线都远。整份跑一次 7.7 秒（`time` 量的；扫描段 0.6 秒），没缩任何取样。开跑时 `ps` 看到别的会话的 `cargo build` 与 checker 档 `cargo test` 在跑，没有 `qemu-system`、`vm-bench.sh`、`e152`、`fio`；资源提醒报本机可用 17–30 线程、空闲内存 11–18 GiB，结论都是「可以跑」，在本机加 `nice -n 19` 跑。第一次按 8G 上限起时内存包装排队，我停掉了自己那一条（`proc.py stop`，排队 4 分钟、没起跑），改 2G 重跑。

## 各格判定一览

| 格 | 判定 | 一句话 | 复现 |
|---|---|---|---|
| W1 | **打中（副本上量过）**；另「错拒」那一半没打中 | 「S × 槽距 ≤ P × chunk」里的 P 与 chunk 是系统配置偏移 363、367 两个盘上字段，**没有一个读者把它们钉成第一版常量**：core 按编译期常量放区域，池级 checker 与 GPU checker 内核按盘上字段放区域。这一判按常量算，P = 0 的盘 checker 判 I-7.17 红而 core 读得对、P = 5 或 chunk = 4 MiB 的盘 checker 在 1/3 的根环上读不到根也一格不红；按盘上字段算，第一轮那一盘（S = 16、512 KiB）只要 chunk 写成 4 MiB 就过这一判，25 / 25 个状态照样 I-7.17 红。P 取常量时：格式允许的每个槽距 × S 上，这一判与入库 mkfs 收的集合逐格相同，没有罩不住的相叠 | `out/probe.out` 的 `name=p_chunk*` 行（第 17–46 行）；`name=sweep` 两行 |
| W2 | **打中（量过）** | min(8, ⌊3 MiB ÷ 槽距⌋) 在槽距 629248..786432（pbs 512 时 308 个、pbs 4096 时 39 个）上写 **4**，正落在下界上，而入库 mkfs 在这些几何上收 S = 4；「小于 4 拒绝」与「所写的值不取两条边」在同一句里互相矛盾。只按 D2 已定项 19 的式子从 2 的幂 io_min 取槽距时只写得出 8 与 6，「小于 4 拒绝」那一支永远不走。编译期断言 `AT_MAKE_FILESYSTEM` 与 harness 模型的「S 取 mkfs 那一档」在 512 KiB 槽距上对不上 | `name=formula` 各行、`name=formula_spacing_from_io_min_*` 两行 |
| W3 | 第一轮的结论站住（量过）；方法的射程打中一处（量过） | 打上补丁的「只照条款判盘长的写者」直接写出 S = 16、512 KiB：与改字节盘 0 个扇区不同，25 / 25 行历史逐字相同，I-7.17 红 25 / 25（含一次都没挂的那一刻，第一轮只数了挂载后的 24 个）——红不是改字节造成的。但在用过的盘上（同一 fsid 先做过 S = 6、512 KiB 的池）两种造法差 5 个扇区，改字节盘择根择到旧池的 txg 16，合法写者择到 txg 0：「改字节等于合法写者」只在新盘上成立 | `name=legit_writer*`、`name=used_disk_old_pool` 行（第 47–49、100–108 行） |
| W4 | **打中（条文）三处**，一处没打中 | ① 同一格前半句写「「环深」这个词…这里不用它」，被判的那句随后就写「环深上界 3 × 16」；D16 的「环深下限」量的是每区槽数 S，被判的那句量的是 R × S，同一个词两个量纲。② 「焊死的是管理员回退最多能退的发布数」与 D22 已定项 25 定案「不要把根环读成…可以回滚 K 代」直接相反；管理员回退从回退候选集里选（txg ≥ F_生效），正常卸载把 F 抬到现行那一版，每个池的环深是它自己的 3S（mkfs 写 6 时是 18），48 只是各池的上确界。③ 「GPU 判器根表容量 48 … 抬上界先抬它」漏了同样按 48 定的候选根表 `C_MAX_CANDIDATES`，GPU checker 内核还按盘上 P、chunk 用 u32 乘区域偏移（推的：会绕回）。「冻结前随之抬」与 D15 已定项 8 不打架，但把 k_tol 一并冻住了而没说 | W4 节逐句并排 |

## W1：「S × 槽距 ≤ P × chunk」这一判本身

### 打中（副本上量过）：P 与 chunk 是没人钉住的盘上字段，这一判在两种读法下各罩不住一类

条款与实现今天的样子：

- 区域位置是「区域 r 落在 r × P × chunk，P 素数且 P > devs」（`.claude/kb/decisions/22-单元原子性怎么合成.md:58`），而「P = 3、chunk = 1 MiB 是第一版的 mkfs 常量」（`.claude/kb/decisions/22-单元原子性怎么合成.md:350`）——mkfs 常量，写进系统配置偏移 363、367 两个字段（layout 字段表那一行「| 几何 | 素数步长 P | 4 | 3 |」（`.claude/kb/layout/01-first-txn.md:141`））。
- 读者要钉成第一版常量的只点了根环起点、journal 环起点与 R：「读者解槽时读这个字段与 journal 环起点字段（偏移 325），与第一版常量（64、1024）不等一律整池拒；根环区域数 R（偏移 361 那 1 字节）同样读，不等于 3 整池拒」（`.claude/kb/decisions/22-单元原子性怎么合成.md:348`）。P、chunk 没在里面。
- core 只判那几样（「if recorded_root_ring_base_slot != ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION {」（`crates/singlefs-core/src/recovery.rs:1139`）、「if u64::from(recorded_root_ring_region_count) != ROOT_RING_REGIONS {」（`crates/singlefs-core/src/recovery.rs:1157`）），不读 363、367，区域起点用编译期常量：「ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES,」（`crates/singlefs-core/src/root_ring.rs:96`）；写系统配置时照常量写回：「writer.put_u32(u32::try_from(ROOT_RING_PRIME_STEP).expect("P"));」（`crates/singlefs-core/src/system_configuration.rs:593`）、「writer.put_u32(u32::try_from(ROOT_RING_CHUNK_BYTES).expect("chunk"));」（`crates/singlefs-core/src/system_configuration.rs:594`）。
- 池级 checker 读盘上字段：「let prime_step = u64::from(read_u32(slot, 363));」（`crates/singlefs-checker/src/image.rs:773`）、「let chunk_bytes = u64::from(read_u32(slot, 367));」（`crates/singlefs-checker/src/image.rs:774`），按它放根槽：「+ region * geometry.prime_step * geometry.chunk_bytes」（`crates/singlefs-checker/src/image.rs:1306`），`geometry_of` 只判这几步乘加溢不溢出，不判等于常量。
- GPU 两个内核也分成两边：恢复内核用常量步长「const ROOT_RING_REGION_STRIDE: u32 = 3145728u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:113`），checker 内核读盘上字段「c.prime_step = rd32(base, 363u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:173`）、按它算偏移「let offset = add64(slot_to_bytes(g.base_slot), u64_of(region * g.prime_step * g.chunk_bytes + slot * g.spacing));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:308`）。

所以被判那句「同一槽另判 S × 固定结构槽距 ≤ P × chunk」的 P × chunk 有两种读法：取第一版常量（core 今天放区域的读法），或取同一槽的盘上字段（「同一槽」的字面，checker 今天放区域的读法）。造法：mkfs 收的一格建池，按需先只暖机可写挂载 24 次，再把四个系统配置槽的 P、chunk（与 S）改写、整槽校验和重封（第一轮的同一个改法）。每份盘先报两个读者收不收、各把区域放在哪、这一判按两种读法各算出什么（模型按判据算的值，不是实现），再接着挂 4 次。原样输出（`out/probe.out` 第 23、24、25 行）：

```text
name=p_chunk arm=S8_4KiB_P0_after_24_mounts S=8 spacing=4096 P=0 chunk=1048576 region_bytes=32768 core_choose_system_configuration=accepted(S=8) checker_geometry_of=accepted(S=8,P=0,chunk=1048576) core_region_starts=[1048576, 4194304, 7340032] checker_region_starts=[1048576, 1048576, 1048576] checker_region2_start_inside_journal_ring=false judgment_with_first_version_gap=true judgment_with_on_disk_gap=false p_and_chunk_equal_first_version=false
name=p_chunk_history arm=S8_4KiB_P0_after_24_mounts mount_after_rewrite=0 mount_result=not_mounted disk0_newest_slot_P=0 disk0_newest_slot_chunk=1048576 judgment_with_on_disk_gap=false core_chosen_root_txg=70 red=["I-7.17<实例 17、txg 48 的根住在区域 2 槽 0：按 txg 算该在区域 Some(0)（第 0 代不判区域）槽 0>"]
name=p_chunk_history arm=S8_4KiB_P0_after_24_mounts mount_after_rewrite=1 mount_result=ok disk0_newest_slot_P=3 disk0_newest_slot_chunk=1048576 judgment_with_on_disk_gap=true core_chosen_root_txg=73 red=[]
```

读法：P = 0 时 checker 把三个区域都放在 1 MiB，盘 0 上区域 0 与区域 2 是同一段字节（这就是第一轮那一类「同盘区域相叠」，只是叠在 checker 的几何里），判 I-7.17 红；core 照常量放区域、择到 txg 70、挂得上。这一判按常量算是 `true`（32 KiB ≤ 3 MiB），罩不住；按盘上字段算是 `false`。core 挂一次就把两槽里的一槽照常量写回，checker 换读新槽，红没了——同一个池的 checker 几何被 core 的一次挂载悄悄改了。

第 29、30、35、36 行（chunk = 4 MiB、P = 5，各在 24 次挂载之后改写）：

```text
name=p_chunk arm=S8_4KiB_chunk4MiB_after_24_mounts S=8 spacing=4096 P=3 chunk=4194304 region_bytes=32768 core_choose_system_configuration=accepted(S=8) checker_geometry_of=accepted(S=8,P=3,chunk=4194304) core_region_starts=[1048576, 4194304, 7340032] checker_region_starts=[1048576, 13631488, 26214400] checker_region2_start_inside_journal_ring=true judgment_with_first_version_gap=true judgment_with_on_disk_gap=true p_and_chunk_equal_first_version=false
name=p_chunk_history arm=S8_4KiB_chunk4MiB_after_24_mounts mount_after_rewrite=0 mount_result=not_mounted disk0_newest_slot_P=3 disk0_newest_slot_chunk=4194304 judgment_with_on_disk_gap=true core_chosen_root_txg=70 red=[]
name=p_chunk arm=S8_4KiB_P5_after_24_mounts S=8 spacing=4096 P=5 chunk=1048576 region_bytes=32768 core_choose_system_configuration=accepted(S=8) checker_geometry_of=accepted(S=8,P=5,chunk=1048576) core_region_starts=[1048576, 4194304, 7340032] checker_region_starts=[1048576, 6291456, 11534336] checker_region2_start_inside_journal_ring=false judgment_with_first_version_gap=true judgment_with_on_disk_gap=true p_and_chunk_equal_first_version=false
name=p_chunk_history arm=S8_4KiB_P5_after_24_mounts mount_after_rewrite=0 mount_result=not_mounted disk0_newest_slot_P=5 disk0_newest_slot_chunk=1048576 judgment_with_on_disk_gap=true core_chosen_root_txg=70 red=[]
```

读法：chunk = 4 MiB 时 checker 把区域 2 放在 25 MiB，**落在 journal 环 [16 MiB, 784 MiB) 里面**（正是 W1 题面点名的「区域 2 越过 journal 环起点」），区域 1 放在盘 1 的 13 MiB；P = 5（「P 素数且 P > devs」在 4 块盘上会取的值）时放在 6 / 11 MiB。两份盘上 core 写根的区域 1、2 都不在 checker 读的地方，checker 只读到区域 0 的根，**一格都不红**。这一判两种读法都是 `true`：被判的那句在这两格上什么都没判。

第 41、42、43 行（第一轮那一盘，S = 16、512 KiB，只多把 chunk 写成 4 MiB）：

```text
name=p_chunk arm=S16_512KiB_chunk4MiB_from_S4_fresh S=16 spacing=524288 P=3 chunk=4194304 region_bytes=8388608 core_choose_system_configuration=accepted(S=16) checker_geometry_of=accepted(S=16,P=3,chunk=4194304) core_region_starts=[1048576, 4194304, 7340032] checker_region_starts=[1048576, 13631488, 26214400] checker_region2_start_inside_journal_ring=true judgment_with_first_version_gap=false judgment_with_on_disk_gap=true p_and_chunk_equal_first_version=false
name=p_chunk_history arm=S16_512KiB_chunk4MiB_from_S4_fresh mount_after_rewrite=0 mount_result=not_mounted disk0_newest_slot_P=3 disk0_newest_slot_chunk=4194304 judgment_with_on_disk_gap=true core_chosen_root_txg=0 red=["I-7.17<实例 0、txg 0 的根住在区域 0 槽 12：按 txg 算该在区域 None（第 0 代不判区域）槽 0>"]
name=p_chunk_history arm=S16_512KiB_chunk4MiB_from_S4_fresh mount_after_rewrite=1 mount_result=ok disk0_newest_slot_P=3 disk0_newest_slot_chunk=1048576 judgment_with_on_disk_gap=false core_chosen_root_txg=2 red=["I-7.17<实例 1、txg 2 的根住在区域 0 槽 12：按 txg 算该在区域 Some(2)（第 0 代不判区域）槽 0>"]
```

读法：按盘上字段算，8 MiB ≤ 3 × 4 MiB，这一判放行；区域 0 自己就占 [1 MiB, 9 MiB)，盖住 core 写区域 2 槽 0 的 7 MiB，第一轮打中的那一盘原样回来，5 / 5 个状态 I-7.17 红。core 挂一次之后槽里的 chunk 变回 1 MiB，同一判在同一个池上翻成 `false`：照「同一槽」读法实现的读者，第一次挂载放行、第二次挂载整池拒（推的：这一判没实现，只算了判据值）。

计数（在模型目录里跑，原样输出依次是 5、5、5、5、4）：`grep 'name=p_chunk_history arm=S16_512KiB_chunk4MiB_from_S4_fresh' out/probe.out | grep -c 'red=\["I-7.17'`；`grep 'name=p_chunk_history arm=S8_4KiB_chunk4MiB_after_24_mounts ' out/probe.out | grep -c 'red=\[\]'`；同一条换 `S8_4KiB_P5_after_24_mounts`、`control_S8_4KiB_P3_chunk1MiB_after_24_mounts`；`grep 'name=p_chunk_history arm=S8_4KiB_P0' out/probe.out | grep -c 'red=\[\]'`（P = 0 那条臂只有改写后、挂载前那一个状态红）。

GPU checker 内核另有一处（推的，没起 GPU）：第 308 行那一乘是 u32，WGSL 的 u32 乘法溢出时绕回；chunk 写 2³¹、P 写 1 时区域 2 的步长 2 × 2³¹ 绕回 0，GPU checker 把区域 2 读在区域 0 的地方（同盘相叠），而 CPU checker 用 u64 `checked_mul` 算出 1 MiB + 4 GiB。这一判按盘上字段算放行（S × 槽距 ≤ 2³¹）。

四句：

1. 分不分辨臂：分辨「P × chunk 取常量」「取盘上字段」「先把 P、chunk 钉成第一版常量再判」三种写法；不分辨「S 上界格式常量 / 挂载时现算」（两臂都要先回答 P × chunk 是哪个数）。
2. 被判的系统看不看得到判别它的东西：看得到——P、chunk 与 S、槽距在同一槽里，读者择槽时就能判。
3. 满足判据字面哪一分句：被判形态「同一槽另判 S × 固定结构槽距 ≤ P × chunk（区域不越过下一个区域的起点）」那一分句；「区域不越过下一个区域的起点」要成立，「下一个区域的起点」本身得是读者们共同认的那个数，今天 core 与 checker 认的不是同一个。打中的不变量是 I-7.17（P = 0、第一轮那一盘）与「两个读者收同一张表」（D22 已定项 16 第 1 句那条用户定案的要求，chunk = 4 MiB、P = 5 两份）。
4. 改法在打中的格上还中不中：见本节末的表。

### 没打中：P × chunk 取第一版常量时，这一判与入库 mkfs 收的集合逐格相同

扫法：pbs ∈ {512, 4096}，io_min = pbs（判定宽度最小，对齐那一判放得最宽），槽距取格式允许的每一个（4096 到 1 MiB − 4096，按 pbs 步进；格式自己的口径是「读者在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过同一读者收的根环起点减 4096）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:346`）），S ∈ 4..16。每一格调一次入库 mkfs，另按 0 / 1 / 0 归属逐盘算真落字节的几段（两个系统配置槽、本盘区域的每个根槽按 pbs 宽、journal 环）有没有相叠，与区域 2 末端过不过 16 MiB。原样输出（第 1、10 行）：

```text
name=sweep pbs=512 io_min=512 spacings=2033 geometries=26429 mkfs_accepted=9411 judgment_passes=9411 mkfs_accepted_and_judgment_refuses=0 judgment_passes_and_bytes_overlap_on_a_device=0 judgment_passes_and_region2_end_past_journal_16MiB=0 judgment_refuses_and_no_bytes_overlap=13784 judgment_passes_and_mkfs_refuses={}
name=sweep pbs=4096 io_min=4096 spacings=255 geometries=3315 mkfs_accepted=1184 judgment_passes=1184 mkfs_accepted_and_judgment_refuses=0 judgment_passes_and_bytes_overlap_on_a_device=0 judgment_passes_and_region2_end_past_journal_16MiB=0 judgment_refuses_and_no_bytes_overlap=1717 judgment_passes_and_mkfs_refuses={}
```

- 罩不住的相叠：0（区域 2 越过 journal 环、与系统配置槽相叠、槽距不是 2 的幂都在扫描里；系统配置槽 1 整槽在根环起点之前由槽距上界管，与这一判无关）。
- 错拒 mkfs 今天写得出的几何：0；这一判过而 mkfs 拒的：0。两边收的集合一样大（9411 / 9411、1184 / 1184），两个 0 一起说明逐格相同。
- 这一判拒、盘上并不相叠的：13784 + 1717 格。mkfs 今天也全拒（集合相同），是第一轮已登的那笔欠账（mkfs 按区域长 ≤ P × chunk 判、比 0 / 1 / 0 下盘上要的严），不是这一轮的新东西。
- 加在哪条不变量下（推的）：越界的处置写的是「读者整池拒」，D22 已定项 2 射程里同一处置的槽距越界由 checker 报 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量报不适用；这一判照它落 I-7.13 才与 core 的整池拒同口径。落 I-7.17 的话，core 拒、checker 报 I-7.17 红而别的照判，两边不同口径。I-7.17 是这一判缺席时红的那一条（第一轮与本节第 42 行）。

### W1 各改法在打中的格上（只在我的模型上量过、被攻过零轮）

| 打中的格 | 被判形态原句（P × chunk 没说取哪个） | 判按第一版常量 3 MiB | 判按同一槽的盘上 P × chunk | 读者把 P、chunk 与第一版常量（3、1 MiB）比、不等整池拒（同 R、根环起点那一句），再按常量判 |
|---|---|---|---|---|
| P = 0：checker I-7.17 红、core 读得对（第 23–24 行） | 两读法分叉 | 不中：判据值 `true`，放行（量过：模型按判据算的值） | 中：判据值 `false`（量过：判据值；core 怎么拒是推的） | 中（推的：没实现） |
| chunk = 4 MiB / P = 5：checker 读不到 2/3 的根环而不红（第 29–30、35–36 行） | 照中 | 照中：判据值 `true`（量过：判据值） | 照中：判据值 `true`（量过：判据值） | 中（推的） |
| 第一轮那一盘加 chunk = 4 MiB：I-7.17 红 5 / 5（第 41–43 行） | 两读法分叉 | 中：判据值 `false`（量过：判据值） | 照中：挂载前判据值 `true`，挂一次后翻 `false`（量过：判据值） | 中（推的） |
| GPU checker u32 绕回（推的） | 两读法分叉 | 照中 | 照中 | 中（推的） |

只有最后一列在四格上都不中。它比原句多要求的是「P、chunk 两个字段读并与第一版常量比」，与 D22 已定项 16 第 1 句对根环起点、journal 环起点、R 已有的那句同形；GPU checker 内核与池级 checker 的 `geometry_of` 同一处加。

## W2：mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)、所写的值不取两条边

### 打中（量过）：min 在一段 mkfs 收的槽距上正落在下界 4

同一次扫描里按每个槽距算 min(8, ⌊3 MiB ÷ 槽距⌋)，再查入库 mkfs 在那个槽距、那个 S 上收不收。原样输出（第 2–7、11–12 行）：

```text
name=formula pbs=512 written_S=3 below_lower_edge_refuse_mkfs spacing_from=786944 spacing_to=1044480 spacings=504 mkfs_accepts_that_S=0
name=formula pbs=512 written_S=4 on_an_edge spacing_from=629248 spacing_to=786432 spacings=308 mkfs_accepts_that_S=308
name=formula pbs=512 written_S=5 inside spacing_from=524800 spacing_to=628736 spacings=204 mkfs_accepts_that_S=204
name=formula pbs=512 written_S=6 inside spacing_from=449536 spacing_to=524288 spacings=147 mkfs_accepts_that_S=147
name=formula pbs=512 written_S=7 inside spacing_from=393728 spacing_to=449024 spacings=109 mkfs_accepts_that_S=109
name=formula pbs=512 written_S=8 inside spacing_from=4096 spacing_to=393216 spacings=761 mkfs_accepts_that_S=761
name=formula pbs=4096 written_S=3 below_lower_edge_refuse_mkfs spacing_from=790528 spacing_to=1044480 spacings=63 mkfs_accepts_that_S=0
name=formula pbs=4096 written_S=4 on_an_edge spacing_from=630784 spacing_to=786432 spacings=39 mkfs_accepts_that_S=39
```

- 槽距 ∈ [629248, 786432]（pbs 512 时 308 个、pbs 4096 时 39 个）：公式写 4，入库 mkfs 在 S = 4 上全收。被判那句「小于 4 拒绝 mkfs；所写的值不取两条边」在这一段上前后矛盾：拒的门槛是 < 4，留下的最小值恰是下界。要让后半句成立，门槛得是 ⌊3 MiB ÷ 槽距⌋ < 5（槽距 > 629145 拒），不是 < 4。
- 槽距怎么来：mkfs 的参数结构里槽距是调用方给的字段（「pub fixed_structure_slot_spacing: u32,」（`crates/singlefs-core/src/system_configuration.rs:231`）），`check_geometry` 的判定清单（「固定结构两两不重叠、系统配置槽与根槽按判定宽度对齐、区域归属。」（`crates/singlefs-core/src/make_filesystem.rs:444`）那一句收尾的次序表）里没有「槽距等于 D2 已定项 19 的式子」，读者也不判它。上面这一段的槽距在今天的 mkfs 内层入口与两个读者那里都合法。
- 只按条款的式子取槽距时（「槽距 = max(4096, mkfs 时探测的 io_min)」（`.claude/kb/decisions/22-单元原子性怎么合成.md:349`）），io_min 只能是能整除 1 MiB 的 2 的幂才过对齐那一判（非 2 的幂六个都被 `FixedStructureSlotNotAlignedToTheDecisionWidth` 拒，第 9 行），写出的 S 只有两种（第 8 行原样）：

```text
name=formula_spacing_from_io_min_power_of_two io_min=4096:spacing=4096:S=8:mkfs=accepted io_min=8192:spacing=8192:S=8:mkfs=accepted io_min=16384:spacing=16384:S=8:mkfs=accepted io_min=32768:spacing=32768:S=8:mkfs=accepted io_min=65536:spacing=65536:S=8:mkfs=accepted io_min=131072:spacing=131072:S=8:mkfs=accepted io_min=262144:spacing=262144:S=8:mkfs=accepted io_min=524288:spacing=524288:S=6:mkfs=accepted
```

  这时「不取两条边」成立，而「小于 4 拒绝 mkfs」那一支一次都走不到（最大的合法 2 的幂槽距 512 KiB 写 6；1 MiB 已被槽距上界拒）。两种读法里这句总有半句是空的或错的。

四句：① 分不分辨臂：分辨「门槛 < 4」与「门槛 < 5」（或「mkfs 判槽距等于 D2 已定项 19 的式子」）；不分辨 S 上界的两臂。② 系统看得到：mkfs 手里就有槽距。③ 满足的分句：「mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，小于 4 拒绝 mkfs；所写的值不取两条边」。④ 改法见本节末的表。

### 打中（读代码）：按「S = 8」写死、S 随槽距变之后对不上的地方

- 编译期断言：「pub const AT_MAKE_FILESYSTEM: Self = Self::at_make_filesystem_checked_at_compile_time(」（`crates/singlefs-core/src/root_ring.rs:44`）判的是常量 8 严格落在两条边之间（「slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM」（`crates/singlefs-core/src/root_ring.rs:50`））。S 改成槽距的函数之后它判不到运行时那个值，「不取两条边」要么退成运行时判、要么只对 8 那一档说话。常量本身「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」（`crates/singlefs-format/src/lib.rs:309`）同理。
- harness 模型：「const MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =」（`crates/singlefs-harness/src/model.rs:1815`），注释写「/// S 取 mkfs 那一档：模型只建 mkfs 默认参数的池」（`crates/singlefs-harness/src/model.rs:1812`）。默认参数的槽距是 4096、公式仍写 8，今天不错；io_min 512 KiB 的池上 mkfs 写 6，模型照 8 算落点会与实现分叉（推的，没跑模型差分）。
- layout 字段表「| 几何 | 每区槽数 S | 1 | 8 |」（`.claude/kb/layout/01-first-txn.md:140`）与根环参数一览「R = 3、S = 8、P = 3、chunk = 1 MiB、基址 1 MiB、槽宽 512、槽距 4096」（`.claude/kb/layout/01-first-txn.md:164`）：后者连着槽距 4096 写，仍对；前者那一格的 8 要写成「槽距 ≤ 384 KiB 时」。
- 没打中：D28 环窗口读的是系统配置里的 S（「S 是系统配置每区槽数（S = 4 / 8 / 16 时环窗口 13W / 25W / 49W）」（`.claude/kb/decisions/28-挂载期承诺量.md:31`）），不写死 8；E50 扫的是槽总数 N，与 mkfs 写几无关；core 产品路径里 `AT_MAKE_FILESYSTEM` 的用处全在 `#[cfg(test)]` 之后（`grep -n 'AT_MAKE_FILESYSTEM'` 在 `admission.rs`、`allocator.rs`、`mount.rs` 的命中行号都大于各自第一个 `#[cfg(test)]`：1985 > 1726、3909 > 2972、10496 > 9866）。

### W2 各改法（只在我的模型上量过、被攻过零轮）

| 打中的格 | 原句（门槛 < 4） | 门槛改成 ⌊3 MiB ÷ 槽距⌋ < 5 | mkfs 另判槽距 = 4096 向上取整到 io_min（D2 已定项 19） |
|---|---|---|---|
| 槽距 629248..786432 写 4（第 3、12 行） | 照中（量过） | 中（推的：这一段改成拒） | 中（推的：这一段的槽距 io_min 取 512 / 4096 时不等于式子，拒；式子只给 2 的幂） |
| 「小于 4 拒绝」在 2 的幂槽距上走不到（第 8 行） | 照中（量过） | 照中：仍走不到 | 照中：仍走不到，可删那半句 |
| 编译期断言判不到运行时 S | 照中（读代码） | 照中 | 照中 |

## W3：第一轮「改字节造盘」这一法

### 第一轮的结论站住（量过）：S = 16、512 KiB 直接写出来，与改字节盘逐扇区相同，I-7.17 红不是改字节造成的

造法：冻结副本打上 `patches/clause-only-writer.patch`，设 `C506_R2_CLAUSE_ONLY_WRITER` 时 mkfs 跳过「固定结构两两不重叠」那一判（`crates/singlefs-core/src/make_filesystem.rs:528` 那一行 `fixed_structure_extents` 换成空表），盘长、对齐、区域归属等别的判照旧——这就是第一轮判决说的「只照 D22 已定项 16 第 3 句判盘长的写者」。不设时行为与冻结副本相同（第 47 行：同一格补丁关着照旧拒）。原样输出（第 47–50、100 行）：

```text
name=legit_writer_patch_off mkfs_S16_512KiB=refused:FixedStructuresOverlap
name=legit_writer_patch_on mkfs_S16_512KiB=accepted
name=legit_writer_fresh_sectors nonzero_rewritten=11 nonzero_legit=11 differing_sectors=0 where=[]
name=history arm=legit_writer_S16_512KiB_fresh mount=0 mount_result=not_mounted chosen_root_txg=0 misplaced_roots=["r0s12:txg0"] red=["I-7.17<实例 0、txg 0 的根住在区域 0 槽 12：按 txg 算该在区域 None（第 0 代不判区域）槽 0>"]
name=legit_writer_fresh_history lines=25 identical_lines=25 legit_states_with_i717_red_first=25
```

- 新盘上合法写者与改字节盘（S = 4、512 KiB 的 mkfs 再把 S 那一字节改成 16）非零扇区各 11 个、0 个不同；两份盘接连只暖机可写挂载 0..24 次，25 行历史逐字相同（比的是去掉臂名之后的整行：挂载结果、择到的根、按 core 几何读出的错位根、checker 红的条目）。第一轮那一推（「只在三对 mkfs 两边都收的几何上比过、推到 S = 16 / 512 KiB 是外推」）现在在那一格上直接比过了。
- I-7.17 红 25 / 25，比第一轮多一格：**一次都没挂的 mkfs 刚写完那一刻就红**——合法写者把第 0 代根种进区域 2 槽 0（盘 0 的 7 MiB），那也是区域 0 槽 12 的位置。计数（模型目录里跑）：`grep -c 'name=history arm=rewritten_S4_to_S16_512KiB_fresh .*red=\["I-7.17' out/probe.out` 原样输出 `25`，合法写者那条臂由第 100 行的 `legit_states_with_i717_red_first=25` 报。
- 改字节盘与合法写者盘在别的字段上的差别：系统配置里没有随 S 变的派生字段（单元区起点、在飞上限、最坏占用都只随环长；第 0 代根的钉槽 p 写 255），新盘上两边只差「mkfs 清了多长」，而清零写在新盘上不改字节——上面 0 个不同的扇区就是这件事。

### 打中（量过）：在用过的盘上两种造法不等价，改字节盘择根择到旧池

造法：同一 fsid 先做一个 S = 6、512 KiB 的池（6 × 512 KiB = 3 MiB，mkfs 收），只暖机可写挂载 6 次；再在这两块盘的两份拷贝上各做一次：合法写者直接写 S = 16，或 S = 4 的 mkfs 再改字节成 16。原样输出（第 101–103、106 行）：

```text
name=used_disk_old_pool S=6 spacing=524288 chosen_root_txg=16 roots_beyond_slot3_of_region0=["r0s12:txg2", "r0s13:txg5", "r0s14:txg8", "r0s15:txg11"]
name=legit_writer_used_sectors legit_mkfs=accepted nonzero_rewritten=86 nonzero_legit=81 differing_sectors=5 where=["left:d0@3145728", "left:d0@3670016", "left:d0@9437184", "left:d1@6291456", "left:d1@6815744"]
name=history arm=legit_writer_S16_512KiB_used mount=0 mount_result=not_mounted chosen_root_txg=0 misplaced_roots=["r0s12:txg0"] red=["I-7.7<盘 0 槽 50240 的单元写序带实例代号 2，高于各盘系统配置的最大者 0（①）>", "I-7.17<实例 0、txg 0 的根住在区域 0 槽 12：按 txg 算该在区域 None（第 0 代不判区域）槽 0>"]
name=history arm=rewritten_S4_to_S16_512KiB_used mount=0 mount_result=not_mounted chosen_root_txg=16 misplaced_roots=["r0s12:txg0"] red=["I-7.7<根环区域 0 槽 4 的根带实例代号 5，高于各盘系统配置的最大者 0（①）>", "I-7.17<实例 0、txg 0 的根住在区域 0 槽 12：按 txg 算该在区域 None（第 0 代不判区域）槽 0>"]
```

（`used_disk_old_pool` 那一行的 `roots_beyond_slot3_of_region0` 是按 S = 16 的落点公式读 S = 6 旧池时不合公式的根，名字起得不准，读数照上。）

读法：S = 4 的 mkfs 每个区域只清 2 MiB，旧池住在 3 / 3.5 MiB（盘 0）、9 MiB（盘 0）、6 / 6.5 MiB（盘 1）的五条根留下来；改成 S = 16 之后它们落在新几何的合法槽里（例：盘 1 的 6.5 MiB 是区域 1 槽 5，txg 16 按 S = 16 的公式正该住那里），core 择到旧池的 txg 16。合法写者每个区域清 8 MiB，旧根全没了，择到 txg 0。两份盘上 I-7.17 照样都红（区域相叠与造法无关）；I-7.7 两边都红、点名的东西不同，属于同 fsid 在用过的盘上重做 mkfs 那一类（内层入口给定 fsid），不归这一轮。

四句：① 分不分辨臂：不分辨 S 上界的两臂，分辨「改字节造盘」与「合法写者」这两种方法。② 看得到：盘上就是这几个扇区。③ 满足的分句：第一轮报告「这一改等于…一个只照条款判盘长、照写 S 的 mkfs…写出的盘」（`research/prompts/m3-close-c506-r1-opus-output.md:138`）那一句；它只在新盘上成立。④ 改法：之后再用改字节造盘时，先在合法写者（这份补丁）上直接写一遍、逐扇区比，或只在新盘上用，并写明射程；对第一轮的 I-7.17 那一格不改判。

## W4：被判形态里与已定条款打架的句子

### 打中一（条文）：「环深上界」与同一格自己的禁用词、与 D16「环深下限」的量纲

被判的替换只换掉那一格「上界由 I-7.4…」起的后半，前半原样留着：「这个词在 C131（栈深与保护窗口的量纲） 出过一次事，这里不用它」（`.claude/kb/decisions/22-单元原子性怎么合成.md:60`，那一句之前是被引号括起的「环深」二字）。替换进来的文字却写「环深上界 3 × 16 进格式」。同一格先说不用这个词、后面就用。

量纲也不同：D16 的同名格是「每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬）」（`.claude/kb/decisions/16-发布语义.md:46`），「环深下限」量的是每区槽数 S；被判那句的「环深上界 3 × 16」量的是 R × S。读者把两格并读会得出「环深 ∈ [4, 48]」这种单位混着的区间——这正是那一格点名 C131 要防的事。

四句：① 不分辨 S 上界的两臂，是措辞。② 不涉及系统。③ 满足 D22 已定项 2 那一格自己的「这里不用它」。④ 改法：写成「环长（R × S 个槽）上界 48」或「每区槽数上界 16」，不用「环深」（推的）。

### 打中二（条文）：「焊死的是管理员回退最多能退的发布数」

- 与 D22 已定项 25 的定案句相反：「不要把根环读成…可以回滚 K 代」「它买到的只是…本代根写坏了还能读到上一代根」（`.claude/kb/decisions/22-单元原子性怎么合成.md:495`）。被判那句把环深上界直接读成「管理员回退最多能退几次」。同一项依据段有「恢复择根回退恰 2 代、管理员回退要整个根环深度」（`.claude/kb/decisions/22-单元原子性怎么合成.md:507`），那是 E92 对消费者的计价，不是定案；被判那句站在依据段这一边、与定案句相撞。
- 管理员回退实际受什么限：「管理员在一次可写挂载里从回退候选集选一个旧根 R_old」（`.claude/kb/decisions/23-journal的角色与格式.md:380`），回退候选集是「按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效」（`.claude/kb/decisions/16-发布语义.md:41`），而 F「平时不动，窗口就是整个根环；准入不够时抬，正常卸载时抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:37`）。所以一个池上能退多少由 F 与这个池自己的 3S 定：mkfs 默认写 8 时 24，照被判形态在 io_min = 512 KiB 上写 6 时 18（第 8 行），正常卸载之后再挂是 0。48 只是所有池的 3S 的上确界，焊死的是「读者承诺收的最大环长」，不是哪个池「最多能退的发布数」。
- 单位：D16 已定项 1 的保留承诺按「可退到的不同状态」数，空发布不算；「发布数」把空发布也数进去。

四句：① 不分辨 S 上界的两臂（两臂下管理员回退都由 F 与本池 3S 限）。② 系统看得到 F 与本池 S。③ 满足 D22 已定项 25 定案句。④ 改法：改成「环长上界 3 × 16 进格式，焊死的是读者承诺收的最大环长；一个池上管理员回退能退到哪里由回退候选集（txg ≥ F_生效）与这个池自己的 3S 定」（推的）。

### 打中三（读代码）：「GPU 判器根表容量 48 = 3 × 16，抬上界先抬它」点名不全

- 根表：「const MAX_ROOTS: u32 = 48u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:241`）。同一批内核里另有「const C_MAX_CANDIDATES: u32 = 48u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:567`），候选根表，「不多于 C_MAX_CANDIDATES 行」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:587`）——候选根出自根环，同样随 3S 涨（推的：没逐处追到它只受 3S 限）。被判那句只点一个，照它抬上界会漏第二个。
- GPU checker 内核按盘上 P、chunk 用 u32 乘区域偏移（W1 节，第 308 行），抬 S 之外动 P、chunk 也要先改它。
- 这些是 checker 档的实现容量，进格式条款就得在条款里列全、随实现改；第一轮判决已登「S 区间的两条边在 GPU 内核三处是 `4u` / `16u` 字面量……format-const 钉不住」那笔欠账。改法（推的）：条款只写「上界的每个实现容量登记在 format-const 名下、门禁按名字核」，不在 D22 里点 GPU 的某一张表。

### 没打中（只记一笔）：「冻结前随之抬」与 D15 已定项 8

D15 已定项 8：「已冻结字段的宽度 / 存在性 / 含义不能再改；只能新增」（`.claude/kb/decisions/15-格式冻结政策.md:171`）。「冻结前随之抬」只在冻结前动下界，与它不冲突。留下两处没说的：① 冻结之后 k_tol 再抬，D16 那一格「S ≥ k_tol + 2」在已有 S = 4 的池上就不成立，而下界不能再抬——等于 k_tol 随系统配置槽一起冻住，被判那句没写。② 同一张表「已冻结组件的选取规则本身也被钉死」（`.claude/kb/decisions/15-格式冻结政策.md:173`）：min(8, ⌊P × chunk ÷ 槽距⌋) 是 mkfs 自动选取规则，冻结后连 8 与门槛一起钉死，W2 那一处的门槛要在冻结前改对。都是推的。

## 几个改法各修哪一格（汇总；只在我的模型上量过、被攻过零轮）

| 打中的格 | 被判形态原样 | 读者把 P、chunk 钉成第一版常量（不等整池拒）再按 3 MiB 判 | mkfs 门槛改成 < 5（或判槽距等于 D2 已定项 19 的式子） | 「环深」改成「环长 / 每区槽数」、「焊死」那半句改写 | 条款不点 GPU 表名，上界的实现容量登 format-const |
|---|---|---|---|---|---|
| W1 P = 0 / chunk = 4 MiB / P = 5 / 第一轮盘加 chunk = 4 MiB | 照中（量过：判据值与 checker 红） | 不中（推的：没实现） | 照中 | 照中 | 照中 |
| W2 公式写 4 | 照中（量过） | 照中 | 不中（推的） | 照中 | 照中 |
| W2 编译期断言、harness 模型写死 8 | 照中（读代码） | 照中 | 照中（断言要改成运行时判，推的） | 照中 | 照中 |
| W3 改字节盘在用过的盘上择到旧池 | 方法问题，不涉被判形态（量过） | 无关 | 无关 | 无关 | 无关 |
| W4 「环深」两种量纲 | 照中（条文） | 照中 | 照中 | 不中（推的） | 照中 |
| W4 「焊死管理员回退」与已定项 25 相反 | 照中（条文） | 照中 | 照中 | 不中（推的） | 照中 |
| W4 GPU 表点名不全 | 照中（读代码） | 照中（P、chunk 那一乘不再有非常量输入，推的） | 照中 | 照中 | 不中（推的） |

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| P × chunk 取常量时这一判罩不住的同盘相叠（区域 2 过 16 MiB、与系统配置槽相叠、槽距不是 2 的幂） | pbs 512 / 4096 × 格式允许的每个槽距（2033 / 255 个）× S 4..16，共 29744 格，每格调入库 mkfs、逐盘算真落字节 | 0 格（`name=sweep` 两行） |
| 这一判错拒 mkfs 今天写得出的几何 | 同上 | 0 格；两边收的集合逐格相同 |
| min 公式撞上界 16 | 同上的每个槽距 | 不会：min(8, …) ≤ 8 |
| D28 环窗口、E50、core 产品路径按 S = 8 写死 | 读 D28 第 31 行、E50 扫的量、`grep -n AT_MAKE_FILESYSTEM` 的 core 命中 | 都读运行时 S 或在测试里 |
| 改字节造的盘在新盘上与合法写者不同 | S = 16、512 KiB 一格，非零扇区逐个比 + 25 行历史逐行比 | 0 个不同、25 / 25 行相同 |
| 「冻结前随之抬」与 D15 已定项 8 冲突 | 读 D15 已定项 8 全表 | 不冲突；记了两处没说的（W4 末节） |
| `MAX_ROOTS` 不等于 3 × 16 | 读 `checker_collect_roots` 与恢复内核的 `choose_root`：都按 R × S 个槽逐个收，R 钉在 3 | 等于；问题只在点名不全 |

## 这条腿自己的限度

- W1 的「这一判」两种读法都没实现进读者：表里「判据值」是模型按盘上字段算出的布尔值，读者真按它拒的样子、拒报哪个成员、checker 报哪一条都是推的。P、chunk 钉成常量那一列没在副本上实现。
- W1 的 P / chunk 盘是改字节造的（与第一轮同一个改法）。写得出 P ≠ 3 的合法写者今天不存在（core 写系统配置一律写常量）；条款上「P 素数且 P > devs」在多盘时会选别的 P，那是唯一的合法来源，是推的。这一腿 W3 证的「改字节 = 合法写者」只证了 S 那一字节、只在新盘上。
- GPU 那两条（u32 绕回、`C_MAX_CANDIDATES` 随 3S）是读 WGSL 推的，没起 GPU（checker 档，归重型）。
- W2 harness 模型在 512 KiB 槽距上分叉是推的，没跑模型差分。
- 历史只扫了「只暖机可写挂载」一种用户动作（P / chunk 臂 24 + 4 次，合法写者臂 24 次，用过的盘 2 次）；不写文件、不删、不崩、不注故障。打中的红在第一个状态就出现，不靠后面的动作。
- 没做 s2 抽样：「没打中」的几格（W1 常量读法、W3 新盘等价）是一次观测。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没读禁读清单里的三份（`m3-close-c506-r2-sonnet-output.md`、`m3-close-c506-r2-sonnet-model/`、`m3-close-c506-r2-main-verification.md`）。
- 没跑重型测试、门禁阶段、崩溃枚举（历史不含崩溃点，崩溃状态 0）、GPU 判器、E50 / E139 复跑。
- 没改仓里任何 `crates/` 文件；补丁只在草稿目录与模型目录的副本上。

## 要主 agent 跑的命令

无必须的。判决要引这份的数时，照开头「复跑命令」在新草稿目录复跑，看末行 `name=rerun_compare same=1`。

## 草稿目录里留着的

- `~/.local/share/singlefs/work/m3-close-c506-r2-opus/tree`：冻结副本（不带 target，37 MiB），复跑命令的 `--frozen` 指它，所以不删。
- 交回前删：`dev/`（开发用的副本与编译目录）、`rerun1/`（复跑目录，含编译目录）。

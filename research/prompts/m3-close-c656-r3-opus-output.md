# m3-close-c656-r3 攻方腿（Opus）报告（2026-10-20）

攻击面：正文第三节 S1、S2、S3（最终轮）。所有数都在草稿目录的仓副本上量（副本数，不是入库装置上的数）。没读禁读清单里这一轮辩方腿的产物。

两套 crates 各量一遍：
- **冻结副本**（开工时拷的，`crates` 下 `.rs` 全部 sha256 再取 sha256 `3dc1ba5195ee541b85c7248f6ceeba3f82026f17a8ab6b382463688b9fc6ad8b`，与第二轮攻方冻结副本逐字节相同），产物在模型目录 `out/`。
- **主工作区后来的样子**（干活当中别的会话往主工作区合进了系统配置第三轮的改动：`recovery.rs`、`mount.rs`、`transaction.rs`、`walk.rs` 变了，`make_filesystem.rs`、`mounted_read.rs` 没变；拷成第二份冻结副本，同一个取法 sha256 `9a136ba411d02a6a01fbac29769ab8bf43ddf6f91ac36043bebe334ca72ba63f`），产物在模型目录 `out-live/`。下文写「今天的 crates」指这一份。两份的差别只在下面点名的几格，别的格逐字段相同（去掉世代号之后比的）。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| S1 生成（随机源） | 修得到 I-7.7、屏障那一格的混合；**打中「同参数两次 mkfs 逐字节相同」** | 量过 | 换 fsid 两次 mkfs 差 11 个扇区（系统配置 4、根环 3、单元区 4）；同参数 0 个。要保这条验收就得留一个注入 fsid 的口子，强制只管到生成那一个入口 |
| S1 生成（时间做前缀） | 打中（推的） | 推的 | 同一秒里两次 mkfs 前 8 字节相同，回到 `same` 那一形；前 8 字节必须取随机源 |
| S1 拒（读 [0, 根环基址) 的系统配置） | **打中：隔一代重用**；另有误拒原命令重跑 | 量过 | 池 A 用 X → 池 B 用 Y 盖上 → 新池又用 X：守卫读到的只有 Y、放行，做完 I-7.7 红（`aba`、`aba-used` 两份历史）；照原命令重跑 mkfs 在新系统配置落了的 65 / 31 195 态上被拒 |
| S1 别的落法：扫单元区单元头（实例代号 > 0 才拒） | 没打中 I-7.7；代价是每次 mkfs 读整个单元区 | 量过 | 六份历史上「守卫拒」与「做完 I-7.7 红」逐份一致；不被乙的作废掏空（作废之后 31 122 个拒态上照样认得出旧池单元）；原命令重跑一态都不误拒 |
| S1 扫单元头（不论实例代号） | 打中：误拒原命令重跑 | 量过 | 29 243 / 31 195 态上误拒（中断的那次 mkfs 自己写下的两个第 0 代单元） |
| S1 与乙的作废段怎么排 | 生成无约束；两种守卫都要排在作废之前 | 推的 | 守卫排在作废之后，拒的那一刻旧池已毁、新池没有 |
| S2 乙 + 生成：三段各落什么 | 没打中 | 量过（两套 crates） | 作废之前择到旧池 8 态全读回最后确认的一版；作废持久之后择到旧池 0、混合 0；新池 65 态 I-7.7 全绿；任一态上再跑一次 mkfs（又一个新 fsid）全部第 0 代空池、checker 绿 |
| S2 fsid 换了哪几格的结局 | 三格变好、一格随 crates 版本变 | 量过 | I-7.7 红 → 绿；屏障那一格混合 → 拒或旧池；今天的 crates 上「一块盘缺新系统配置」的 40 态可写挂载由拒变成做成（同前缀旧单元在就拒，不在就补写） |
| S2 前 8 字节偶然相同 | 后果同 `prefix` 历史 | 量过 + 推的 | 均匀随机 16 字节，一对旧池 / 新池撞前 8 字节的概率 2⁻⁶⁴（推的）；撞上就是 I-7.7 红（量过） |
| S3 乙两趟写序、作废之后整池拒 | 对得上 | 量过（两套） | 段序列 `2+12+1+1+1+4`、31 195 态、拒 31 122 态与前两轮相同 |
| S3 附带一「一盘作废一盘没作废时旧池只读挂得上而可写挂载拒」 | **打中：今天的 crates 上不成立** | 量过 | 今天的 crates 补写被作废那块盘、可写挂载做成（`same`、`distinct` 两份历史各 8 / 8 态）；冻结副本上 4 / 8 拒。另：旧池从没发布过文件时只读挂载报 `Open(TreeTableHasNoEntries)` |
| S3 附带二「择系统配置取第一块可择盘那一份」 | 对得上 | 量过 | drift 历史择盘 0 那一份 |
| S3 U3 宽条文 X「可写挂载拒、只读照读」 | **打中：两半都与读数对不上** | 量过 | 只读挂载在全部新池态上报 `Open(TreeTableHasNoEntries)`（两套）；今天的 crates 上可写挂载在 fsid 前 8 字节不同的历史里 64 / 64 做成，同前缀旧单元在时 40 态拒 |
| S3 「不管」一句 | 对得上；第二轮判决「强制新 fsid 修到屏障那一格」比读数宽 | 量过（4 个手搭态） | fsid 不同时屏障那一格没有混合，但 4 态里 2 态挂上旧池（1 态择根 (1, 4)、只读挂载经 journal 读到 (1, 6)），不是整池拒 |
| S3 「跨盘世代号差大于 1 就拒」（第二轮我提的，零轮） | **撤回** | 量过（两套） | 两次崩在可写挂载里盘 1 写系统配置之前，两盘世代号差 2、之后每次挂载都保持差 2；池完好、checker 绿、两种挂载都做成——这条规则会永远整池拒一个好池 |

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-close-c656-r3-opus-model/`。两份冻结副本留在草稿目录（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，各 35 MB）：`~/.local/share/singlefs/work/m3-close-c656-r3-opus/frozen/`（对 `out/`）与 `~/.local/share/singlefs/work/m3-close-c656-r3-opus/frozen-live/`（对 `out-live/`）。

```
# 冻结副本那一套（tools/items.txt 对 out/）
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-c656-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c656-r3-opus/frozen --work <新的空草稿目录>
# 今天的 crates 那一套（tools/items-live.txt 对 out-live/）
RERUN_ITEMS=items-live.txt RERUN_OUT_SUBDIR=out-live bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-c656-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c656-r3-opus/frozen-live --work <又一个新的空草稿目录>
```

`rerun_steps`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只改了这一段）：拷冻结副本 → `tools/patch_copy.py` 打两处补丁（与第二轮攻方那一份逐字节相同：`crash.rs` 的逐态展开函数改 `pub`；`make_filesystem.rs` 加 `make_filesystem_with_order`，乙写序与屏障只到盘 0 那一支照第二轮）→ 拷原型 crate、`tools/add_probe_member.py` 加进工作区 → 经 8G 包装、`capped.sh 31` 编 release（`-j 8`）→ 照条目表逐行跑、只比 `name=` 开头的行（`name=done` 带挂钟，不比），打 `name=same` / `name=differs`。可选 `RERUN_SEED_TARGET=<编译目录>`：硬链接拷进来当种子，省掉 librocksdb-sys 的 C++ 现编（空编译目录现编要多大内存没量）。

整条复跑过一次（冻结副本那一套，种子取我自己的编译目录）：退 0、挂钟 304 秒，21 行全是 `name=same`（那时条目表 21 行；之后两份条目表各追加了两行手搭态，追加的四份产物是同一个二进制同一种起法出的，没再整条复跑）。今天的 crates 那一套没经 `rerun.sh` 整条复跑过（`out-live/` 是同一个原型在第二份冻结副本上编、同一种起法逐条跑出的），交主 agent。脚本自证 `bash tools/rerun.sh --selftest` 末行「✓ 自检通过（查了 5 格）」。

`probe/src/main.rs` 全长 sha256 `5d9e402869ff30c6ff086ff54643073a5a156fdb5be516b01543753cadb88867`，`tools/rerun.sh` `a9912551fd682c70d19b0cf621e21a71fa3a6d5e8f491de021b9db9f290f0b0c`，`tools/patch_copy.py` `de7d0978b4fa52330ab2078b2507c7d83834721a07c3de674a4bf07e0481add5`。各文件 sha256（`find . -type f | sort | xargs sha256sum`，前 16 位）：

| 文件 | sha256 前 16 位 |
|---|---|
| ./out/bytes.log | 60b0ff4385c8a580 |
| ./out/count-aba-used-yi.log | f2424bbecf01737a |
| ./out/count-aba-yi.log | 9066366e2a8e1981 |
| ./out/count-distinct-yi.log | f44fe4c28625127d |
| ./out/count-mixed-origin-yi.log | 516e51c6934659a1 |
| ./out/drift-2.log | f99446c8acee9a8f |
| ./out/drift-3.log | f2885e67a948493f |
| ./out/final-aba-used-yi.log | 70b55a3c572503c4 |
| ./out/final-aba-yi.log | f4ab7c2f5dc1f383 |
| ./out/final-distinct-yi.log | 22d14f8fffe8a5a3 |
| ./out/final-mixed-origin-yi.log | 6d9c4da26f5eecc7 |
| ./out/final-prefix-yi.log | 3957568ca1d3436b |
| ./out/final-same-yi.log | 2280960d67f6a090 |
| ./out/handbuilt-distinct-d0-up-to-roots.log | dd8ca2ed2ea7e0ef |
| ./out/handbuilt-distinct-d1-cache-lost.log | 90f135e21cde28a8 |
| ./out/handbuilt-distinct-d1-everything-but-invalidation.log | 11475572006f40af |
| ./out/handbuilt-distinct-d1-rings-without-invalidation.log | 60a036dac7db05be |
| ./out/handbuilt-distinct-yi-d0-invalidated.log | 0a80b51cf71bdec8 |
| ./out/handbuilt-same-yi-d0-invalidated.log | 78b44ebd656da7fb |
| ./out-live/bytes.log | 60b0ff4385c8a580 |
| ./out-live/count-aba-used-yi.log | 1276c877e9ca07c3 |
| ./out-live/count-aba-yi.log | 9066366e2a8e1981 |
| ./out-live/count-distinct-yi.log | 2bd4613b8e014f8b |
| ./out-live/count-mixed-origin-yi.log | 516e51c6934659a1 |
| ./out-live/drift-2.log | 1f6b7b5f7bd8fdf8 |
| ./out-live/drift-3.log | c53f06415837d057 |
| ./out-live/final-aba-used-yi.log | 5ce75eb89b56b3e8 |
| ./out-live/final-aba-yi.log | f4ab7c2f5dc1f383 |
| ./out-live/final-distinct-yi.log | c1bef0b7eaa4dd75 |
| ./out-live/final-mixed-origin-yi.log | 6d9c4da26f5eecc7 |
| ./out-live/final-prefix-yi.log | 407838acab1a1968 |
| ./out-live/final-same-yi.log | 75a632d975d0ad94 |
| ./out-live/handbuilt-distinct-d0-up-to-roots.log | 4cec89c4ff760487 |
| ./out-live/handbuilt-distinct-d1-cache-lost.log | 8c92b8f3e34d6b31 |
| ./out-live/handbuilt-distinct-d1-everything-but-invalidation.log | 1690075ac86df78b |
| ./out-live/handbuilt-distinct-d1-rings-without-invalidation.log | 02b587d911e4c783 |
| ./out-live/handbuilt-distinct-yi-d0-invalidated.log | b4e8c98da3c10879 |
| ./out-live/handbuilt-same-yi-d0-invalidated.log | 94ba671166ae85c8 |
| ./out-live/run-aba-used-yi.log | 3a59df1b37b3fd22 |
| ./out-live/run-aba-yi.log | 26237c8cba3b329b |
| ./out-live/run-distinct-yi.log | 4a176b91d6cd5968 |
| ./out-live/run-mixed-origin-yi.log | 6602eb9203bdb770 |
| ./out-live/run-same-yi.log | f6f04ec15c2cb68d |
| ./out/run-aba-used-yi.log | 77427ffa9527c0fa |
| ./out/run-aba-yi.log | 539397bfe817f9a7 |
| ./out/run-distinct-yi.log | 02336509876b3859 |
| ./out/run-mixed-origin-yi.log | 86d01f526a2a91e3 |
| ./probe/Cargo.toml | e5083606b9e9d8d3 |
| ./probe/src/main.rs | 5d9e402869ff30c6 |
| ./tools/add_probe_member.py | 5836beb6b6ee4c40 |
| ./tools/items-live.txt | 91b835607310dbc5 |
| ./tools/items.txt | 2bdc3336e188e15d |
| ./tools/patch_copy.py | de7d0978b4fa5233 |
| ./tools/rerun.sh | a9912551fd682c70 |
| ./tools/summarize.py | 47e557c683449d70 |

## 原型怎么取样（第 3b、3c 步）

- 历史与崩溃态的取法照第二轮攻方原型：盘是内存稀疏盘（`SparseBlockDevice`，每块 2 799 槽），每份历史的起点只造一次，崩溃态从内存里那一份拷；新池 io_min 512、fsid `c656-r2-new-pool`。崩溃态走 `visit_crash_state_range_of_the_read_back_flow_with_slices`（副本里改 `pub`），展开取 `full_expansion`（每段任意真子集、原地覆写带撕裂镜像），每个状态恢复之后都跑池级 checker。没走 `enumerate_crash_states` 本身，理由同第二轮：它要一个判的根与文件版本表、经判定库录入，mkfs 这条流没有文件版本（推的，没试）。
- 历史：沿用第二轮的 `same`、`prefix`、`distinct`、`mixed-origin`；新造两份：`aba`（池 A 用 fsid X = `OLDPOOL!distinct` 用到 u = 5 → 池 B 用 Y = `BBPOOL!!between!` 在同两块盘上照今天的写序 mkfs、不再用 → 新 mkfs 又用 X）、`aba-used`（同上，池 B 另用到 u = 3）。另两个不是枚举的模式：`drift`（一个用过的池，每一轮录一次可写挂载、崩在录制流里盘 1 第一次写系统配置之前，即流的一个前缀）、`bytes`（两对全新的盘上各 mkfs 一次，逐 512 字节扇区比）。
- 每个状态多算三样（`landings`）：「生成」= 在这个态上再跑一次乙写序的 mkfs，fsid 取又一个新的 `R3REGEN!fresh-n2`，记恢复与 checker；「拒」= 第二轮那个守卫（每盘 [0, 根环基址) 逐 4 096 字节试解系统配置，前 8 字节相同就拒），对新池 fsid 与旧池 fsid 各判一次；「单元头」= 逐槽扫两盘单元区，照 checker 认单元写序的写法（`crates/singlefs-checker/src/walk.rs` 的 `unit_write_order_instance`：魔数、类标签、头校验和、fsid 前 8 字节）认头，数实例代号 > 0 与 = 0 的个数。
- 每次跑之前用 `crash_state_count_with_torn_in_place_overwrites` 算全量（`count-*` 产物）：`distinct` 31 195、`mixed-origin` 31 168、`aba` 13 915、`aba-used` 31 195、`same` 31 195。都跑全，没缩：试跑 1 000 态 1.6 秒（31 线程），按它估四份共约 3 分钟。跑完的态合计：冻结副本 1 000 + 107 473（正式）+ 107 473（`rerun.sh` 整条复跑）；今天的 crates 107 473 + 31 195（`same`）；共约 35 万态，单次都不超过 10⁶，合计远低于 10⁷。另有手搭的单态 6 个（`handbuilt`，不是枚举）。屏障只到盘 0 的写序一段 21 写、全量 181 398 528 态，不跑（第 3b 步），只复用第二轮的 4 个手搭态。
- 编译：冻结副本那一份拷了第二轮主 agent 复跑留下的编译目录（`cp -a`，同一份 crates，librocksdb-sys 没重编），经 8G 包装 38 秒编完；今天的 crates 那一份硬链接拷它当种子，32 秒。开跑时 `ps` 看到别的会话的变异轻型、崩溃点重放与 `rustc` 在跑，没有性能测量；资源提醒报本机 loadavg 103、可用 0 线程，照常 `nice -n 19` 跑，今天的 crates 那几份因此慢一倍多（`distinct` 95 秒对 38 秒）。
- `rerun.sh` 整条复跑与今天的 crates 四份枚举两次超过前台 240 秒，被工具挪到后台跑完（不是我起的后台命令）；等的是完成通知，没有轮询。

## S1　「强制新 fsid」三种落法

今天 fsid 是调用方给的：「/// mkfs 的参数：fsid 与时间戳都是参数，同参数两次 mkfs 逐字节相同（里程碑步 1 验收）。」（`crates/singlefs-core/src/make_filesystem.rs:66`），注释写死「mkfs 自己不生成」：「    // 而 fsid 是调用方给的（`MakeFilesystemParameters::filesystem_identifier`，mkfs 自己不生成）⇒」（`crates/singlefs-core/src/make_filesystem.rs:532`）。顺带：第 66 行说「时间戳」也是参数，而参数结构只有 fsid、区域归属、几何三样（同一文件第 70–74 行）；时间戳是发布第一个文件时的参数，不是 mkfs 的。改 fsid 的来源时这一句要一起改。

### 生成：mkfs 内部生成

写成改动（推的，没在副本上实现）：`MakeFilesystemParameters` 去掉 `filesystem_identifier`，`make_filesystem` 开头从随机源取 16 字节，经 `MakeFilesystemOutput` 交回；可写挂载要调用方给全参数（fsid 在比对的字段里：「/// `MakeFilesystemParameters` 里的一项，按系统配置字段表的次序（`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`）。」（`crates/singlefs-core/src/mount.rs:683`）），调用方要从交回值或盘上（`recovery::choose_system_configuration` 是 pub）取 fsid。`singlefs-core` 今天没有随机源（`Cargo.lock` 里的 getrandom 只经构建期的 jobserver 进来，推的，读表）。

- **修 I-7.7（量过）**：新 fsid 前 8 字节与盘上每一代旧池都不同的两份历史（`distinct`、`mixed-origin`），mkfs 做完池级 checker 全绿；与某一代旧池相同的四份（`same`、`prefix`，与 `aba`、`aba-used` 里的池 A）全红。原样（各份 `out/final-*-yi.log` 第 2 行，截到 `raw_red` 为止）：`same` 「raw_red=["I-7.7"]」、`prefix` 「raw_red=["I-7.7"]」、`aba` 「raw_red=["I-7.7"]」、`aba-used` 「raw_red=["I-7.7"]」、`distinct` 「raw_red=[]」、`mixed-origin` 「raw_red=[]」。
- **修屏障只到一块盘那一格的混合（量过，4 个手搭态）**：见 S3「不管」一句；没有混合，但不全是整池拒。
- **重跑不受影响（量过）**：四份历史、两套 crates 上每一个崩溃态再跑一次 mkfs（又一个新 fsid）全部是 `regen_rerun=NoFile(i0t0)|[]`（`out/run-*-yi.log`、`out-live/run-*-yi.log` 的 `name=sig` 行，没有一行例外）。中断那次写下的第 0 代单元带着上一个 fsid 的前缀，checker 不认它们。
- **打中「同参数两次 mkfs 逐字节相同」（量过）**：「name=bytes compare=same_parameters differing_sectors_by_area={} total=0」（`research/prompts/m3-close-c656-r3-opus-model/out/bytes.log:1`），「name=bytes compare=other_fsid differing_sectors_by_area={"root_ring_area": 3, "system_configuration_area": 4, "unit_area": 4} total=11」（`research/prompts/m3-close-c656-r3-opus-model/out/bytes.log:2`）。fsid 进了系统配置、根记录与单元头，连带各自的校验和；不是「比较时去掉 fsid 那几个字节」修得掉的。
- 要保这条验收，只有留一个注入 fsid 的入口（同参数同 fsid 两次 mkfs 逐字节相同）。那样「强制」只管到生成的那一个入口，注入口照样能同 fsid 重用。离不开注入口的（推的，读码）：
  - C484 的回归用例拿同一个 fsid 在用过的盘上重做：「        make_filesystem(&parameters, &mut devices).expect("同一个 fsid 在用过的盘上重做 mkfs");」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:538`）；
  - C651 的回归用例同样：「    .expect("同一个 fsid 再 mkfs 一次 io_min 8192 的池");」（`crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs:170`）。用户定「清根环照留」，而清根环这件事只在同 fsid 时才有可观测的差别（根记录按整 16 字节比 fsid）：没有注入口，它的必红形式就没了；
  - 钉整盘 CRC 的用例「const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 1_715_986_665;」（`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:183`）、E142 装置那个固定 fsid「pub const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [」（`crates/singlefs-harness/src/scenario.rs:35`）。`crates/` 里调 `make_filesystem(` 的文件 94 个（测试目录下 70 个、别处 24 个），`filesystem_identifier: *b…` / `[…]` 字面量 31 处、19 个文件（grep 数的）。
  - 崩溃点重放的录制流从 mkfs 起，判定库按流复用判定；不走注入口，每次 mkfs 流的字节都不同，复用全失（推的）。
- **时间做生成源（推的，打中）**：fsid 前 8 字节取时间（秒或更粗），同一秒里在同两块盘上做两次 mkfs（脚本里「建、发现参数错、再建」）前缀相同，回到 `same` 那一形（I-7.7 红，量过的那一份）。条文要写「前 8 字节取随机源」，不只写「生成」。
- 与乙的作废段：fsid 在第一个写之前定下，与作废段没有先后约束；中断后每次重跑都换一个新 fsid（量过：重跑全绿）。

### 拒：读盘上已有的 fsid，前 8 字节相同就拒

写成改动（推的；原型里实现了守卫本身）：mkfs 第一个写之前读每盘 [0, 根环基址)，试解系统配置，fsid 前 8 字节与要用的相同就报错退出。第二轮量过它被作废掏空（不重复）。这一轮打中两处别的：

**一、只看得到上一个池（量过）。** `aba`：池 A 用 X 用到 u = 5，池 B 用 Y 在同两块盘上 mkfs（今天的写序：清 [0, 根环基址)、根环、journal 环，不碰单元区），新 mkfs 又用 X。守卫读到的只有 B 的系统配置，放行；A 留在单元区的 80 个 X 前缀、实例代号 1 的单元原样在，做完 I-7.7 红。原样（`research/prompts/m3-close-c656-r3-opus-model/out/final-aba-yi.log` 第 2 行开头）：「name=final h=aba order=yi before_mkfs:sysconf_guard[new]=allow/1 before_mkfs:unit_scan[new]=80/0」，同一行「raw_red=["I-7.7"]」。`aba-used`（B 另用到 u = 3，按分配次序盖掉 A 的一半单元）还剩 40 个、同样红（`out/final-aba-used-yi.log` 第 2 行「before_mkfs:unit_scan[new]=40/0」）。崩溃态里（`out/run-aba-yi.log`）：新池两盘都有系统配置的 25 态全红 I-7.7。用户为什么会隔一代重用：按池名派生 fsid、两套配置轮流重建（推的，用户动作，没写死——这一形不要崩溃，一次干净的 mkfs 就到）。
**二、误拒原命令重跑（量过）。** 新系统配置落了一份以上的态（`distinct` 65 态、`mixed-origin` 46 态）上，照原命令（同一个 fsid）重跑被拒：`out/run-distinct-yi.log` 里 `choice=new@…` 的 `name=sig` 行全是「sysconf_guard[new]=refuse」。这些态 mkfs 没有交回成功，用户本来就该重跑；守卫分不出「自己中断的那一次」与「另一个同前缀的池」。这一格是可用性的代价，不是对错。

- 修 I-7.7：只修上一个池同前缀那一形（量过：`same`、`prefix` 起点拒），漏隔一代那一形（量过）。
- 屏障那一格：上一个池同前缀时起点就拒，这一形出不来（推的）。
- 与验收：fsid 仍是参数，没有冲突；但 C484、C651 两条回归用例的第二次 mkfs 会被拒（推的，读码，同上两行），要绕过守卫或手写旧字节。
- 与乙的作废段：守卫只能排在作废之前（作废抹的正是它要读的 [0, 根环基址)）。

### 别的落法：扫单元区单元头里的 fsid 前缀

写成改动（推的；原型里实现了守卫要读的东西）：mkfs 第一个写之前逐槽读两盘单元区的单元头，照 checker 认单元写序的同一种写法认头（头校验和过、fsid 前 8 字节与要用的相同），有实例代号 > 0 的就拒。I-7.7 ① 的射程正是「fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者」（`crates/singlefs-checker/invariants/02-根环.md:14`），mkfs 清根环与 journal 环、自己写的单元实例代号是 0，所以做完会不会红只看单元区。checker 那一侧逐槽扫的写法：「                Box::new(geometry.unit_area_start_slot..end)」（`crates/singlefs-checker/src/walk.rs:4855`）。

- **修 I-7.7（量过）**：六份历史上「起点扫到实例代号 > 0 的同前缀单元」与「做完 I-7.7 红」逐份一致：`same` 80 / 红、`prefix` 80 / 红、`aba` 80 / 红、`aba-used` 40 / 红、`distinct` 0 / 绿、`mixed-origin` 0 / 绿（各份 `out/final-*-yi.log` 的 `before_mkfs:unit_scan[new]=` 与 `raw_red=`）。
- **不被乙的作废掏空（量过）**：`distinct` 31 195 态、`mixed-origin` 31 168 态上，对旧池 fsid 的扫描每一态都扫到实例代号 > 0 的单元（`name=sig` 行全是 `unit_scan[old]` 前一个数 > 0），作废之后的 31 122 个拒态也一样——作废只抹 [0, 根环基址)，不碰单元区。
- **不误拒原命令重跑（量过）**：对新池 fsid，四份历史里新池自己的单元实例代号都是 0（`distinct`、`mixed-origin` 每一态 `unit_scan[new]` 前一个数都是 0）。若改成「不论实例代号、有同前缀单元就拒」，`distinct` 里 29 243 / 31 195 态误拒（65 个新池态 + 29 178 个别的态都扫到中断那次写下的第 0 代单元），`mixed-origin` 29 216 / 31 168。
- 代价（推的）：每次 mkfs 读整个单元区的头，读量随盘容量涨，与第二轮甲七「写满单元区」同一个量级的 I/O（读不是写）；读故障时放不放行条款要写（第二轮对「拒」已提过同一问，不重复）。今天的 crates 里已经有一个逐槽扫单元区按前缀认实例代号的读法（`system_configuration_repair::write_order_and_birth_in_the_unit_area_of_the_device`，丙' 补写闸第四件），可以共用（推的）。
- 屏障那一格：同前缀而单元区没有实例代号 > 0 的单元（旧池只做过 mkfs、没写过东西）时放行，这一形屏障只到一块盘还能混合；混的是两个第 0 代空池（推的，没量）。
- 与验收：fsid 仍是参数，没有冲突；C484 那条回归用例的旧池跑完了新池新建文件（有实例代号 1 的单元），第二次 mkfs 会被拒（推的）。
- 与乙的作废段：要排在作废之前——排在之后，拒的那一刻旧池的系统配置已经抹了、新池又不写，盘上两个池都没有（推的）。

### 打中之后的四句

**拒漏隔一代重用（`aba`）。**
1. 分辨臂：分辨。同一段历史上「拒」放行、做完红；「单元头」起点拒；「生成」用户给不出 X，重跑全绿（三样都量过）。
2. 被判的系统当时看不看得到：看得到——A 的 80 个同前缀、实例代号 1 的单元就在单元区（`unit_scan` 量到）；「拒」只读 [0, 根环基址)，那里只有 B 的系统配置。
3. 满足的是字面哪一分句：I-7.7 ① 那一句里的「单元写序」；以及用户定的「不许同 fsid 重用」——字面没限定「上一个池」。
4. 改法在这一格上还中不中：「生成」不中（量过），「单元头」不中（量过），「拒」照中（量过）。

**拒误拒原命令重跑。**
1. 分辨臂：分辨「拒」与「单元头」（后者 0 态误拒，量过）；「生成」没有这一形。
2. 看不看得到：看得到的是一份世代 1、实例代号 0 的新系统配置；要分出「自己中断的那一次」得另加条件（例如世代号是 1 且根环里只有第 0 代根就放行），那是新改法，被攻过零轮。
3. 字面哪一分句：不违反任何条款；是可用性代价。可以读成「打中归错了判据」——它不是对错格。
4. 改法：「单元头」不中，「生成」不中（量过）。

**生成与「同参数两次 mkfs 逐字节相同」。**
1. 分辨臂：分辨（「拒」「单元头」fsid 仍是参数，没有冲突，推的）。
2. 看不看得到：不适用（验收对的是 mkfs 的输出字节）。
3. 字面哪一分句：「同参数两次 mkfs 逐字节相同（里程碑步 1 验收）」（`crates/singlefs-core/src/make_filesystem.rs:66`）——fsid 不再是参数之后，「同参数」两次的 fsid 不同，11 个扇区不同（量过）。
4. 改法：留注入口、验收改成「同参数同 fsid」可以过，但「强制」从此只管生成那一个入口，C484、C651 两条回归用例、钉值用例与崩溃点重放都走注入口（推的）。

### 几个改法各修哪一格

「生成」取随机源；「单元头」只在实例代号 > 0 时拒。`量过` 指本轮或第二轮的副本产物（括注文件），`推的` 指按代码推、没实现没跑。

| 格 | 乙 + 生成 | 乙 + 拒 | 乙 + 单元头 |
|---|---|---|---|
| mkfs 做完 I-7.7，上一个池同前缀（`same`、`prefix`） | 不会出现：用户给不出那个 fsid；重跑全绿（量过，`regen_rerun`） | 起点拒（量过，`before_mkfs:sysconf_guard[new]=refuse`） | 起点拒（量过，`unit_scan` 80） |
| mkfs 做完 I-7.7，隔一代同前缀（`aba`、`aba-used`） | 不会出现（推的）；重跑全绿（量过） | **放行、做完红（量过）** | 起点拒（量过，80 / 40） |
| 作废之后整池拒、用旧池 fsid 重跑 | 不会出现（推的） | 放行、红（第二轮量过） | 拒（量过：作废之后每一态都扫到旧单元） |
| 中断之后照原命令重跑 | 换新 fsid，全绿（量过） | **新池态上误拒（量过，65 / 46 态）** | 放行（量过，0 态误拒） |
| 屏障只到一块盘，同前缀混合 | 没有混合，2 / 4 手搭态挂上旧池、读回最后确认的那一版的字节（量过） | 上一个池同前缀时起点就拒，出不来（推的） | 旧池写过东西时起点就拒；只做过 mkfs 的同前缀旧池还能混合（推的） |
| 同参数两次 mkfs 逐字节相同 | **11 个扇区不同（量过）**；留注入口才过（推的） | 不冲突（推的） | 不冲突（推的） |
| C484、C651 两条同 fsid 回归用例 | 要注入口（推的） | 第二次 mkfs 被拒，要绕守卫（推的） | C484 那条被拒（推的） |
| 每次 mkfs 多读多少 | 0（推的） | 每盘 [0, 根环基址)（推的） | 两盘整个单元区的头（推的） |
| 读故障 | 不读盘（推的） | 条款要写（第二轮提过） | 条款要写（推的） |

## S2　乙 + 强制新 fsid 的合成形态

「强制新 fsid」下新池的 fsid 与盘上旧池前 8 字节相同只剩撞上的概率，第二轮四份历史里能出现的只有 `distinct` 与 `mixed-origin`（新 fsid `c656-r2-new-pool` 与两个旧池前 8 字节都不同）；再加本轮的 `aba`、`aba-used`（新池用的 X 与池 A 相同，在「生成」下用户给不出 X，那两份只用来看守卫与重跑）。每一态上「整池拒之后再跑 mkfs」都放开：再跑一次乙写序的 mkfs，fsid 取又一个新的。

`distinct` 两套 crates 的原样（`tools/summarize.py <产物> phase choice verdict writable raw_red regen_rerun`）：

```
$ python3 tools/summarize.py out/run-distinct-yi.log phase choice verdict writable raw_red regen_rerun
# count phase choice verdict writable raw_red regen_rerun  (total 31195)
1 all new new_complete Ok(i0t0) [] NoFile(i0t0)|[]
4 seg0 old old_readback_intact Err:SystemConfigurationUnreadableAfterTheReadStage [] NoFile(i0t0)|[]
4 seg0 old old_readback_intact Ok(i1t6) [] NoFile(i0t0)|[]
31103 seg1 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
1 seg2 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
1 seg3 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
1 seg4 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
40 seg5 new new_complete Err:SystemConfigurationUnreadableAfterTheReadStage [] NoFile(i0t0)|[]
24 seg5 new new_complete Ok(i0t0) [] NoFile(i0t0)|[]
16 seg5 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
$ python3 tools/summarize.py out-live/run-distinct-yi.log phase choice verdict writable raw_red regen_rerun
# count phase choice verdict writable raw_red regen_rerun  (total 31195)
1 all new new_complete Ok(i0t0) [] NoFile(i0t0)|[]
8 seg0 old old_readback_intact Ok(i1t6) [] NoFile(i0t0)|[]
31103 seg1 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
1 seg2 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
1 seg3 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
1 seg4 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
64 seg5 new new_complete Ok(i0t0) [] NoFile(i0t0)|[]
16 seg5 Err:NoValidSystemConfiguration refused not_tried ["I-8.5"] NoFile(i0t0)|[]
```

三段各落什么（量过）：

| 段 | `distinct` | `mixed-origin` | I-7.7 |
|---|---|---|---|
| 作废之前（第 0 段） | 8 态择到旧池、8 / 8 读回最后确认的一版 (1, 6)；可写挂载：冻结副本 4 拒 4 做成，今天的 crates 8 / 8 做成 | 8 态：旧池 P 读回对 3、整池拒 2、择到池 Q 读回旧一版 3（第二轮那一形，不重复） | 第 0 段读回旧池的态 checker 绿（`distinct`） |
| 作废持久之后 | 择到旧池 0、混合 0；整池拒 31 122；新池 65（第 0 代空池） | 择到旧池 0、混合 0；整池拒 31 114；新池 46 | 新池态全绿（两份、两套） |
| 任一态上再跑 mkfs（新 fsid） | 31 195 / 31 195 第 0 代空池、checker 绿 | 31 168 / 31 168 同 | 全绿 |

`aba`、`aba-used` 两份在「生成」下的重跑同样全绿（13 915 / 13 915、31 195 / 31 195，两套）：池 A 的 X 前缀单元还在单元区，新 fsid 不认它们。

### fsid 变了，哪几格换了结局（量过）

- **I-7.7**：做完 mkfs，同前缀（`same`）红、不同前缀（`distinct`）绿；崩溃态里新池两盘都有系统配置的 25 态同样红 / 绿两分（`out-live/run-same-yi.log` 对 `out-live/run-distinct-yi.log`，上面那两段原样）。
- **屏障只到一块盘**：同前缀时那一态新旧混合、可写挂载做成（第二轮量过）；不同前缀时同一态整池拒（`out/handbuilt-distinct-d1-cache-lost.log`，本轮两套都复跑一样）。
- **今天的 crates 上「一块盘缺新系统配置」那 40 态的可写挂载**：同前缀旧单元在（`same`、`aba`、`aba-used`）时 40 态拒 `SystemConfigurationUnreadableAfterTheReadStage`；不同前缀（`distinct`）时 0 态拒、64 / 64 做成。冻结副本上两种都是 40 态拒。原因（读码）：今天的 crates 可写挂载对两槽全废的盘过一道补写闸，第四件扫这块盘的单元区：「有写序实例大于所选那一版实例的单元就不补写（闸四，零轮改法，用户 2026-10-20 采）」（`crates/singlefs-core/src/mount.rs:5845`）——按本池 fsid 前 8 字节认单元；同前缀的旧单元实例代号 1 > 0，闸不过。这一格在冻结副本上不存在，是这一轮当中别的会话合进来的行为（那一段注释从「/// 丙' 补写闸」（`crates/singlefs-core/src/mount.rs:5838`）起）。
- 没有一格因为 fsid 变了而变坏（四份历史、两套 crates，量过）。

### 前 8 字节偶然相同

16 字节均匀随机时，新池与盘上某一代旧池前 8 字节撞上的概率是 2⁻⁶⁴（推的；盘上有 k 代旧池时 k × 2⁻⁶⁴）。撞上就是 `prefix` 那一形：做完 I-7.7 红（量过，`out/final-prefix-yi.log`），屏障那一格整池拒而不是混合（16 字节不同，跨盘比 fsid 拒，第二轮量过），今天的 crates 上那 40 态可写挂载拒（推的：补写闸只看前 8 字节，与 `same` 同；`prefix` 的崩溃态本轮没在今天的 crates 上枚举）。可写挂载里另有一处按前 8 字节认「本池更新实例的单元」（「有一个指针两份都已被本池更新实例的单元盖掉（单元头里的 fsid 与本池的相同才算」（`crates/singlefs-core/src/mount.rs:2115`）），撞上时旧池单元会被当成本池的（推的，没造那一形）。

## S3　条文与前两轮读数

### 乙的条文：两趟写序、作废之后整池拒（对得上）

两套 crates 上段序列、态数、拒态数都与前两轮相同：「segment_sizes=[2, 12, 1, 1, 1, 4] closed_form_two_state=4117 states_with_torn=31195」（`research/prompts/m3-close-c656-r3-opus-model/out-live/count-distinct-yi.log:1`）；作废持久之后择到旧池 0、混合 0、拒 31 122（上面两段原样）。

### 附带一「一盘作废一盘没作废时旧池只读挂得上而可写挂载拒」：今天的 crates 上不成立（量过）

手搭一态：只落了盘 0 的 [0, 根环基址) 清零（写下标 0）。冻结副本：「name=handbuilt h=distinct order=yi landed=[0]…writable=Err:SystemConfigurationUnreadableAfterTheReadStage」（`research/prompts/m3-close-c656-r3-opus-model/out/handbuilt-distinct-yi-d0-invalidated.log:2`）；今天的 crates：「name=handbuilt h=distinct order=yi landed=[0]…read_only=Ok(i1t6) writable=Ok(i1t6) verdict=old_readback_intact」（`research/prompts/m3-close-c656-r3-opus-model/out-live/handbuilt-distinct-yi-d0-invalidated.log:2`），`same` 同（`research/prompts/m3-close-c656-r3-opus-model/out-live/handbuilt-same-yi-d0-invalidated.log` 第 2 行）。枚举里第 0 段 8 态：今天的 crates 上 `distinct`、`same`、`aba`、`aba-used` 四份都是 8 / 8 可写挂载做成；`mixed-origin` 3 个择到池 P 的态照旧拒（盘 1 是池 Q 的盘，补写闸第一件「本盘带所选那一版的全部单元」不过，推的读码）。

- 结局本身不坏：可写挂载补写被作废那块盘的两槽，旧池恢复成两盘都有系统配置，读回的是最后确认的一版——这正是乙那一句「作废持久之前照没跑过 mkfs 处置」。错的是条文把冻结副本上的一个观测写成了规定。
- 另一半「只读挂得上」也有例外（量过，两套）：旧池从没发布过文件（`aba` 的池 B 只做过 mkfs）时只读挂载报 `Err:Open(TreeTableHasNoEntries)`，`out/run-aba-yi.log` 第 0 段 8 态全是这样，恢复照样读回 B 的第 0 代空池。

打中之后的四句：
1. 分辨臂：不分辨乙的写序（同一条写序，结局随 crates 版本变）；分辨的是两版 crates。
2. 看不看得到：看得到——可写挂载那一刻盘 0 两槽全 0、盘 1 两槽是本池的，补写闸逐件查得到。
3. 字面哪一分句：附带一的「可写挂载拒」；今天的 crates 是「可写挂载做成（过补写闸时）」。也可以读成「打中归错了判据」：这一句本来就不该进 C656 的条文，它是系统配置那一轮（丙'）的射程。
4. 改法：条文改写成「一盘作废一盘没作废时照没跑过 mkfs 处置：恢复读回最后确认的一版；可写挂载照 D22 已定项 8 射程现行那一句处置（今天的 crates 过补写闸就补写）」——零轮措辞，只在我的读数上对过。

### U3 宽条文 X「可写挂载拒、只读照读」：两半都对不上（量过）

- **只读**：两套 crates 上全部新池态（`distinct` 65、`mixed-origin` 46、`aba` 65、`aba-used` 65、`same` 65）只读挂载都是 `Err:Open(TreeTableHasNoEntries)`，包括两盘系统配置齐全、可写挂载做成的那几态；`final` 全落那一态也一样（`out/final-distinct-yi.log` 第 2 行「read_only=Err:Open(TreeTableHasNoEntries)」）。这是只读挂载自己的规定，不是原型的限度：「            // 树表 0 条不是走读失败（这一版还没发布过文件版本），不退。」（`crates/singlefs-core/src/mounted_read.rs:647`）。**我在第二轮报告里写「原型的只读挂载要打开一个文件」是错的，撤回**；第二轮辩方表里「空池没有文件可开」是对的。
- **可写**：冻结副本上 `distinct` 40 态拒、24 做成（第二轮读数）；今天的 crates 上 `distinct` 64 / 64 做成、`mixed-origin` 45 / 45 做成；同前缀旧单元在时（`same`、`aba`、`aba-used`）仍是 40 态拒。在「生成」下能出现的只有前一种，所以合成形态下「一块盘缺新系统配置」那批态在今天的 crates 上可写挂载做成。
- 能写死的只有恢复那一半：65 / 46 态恢复都读回第 0 代空池 `NoFile(i0t0)`、池级 checker 绿（两套）。

打中之后的四句：
1. 分辨臂：不分辨乙的写序；只读那一半与 crates 版本无关，可写那一半随 crates 版本与 fsid 前缀变。
2. 看不看得到：看得到（树表 0 条、两槽全废都是挂载那一刻读得到的）。
3. 字面哪一分句：X 的「（可写挂载在取号那一步拒，读照读）」两个分句。
4. 改法：X 只写「恢复读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）；挂载照 D22 已定项 8 射程现行那几句处置」，不写挂载结局——零轮措辞。D22 已定项 8 射程里「某块盘一份自证过的系统配置都没有时，取号那一步也拒（用户 2026-10-03 定，原话在变更史）。」（`.claude/kb/decisions/22-单元原子性怎么合成.md:180`，已定项 8 射程那一行）与今天的 crates 也对不上（今天过补写闸就补写），那是系统配置那一轮的写回，不归这一题（推的，没查那一轮的判决）。

### 「不管」一句：对得上；第二轮判决的一句比读数宽（量过）

fsid 前 8 字节不同时屏障只到盘 0 的四个手搭态（两套 crates 择池与读回相同；「盘 1 只落了根环清零」那一态的可写挂载今天的 crates 上做成）：盘 1 缓存全丢 → 整池拒（跨盘 fsid 不同）；盘 1 只落了根环清零 → 择到旧池、读回 (1, 6) 对；盘 1 除作废外全落 → 择到旧池、`NoValidRoot`；盘 0 落到根为止 → 择到旧池、择根 (1, 4)、只读挂载经 journal 读到 (1, 6)（「choice=old@4096g8 root=i1t4 recover=FileRead(i1t4):old_last_other_root」（`research/prompts/m3-close-c656-r3-opus-model/out/handbuilt-distinct-d0-up-to-roots.log:2`））。没有混合；但第二轮判决交用户表里「修到 I-7.7 与屏障只到一块盘那一格」读起来像「那一格变成整池拒」，读数是 4 态里 2 态挂上旧池（字节对）、1 态择到旧池的系统配置而没有根（等于拒）。条文写「混合消失」即可，「不管」一句照收。

### 「跨盘世代号差大于 1 就拒」（第二轮我提的，零轮）：撤回（量过）

每块盘写系统配置时世代号取本盘两槽最大号加 1：「        let slot_generation = verified_on_this_device」（`crates/singlefs-core/src/transaction.rs:592`），不看别的盘。所以只要某次轮换盘 0 那一写落了、盘 1 那一写没落，两盘就差 1，这个差之后不会自己收回；第二轮我写「两盘差最多一次轮换（推的，读码）」是错的。

`drift` 历史（流的前缀，任何崩溃枚举都含这一态）：一个用过的池，可写挂载里崩在盘 1 第一次写系统配置之前，挂载、再崩，两轮。原样：
- 冻结副本：「name=drift_end rounds=2 generations=["d0[10,9]", "d1[8,7]"] choice=d0g10 recover=FileRead(i1t6):old_last_same_root raw_red=[] read_only=Ok(i1t6) writable=Ok(i1t6) generations_after_one_more_writable_mount=["d0[12,13]", "d1[10,11]"]」（`research/prompts/m3-close-c656-r3-opus-model/out/drift-2.log:4`）
- 今天的 crates（取号写两轮）：「name=drift_end rounds=2 generations=["d0[10,11]", "d1[8,9]"] choice=d0g11 recover=FileRead(i1t6):old_last_same_root raw_red=[] read_only=Ok(i1t6) writable=Ok(i1t6) generations_after_one_more_writable_mount=["d0[14,15]", "d1[12,13]"]」（`research/prompts/m3-close-c656-r3-opus-model/out-live/drift-2.log:4`）
- 三轮差 3（`research/prompts/m3-close-c656-r3-opus-model/out/drift-3.log` 第 5 行）。

池完好：恢复读回最后确认的一版、池级 checker 绿、只读与可写挂载都做成；正常挂载一次之后差照旧是 2。这条规则会把这个池永远整池拒——拒的是一个好池里全部已确认的写。择系统配置取第一块可择盘（附带二）在这一形上择盘 0（世代号大的那块），与「取全池最大」一致。

打中之后的四句：
1. 分辨臂：分辨「加这条规则」与「不加」（不加：今天四样都绿，量过）。
2. 看不看得到：规则看得到的只有两盘世代号；「两次轮换各丢盘 1 那一写」与「屏障只到一块盘」在世代号上同形，判别子观测不到。
3. 字面哪一分句：用户 2026-10-20 定的前提「不接受挂上丢了已确认写的旧池」的反面——整池拒一个完好的池，丢的是全部可读性。
4. 改法：不加这条规则。屏障那一格在「生成」下已经没有混合（S3「不管」一句），这条规则没有剩下的靶子。

### 小节里的数能不能核

- 本轮的数都有产物与复跑命令（开头）。冻结副本那一套整条复跑过一次、21 / 21 same；今天的 crates 那一套没经 `rerun.sh` 整条复跑。
- 第二轮读数在冻结副本上逐字段对上（同一份 crates，第二轮攻方的 `out/run-distinct-yi.log` 与本轮 `out/run-distinct-yi.log` 按类数逐类相同：第 0 段 4 + 4、拒 31 122、新池 40 + 24 + 1；只读挂载那一列本轮多打了错误的内层成员）。今天的 crates 上世代号整体大 1（取号写两轮），去掉世代号之后只有可写挂载那一列在三处变了（附带一、X、屏障里「盘 1 只落了根环清零」那一态由拒变做成，`out-live/handbuilt-distinct-d1-rings-without-invalidation.log`）。

## 没打中的形状

| 形状 | 打的是谁 | 取样 | 结局 |
|---|---|---|---|
| 乙 + 生成，作废之前读回 | 合成形态 | `distinct`、`mixed-origin` 第 0 段，两套 | `distinct` 8 / 8 读回最后确认的一版；`mixed-origin` 只有第二轮那一形 |
| 乙 + 生成，作废之后择到旧池或混合 | 合成形态 | 四份历史 107 473 态 × 两套，另 `same` 31 195（今天的 crates） | 0 |
| 乙 + 生成，新池态 I-7.7 | 合成形态 | `distinct` 65、`mixed-origin` 46 态，两套 | 全绿 |
| 任一态上再跑 mkfs（新 fsid） | 合成形态 | 四份历史全部态，两套；`same` 一份（今天的 crates） | 全部第 0 代空池、checker 绿 |
| 单元头守卫被作废掏空 | 单元头那一种落法 | `distinct`、`mixed-origin` 全部态 | 每一态都扫到旧池单元 |
| 单元头守卫误拒原命令重跑（实例代号 > 0 才拒） | 同上 | 同上 | 0 态 |
| 屏障只到一块盘、前缀不同时的混合 | 合成形态 | 4 个手搭态，两套 | 0 |
| 「生成」在冻结副本与今天的 crates 之间变坏的格 | 合成形态 | 全部条目逐字段比（去掉世代号） | 没有；只有可写挂载由拒变做成 |

## 这条腿提的改法（只在我的模型上量过、被攻过零轮）

- 「生成」的条文写「fsid 的前 8 字节取随机源」，不只写「生成」（推的，时间源那一形没量）。
- 若要「拒」那一类，守卫读单元区单元头、只在实例代号 > 0 时拒（量过：六份历史上与「做完 I-7.7 红」逐份一致；没实现进 `make_filesystem`，只在原型里算了守卫要读的东西）。
- 「拒」守卫想不误拒原命令重跑，可以在「世代号 1、根环里只有第 0 代根」时放行（推的，没量）。
- 附带一与 X 只写恢复的结局，挂载照 D22 已定项 8 射程现行那几句（措辞见 S3，只对过我的读数）。
- 撤回第二轮我提的「跨盘世代号差大于 1 就拒」与「两盘差最多一次轮换」那句推测；撤回第二轮「原型的只读挂载要打开一个文件」那句解释。

## 这条腿自己的限度

- 几何同第二轮：u = 5、新池 io_min 512、journal 环 6 MiB、2 799 槽的小盘；`aba` 只有「B 不用」与「B 用到 u = 3」两种，B 用得比 A 多（盖掉 A 全部单元）那一种没造（推的：那时「拒」也就不漏了）。
- 今天的 crates 是干活当中从主工作区拷的第二份，主工作区当时有大量未提交的改动（别的会话在合入）；它之后再变，我的 `out-live/` 不跟着变。报告里引的 `mount.rs`、`recovery.rs`、`transaction.rs`、`walk.rs` 行号是写报告时主工作区的行号，那几份文件在我开工之后改过。
- 「生成」没实现进 `make_filesystem`：原型用「换一个不同前缀的 fsid」代替，随机源、交回 fsid、注入口都没写。时间源那一形、「前 8 字节撞上」的 `prefix` 崩溃态在今天的 crates 上都没枚举。
- 「单元头」守卫是我写的（头校验和照 checker 的写法），读故障、部分槽读不出都没造。
- 可写挂载里按前 8 字节认「本池更新实例的单元」那一处（C834 那一支）在同前缀旧单元下会怎样，没造那一形，只读了注释。
- 屏障只到一块盘仍只有第二轮的 4 个手搭态，全量 1.8 亿态没跑。
- 崩溃态取 `visit_crash_state_range_of_the_read_back_flow_with_slices`（副本改 `pub`），不是 `enumerate_crash_states` 本身，理由同第二轮。
- 只抽了一次样：这一份里「没打中」的格按规则不算没打中。

## 要主 agent 跑的命令

- 复跑两套（开头「复跑命令」两条）；今天的 crates 那一套我没整条复跑过。编译目录空着时 librocksdb-sys 要现编，8G 够不够没量；有一份编过的编译目录可设 `RERUN_SEED_TARGET`。
- 冻结副本留在 `~/.local/share/singlefs/work/m3-close-c656-r3-opus/frozen/` 与 `~/.local/share/singlefs/work/m3-close-c656-r3-opus/frozen-live/`，复跑要它们。

## 没做什么

- 没判辩方那几格，没替主 agent 采纳；副本上的数不是入库装置上的数。
- 没跑重型测试、没调 checker 档任何测试目标，轻型三条都没起（这一件不改 `crates/`）。
- 没实现任何一种落法进 `make_filesystem`；C484、C651 两条回归用例在三种落法下会怎样是读码推的，没跑。
- 草稿目录里留着两份冻结副本（复跑要它们，各 35 MB）、`runs/`、`live/runs/`、`verify/runs/`、`dev/`（原型开发用的脚本与几份手搭态输出）与编译日志；仓副本 `tree/`、`live/tree/`、`verify/tree/` 与编译目录 `target/`、`live/target/` 交回前删了。
- 模型目录里的 `tools/rerun.sh` 建好之后用 `scripts/kb/replace-once.py` 改过三处（加种子编译目录、两套条目表与产物目录），报告这一份也用它逐处改过措辞与引文写法（每次一处定点替换）；两份条目表各追加过两行。交回时的样子以开头的 sha256 为准（`tools/items.txt`、`tools/items-live.txt` 的 sha256 是追加之后算的）。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-c656-r3-opus-output.md --root . --background research/prompts/_m3-close-c656-r3-background.md`（写这一节之前跑的；这一节不含引文）：

```
  ✓ 核了 24 处引文、0 处裸行号，对上 24 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```

# m3-close-c656-r2 攻方腿（Opus）报告（2026-10-20）

攻击面：正文第三节 U2、U3、U4。所有数都是草稿目录仓副本上量的（副本数，不是入库装置上的数）。没读禁读清单里的辩方腿产物。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| U2 乙写成改动 | 照写（`tools/patch_copy.py` 的 `InvalidateFirst`） | 量过 | 段序列 `2+12+1+1+1+4`，带撕裂全量 31 195，与第一轮对上 |
| U2 两块盘来自两个池、槽距不同（mixed-origin） | **打中（字面）**，不分辨乙与今天 | 量过（31 168 态） | 起点两盘 fsid 不同、整池拒；作废之前 3 态择到盘 1 的池 Q，只读挂上 Q 的 (1, 4)，Q 最后确认的是 (1, 6) |
| U2 屏障只到一块盘 | **打中**（fsid 相同时新旧混合、可写挂载做成），不分辨乙与今天；fsid 前 8 字节不同时变成整池拒 | 手搭 12 态量过；全量 181 398 528 态没跑 | 盘 1 不认 flush、掉电丢光它的缓存：盘 0 新系统配置 + 盘 1 旧根 |
| U2 读故障、一块盘中途掉线 | 没打中 | 推的 | 乙的 mkfs 不读盘；掉线时 mkfs 报错停下，落盘的是流的一个前缀，结局在已枚举的态里 |
| U2 整池拒之后再跑 mkfs | 乙本身没打中；**打中丙七的「读旧池」守卫** | 量过（31 122 态） | 作废落了之后守卫读不到任何系统配置，拿旧池 fsid 重跑全部放行，做完 I-7.7 全红 |
| U2 附带两样写不写进条文 | 写；第二样另有一形（推的改法，被攻过零轮） | 量过 + 推的 | 跨盘只比 fsid 与设备数，世代 1 与世代 8 并存照收，屏障那一格的混合就出在这里 |
| U3「完整的新池」 | 给写法 | 量过 | 40 态：择到新池系统配置、恢复读回第 0 代空池、池级 checker 全绿、可写挂载拒；态数随几何变（mixed-origin 是 26） |
| U4 甲七 mkfs 清单元区 | 修 I-7.7（量过）；**错排就打中 C656 本身**（量过）；代价大 | 量过 | 排在作废屏障之后：280 027 态里新池态 checker 全绿；与作废同段：72 个择到旧池的态里 32 个读回坏、8 个可写挂载做成 |
| U4 乙七 I-7.7 ① 收窄 | **打中**：判别子观测不到 | 推的（读码） | 旧池同前缀的单元与新池自己崩溃留下的孤儿，checker 读到的字段逐条同形；收窄就让钉住的 ① 坏镜像判绿 |
| U4 丙七 不许同 fsid 重用 | 「读旧池拒」**被乙掏空**、且要比前 8 字节；「强制新 fsid」修得到 | 量过 + 推的 | 前 8 字节不同：乙做完 I-7.7 绿、屏障那一格变整池拒（量过）；前 8 字节相同而 16 字节不同：照红（量过） |
| 第一轮读数复核 | 第一轮第二次抽样「fsid 不同、槽距相同也红」的两个 fsid 前 8 字节相同 | 读码 + 量过 | 那份原型的 `c656-s2-pool-A!!` 与 `c656-s2-new-pool` 前 8 字节都是 `c656-s2-`；前 8 字节不同的历史上乙做完绿 |

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-close-c656-r2-opus-model/`。冻结副本是开工时从主工作区拷的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，在 `~/.local/share/singlefs/work/m3-close-c656-r2-opus/frozen/`（`crates` 下 `.rs` 全部文件 sha256 再取 sha256：`3dc1ba5195ee541b85c7248f6ceeba3f82026f17a8ab6b382463688b9fc6ad8b`；与第一轮第二次抽样的冻结副本比，`make_filesystem.rs`、`recovery.rs`、`mount.rs`、`crash.rs`、`walk.rs`、`memory_pool.rs` 逐字节相同，`transaction.rs` 变了而原型用到的五个公开函数签名没变）。

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-c656-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c656-r2-opus/frozen \
  --work <新的空草稿目录>
```

`rerun_steps` 做的事：拷冻结副本 → `tools/patch_copy.py` 打两处补丁（`crash.rs` 的逐态展开函数改 `pub`；`make_filesystem.rs` 加 `make_filesystem_with_order`，五种写序）→ 拷原型 crate、`tools/add_probe_member.py` 加进工作区 → 经 4G 内存包装、`capped.sh 31` 编 release（`-j 4`）→ 照 `tools/items.txt` 48 行逐行跑、只比 `name=` 开头的行（`name=done` 带挂钟不比），打 `name=same` / `name=differs`。整条没复跑过（只过了 `bash -n` 与模板自证，自证末行「✓ 自检通过（查了 5 格）」）。编译要 librocksdb-sys：我拷了 `~/.local/share/singlefs/work/m3-close-c656-r1-s2-main-checks/rerun/target` 当编译目录（`cp -a` 保住时间戳，指纹认了，没重编 C++）；复跑时编译目录是空的，C++ 要现编，4G 上限可能不够（第一轮第二次抽样在 8G、10G 两次撞上限，最后 10G 加 `-j 8` 编过）。

`out/` 里 42 份小产物（count、final、handbuilt）由草稿目录 `gen-small.sh` 照 `items.txt` 同一种起法生成；6 份 run 产物是正式跑的那几次原样拷进来的（同一条命令、同一组参数）。

各文件 sha256（`find . -type f | sort | xargs sha256sum`，前 16 位）：

| 文件 | sha256 前 16 位 |
|---|---|
| out/count-distinct-today.log | 2b0d6b829ef5a139 |
| out/count-distinct-yi-barrier-d0.log | a26b562c603fb8a9 |
| out/count-distinct-yi.log | f44fe4c28625127d |
| out/count-distinct-yi-unitclear-early.log | d5e4e2a4968caf68 |
| out/count-distinct-yi-unitclear.log | a39b39750c412233 |
| out/count-mixed-origin-today.log | 72d51cf1ca6c34e7 |
| out/count-mixed-origin-yi-barrier-d0.log | ae95417f9fa4beb2 |
| out/count-mixed-origin-yi.log | 516e51c6934659a1 |
| out/count-mixed-origin-yi-unitclear-early.log | 3ffbe554a0dc3a85 |
| out/count-mixed-origin-yi-unitclear.log | 9a4bf86c5f58749d |
| out/count-prefix-today.log | ada74a12b1ea0d12 |
| out/count-prefix-yi-barrier-d0.log | 6f0e1938c60ce635 |
| out/count-prefix-yi.log | ef75fe243e3b6d5a |
| out/count-prefix-yi-unitclear-early.log | 16f7fcdc6efc15e4 |
| out/count-prefix-yi-unitclear.log | e813064d52235aa6 |
| out/count-q-intact-yi.log | 32f1687b83a5199b |
| out/count-q-with-blank-d0-yi.log | f071cf0321dcde84 |
| out/count-same-today.log | be666ed8ff2dcdfa |
| out/count-same-yi-barrier-d0.log | fe92138a325eeaf3 |
| out/count-same-yi.log | dd120199f37b9c2b |
| out/count-same-yi-unitclear-early.log | b9c8c79046b5b7de |
| out/count-same-yi-unitclear.log | c62dd290e328c929 |
| out/final-distinct-yi.log | 6ba1c914bb6209c4 |
| out/final-distinct-yi-unitclear.log | ce08bce8c1cd0c8e |
| out/final-mixed-origin-yi.log | 288170ca716034dc |
| out/final-mixed-origin-yi-unitclear.log | f899583c3669d898 |
| out/final-prefix-yi.log | d9a43f2c95a3b57f |
| out/final-prefix-yi-unitclear.log | 3abb441e17e960b8 |
| out/final-same-yi.log | 8cfa26dc0022e5f3 |
| out/final-same-yi-unitclear.log | cf1aa188e14d4ddc |
| out/handbuilt-distinct-d0-up-to-roots.log | 6e889756bc801d0d |
| out/handbuilt-distinct-d1-cache-lost.log | 81a953abcfaa991e |
| out/handbuilt-distinct-d1-everything-but-invalidation.log | b9ac6658b0aab4f3 |
| out/handbuilt-distinct-d1-rings-without-invalidation.log | a15f4d90f93e5b2f |
| out/handbuilt-prefix-d0-up-to-roots.log | 51f67b81eda5b50c |
| out/handbuilt-prefix-d1-cache-lost.log | a38a9c43ac99bfa1 |
| out/handbuilt-prefix-d1-everything-but-invalidation.log | bf77ba69a9da6223 |
| out/handbuilt-prefix-d1-rings-without-invalidation.log | 8f959cd4fc1d9857 |
| out/handbuilt-same-d0-up-to-roots.log | 1c997decc31c2b18 |
| out/handbuilt-same-d1-cache-lost.log | cf4617ab2dca7184 |
| out/handbuilt-same-d1-everything-but-invalidation.log | 85333eabf465a47d |
| out/handbuilt-same-d1-rings-without-invalidation.log | 4da8ff21f7ff6fb4 |
| out/run-distinct-yi.log | 9fb0710ed18256df |
| out/run-mixed-origin-yi.log | 7685d6e869d89016 |
| out/run-same-yi-unitclear-0-140000.log | d5d2f3f72fcfedbb |
| out/run-same-yi-unitclear-140000-210000.log | b01813f88530fa65 |
| out/run-same-yi-unitclear-210000-280027.log | efeb563ada3597fe |
| out/run-same-yi-unitclear-early.log | bdc3cb7aea310fa0 |
| probe/Cargo.toml | 7cbd6d790ef8fc21 |
| probe/src/main.rs | 692bee31973970a4 |
| tools/add_probe_member.py | 902dfc2acf7557de |
| tools/items.txt | 74457c4b84b2c17a |
| tools/patch_copy.py | de7d0978b4fa5233 |
| tools/rerun.sh | 440065f6692d5169 |
| tools/summarize.py | 484e8e5034e66221 |

`probe/src/main.rs` 全长 sha256 `692bee31973970a4832a844aa127e215bf675bde9d40d84ea65c49eaa6354497`，`tools/patch_copy.py` `de7d0978b4fa52330ab2078b2507c7d83834721a07c3de674a4bf07e0481add5`，`tools/rerun.sh` `440065f6692d5169eea9d76b080f5b1563e1c841fcec6394999d20598b5889dd`。

## 原型怎么取样（第 3b、3c 步）

- 盘是内存稀疏盘（`SparseBlockDevice`，每块 2 799 槽），每份历史的旧池只造一次，崩溃态从内存里那一份拷。旧池：mkfs（今天的写序）+ 取号 + 暖机 + 第一个文件 + 3 次覆盖写（u = 5，最后确认的一版 (1, 6)），两块盘都重用；新池 io_min 512、fsid `c656-r2-new-pool`。历史四份：`same`（旧池 fsid 与新池相同）、`prefix`（`c656-r2-old-pool`，前 8 字节与新池相同、后 8 字节不同）、`distinct`（`OLDPOOL!distinct`，前 8 字节就不同）、`mixed-origin`（盘 0 取自池 P：io_min 512、fsid `OLDPOOL!distinct`；盘 1 取自池 Q：io_min 65536、fsid `QQPOOL!!other-q!`；两池各自 u = 5）。另有两份对照只看起点：`q-intact`（池 Q 两块盘）、`q-with-blank-d0`（池 Q 的盘 1 配一块全新空盘）。
- 崩溃态照崩溃点重放的计划取：`visit_crash_state_range_of_the_read_back_flow_with_slices`（副本里改 `pub`），展开取 `full_expansion`（每段任意真子集、原地覆写带撕裂镜像），与第一轮第二次抽样同一种取法；没走 `enumerate_crash_states` 本身，因为它要一个判的根与文件版本表、经判定库录入，mkfs 这条流没有文件版本（推的，没试）。每态：择系统配置、择根、恢复读回、池级 checker、只读挂载、按择到那份的参数可写挂载；整池拒的态另放开用户接着做的事：照今天写序再跑 mkfs，fsid 取新池的与旧池的两种，跑之前先过丙七「读旧池」守卫（原型里的最强形：每块盘 [0, 根环基址) 里逐 4 096 字节试解系统配置，有一份 fsid 前 8 字节与要用的相同就拒）。
- 每次跑之前 `count` 模式用 `crash_state_count_with_torn_in_place_overwrites` 算全量（`out/count-*.log` 22 份）。跑完的态合计：试跑 1 000 + `distinct yi` 31 195 + `mixed-origin yi` 31 168 + `same yi-unitclear-early` 31 267 + `same yi-unitclear` 280 027 = 374 657 态，单次都不超过 10⁶，合计远低于 10⁷。另有手搭的 12 个单态（`handbuilt`，不是枚举）。
- 估时：试跑 1 000 态 2.2 秒（31 线程，带两次重跑 mkfs），按它估三份 3 万态的历史各约 70 秒，没缩。`same yi-unitclear` 每态多一次 22 MB 写零，实测 140 000 态 280.7 秒，分三次跑（0–140 000、140 000–210 000、210 000–280 027），这一份关了重跑 mkfs（`PROBE_RERUN=0`：它的重跑是今天写序、不清单元区，替甲七答不了）。
- 没跑的：`yi-barrier-d0` 五份历史各 181 398 528 / 120 932 352 态（整条流一段 21 写），超过 10⁶，不跑，改手搭单态（第 3b 步「一段历史自己就超过约 10⁶ 的，那段不跑」）。
- 编译排了别的会话占满的内存队列约 40 分钟（`run-with-memory-cap` 排队行在草稿目录 `build.log`），之后正式跑合计约 15 分钟。

## U2　乙的改法本身

### 写成改动

照第一轮正推第 2.1 节草稿写进副本（`tools/patch_copy.py` 的 `MkfsOrderForProbe::InvalidateFirst` 那一支）：两盘各清 [0, 根环基址) → 两盘各一道屏障 → 两盘清根环三区与 journal 环 → 两盘各写两个单元 → 两盘各一道屏障 → 三条第 0 代根 FUA → 两盘四个系统配置槽 → 两盘各一道屏障。今天那一句是「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」（`crates/singlefs-core/src/make_filesystem.rs:559`），乙把它提进单独一趟循环。段序列量过（`out/count-distinct-yi.log`）：`segment_sizes=[2, 12, 1, 1, 1, 4] closed_form_two_state=4117 states_with_torn=31195`，与第一轮对上。

### 两块盘来自两个池、槽距不同：打中字面（量过，`out/run-mixed-origin-yi.log`）

历史 `mixed-origin`：盘 0 是池 P 的盘 0（槽距 4 096），盘 1 是池 Q 的盘 1（槽距 65 536）。起点（mkfs 一个写都没落）两盘系统配置 fsid 不同，整池拒：`base_choice=Err:SystemConfigurationsDisagree base_recover=Failed:SystemConfigurationsDisagree`（`out/count-mixed-origin-yi.log`）。作废之前那一段 8 态的原样（`name=sig` 行，第 0 段全部）：

```
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=false choice=Err:SystemConfigurationsDisagree root=- recover=Failed:SystemConfigurationsDisagree raw_red=["I-1.4", "I-2.1", "I-3.1", "I-4.8", "I-7.4", "I-7.5"] read_only=Err:Recovery writable=not_tried verdict=refused rerun_new[guard=allow found=2]=NoFile(i0t0)|[] rerun_old[guard=refuse found=2]=NoFile(i0t0)|["I-7.7"]
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=false choice=old@4096g8 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] read_only=Ok(i1t6) writable=Err:SystemConfigurationUnreadableAfterTheReadStage verdict=old_readback_intact
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=false choice=other_old@65536g8 root=i1t4 recover=FileRead(i1t4):other_old_older_version raw_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] read_only=Ok(i1t4) writable=Err:NewerStateStillUnreadableAfterOneReread verdict=OTHER_OLD_READBACK_NOT_INTACT
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=true choice=Err:SystemConfigurationsDisagree root=- recover=Failed:SystemConfigurationsDisagree raw_red=["I-1.4", "I-2.1", "I-3.1", "I-4.8", "I-7.4", "I-7.5"] read_only=Err:Recovery writable=not_tried verdict=refused rerun_new[guard=allow found=2]=NoFile(i0t0)|[] rerun_old[guard=refuse found=2]=NoFile(i0t0)|["I-7.7"]
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=true choice=old@4096g7 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] read_only=Ok(i1t6) writable=Err:SystemConfigurationUnreadableAfterTheReadStage verdict=old_readback_intact
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=true choice=old@4096g8 root=i1t6 recover=FileRead(i1t6):old_last_same_root raw_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] read_only=Ok(i1t6) writable=Err:SystemConfigurationUnreadableAfterTheReadStage verdict=old_readback_intact
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=true choice=other_old@65536g7 root=i1t4 recover=FileRead(i1t4):other_old_older_version raw_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] read_only=Ok(i1t4) writable=Err:NewerStateStillUnreadableAfterOneReread verdict=OTHER_OLD_READBACK_NOT_INTACT
name=sig h=mixed-origin order=yi count=1 phase=seg0 torn=true choice=other_old@65536g8 root=i1t4 recover=FileRead(i1t4):other_old_older_version raw_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] read_only=Ok(i1t4) writable=Err:NewerStateStillUnreadableAfterOneReread verdict=OTHER_OLD_READBACK_NOT_INTACT
```

最小历史是序号 2：只落了第 0 写（盘 0 的 [0, 根环基址) 清零），不带撕裂。每一步许可它的那一句：
1. 盘 0 两槽全 0，跳过；跨盘取第一块交得出的盘那一份：「None => chosen = Some(best_on_device),」（`crates/singlefs-core/src/recovery.rs:1442`），交得出的只剩盘 1 上池 Q 的系统配置，没有第二份可比，`SystemConfigurationsDisagree`（`crates/singlefs-core/src/recovery.rs:1448`）走不到。
2. 池 Q 最新的 (1, 5)、(1, 6) 两条根落在区域 0 与区域 2，按 `region_devices` 在盘 0 上，而盘 0 是池 P 的盘；盘 1 上区域 1 留着 (1, 4)。恢复读回池 Q 的旧一版，只读挂载做成。
3. 对照（量过）：池 Q 的盘 1 配一块全新空盘，起点就读回同一个旧版：`base_choice=QQPOOL!!other-q!g8 base_recover=FileRead(i1t4):other_old_older_version`（`out/count-q-with-blank-d0-yi.log`）；池 Q 两块盘都在时 `base_recover=FileRead(i1t6):other_old_last_same_root`（`out/count-q-intact-yi.log`）。所以这一态等于「池 Q 缺一块盘只读挂载」，那是今天就有的降级读法（推的：缺盘只读挂载读回区域深度以内的旧根，D22 已定项 8 那句「根最多退回区域深度那么多代」是同一件事），mkfs 做的只是把池 P 那份挡着的系统配置抹掉。

打中之后的四句：
1. 分不分辨臂：不分辨乙与今天（今天写序的第 0 段含同一次 [0, 根环基址) 清零，同一个态在今天的 280 019 态里，推的，没跑 `mixed-origin today`）；乙与甲分辨（甲在提交点之前不碰 [0, 根环基址)，起点的 `Disagree` 留着，推的），但甲已出局。
2. 被判的系统当时看不看得到判别它的东西：看得到一半——挂载那一刻盘 0 上没有本池系统配置、根环区域 0 与 2 里只有别的 fsid 的根，可写挂载据此拒（`NewerStateStillUnreadableAfterOneReread`）；只读挂载照挂。mkfs 那一刻也看得到（它可以先读两盘的系统配置，fsid 不同即「这不是一个池」），今天 mkfs 不读盘。
3. 满足的是字面哪一分句：第一轮正文乙那一条「第一道屏障之前崩溃（作废还没持久）与甲同问」，甲同问的是「旧池读回与它最后确认的那一版逐字节相同」；这里只读挂上的池 Q 读回 (1, 4)，Q 最后确认的是 (1, 6)，即用户 2026-10-20 不接受的「挂上一个丢了最后几版已确认写的旧池」。但条款默认盘上只有一个旧池，这份历史起点根本挂不上，「旧池」指谁条款没写——也可以读成「打中归错了判据」。
4. 跑前条款给的改法还中不中：正文没登记改法。乙在这一格照中（量过）；今天照中（推的）。我能想到的改法（被攻过零轮、没实现没跑）：条款把「作废之前」那一段写成「盘上若有一个两盘系统配置一致的旧池，读回逐字节相同；否则整池拒」，实现上只读挂载在「有一块盘没有本池系统配置」时也拒——这会把第一轮量过的「一盘作废一盘没作废时旧池只读挂得上」那 4 态一起改成拒。

### 屏障只到一块盘：打中，不分辨乙与今天（手搭单态，量过）

写序 `yi-barrier-d0`：与乙相同，只是盘 1 一道屏障都不发、它那条根 FUA 写当普通写（不认 flush 的盘一般也不认 FUA）。录制流 21 写成一段，全量 181 398 528 态，不跑。手搭了四个单态（落了的写下标见 `out/count-same-yi-barrier-d0.log` 的 `kinds`）。最接近真掉电的是「盘 1 缓存全丢、盘 0 全落」（下标 0、2–5、10、11、14、16、17、18）。三份历史在这一态上的原样（`out/handbuilt-*-d1-cache-lost.log`，去掉 `::` 之后的写表）：

```
name=handbuilt h=same order=yi-barrier-d0 landed=[0, 2, 3, 4, 5, 10, 11, 14, 16, 17, 18] phase=seg0 torn=false choice=new@4096g1 root=i1t4 recover=FileRead(i1t4):old_last_other_root raw_red=["I-3.1", "I-7.7"] read_only=Ok(i1t6) writable=Ok(i1t6) verdict=MIXTURE_OR_BROKEN_NEW(root=i1t4)
name=handbuilt h=prefix order=yi-barrier-d0 landed=[0, 2, 3, 4, 5, 10, 11, 14, 16, 17, 18] phase=seg0 torn=false choice=Err:SystemConfigurationsDisagree root=- recover=Failed:SystemConfigurationsDisagree raw_red=["I-1.4"] read_only=Err:Recovery writable=not_tried verdict=refused rerun_not_tried
name=handbuilt h=distinct order=yi-barrier-d0 landed=[0, 2, 3, 4, 5, 10, 11, 14, 16, 17, 18] phase=seg0 torn=false choice=Err:SystemConfigurationsDisagree root=- recover=Failed:SystemConfigurationsDisagree raw_red=["I-1.4", "I-2.1", "I-4.8", "I-7.4"] read_only=Err:Recovery writable=not_tried verdict=refused rerun_not_tried
```

- fsid 相同：择到盘 0 的新池系统配置（世代 1），择根按 fsid 认、按 txg 取最大，择中盘 1 区域 1 里旧池的 (1, 4)，恢复读回旧池的文件，按新池参数可写挂载做成 `Ok(i1t6)`。这是第一轮正文乙那一条「不许挂上旧池或新旧混合」里的「新旧混合」。许可它的是跨盘只比 fsid 与设备数（`crates/singlefs-core/src/recovery.rs:1448` 那一个 `return`）：盘 0 世代 1、盘 1 世代 8 并存照收。
- fsid 前 8 字节相同、后 8 字节不同（`prefix`）与前 8 字节不同（`distinct`）：两盘 fsid 不同，`SystemConfigurationsDisagree` 整池拒。
- 另三个手搭态（`out/handbuilt-*-d1-rings-without-invalidation.log` 等）：盘 1 作废没落而它的根环清零落了，择到盘 1 上的旧池、读回 (1, 6) 对；盘 1 除作废外全落，择到旧池而根全清，`NoValidRoot`，等于整池拒；盘 0 落到根为止，择到盘 1 的旧池，择根 (1, 4)，读回的字节仍等于最后确认的那一版（判定列写 `OLD_READBACK_NOT_INTACT` 是因为根不是 (1, 6)，我的判定只认「字节与根都对」；只读挂载 `Ok(i1t6)`），字节上没打中。

打中之后的四句：
1. 分不分辨臂：不分辨乙与今天。今天写序遇到同一个故障，「盘 1 全丢、盘 0 全落」这一态的盘面与乙逐字节相同（两种写序发的写集合一样，只是次序不同，推的）。病根在两者共用的前提：每块盘的屏障都到盘。
2. 看不看得到：挂载那一刻看得到——两盘系统配置世代号 1 与 8、journal 实例代号不同，跨盘只比 fsid 与设备数才放过去。
3. 字面哪一分句：「作废持久之后到 mkfs 最后一道屏障之间崩溃……不许挂上旧池或新旧混合」的「新旧混合」；前提「作废持久」在这个故障下对盘 1 不成立，所以也可以读成故障在 C6 那类「设备不守屏障」的射程里、不归这一条——由主 agent 判。
4. 改法在这一格上中不中：丙七「强制新 fsid」让这一态变成整池拒（量过：`prefix`、`distinct` 两份都是 `Disagree`）；甲七不碰根环，照中（推的）；乙七只动 checker，照中（推的）。另一个改法（被攻过零轮、没实现）：跨盘择系统配置时世代号不同就拒或取全池最大，见下面「附带两样」。

### 读故障、一块盘中途掉线：没打中（推的）

- 读故障：乙的 mkfs 一次盘都不读（今天的 mkfs 只按盘的字节数判几何：「let unit_area_start = check_geometry(parameters, &sizes)?;」（`crates/singlefs-core/src/make_filesystem.rs:485`）），乙不改这一点；挂载时的读故障落在 R2（D22 已定项 8 射程那一段「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过」），作废落了之后两盘 [0, 根环基址) 全 0，没有一个能择的槽，有没有读错都是整池拒。读故障真正打到的是丙七「读旧池」守卫：守卫读不出旧槽时放不放行，条款得写（见 U4）。
- 一块盘中途掉线：乙的每一步都 `?` 往上交错，掉线那一刻 mkfs 停下；落盘的是录制流的一个前缀加上掉线前已发的写的任意子集，最后一段没关，它的每个子集都是枚举里那一段的子集（推的）；掉线的盘回来，就是第 0 段「一盘作废一盘没作废」或之后的整池拒。没有新形状。

### 整池拒之后用户再跑 mkfs（量过，`out/run-distinct-yi.log`）

放开的用户动作：拒的态上照今天写序再跑 mkfs，fsid 取新池的（`c656-r2-new-pool`）与旧池的（`OLDPOOL!distinct`）两种，跑之前先过丙七守卫。`python3 tools/summarize.py out/run-distinct-yi.log` 原样（列：态数、段、择到哪个、恢复、只读挂载、可写挂载、起点 checker 红、判定）：

```
1 all new@4096g1 NoFile(i0t0) Err:Open Ok(i0t0) [] new_complete
2 seg0 old@4096g7 FileRead(i1t6):old_last_same_root Ok(i1t6) Err:SystemConfigurationUnreadableAfterTheReadStage [] old_readback_intact
2 seg0 old@4096g7 FileRead(i1t6):old_last_same_root Ok(i1t6) Ok(i1t6) [] old_readback_intact
2 seg0 old@4096g8 FileRead(i1t6):old_last_same_root Ok(i1t6) Err:SystemConfigurationUnreadableAfterTheReadStage [] old_readback_intact
2 seg0 old@4096g8 FileRead(i1t6):old_last_same_root Ok(i1t6) Ok(i1t6) [] old_readback_intact
31103 seg1 Err:NoValidSystemConfiguration Failed:NoValidSystemConfiguration Err:Recovery not_tried ["I-8.5"] refused rerun_new[guard=allow found=0]=NoFile(i0t0)|[] rerun_old[guard=allow found=0]=NoFile(i0t0)|["I-7.7"]
1 seg2 Err:NoValidSystemConfiguration Failed:NoValidSystemConfiguration Err:Recovery not_tried ["I-8.5"] refused rerun_new[guard=allow found=0]=NoFile(i0t0)|[] rerun_old[guard=allow found=0]=NoFile(i0t0)|["I-7.7"]
1 seg3 Err:NoValidSystemConfiguration Failed:NoValidSystemConfiguration Err:Recovery not_tried ["I-8.5"] refused rerun_new[guard=allow found=0]=NoFile(i0t0)|[] rerun_old[guard=allow found=0]=NoFile(i0t0)|["I-7.7"]
1 seg4 Err:NoValidSystemConfiguration Failed:NoValidSystemConfiguration Err:Recovery not_tried ["I-8.5"] refused rerun_new[guard=allow found=0]=NoFile(i0t0)|[] rerun_old[guard=allow found=0]=NoFile(i0t0)|["I-7.7"]
16 seg5 Err:NoValidSystemConfiguration Failed:NoValidSystemConfiguration Err:Recovery not_tried ["I-8.5"] refused rerun_new[guard=allow found=0]=NoFile(i0t0)|[] rerun_old[guard=allow found=0]=NoFile(i0t0)|["I-7.7"]
40 seg5 new@4096g1 NoFile(i0t0) Err:Open Err:SystemConfigurationUnreadableAfterTheReadStage [] new_complete
24 seg5 new@4096g1 NoFile(i0t0) Err:Open Ok(i0t0) [] new_complete
```

- 乙本身没打中：作废之前 8 态择到旧池、读回 8 / 8 对；之后 0 态择到旧池、0 态混合；拒 31 122 态（31 103 + 1 + 1 + 1 + 16）。
- 用新 fsid 重跑：31 122 / 31 122 恢复成第 0 代空池、checker 全绿。
- 用旧池的 fsid 重跑：守卫在 31 122 / 31 122 态上 `found=0`（一份系统配置都读不到）、放行，做完 I-7.7 全红。同一个守卫在起点镜像上对旧池 fsid 是拒的（`out/final-distinct-yi.log` 打的是对新池 fsid 放行、`found=1`；`mixed-origin` 第 0 段对旧池 fsid `guard=refuse found=2`）。乙的作废抹掉的正是丙七「读旧池」要读的那几个字节。
- 为什么会有人拿旧 fsid 重跑（用户动作，没写死）：第一次 mkfs 用了新 fsid、崩在作废之后，用户改回原来那个 fsid 再跑（例如脚本按池名派生 fsid、或照旧配置重建）；同一形也出在「旧池的头 1 MiB 被别的工具清过」的盘上（推的，没造）。

### 附带两样写不写进条文

1. 一盘作废一盘没作废时旧池只读挂得上、可写挂载拒：写进条文。量过（`out/run-distinct-yi.log` 第 0 段 4 + 4 态）。但「只读挂得上的是哪个池」要连同 `mixed-origin` 那一形一起写，见上面的改法。
2. 择系统配置取第一块可择盘那一份、不取全池最大世代号：写进条文，并补一形。手搭的屏障态上，盘 0 世代 1、盘 1 世代 8、fsid 相同，跨盘那一步照收、择盘 0，恢复择到盘 1 的旧根（量过，见上）。今天写序正常落盘时两盘世代号不可能差这么多：每盘按自己两槽最大号加 1 轮换（「let slot_generation = verified_on_this_device」（`crates/singlefs-core/src/transaction.rs:575`）起那几行，槽号取「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」（`crates/singlefs-core/src/transaction.rs:626`）），两盘差最多一次轮换（推的，读码）。我提的改法（**只在我的推理里、没实现没跑、被攻过零轮**）：跨盘择系统配置时两盘世代号差大于 1 就整池拒。它在屏障那一格上把混合变成拒（推的），在第一轮量过的「撕裂择到世代 7、另一盘世代 8」那几态上不拒（差 1）。

## U3　「完整的新池」（量过）

`out/run-distinct-yi.log` 第 5 段（四次新系统配置写）共 80 态：16 态整池拒；64 态择到新池系统配置、恢复读回第 0 代空池 `NoFile(i0t0)`、池级 checker 全绿 `[]`，其中 24 态可写挂载做成、40 态可写挂载拒 `SystemConfigurationUnreadableAfterTheReadStage`（有一块盘一份自证过的新系统配置都没有）。`mixed-origin` 同一段是 19 / 26 / 8（`out/run-mixed-origin-yi.log`），`same yi-unitclear` 是 24 / 40 / 16：态数随几何变，条文不能写成「那 40 个」。

- 这 40 态 checker 全绿、内容就是 mkfs 要写的全部（根、单元在前面的屏障之后已持久），缺的只是一块盘上的系统配置冗余。D22 已定项 8 的承诺「每盘一槽系统配置坏在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:180`），一块盘两槽都没有已经出了单故障的射程；但池是空的，没有用户的写可丢，而 mkfs 没有返回成功，用户本来就该重跑。
- 攻它能攻到的只有一处：只读挂载在这 64 态上都报 `Err:Open`（原型的只读挂载要打开一个文件，空池没有文件；24 个可写挂载做成的态也一样），所以「挂上」只能按可写挂载与恢复判，「完整」不能用「只读挂载做成」来定义（原型的限度，不是实现的结论）。
- 建议的写法（推的）：「完整的新池」= 择到的系统配置是这一次 mkfs 写的（fsid 与世代 1）、恢复读回第 0 代空池、池级 checker 全绿；可写挂载可以因某块盘没有新系统配置而拒，拒的错误要能让用户分出「mkfs 没做完、重跑」与「盘坏」——今天那个错误名 `SystemConfigurationUnreadableAfterTheReadStage` 分不出（推的）。

## U4　I-7.7 与 C659：三个改法

### 先复核一个读数：「fsid 不同也红」的两个 fsid 前 8 字节相同

checker 认单元与 journal 记录只用 fsid 前 8 字节：「geometry.filesystem_identifier[..8]」（`crates/singlefs-checker/src/walk.rs:4900`），按它认单元「if let Some(instance) = unit_write_order_instance(&bytes, filesystem_identifier_low) {」（`crates/singlefs-checker/src/walk.rs:4861`）、认 journal 记录「if let Ok(record) = check_journal_record(&bytes, filesystem_identifier_low) {」（`crates/singlefs-checker/src/walk.rs:4841`）。第一轮第二次抽样报「比 C659 的字面宽：fsid 不同、槽距相同也红」，它原型里的旧池 fsid `c656-s2-pool-A!!` 与新池 `c656-s2-new-pool` 前 8 字节都是 `c656-s2-`（读的是 `research/prompts/m3-close-c656-r1-opus-s2-model/probe/src/main.rs` 的两个常量）。mkfs 全落之后的原样（`out/final-*-yi.log` 与 `out/final-*-yi-unitclear.log`，各一行）：

```
name=final h=same order=yi guard_on_the_old_pool_for_the_new_fsid=refuse found=1 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=["I-7.7"] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
name=final h=same order=yi-unitclear guard_on_the_old_pool_for_the_new_fsid=refuse found=1 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=[] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
name=final h=prefix order=yi guard_on_the_old_pool_for_the_new_fsid=refuse found=1 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=["I-7.7"] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
name=final h=prefix order=yi-unitclear guard_on_the_old_pool_for_the_new_fsid=refuse found=1 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=[] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
name=final h=distinct order=yi guard_on_the_old_pool_for_the_new_fsid=allow found=1 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=[] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
name=final h=distinct order=yi-unitclear guard_on_the_old_pool_for_the_new_fsid=allow found=1 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=[] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
name=final h=mixed-origin order=yi guard_on_the_old_pool_for_the_new_fsid=allow found=2 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=[] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
name=final h=mixed-origin order=yi-unitclear guard_on_the_old_pool_for_the_new_fsid=allow found=2 phase=all torn=false choice=new@4096g1 root=i0t0 recover=NoFile(i0t0) raw_red=[] read_only=Err:Open writable=Ok(i0t0) verdict=new_complete
```

- 乙做完：`same`、`prefix` 红，`distinct`、`mixed-origin` 绿。C659 的根因不是「fsid 相同」，是「旧池单元头上的 fsid 前 8 字节与新池相同」（量过）。调查员报告与第一轮判决把 C659 写成同 fsid 的问题，射程要改成前 8 字节。
- 甲七做完：四份全绿（量过）。

### 甲七：mkfs 清单元区

写进副本两种排法（`tools/patch_copy.py`）：`InvalidateFirstThenClearUnitArea`（作废屏障之后、与根环 journal 环同段，每盘一次写零，从第 0 版两个单元之后清到盘尾）；`ClearUnitAreaTogetherWithInvalidation`（与 [0, 根环基址) 同段、同一道屏障之前）。

- 修哪一格（量过）：排在作废屏障之后，`same` 历史 280 027 态全跑（`out/run-same-yi-unitclear-*.log` 三份）：作废之前 8 态择到旧池、读回 8 / 8 对、checker 全绿；之后择到旧池 0、混合 0；择到新池的 65 态 checker 全绿（乙不清单元区时同一份历史做完报 I-7.7）。
- 会不会造出新的错（量过）：排进作废那一段就打中 C656 本身。`out/run-same-yi-unitclear-early.log` 第 0 段 80 态：拒 8，择到旧池 72，其中读回坏 32（`Failed:MappingStillUnreadable` 或 `Failed:UnitUnreadable`），这 32 态里 24 态只读挂载做成、8 态可写挂载做成；不带撕裂的 3 态。最小一态序号 60：两盘作废都没落、两盘单元区清零都落了，恢复报 `UnitUnreadable`。所以甲七的条文要写死次序：单元区清零在两盘作废都过屏障之后。
- 代价（量过的部分）：段序列 `2+12+1+1+1+4` → `2+14+1+1+1+4`，带撕裂全量 31 195 → 280 027（9 倍，`out/count-same-yi-unitclear.log`）；录制流多 2 写。写的字节数是整个单元区：原型的小盘每盘从槽 1 411 清到 2 799，约 22.7 MB。
- 代价（推的）：`O_DIRECT` 后端的写零是逐块真写 0（「write_zeroes_in_chunks(offset, length, scratch.as_slice(), |chunk_offset, chunk| {」（`crates/singlefs-core/src/block_device.rs:977`）），mkfs 的时长随盘容量线性涨：按 200 MB/s 估一块 4 TB 的盘约 5.6 小时（拍的速率，没量）。文件后端走打洞，不受这个影响。另一个代价：之后在用过的盘上挂载与扫描都看不到旧池单元——这正是要的，也意味着第一轮正推提到的离线救旧池出路（`crates/` 今天没有）在 mkfs 做完之后彻底没了。
- 不修的格：屏障只到一块盘那一格的混合（根环问题，不在单元区，推的）；`mixed-origin` 作废之前读回池 Q 旧版（推的，清零排在作废之后，第 0 段不变）。

### 乙七：I-7.7 ① 射程收窄到新池分配到的单元（推的，读码；没在副本上改 checker）

- I-7.7 ① 今天的射程逐字是「fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者」（`crates/singlefs-checker/invariants/02-根环.md:14`），checker 照它逐槽扫整个单元区：「Box::new(geometry.unit_area_start_slot..end)」（`crates/singlefs-checker/src/walk.rs:4853`），按 fsid 前 8 字节认单元：「if let Some(instance) = unit_write_order_instance(&bytes, filesystem_identifier_low) {」（`crates/singlefs-checker/src/walk.rs:4861`）。
- 这一扫描要抓的就是「没人引用」的单元：入库的坏镜像是「没人引用的槽 50302 上放一份写序实例代号 2 的数据单元」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1030`），I-7.7 那一行自己写着「坏镜像三份：没人引用的实例 2 孤儿（①）」（`crates/singlefs-checker/invariants/02-根环.md:14`）。C322 定案（2026-09-14）把 ① 改成扫单元区，为的是取号那一步只落一块盘、之后的发布没做完时留下的孤儿。
- 收窄到「新池分配到的单元」，checker 只有两种认法：按择到的根走得到的单元，或按分配记录里记了的槽。那份孤儿两样都不是（它在一次没做完的发布里写下、分配记录还没发布），收窄之后它判绿：**钉住的 ① 那份坏镜像失去会红的形式**（推的；没在副本上改 checker 跑 `checker_known_bad_images`）。② 那份（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1054` 那一行注释起）只在各盘号不等时判、扫的是同一批载体，只收窄 ① 时它照旧红。
- 判别子观测不到：旧池留下的同前缀单元与新池自己一次崩溃留下的孤儿，在 checker 读得到的字段上逐条同形——都没人引用、头里 fsid 前 8 字节与本池相同、实例代号高于系统配置。单元头里没有区分「哪一个池写的」的字段（只带 fsid 前 8 字节，`crates/singlefs-checker/src/walk.rs:4900` 那一行取的就是 `geometry.filesystem_identifier[..8]`）。所以任何一种收窄，要么照旧在旧单元上红（C659 不修），要么在孤儿上不红（C322 那一格丢掉），两边不可兼得。这是 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 表里「判别子观测不到」那一形，出路只有改格式（单元头带全 16 字节 fsid 或一个池代号）——那就不是乙七，是另一个改法，被攻过零轮。
- 修哪一格：只修「mkfs 刚做完、checker 红」那一格的表面（推的）；C659 的根因（旧池单元留在盘上、头上的前缀与新池相同）不动。
- 代价：坏镜像两份要撤或改；I-7.7 的条文要改射程；C322 那一格回到 2026-09-14 之前的状态（推的）。

### 丙七：不许同一个 fsid 在用过的盘上重用

两种做法分开看。

**丙七甲：mkfs 先读旧池，同 fsid 就拒。**
- 比较的口径要是前 8 字节（量过：`prefix` 历史 16 字节不同、前 8 字节相同，乙做完 I-7.7 红）。比 16 字节的守卫放过它。
- 被乙掏空（量过）：作废落了之后的 31 122 个拒态上守卫一份系统配置都读不到，拿旧池 fsid 重跑全部放行、做完全红（见 U2）。守卫读的 [0, 根环基址) 正是乙第一步抹掉的那一段。要补就得让守卫扫单元区的单元头（读整个单元区，代价与甲七同量级，推的），或让作废写下一个「这里曾是 fsid 前缀 X 的池」的墓碑而不是全 0（**被攻过零轮、没实现**；墓碑本身又是一种 [0, 根环基址) 里的新格式）。
- 读故障（推的）：守卫读旧槽报读错时放不放行，条款得写。放行就漏；拒则一块头部有坏扇区的盘永远做不了 mkfs（覆写本来是让盘重映射坏扇区的常规办法）。
- 修哪一格：起点镜像上同前缀的 mkfs 被拒（推的加原型守卫量过 `found=1`）；不修乙之后的重跑、不修头 1 MiB 被清过的盘。

**丙七乙：mkfs 自己生成 fsid（强制新 fsid）。**
- 修哪一格（量过两格）：前 8 字节不同，乙做完 I-7.7 绿（`out/final-distinct-yi.log`）；屏障只到一块盘那一态由混合变成整池拒（`out/handbuilt-distinct-d1-cache-lost.log`）。第一轮甲出局的那一形（同 fsid 时提交点之后混合）也靠 fsid 相同（第一轮第二次抽样报告第 V1 节，推的：fsid 不同时那条候选在那份历史上站住）。
- 撞前缀的概率（推的）：16 字节随机、按前 8 字节算一对撞上 2⁻⁶⁴。
- 代价（推的）：`MakeFilesystemParameters::filesystem_identifier` 今天由调用方给，代码自己写着「而 fsid 是调用方给的（`MakeFilesystemParameters::filesystem_identifier`，mkfs 自己不生成）⇒」（`crates/singlefs-core/src/make_filesystem.rs:532`）；改成 mkfs 生成，装置与用例要能注入固定 fsid（崩溃点重放、坏镜像都钉了 fsid）；按 fsid 筛的挂载（S3）要把生成的 fsid 交给用户。
- 不修的格：`mixed-origin` 作废之前读回池 Q 旧版（与 fsid 无关）；单元区里旧池不同前缀的单元照留（checker 不认，不红）。

### 几个改法各修哪一格

| 格 | 乙（不加改法） | 乙 + 甲七（屏障后清单元区） | 乙 + 甲七错排（与作废同段） | 乙 + 乙七 | 乙 + 丙七甲（读旧池拒） | 乙 + 丙七乙（强制新 fsid） |
|---|---|---|---|---|---|---|
| mkfs 做完 I-7.7，同 fsid / 同前缀 | 红（量过） | 绿（量过） | 绿（推的，做完盘面与正确排法相同） | 绿（推的） | 起点被拒（推的） | 不会出现（推的） |
| 整池拒后用旧 fsid 重跑，I-7.7 | 红（量过） | 绿（推的：重跑也清单元区） | 绿（推的） | 绿（推的） | 放行、红（量过） | 不会出现（推的） |
| 作废之前旧池读回 | 8 / 8 对（量过） | 8 / 8 对（量过） | 72 态里 32 态坏（量过） | 同乙（推的） | 同乙（推的） | 同乙（推的） |
| 屏障只到一块盘，同 fsid | 混合（量过） | 混合（推的） | 混合（推的） | 混合（推的） | 混合（推的：起点同 fsid 已被拒则无此形） | 整池拒（量过，`distinct`） |
| `mixed-origin` 作废之前读回池 Q 旧版 | 中（量过） | 中（推的） | 中（推的） | 中（推的） | 中（推的） | 中（推的） |
| 钉住的 I-7.7 ① 孤儿坏镜像 | 红（入库坏镜像） | 红（推的） | 红（推的） | **判绿（推的）** | 红（推的） | 红（推的） |
| C656 验收要枚举的态（用过的盘、`same`） | 31 195（量过） | 280 027（量过） | 31 267（量过） | 31 195（推的） | 31 195 + 守卫读（推的） | 31 195（推的） |

### 与 U1「同 fsid 是不是合法输入」是同一个问题的两面

量过的三格（乙做完 I-7.7 红、屏障只到一块盘时混合、第一轮甲的混合）都要「前 8 字节相同」才出；前 8 字节不同时三格都不出。所以要问的不是「同 fsid 合不合法」，是「前 8 字节相同合不合法」。若定为不合法，丙七乙（mkfs 生成）是唯一一个不被乙的作废掏空的做法；若定为合法，I-7.7 那一格只剩甲七修得到，代价是 mkfs 时长随盘容量涨、C656 验收态数 9 倍。乙七两种定法下都不该选（判别子观测不到）。

## 没打中的形状

| 形状 | 打的是谁 | 取样 | 结局 |
|---|---|---|---|
| 作废之前择到旧池之后接着可写挂载、再写，两盘系统配置世代号会不会错开 | 乙 | 读码（每盘按自己两槽最大号加 1、写到「号 mod 2」那一槽） | 每盘自洽，不会出现同号两槽（推的，没跑） |
| 两块盘来自两个池时作废之后择到旧池 | 乙 | `mixed-origin` 31 168 态 | 作废之后 0 |
| 两块盘来自两个池时的混合 | 乙 | 同上 | 0（两池 fsid 都与新池不同） |
| 整池拒之后用新 fsid 重跑 | 乙 | `distinct`、`mixed-origin` 各 31 122 拒态 | 全部第 0 代空池、checker 绿 |
| 甲七排对次序之后的旧池读回与混合 | 乙 + 甲七 | `same` 280 027 态 | 读回 8 / 8 对，混合 0 |
| 屏障只到一块盘时 fsid 不同的混合 | 乙 | 手搭 8 态（`prefix`、`distinct` 各 4） | 0（拒或旧池读回字节对） |

## 这条腿自己的限度

- 只量了 u = 5、新池 io_min 512、journal 环 6 MiB、2 799 槽的小盘；`mixed-origin` 只有一种拼法（盘 0 来自 P、盘 1 来自 Q），反过来拼（区域 0、2 在 Q 的盘上）没跑。
- 屏障只到一块盘没枚举（1.8 亿态），只手搭四态；「今天写序在同一个故障下盘面相同」是推的。
- 丙七守卫是我写的最强形（扫 [0, 根环基址) 每 4 096 字节），不是哪一份设计；甲七的清零范围（第 0 版两个单元之后到盘尾）也是我定的。
- 判定列 `OLD_READBACK_NOT_INTACT` 要求字节与根都对，有一个手搭态是字节对、根不对（已在正文写明），读那一列要看恢复列。
- 只读挂载在原型里要打开一个文件，新池（空池）上一律 `Err:Open`，这一列对新池没有信息。
- 这条腿提的改法（只读挂载在缺本池系统配置的盘时也拒、跨盘世代号差大于 1 就拒、作废写墓碑）都只在推理里，**没在模型上量过，被攻过零轮**。
- 原型取崩溃态用的是 `visit_crash_state_range_of_the_read_back_flow_with_slices`（副本改 `pub`），不是 `enumerate_crash_states` 本身，理由同第一轮第二次抽样。

## 要主 agent 跑的命令

- 复跑：开头「复跑命令」那一条（编译 librocksdb-sys 可能要比 4G 大的上限，或照我这样拷一份已编好的 release 编译目录）。
- 若要把「屏障只到一块盘」枚举全：`count same yi-barrier-d0` 是 181 398 528 态，要按序号段分 182 次以上、每次 ≤ 10⁶，超过这一条腿的 10⁷ 预算，交主 agent 定要不要跑。

## 没做什么

- 没判辩方那几格（U1 只答了与 U4 共用的那一面），没替主 agent 采纳；副本上的数不是入库装置上的数。
- 没跑重型测试、没调 checker 档任何测试目标，轻型三条都没起（这一件不改 `crates/`）；乙七没在副本上改 checker 跑 `checker_known_bad_images`。
- 模型目录里本轮新建的 `probe/src/main.rs`、`tools/rerun.sh` 开发中用 python 整份读写与一次 `sed -i` 改过几次（`rerun.sh` 经同目录临时文件再 `mv` 换上），没走 `scripts/kb/replace-once.py`；交回时的样子以开头的 sha256 为准。
- 草稿目录里留着冻结副本 `frozen/`（复跑要它，35 MB）与 `runs/`；仓副本 `tree/` 与编译目录 `target/` 交回前删。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-c656-r2-opus-output.md --root . --background research/prompts/_m3-close-c656-r2-background.md`（写这一节之前跑的）：

```
  ✓ 核了 16 处引文、4 处裸行号，对上 16 处，没判 4 处（其中裸行号没有引文可比 4 处；逐处列在上面）
```

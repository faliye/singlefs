# m3-batch14-code-r1 云端攻方（Opus）报告（2026-10-04）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 GA GB GC K1 K2 K3 FX MQ1 MQ3 MG1 MG2 MG4 IMM HELD -->

攻击面：正文第三节 K2，重点 N1、N3；立场：找反例。四份拷贝都从冻结副本 `/tmp/claude-1000/m3-batch14-code-r1/tree/` 拷出：GA（原样，入库的甲 GA）、IMM（丙1：卸载那一格整截当场放开，与变异表「删除批第 14 批 GA」那两行同一刀）、HELD（改之前那一版：卸载那一格整截扣到生效）、FX（GA 再加我提的零轮改法，见第五节）。原型是拷贝里新加的 bin crate `attack-b14-probe`，底子是上一轮 `research/prompts/m3-unlink-code-r3-opus-model/probe/main.rs` 的 N1 那一半（起点四族、录卸载、前缀刀、三种丢盘、读法三交集、只读挂载整读），移植之后在 IMM 与 HELD 上复出上一轮的数（第二节第 1 行），证明移得对。只跑 harness 档测试目标与这个原型；checker 档一条没跑。副本上的数不是入库装置上的数。

线程上限开工是 15，跑到第 2 批（形 R 的 GA 窄盘前两档覆盖写）之后收到主 agent 改成 7，从下一条命令起照 7（`scripts/run.sh`、`scripts/build.sh` 改成经 `capped.sh 7`、同批并行件数不超过 7）。之后有两次越界，照实记在「这条腿自己的限度」。

## 复跑

模型目录 `research/prompts/m3-batch14-code-r1-opus-model/`。复跑：`bash research/prompts/m3-batch14-code-r1-opus-model/rerun.sh <阶段> <空工作目录>`，阶段按脚本里 `STAGES` 的次序逐个跑（每段前台 4 分钟以内）；`<阶段>` 写 `all` 一次跑完（约 50 分钟，要放后台由主 agent 起）；最后一段 `compare` 按同样的命令从复跑日志重出汇总，与 `out/` 逐行比。交回前在空目录 `/tmp/claude-1000/m3-batch14-code-r1-opus/rerun/` 上按 `STAGES` 的次序逐段跑完（`all` 那一种写法没整条跑过：单条命令超过前台 240 秒），`compare` 的输出原样在 `out/rerun-compare.txt`，12 行全是 ✓、退出码 0。

每份文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里算）：

```
d572c54d9603bd612df6c61ce58385daad24937fc5a167a700006b4c374fda0c  out/abandon-chosen.txt
49d04f39a91d914bca005063593ea1880249b9022fcc86c23a0fe51f0514b726  out/n1-rescan-summary.txt
22394b02e4daf5b6a94c4fa3943db0ce296c43bae6a94af100a9e4a686636d9c  out/qf-ga.log
fc76010fab4e7253b9473a90d0becd045cac61c5f9166cc5623895499b2a8cb5  out/remount-summary.txt
295b94e7e64ee3adc63d6fe03e592eefd30ed2e9674e8ff895a6e45152e59f6a  out/renum-h2-ga.log
11b2057644382c982ecbb55b634e064bc3cdcde0441b7beeaafc76614c6d97e6  out/rerun-compare.txt
ee05d6e4c4733f8eb4f312943df9bf2cafecb38806e620831cc3665d15d1670b  out/r-h2-ga-detail.log
2ab4c9e5fb6bcff2c28a75d382e7e773a68c705dfad77e8a53ab3faaf51ce1f4  out/rp-ga-4-1.log
7c211c2832562ac8f7f856a59bfc34c1f79e7a0b2f68a591586fbf13d529ce9f  out/tests-fx.log
28d4812da219d1e0f48b24194963595d0548fa4e9b6da7660a80c3b469c3bf4d  out/tests-mg1.log
c9b45e3f905a66735a80b4cc892a9e971af9441898fc5565d085d51f1b57e55c  out/tests-mg4.log
30850a3b9d756d78974c9b2a4649863e423cc6f3a675e086ab97afdf88e97485  out/tests-mq1.log
fc34e43884925e2cd2264acda86d392771398575a927767ed5f79f787d478ce6  out/tests-mq3.log
21079fa5c2c610d6ba9c4fe071f934575f3c3c5715333b3d47cad5275d86c1b5  patches/fx-mount-holds-above-the-newest-root-on-every-device.patch
718d7c503ac00e0c9475b50c6ae724be633d5005e839639ab9d63bc1dc6ec881  patches/held-unmount-holds-everything.patch
2354b56b78f882cbafe92b5527d0447c1a3b82c97d53630ab5ed43ea671870d6  patches/imm-bing1-releases-everything.patch
45ba957d6e99c97e0a6ba3adeaf3e8a5fa91f328f1a6b36ca07ec3bd0f1baf25  patches/mg1-max-over-devices.patch
6d61630b80160ed80fdac92b348860b8d124fbbb8627533f47ddd912e2ab5138  patches/mg4-remainder-not-reclaimed.patch
6bcaa3d0c0de36f7193a8d11e504e6c9b0cc2be8d1c229919c27d11079cde80e  patches/mq1-session-drops-the-unlink-time.patch
23a38902bbafaef7753d1dd55fd811c101a36eeeaa8caed12eb6de18de6ba837  patches/mq3-intent-keeps-the-previous-time.patch
49b963baa7520bfdd31282e8be63e00320ceff9a970e5e38d81a0a81988a78d7  patches/workspace-member.patch
83c5831d1dbec67434395e48434d7e04de01b1bd2e241e0c82b4caa5f3f1ca82  probe/Cargo.toml
6b81319521a25175b8940696f09dbe941cb7cef4a8c254b91be213afceda07fe  probe/main.rs
59c620ec7f57a9dbb7367b8ef0c0424c1a4e64acbb24df14a73ee3ab63f0eecf  rerun.sh
e4d5c33f8790557b33c4745db5a05c8d4291da5c4a1406e8722eed88f787f87f  scripts/build.sh
51e2eff542aee0cc91e20499629477ae399375fb967ccff060c90178e9b48734  scripts/compare.sh
90beaeb129adce7e30795c12b31a52889071ec965133f615bb1834780764038d  scripts/grid-batch.sh
a7cfae5ff1c04f383893a0363e3f8a60c1a1b34f0591833ac9c50eaad24ddbff  scripts/rsummarize.sh
93b1297f72daa16f3002f12ac8cb45a2b88c187c72713aad83ae48e55721b5e5  scripts/run.sh
db86967a5e1c25e74ea0e60d6a57c4fc4012354683d99b24bfcf2610032dc5ae  scripts/summarize.sh
211a542b06602de5711438b0af353923055190fb314b9ad2482c1a9744ecf138  scripts/sweep-batch.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| N1 GA 在入库代码上按上一轮四族重扫（卸载那一串自己写掉被择中的根引用的单元） | 没打中（量过，一次抽样） | 窄盘 1900 个起点、卸载—再挂两族 400 + 186 个起点，GA 交集 0、只读挂载读不出 0、卸载被拒 0；同一份原型 IMM 复出上一轮的 174 / 37 / 1、HELD 复出 252 个被拒 | 第一节 1.1 |
| N1 形 R：卸载里崩（新 F 已写进一块盘的系统配置、带新 F 的根没落满每块盘）→ 两盘再可写挂载 → 挂载的写行或暖机里再崩 → 丢住着现行那一版的那块盘 | **打中（量过）**，不分辨臂 | 再挂重建分配器按系统配置里的 F_生效 = C 当场回收释放代 ≤ C 的槽，写行那次的单元写掉另一块盘上最新的根（C − 1 或更早）引用的单元；剩下那块盘只读挂载失败或打开活文件失败。GA 窄盘 186 个起点里 13 个、普通盘 95 个里 9 个（含 4 GiB 那一档）；IMM 14 / 9，HELD 9 / 8；我的零轮改法 FX 0 / 0 | 第一节 1.2 |
| N1 形 R 的层 0 枚举域 | **打中（量过）** | h2 那一份（窄盘 176 槽、覆盖写 0、4 单元、崩在第 3 批之后）卸载刀 n = 1、n = 16 上，再挂写流每段展开各 20511 个状态，丢盘 0 时只读挂载失败 12302 / 8195 个；两盘都在时池级 checker 0 违例 | 第一节 1.2 |
| N1 单盘恢复择中被抛弃的根 | 没打中（量过，一次抽样） | 造出被抛弃的根之后卸载，9027 个（刀 × 丢盘）判定里择中被抛弃的根 0 次：那次挂载的暖机已在每块盘上落了比它新的根（`warm_up_publish_txgs`），GA 不算它只会让当场放开的更少 | 第一节 1.3 |
| N1 根环里读不出的槽、有洞 | 没打中（读代码，没造） | 卸载那一遍读根环时，这个进程写过的槽读坏要重读、仍坏就拒这次抬 F；各盘最新的根都是这次挂载写的；有洞（journal 施加出来的那一版不写根）只让那块盘的最新根更旧、最小值更小 | 第一节 1.4 |
| N1 卸载—再挂循环 | 没打中（量过，一次抽样） | 循环两族上 GA 交集 0；形 R 是这一族加一次崩溃的变形，打中记在第二行 | 第一节 1.1、1.2 |
| N3 ①b：同一份崩溃镜像各自续做，两份盘逐字节相同 | 没打中（量过，一次抽样） | 96 种盘面 × 意图时间 3 种（含 0 与 u64::MAX）= 288 份，两次续做逐字节相同 288 份；续做各步里还有这条记录的 1275 步，写入时间都等于意图那次给的 | 第二节 2.1 |
| N3 模型不比时间留下的缝 | **打中缝（量过）** | 会话那一层把调用方的时间丢掉、意图写 0（MQ1）：四个相关测试目标 80 个用例全绿；意图照旧带上一版的时间（MQ3，变异表已有那一行）：只有那一条专门用例红，模型对拍的 8 条与挂载透明的 33 条全绿 | 第二节 2.2 |
| N3 钉了字节的地方有没有悄悄变 | 没打中（读代码） | checker 档里用 `HistoryOperation::Unlink` 的两段历史只钉计数与结局，不钉字节；随机历史的时间按步号取，不碰系统时钟 | 第二节 2.3 |
| N2、N4–N6 错实现照样过 | GA 那一段没打中（量过）；Q-F 那一段见 N3 第二行 | 「各盘取最大」（MG1）被 GC 两条红；「其余那一截不回收」（MG4）被四条红 | 第三节 |
| 「GA 只看这一串」的条款射程 | 条款没写（推的，形 R 量过） | 「正常卸载」那一行括注只管「这一串」；卸载半途崩溃之后下一次可写挂载按系统配置里的 F 回收，条款没说那时剩下那块盘上最新的根还保不保 | 第一节 1.2 四句 |

## 一、N1 GA

### 1.1 上一轮四族在入库代码上重扫（没打中）

造法与上一轮第一次抽样相同（窄盘：单元区 176–352 槽步长 16、覆盖写 0–10、文件 1/2/4/8/14 单元、意图之后崩在第 0/1/3/7/13 批之后，重开可写、准入判着、批大小 1；窄盘循环；普通盘循环：384/256/240 槽与 4 GiB、覆盖写 0/2/5/9/14、崩了再挂 0/1 次、之后覆盖写 0/2 次、卸载再挂之后覆盖写 0/1/2/4/8 次）。每个起点录一次卸载，前缀刀 n = 0…全长逐个判三种丢盘：读法三交集（卸载之前分配器记录里「已释放、释放代 > R」且单元头诞生代 ≤ R 的槽，∩ 这一刀持久了的单元写）与剩下那块盘只读挂载、整读每个活文件。三臂同一份原型、同一批起点：

- `research/prompts/m3-batch14-code-r1-opus-model/out/n1-rescan-summary.txt:1`「narrow ga: starts=1900 start_failed=1400 unmount_raised=1900 unmount_err=0 starts_with_intersection=0 intersection_slot_lines_sum=0 start_lost_pairs_with_old_root_read_failure=0 such_cuts=0」
- `research/prompts/m3-batch14-code-r1-opus-model/out/n1-rescan-summary.txt:2`「narrow imm: starts=1900 start_failed=1400 unmount_raised=1900 unmount_err=0 starts_with_intersection=174 intersection_slot_lines_sum=288 start_lost_pairs_with_old_root_read_failure=37 such_cuts=728」
- `research/prompts/m3-batch14-code-r1-opus-model/out/n1-rescan-summary.txt:3`「narrow held: starts=1900 start_failed=1400 unmount_raised=1648 unmount_err=252 starts_with_intersection=0 intersection_slot_lines_sum=0 start_lost_pairs_with_old_root_read_failure=0 such_cuts=0」
- 循环两族在同一份文件第 4–7 行：GA 400 + 186 个起点交集 0，IMM 1 + 13 个起点有交集。

IMM 与 HELD 两行和上一轮 `research/prompts/m3-unlink-code-r3-opus-model/out/sweeps-summary.txt` 第 1、2 行的数逐项相同（174、288、37、728；1648、252），原型移植到这份冻结副本上没走样。GA 这一行就是入库代码在这一族上的结局：卸载做成 1900 个、交集 0、读不出 0。

### 1.2 形 R：卸载半途崩溃之后的下一次可写挂载（打中，不分辨臂）

被攻的句子：GA 的理由写在 `/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/mount.rs:2341`「    /// （引用它的根都早于 g），丢哪块盘恢复都择不到引用它的根。挂载续做推满仍放不下之后卸载靠这一截做成（槽被续做吃光，整截扣着的话这一串在预演里」。它管的是卸载这一串自己的写。卸载这一串在任何根之前先写系统配置：`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/mount.rs:3006`「        match write_the_raised_floor_into_every_system_configuration(」。下一次可写挂载重建分配器时，F_生效 取根上与系统配置里的最大值（`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/recovery.rs:1881`「    let highest_in_the_system_configurations = reader」，第 1895 行「        .max(highest_in_the_system_configurations)」），回收门槛按它当场放开：`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/mount.rs:1846`「        reclaim_floor(effective_floor, oldest_valid_root_in_the_window),」、第 1847 行「        ReclaimedReuse::Immediately,」。恢复择根不看 F：`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/recovery.rs:1263`「/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7 择新；」。

于是：现行那一版 C 的根在盘 A，盘 B 上最新的根是 r < C。卸载在系统配置写了 F = C（哪怕只写成一块盘）、带新 F 的根还没落到盘 B 时掉电；用户照常两盘再可写挂载，恢复择到 C，重建分配器按 F_生效 = C 当场回收释放代 ≤ C 的槽——其中释放代在 (r, C] 的那些正是 r 引用的；写行那次发布（它的根按落点公式可能落在盘 A）的单元写进这些槽；暖机的根落到盘 B 之前再掉电，盘 A 丢了，盘 B 单独恢复择到 r，读到被写掉的单元。卸载那一截按 GA 一个都没写掉（下面各行 `inter_unmount` 全是 0），写掉它们的是再挂。

造法（用户决定的几步放开）：起点用 1.1 的窄盘族（单元区 176/208/240/272/304/336 槽、覆盖写 0/2/5/8、文件 1/4/14 单元、崩在第 0/1/3 批之后）与普通盘族（384/256/240 槽与 4 GiB、覆盖写 0/1/3/7、崩了再挂 0/1/2 次、每次之后覆盖写 0/1 次）；每个起点录一次卸载，卸载写流的前缀刀 n 取「至少一个系统配置写已落、根槽写没落完」那一截的每一刀（`ALL_CUTS=1` 可取全部）；每一刀上两盘都在、再可写挂载一次（窄盘族与起点同一种开关，普通盘族与 `crash_and_mount_writable` 同一个入口），录它的写流；再对那条写流的前缀刀 m 逐个丢盘 0、丢盘 1。判：剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的比（`same`；读出的是卸载之前两盘都在时的现行内容记 `newer`，不算不同；只读挂载或打开、读失败记 `differs[ro/open/read`）；另算交集，分「卸载那一截写的」「再挂写的」两份。

h2 那一份（GA，n = 1：只有盘 0 的系统配置写落了；再挂写流 38 个写，m = 6：取号两个系统配置写加写行那次的前 4 个单元写）：

- `research/prompts/m3-batch14-code-r1-opus-model/out/r-h2-ga-detail.log:4`「RCUT narrow U=176 k=0 u=4 b=3 cont=WalkedToEmpty n=1 sc=1 roots=0 m=6 next2=unit_write survivor_sc_floor=Some(14) RVerdict { lost: 0, chosen: Some(13), inter_unmount: 0, inter_remount: 1, read: "differs[ro:Open(Walk(MappingStillUnreadable { slot: SlotNumber(1411) }))]" }」
- 同一份起点卸载之前丢盘 0 读得出（基线：同一份日志 `RSTART` 行的 `baseline` 里键 0 是 `(Some(13), "ok", …)`）；这一形前缀刀里丢盘 0 读不出的 735 对：`research/prompts/m3-batch14-code-r1-opus-model/out/r-h2-ga-detail.log:810`「RTALLY narrow U=176 k=0 u=4 b=3 cont=WalkedToEmpty prefix lost=dev0 inter_unmount=0 inter_remount=1 read=differs[ro count=735」
- 被写掉的槽 1411 与上一轮丙1 打中的是同一个槽（树 65537 的节点，释放代 14），这一回写它的是再挂那次写行。

4 GiB 那一档（随机历史默认盘宽；第一个文件之后崩了再挂 1 次、覆盖写 1 次，现行 txg 6，盘 1 上最新的是 txg 4）：

- `research/prompts/m3-batch14-code-r1-opus-model/out/rp-ga-4-1.log:50`「RCUT plain W=4 k=0 m=1 k2=1 n=1 sc=1 roots=0 m=4 next2=unit_write RVerdict { lost: 0, chosen: Some(4), inter_unmount: 0, inter_remount: 1, read: "differs[open:ExtentTreeWalk(MappingStillUnreadable { slot: SlotNumber(50240) })]" }」——只读挂载做成，打开活文件时 extent 树走不下去。
- `research/prompts/m3-batch14-code-r1-opus-model/out/rp-ga-4-1.log:101`「RTALLY plain W=4 k=0 m=1 k2=1 prefix lost=dev0 inter_unmount=0 inter_remount=1 read=differs[open count=40」、第 102 行「RTALLY plain W=4 k=0 m=1 k2=1 prefix lost=dev0 inter_unmount=0 inter_remount=1 read=differs[ro count=424」。

四臂汇总（`research/prompts/m3-batch14-code-r1-opus-model/out/remount-summary.txt`，每臂一段，段首 `== rn <臂>` 是窄盘族、`== rp <臂>` 是普通盘族）：

| 臂 | 窄盘族（原样） | 普通盘族（原样） |
|---|---|---|
| GA | 第 8 行「starts=186 start_failed=30 unmount_raised=186 unmount_err=0 remounts=7253 remount_err=0 starts_with_lone_read_differs=13 of_which_with_remount_intersection=13」 | 第 45 行「starts=95 start_failed=1 unmount_raised=95 unmount_err=0 remounts=4219 remount_err=0 starts_with_lone_read_differs=9 of_which_with_remount_intersection=9」 |
| IMM（丙1） | 第 24 行：14 / 14 | 第 54 行：9 / 9 |
| HELD（改之前） | 第 32 行：卸载做成 147、被拒 39，9 / 9 | 第 63 行：8 / 8 |
| FX（零轮改法） | 第 36 行：0 / 0，再挂被拒 0 | 第 70 行：0 / 0，再挂被拒 0 |

「读不出」的每一对都同时有再挂那一截的交集（汇总里 `inter_remount=0` 的那几类没有一类是 `differs`）。普通盘族 `baseline_err` 那 14192 对出自卸载之前丢盘 0 就读不出的 8 个起点（覆盖写 0 次：剩下那块盘择到 txg 2 那条根，只读挂载报 `UnitUnreadable { slot: SlotNumber(0) }`），四臂相同，不算打中，见「顺手看到」。

层 0 枚举域（`enumerate_layer0_selecting_versions_observing_each_state`，再挂写流每段都展开，每个状态丢盘 0、丢盘 1 各判一遍；跑前 `closed_form_state_count` 算出每刀 20496，不超过约 10⁶），只跑 GA、h2 那一份、卸载刀 n = 1 与 n = 16（盘 1 系统配置写之后、第一条根之前）：

- `research/prompts/m3-batch14-code-r1-opus-model/out/renum-h2-ga.log:153`「RENUM narrow U=176 k=0 u=4 b=3 cont=WalkedToEmpty n=1 closed_form_states=20496 states=20511 observed=20511 checker_violated={}」
- 丢盘 0、只读挂载失败的三类：第 155 行「RENUMV narrow U=176 k=0 u=4 b=3 cont=WalkedToEmpty n=1 count=2048 RVerdict { lost: 0, chosen: Some(13), inter_unmount: 0, inter_remount: 1, read: "differs[ro:Open(Walk(MappingStillUnreadable { slot: SlotNumber(1411) }))]" }」、第 157 行 count=4096（交集 2）、第 159 行 count=6158（交集 3），合计 12302 个状态。
- n = 16：第 327 行「RENUM narrow U=176 k=0 u=4 b=3 cont=WalkedToEmpty n=16 closed_form_states=20496 states=20511 observed=20511 checker_violated={}」；第 329 行 count=4096、第 331 行 count=4099，合计 8195 个状态读不出。
- 两盘都在的状态上池级 checker 0 违例（`checker_violated={}`）：两盘都在时这一形看不出来，只有丢盘之后看得出。

#### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 不分辨。GA、IMM（丙1）、HELD（改之前那一版）同中（窄盘 13 / 14 / 9 个起点，普通盘 9 / 9 / 8 个，上表）；病根在三臂共用的那一步：可写挂载重建分配器按含系统配置 F 的 F_生效当场回收。只有我加的 FX 不中 |
| 被判的系统当时看不看得到判别它的东西 | 看得到。再挂重建分配器时手里同时有系统配置里的 F（`recovery.rs` 第 1881 行起那一段）与这一遍读出的每条根和它的槽位（同一函数里 `roots_with_ring_slots`，区域号就是盘），「F_生效 高于某块盘上最新的有效根」当场算得出——FX 就是这么算的 |
| 满足的是判据字面的哪一个分句 | 正文第五节 `research/prompts/_m3-batch14-code-r1-body.md:73`「- N1 判的是用户看得到的结局：丢一块盘之后，剩下那块盘只读挂载做不做得成、整读活文件与卸载之前读出的相同不相同。池级 checker 的条名在单盘镜像上饱和，不拿它分辨 GA 与丙1、改之前那一版（上一轮判决第二节）。」：只读挂载做不成（`differs[ro`）、打开活文件失败（`differs[open`）。但这段历史在卸载与丢盘之间多了「两盘再可写挂载、挂载里再崩」两步；GA 的条款字面（D16（发布语义） 已定项 1「正常卸载」那一行，原文整行抄在下面）只说「这一串」的单元写不能写掉剩下那块盘上最新的根引用的单元——这一形里「这一串」一个都没写掉，写掉它们的是下一次挂载按「生效」「可再分配」两行回收的槽。所以按「打中归错了判据」那一格核：它打中的是第五节的结局判据，不是 GA 那一句的字面；条款对「卸载半途崩溃之后的下一次可写挂载」没写剩下那块盘上的旧根保不保 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 第五节只写「改法进下一批」，没给候选。上一轮岔路单 `research/prompts/m3-unlink-code-r3-forks.md` 的三个候选：甲 GA 中（量过）；乙 回到改之前那一版中（量过，HELD）；丙 留着丙1、条款写明「系统配置的 F 高于剩下那几块盘上全部根时不承诺读出旧版」——在这一形上剩下那块盘系统配置里的 F 正是 14 而择到的根是 13（`r-h2-ga-detail.log` 第 4 行 `survivor_sc_floor=Some(14)`），按丙的条款这一格不算违约（推的：丙的条款没写成文，只按岔路单那一句读） |

「正常卸载」那一行原文（`.claude/kb/decisions/16-发布语义.md` 第 42 行，整行）：

```
| 正常卸载 | 推一串空发布把 F 抬到现行那一版的 txg（「抬 F 那一串」照准入的走法，上限见「抬 F 的上限」那一行）；这一串回收出来的槽，只有释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场可用（预演与真发都算上），其余与准入那一串一样扣到生效（恢复择根不看 F，丢一块盘时择的是剩下那块盘上最新的根，引用它的单元不能被这一串写掉）；这一串写的根在根记录 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）；第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次（现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次）；不加干净关闭标记，下次挂载照旧走恢复（D23（journal 的角色与格式） 已定项 14）；现行那一版树表 0 条（mkfs 之后只暖机、还没写过第一个文件）时一个字节都不写、F 不动、不推空发布，卸载照常做成（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile`） |
```

「生效」那一行原文（同一份文件第 39 行，整行）：

```
| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)；系统配置里读得出的取每块幸存盘两槽里全部自证过（校验和过且 fsid 与本池相同）的槽，与取号式子同一取法（D18（块里携带什么信息） 已定项 11） |
```

准入抬 F 没有这一形（推的）：准入的新 F 不高于「每块盘上最新的持久有效根」（同一份文件第 37 行「抬 F 的上限」那一行的准入式子），半途崩溃之后再挂按它回收，回收的槽不被任何一块盘上最新的根引用。卸载「不另判上限」，新 F = C 高于另一块盘上最新的根，这一格只出在卸载那一入口。

推翻条件：在 GA 副本上，形 R 的任何一刀丢盘之后剩下那块盘择到的根读得出，且读不出的状态能归到别的原因（比如卸载之前那块盘自己就读不出）；或者入库装置上同一段历史复跑不出 `differs`。

### 1.3 单盘恢复择中被抛弃的根（没打中）

造法：普通盘起点覆盖写 k 次；再录一次覆盖写，崩在它的根槽写已落、系统配置轮换没落那一刀（系统配置没见证它，C554 乙不拒可写）；在那一刀上可写挂载，挂载期间把那条根的根槽与那次发布的单元写读回全 0——恢复落到上一版、取新实例，那条根留在环里成了被抛弃的根；挂载交回之后照常读得出；在这次挂载的会话上再覆盖写 k2 次，然后录卸载，前缀刀 × 三种丢盘。取样：384/256/240 槽与 4 GiB、k = 1/2/3/5/8、k2 = 0/1/2/4，78 个起点（造不出 2 个）。

- `research/prompts/m3-batch14-code-r1-opus-model/out/abandon-chosen.txt:1`「abandon ga: cut_verdicts=9027 chosen_the_abandoned_root=0」
- 交集与读不出：`research/prompts/m3-batch14-code-r1-opus-model/out/n1-rescan-summary.txt` 第 8–10 行，三臂都是 0。

为什么择不到：被抛弃的根出现之后的那次挂载已经在每块盘上各落一条新实例的根（写行加暖机，`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/mount.rs:3564`「    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish_txg)];」那一段推到每块盘都落过为止），新实例的 txg 大于环里每一条根，卸载时每块盘上最新的根都不是被抛弃的；GA 把被抛弃的根滤掉（第 2380 行「                    !abandoned_by_table(root, table)」）只会让某块盘的最大值变小、当场放开的变少。要让被抛弃的根成为某块盘上最新的，得那块盘上新实例的根全读不出——丢盘之外再一个故障，不在单故障的射程里（推的）。

GA 文档里「若在回退目标那一版之前就释放了，被抛弃的那条时间线接着那一版走，也不引用它」（`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/mount.rs:2364`「///   有效的根定；若在回退目标那一版之前就释放了，被抛弃的那条时间线接着那一版走，也不引用它。」）字面不对：被抛弃的时间线可以复用回退目标之前释放的槽（C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）那一形）。它不影响 GA 的结论，理由同上一段；只是那句理由写错了（推的，没造被抛弃时间线复用旧槽的盘面）。

### 1.4 根环里读不出的槽、有洞（没打中，读代码）

卸载那一遍读根环（`readable_roots_rereading_ring_slots_known_to_hold_a_root_once`）对这个进程知道住着根的槽读坏就重读一次，仍坏就拒这次抬 F（`NewerStateStillUnreadableAfterOneReread`），GA 的最小值算不到；挂载那一刻就读不出、之后没写过的槽当没有根。各盘最新的根都是这次挂载自己写的（写行加暖机），所以最小值取的是这个进程写过、读过的根；一条读不出的根只会让那块盘的最大值变小。有洞（journal 施加出来的那一版不写根）同理：那块盘上最新的根更旧、最小值更小，当场放开的更少。恢复在剩下那块盘上择根之后按 journal 前缀往前走，走到的那一版比它择的根新，引用的槽释放代更高，GA 也不放开它们。没造读不出、有洞的盘面去量。

## 二、N3 Q-F 乙

### 2.1 ①b：同一份崩溃镜像各自续做（没打中）

被攻的句子：意图那次写调用方给的值（`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/transaction.rs:5882`「                write_time_seconds,」），各批照抄这条记录上一版的值（第 5945 行「                write_time_seconds: record.write_time_seconds,」）。挂载续做的 `previous` 从盘上读出来，读者只读偏移 64 那一格（`records.rs` 的 `InodeRecord::parse`），写者三格写同一个值。

造法（`MODE=qf`）：窄盘（176–336 槽 6 档）、覆盖写 0/3、文件 1/4/14 单元、意图之后 0/1/3 批就崩，意图那次给的时间取 1788100000、0、u64::MAX 三种；同一份崩溃镜像拷两份，各自可写挂载续做（批大小 1），两份盘逐字节比；续做每一步里这条记录还在的，写入时间与意图那次给的比；再在续做之后的盘上只读挂载读这条记录。

- `research/prompts/m3-batch14-code-r1-opus-model/out/qf-ga.log:326`「QFTALLY identical=true wrong_times_nonzero=false count=288」
- 288 份全部逐字节相同，续做里还带这条记录的 1275 步写入时间全等于给的值（同一份日志各 `QF` 行的 `steps_with_record` 加起来）；造不出的 36 份是批数不少于单元数（`UnlinkOfAnInodeThatIsNotALiveRecord`）。

### 2.2 模型不比时间留下的缝（打中缝，量过）

两刀，各在冻结副本的一份拷贝上打、跑相关的 harness 测试目标（`cargo test -p singlefs-harness --test …`，线程上限 7）：

- MQ1（`patches/mq1-session-drops-the-unlink-time.patch`）：会话那一层不把调用方的时间交下去，意图写 0（`/tmp/claude-1000/m3-batch14-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:257`「            write_time_seconds,」改成 `{ let _ = write_time_seconds; 0 }`）。四个经会话 unlink 的测试目标全绿：`research/prompts/m3-batch14-code-r1-opus-model/out/tests-mq1.log:55`「test result: ok. 25 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 13.55s」、第 94 行「test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s」、第 108 行「test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.46s」、第 128 行「test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.40s」。专门那条用例（`unlink_mount_continuation_and_the_emptied_version.rs` 第 552 行 `the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it`）直接调 `transaction::publish_unlink_intent`，不经会话，罩不到这一刀。
- MQ3（`patches/mq3-intent-keeps-the-previous-time.patch`，与变异表「删除批第 14 批 Q-F 乙：unlink 的意图照旧带这条记录上一版的写入时间，不写调用方给的」同一刀）：只有那一条专门用例红（`research/prompts/m3-batch14-code-r1-opus-model/out/tests-mq3.log:66`「test the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it ... FAILED」）；模型对拍的 8 条（第 57 行「test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.38s」）与挂载透明的 33 条（第 43 行）全绿。

缝有多大：随机历史与模型对拍对写入时间一个字节都不比，意图写什么值、各批抄不抄，只有那一条直接调事务层的用例判；会话、执行器（`history.rs` 按步号取的时间）这两层把时间丢了或换了，今天没有用例红（MQ1 量过；执行器那一层推的，没打刀）。改法（零轮）：MQ1 那一刀补一条经会话发 unlink、读盘上偏移 64/72/80 的用例，或让模型带写入时间。

### 2.3 钉了字节的地方（没打中，读代码）

`crates/singlefs-checker-tier/src/crash_injection.rs` 里经 `HistoryOperation::Unlink` 的两段历史（`the_publish_after_a_writable_mount_on_the_emptied_version_creates_an_inode_and_is_published` 与 `crash_injections_into_histories_reaching_every_mount_phase` 的第二段）只断言结局与计数（再挂次数、读出内容相同、恢复失败为 0、挂载透明判过的格数），不钉字节或摘要；随机历史的时间按步号取（`history.rs` 第 4662 行起），同一段历史两次跑相同。checker 档没跑，这一格是读代码的结论。

## 三、N2、N4–N6：错实现照样过不过

| 刀 | 改坏什么 | 结局（原样） |
|---|---|---|
| MG1（`patches/mg1-max-over-devices.patch`） | GA 的「各盘最新有效根」取最大而不是最小（实际退回丙1） | GC 两条红：`research/prompts/m3-batch14-code-r1-opus-model/out/tests-mg1.log:32`「test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.56s」 |
| MG4（`patches/mg4-remainder-not-reclaimed.patch`） | 卸载那一格释放代高于最小值的那一截不回收（GA 第二次 `reclaim_released_up_to` 删掉） | 四条红：`research/prompts/m3-batch14-code-r1-opus-model/out/tests-mg4.log:87`「test result: FAILED. 21 passed; 4 failed; 1 ignored; 0 measured; 0 filtered out; finished in 14.54s」 |
| MG2（没打刀，推的） | 不滤被抛弃的根 | 在可达状态上与原样等价（1.3：卸载时各盘最新的根都不是被抛弃的），用例罩不到也不用罩 |
| MQ1、MQ3 | 见 2.2 | MQ1 没有一条红；MQ3 只有专门那一条红 |

GC 两条用例（`a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` 第 250、318 行）判的是剩下那块盘只读挂载、整读活文件与卸载之前相同，是用户看得到的结局；它们只摆卸载那一串里的刀，罩不到形 R（卸载之后的下一次挂载），FX 那一条改法要另补用例。

## 四、改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| FX（`patches/fx-mount-holds-above-the-newest-root-on-every-device.patch`，打在 GA 之上）：可写挂载重建分配器时，按这一遍读出的根算「每块盘上最新有效根的 txg 取小」m；当场放开的只到 `reclaim_floor(min(F_生效, m), 窗口里最旧有效根)`，其余（到原来的门槛为止）扣住，暖机落满每块盘之后 `release_reclaim_holds` 放开；某块盘一条有效根都没读出时照原样 | 1.2 形 R | 量过（副本）：窄盘 186 个起点、普通盘 95 个起点读不出 0、再挂被拒 0（`out/remount-summary.txt` 第 36、70 行）；冻结副本相关 7 个 harness 测试目标在 FX 拷贝上全绿（`out/tests-fx.log` 第 30、62、101、109、131、151、164 行，`ignored` 的没跑）。推的：准入抬 F 半途崩溃那一形 m 不低于 F_生效，FX 不动它；扣住的那一截在取号之前的预演里不算，窄得多的盘上可能让再挂多拒（这一轮的取样里 0 次） |
| 条款补一句（推的）：D16（发布语义） 已定项 1「正常卸载」那一行的括注写明射程——卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载，回收也只放开到每块盘上最新有效根取小，其余扣到那次挂载的暖机落满每块盘 | 1.2 四句第三行：条款对「下一次挂载」没写 | 推的 |
| GA 文档第 2364 行那句理由改写：被抛弃的时间线可以复用回退目标之前释放的槽，GA 不算被抛弃的根靠的是「挂载的暖机先在每块盘上落了新实例的根」 | 1.3 | 推的 |
| 补一条经会话 unlink、读盘上三个时间秒的用例（MQ1 那一刀要红） | 2.2 | 推的 |

| 改法 \ 打中的格 | 形 R 窄盘 | 形 R 普通盘（含 4 GiB） | 上一轮四族（卸载那一串自己） |
|---|---|---|---|
| 甲 GA（入库） | 中，量过（13 个起点） | 中，量过（9 个） | 不中，量过 |
| 乙 回到改之前那一版（HELD） | 中，量过（9 个） | 中，量过（8 个） | 不中，量过；卸载被拒 252 个 |
| 丙 留着丙1 加条款 | 中，量过（IMM 14 个）；按丙的条款字面不算违约，推的 | 同左（IMM 9 个） | 中，量过（174 个） |
| FX（GA 加我的改法） | 不中，量过（0） | 不中，量过（0） | 不中，推的（FX 不动卸载那一串，与 GA 相同） |

## 没打中的形状

- 上一轮四族（窄盘 1900、窄盘循环 186、普通盘循环 400）在入库 GA 上：交集 0、读不出 0、卸载被拒 0。普通盘不循环那一族（上一轮 553 个起点、丙1 下也是 0）这一轮没重跑。
- 被抛弃的根：78 个起点、9027 个判定，择中 0 次。
- N3 ①b：288 份崩溃镜像两次续做逐字节相同；1275 步写入时间都对。
- MG1、MG4 两刀都被入库用例抓到。
- 形 R 里丢盘 1 那一边也有读不出的（现行那一版的根在盘 1 上的起点），GA 窄盘族 276 对（`out/remount-summary.txt` 第 2–7 行那一段），与丢盘 0 同一个机制。

## 这条腿自己的限度

- 一次抽样；没打中的格照规则要再抽一次。
- 形 R 只取了卸载写流里「已有系统配置写、根槽写没落完」那一截的刀，再挂写流只取前缀刀；层 0 枚举只跑了 GA、h2 一份的两刀（各 20511 个状态，复跑又各一遍），4 GiB 那一份没枚举。层 0 枚举合计 4 × 20511 = 82044 个状态（首跑 2 刀加复跑 2 刀），远低于约 10⁷。
- 形 R 的「读不出」按只读挂载加整读活文件判；删除中的文件（窄盘族的第一个文件 nlink = 0）不整读，它的单元被写掉看不到。
- 交集用的是上一轮的读法三（卸载之前分配器的账加单元头），没另起读法二、读法一；判打中靠的是只读挂载与整读的结局，交集只作说明。
- 线程上限：收到改成 7 的消息之后有两次越界——普通盘族 HELD 那一批一次起了 9 件（时长没记；同一档 GA 6 件那一批 24 秒）；N1 窄盘重扫 GA 那一批把覆盖写也当成件拆开，一次起了 66 件、跑了约 61 秒（`user` 约 27 分钟）。两批结果都完整（.rc 全是 0），后面的批都拆到 7 件以内；复跑脚本里每批不超过 7 件。
- 有四次单条命令被转成后台：形 R 的 GA 窄盘第一批（5 分 22 秒）、h2 的层 0 枚举两刀一起跑那次（4 分 43 秒）、FX 普通盘族前两批连跑那次、上面那 66 件那一批（命令的前台时限误写成 10 秒）；我没停，等完成通知读了结果，之后都拆小。
- 内存包装 252（排不上，命令没跑）：N1 重扫 HELD 那一批 3 件，加了等待秒数之后单独重跑了那 3 件；编原型一次 252，同样加等待重编。
- 第一批形 R（GA 窄盘，四档覆盖写一起）的日志出自分类细化之前的二进制，那一批已删掉、用细化之后的二进制重跑（`rn-ga-*`）；`out/renum-h2-ga.log` 出自加 `survivor_sc_floor` 之前的二进制，那一项只出现在 `RCUT` 行，`RENUM` 行格式不变，复跑比对相同。
- FX 只在 7 个 harness 测试目标上跑过，没跑整包 harness，也没跑 checker 档。

## 顺手看到、不在这一轮判的

- 被抛弃的根的账读不出时卸载被拒：78 个起点里 11 个卸载报 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot …)`，在任何写之前，三臂相同（`out/abandon-chosen.txt` 第 2 行）。造法里那条根的单元在取新实例的那次挂载期间读回全 0，那次挂载的写行与暖机可能写进了它的槽（C314 那一形），之后卸载算影子账读那条根的账读不出就拒。这几份盘面上正常卸载做不成，没查是不是一直做不成。
- 普通盘族覆盖写 0 次、没崩了再挂的 8 个起点上，卸载之前丢盘 0 就读不出：剩下那块盘择到 txg 2 的根、只读挂载报 `UnitUnreadable { slot: SlotNumber(0) }`（树表 0 条那一版在单盘上读不出，不是卸载造成的），四臂相同。

## 要主 agent 跑的命令

没有必须的。要核形 R：`bash research/prompts/m3-batch14-code-r1-opus-model/rerun.sh <阶段> <新的空目录>`，按脚本里 `STAGES` 的次序逐段跑（工作目录任意；内存上限写在 `scripts/run.sh`（2G）与 `scripts/build.sh`（8G）；线程 `ATTACK_THREADS`，默认 7）；最后 `compare` 那一段看 12 行是不是全 ✓。只核形 R 的最小一组：`build-ga`、`r-h2-detail`、`compare`（看 r-h2-detail 那一行）。

## 没做什么

- K1、K3 不归这条腿；N2 只攻了「错实现照样过」那一面，N4–N6 的文档与注释没逐处核。
- 没跑 checker 档、没跑整包 harness、没跑 `random_histories`；Q-F 执行器那一层没打刀。
- 副本上的数不进 kb；形 R 与 FX 的数都在拷贝上量。
- 交回前删掉草稿目录里的仓副本与编译目录（ga、imm、held、fx、qf、rerun 与各 `target-*`），日志留在 `/tmp/claude-1000/m3-batch14-code-r1-opus/out/`（模型目录 `out/` 只收了汇总与几份代表日志）。

# 根环实现第 10 批代码轮第二轮：云端攻方腿报告（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 Q3 Q4 rr2 G3 G4 H1 H2 L1 L2 -->

这条腿是同一攻方腿的接续：上一条在客户端重启时被停，报告一行没写；它在草稿目录留下的 A、B、C、D 四段原型日志我核过（源码、开关、日志与 `.rc` 对得上，见第五节「现场核对」），没重做，接着补了「入库用例在三条臂下红不红」与层 0 崩溃态两段。
所有数都在冻结副本的拷贝 `/tmp/claude-1000/m3-batch10-code-r2-opus/tree/` 上量，是副本上的数，不进 kb。代码行号一律引冻结副本 `/tmp/claude-1000/m3-batch10-code-r2/tree/`。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-batch10-code-r2-opus-model/`。原型是两份：A、B、C、D 四段在 harness 测试目标 `opus_r2_rr2_attack.rs`（上一条腿写的）；L1、L2 与卸载之前那一份的丢盘读回在独立的二进制包 `opus-r2-crash-states`（这一条腿写的，依赖 checker 档库，调 `crates/singlefs-checker-tier/src/crash.rs` 的 `enumerate_layer0_versions` 与同一份文件里每一段都展开的观察者入口）。两份都只在冻结副本的拷贝上编、跑。

| 文件 | sha256 |
|---|---|
| `mount-switches.patch`（上一条腿留的，只有两个开关，已被下一行盖过） | acf48778cf828ad7d0d55f3fc38ca758e504beb05638a25adf44e912f0e14952 |
| `mount-switches-full.patch`（副本 `mount.rs` 相对冻结副本的全部改动：`SINGLEFS_OPUS_RR2_OFF`、`SINGLEFS_OPUS_ATTRIBUTE_BY_SLOT`、`SINGLEFS_OPUS_REREAD_REFUSES`） | 0b57e3be8477ba575fc567f88ae6a91e690402df88b27e04240e93bfc8a6b56c |
| `workspace-member.patch`（副本工作区 `Cargo.toml` 加一个成员） | faf829c320ef9a1716cfcdc87b405dde785d17080c364f9678acdec4697da5be |
| `opus_r2_rr2_attack.rs` | e9e4cbf82116571ed23b700fb564c5f201f21f1fcc03f4b7462bbeb971e753e6 |
| `opus-r2-crash-states/Cargo.toml` | e1c9b4cad2acf446b6776817262174d7d12e7d17123b3ceab8c8a463ac40ecb7 |
| `opus-r2-crash-states/main.rs` | 2bbd01474e4342b48a43ed0ecc6f7ed023fed1bf8e75013568ae67eea87f87aa |
| `scripts/run-bin.sh` | 10ec982c7558a444e954041d6617c77a460e3b1f9c553b78a16f7d77a88a4766 |
| `scripts/run-proto.sh` | 659538a13656ff60e83d4f580580eca517c564c9fbd3f7c8a3624501360600c0 |
| `scripts/run.sh` | 4df71920a58053b72f02e062d74e15600bc57862def00905a8b04afbede5b330 |
| `scripts/run-test-bin.sh` | e26b4aeb15698dc5254cc903168dd4e8fdc994c2c766fd65c7e2eca26f1b5a9d |
| `scripts/agg-a.py` | 9bfe1c91a799f867c7db486a1fcbab671fef4837fd83f45aa26c92b1f23cc4f3 |
| `scripts/agg-b.py` | 7c8ffa080055c9530eac9140ef822aab48714d1bdf6e54426e7b1bd07bfca9ef |
| `logs.tar.gz`（报告引的全部原样日志：`a/ b/ t/ l/`、`c-*.log`、`d.log`、两份 summary） | e9ab098b12490fd2ec48d4247a977eec372e0645fe60b90c6fe54b4991163cfa |

复跑（`<W>` 取 `/tmp/claude-1000/m3-batch10-code-r2-opus`：四份 `scripts/*.sh` 第 4 行写死了 `cd <W>/tree`，放到别处要改那一行；每条 cargo 与每个二进制都经 `research/scripts/run-with-memory-cap.sh`，线程经 `research/scripts/capped.sh`）：

```
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-batch10-code-r2/tree/ <W>/tree/
cd <W>/tree && patch crates/singlefs-core/src/mount.rs < <模型目录>/mount-switches-full.patch && patch Cargo.toml < <模型目录>/workspace-member.patch
cp <模型目录>/opus_r2_rr2_attack.rs crates/singlefs-harness/tests/ && mkdir -p crates/opus-r2-crash-states/src && cp <模型目录>/opus-r2-crash-states/Cargo.toml crates/opus-r2-crash-states/ && cp <模型目录>/opus-r2-crash-states/main.rs crates/opus-r2-crash-states/src/
bash research/scripts/run-with-memory-cap.sh 3G bash research/scripts/capped.sh 12 cargo test --release --offline -p singlefs-harness --test opus_r2_rr2_attack --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root --test writable_mount_of_a_formatted_pool --test writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring --no-run
bash research/scripts/run-with-memory-cap.sh 1800M bash research/scripts/capped.sh 12 cargo build --offline --release -p opus-r2-crash-states
```

各段（臂由环境变量选：不设是 rr2_on，`SINGLEFS_OPUS_RR2_OFF=1`，`SINGLEFS_OPUS_ATTRIBUTE_BY_SLOT=1`）：

| 段 | 命令（在 `<W>` 下） | 看哪几行 |
|---|---|---|
| A | 每个起点、每臂一件：`SINGLEFS_OPUS_POINT=<起点> scripts/run-bin.sh 400M <日志> --nocapture --test-threads=1 a_one`（起点名见 `a-summary.txt` 第一栏，13 个） | `python3 scripts/agg-a.py` 与 `a-summary.txt` 逐行比 |
| B | `SINGLEFS_OPUS_B_D=<0/1> SINGLEFS_OPUS_POINT=<起点> [SINGLEFS_OPUS_B_SHARD=<i>/4] scripts/run-bin.sh 1G <日志> --nocapture --test-threads=1 b_reread`，5 个起点 × 两块盘 × 两臂 | `python3 scripts/agg-b.py` 头 8 行 |
| C | `scripts/run-bin.sh 1200M <日志> --nocapture --test-threads=1 c_q3`；错实现那一份前面加 `SINGLEFS_OPUS_REREAD_REFUSES=1`、上限 150M | `OPUS-C1-SUM`、`OPUS-C2` |
| D | `scripts/run-bin.sh 400M <日志> --nocapture --test-threads=1 d_live` | `OPUS-D` |
| 入库用例 | `scripts/run-test-bin.sh 1G <日志> <用例文件名> --test-threads=2`，五个文件 × 三臂 | `test result` 行 |
| L1 | `scripts/run-proto.sh 2G 2 <日志> l1`，三臂 | `OPUS-L-TALLY`、`OPUS-L1-SUM`；三份去掉结局、钩子、读次数三栏后 `cmp` |
| L2 计数 | `SINGLEFS_OPUS_FROM=0 SINGLEFS_OPUS_TO=0 scripts/run-proto.sh 4G 14 <日志> l2` | `OPUS-L-TALLY`、`OPUS-L-OBSERVED` |
| L2 rr2_off | `SINGLEFS_OPUS_RR2_OFF=1 SINGLEFS_OPUS_K=6 SINGLEFS_OPUS_SKIP_TALLY=1 SINGLEFS_OPUS_WORKERS=4 SINGLEFS_OPUS_B_STEPS=2 scripts/run-proto.sh 6G 12 <日志> l2` | `OPUS-L2-SUM` |
| L2 rr2_on | 五段，`SINGLEFS_OPUS_FROM` / `TO` 取 0–2741、2741–4111、4111–5482、5482–6852、6852–8223：`SINGLEFS_OPUS_K=6 SINGLEFS_OPUS_SKIP_TALLY=1 SINGLEFS_OPUS_WORKERS=28 SINGLEFS_OPUS_B_STEPS=2 scripts/run-proto.sh 8G 28 <日志> l2` | `OPUS-L2-SUM` 五段相加 |
| 卸载之前的丢盘读回 | `scripts/run-proto.sh 1G 2 <日志> d` | `OPUS-D-BEFORE` |

## 各格判定一览

| 格 | 判定 | 一句话 | 依据在 |
|---|---|---|---|
| K2-1 rr2 的触发与取 m 是不是同一处定义 | **打中（读法错）** | 同一处定义，但那一处按 txg 落点公式把根归到盘；mkfs 在三个区域各写一份第 0 代根，公式把三份都归到盘 0。环里只有第 0 代根时盘 1 实际读出了有效根，触发照样认成「盘 1 没读出」去重读。实现员读法 ① 的前提「只有 mkfs 那条根、落在一块盘上」与盘面不符 | 第一节 H1 |
| K2-2 「新池与年轻的环每次可写挂载都重读」的射程 | **打中（原因写错，射程写大）** | 只在环里只剩第 0 代根时重读：mkfs 之后、第一次可写挂载的 txg 1 根还没落到盘 1 的那几个崩溃态；之后一次都不重读 | 第一节 H1 |
| K2-3 有没有用例因此钉错 | **打中** | `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread` 把公式归盘造出来的那次重读钉成 `OnTheOneReread`，注释写的理由与盘面不符；按槽归盘的臂下五个用例文件里只有这一条红，rr2 自己的 14 条用例全绿 | 第一节 H1 |
| K2-4 重读比第一遍坏时多拒 | 没打中（与读法 ② 写的一样大） | 865 条臂里 rr2 比改之前多拒的 20 条全是「重读那一遍整环都坏」那一形，报 Q3；别的「重读更坏」形状都挂上 | 第二节 |
| K2-5 回收门槛越过盘上真住着、可回退到的有效根 | 没打中 | B 段 865 条臂、挂上之后每条再发至多 16 次改动；L2 rr2 用例那段卸载的 8223 个层 0 崩溃态 × 9 条臂、挂上之后 2 次改动：两块盘的池级 checker 0 违例，F 及以上的根丢盘整读 0 坏 | 第二节 |
| K2-6 重读之后三样用不用同一遍读数 | 没打中 | 根环表、回收门槛、影子账都取重读那一遍的 `ring_slot_readings` | 第二节 |
| K2-7 rr2 用例的丢盘整读断言罩着什么 | **打中（判别力缺口）** | rr2 用例两个起点丢任一块盘之后只读挂载都没有活文件，「丢盘整读与崩溃之前相同」那一项比的是两个空表 | 第一节 H2 |
| K1-1 Q3 复算在没注读故障的合法镜像上为真（假红） | 没打中 | 18 张起点镜像与第一次可写挂载的 33 个层 0 崩溃态上复算都为假 | 第三节 |
| K1-2 第三截 Q3 报红那一支摆不摆得出 | 线索（推的，量过一形） | 择得到根的镜像上复算恒为假（代码注释自己写了「推的」），择不到根时真挂载报 `Recovery`、不是 Q3：第三截 Q3 那一条报红分支在任何实现下都摆不出；错拒 Q3 只由第二截抓（量过 mkfs 那一形），二次崩溃镜像上的错拒第三截不报。与 G4 同口径，是既有设计 | 第三节 |
| K4-1 读序号后挪一遍根环之后的装置 | 线索（推的，要主 agent 跑 checker 档） | 随机历史起点 `AfterMakeFilesystem` 的第一次可写挂载正是只有第 0 代根的环：多读 24 次、多调一次钩子，按「步内第几次读」「整池第 n 次读」摆的读故障在这一步里落到别的读上 | 第四节 |
| K4-2 第三截与第二截是不是同口径 | 没打中 | Q3 在第三截的计数与取舍逐行照 G4 那一支接 | 第四节 |

## 一、K2：rr2 的触发（H1）与它的用例（H2）

### H1　只有第 0 代根的环上，「盘 1 没读出有效根」是落点公式造出来的

**机制（读码）。**

- mkfs 在三个区域的槽 0 各写一份第 0 代根：`crates/singlefs-core/src/make_filesystem.rs:669`「/// mkfs 在根环上种第 0 代根的那几个槽：三个区域各自的槽 0。`make_filesystem` 按它写，」，`crates/singlefs-core/src/make_filesystem.rs:619`「for ring_slot in genesis_root_ring_slots() {」。
- 区域归盘：`crates/singlefs-core/src/make_filesystem.rs:61`「/// 第一版两块盘时根环三个区域的归属：区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0（D2（RAID 条带策略） 已定项 7，写死、不是 mkfs 参数）。」。所以盘 1 的区域 1 槽 0 上住着一份自证得过的第 0 代根。
- 重建分配器判「每块盘都读出有效根」时，根归哪块盘不看它从哪个槽读出来，按 txg 的落点公式算：`crates/singlefs-core/src/mount.rs:1922`「// 一条根落在哪块盘由根环落点公式定（下面判「每块盘都读出了有效根」与当场放开到哪儿都用它）。」，`crates/singlefs-core/src/mount.rs:1924`「let target = target_for_publish(txg, slots_per_region);」，公式是 `crates/singlefs-core/src/root_ring.rs:114`「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」，取 m 那一处的比较是 `crates/singlefs-core/src/mount.rs:2675`「&& device_of_txg(root.checkpoint_txg) == *device」。txg 0 落到区域 0、盘 0：三份第 0 代根全归盘 0，盘 1 一条都没有。
- 触发与取 m 确是同一处定义（`crates/singlefs-core/src/mount.rs:1929`「// 这一遍每块盘都读出了一条不被抛弃的自证有效根（与下面当场放开到哪儿那一步取 m 同一处定义 [`newest_valid_root_txg_on_every_device`]）。」），K2 问的「是不是同一处」答是；毛病在那一处的归盘法本身，两边一起继承。
- 用户定的候选原文（`research/prompts/m3-batch10-code-r1-forks.md:12`）：「| Q4 重建那一遍有一块盘一条有效根都没读出时要不要也整环重读 | 照今天只在两块盘都没读出时重读；rr2：任一块盘没读出就重读 | 某个候选在只坏重建那一遍的历史上多拒；或在重读之后让 checker 红、丢盘读回坏 | 两个候选在同一组历史上各量过被拒次数、checker 红、丢盘读回坏，层 0 段也跑过 | 用户已定（2026-10-05）：采用 rr2 |」。只有第 0 代根的环上盘 1 读出了有效根，照这句字面不该重读；实现照样重读。
- 实现员读法 ①（正文第二节转述的）与钉它的用例注释：`crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:495`「/// 只做过 mkfs 的环里只有 mkfs 那一条根，落在一块盘上，另一块盘一条有效根都没有。」。盘面上那条根落在两块盘上（三份），这句与盘面不符。

**三条臂。** 副本 `mount.rs` 上两个开关（`mount-switches-full.patch`）：`rr2_on` 是冻结副本原样；`rr2_off` 退回 rr2 之前的触发；`slot` 把触发与取 m 两处的归盘改成「按读出它的那个槽所在区域归盘」（`opus_newest_valid_root_txg_on_every_device_by_the_slot_read`），别的不动。

**量 1：A 段年轻环起点（上一条腿跑的，日志我核过）。** 起点是 mkfs 之后、第一次可写挂载在第 w 次写起掉电（w 扫 1..=12，去重后 11 个）、第一次挂载做完，共 13 个。不注错挂一次的那一行（`/tmp/claude-1000/m3-batch10-code-r2-opus/a/` 下 on-、off-、slot- 三份日志；slot 臂的行标签仍印 `rr2_on`，因为标签只看 `SINGLEFS_OPUS_RR2_OFF`，文件名与同一份日志里 `OPUS-A-IMAGE … attribute_by_slot=true` 区分）：

```
OPUS-A-BASE rr2_on point=mkfs roots={0: [0, 0], 1: [0]} outcome=Ok(reread) hook_calls=1 reads=3527 reads_per_slot_min=Some(15) max=Some(15)
OPUS-A-BASE rr2_off point=mkfs roots={0: [0, 0], 1: [0]} outcome=Ok(first) hook_calls=0 reads=3503 reads_per_slot_min=Some(14) max=Some(14)
OPUS-A-BASE rr2_on point=mkfs roots={0: [0, 0], 1: [0]} outcome=Ok(first) hook_calls=0 reads=3503 reads_per_slot_min=Some(14) max=Some(14)
```

（第三行是 slot 臂。`roots` 是按读出的槽归盘的自证有效根的 txg：盘 0 两份、盘 1 一份，全是 txg 0。）13 个起点上 rr2_on 重读的恰是 `roots` 全是 txg 0 的那 5 个（mkfs、w = 2、3、4、5）；w ≥ 6 之后盘 1 上有了 txg 1 的根（`roots={0: [0, 0], 1: [1]}`），三臂都不重读。三臂不注错挂完之后的盘面哈希逐个相同（`a-summary.txt` 里 13 行 `image SAME`）。

**量 2：层 0 崩溃态（这一条腿跑的，L1）。** 流是 mkfs 之后第一次可写挂载（`mount_writable`）整条录下来，12 次写、7 段，`closed_form_state_count` 算 18；`enumerate_layer0_versions`（每一段都展开）实际枚举 33 个（原地覆写的撕裂态另算，闭式不计它们）。每个崩溃态上不注错可写挂载一次。原样输出（`l/l1-on.log`、`l/l1-off.log`、`l/l1-slot.log`）：

```
OPUS-L-TALLY rr2_on stream=first-writable-mount states=33 oracle_violations_with_empty_versions=0 checker_violated={} checker_first={}
OPUS-L1-SUM rr2_on only_genesis=false outcome=Ok(first) hooks=0 checker_after_nonempty=false q3=false n=21
OPUS-L1-SUM rr2_on only_genesis=true outcome=Ok(reread) hooks=1 checker_after_nonempty=false q3=false n=12
OPUS-L1-SUM rr2_off only_genesis=true outcome=Ok(first) hooks=0 checker_after_nonempty=false q3=false n=12
OPUS-L1-SUM slot only_genesis=false outcome=Ok(first) hooks=0 checker_after_nonempty=false q3=false n=21
OPUS-L1-SUM slot only_genesis=true outcome=Ok(first) hooks=0 checker_after_nonempty=false q3=false n=12
```

33 个崩溃态里 rr2_on 重读的恰是环里只有第 0 代根的 12 个，一个不多一个不少；slot 臂 0 个。三臂逐状态比挂载之后的盘面哈希与 checker（去掉结局、钩子次数、读次数三栏之后整行比）：`cmp` 三份全同（输出 `all-after-images-and-checker-equal`）。也就是这次重读除了多读、多调一次钩子，挂载写下的东西一个字节都没变。

**量 3：五个入库用例文件在三臂下（副本，release，这一条腿跑的，`t/` 下日志）。**

| 用例文件 | rr2_on | rr2_off | slot |
|---|---|---|---|
| `a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable` | 8 过 | 7 过 1 红 | 7 过 1 红 |
| `rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read` | 6 过 | 5 过 1 红 | 6 过 |
| `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root` | 8 过 | 0 过 8 红 | 8 过 |
| `writable_mount_of_a_formatted_pool` | 15 过 | 15 过 | 15 过 |
| `writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring` | 5 过 | 5 过 | 5 过 |

slot 臂红的那一条是 `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread`（`slot-a_mount_…log`：「test the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread ... FAILED」），它钉的是 `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:518`「root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,」。rr2 为之而设的 14 条用例在 slot 臂下全绿：按槽归盘保住了 rr2 要的那件事，只拿掉了只有第 0 代根时那一次重读。

**四句。**

1. 分不分辨臂：分辨。rr2_on 与 slot 在同一组盘面上结局不同（上面三量），rr2_on 与 rr2_off 在这几个盘面上也不同。
2. 系统当时看不看得到判别它的东西：看得到。重建分配器手里就有每条根是从哪个槽读出的（`roots_with_ring_slots`），归盘按槽算不要多读一次。
3. 满足判据的哪一句：正文第五节「第二节实现员交出的三处读法，腿判出哪一读法错或代价比写的大，算打中」——读法 ① 的理由（「落在一块盘上」）错；射程「新池与年轻的环每次可写挂载」写大了：只在环里只剩第 0 代根时，第一次挂载的 txg 1 根一落到盘 1 就停。也满足 K2 那一问「有没有一条用例因此钉错」：上面那条用例的名字说「does_not_reread」，断言钉 `OnTheOneReread`，注释给的理由与盘面不符。
4. 改法在打中的几格上中不中：

| 改法 | 只有第 0 代根时重读 | 那条用例 | rr2 的 14 条 | 挂载写下的盘面 |
|---|---|---|---|---|
| 甲：归盘按读出的槽算（副本 slot 臂） | 不重读（量过：A 5/5、L1 12/12 不重读） | 要改回 `OnTheFirstRead`（量过：slot 臂下它红） | 全绿（量过） | 不变（量过：L1 33 态、A 13 起点） |
| 乙：留着公式，把 txg 0 的根当作每块盘都有 | 不重读（推的，没实现） | 同甲（推的） | 推的：只碰 txg 0，不碰 14 条的起点 | 推的 |
| 丙：照今天，改注释与用例名 | 照旧重读 | 注释与名字对上（推的） | 不碰 | 不变 |

甲、乙是这条腿提的，只在我的模型上量过（乙连量都没量），被攻过零轮。丙不改代码，代价是读法 ① 那一行要改写成「只有 mkfs 那几份第 0 代根时，公式把它们都归到盘 0，触发认成盘 1 没读出」，交用户时连同下面 K4 那一项（这一步多读 24 次、多调一次钩子）一起写。

**什么现象会推翻 H1。** 只有第 0 代根的环上按槽归盘时盘 1 读不出自证有效根（上面 `roots={0: [0, 0], 1: [0]}` 那一栏是错的）；或者存在一种盘面，按槽归盘与按公式归盘对同一组读数给出不同的 m 而后者才对（即某块盘某个槽里住着一条 txg 不归它的根，而那条根丢掉另一块盘之后读不回）。第二种我没找到：除了 mkfs 的三份第 0 代根，根槽都按 `target_for_publish` 写。

### H2　rr2 用例的「丢盘整读与崩溃之前相同」在两个起点上比的是两个空表

rr2 那个用例文件每条用例判六项，其中一项是 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:531`「/// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与崩溃之前同一块盘上读出的相同。」，预期 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:660`「lone_survivor_reads_what_it_read_before_the_crash: [true, true],」。整读跳过删除中的 inode（同一文件 `:490`「if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」）。两个起点丢任一块盘之后只读挂载读到的那一版，活文件都是零个（原样输出：卸载之前那一份 `l/d-before.log`，崩溃之后那一份 `d.log`，216 那个起点崩溃之前与之后是同一份盘面）：

```
OPUS-D-BEFORE point=192:9:3:1-before-the-unmount lost=0 root_txg=19 live_files=Ok([]) inode_records(inode,nlink,size)=[(1, 0, 65268)]
OPUS-D-BEFORE point=192:9:3:1-before-the-unmount lost=1 root_txg=21 live_files=Ok([]) inode_records(inode,nlink,size)=[]
OPUS-D point=216:7:5:4-now lost=0 root_txg=22 live_files=Ok([]) inode_records(inode,nlink,size)=[(1, 0, 32634)]
OPUS-D point=216:7:5:4-now lost=1 root_txg=23 live_files=Ok([]) inode_records(inode,nlink,size)=[]
```

所以那一项在这两个起点上只核「再挂之后丢盘只读挂载还挂得上、仍是零个活文件」，回收门槛越过一条盘上有效根、把它引用的单元写掉，这一项看不见（它只读丢盘之后择到的那一版，而那一版没有要读的字节）。这几条用例的判别力在第一项（拿掉 rr2 时取号之前被拒），这一点不受影响；受影响的是「重读之后没有越过有效根」那一半没有会红的东西罩着，只剩两块盘都在时的池级 checker。

- 四句：不分辨臂（rr2_on、rr2_off 在这一项上都绿，量过：rr2_off 下这 8 条红在第一项，不在这一项）；不是被判系统的判据，是用例的；满足的是 K2「有没有一条用例因此钉错」的弱读法（钉的东西是空的）；改法：起点换成丢盘之后还有活文件的（B 段那三个 `live-crash` 起点丢盘 0 后各有 98002、65368、2999 字节的活文件，`d.log`），或把那一项改成「环里每条自证有效根丢盘整读与挂载之前相同」（B 段与 L2 段原型的 `broken_roots` 就是这么判的）。两个改法都是推的、被攻过零轮。
- 推翻条件：两个起点上丢盘之后只读挂载读到的版本里有 nlink ≠ 0 的 inode。

## 二、K2：重读比第一遍坏（多拒与不安全方向）

**B 段（上一条腿跑的，日志我核过）。** 5 个成熟窄盘起点：rr2 用例那两个（`192:9:3:1-n8`、`216:7:5:4-now`），加三个会话崩掉的（`264:6:4`、`192:9:3`、`384:20:2`，崩前不卸载）。重建那一遍是每槽第 6 次读（五个起点各自探出来，`OPUS-B-POINT … rebuild_ordinal=6`）。每个起点的臂：第一遍盘 d（0、1）的每个根环槽读错或读回全 0，重读那一遍按 `none`、`same`、`other-all`、`whole`、`other-but-newest`、逐个有根的槽 `one:*:Fails / Zeros` 坏；加一条不注错的。挂上的再发至多 16 次改动（有活文件覆盖写，没有就建 inode），每一步核两块盘的池级 checker 与「环里每条自证有效根丢盘 0、丢盘 1 整读与挂载之前相同」（F 及以上的算受保护）。rr2_on 与 rr2_off 两臂。`agg-b.py` 原样输出：

```
records 865 with both arms 865
findings with violations or protected broken: 0
off=Err(PlacementRefusedBeforeAcquisitionMountAdmissionUndecided) on=Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread) n=8
off=Err(PlacementRefusedBeforeAcquisitionMountAdmissionUndecided) on=Err(PlacementRefusedBeforeAcquisitionMountAdmissionUndecided) n=16
off=Err(PlacementRefusedBeforeAcquisitionMountAdmissionUndecided) on=Ok(reread) n=384
off=Ok(first) on=Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread) n=12
off=Ok(first) on=Ok(first) n=5
off=Ok(first) on=Ok(reread) n=440
```

rr2_on 报 Q3 的 20 条逐条是「重读那一遍整环都坏」（`reread=whole`）：5 个起点 × 盘 0、盘 1 × 第一遍读错、读回全 0，正好 20；20 条盘面逐字节不变（`image_unchanged=true` 20 次）。其中 12 条改之前挂得上（第一遍另一块盘读得出、不重读），这是 rr2 多拒的全部；另 8 条改之前也拒（取号之前、落点）。别的「重读比第一遍坏」形状（重读时另一块盘全坏 `other-all`、另一块盘只剩最新那条 `other-but-newest`、逐个槽坏）一条都没有多拒。这与实现员读法 ② 写的一样大：「第一遍一块盘读出、重读那一遍整个环一条都没有」才报 Q3，且要两次瞬时读错（第一遍一块盘、重读整环）。

**A 段：只注一次瞬时读错时多不多拒（上一条腿跑的，汇总这一条腿重算）。** 13 个年轻环起点，整池第 n 次读报错或翻一位，n 扫遍那次挂载的每一次读（rr2_on 92004 次挂载，rr2_off 与 slot 各 91764 次）。三臂每个起点、每种错的拒因计数逐项相同（重算输出：「points x faults 26 with err tallies differing across arms 0」），Q3 三臂都是 0。只注一次读错时 rr2 一次都不多拒。

**L2：rr2 用例 `192:9:3:1` 那段卸载的层 0 崩溃态（这一条腿跑的）。** 那条用例只取卸载落下 8 个写的那一个前缀；这里把同一个池的卸载整条录下来（36 次写、9 段，`closed_form_state_count` 8208，枚举实际 8223，撕裂态另算），`enumerate_layer0_versions` 每一段都展开，8223 个崩溃态一个不落。checker 计数（`l/l2-tally.log`）：

```
OPUS-L-TALLY rr2_on stream=unmount-of-192:9:3:1 states=8223 oracle_violations_with_empty_versions=0 checker_violated={} checker_first={}
```

（版本表给的是空表：这段卸载之前文件已删完，`oracle_violations_with_empty_versions` 只说明没有一个状态读出了文件，不判内容。）每个崩溃态上：重建那一遍（第 6 次读，逐条臂在钩子快照上核过，`k_mismatch` 0 次）盘 d 全读错、重读 `none` / `whole` / `other-all`，加不注错一条；挂上的再发 2 次改动，每一步同 B 段核 checker 与丢盘整读。rr2_on 五次分段跑的汇总合起来（每个格的 n 是五段之和）：

| 臂 | rr2_on 结局 | n | 坏 | rr2_off 结局 | n | 坏 |
|---|---|---|---|---|---|---|
| nofault | `Ok(first)` | 8223 | 0 | 同 rr2_on，不另跑 | — | — |
| 第一遍盘 0 全坏，重读不坏 | `Ok(reread)` | 8223 | 0 | `Err(PlacementRefusedBeforeAcquisitionMountAdmissionUndecided)` | 8223 | 0 |
| 第一遍盘 1 全坏，重读不坏 | `Ok(reread)` | 8223 | 0 | 同上 | 8223 | 0 |
| 第一遍盘 d 全坏，重读整环全坏 | `Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)` | 2 × 8223 | 0 | 不重读，同「重读不坏」那一行 | — | — |
| 第一遍盘 d 全坏，重读另一块盘全坏 | `Err(PlacementRefusedBeforeAcquisitionMountAdmissionUndecided)` | 2 × 8223 | 0 | 同上 | — | — |

「坏」= 任一步 checker 违例、F 及以上的根丢盘整读不同、或被拒之后盘面变了。57561 条 rr2_on 记录与 16446 条 rr2_off 记录都是 0；拒了的盘面逐字节不变（rr2_off 16446 次、rr2_on 32892 次 `image_unchanged=true`）。L2 上 rr2 把 16446 次「改之前取号之前被拒」变成挂上；多拒的只有「重读整环全坏」那一形，与 B 段同。

**K2-6 三样用不用同一遍读数（读码）。** `crates/singlefs-core/src/mount.rs:1980`「RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,」那一支把 `readings_of_the_reread` 交成 `ring_slot_readings`；之后根环表（`occupants`）、回收门槛（`oldest_valid_root_in_the_ring`）、影子账（`isolate_slots_referenced_only_by_abandoned_roots` 收的 `roots`）都从 `ring_slot_readings` 推出，没有第三遍读。重读那一遍仍有一块盘没读出的，取 m 取不到、走 `crates/singlefs-core/src/mount.rs:2078`「None => oldest_valid_root_in_the_ring,」，L2 那一行（重读另一块盘全坏）在窄盘上照旧取号之前被拒，与改之前第一遍那一形同结局。

**没打中的推翻条件。** 有一条「重读比第一遍坏」的形状，rr2_on 挂上之后两块盘的 checker 判违例、或 F 及以上的根丢盘读回不同；或 rr2_on 多拒的不止「重读整环都坏」那一形。

## 三、K1：Q3 拒因与它的复算

**新拒因只在哪一处交回（读码）。** core 里构造这个成员的只有一处：`crates/singlefs-core/src/mount.rs:1971`「newest_valid_root_txg_of_the_readings(&readings_of_the_reread)」取不到就 `crates/singlefs-core/src/mount.rs:1974`「MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,」。`newest_valid_root_txg_of_the_readings` 只数自证得过、不被最新那条根的实例表抛弃的根，读坏与读回全零都不算（同一个闭包里两类都映到 `None`）。判官的复算 `crates/singlefs-harness/src/mount_transparency.rs:2535` 起那个函数同样只数 `readable_roots`（自证得过的）、按 `choose_root` 那条根的表判抛弃：全零与读坏两边同口径，被抛弃按同一条根的表判（实现用 `instance_table_of_the_newest_root_read_at_most_twice` 读到的那张，没读故障时就是 `choose_root` 那条根指的那张）。读阶段择不到根报的是 `Recovery(NoValidRoot)`，在重建分配器之前，走不到 Q3；反过来 Q3 那一支只在读阶段择到了根之后，报不成 `Recovery`。没找到把一个报成另一个的路径。

**K1-1 合法镜像上复算为真（挂载透明假红）：没打中。** 两组没注读故障的合法镜像：

- C 段（上一条腿跑的，`c-rr2on.log`）：13 个年轻环起点加 5 个成熟窄盘起点，「OPUS-C1-SUM rr2_on images=18 criterion_true=0」；把这 18 张的根环每槽清 0 之后复算 18 张都为真，而真去可写挂载 18 张都是 `real_writable_mount=Err(Recovery)`——判据为真的盘面上真挂载走不到 Q3。
- L1（这一条腿跑的）：第一次可写挂载的 33 个层 0 崩溃态，三臂都是 `q3=false`（第一节 L1 汇总那几行）。

**读法 ③「实例表读不出时不按表滤」（读码，推的）。** 复算那一处见 `crates/singlefs-harness/src/mount_transparency.rs:2533`「/// 最新那条根的实例表读不出时没有「被抛弃」可判，不按表滤（读得出的根都算有效，与 F 生效值那一处的取法同向），择得到根就为假。」。实现这一侧，那张表读不出（重读一次仍读不出）在读根环之前就拒了：`crates/singlefs-core/src/mount.rs:1873`「instance_table_of_the_newest_root_read_at_most_twice(」那一调用带 `?`，报的是实例表那一条拒因，不是 Q3。所以一次 Q3 拒对应的崩溃镜像，若表读不出，被拒的那次挂载本该先报实例表那一拒；读法 ③ 只在「挂载时读得出表、复算时读不出」才起作用，而崩溃注入第二、三截被拒的那次挂载起在不带故障层的崩溃镜像上（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2584`「const READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE: ReadFaultsUnderTheRefusedMount =」），两边读的是同一份字节。没找到能让它判反的一格；没量。

**K1-2 第三截 Q3 那一条报红分支摆不摆得出：摆不出（推的，量过一形）。**

- 判据为真只在 `choose_root` 择不到根（或择不到系统配置）时：代码自己写了 `crates/singlefs-harness/src/mount_transparency.rs:2534`「/// 没有读故障时最新那条根按它自己指着的表恒不被抛弃（表里的行只记更早的实例，推的），所以择得到根的崩溃镜像上这条判据为假。」。择不到根时真挂载报 `Recovery`（C 段 18/18 量过），不是 Q3。所以「崩溃态没注读故障、拒因是 Q3、判据复算为真」这一组合，正确实现摆不出、错实现也摆不出（错实现能报 Q3 的盘面上读阶段择到了根，判据为假）。第三截新加的那条计数 `remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults` 恒为 0、报红分支是死的。
- 错拒 Q3 谁抓：副本上造了一个「重读那一遍恒报 Q3」的错实现（开关 `SINGLEFS_OPUS_REREAD_REFUSES`）。mkfs 起点上它拒了，判官给（`c-refuses.log` 原样）：「OPUS-C2 point=mkfs wrong_implementation_refusal=NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread second_cut_verdict=Illegal { reason: NoValidRootInTheRootRingOfTheRebuiltAllocator, conditions_not_met: [CriterionFalseWhenRecomputedOnTheCrashImage] } third_cut_reports_red=false」。第二截见 `Illegal` 就报红（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2650`「tally.writable_mount_refusals_on_the_d23_list_illegal += 1;」那一支），第三截只认「复算为真」那一条（同一文件 `:3806`「criterion_true_without_read_faults.then_some((reason, conditions_not_met))」），`CriterionFalseWhenRecomputedOnTheCrashImage` 落在 `:3801` 那一支、不报。二次崩溃镜像上的错拒 Q3 第三截不报红。
- 这是不是 10g 漏的一形：不是。第三截的取舍逐行照 G4 那一支接（第四节 K4-2），设计上就写了 U6 那几条只管第二截（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3636`「/// （r5 判决第二节 U6）条款只管第二截，这一截不拿它们定红绿：只差那几条的不报红、不计不合法，判官列出的没满足的条件只随 C670 那一条的红带出来。」）。差别只在 G4、C670 那两条的「复算为真」在合法历史里有意义（盘面本身坏了），Q3 这一条按定义摆不出：第三截对 Q3 实际上什么都不判。记成线索交主 agent，不算打中。
- 四句：不分辨臂（实现的三臂都不碰它）；判别子：第三截在判的那一刻看得到「判据为假」，是取舍把它扔了；归到判据：K1「崩溃注入第三截的计数与报红有没有漏一形」——漏的是「错拒 Q3、判据为假」那一形，但与 G4 同口径；改法（推的、零轮）：第三截对 Q3 照第二截那样见 `Illegal` 就报红，或在这一条旁边写明「这一支摆不出、只留作对称」。

**没打中的推翻条件。** 一张没注读故障、读阶段择得到根的合法崩溃镜像上 Q3 复算为真；或一条正确实现的挂载在不带故障层的盘上报 Q3。

## 四、K4：checker 档装置

**K4-2 第三截与第二截是不是同口径（读码）：没打中。** Q3 在第三截的计数与取舍（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3723` 起那个函数）与 G4 那一支逐行同形：合法的记一次、不合法的记一次判过，再在没满足的条件里找「没注读故障时复算为真」那一条，找到才报红。与第二截（见 `Illegal` 就报红）的差别是既有设计，见第三节 K1-2。

**K4-1 读序号后挪一遍根环之后（读码 + 一个量过的锚，checker 档没跑）。**

- 后挪只发生在环里只剩第 0 代根的那次可写挂载（第一节 H1）：多 24 次根环槽读、多调一次 `before_rereading`（A 段 mkfs 起点 3503 → 3527 次读、钩子 0 → 1；L1 那 12 个崩溃态同）。重读插在重建那一遍之后，这次挂载里重建那一遍之后的每一次读，整池序号后挪 24、每槽序号加 1。
- 随机历史有一半从这种盘面起：`crates/singlefs-harness/src/history.rs:705`「(HistoryStartingPoint::AfterMakeFilesystem, 1),」（与 `AfterFirstFile` 各 1 份），起点 `AfterMakeFilesystem` 第一步就是这次挂载。E158 的读故障点按「这一步里第几次这一种调用」摆（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:20340`「let arm_ordinal = calls_before.of(kind) + point.ordinal_within_the_step;」），崩溃注入、坏盘输入、E171 的随机历史同用 `the_nth_call_across_the_pool`。落在这一步重建那一遍之后的读故障点，rr2 之后打中的是另一次读；同一个种子的产物与 rr2 之前的不再逐行可比。
- 钉值：harness 档里按序号注读错的那几条（`writable_mount_of_a_formatted_pool.rs` 按系统配置槽数、`fault_injection_fast_tier.rs`）在三臂下都绿（第一节表、合入后验证）。checker 档（`crash_injection_campaign.rs:461`、`:616`，`bad_disk_input_campaign.rs:750`，E158 第 17621 行起那一组「最短复现：起点 AfterMakeFilesystem，3 步」）这条腿不能跑，钉值变没变要主 agent 跑 checker 档看；按 H1 的甲改法（按槽归盘），这次后挪本身就没了。
- checker 档没有一处用 `CallTheTestOnlyHookFirst`（`grep -rln` 在 `crates/singlefs-checker-tier/` 下零命中），多调一次钩子不影响它。

## 五、现场核对、取样与挂钟

**接手时核的（上一条腿留下的东西）。** 副本相对冻结副本只差 `mount.rs`（三个开关）与新加的 `opus_r2_rr2_attack.rs`（`diff -rq --exclude target --exclude .git` 只列这两处）；编出来的测试二进制都比 `mount.rs` 最后一次改动新。A 段两批 `.rc` 13 + 26 个全 0；B 段四批里第 4 批有 6 件撞了它自己套的 `timeout 232`，那 6 件的日志删掉、在第 5、6 批分 4 片重跑（第 5 批 rr2_on 那几片把 `SINGLEFS_OPUS_RR2_OFF` 设成了空串，开关按「设没设」判、等于跑了 rr2_off，第 6 批删掉重跑）；留下的 38 份 B 日志都以 `test result: ok` 收尾，`on-` 22 份行标签全是 `rr2_on`、`off-` 16 份全是 `rr2_off`；`agg-b.py` 重跑得到 865 条两臂齐全。A 段 39 份日志的臂标签与文件名对得上（slot 臂 13 份 `attribute_by_slot=true`）。模型目录里的 `mount-switches.patch` 是它早先只有两个开关的版本，不是副本今天的样子，今天的在 `mount-switches-full.patch`。

**取样与缩了什么（照定义第 3b 条）。**

- 盘：全部用内存稀疏盘；L1、L2 的起点池各只建一次，崩溃态从基线 + 持久集合现叠。
- 崩溃态没缩：L1 33 个、L2 8223 个，全部来自 `enumerate_layer0_versions` 那一种每段都展开的枚举（观察者那一趟用同一份文件的 `enumerate_layer0_selecting_versions_observing_each_state`、展开函数恒真，两趟状态数相等）。
- 缩的是 L2 的臂与后缀：B 段每个起点 98 条臂（读错、读回全 0 两种第一遍形状，逐槽的重读形状），L2 每个崩溃态 rr2_on 7 条、rr2_off 2 条（第一遍只取读错，重读只取 `none`、`whole`、`other-all`）；挂上之后的改动 16 次 → 2 次；历史只取 rr2 用例 `192:9:3:1` 那一段卸载，B 段另外四个起点的崩溃态没枚举。
- 估时：先跑 40 个状态（`l2-est2.log`，每状态逐个找 k）量得每状态约 3.5 秒·线程，全量两臂约 8223 × 3.5 × 2 / 24 ≈ 40 分钟，到线；把 k 固定为 6（逐条在钩子快照上核）、去掉 rr2_off 下与 `none` 相同的两条臂之后再跑 80 个状态（`l2-est3-*.log`），rr2_on 约 1.25 秒·线程。正式跑：计数一次 + rr2_off 一次并跑 177 秒；rr2_on 分五批（2741、1370、1371、1370、1371 个状态，28 条工作线程），挂钟 245、172、175、138、135 秒。
- 状态数总账：每次调用前 `closed_form_state_count` 算 18（L1）与 8208（L2），都在 10⁶ 之内；全部调用实际枚举 L1 3 × 33 × 2 = 198 个、L2 约 11 × 8223 + 第一次估时被我停掉前的一段，约 9.3 × 10⁴ 个，在 10⁷ 之内。
- 两次超了前台 240 秒、被工具转进后台：第一次估时（`l2-est.log`，用 `proc.py stop` 停了它那个二进制，停完列一遍进程确认不剩），与 rr2_on 第一批（跑完、退出码 0，结果照用）。之后的批都缩到 1371 个状态以内。

## 没打中的形状

- 只注一次瞬时读错（读报错或翻一位）在年轻环挂载的每一次读上：rr2 一次都不多拒（A 段 13 个起点、三臂约 27.6 万次挂载）。
- 「重读比第一遍坏」的十几种形状（重读整环坏、另一块盘全坏、另一块盘只剩最新那条、逐个有根的槽读错或读回全 0、与第一遍同样坏）：只有「重读整环坏」多拒，别的都挂上或与改之前同拒；挂上之后至多 16 步，checker 与 F 及以上的根丢盘整读 0 坏（B 段 865 条臂、5 个起点）。
- L2：rr2 用例那段卸载的 8223 个崩溃态 × 9 条臂，0 坏。
- Q3 复算在没注读故障的合法镜像上为真：18 张起点镜像 + 33 个层 0 崩溃态，0 张。
- 三样不同读：读码没找到第三遍读。
- 影子账、实例表重读与根环重读共用一个钩子：checker 档没有一处用它；harness 档三臂都绿。

## 这条腿自己的限度

- 只有一次抽样；「没打中」那几格照规则要再抽一次才记。
- A、B、C、D 四段是上一条腿跑的，我核了日志与源码对得上，没有重跑；它们的数与我这一段的数都在副本上。
- L2 只枚举了一段历史的崩溃态，后缀只有 2 步；B 段另外四个起点、216 那段续做挂载的崩溃态没枚举。
- H1 的甲改法只在两处（触发与取 m）换了归盘法，卸载那一格（`crates/singlefs-core/src/mount.rs:3009`「let device_of_txg = |txg: CheckpointTxg| {」那一处）没换；只有第 0 代根的环在卸载时摆不出（挂载自己就落了 txg 1 的根），是推的。
- K4 的钉值要跑 checker 档才知道，这条腿不能跑。
- 判官、核对器都是被判系统一侧的东西：B、L2 的「丢盘整读」是原型自己写的判法（`broken_roots`），没被独立路子核过。

## 要主 agent 跑的命令

- checker 档（提交时那一次即可）：看 `crash_injection_campaign.rs` 起点 `AfterMakeFilesystem` 那两条（第 461、616 行附近）、`bad_disk_input_campaign.rs` 第 750 行附近那一条、E158 第 17621 行起「最短复现：起点 AfterMakeFilesystem，3 步」那一组的钉值，在 rr2 合入前后变没变；变了的，看是不是落在只有第 0 代根那次挂载的重建那一遍之后（第四节 K4-1）。命令照 `.claude/skills/crash-test/commit-stages.md`，这条腿不给。

## 没做什么

- 不判正推那几格（K3、K5、K6），K1 只答了分给攻方的那几问。
- 没碰入库装置；副本上量的数不算入库装置上的数。
- 甲、乙两个改法没实现进 `crates/`，只有甲在副本上以开关的形式跑过。
- 草稿目录里的仓副本与编译目录交回之前删掉（见交回）。

- 删了：草稿目录里的仓副本与编译目录 `/tmp/claude-1000/m3-batch10-code-r2-opus/tree/`（3.3G，其中 `target/` 2.4G；上一条腿建的副本，这一条腿在它上面加了原型包又编过）。草稿目录剩 18M 日志与脚本，日志已打包进模型目录的 `logs.tar.gz`。

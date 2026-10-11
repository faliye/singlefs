# m3-batch58-code-r1 攻方腿（Opus）报告：删除线第 58 批代码轮第一轮（2026-10-10）

立场：云端攻方，攻正文第三节 K1、K2、K3，在冻结副本 `~/.local/share/singlefs/work/m3-batch58-code-r1/repo/` 的拷贝上造反例（开工时 `sha256sum -c research/prompts/m3-batch58-code-r1-snapshot/crates-sha256.txt` 4 行全 OK；下面引的代码文件在主工作区与冻结副本上逐字节相同，`cmp` 现查过，行号两边一样）。前一轮判决 `research/prompts/m3-batch57-code-r1-main-verification.md`。没读禁读清单里的三样。J1–J4 修的那几形不当打中报，正文第二节列的已知欠账不当打中报。副本上量出的数都是副本上的数，不算入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-batch58-code-r1-opus-model/`。在仓根、从空目录起（模型目录拷出仓用时设 `SINGLEFS_SCRIPTS=<仓根>/scripts`；`SINGLEFS_REPO` 缺省就是这一轮的冻结副本）：
```
OPUS_THREADS=10 bash research/prompts/m3-batch58-code-r1-opus-model/rerun.sh <空工作目录> all
```
`all` 依次跑 14 个阶段再 compare（`rerun.sh <工作目录> list` 列阶段）：base-build（只编不跑）、base-k1（K1 的 GPU 流 CPU 替身、checker 单测、坏镜像默认档与三条慢档、容器树长层）、base-k3（`corrupt_on_disk…` 41 条，含我接在末尾的打印原型）、k2-j4（J4 两种造法的层 0 表逐项比、建 GPU 判表不上卡）、7 个 `mut-*` 改坏（walk.rs 三种、recovery.rs 四种）、`mut-row3859`…`mut-row3861`（第 58 批变异表里 J1 那三行，照点名的测试看红没红）。只拷冻结副本的 `Cargo.toml`、`Cargo.lock`、`crates/`；原型 crate `probe-crate/` 拷成副本工作区成员 `crates/opus-probe-b58`（测试目标名不带 layer0，不开 gpu、verdict-store 特性）；`probes/corrupt_on_disk_k3_append.rs` 接到副本那份用例文件末尾。base、mut 两个副本各用自己的 `CARGO_TARGET_DIR`；同一时刻一条 cargo；每条先只编不跑（经 `capped.sh`、不经内存包装），再经 `capped.sh` 与 `run-with-memory-cap.sh` 跑（上限缺省 3G，k2-j4 那一段 10G：4G 时撞上限，见「这条腿自己的限度」）。前台有超时的环境按阶段名分几次起、同一个工作目录接着跑。挂钟（本机负载 20–60，别的会话在跑 checker 档）：交回前从空目录 `run-final/` 分 5 次跑完，合计约 8.6 分钟，compare 原样末行「compare：14 份，不一致 0 份」。

每个文件的 sha256（在 `research/prompts/` 下跑 `find m3-batch58-code-r1-opus-model -type f | sort | xargs sha256sum`）：
```
caa8294fa130624eb09fbdb7a1c63d2cc8a0f2cb7a28cab71c6071010b3f27be  m3-batch58-code-r1-opus-model/compare.py
56f6e6304d02d413bb7d5d463ac5cdc58551266dc7269b467fc385815f39b913  m3-batch58-code-r1-opus-model/expected/base-build.txt
66caca6113a36c926f7d8dea63e5c177d7432ca88d3ac618909fa892918f34e4  m3-batch58-code-r1-opus-model/expected/base-k1.txt
ee8cb2ee99a0a92661ae1d07f26b6b295e69dcc98a37d784f2f10e2eef249a38  m3-batch58-code-r1-opus-model/expected/base-k3.txt
5fd0db2fe4e56a84cac22bcd89036769f18db204c6e64ec5373d85cb6dee3dcd  m3-batch58-code-r1-opus-model/expected/k2-j4.txt
ee8cb2ee99a0a92661ae1d07f26b6b295e69dcc98a37d784f2f10e2eef249a38  m3-batch58-code-r1-opus-model/expected/mut-r-count-skips-unregistered-tag.txt
4b836c2dde4f90fa1184de0d33b51ec211f52e438f4df8fe6deaa8c6991f6c2e  m3-batch58-code-r1-opus-model/expected/mut-row3859.txt
1d7b9267858fff26aafdd9756e9fa0d1bbe10bf86dfe6843f588c17748e46ea4  m3-batch58-code-r1-opus-model/expected/mut-row3860.txt
fb6832bfc5b8c34e85e08889ab01c3ab77836acab2630836dc8c4018ad699e4a  m3-batch58-code-r1-opus-model/expected/mut-row3861.txt
f6994bbcaa9572a6041501184bda6a0941b02e411dfaab465e05ba804b46b85e  m3-batch58-code-r1-opus-model/expected/mut-r-rebuild-alloc-after-each-entry.txt
2a2c54c9338ea3ce502b9c77a026b905733ea6442e2ebc1fa069664dd5a6377c  m3-batch58-code-r1-opus-model/expected/mut-r-rebuild-mapping-first.txt
ee8cb2ee99a0a92661ae1d07f26b6b295e69dcc98a37d784f2f10e2eef249a38  m3-batch58-code-r1-opus-model/expected/mut-r-tag-judged.txt
bc2cce218ab24a09bcc9fdc904d8c1c32d5a0d44e322c03582cb7a473a1c183a  m3-batch58-code-r1-opus-model/expected/mut-w-first-only.txt
66caca6113a36c926f7d8dea63e5c177d7432ca88d3ac618909fa892918f34e4  m3-batch58-code-r1-opus-model/expected/mut-w-last-two.txt
9726f3d0cac754e3f9de06c571f33d30c522f6bfb3ce760a91a0c4332715348d  m3-batch58-code-r1-opus-model/expected/mut-w-second-to-last.txt
388658ae1ec7e7e2c95a4e59e634d9e4f198133d0fe6aa93c288aa26d2d14dc5  m3-batch58-code-r1-opus-model/mutants.py
02900ecfc8194479348f8bef447fd62a430e33a8686517756711a44c56e3933e  m3-batch58-code-r1-opus-model/probe-crate/Cargo.toml
aeb7d530de1b7bf4ccd105e9dd5c09f86999fdb0e88db85eae42b35e98ee6cf2  m3-batch58-code-r1-opus-model/probe-crate/tests/flows_b58/mod.rs
04598c71fcc29c32f803da7ff4e56351a65bc98e8cc468141f3ca0d77bbbc911  m3-batch58-code-r1-opus-model/probe-crate/tests/k1_gpu_flow_proxy.rs
000a772146b10ce56521bb37f2b7db799533d9b27567ae4fea026b95e8bf8d5c  m3-batch58-code-r1-opus-model/probe-crate/tests/k2_j4_state_sets.rs
1fbbb34dd6b89ea5816aa288d26443d0c49e680aaa28ad88fe1dbe172e469159  m3-batch58-code-r1-opus-model/probes/corrupt_on_disk_k3_append.rs
8d773a8c5a46a9eac464b0e75ad850d4294fe6c9b7ccb95cd26c75b62aa32c34  m3-batch58-code-r1-opus-model/rerun.sh
124b85cca5ba95fcdd1fd4054a53e1924bce34906ed32f7fdfc0d8cd96f80003  m3-batch58-code-r1-opus-model/sort_lines.py
```

## 各格判定一览

| 格 | 判定 | 依据（「量过」= 本腿副本上的原样输出，产物在模型目录 `expected/`；「推的」= 读码） |
|---|---|---|
| K1-1 新用例是不是只因点名的那一种写错而红 | 没打中（量过） | 断言是「违例清单恰好等于这一棵那一条」加细节片段；副本上 9 条（6 默认档 + 3 慢档）照绿（`base-k1.txt`）；只判叶序第一片红默认档 6 条、慢档 3 条（`mut-w-first-only.txt`） |
| K1-2 转一圈根环之后 [2,1,2,1] 那一版是不是真被判到 | 没打中（量过） | 只判倒数第二片（`w-second-to-last`）只红那 3 条慢档（`mut-w-second-to-last.txt`）：要是 [2,1,1] 那几版还在候选集里，这一改坏会照报容器 2、慢档照绿 |
| K1-3 GPU 那三条是不是真在比旋钮支 | **打中一处（分辨批次；GPU 那一半推的，CPU 替身量过）** | 三条流每个状态上 CPU 带旋钮 2 的新入口确实红（墓碑 153 态里 21、谱系 105 里 9、退役 189 里 9），但 GPU 扫描内核那一个共用函数改成「只判倒数第二片」时，三条流 447 个状态上 CPU 同形改坏的违例位与成立位摘要逐个与正确判法相同（`mut-w-second-to-last.txt` 与 `base-k1.txt` 的三行 digest），GPU 别的旋钮流都只两片：这一改坏在全部 GPU 对拍上照绿；替身会红的自证：只判叶序第一片时三个摘要都变、违例态数归 0（`mut-w-first-only.txt`） |
| K1-4 旋钮支只判最右之前的两片 | 打中（弱，量过） | `w-last-two` 在 checker 单测 32、坏镜像默认档 259、慢档 3、容器树 6、GPU 流替身上与基线输出逐字节相同（`mut-w-last-two.txt` 与 `base-k1.txt` sha256 相同）：三种排布里少一条的那片都在最右之前两片以内 |
| K2-1 新起点镜像下崩溃状态集合与原造法相同吗 | 没打中（量过） | 两种造法 25 = 25 个状态，展开段、写、每次写取几态、撕裂镜像逐项相同，原造法分界之前的段 0 个状态、没有跨分界的段（`k2-j4.txt`）；用例注释那句成立 |
| K2-2 会不会撞内核别的读宽上限 | **打中（不分辨造法：J4 两种改法都中；GPU 行为推的，记录数量过）** | 这条流第 816 个 unlink 前后那几版各有 1242 条分配记录（已释放 1202–1204 条），GPU 恢复内核读分配记录树的上限 1024、池级 checker 已释放记录表上限 1024，越过就置「判不了」；用例要求一个状态都不许判不了。分配记录数在 txg 294 就过了 1024（`k2-j4.txt` 的 growth 行） |
| K2-3 [815, 1] 改号那一形 GPU 与 CPU 判得一样吗 | 推的（同 K2-2） | GPU 在走到产品支三判之前就判不了；CPU 那一侧照判 |
| K3-1 三张新盘是不是只由点名的那一次序分得出 | 没打中（量过） | 只坏一处的四张盘两条路各报自己那一处，两处坏的三张盘两条路都报分配记录（`base-k3.txt`）；三行变异各红点名的那条（`mut-row3859..3861.txt`） |
| K3-2 J2 删 key 最小那一行会不会碰到水位行或别的判的前提 | 没打中（量过） | 删的是 `AccountingEntry { statistic: 1, … device: DeviceIdentity(0), generation: CheckpointTxg(4) … }`（已分配字节、盘 0），只坏这一处两条路都报记账条目数；上一轮两种读者改法下 41 条照绿、输出与基线逐字节相同（`mut-r-tag-judged.txt`、`mut-r-count-skips-unregistered-tag.txt`） |
| K3-3 可写挂载那一条钉的是重建里的哪一个次序 | 没打中一半、**打中一半（分辨批次，量过）** | 钉的是「重建里分配记录判在数映射条目之前」：重建先数映射条目再判分配记录（`r-rebuild-mapping-first`）只红它（`mut-r-rebuild-mapping-first.txt`）。没钉的：重建把分配记录判挪到条目宽与逐条判之后、数记账条目之前（`r-rebuild-alloc-after-each-entry`），41 条全绿；J1 ② 那张盘交可写挂载时正确次序报分配记录、这一改坏报记账条目的代（`mut-r-rebuild-alloc-after-each-entry.txt`），而那张盘只有只读恢复一条用例 |

## K1　旋钮多片坏镜像（甲 J3）

### 被判的判法与新用例（读码，行号按冻结副本）

- CPU 两个判法装了旋钮那一支各有一句「是不是最右那片」：`crates/singlefs-checker/src/walk.rs:12152`：「let is_the_rightmost_tombstone_container_under_the_knob =」，`crates/singlefs-checker/src/walk.rs:12053`：「let is_the_rightmost_under_the_knob =」。
- GPU 扫描内核三棵树共用一个函数，「最右之外每片 == 旋钮值」是 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3357`：「if (d_container_record_count(container_index) != knob_records) {」；墓碑树从 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3368`：「d_judge_containers_under_the_capacity_knob(D_TOMBSTONE_CONTAINER_CAPACITY_KNOB, INVARIANT_I_9_22);」进，谱系与退役从 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3389`：「d_judge_containers_under_the_capacity_knob(knob_records, INVARIANT_I_9_37);」进。所以 GPU 上一处改坏三棵树一起坏。
- harness 新用例的断言是违例清单恰好一条：`crates/singlefs-harness/tests/checker_known_bad_images.rs:16759`：「[tree.knob_capacity_invariant()],」；[2,1,2,1] 那一形的写法与转根环在 `crates/singlefs-harness/tests/checker_known_bad_images.rs:17081`：「records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],」、`crates/singlefs-harness/tests/checker_known_bad_images.rs:17084`：「root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,」。
- GPU 新的三条流都是 [2, 1, 1]：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13838`：「const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER: [u64; 4] = [2, 2, 1, 1];」。GPU 变异只登记了一行：`crates/mutations.tsv:3858`：「第58批 J3 GPU 判器：旋钮那一支只判叶序第一片」。
- 条款：「满了才关」（`.claude/kb/decisions/18-块里携带什么信息.md:578`）。

### 原型怎么造的（`probe-crate/tests/k1_gpu_flow_proxy.rs`、`flows_b58/mod.rs`）

把被判那三条 GPU 流的造法照抄（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 名字不带 layer0），盘换成内存稀疏盘外包录制器（`build_pool` 的同一组 mkfs 参数、取号、暖机、新池新建文件与分配器两步装载）。状态域取被判用例自己的展开：段长 ≤ 8 的段每个真子集、更长的段不展开（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:182`：「const DEFAULT_EXPAND_UP_TO: usize = 8;」）。每个状态经 `judge_layer0_state_range_against_the_version_table` 恢复、再交 CPU 池级 checker 带旋钮 2 的新入口，取全部 `IMPLEMENTED_INVARIANTS` 的违例位与成立位，按序号排好取摘要。正确的 walk.rs 与改坏的 walk.rs 各跑一次，摘要相同就是那种改坏在这三条流的每个状态上与正确判法同判；GPU 比的正是这几位，扫描内核里同形的改坏也就在这三条对拍上与 CPU 逐位相同（这一步是推的：GPU 我不跑）。

状态数（跑前用 `layer0_plan_state_count` 算、`closed_form_state_count` 报全量）原样：
```
OPUS_K1P plan flow=lineage-2-1-1 domain=gpu-test writes=166 segments=31 longest_segment=24 closed_form_full=22020150 states=105
OPUS_K1P plan flow=retirement-2-1-1 domain=gpu-test writes=387 segments=59 longest_segment=30 closed_form_full=1394606176 states=189
OPUS_K1P plan flow=tombstone-2-1-1 domain=gpu-test writes=298 segments=47 longest_segment=30 closed_form_full=1312817230 states=153
```
全量（每段都展开）三条流各 2.2 × 10⁷、1.39 × 10⁹、1.31 × 10⁹，都超过每次 10⁶ 的线，没跑（见「这条腿自己的限度」）。

### K1-3 GPU 只判倒数第二片照绿：打中

基线与改坏（`w-second-to-last`：两个 CPU 判法的「最右」那句都换成「不是倒数第二片」，即 `crates/mutations.tsv:3854`：「第58批 J3 I-9.22：旋钮支只判倒数第二片」那一形，墓碑与谱系 / 退役两处一起）的替身输出逐行相同，原样（`base-k1.txt` 与 `mut-w-second-to-last.txt`）：
```
OPUS_K1P judged flow=lineage-2-1-1 domain=gpu-test states=105 I-9.37_violated_states=9 I-9.37_held_states=36 states_with_any_violation=9 digest=26103a906f2cda2c
OPUS_K1P judged flow=retirement-2-1-1 domain=gpu-test states=189 I-9.37_violated_states=9 I-9.37_held_states=108 states_with_any_violation=9 digest=55bde54b4a31d7d8
OPUS_K1P judged flow=tombstone-2-1-1 domain=gpu-test states=153 I-9.22_violated_states=21 I-9.22_held_states=72 states_with_any_violation=21 digest=1d0b97c82ec31027
```
替身会红的自证（只判叶序第一片 `w-first-only`，`mut-w-first-only.txt`）：
```
OPUS_K1P judged flow=lineage-2-1-1 domain=gpu-test states=105 I-9.37_violated_states=0 I-9.37_held_states=45 states_with_any_violation=0 digest=802408710675015a
OPUS_K1P judged flow=retirement-2-1-1 domain=gpu-test states=189 I-9.37_violated_states=0 I-9.37_held_states=117 states_with_any_violation=0 digest=c52f7472757b53b9
OPUS_K1P judged flow=tombstone-2-1-1 domain=gpu-test states=153 I-9.22_violated_states=0 I-9.22_held_states=93 states_with_any_violation=0 digest=5a26345d7699aeaa
```
GPU 别的带旋钮的流（第 56 批 G3、G6 与第 57 批 H1 那几条）都只两片或三片、少的那片只可能在倒数第二位（读码：`record_rollbacks_under_the_lineage_knob` 回退至多 3 次，其余是两片）。所以 GPU 内核把第 3356–3357 行改成只判 `container_index + 2u == d_container_count` 时，全部 GPU 对拍照绿；CPU 这边它只被三条慢档抓到（同一个改坏在 harness 上原样：默认档「test result: ok. 259 passed; 0 failed; 5 ignored」，慢档「test result: FAILED. 0 passed; 3 failed」）。

四句：
1. 分不分辨：分辨。正确的 GPU 判法与「只判倒数第二片」在这三条流上判不出差别，判决 J3 的验收（「只判倒数第二片」变异要红）在 GPU 那一半够不着。
2. 判别它的东西当时看得到吗：GPU 判器看得到整张容器表，看不到的是用例：三条流每个状态的每一版容器表只会是 [2]、[2, 1]、[2, 1, 1] 之类，少一条的那片只会落在倒数第二位，判别子在这组状态里从不出现。
3. 满足的分句：K1 格「GPU 那三条是不是真在比旋钮支」——它们在比旋钮支（违例态 21 / 9 / 9），但只比得出「只判第一片」那一种，分不开「只判倒数第二片」。判决 J3 原文要的是「GPU 对拍每棵树一条」，字面满足；没罩到的是正推「只判倒数第二片」那一形的 GPU 版本。
4. 改法在这格上中不中：J3 的写法（每棵树一条 [2, 1, 1]）在这格上照中。能修的是加一条 GPU 流 [2, 1, 2, 1] 并转过根环（与 harness 慢档同形；转环是因为没转之前候选集里 [2, 1, 1] 那一版也红）。只在我的模型上推过、被攻过零轮；GPU 状态数会随多出来的 30 次发布涨，没量。

### K1-4 只判最右之前的两片照绿：打中（弱）

`w-last-two`（不是最右、且不在倒数第三或倒数第二位的片不判）：`mut-w-last-two.txt` 与 `base-k1.txt` 两份 sha256 都是 `66caca6113a3…`，逐字节相同，含三条慢档「test result: ok. 3 passed; 0 failed」。三种排布 [2,1,1]、[2,2,1,1]、[2,1,2,1] 里少一条的那片离最右都不超过两片。四句：分辨（正确判法与它在全部用例上判不出差别）；判别子没出现在任何一张盘上；满足 K1 格「改坏照绿」那一分句；J3 的三种排布在这格上照中，加一张 [2,1,2,2,1]（少的那片之后还有两片满的）能修，推的、零轮。弱在：任何有限组排布都留着「只判最右之前 k 片」对足够大的 k 照绿，这一族只能缩小窗口、关不死；交主 agent 判要不要收。

### K1-1、K1-2：没打中

- 基线上 9 条新用例照绿（`base-k1.txt` 原样「test result: ok. 259 passed; 0 failed; 5 ignored」「test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 261 filtered out」）。断言要违例清单恰好是这一棵那一条，另两棵、别的不变量红一条都过不了；正确判法下没有别的理由红。
- 只判叶序第一片：默认档 6 条（三棵树各 [2,1,1]、[2,2,1,1]）红、慢档 3 条红（`mut-w-first-only.txt`）——用例不是在错的理由上红，红在第 16757 行那一句断言上（清单变空）。
- 转根环：只判倒数第二片只红 3 条慢档（K1-3 那一段原样），说明慢档判的候选集里已经没有 [2, 1, 1] 那一版、[2, 1, 2, 1] 那一版判到了。

## K2　J4 改造法

### 新旧两种造法的层 0 表逐项比（`probe-crate/tests/k2_j4_state_sets.rs`，量过）

造法照抄被判那一份（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13758`：「let operations_before_the_last_unlink = pool.stream.operation_count();」、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13781`：「base.apply(&operations[pool.mkfs_operation_count..operations_before_the_last_unlink]);」），原造法照第 57 批那一份（mkfs 之后整条流进写表，第一次写的下标早于第 816 个 unlink 的段不展开）。不改号：改号只改第 816 个 unlink 里几次写的字节，不改写的个数、落点与切段。`k2-j4.txt` 原样：
```
OPUS_K2 pool operations=55717 mkfs_operations=25 operations_before_the_last_unlink=55649 versions=1634 last_txg=1636 
OPUS_K2 old writes=45878 segments=6543 first_write_of_the_last_unlink=45822 segments_straddling_the_boundary=[]
OPUS_K2 old-tables states=25 enumerated_writes=49152 torn_images=3274 
OPUS_K2 new writes=56 segments=8 segment_lengths=[26, 2, 1, 2, 20, 2, 1, 2] closed_form_full=68157453 states=25 enumerated_writes=60 torn_images=4
OPUS_K2 compare same_segments=true same_writes=true tearable_differences=[] tearable_new=4 in_place_writes_new=10 same_state_ranges=true old_states_before_the_boundary=0 same_torn_images=true state_count_old=25 state_count_new=25
OPUS_K2 judge-tables-old Err("EnumeratedWritesPastTheMaskWidth { enumerated_writes: 49152 }")
OPUS_K2 judge-tables-new states=25 locations=4704 total_words=15190760 version_bytes_words=15076608 table_mebibytes=58
```
原造法的枚举写表 49152 次写与第 57 批判决第一节记的拒绝原样相同，可见原型的流与被判那条流是同一条。原造法里不展开的段状态数是 0（`crates/singlefs-checker-tier/src/crash.rs:1869`：「Self::NotExpanded => 0,」），分界之前 0 个状态、没有跨分界的段，之后 8 段与新造法逐段同写、同取几态、同撕裂镜像，所以用例注释（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13744`：「之前的段在原来那一造法里本就整段持久、不展开，崩溃状态与这里相同」）成立。K2-1 没打中。新表 58 MiB，远低于判器建判器时那句绑定上限断言要的量级（推的：本机卡的上限按 `adapter.limits()` 取）。

顺带的观测（不是打中，两种造法一样）：第 816 个 unlink 两次发布的单元写段各 26、20 次写，段长超过展开上限 8，整段不展开；这一次 unlink 的层 0 全量是 68 157 453 个状态，用例判的是其中 25 个。

### K2-2 撞 GPU 内核别的读宽上限：打中

同一份输出里每一版的分配记录数（原样）：
```
OPUS_K2 growth first_txg_with_more_than_1024_allocation_records=Some(294)
OPUS_K2 growth unlinked=100 txg=204 allocation_records=950 released=910
OPUS_K2 growth unlinked=800 txg=1604 allocation_records=1242 released=1204
OPUS_K2 version-before-the-last-unlink txg=1634 allocation_records=1242 released=1204 mapping_keys=16 accounting_entries=17
OPUS_K2 last-version allocation_records=1242 released=1202 mapping_keys=17 accounting_entries=17 inode_leaf_containers=1 inode_records=1 tombstone_containers=[815, 1]
```
GPU 两处上限（读码）：
- 恢复内核读分配记录树：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:151`：「const MAX_ALLOCATION_RECORDS: u32 = 1024u;」，越过时 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2304`：「if (allocation_count >= MAX_ALLOCATION_RECORDS) {」、`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2305`：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;」。这段在 `read_allocation_tree` 里，由走到文件那一步调（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3300`：「let read = read_allocation_tree(pointer, tree_table.base, o + 14u, tree, root, fsid_low);」）。
- 池级 checker 已释放记录表：`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:467`：「const C_MAX_RELEASED: u32 = 1024u;」，`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:710`：「if (c_released_count >= C_MAX_RELEASED) {」、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:711`：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (29u << 8u);」。
- 判不了进对拍字：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:378`：「gpu_could_not_judge: words[0] & undecidable_bit != 0」；用例要求「一个状态都不许判不了」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5`）。

这条流 25 个状态都落在 txg 1634–1636 附近那几版（起点镜像里已经是 txg 1634），每一版 1242 条分配记录、1202–1204 条已释放，两处上限都过。推出来的结局：GPU 上 25 个状态全是判不了，对拍字与 CPU 逐个不同，用例红；走不到产品支三判，第 57 批那 4 行 GPU 变异仍「基线不绿」证不了。这一段 GPU 行为是推的（GPU 我不跑），记录数是量过的（副本、内存稀疏盘、照抄的造法）。推翻它的观测：主 agent 的带 gpu 那一趟里这条用例真跑了（不是「本次未跑」）、0 个不一致——那就是我读错了 `read_allocation_tree` 的计数口径或这条流的记录数。

四句：
1. 分不分辨：J4 的两种改法都中，不分辨改法。判决 J4 原文给的两条路——「把之前的发布当起点镜像」（被判那一份）与「换一种不用录 1630 次发布的造法」——前一条起点镜像的内容与原造法相同，记录数一样；后一条只要还是逐个 unlink 空 inode 填满一片 815 条的容器，分配记录在 txg 294（约第 145 个 unlink）就过了 1024（growth 行），也中。
2. 判别它的东西当时看得到吗：看得到；内核自己数到上限就置判不了，是有意的拒判。
3. 满足的分句：K2 格「会不会撞内核别的读宽上限」。正文第二节的已知欠账只有「GPU 谱系记录表上限 256 条」，分配记录与已释放记录这两张表不在里面。
4. 改法在这格上中不中：被判那一种照中。不中的只有改 GPU 内核两处上限（`crates/*/src` 的改动，不是只改测试），或者找一种发布次数少到 txg 294 以内的造法；后者我没找到，推的、零轮。另一条出路是这一形只在 CPU 上钉（第 57 批 CPU 那几行变异已经全红），GPU 那 4 行改登「判不了」；交主 agent 判。

### K2-3：推的（同 K2-2）

GPU 在恢复那一遍就判不了，池级 checker 也判不了，走不到 [815, 1] 改号那一形的「相邻两片加一」那一判；CPU 那一侧照判。没有能量的 GPU 输出。

## K3　冷走读次序补钉与换造法（乙 J1、J2）

### 重建与冷走读的次序（读码）

重建（可写挂载）里分配记录判在读记账树之前：`crates/singlefs-core/src/recovery.rs:2894`：「allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;」，之后条目宽 `crates/singlefs-core/src/recovery.rs:2918`：「every_accounting_entry_is_as_wide_as_its_field_table(」、逐条判、数记账条目 `crates/singlefs-core/src/recovery.rs:2928`：「accounting_entry_count_is_three_plus_six_per_device(」、水位行在不在、数映射条目 `crates/singlefs-core/src/recovery.rs:2944`：「mapping_entry_count_is_one_per_mapped_unit(」。冷走读同序：`crates/singlefs-core/src/recovery.rs:5395`：「allocation_record_generations_and_spans_are_judged(allocation_records, root)?;」。

### 原型（`probes/corrupt_on_disk_k3_append.rs`，接在副本那份用例文件末尾，只打印）

台阶行那个池（txg 4）上造 7 张盘，每张交只读恢复与可写挂载（可写挂载在拷贝上起）各报什么。基线上原样（`base-k3.txt`，摘 J2 删的那一行与只坏一处的四张）：
```
OPUS_K3 j2-dropped-row AccountingEntry { statistic: 1, tree: TreeIdentifier(0), device: DeviceIdentity(0), generation: CheckpointTxg(4), value: 458752, sequence: 1 }
OPUS_K3 image=only-allocation-after-root writable=refused Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
OPUS_K3 image=only-largest-mapping-entry-dropped writable=refused Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数" })
OPUS_K3 image=only-smallest-accounting-row-dropped writable=refused Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" })
OPUS_K3 image=only-this-versions-watermark-row-after-root writable=refused Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" })
```
（只读恢复那四行报的成员与可写挂载相同，见 `base-k3.txt`；两处坏的三张盘两条路都报「分配记录跨度为 0，或分配代 / 释放代晚于根」。）

### K3-1、K3-2：没打中

- 三张新盘只坏一处时各报自己那一处，两条路一致；两处一起坏时正确次序都报分配记录。三行登记的变异各红点名的那条（`mut-row3859.txt`、`mut-row3860.txt` 各「test result: FAILED. 40 passed; 1 failed」，`mut-row3861.txt`「test result: FAILED. 39 passed; 2 failed」）。没有别的判先拒。
- J2 删的是 key 最小那一行：已分配字节、盘 0、代 4（D5 的行表里那一行是「已分配字节（1）」，`.claude/kb/decisions/05-快照-空间记账机制.md:208`）。不是 inode 号水位，台阶行与这一版的水位行都在，水位行那一判的前提不动。上一轮两种读者改法（逐条判拒标签 > 12、数条目不算标签 > 12 的行）下 41 条照绿，输出与基线逐字节相同（`base-k3.txt`、`mut-r-tag-judged.txt`、`mut-r-count-skips-unregistered-tag.txt` 三份 sha256 都是 `ee8cb2ee99a0…`）：换造法之后用例钉的是次序、不是标签。

### K3-3 可写挂载钉的是哪一个次序：一半钉住、一半没钉（打中）

钉住的：J1 ① 的可写挂载那条钉「重建里分配记录判在数映射条目之前」。改坏 `r-rebuild-mapping-first`（重建读完记账树就先数映射条目、再判分配记录，之后才是条目宽、逐条判、数记账条目）只红它，第 57 批可写挂载那条照绿（`mut-r-rebuild-mapping-first.txt` 原样：失败名「allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_writable_mount」，「test result: FAILED. 40 passed; 1 failed」）。顺带：登记给它的那一行（`crates/mutations.tsv:3861`：「第58批 J1 重建分配记录代与跨度挪到数映射条目之后」）第 57 批可写挂载那条也红（39 passed; 2 failed），这一行证不出新用例自己多钉了什么；能证的是 `r-rebuild-mapping-first` 这一形。

没钉的：重建里把分配记录判挪到条目宽与逐条判之后、数记账条目之前（`r-rebuild-alloc-after-each-entry`，冷走读不动）。`corrupt_on_disk…` 41 条全绿（「test result: ok. 41 passed; 0 failed」）。J1 ② 那张盘分得出它，原样（`mut-r-rebuild-alloc-after-each-entry.txt`）：
```
OPUS_K3 image=j1b-watermark-row-and-allocation recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" } }
OPUS_K3 image=j1b-watermark-row-and-allocation writable=refused Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" })
```
正确次序下同一张盘两条路都报分配记录（基线）。而这张盘只有只读恢复一条用例（`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:2118`：「fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_recovery(」），它的注释还写着两条路同序（`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:2115`：「（冷走读那一段的次序注释与重建相同：分配记录、条目宽、逐条判、数）。」）。

四句：
1. 分不分辨：分辨。正确的重建与这一改坏在全部 41 条上判不出差别；J1 ② 那张盘交可写挂载就分得出（量过）。
2. 判别它的东西当时看得到吗：看得到，两个坏处都在重建读得到的那一版里，重建先碰到哪一个就报哪一个。
3. 满足的分句：K3 格「可写挂载那一条钉的是重建里的哪一个次序」，以及派发点名的「可写挂载重建里的次序」改坏照绿。判决 J1 ② 原文只要「冷走读报分配记录」，所以这是修法没罩到的地方，不是 J1 的验收没过。和已知欠账「分配记录判在条目宽之前」还没钉的关系：这一改坏同时越过条目宽与逐条判两判，冷走读那边逐条判这一半已由 J1 ② 钉住，重建这边两半都没钉；只挪过条目宽、不越过逐条判的那一形才是已知欠账本身，我没把它当打中报。
4. 改法在这格上中不中：J1 的写法照中。加一条 J1 ② 盘的可写挂载用例（与 J1 ① 那一对同形）就不中：原型上正确次序报分配记录、改坏报记账条目的代，量过；只在我的模型上量过、被攻过零轮。

## 几个改法各修哪一格

| 改法（都只在我的模型上量过或推过、被攻过零轮） | K1-3 GPU 只判倒数第二片 | K1-4 只判最右之前两片 | K2-2 GPU 读宽上限 | K3-3 重建分配记录挪到逐条判之后 |
|---|---|---|---|---|
| 加 GPU 流 [2,1,2,1] + 转根环（三棵树） | 推的：修（CPU 慢档同形已红，`mut-w-second-to-last.txt`） | 推的：不修 | 推的：不修 | 无关 |
| 加 harness 盘 [2,1,2,2,1] | 推的：不修（CPU 侧） | 推的：修 | 无关 | 无关 |
| 抬 GPU 内核 `MAX_ALLOCATION_RECORDS`、`C_MAX_RELEASED`（改 `crates/*/src`） | 无关 | 无关 | 推的：修这两处；后面还有没有别的上限没查到底 | 无关 |
| J4 换一种发布更少的造法 | 无关 | 无关 | 推的：只有发布数压到 txg 294 以内才修（量过的是逐个 unlink 时 txg 294 过 1024） | 无关 |
| 加 J1 ② 盘的可写挂载用例 | 无关 | 无关 | 无关 | 量过：修（原型上正确次序报分配记录、改坏报记账条目的代，`base-k3.txt` 与 `mut-r-rebuild-alloc-after-each-entry.txt` 的 j1b 那两行） |

## 没打中的形状

- K1：新用例在正确判法下被别的不变量拒——试了三棵树 × 三种排布 9 张盘，断言要违例清单恰好一条，全绿；只判叶序第一片下红在同一句断言（清单空）。转根环后 [2,1,1] 那几版还在候选集里——只判倒数第二片红了慢档 3 条，排除。
- K1：GPU 流的状态取样只取了用例自己的展开（段长 ≤ 8），三条流共 447 个状态；全量每条 2.2 × 10⁷ 起，没跑。
- K2：新起点镜像漏掉原造法的状态——逐项比了状态数、段、写、取几态（原地覆写 4 次三态两边相同）、撕裂镜像、跨分界的段（没有），都相同。没打中。新表撞掩码宽或绑定上限——新表 60 次枚举写、58 MiB，不撞。
- K3：J2 那一行碰到水位行或别的判的前提——删的是已分配字节那一行，只坏这一处两条路都报记账条目数；上一轮两种读者改法下照绿。J1 ① 的映射条目删除碰到别的判——只坏这一处两条路都报映射条目数。J1 ② 改水位行的代碰到「水位行在不在」那一判——那一判在数条目之后，逐条判先红，两条路都报记账条目的代。都没打中。
- 冷走读里别的次序改坏：J1 三行（3859、3860、3861）都红；我没再造冷走读的新改坏（「分配记录判在条目宽之前」是已知欠账）。

## 这条腿自己的限度

- GPU 一条没跑。K1-3 的 GPU 结论靠 CPU 替身：替身判的是同一组状态（照抄的造法、同一个展开上限），比的是 GPU 对拍里池级 checker 那几位；替身会红已自证。它推不到 GPU 内核在这些状态上走不走得到旋钮那一支（例如 GPU 在别处先判不了）。K2-2 全靠读码加 CPU 侧数的记录数。
- 原型的池是内存稀疏盘上照 `build_pool` 搭的，不是被判用例的文件镜像；流相同的证据是原造法的枚举写表 49152 次与第 57 批判决记的拒绝原样一致、[2,1,1] 三条流的容器记录数断言过。
- k2-j4 那一段第一次按 4G 起时撞内存上限（包装退出码 250，`run-with-memory-cap: 撞了内存上限 4G（scope singlefs-memory-cap-259695-2996729574 的 Result=oom-kill，命令退出码 137）`），那一次的输出不算；改 10G 重跑出上面的数。
- 本机同时有别的会话在跑 checker 档（`ps` 看到主工作区的 `cargo test --release -p singlefs-checker-tier …` 几条与正推腿的 cargo），内存包装排队每次 0–75 秒；有一次我起的 10G 那一条排了 4 分钟没起跑、被工具挪到后台，我停了它自己的排队进程（`proc.py stop 246590`，一个），之后改成先只编不跑、再按小上限跑。
- 跑过的状态量：K1 替身每次 447 个状态，开发与交回前复跑共 9 次，合计约 4 × 10³；K2 只建表不判（25 个）。没有一次超过 10⁶，合计远低于 10⁷。
- 单次取样：「没打中」的几格只抽了这一次。

## 要主 agent 跑的命令

不要求我这条腿的结论成立才跑；是核 K1-3 与 K2-2 唯一能落到 GPU 上的两步（checker 档、带 gpu，重型，归主 agent）：
1. 冻结副本或主工作区上单跑 J4 那一条 GPU 对拍，看是不是 25 个状态全「判不了」：`SINGLEFS_HEAVY_TESTS=user-request bash scripts/hosts/run-with-memory-cap.sh 12G cargo test --offline --release -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number`（工作目录仓根；线程照 `capped.sh`）。判结果看失败信息里的不一致数与「判不了」那一列；用例绿而 0 个不一致就推翻 K2-2。
2. K1-3：在副本上把 `crash_judge_checker_scan.wgsl` 第 3357 行改成 `if (container_index + 2u == d_container_count && d_container_record_count(container_index) != knob_records) {`，跑同一个测试目标的 `the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs` 等三条：三条都绿就坐实 K1-3。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑任何门禁阶段、checker 档测试、GPU；没跑崩溃点与变异两条轻型（不需要）。
- 没造冷走读「分配记录判在条目宽之前」那一形（已知欠账）；没量 K1、K2 改法加进去之后 GPU 状态数与挂钟。
- 正推腿的报告、本轮判决没读（禁读）。

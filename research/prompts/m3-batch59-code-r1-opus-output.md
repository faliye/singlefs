# m3-batch59-code-r1 攻方腿（Opus）报告：删除线第 59 批代码轮第一轮（2026-10-11）

立场：云端攻方，攻正文第三节 M1、M2、M3，在冻结副本 `~/.local/share/singlefs/work/m3-batch59-code-r1/repo/` 的拷贝上造反例。开工时 `sha256sum -c research/prompts/m3-batch59-code-r1-snapshot/crates-sha256.txt` 在冻结副本上 5 行全 OK；下面引的代码文件在主工作区与冻结副本上逐字节相同（13 份逐个 `cmp` 过），行号两边一样。没读禁读清单里的三样（另一条腿的报告与模型、本轮判决）。L1–L5 修的那几形、正文第二节列的已知欠账不当打中报。副本上量出的数都是副本上的数，不算入库装置上的数。GPU 一条没跑：GPU 那一半靠 CPU 替身加读码。

## 复跑

模型目录 `research/prompts/m3-batch59-code-r1-opus-model/`。从空工作目录起（模型目录拷出仓用时设 `SINGLEFS_SCRIPTS=<仓根>/scripts`；`SINGLEFS_REPO` 缺省就是这一轮的冻结副本）：
```
SINGLEFS_SCRIPTS=<仓根>/scripts OPUS_THREADS=10 OPUS_MEMORY_CAP=3G OPUS_TEST_TMPDIR=/dev/shm/<目录> \
  bash research/prompts/m3-batch59-code-r1-opus-model/rerun.sh <空工作目录> all
```
`rerun.sh <工作目录> list` 列出 46 个阶段，`all` 依次跑完再 compare；前台有时限的环境按阶段名分几次起、同一个工作目录接着跑（每个阶段被打断后重起照样接着编）。阶段：
- `base-build`（只编不跑）；`base-m1-{j3,l1,l2}`：GPU 旋钮流的 CPU 替身，各三条流（j3 = 第 58 批 J3 的 [2,1,1]，l1 = 第 59 批 L1 的 [2,1,2,1] 转根环，l2 = 我提的 [2,1,2,2,1] 转根环）；`base-knob-harness-{a,b}`：checker 单测、旋钮坏镜像里调到带旋钮入口的那几条、容器树长层；`base-m3`：`corrupt_on_disk…` 与 `core_review…` 两份（含我接进去的两条原型）再单跑打印 L4 几种造法结局的那一条；`base-legit-{a,b,unlink}`：合法历史的 8 份 harness 用例；`debug-fast-tier`：debug（快档）整份跑 `checker_known_bad_images`。
- walk.rs 的 5 种改坏（`w-second-to-last`、`w-first-only`、`w-last-two`、`w-last-three`、`w-last-four`），每种 `-m1-{j3,l1,l2}` 与 `-harness-{a,b}` 五段；recovery.rs 的 7 种（`row3868`、`row3861`、`row3668` 取副本 `crates/mutations.tsv` 那一行；`r-tag-judged`、`r-count-skips-unregistered-tag` 是第 57 批那两种读者改法；`r-each-entry-judges-i313` 是读者逐条判 I-3.13；三者合起来一种），I-3.13 那一种另跑 `-legit-{a,b,unlink}`。
- 只拷冻结副本的 `Cargo.toml`、`Cargo.lock`、`crates/`；原型 crate `probe-crate/` 拷成副本工作区成员 `crates/opus-probe-b59`（测试目标名不带 layer0，不开 gpu、verdict-store 特性）；`probes/core_review_l4_append.rs` 接到副本那份用例文件末尾。base、mut、debug 三个副本各用自己的 `CARGO_TARGET_DIR`；同一时刻一条 cargo；每条先只编不跑（经 `capped.sh`），再经 `capped.sh` 与 `run-with-memory-cap.sh` 跑（上限 3G）。`extract.py` 只留结果行、失败用例的断言消息与 OPUS 行，次序与线程先后无关；挂钟写 `.wall`，不进 compare。
- 测试镜像落 `OPUS_TEST_TMPDIR`（harness 的 `build_pool` 照 `std::env::temp_dir()` 落文件）。本机盘的 IO 压力 86% 时（`/proc/pressure/io` 原样「some avg10=86.23 avg60=86.09 avg300=74.84」），落在草稿目录里的旋钮坏镜像 12 条 200 秒跑不完；换成 `/dev/shm/m3-batch59-code-r1-opus` 后 5 秒。这个目录在草稿目录之外，交回前删了。

每个文件的 sha256（在 `research/prompts/` 下跑 `find m3-batch59-code-r1-opus-model -type f | sort | xargs sha256sum`）：
```
8a6ffad055562474d4a357f843941097d382c197e1e0598238e445865270a507  m3-batch59-code-r1-opus-model/by_root_diff.py
caa8294fa130624eb09fbdb7a1c63d2cc8a0f2cb7a28cab71c6071010b3f27be  m3-batch59-code-r1-opus-model/compare.py
abbc1a380f4dc7555d3abafdad082182c71a01f1fe2bfe2742582d01ebcc2234  m3-batch59-code-r1-opus-model/expected/base-build.txt
2f20e948b9f0133eda12bbb0597de3d36fb7d9829b433f8ecb6d7977e84d5d71  m3-batch59-code-r1-opus-model/expected/base-knob-harness-a.txt
d9c18cf0843dfb6139beba7a8ecba6ae928495c7cfc16137d7e47394248976ab  m3-batch59-code-r1-opus-model/expected/base-knob-harness-b.txt
b24f3c5a82ecf26155be05260039e2c93d09e0c73667c9ced918e85da76c394e  m3-batch59-code-r1-opus-model/expected/base-legit-a.txt
e599436c560963b1c30aa28751a7e019fc218131d40997a6412f8400ff3009b0  m3-batch59-code-r1-opus-model/expected/base-legit-b.txt
7771b90fc0bb6e7077de2c9f36f1a32528e5b0d50526eae26687f240690810ea  m3-batch59-code-r1-opus-model/expected/base-legit-unlink.txt
42ce33853af1e3d88246ea8a807226e3d2beaa996b2e2fb3f55db24f65bac6cd  m3-batch59-code-r1-opus-model/expected/base-m1-j3.txt
ba6a4474777f9fb67bdee65d9d3cdb4fcb72c85e501087cf31ccb25b7a9d3cb1  m3-batch59-code-r1-opus-model/expected/base-m1-l1.txt
30f7a7a71bae762845fe3cb88196a08cf3083a644163dffb8c1e13a0a23d739d  m3-batch59-code-r1-opus-model/expected/base-m1-l2.txt
fced8a6f7702e5415a478ed4ef35e7f5cd8db7b6385b03561a4d0071e6776a6b  m3-batch59-code-r1-opus-model/expected/base-m3.txt
a78968566e078c9b0ac065c89544472bcc063484b482a7bd955a0a7f7e34fd9a  m3-batch59-code-r1-opus-model/expected/debug-fast-tier.txt
53128cb7bba924a38f818708bcef67159cc952debf5ce37f57ed257f64ca1b9c  m3-batch59-code-r1-opus-model/expected/mut-r-count-skips-unregistered-tag.txt
b24f3c5a82ecf26155be05260039e2c93d09e0c73667c9ced918e85da76c394e  m3-batch59-code-r1-opus-model/expected/mut-r-each-entry-judges-i313-legit-a.txt
e599436c560963b1c30aa28751a7e019fc218131d40997a6412f8400ff3009b0  m3-batch59-code-r1-opus-model/expected/mut-r-each-entry-judges-i313-legit-b.txt
7771b90fc0bb6e7077de2c9f36f1a32528e5b0d50526eae26687f240690810ea  m3-batch59-code-r1-opus-model/expected/mut-r-each-entry-judges-i313-legit-unlink.txt
e3bf4b454e764fc5d8f72b6f3d55eb2b3009bb0013585b1c750040287b2f5b91  m3-batch59-code-r1-opus-model/expected/mut-r-each-entry-judges-i313+r-tag-judged+r-count-skips-unregistered-tag.txt
36a5690c01e8d1166251574532039b9e69b679074b983221fe77ce5388f15490  m3-batch59-code-r1-opus-model/expected/mut-r-each-entry-judges-i313.txt
890ba3cb80a96a5e947feffcacfc0f1ca50666254990b668bb91a049dcb273c3  m3-batch59-code-r1-opus-model/expected/mut-row3668.txt
4e97bf3feba202d1ed09bc5273203fc675504638302ccc80abb0c876f91df415  m3-batch59-code-r1-opus-model/expected/mut-row3861.txt
49fa27394ff9e49639bcbd0a4d511600ad024d0e7b7eacc5eef1bc5c3640c175  m3-batch59-code-r1-opus-model/expected/mut-row3868.txt
59fd21e9820de203871136c67638837a22a77d0715972d4bf2ca35edf977c100  m3-batch59-code-r1-opus-model/expected/mut-r-tag-judged.txt
4b8c8514b5bb6c7f40f4438138c2bac99127368a4204011981e75589b2496712  m3-batch59-code-r1-opus-model/expected/mut-w-first-only-harness-a.txt
d9c18cf0843dfb6139beba7a8ecba6ae928495c7cfc16137d7e47394248976ab  m3-batch59-code-r1-opus-model/expected/mut-w-first-only-harness-b.txt
c300b14c27523fe33ddd4f848971236c0b69976b57b209a2bfc80fe5a14d8aa4  m3-batch59-code-r1-opus-model/expected/mut-w-first-only-m1-j3.txt
611cac9bbf86e793d0d876bf7928d4a4796a29649aa48450963af15174fba2fe  m3-batch59-code-r1-opus-model/expected/mut-w-first-only-m1-l1.txt
c53895b633a3a664497ea2f4fcc089461b126336d54c52e720868a0ca67ac823  m3-batch59-code-r1-opus-model/expected/mut-w-first-only-m1-l2.txt
2f20e948b9f0133eda12bbb0597de3d36fb7d9829b433f8ecb6d7977e84d5d71  m3-batch59-code-r1-opus-model/expected/mut-w-last-four-harness-a.txt
d9c18cf0843dfb6139beba7a8ecba6ae928495c7cfc16137d7e47394248976ab  m3-batch59-code-r1-opus-model/expected/mut-w-last-four-harness-b.txt
42ce33853af1e3d88246ea8a807226e3d2beaa996b2e2fb3f55db24f65bac6cd  m3-batch59-code-r1-opus-model/expected/mut-w-last-four-m1-j3.txt
ba6a4474777f9fb67bdee65d9d3cdb4fcb72c85e501087cf31ccb25b7a9d3cb1  m3-batch59-code-r1-opus-model/expected/mut-w-last-four-m1-l1.txt
30f7a7a71bae762845fe3cb88196a08cf3083a644163dffb8c1e13a0a23d739d  m3-batch59-code-r1-opus-model/expected/mut-w-last-four-m1-l2.txt
2f20e948b9f0133eda12bbb0597de3d36fb7d9829b433f8ecb6d7977e84d5d71  m3-batch59-code-r1-opus-model/expected/mut-w-last-three-harness-a.txt
d9c18cf0843dfb6139beba7a8ecba6ae928495c7cfc16137d7e47394248976ab  m3-batch59-code-r1-opus-model/expected/mut-w-last-three-harness-b.txt
42ce33853af1e3d88246ea8a807226e3d2beaa996b2e2fb3f55db24f65bac6cd  m3-batch59-code-r1-opus-model/expected/mut-w-last-three-m1-j3.txt
ba6a4474777f9fb67bdee65d9d3cdb4fcb72c85e501087cf31ccb25b7a9d3cb1  m3-batch59-code-r1-opus-model/expected/mut-w-last-three-m1-l1.txt
30f7a7a71bae762845fe3cb88196a08cf3083a644163dffb8c1e13a0a23d739d  m3-batch59-code-r1-opus-model/expected/mut-w-last-three-m1-l2.txt
4b878c8ac02fecf106d672951de99c77e2a2a6d19029ff9408808ecdf4e91ecb  m3-batch59-code-r1-opus-model/expected/mut-w-last-two-harness-a.txt
d9c18cf0843dfb6139beba7a8ecba6ae928495c7cfc16137d7e47394248976ab  m3-batch59-code-r1-opus-model/expected/mut-w-last-two-harness-b.txt
42ce33853af1e3d88246ea8a807226e3d2beaa996b2e2fb3f55db24f65bac6cd  m3-batch59-code-r1-opus-model/expected/mut-w-last-two-m1-j3.txt
ba6a4474777f9fb67bdee65d9d3cdb4fcb72c85e501087cf31ccb25b7a9d3cb1  m3-batch59-code-r1-opus-model/expected/mut-w-last-two-m1-l1.txt
ebaeeb4aaf385197fb8129728efcef6748ada8678f5a22ec44f8df39915850ba  m3-batch59-code-r1-opus-model/expected/mut-w-last-two-m1-l2.txt
bf3222a1db24fd25203eb4a3d32e4e1940903f9f2fb437036b83e14fadf352d5  m3-batch59-code-r1-opus-model/expected/mut-w-second-to-last-harness-a.txt
d9c18cf0843dfb6139beba7a8ecba6ae928495c7cfc16137d7e47394248976ab  m3-batch59-code-r1-opus-model/expected/mut-w-second-to-last-harness-b.txt
42ce33853af1e3d88246ea8a807226e3d2beaa996b2e2fb3f55db24f65bac6cd  m3-batch59-code-r1-opus-model/expected/mut-w-second-to-last-m1-j3.txt
42a3b2560a5297cd2f6a80a1279770083952414cfe5f8346457238c31d3a0308  m3-batch59-code-r1-opus-model/expected/mut-w-second-to-last-m1-l1.txt
545a8a7289229e89832453340d598e140089680d3a01cdae5cd567e16196869f  m3-batch59-code-r1-opus-model/expected/mut-w-second-to-last-m1-l2.txt
55155d4f1e478785d828cfad022fa60e4ce874d2f1c770fada3fc8e929091e72  m3-batch59-code-r1-opus-model/extract.py
f3ba9a7e3d76e419b996cc7a68d5f60cdc205541787963010d3db2c9cd049b80  m3-batch59-code-r1-opus-model/mutants.py
e8b65839f5f14be20ac2b90ca9455d9d7c1c47f61918a9cc95b25eae6b36c7b4  m3-batch59-code-r1-opus-model/probe-crate/Cargo.toml
90727beff11b51441c1dee4395a51bd908379468fa75273eb109c8aec015136c  m3-batch59-code-r1-opus-model/probe-crate/tests/flows_b59/mod.rs
230abea9155c2b4125d8ffed8593fffff866c5871f12716cbe32e38e12ce6b9e  m3-batch59-code-r1-opus-model/probe-crate/tests/m1_gpu_flow_proxy.rs
587857ec309881801d1d5211961c5f7df90a3e4ad70517fe466644d2a9a56033  m3-batch59-code-r1-opus-model/probes/core_review_l4_append.rs
a205d5fb50ada553f19b7c2b6677721107878d9181f4160be1eedbddb4291685  m3-batch59-code-r1-opus-model/rerun.sh
```

交回前从空目录 `~/.local/share/singlefs/work/m3-batch59-code-r1-opus/run-final2/` 用仓里这份模型目录从头跑完 46 段（`OPUS_SEED_TARGET` 指我开发时那份 release 编译目录当三方依赖的种子，副本源码先全部 touch；本机负载 6–170，分 15 次起、合计挂钟约 28 分钟），compare 原样末行「compare：46 份，不一致 0 份」。

## 各格判定一览

| 格 | 判定 | 依据（「量过」= 本腿副本上的原样输出，产物在模型目录 `expected/`；「推的」= 读码） |
|---|---|---|
| M1-1 三条新对拍是不是只因旋钮支判红 | 没打中（CPU 那一半量过，GPU 那一半推的） | 替身照抄被判那三条流，状态数 561 / 489 / 621、旋钮那一条的违例态 429 / 393 与主 agent 卡上那一趟的 CPU 数逐个相同；每个状态上 CPU 带旋钮 2 的入口除了这一条一条别的都不红（`states_with_another_violation=0`，`base-m1-l1.txt`） |
| M1-2 转过根环后候选集里剩哪几版，只判倒数第二片在哪分得开 | 没打中（量过） | 只判倒数第二片丢掉的违例态全落在恢复落到最后 8–9 版的状态上：谱系 txg 33..=40 共 93 个、墓碑 38..=46 共 105 个、退役 44..=51 共 93 个；更早的状态候选集里还有 [2,1,1] 或 [2,1,2] 那一版，两种判法同判。根环 8 槽 × 3 区 = 24 版，30 次发布多出 6–7 版余量 |
| M1-3 建 30 次空 inode 会不会改掉被判那一版的容器树 | 没打中（量过） | 三棵树 30 次发布每一次之后这一棵都是 [2, 1, 2, 1]（`ring_turn_shapes=[2, 1, 2, 1]x30`） |
| M1-4 GPU 只判最右之前两片（k = 2） | **打中（弱；分辨批次；CPU 替身量过，GPU 推的）** | GPU 全部旋钮对拍流里少一条的那片离最右至多两片：k = 2 的改坏在 J3 三条与 L1 三条上替身摘要逐个等于基线（`mut-w-last-two-m1-{j3,l1}.txt`），其余 GPU 旋钮流至多三片（读码）。CPU 那一侧 L2 钉住了 k = 2，GPU 那一侧没有对应的流。加三条 [2,1,2,2,1] 转根环的 GPU 流就分得开（替身上违例态 417→324、477→372、513→420，量过、零轮） |
| M2-1 [2,1,2,2,1] 三张盘是不是只由点名的写错而红 | 没打中（量过） | 基线 12 条照绿；k = 2 只红这三条，红在「违例清单恰好是这一棵那一条」那句断言上（left 是空表）；只判倒数第二片红六条（加上 [2,1,2,1] 三条）；只判叶序第一片红 12 条 |
| M2-2 k = 3、4 在已有盘上哪一张分得开 | 一张也分不开（量过）——正文第二节已知欠账，不当打中报 | 旋钮坏镜像里名字带 knob 的与按产品镜像判的两条共 39 条（12 + 27，带旋钮入口的 36 条都在里面）、checker 单测 32 条、容器树长层 6 条照绿；GPU 替身 9 条流摘要逐个等于基线 |
| M2-3 去掉 `#[ignore]` 后在快档里与别的用例一起跑 | 没打中（量过） | debug 整份 `checker_known_bad_images` 原样「test result: ok. 265 passed; 0 failed; 2 ignored」，挂钟 111 秒（10 线程、本机负载约 60）；只跑那 12 条 12 秒。用例之间不共享状态：镜像文件名带标签、进程号与原子计数（读码），标签没有重复（237 个，`uniq -d` 空） |
| M3-1 J1 ② 可写挂载那条是不是只由「分配记录判在逐条判之前」分得出 | 没打中（量过） | 第 3868 行那一形在两份用例 56 条里只红它，报「记账条目的代晚于根或 seq 为 0」；「分配记录排在条目宽之前」那半句照旧没钉（用例注释自己写了，已知） |
| M3-2 第 3861 行换成的那一形是不是只红 J1 ① 可写挂载那条 | 没打中（量过） | 56 条里只红 `…before_the_mapping_entry_count_by_the_writable_mount`，报映射条目数 |
| M3-3 L4 新造法在第 57 批两种读者改法下 | 没打中（量过） | 两种改法下 15 + 41 条照绿，重建与可写挂载都报水位行缺；只看统计量那一形（第 3668 行）照红 |
| M3-4 读者将来逐条判 I-3.13 时 L4 那一条红在哪 | **打中（一半不分辨造法，一半分辨；量过）** | 读者逐条判 I-3.13 时它是两份用例 56 条、合法历史 71 条里唯一红的一条，红在第 627–633 行那句 `assert_eq`，重建报的是 I-3.13。用例注释第 543–544 行那句推理（数得进去的只能是……「14 个 key 都已经有行」）不成立：统计量 7、13、14 登记过而这一版没写。换成补一行 (7, 0, 0xFFFF_FFFF, 4)，三种读者改法分开或合起来都照报水位行缺，只看统计量那一形下照红（量过、零轮）。不分辨的那一半：按今天的数条目规则，任何造法补进来的那一行都要违反某一条，统计量 7 那一行违反的是 D5 已定项 8 的「没有对象的统计量不写行」 |

## M1　GPU 旋钮流转根环（甲 L1）

### 被判的判法与流（读码，行号按冻结副本）

- GPU 扫描内核三棵树共用一个函数，「最右之外每片 == 旋钮值」那一圈是 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3356`：「for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {」，判的那一句 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3357`：「if (d_container_record_count(container_index) != knob_records) {」。CPU 两个判法各一句「是不是最右」：`crates/singlefs-checker/src/walk.rs:12153`：「position_in_leaf_order + 1 == containers.len();」、`crates/singlefs-checker/src/walk.rs:12054`：「position_in_leaf_order + 1 == container_numbers_and_record_counts.len();」。
- 新三条流写完之后转根环：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14157`：「turn_the_root_ring_over_by_creating_inodes(&mut pool);」，次数 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:115`：「pub const PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER: u64 = 30;」；根环每区 `crates/singlefs-format/src/lib.rs:291`：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」，三区共 24 槽。
- 坏流的断言只要 CPU 新入口在某个状态上红这一条、GPU 与 CPU 逐位相同：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13060`：「.contains(knob_capacity_invariant),」。它不要求「只红这一条」，所以「有没有因别的理由红」要看每个状态上别的位。

### 原型（`probe-crate/tests/m1_gpu_flow_proxy.rs`、`flows_b59/mod.rs`）

照抄被判那几条流的造法（名字不带 layer0 的 GPU 用例文件），盘换成内存稀疏盘外包录制器；状态域取被判用例自己的展开（段长 ≤ 8 的段每个真子集，更长的不展开）。每个状态经 `judge_layer0_state_range_against_the_version_table` 恢复，再交 CPU 池级 checker 带旋钮 2 的新入口，按序号排好取摘要，另按 consulted 那一遍的 `effective_root` 分组数（`by-root` 行）。正确的 walk.rs 与改坏的 walk.rs 各跑一次；摘要相同就是那种改坏在这条流的每个状态上与正确判法同判，GPU 内核里同形的改坏也就在这条对拍上与 CPU 逐位相同（这一步推的）。改坏只改 CPU 两个判法里「跳过哪几片」那一句，与 GPU 第 3356–3357 行同形。

替身与被判那一份是同一组状态的证据：L1 三条的状态数与旋钮那一条的违例态，与正文第二节主 agent 卡上那一趟（「谱系 states=489 … cpu_checker_red=393」「墓碑 states=561 … 429 / 429」「退役 states=621」）逐个相同。`base-m1-l1.txt` 原样：
```
OPUS_M1 judged flow=lineage-2-1-2-1-ring states=489 I-9.37_violated_states=393 I-9.37_held_states=36 states_with_any_violation=393 states_with_another_violation=0 digest=552523ac1818b70b
OPUS_M1 judged flow=retirement-2-1-2-1-ring states=621 I-9.37_violated_states=441 I-9.37_held_states=108 states_with_any_violation=441 states_with_another_violation=0 digest=ce44aa9a3f7693a2
OPUS_M1 judged flow=tombstone-2-1-2-1-ring states=561 I-9.22_violated_states=429 I-9.22_held_states=72 states_with_any_violation=429 states_with_another_violation=0 digest=8f3e664d2c610de5
```
层 0 全量（每段都展开）三条各 87 032 054、1 570 767 160、1 480 589 594 个状态（`closed_form_full`），都超过每次 10⁶ 的线，没跑。

### M1-1、M1-3：没打中

- 正确判法下每个状态除了旋钮那一条一条别的都不红（`states_with_another_violation=0`，三条都是）。用例在错的理由上红的形状，CPU 这一侧没有；GPU 那一侧退役那条 24 个状态判不了是正文第二节的已知欠账。
- 建 30 次空 inode 不改那棵树：写的每一步与转根环的每一步这一棵按叶序的记录数，`base-m1-l1.txt` 原样：
```
OPUS_M1 shape flow=lineage-2-1-2-1-ring written=[1]x1 [2]x1 [2, 1]x1 [2, 1, 1]x1 [2, 1, 2]x1 [2, 1, 2, 1]x1 ring_turn_txgs=11..=40 ring_turn_shapes=[2, 1, 2, 1]x30 last_txg=40
OPUS_M1 shape flow=retirement-2-1-2-1-ring written=[1]x1 [2]x1 [2, 1]x1 [2, 1, 1]x1 [2, 1, 2]x1 [2, 1, 2, 1]x1 ring_turn_txgs=22..=51 ring_turn_shapes=[2, 1, 2, 1]x30 last_txg=51
OPUS_M1 shape flow=tombstone-2-1-2-1-ring written=[1]x1 [2]x1 [2, 1]x1 [2, 1, 1]x1 [2, 1, 2]x1 [2, 1, 2, 1]x1 ring_turn_txgs=17..=46 ring_turn_shapes=[2, 1, 2, 1]x30 last_txg=46
```

### M1-2：只判倒数第二片在哪几版上分得开（没打中）

`by_root_diff.py` 比基线与改坏的 `by-root` 行，列出旋钮那一条违例态数不同的根（命令 `cd expected && python3 ../by_root_diff.py base-m1-l1.txt mut-w-second-to-last-m1-l1.txt`），原样：
```
lineage-2-1-2-1-ring differing_roots=8 root_txgs=33..=40 knob_violated_states_lost=93
retirement-2-1-2-1-ring differing_roots=8 root_txgs=44..=51 knob_violated_states_lost=93
tombstone-2-1-2-1-ring differing_roots=9 root_txgs=38..=46 knob_violated_states_lost=105
```
谱系那条 [2, 1, 2] 是 txg 9、[2, 1, 2, 1] 是 txg 10；恢复落到 txg 33 起的状态，候选集（24 版）里才不再有 txg 9 及更早那几版，只判倒数第二片在这些状态上不红、正确判法红。分得开的状态只在最后 8–9 版里，30 次发布比转出所要的 23 次多 7 次。所以甲的 GPU 变异第 3862–3864 行在墓碑、谱系两条上能红（与主 agent 卡上「点名 2 条：跑了 2 条，抓到 2 条」相容），注释那句「写完之后的状态上候选集里只剩 [2, 1, 2, 1]」成立。J3 三条在这一改坏下摘要逐个等于基线（`mut-w-second-to-last-m1-j3.txt`），与第 58 批 K1-3 一致。

### M1-4 GPU 只判最右之前两片（k = 2）照绿：打中（弱）

改坏 `w-last-two`（两个 CPU 判法里离最右三片及更远的不判，与变异表第 3865–3867 行同形；GPU 同形是把第 3357 行换成 `if (container_index + 3u >= d_container_count && d_container_record_count(container_index) != knob_records) {`）。替身上 J3 三条与 L1 三条的摘要逐个等于基线（`mut-w-last-two-m1-j3.txt`、`mut-w-last-two-m1-l1.txt` 与 `base-m1-*.txt` 的 `judged` 行逐字相同），`by_root_diff.py` 原样：
```
lineage-2-1-2-1-ring differing_roots=0 root_txgs=- knob_violated_states_lost=0
retirement-2-1-2-1-ring differing_roots=0 root_txgs=- knob_violated_states_lost=0
tombstone-2-1-2-1-ring differing_roots=0 root_txgs=- knob_violated_states_lost=0
```
别的 GPU 带旋钮的流至多三片（读码：第 56 批 G3、G6 与第 57 批 H1 那几条是两片或回退两次，最多的是 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13362`：「"three-rollbacks-rightmost-lineage-container-empty",」那一条三片），少一条的那片离最右至多两片。所以 GPU 内核里 k = 2 的改坏在全部 GPU 对拍上照绿（推的，GPU 不跑）。

我提的 [2, 1, 2, 2, 1] 转根环三条 GPU 流（与 harness L2 同形）在替身上分得开，`by_root_diff.py base-m1-l2.txt mut-w-last-two-m1-l2.txt` 原样：
```
lineage-2-1-2-2-1-ring differing_roots=8 root_txgs=35..=42 knob_violated_states_lost=93
retirement-2-1-2-2-1-ring differing_roots=8 root_txgs=50..=57 knob_violated_states_lost=93
tombstone-2-1-2-2-1-ring differing_roots=9 root_txgs=42..=50 knob_violated_states_lost=105
```
状态数 513 / 693 / 609（`base-m1-l2.txt`），比 L1 三条各多 24–72 个。

四句：
1. 分不分辨：分辨。正确的 GPU 判法与「只判最右之前两片」在全部 GPU 对拍流上判不出差别；CPU 那一侧第 59 批 L2 已经把它分开（harness 那三张盘），GPU 那一侧没有。
2. 判别它的东西当时看得到吗：GPU 判器看得到整张容器表；看不到的是用例，GPU 每条流每一版的容器表里少一条的那片都离最右至多两片。
3. 满足的分句：正文第三节 M1「三条新对拍是不是只因旋钮支判红」之外的「改坏照绿」那一类，具体是派发点名的「只判最右之前 k 片 k = 2」在 GPU 那一半。和已知欠账「只判最右之前 k 片对足够大的 k 有限组排布关不死」的关系：同一族；CPU 那一侧窗口已推到 3，GPU 那一侧还停在 2（L1 只关了 k = 1）。主 agent 判它算不算已知欠账里的那一笔。
4. 改法在这格上中不中：L1 的写法照中。加三条 [2, 1, 2, 2, 1] 转根环的 GPU 对拍、各登一行 GPU 变异（第 3357 行换成上面那一句）就不中：替身上量过、被攻过零轮；GPU 上的状态数、挂钟与退役那条会不会照样判不了，都没量。

## M2　旋钮支离最右更远与慢档（乙 L2、L5）

### 量法

旋钮坏镜像一份 265 条，在本机负载下一次跑不进前台时限，只跑调到带旋钮入口的那几条：名字带 `records_under_changing_knobs` 的 12 条（第 58 批 J3 九条、第 59 批 L2 三条，a 段），其余名字带 `knob` 的与两条「按产品镜像判」的共 27 条（b 段）；带旋钮入口（`check_pool_image_given_the_container_capacity_knobs`）的调用方我按调用图从 7 处调用往上找过，36 条用例都在这两段里。另跑 checker 单测 32 条、容器树长层 6 条。新三张盘的写法 `crates/singlefs-harness/tests/checker_known_bad_images.rs:17093`：「records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],」，断言要违例清单恰好一条 `crates/singlefs-harness/tests/checker_known_bad_images.rs:16759`：「[tree.knob_capacity_invariant()],」。

### M2-1：新三张盘只由点名的写错而红（没打中）

基线 a 段「test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 255 filtered out」、b 段「test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 240 filtered out」。各改坏下 a 段（`mut-*-harness-a.txt`）：

| 改坏 | a 段 12 条 | 红的是哪几条 |
|---|---|---|
| `w-last-two`（k = 2） | 「test result: FAILED. 9 passed; 3 failed」 | 只有 [2,1,2,2,1] 三条 |
| `w-second-to-last` | 「test result: FAILED. 6 passed; 6 failed」 | [2,1,2,1] 三条与 [2,1,2,2,1] 三条 |
| `w-first-only` | 「test result: FAILED. 0 passed; 12 failed」 | 全部 |
| `w-last-three`、`w-last-four` | 「test result: ok. 12 passed」 | — |

b 段、checker 单测、容器树长层在五种改坏下都照绿。k = 2 下红在「违例清单恰好是这一棵那一条」那一句、left 是空表（`mut-w-last-two-harness-a.txt` 里 `left: []`、`right: ["I-9.22"]`），不是被别的不变量先拒。

### M2-2：k = 3、4（已知欠账，不当打中报）

`w-last-three`、`w-last-four` 下 a、b 两段 39 条、checker 单测 32 条、容器树长层 6 条全绿，GPU 替身九条流的摘要逐个等于基线（`mut-w-last-three-m1-*.txt`、`mut-w-last-four-m1-*.txt`）。已有盘里少一条的那片离最右至多三片（[2,1,2,2,1] 的容器 2），k ≥ 3 一张都分不开。正文第二节「「只判最右之前 k 片」对足够大的 k 有限组排布关不死」就是这一笔。

### M2-3：去掉 `#[ignore]` 后在快档里跑（没打中）

debug（快档）整份跑一次，`debug-fast-tier.txt` 原样：
```
command args=-p singlefs-harness --test checker_known_bad_images rc=0
binary checker_known_bad_images
test result: ok. 265 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
command args=-p singlefs-harness --test checker_known_bad_images -- records_under_changing_knobs rc=0
binary checker_known_bad_images
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 255 filtered out
```
挂钟（`.wall`，不进 compare）：整份 111 秒、只跑那 12 条 12 秒，10 线程，本机负载约 60。剩下的 2 条 `#[ignore]` 是产品容量那两条（退役 960 次、回退 1360 次）。用例之间不共享会变的状态：镜像文件名是 `crates/singlefs-harness/tests/common/mod.rs:49`：「let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);」加标签与进程号，这一份里 237 个标签没有重复（`grep -o '"known-bad-[a-z0-9-]*"' … | sort | uniq -d` 空）；这一份与它 `mod` 进来的三份公共模块里没有 `set_var`、`static mut`、`OnceLock`。变异表里点名这几条的行：第 3854、3857 行带 `--include-ignored`，去掉 `#[ignore]` 之后照样选得到；第 3852、3853、3855、3856 行的过滤串现在多选上这六条（12 条合起来在上面那一次里 12 秒）。

## M3　重建次序补钉与水位用例换造法（丙 L3、L4）

### 被判的判法（读码）

重建里分配记录逐条判在读记账树之前 `crates/singlefs-core/src/recovery.rs:2894`：「allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;」；逐条判记账条目 `crates/singlefs-core/src/recovery.rs:4876`：「if entry.generation > root.checkpoint_txg || entry.sequence == 0 {」（冷走读与重建共用）；数记账条目不算台阶行 `crates/singlefs-core/src/recovery.rs:4795`：「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg」；水位行那一判 `crates/singlefs-core/src/recovery.rs:2938`：「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg」。I-3.13 在 kb 里只由池级 checker 判：`.claude/kb/invariants.md:155`：「记账行代段等于这一版的 txg」。

### M3-1、M3-2：两行变异各只红点名的那一条（没打中）

两份用例 56 条（`core_review…` 13 条加我接的 2 条原型、`corrupt_on_disk…` 41 条），`--no-fail-fast`：
- 第 3868 行（L3）：`corrupt_on_disk…`「test result: FAILED. 40 passed; 1 failed」，红的是 `allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_writable_mount`，消息原样「可写挂载报分配记录的代晚于根，不报记账条目的代：Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" })」；`core_review…` 15 条照绿（`mut-row3868.txt`）。
- 第 3861 行（换过的那一形）：同样「40 passed; 1 failed」，红的只是 `…before_the_mapping_entry_count_by_the_writable_mount`，报映射条目数（`mut-row3861.txt`）。第 57 批可写挂载那条与 J1 ② 可写挂载那条在这一形下照绿：这一形把数映射条目提到分配记录判之前，那两张盘的映射条目数是对的。
- 「分配记录排在条目宽之前」那半句照旧没钉：用例注释自己写了（`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:2117`：「条目宽那一判不坏」），属已知欠账，我没再造那一形。

### M3-3：L4 新造法在第 57 批两种读者改法下照绿（没打中）

原型 `opus_probe_b59_l4_constructions` 照被判那一条的步骤造盘（同一个池、同一个改记账行的写法、同一个根槽原地换），三种造法各报什么。基线（`base-m3.txt`）三种都是 `rebuild=Some(Walk(InodeNumberWatermarkRowMissingFromTheAccountingTree)) writable=Some(Recovery(InodeNumberWatermarkRowMissingFromTheAccountingTree))`。补进来的那一行 (1, 0, 0, 3) 在基线上没有被别的判先拒；水位行缺是因为删了自己那行（同一份输出里 16 行的 key 原样列着，统计量 12 只剩 `(12, 65537, 4294967295, 3)`）。
- `r-tag-judged`、`r-count-skips-unregistered-tag` 下两份用例 15 + 41 条照绿；被判那一造法两种改法下都照报水位行缺；第 57 批标签 13 那一造法分别报「记账条目的代晚于根或 seq 为 0」与「记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数」（`mut-r-tag-judged.txt`、`mut-r-count-skips-unregistered-tag.txt`），与上一轮的结论一致。
- 判别力那一形（第 3668 行，只看统计量）下被判那一条照红，`left: None`：重建与可写挂载都收下了只剩台阶的那一版（`mut-row3668.txt`）。

### M3-4 读者逐条判 I-3.13 时 L4 那一条红在哪：打中

改坏 `r-each-entry-judges-i313`：逐条判记账条目时，台阶行之外代段不等于这一版 txg 的行报一个自己的说明（排在原来那一判之后，不改别的判报的成员）。`mut-r-each-entry-judges-i313.txt` 原样（节选失败那一条）：
```
binary core_review_tree_table_duplicates_and_slot_one_search
  failed a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step_is_judged_damaged_by_the_rebuild_and_refused_by_the_writable_mount
    thread 'a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step_is_judged_damaged_by_the_rebuild_and_refused_by_the_writable_mount' panicked:
    assertion `left == right` failed: 从盘上重建 txg 4 那一版：只剩台阶那一行水位，判记账树里没有这一版的水位行
      left: Some(Walk(InvariantViolated { invariant: "I-3.13", detail: "记账行代段不等于这一版的 txg（inode 号水位的台阶行之外）" }))
     right: Some(Walk(InodeNumberWatermarkRowMissingFromTheAccountingTree))
```
红在 `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:632`：「"从盘上重建 txg 4 那一版：只剩台阶那一行水位，判记账树里没有这一版的水位行"」那一句 `assert_eq`，可写挂载那一句也会报 I-3.13（原型那一行 `writable=Some(Recovery(InvariantViolated { invariant: "I-3.13", …`）。同一改坏下 `core_review…`「test result: FAILED. 14 passed; 1 failed」、`corrupt_on_disk…`「test result: ok. 41 passed」，合法历史 8 份 71 条全绿（`mut-r-each-entry-judges-i313-legit-{a,b,unlink}.txt`：5、4、2、23、16、5、7、9 条）：这一收严在这些历史上是合法的，只有 L4 那一条拦着它。三种读者改法合起来也是同一个结局（`mut-r-each-entry-judges-i313+r-tag-judged+r-count-skips-unregistered-tag.txt`）。

用例注释为什么只能补这一行，原文 `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:543`：「数得进去、又不是这一版水位行的只能是统计量不为 12 的一行，而这一版里标签登记过、维度对、」接 `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:544`：「代段等于 txg 的 14 个 key 都已经有行」。后半句不成立：登记表是 1 到 14（`.claude/kb/decisions/05-快照-空间记账机制.md:244`：「码 = 行号（1 到 14；第 8 / 9 项退役不复用；0 保留为无效）」），第 7 项（`.claude/kb/decisions/05-快照-空间记账机制.md:124`：「扩展点配额已用量」）、第 13、14 项都登记过、不带设备维也不带树维，这一版一行都没写（`crates/singlefs-core/src/records.rs` 里统计量常量只有 1–6、10–12）。所以「标签登记过、维度对、代段等于 txg」而这一版没有的 key 至少还有三个。

换成补 (7, 0, 0xFFFF_FFFF, 4)（原型里的 `RegisteredStatisticSevenOfThisVersion`）：基线、两种第 57 批读者改法、逐条判 I-3.13、三者合起来，重建与可写挂载都报水位行缺；只看统计量那一形下照红（原型断言那一条 `opus_probe_b59_l4_registered_statistic_seven_reaches_the_watermark_judge` 在 `mut-row3668.txt` 里红、`left: None`，在其余各份里绿）。三者合起来那一份原样：
```
rebuild=Some(Walk(InodeNumberWatermarkRowMissingFromTheAccountingTree)) writable=Some(Recovery(InodeNumberWatermarkRowMissingFromTheAccountingTree))
```
（这一行取自 `mut-r-each-entry-judges-i313+r-tag-judged+r-count-skips-unregistered-tag.txt` 里统计量 7 那一行的后半截。）

但这不是关得死的改法。按今天的数条目规则（台阶行之外要恰好 3 + 6 × 盘数行），删了自己那行水位还要数得进去，补进来的那一行总得违反某一条：(1, 0, 0, 3) 违反 I-3.13 与 D5 已定项 2 的「每条根的记账树只装这条根那一代的行，不留多代」（`.claude/kb/decisions/05-快照-空间记账机制.md:47`）；标签 13 那一行树段是 65537，第 13 项不带树维，维度不对；统计量 7 那一行违反 D5 已定项 8 的「没有对象的统计量不写行」（`.claude/kb/decisions/05-快照-空间记账机制.md:204`）。读者把这几条都判全时，水位行那一判在任何一张盘上都排在别的判之后、先报的永远是别的成员：它在完整的读者里只剩 panic 面那一层用处（`TransactionOutput::inode_number_watermark` 的 expect 靠它），用例能钉的只是「读者少判某一条时它还在」。

四句：
1. 分不分辨：一半分辨。被判那一造法与统计量 7 那一造法，在「读者逐条判 I-3.13」这一格上分得开（前者红在第 632 行那句、后者照报水位行缺，量过）。另一半不分辨：任何造法都要靠读者少判一条，病根在数条目规则与水位行那一判的共同前提里，照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 的「打中不分辨臂」另记一笔，不拿它判造法。
2. 判别它的东西当时看得到吗：看得到。重建读到的那一版里多出来的那一行就在记账叶里，代段、标签、维度都是盘上的字节，读者判不判只是它写没写那一判。
3. 满足的分句：正文第三节 M3「读者将来逐条判 I-3.13 时它红在哪」与「补进来的 (1, 0, 0, 3) 会不会……让「水位行缺」在错的理由上报出」的反面——今天报的理由是对的，收严之后报的是 I-3.13、用例就红；还有用例注释第 543–544 行那句推理不成立。判决 L4 的验收原文只要第 57 批两种读者改法下照绿（那一格照过，M3-3），所以这是修法没罩到的地方，不是 L4 的验收没过。
4. 改法在这格上中不中：被判那一造法照中。换成统计量 7 那一行就不中（量过、被攻过零轮），代价是它依赖「没有对象的统计量不写行」今天、以及可预见的收严里都不判；要关死只能改判法（例如把「水位行缺」并进数条目那一判的报错，或用例改成只判「先报的不是 panic」），那是 `crates/*/src` 的改动，交主 agent 判。用例注释那句推理要改掉，不论换不换造法。

## 几个改法各修哪一格

| 改法（都只在我的模型上量过或推过、被攻过零轮） | M1-4 GPU 只判最右之前两片 | M2-2 k ≥ 3 | M3-4 读者逐条判 I-3.13 |
|---|---|---|---|
| GPU 加三条 [2,1,2,2,1] 转根环的旋钮对拍、各登一行 k = 2 的 GPU 变异 | 量过（替身）：修，违例态 417→324、513→420、477→372（`base-m1-l2.txt` 与 `mut-w-last-two-m1-l2.txt`） | 量过：不修（`mut-w-last-three-m1-l2.txt` 摘要等于基线） | 无关 |
| L4 改成补 (7, 0, 0xFFFF_FFFF, txg) | 无关 | 无关 | 量过：修（三种读者改法分开与合起来都照报水位行缺，`mut-*.txt` 里统计量 7 那一行）；推的：读者判「没有对象的统计量不写行」时照中 |
| 改正用例注释第 543–544 行那句推理 | 无关 | 无关 | 推的：不改判别力，只去掉错的前提 |
| 把「水位行缺」并进数条目那一判、或改判法让它先于别的判（改 `crates/*/src`） | 无关 | 无关 | 推的：能关死，没实现、没跑 |

## 没打中的形状

- M1：新对拍在正确判法下因别的不变量红——三条流每个状态都查了全部 90 位，`states_with_another_violation=0`。30 次建 inode 改了容器树——30 步每一步查了形状，没变。转根环不够、[2,1,1] 那几版还在候选集里——只判倒数第二片在最后 8–9 版上分得开，排除。只判叶序第一片（替身会红的自证）：L1 三条违例态归 0（`mut-w-first-only-m1-l1.txt`）。
- M2：新三张盘在正确判法下被别的不变量拒——12 条照绿、断言要违例清单恰好一条；k = 2 下红在同一句断言（清单空）。快档里与别的用例一起跑结局变了——debug 整份 265 条照绿；查了共享状态（文件名、全局变量），没有。
- M3：第 3868、3861 两行各红点名之外的用例——56 条里各只红一条。L4 那一行 (1, 0, 0, 3) 在基线上被别的判先拒——没有，报水位行缺；第 57 批两种读者改法下照绿。
- 取样：GPU 替身每次 9 条流共 3 933 个状态（被判用例自己的展开）；层 0 全量每条 2.2 × 10⁷ 到 1.7 × 10⁹，没跑。harness 那几份在 release 下只跑调到带旋钮入口的 39 条与 checker 单测、容器树长层，整份 265 条只在 debug 基线上跑过一次。读者收严只造了三种（两种第 57 批的、I-3.13 一种），维度判、登记表判与「没有对象不写行」判没造。

## 这条腿自己的限度

- GPU 一条没跑。M1 的 GPU 结论靠 CPU 替身：替身判的是同一组状态（状态数与卡上那一趟逐个相同），比的是池级 checker 那几位；推不到 GPU 内核在这些状态上会不会在别处先判不了（退役那条今天就是）。
- 原型的池是内存稀疏盘上照 `build_pool` 搭的，不是被判用例的文件镜像。
- 本机负载 6–170、多数时候被别的会话 nice 0 的变异复跑占满（`ps` 看到 `singlefs-crates-mutation-target-w*` 下的测试二进制合计约 3 000% CPU），我这边 nice 19 的 cargo 与测试多次跑不进 215 秒、被 `timeout` 停掉（整个进程组一起停，事后 `ps` 查过没留进程），同一个工作目录接着跑。内存包装有一次按 6G 排队 3 分钟没排上（slice 已占 37.0 GiB），之后改 3G。
- 第一遍复跑（草稿目录 `run-final/`）的 `base-build` 里有一组目标名写错（数组改名后没跟着改），那一组变成不带 `-p` 的 `cargo test --release --no-run`，在那个副本里编了整个工作区的测试二进制（含 checker 档的，只编没跑），编译目录涨到 26G；改正后那一遍 46 份照样「不一致 0 份」，但跑的中途模型改过三处（这一组、`--no-fail-fast`、变异副本的编译种子），不算数，整个目录删了。
- 交回前从空目录 `run-final2/` 用仓里的模型目录从头跑完 46 段，compare 的末行贴在「复跑」一节末尾。
- 跑过的状态量：替身合计约 6 × 10⁴ 个状态（开发、两遍复跑与被停掉的几次），每次至多 3 933 个，没有一次超过 10⁶。
- 单次取样：没打中的几格只抽了这一次。

## 要主 agent 跑的命令

不要求我这条腿的结论成立才跑；是把 M1-4 落到 GPU 上的唯一一步（checker 档、带 gpu，重型，归主 agent）：在冻结副本的拷贝上把 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 第 3357 行换成 `if (container_index + 3u >= d_container_count && d_container_record_count(container_index) != knob_records) {`，跑 `SINGLEFS_HEAVY_TESTS=user-request bash scripts/run-with-memory-cap.sh 12G cargo test --offline --release -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- knob_two_agrees_with_the_cpu_on`（工作目录是那份拷贝的根，线程照 `capped.sh`）。这个过滤串选上 9 条（J3、L1、第 57 批 H1 各三条）；退役 L1 那一条基线就红（正文第二节已知），不看它；墓碑、谱系的 J3 与 L1 四条都绿就坐实 M1-4，其中有一条红就推翻它。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑任何门禁阶段、checker 档测试、GPU；没跑崩溃点与变异两条轻型。
- 没造「分配记录判挪到条目宽与逐条判之间」那一形（已知欠账）；没量我提的 GPU 流在卡上的状态数与挂钟。
- 正推腿的报告与模型、本轮判决没读（禁读）。

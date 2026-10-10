# m3-batch19-code-r1 攻方腿（Opus）报告（2026-10-05）

攻击面：正文第三节 K2（重点 N3、N4、N7，另带 N1、N6）。只在冻结副本的拷贝上编跑 harness 档测试与自己的原型 crate；副本上量出的数不算入库装置上的数。

## 复跑

- 模型目录 `research/prompts/m3-batch19-code-r1-opus-model/`。`bash research/prompts/m3-batch19-code-r1-opus-model/rerun.sh list W` 按次序列出阶段，逐个 `bash research/prompts/m3-batch19-code-r1-opus-model/rerun.sh <阶段> W`（W 是空目录；冻结树认 `FROZEN_TREE`，默认 `/tmp/claude-1000/m3-batch19-code-r1/tree`；线程 `ATTACK_THREADS` 默认 6；每阶段前台 4 分钟以内，最长的 n34-176-3-36 量过 134.77 秒），最后 `summary`、`compare`（与 `out/summary.txt` 逐行比，一致退 0）。
- 交回前在空目录 `/tmp/claude-1000/m3-batch19-code-r1-opus/rerun-W` 从 `setup` 逐阶段跑到 `lone-k3-n15-arm6`；之后加了 `n7-region` 阶段（只读 `out/n7.txt` 做统计）与 summary 的一个匹配模式，这两样用改后的 rerun.sh 重跑，再 `summary`、`compare`，输出「summary 逐行一致」。`out/` 里的文件就是这一遍的产物（35 个）。
- core 的臂开关与错实现不改冻结树：`setup` 把冻结树拷两份，一份照 `scripts/make_arm_patch.py` 加线程各自的臂（`B19_ARM`，0 = 冻结副本原样）与错实现开关（`B19_MUT`，不设 = 原样），另一份原样留给 `tsv-mut` 照 `crates/mutations.tsv` 原文施加两行变异。`patches/core-arms-and-mutants.diff` 是打完之后与冻结树的 diff，只供阅读。

| 文件 | sha256 |
|---|---|
| rerun.sh | d3ff0ebe9503d89ab5e1cbcc055dbf842e5bf52ace60c6685a57024b8224f701 |
| scripts/make_arm_patch.py | 3583a76c972989cd80fdfcb86deda6045b8ef6964f60614183c03f61221a4fff |
| scripts/install_probe.sh | 5719cc42cd5a0fb3e1b21f7588fb64971d0d817bc70fb2c91ee0abdafba30328 |
| scripts/apply_mutation.py | fe35bd15779307ffd24799085ddff5aefa73bd64c272146a153aacca389dd827 |
| scripts/n7_region.py | e9ef0462d4475d8f911641b4bf71af85da619dee344e53263181fae8ac88a7d1 |
| probe/Cargo.toml | 073e8113d44e0a4a6481b4b99687e1bc33eced1229b903c2e6f1b7fcf3572fd9 |
| probe/main.rs | 8e2bf2717c73ea69a2d3481ddf7c5b4ad3ffaa80a432c025bf2dae75cb70eea3 |
| probe/n7.rs | 7f9239e8dc848979c243bdc731fa0180b1a46e0aaaa3f5705dc4e25beaa75d88 |
| probe/extra.rs | c54ea75fa8b94826017a8ed42d83f8862ad6af121212ea15db25fa7742f3905d |
| probe/lone.rs | 5b6c6f347e0b123d76663122b224e6e01b9407d77d36c1b8b759b10c7f18db47 |
| patches/core-arms-and-mutants.diff | 8685ce48443ba48fd360589e701f474c3f0afd1cf6d6884d2b90a12b2d7226b9 |
| out/summary.txt | d74d6da3c02291449e6325900e248827076e21f73b5e6ab95be05fd369c9c134 |

臂的定义（`scripts/make_arm_patch.py` 文件头与 `mount.rs` 补丁里那段注释）：0 合法（冻结副本原样）；1 甲（= `crates/mutations.tsv` 删除批第 19 批 甲那一行：写行之后、暖机之前放开，预演不动）；2 N4（= 删除批第 19 批 N4 那一行：暖机之后不放开）；3 甲同步（暖机之前放开，预演在分配器拷贝上同一处也放开，预演对真发那一判不分叉）；4 预演之前放开（取号之前就放开）；5 第一次暖机之后放开（预演不动）；6 第一次暖机之后放开、预演同一处也放开。错实现：`q-unlink-mono`（会话层 unlink 的时间取给的与这个线程上一次会话 unlink 用过的时间里较大的那个）；`c2-third`（产品批大小取 max(144, ⌈剩下单元数 ÷ 3⌉)）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| N3 变异在 10f 之后红在哪 | 照原样，不算打中 | 甲那一行在两条新用例上都红在 core 的预演对真发断言（`mount.rs` 第 4905 行起），不在用例自己的断言；用例自己的断言只在臂 3、4 上红（第 658 行，`left: [1410]`）——与用例文档注释说的一致 |
| N3 合法实现误红 | 没打中 | 三份起点、卸载写流层 0 枚举域每个状态（各 8223 个）：凡挂载进门时有扣住槽的状态，合法臂写行与暖机一个单元都没写进换下的槽 |
| N3 / N4 错实现漏红 | **打中** | 臂 6「第一次暖机之后就放开」：整份测试文件 6 条全绿；卸载崩在第 15、16 个写之后（暖机推两次）那几个状态写进扣住槽；K=3 那份历史在第 15 个写之后崩、再挂的 24618 个崩溃状态里 3075 个丢盘 0 之后剩下那块盘读文件报 `DataUnitChecksumMismatchEverywhere`，合法臂 0 个 |
| N4 变异红在哪 | 照原样，不算打中 | N4 那一行红在测试文件第 717 行（`NoSpaceAfterRaisingTheFloor`、`FloorAlreadyAtTheCeiling`），4 GiB 那条在第 596 行 |
| N4 合法实现误红 | 没打中 | U = 8、24、30、36、44、52，S = 176、192、240，K = 0、3：有扣住槽的状态合法臂第一次写全都写成、不推抬 F；写不成的 4 个状态是两份系统配置写都没落下（或撕裂）的状态，两臂一样 |
| N4 判别力的边 | 没打中（列进限度） | N4 那一行在 U ≤ 24 与 S = 240 上全绿、U = 30 只在两次暖机的 3 个状态红；用例取 U = 36、S = 176 落在会红的区里；U 的下沿在 30 与 36 之间（用例文档注释说 32 起红，31–35 我没量） |
| N7 产品路径读到开关 | 没打中 | crates 里除定义之外调 `set_inode_number_watermark_source` 的非测试代码 0 处；挂载、会话、管理员回退各自 `PoolWriter::new` |
| N7「关开关零违例」 | 没打中（末态）；崩溃态没跑 | 17 个号、水位、墓碑、管理员回退、崩溃恢复组合，关开关建一个之后末态 17/17 零违例；那次建 inode 的写流层 0 状态数 1048583 或 16777223，超过约 10⁶，照定义不跑 |
| N7 附带（归用例二） | 信息 | 开关压回 p、建 n 个时 I-9.6 判不出的区域是 p + n 大于树内与墓碑最大号；90 个判不出的点里 42 个压回的号不是发过的最大号（都在 n ≥ 2），测试文件头那句只说「压回的正是发过的最大号」 |
| N1 错实现漏红 | **打中** | `q-unlink-mono`：Q-F 那份测试文件 22 条全绿；同一次挂载先 unlink 第一个文件给 T1、再建一个 inode、再 unlink 它给 T2 < T1，意图那一版那条记录写的是 T1 |
| N6 错实现漏红 | **打中**（可能归进正文已排除的「四点分不出」那一项，交主 agent 判） | `c2-third`：N6 用例全绿、四份文件逐次写调用与换下单元数与产品逐项相同；433 个单元的文件第一批换下 145 个 |

## N3：放开挪到暖机之前的钉

### 变异在 10f 之后的 mount.rs 上红在哪

- 冻结副本的 mount.rs 与主工作区那一份同哈希（6f95b6fb…，`sha256sum` 现算）。`tsv-mut` 阶段在原样拷贝上照 tsv 原文施加「删除批第 19 批 甲」：两条新用例都在 core 里 panic，`research/prompts/m3-batch19-code-r1-opus-model/out/tsv-甲.txt:19`「panicked at crates/singlefs-core/src/mount.rs:4906:9:」（施加之后多一行，对应冻结副本 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4905` 起那个 `assert_eq!`，消息行 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4907`「"取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\」）。N4 那条也在同一处红（`research/prompts/m3-batch19-code-r1-opus-model/out/tsv-甲.txt:27`）。
- 用例自己的断言（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:661`「"写行那次与暖机写下的单元不落在卸载之前现行那一版换下的槽 {released_before_the_unmount:?} 上"」）只在预演不分叉的臂上走得到：臂 3 红在第 658 行（`research/prompts/m3-batch19-code-r1-opus-model/out/testfile-arm3.txt:19`「left: [1410]」），臂 4 同样红、另带三条形 R 用例一起红（`research/prompts/m3-batch19-code-r1-opus-model/out/testfile-arm4.txt:51`「test result: FAILED. 2 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.81s」）。这与用例文档注释（同一文件第 618–620 行）说的一致，不算打中。

### 用户决定的几步放开扫（合法臂会不会误红）

装置：窄盘起点（单元区 S 槽、第一个文件之后覆盖写 K 次、U > 1 时顺序写长到 U 个单元再覆盖写缩成一个），录一次不注错的卸载，卸载写流 36 个写、段 [2, 12, 2, 1, 2, 12, 2, 1, 2]，层 0 枚举（每段都展开、每个状态跑 checker）8223 个状态；每个状态上逐臂照 N3 那条的入口再可写挂载（准入判着、影子账开、批大小 1），记挂载进门时扣住位置着的槽（H，core 补丁里的只读钩子）、写行与暖机写下的槽落在换下的槽 R（用例同一个取法）与 H 上几个。

- 卸载写流的每个状态 checker 零违例、恢复 oracle 零违例：`research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-0-1-a013456.txt:102`「ENUM S=176 K=0 U=1 states=8223 observed=8223 violations=0 failed_states=0 checker_violated={} first_violation=None」。
- 合法臂在有扣住槽的状态上一个都没写进 R：`research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-0-1-a013456.txt:328`「HIST S=176 K=0 U=1 count=4100 arm=0 held>0=true n3=ok inR>0=false inH>0=false wu=1 n4=-」，两次暖机的 3 个状态同样（第 329 行）。K = 3、S = 240 两份起点、U > 1 的九份起点同形（各自文件的 `held>0=true` 那几行，`inR>0=false`）。
- 合法臂写进 R 的 4116 个状态（第 326、327 行）全是扣住为空的状态：卸载那一串带新 F 的根已经落到盘 1，R 当场可再分配。用例取卸载只落下第一个写（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:635`「let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);」），落在扣住区里；把崩溃点挪到第 17 个写及以后会在合法实现上红，那是断言的前提不成立，不是今天的用例误红。

### 错实现漏红（打中）：第一次暖机之后就放开

- 臂 6 在卸载只落下第一个写的那个状态上暖机只推一次，与合法臂逐项相同：`research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-0-1-a013456.txt:114`「PREFIX S=176 K=0 U=1 n=1 state=2 arm=6 held=1 n3=ok inR=0 inH=0 wu=1 n4=-」。整份测试文件在臂 6 上全绿：`research/prompts/m3-batch19-code-r1-opus-model/out/testfile-arm6.txt:13`「test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.69s」（臂 5 同样全绿，`testfile-arm5.txt` 第 13 行）。
- 卸载落下前 15、16 个写（第一次抬 F 空发布的 journal 记录已落、它的根还没落）时这次挂载暖机推两次，臂 6 在第一次暖机之后放开，第二次暖机写进扣住槽：同一文件第 198 行「PREFIX S=176 K=0 U=1 n=15 state=4104 arm=6 held=1 n3=ok inR=1 inH=1 wu=2 n4=-」、第 204 行同形；全枚举里这样的状态 3 个（第 354 行「HIST S=176 K=0 U=1 count=3 arm=6 held>0=true n3=ok inR>0=true inH>0=true wu=2 n4=-」），合法臂这 3 个状态 0 写入（第 329 行）。K = 3 那份起点同形（`research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-3-1-a036.txt:150`「PREFIX S=176 K=3 U=1 n=15 state=4104 arm=6 held=24 n3=ok inR=2 inH=4 wu=2 n4=-」）。
- 用户看得到的结局：K = 3 起点（K = 0 那份卸载之前丢盘 0 就只读挂载不上，`research/prompts/m3-batch19-code-r1-opus-model/out/lone-before.txt:3`「LONEBEFORE S=176 K=0 U=1 txg=3 ["lost0=只读挂载:Open", "lost1=ok"]」，判不了），卸载崩在第 15 个写之后，两块盘都在、再可写挂载一次并录写流（55 个写），再挂写流按层 0 枚举 24618 个状态（每个状态 checker 零违例、oracle 零违例），每个状态丢一块盘、剩下那块盘只读挂载整读活文件，与卸载之前同一块盘上读出的比：
  - 臂 6：`research/prompts/m3-batch19-code-r1-opus-model/out/lone-k3-n15-arm6.txt:103`「LONEH S=176 K=3 U=1 n=15 arm=6 count=3075 lost=0 UNREADABLE 读:DataUnitChecksumMismatchEverywhere」；
  - 合法臂：`research/prompts/m3-batch19-code-r1-opus-model/out/lone-k3-n15-arm0.txt:105`「LONEH S=176 K=3 U=1 n=15 arm=0 count=24609 lost=0 same」，另 9 个状态读到的是现行那一版（第 104 行，比卸载之前那块盘读到的新，内容等于现行那一版）。
- 四句：
  1. 分不分辨臂：分。同一份崩溃镜像、同一段再挂，臂 0 与臂 6 只在「第一次暖机之后放开」这一处不同，结局 0 对 3075。
  2. 被判的系统当时看不看得到判别它的东西：看得到。暖机推几次在取号之前就算好（`warm_up_publishes_planned`），最后一次落盘是在挂载自己的循环里。
  3. 满足判据字面的哪一个分句：正文第五节「漏红：一种违反条款或定案的实现上用例照样绿。」。违反的是 D16（发布语义） 已定项 1 表里「正常卸载」那一行讲卸载半途崩溃之后下一次可写挂载扣住的那一截何时放开的那一句（`.claude/kb/decisions/16-发布语义.md` 第 41 行，用户 2026-10-04 定；只指路，不抄句），代码自己的注释也这么写（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4882`「// 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开」）。
  4. 跑前条款给的改法在打中的格上还中不中：跑前条款只写「打中：主 agent 现查坐实，改法进下一批」，没给具体改法。我提的改法见文末改法表 G1。
- 为什么用例漏：两条新用例都把卸载崩在第 1 个写之后（N4 那条在 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:695`「let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);」），那一刻这次挂载的暖机只推一次（上面 PREFIX 行的 `wu=1`），「第一次暖机之后」与「暖机落满之后」是同一刻。

### 放置决定了断言的判别力

- S = 240 时臂 3（甲同步）在有扣住槽的 4103 个状态上一个都没写进 R：`research/prompts/m3-batch19-code-r1-opus-model/out/n34-240-0-1-a036.txt:220`「HIST S=240 K=0 U=1 count=4100 arm=3 held>0=true n3=ok inR>0=false inH>0=false wu=1 n4=-」。那份历史里没写进扣住槽，也就没有用户看得到的后果，不算漏红；说明这条用例对「暖机之前放开」的判别力来自 S = 176 时分配器正好取到槽 1410，换一种取落点次序（合法改动）它就不再钉这一刀。推的，没在别的取落点次序上量。

## N4：暖机之后那一句放开的用户看得到的结局

### 变异红在哪

- 原样拷贝上施加「删除批第 19 批 N4」：N4 那条红在用例第 717 行的 `panic!`（`research/prompts/m3-batch19-code-r1-opus-model/out/tsv-N4.txt:19`「thread 'after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor' (2403735) panicked at crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:717:13:」，下一行是 `NoSpaceAfterRaisingTheFloor` 带 `stop: FloorAlreadyAtTheCeiling { floor: CheckpointTxg(5), ceiling: CheckpointTxg(5) }`），没走到第 721 行起那条三元组断言；4 GiB 那条红在第 596 行（同一文件第 25 行）。与用例文档注释说的一致，不算打中。

### 用户决定的几步放开扫

同一装置，N4 那一形再挂用 `mount_writable_with_test_only_switches`（准入关、影子账开），交回之后第一次顺序写把文件写回 U 个单元，记结局（写成时记推了几串抬 F、发了几版、F 动没动）。每份起点卸载写流全 8223 个状态逐个跑（层 0 枚举，每个状态 checker 零违例）。合法臂（0）与 N4 臂（2）：

| 起点 S:K:U | 合法臂，有扣住槽的状态 | N4 臂，有扣住槽、暖机一次的 8207 个状态 | 出处（模型目录 out/ 下） |
|---|---|---|---|
| 176:0:8 | 全写成 (0, 1, F 不动) | 全写成 | n34-176-0-8-a02.txt 第 179、183 行 |
| 176:0:24 | 全写成 | 全写成 | n34-176-0-24-a02.txt 第 179、183 行 |
| 176:0:30 | 全写成 | 全写成（只在暖机两次的 3 个状态拒） | n34-176-0-30-a02.txt 第 183、187 行 |
| 176:0:36（用例那一点） | 全写成 | 全拒 `FloorAlreadyAtTheCeiling` | n34-176-0-36-a02.txt 第 183、187 行 |
| 176:0:44 | 全写成 | 全拒 | n34-176-0-44-a02.txt 第 183、187 行 |
| 176:0:52 | 全写成 | 全拒 | n34-176-0-52-a02.txt 第 183、187 行 |
| 192:0:36 | 全写成 | 全拒 | n34-192-0-36-a02.txt 第 183、187 行 |
| 240:0:36 | 全写成 | 全写成 | n34-240-0-36-a02.txt 第 179、183 行 |
| 176:3:36 | 全写成 | 全拒 | n34-176-3-36-a02.txt 第 183、187 行 |

（176:0:56 顺序写长不到 56 个单元就被拒，`grow:NoSpaceAfterRaisingTheFloor`，草稿跑过、没进 rerun。）

- 合法实现误红：没打中。合法臂写不成的只有每份起点那 4 个状态，例 `research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-0-36-a02.txt:181`「HIST S=176 K=0 U=36 count=4 arm=0 held>0=false n3=ok inR>0=false inH>0=false wu=1 n4=nospace fr=0 stop=FloorAlreadyAtTheCeiling」；这 4 个状态是卸载一个写都没落、或两份系统配置写落成撕裂镜像（持久向量见同一文件 LEGALFAIL 那四行），新 F 没生效、缩下来的槽还在 defer 里，两臂一样，用例的崩溃点（第 1 个写落下）不在其中。只落下盘 1 那份系统配置写的状态（持久向量 `0100…`）合法臂写成，所以两份系统配置写谁先谁后不影响用例。176:3:36 那 4 个状态停在 `FloorRaiseRefusedForSpace`（第 181 行），同样两臂一样。
- 错实现漏红：在用例那一点上没打中。N4 臂的判别力只在 U 足够大、S 足够小时有：U ≤ 24 或 S = 240 时 N4 臂照样写成（漏是因为那几点上不扣也放得下，不是用例的错）；用例取的 36 在会红的区里，下沿在 30 与 36 之间。这是限度不是漏红：哪一次改动让每版元数据少占十几个槽，这条用例就会在 N4 臂上悄悄变绿，而 4 GiB 那条（直接数扣住位）照样红（推的，没在改过的元数据形状上量）。

## N7：C479 只供测试的开关与用例一

### 产品路径读不读得到开关：没打中

- 开关只经 `PoolWriter` 的字段进发布路径，构造时恒是产品那一臂（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:321`「InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,」）；装开关只有 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:392`「pub fn set_inode_number_watermark_source(&mut self, source: InodeNumberWatermarkSource) {」一处。现数：`singlefs-core/src` 里 `PoolWriter::new(` 7 处；crates 里 core、harness 的 src、checker、checker-tier、replay-prefix、reference-filesystem 除 transaction.rs 之外提到 `set_inode_number_watermark_source` 或 `PressedBackForTests` 的 0 处；没有 `PoolWriter {` 结构体字面量在 `new` 之外拷字段。挂载（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4590`「let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());」）、会话（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:246`「let mut writer = PoolWriter::new(parameters, devices);」）、管理员回退（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:6572`「let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());」）都在自己的函数里新建写入口，几个只供测试的挂载入口也不收这个开关，测试没有办法把压回的水位带进挂载、续做与回退。两处读点（`transaction.rs` 第 5889、8010 行）在这些路径上读到的恒是产品那一臂。

### 号、水位、墓碑、回退的组合里「关开关零违例」成不成立：末态没打中，崩溃态没跑

装置（`probe/n7.rs`）：4 GiB 两块内存稀疏盘起池（mkfs、取号、暖机、第一个文件），经会话的产品路径走一串动作：`cN` 建 N 个空 inode、`uX` unlink X（产品批大小）、`w` 覆盖写、`rI` 管理员回退到第 I 步之后那一版、`x` 崩溃恢复抛弃最新那条根（照 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 的造法改到稀疏盘、藏的是最新那次发布重写的单元，再可写挂载落到前一条根）。走完之后在拷贝上关开关经 `publish_new_inodes` 建 1 个，报全部违例；再把开关压到每个 p ∈ [1, 水位 − 1]、建 n ∈ {1, 2, 3} 个，报 I-9.6 与全部违例。17 个组合：c3；c3,u3；c3,u4；c3,u1；c3,r0；c3,u3,r1；c3,x；c3,u4,x；c3,w,x；c3,u3,c1；c3,u3,u4；c3,u2,r1,u2；c3,x,c2；c2,r0,c2；c3,u3,x；c5,u2,u3,r1,x；c3,w,w,x,c1。

- 关开关末态 17/17 零违例（`grep -c '^OFF .*violations=\[\]' out/n7.txt` 与 `grep -c '^OFF ' out/n7.txt` 都是 17），I-9.6 都成立，例 `research/prompts/m3-batch19-code-r1-opus-model/out/n7.txt:205`「OFF c3,x,c2 wm_after=5 i96=holds violations=[]」（崩溃恢复抛弃建 2–4 那次之后再建 2、3：被抛弃时间线发过的号重发，照 I-9.6 2026-09-16 那次收窄不算）、第 187 行「OFF c3,u2,r1,u2 wm_after=6 i96=holds violations=[]」（unlink 2、管理员回退把 2 复活、再 unlink 2：墓碑树里两条对象 2 的墓碑）。
- 关开关那次建 inode 的写流按层 0 枚举每个崩溃状态：没跑。状态数 1048583（段 [20, 2, 1, 2]）或 16777223（段 [24, 2, 1, 2]），一段历史自己就超过约 10⁶，照定义不跑，例 `research/prompts/m3-batch19-code-r1-opus-model/out/n7.txt:3`「OFFENUM c3 skipped closed_form_states=1048583 (一段历史超过约 10^6，不跑) writes=25 segments=[20, 2, 1, 2]」。所以「关开关零违例」我只核了末态，没核那一次发布的崩溃态。
- 开关压回那一臂（附带，归用例二）：压回的号落在删掉的号上且小于树内最大号时，写路径在任何写之前拒（`research/prompts/m3-batch19-code-r1-opus-model/out/n7.txt:28`「ON c3,u3 p=3 n=1 refused InodeTreeWriteRefused」）；能造出状态的点里 I-9.6 不红的区域逐点等于 p + n 大于树内最大号与最大墓碑号这一预测，数在 `research/prompts/m3-batch19-code-r1-opus-model/out/n7-region.txt:7`「REGION 预测对上 159 对不上 6；I-9.6 不红的点 90，其中压回的号不是发过的最大号 42」。对不上的 6 点都在 c3,r0：管理员回退之后所选根那棵树只剩 1，checker 报的「树内最大 key」是 4（`research/prompts/m3-batch19-code-r1-opus-model/out/n7.txt:75`「ON c3,r0 p=1 n=1 wm_after=2 i96=VIOLATED(记账里的 inode 号水位 2 不大于 inode 树内最大 key 4) violations=["I-9.6"]」），即 checker 取的是它走到的各条根里的最大号，比 invariants.md 那一行写的「该树 inode 树内最大 key」宽；这一宽在关开关时没造出误红（c3,r0 关开关零违例，第 74 行）。n = 1 时判不出的点正是「压回的正是发过的最大号」，与测试文件头（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:5`「//! 「水位压回去、但那个号原来的记录已被换掉」那一形（压回的正是发过的最大号：它的记录还在树里被原地换掉，或已 unlink 删完、」）一致；n ≥ 2 时多出 42 个点（例 `research/prompts/m3-batch19-code-r1-opus-model/out/n7.txt:10`「ON c3 p=2 n=3 wm_after=5 i96=holds violations=[]」：2、3、4 三条活记录被原地换成新对象，水位 5，零违例）。用例一只建 1 个、压到 3（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:82`「create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(3));」），在判得出的区里，用例一本身不受影响；交用例二那一问时这片区域要一起算。
- 用例一在冻结副本原样（臂 0）上绿：`research/prompts/m3-batch19-code-r1-opus-model/out/test-n7-case-one.txt:7`「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s」。

## N1：Q-F 判别形（打中）

- 错实现 `q-unlink-mono`：会话层把 unlink 交给意图的时间换成「给的与这个线程上一次会话 unlink 用过的时间里较大的那个」——第 17 批 qfm「会话里时间只进不退」的另一种写法，只对 unlink 只进不退（改在 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:249`「UserChange::Unlink {」那一支，补丁见 `scripts/make_arm_patch.py`）。
- Q-F 那份测试文件 22 条全绿：`research/prompts/m3-batch19-code-r1-opus-model/out/test-qf-q-unlink-mono.txt:28`「test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.48s」。新加的那条判的是先建一个更晚的 inode 再 unlink，建的那一步在 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:702`「count: 1,」，每条用例一次挂载里只 unlink 一次，碰不到这种写法。
- 违反条款的历史（`probe/extra.rs` 的 MODE=qf，进程内会话、4 GiB 稀疏盘）：同一次挂载先 unlink 第一个文件给 T1 = 1788000600，再建一个 inode，再 unlink 它给 T2 = 1788000300。合法：`research/prompts/m3-batch19-code-r1-opus-model/out/qf-legal.txt:1`「QF mut= T1=1788000600 T2=1788000300 inode=2 (nlink, 写入时间秒) 每一版=[(0, 1788000300)] 意图那一版写的是给的 T2=true」；错实现：`research/prompts/m3-batch19-code-r1-opus-model/out/qf-mono.txt:2`「QF mut=q-unlink-mono T1=1788000600 T2=1788000300 inode=2 (nlink, 写入时间秒) 每一版=[(0, 1788000600)] 意图那一版写的是给的 T2=false」。读的是发布交回的那一版内存里的记录，不是从盘上读回（推的：两者相同，因为 Q-F 那几条用例在盘上读的就是这一份写下去的字节）。
- 四句：① 分辨：同一段历史合法写 T2、错实现写 T1；② 看得到：会话手里就是调用方给的 T2；③ 满足「漏红」那一句；违反的是 D8（核心索引结构） 已定项 6 写入时间那一段里 unlink 意图那次取哪个时间的那一句（`.claude/kb/decisions/08-核心索引结构.md` 第 195 行，用户 Q-F 乙；只指路，不抄句）；④ 改法见改法表 G2。

## N6：C2 的 unlink 斜率检查（打中，归属交主 agent 判）

- 错实现 `c2-third`：产品批大小取 max(144, ⌈剩下单元数 ÷ 3⌉)（「一次 unlink 至多三批」的捷径，改在 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/unlink.rs:119`「let data_units_per_batch = batch_size.data_units_per_batch();」，只供测试的批大小不动）。
- N6 那条用例照样绿，四份文件每一次发布的写调用、换下的单元数与产品逐项相同（1、144、145 个单元时 ⌈n ÷ 3⌉ ≤ 144，432 个单元时三批各 144）：`research/prompts/m3-batch19-code-r1-opus-model/out/test-n6-c2-third.txt:11`「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.78s」。用例只在这四点上取样（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:138`「let three_batches = writes_of_every_publish_of_an_unlink_in_product_batches(432);」是最大的一份）。
- 433 个单元的文件经会话 unlink（`probe/extra.rs` 的 MODE=c2）：合法 `research/prompts/m3-batch19-code-r1-opus-model/out/c2-legal.txt:10`「C2RUN mut= units=433 每一次发布换下的数据单元=[0, 144, 144, 144, 1] 最多=144」；错实现 `research/prompts/m3-batch19-code-r1-opus-model/out/c2-third.txt:10`「C2RUN mut=c2-third units=433 每一次发布换下的数据单元=[0, 145, 144, 144] 最多=145」。单元数再大，第一批换下 ⌈n ÷ 3⌉ 个（同一文件 C2STEP 那几行：4320 个单元第一批留 2880、换下 1440）。
- 四句：① 分辨：同一个 433 单元的文件，合法每批 ≤ 144、错实现第一批 145；② 看得到：`next_unlink_step` 手里就有剩下的单元数；③ 满足「漏红」那一句；违反 D8（核心索引结构） 已定项 16 第 2 条「分批」里产品每批的单元数 K（`.claude/kb/decisions/08-核心索引结构.md` 第 465 行；只指路，不抄句）；④ 跑前条款把「C2 斜率那一项在四点上分不出」列为不判、已留欠账——这一刀打的是用例的断言 ①「每一次发布换下的数据单元不多于一批」而不是写调用的斜率，但病根同是「四点取样到 432 为止」，算不算进那一项由主 agent 定。改法见 G3。

## 改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| G1：N3 那条（或另起一条）把卸载崩溃点从「第 1 个写之后」扩到再加「第 15 个写之后」（第一次抬 F 空发布的 journal 记录已落、根没落；这一刻挂载暖机推两次），断言照旧是写行与全部暖机都不写进换下的槽；用户看得到的那一面再加一条 K = 3 那份历史、再挂期间崩、丢盘 0、剩下那块盘整读与卸载之前相同 | 臂 6（第一次暖机之后放开）漏红 | 量过：臂 6 在 n = 15、16 写进换下的槽（`n34-176-0-1-a013456.txt` 第 198、204 行，`n34-176-3-1-a036.txt` 第 150、153 行），合法臂 0 写入（同一文件 arm=0 的 PREFIX 行）；丢盘读回 3075 对 0（`lone-k3-n15-arm6.txt` 第 103 行、`lone-k3-n15-arm0.txt` 第 105 行）。用例本身没改、没在用例里跑过：推的 |
| G1b：断言不只靠 S = 176 正好取到槽 1410，改成断言「挂载进门时扣住的每一槽在暖机落满之前没被任何一次发布取走」（要一个读扣住集的只读口，或在影子账关时数 `free_slots − unblocked_slots`） | 判别力依赖取落点次序（S = 240 时臂 3 不中） | 推的 |
| G2：Q-F 加一形：同一次挂载里 unlink 两次，第二次给的时间早于第一次（第一次 unlink 第一个文件、建一个 inode、再 unlink 它） | `q-unlink-mono` 漏红 | 量过：`qf-legal.txt` 第 1 行与 `qf-mono.txt` 第 2 行（读的是发布交回的那一版，没从盘上读回）。写成用例没跑：推的 |
| G3：N6 多取一点 433 个单元（3 × 144 + 1），断言 ① 照旧 | `c2-third` 漏红 | 量过：`c2-third.txt` 第 10 行最多 145；在 N6 用例里加 433 这一点没跑（推的，用例单跑会多几秒） |
| G4：N4 那条加一句前提断言：不放开时这一版的空闲槽不够写回 U 个单元（或直接引 4 GiB 那条的计数），让判别力的边变成会红的 | N4 判别力的下沿（U ≤ 30、S = 240 漏）悄悄移过 36 | 推的 |

## 没打中的形状

- N3 合法臂误红：三份 U = 1 起点（176:0、176:3、240:0）与九份 U > 1 起点，每份卸载写流层 0 全 8223 个状态，合法臂在有扣住槽的状态上写进 R 与 H 都是 0。
- N3 错实现：臂 1（= tsv 甲）在有扣住槽的状态上全部 core panic；臂 3、4 在 S = 176 上全部写进 H；臂 5 只在暖机两次的 3 个状态 core panic、测试文件全绿（与臂 6 同一个漏洞，只是预演会拦，用户看得到的后果没量）。
- N4：合法臂在 U ∈ {8, 24, 30, 36, 44, 52}、S ∈ {176, 192, 240}、K ∈ {0, 3} 上，凡新 F 生效的状态第一次写都写成、不推抬 F、F 不动；没找到误红。卸载崩溃点放开（每个状态）、两份系统配置写的先后放开（只落下盘 1 那一写的状态写成）都不改用例那一点的结局。
- N7：产品路径 0 处装开关；17 个组合关开关末态零违例；开关压回之后 I-9.6 判不出的点逐点落在 p + n 大于最大号的区里（c3,r0 那 6 点 checker 反而更严）。
- N1：第 17 批 qfm 那一类里「与现行那一版各记录最晚时间取大」「与会话里任何改动的时间取大」两种写法新用例都拦得住（前一种是 tsv 里那一行，后一种推的：新用例先建的 inode 时间晚于 unlink 给的时间）；只对 unlink 只进不退的那一种拦不住。精度（取整到分钟、存 32 位）两形我没另造写法。
- N6：「一次发布换下整个文件」（tsv 那一行）用例拦得住；只在 432 以上才变大的批策略拦不住。

## 这条腿自己的限度

- 全部数是副本上的：core 打了臂开关与错实现补丁（`B19_ARM` 不设、`B19_MUT` 不设时与冻结副本同行为：臂 0 上整份 N3/N4 测试文件 6 条全绿，`out/testfile-arm0.txt` 第 13 行），臂 3–6、两个错实现是我造的，不在 `crates/mutations.tsv` 里。
- 「用户看得到的结局」只在 K = 3、卸载崩在第 15 个写之后这一份历史、两个臂上量了（再挂写流层 0 全 24618 个状态 × 丢盘 0 / 丢盘 1）；K = 0 那份历史卸载之前丢盘 0 就只读挂载不上，判不了。臂 5 的用户后果没量（它在两次暖机的状态上 core panic，挂载报错，不交会话）。
- 取样：卸载崩溃态与再挂崩溃态都按层 0 枚举域全取，没缩。缩的是起点与臂：起点 12 份（U = 1 三份、U > 1 九份，S ∈ {176, 192, 240}，K ∈ {0, 3}），每份只跑与它有关的臂（U = 1：臂 0、3、6，176:0:1 另加 1、4、5；U > 1：臂 0、2）；丢盘读回只跑一份历史一个崩溃点。估时：先跑 176:0:1 一份（六臂 334 秒、单线程；只枚举不再挂 67 秒），把再挂挪到枚举之后按 6 线程分（176:0:1 三臂 46.1 秒，U = 36 两臂 101–135 秒），全部阶段合计约 25 分钟，没超 40 分钟，没再缩。
- 原型跑的层 0 状态数（`closed_form_state_count` 每次开跑前算，单次都不超过 24618）：草稿阶段约 49.7 万（8223 × 17 份卸载写流 + 再挂写流 356949），rerun 一遍 147912，合计约 64.5 万，没到 10⁷。超过 10⁶ 没跑的：N7 那次关开关建 inode 的写流（1048583 与 16777223 两种，17 个组合全部没跑）。
- 内存包装：退出码 251 出过几次（上限超过峰值表实测峰值两倍而没给理由、同一会话已有在跑的没给并行理由），那几次命令一行都没跑，补上 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` / `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 重跑；没有 250、252–254。编原型给 6G、跑原型 3G、线程 6。
- 四次长跑撞了前台 240 秒、被工具自动转成后台（草稿阶段两次试跑、两次丢盘读回），我等它跑完才读；之后改成每阶段 4 分钟以内的切法，rerun 一遍全是前台。试跑那一条我中途停了（`proc.py stop`，只停原型那一个进程，包装随之退出）。

## 没做什么

- 不判 K1、K3（正推腿的格），不替主 agent 采纳，副本上的数不算入库装置上的数。
- 没跑 checker 档、层 0 重型用例、整轮门禁、变异整表；只跑 harness 档的四个测试目标与自己的原型。
- 没在用例里实现 G1–G4，也没在入库装置上跑；G1–G4 都是零轮。
- N7 关开关那次发布的崩溃态没枚举（超过约 10⁶）；N4 的 U 下沿 31–35 没量；N1 精度两形（存 32 位、取整到分钟）没另造写法；用例二那一问不归这一轮，只记了附带的数。
- 草稿目录 `/tmp/claude-1000/m3-batch19-code-r1-opus/` 里的编译目录与仓副本（`copy/`、`target/`、`rerun-W/`）交回前删掉；留下的只有草稿阶段的输出（`out/`、`logs-*.txt`）与脚本，不进仓、报告不引。

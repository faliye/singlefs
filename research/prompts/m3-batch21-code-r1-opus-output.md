# m3-batch21-code-r1 攻方腿（Opus）报告（2026-10-05）

攻击面：正文第三节 K2（重点 N1、N3、N5、N7），另做 K2 末条列的上一轮「没打中（一次抽样）」那几格的第二次抽样。冻结副本 `/tmp/claude-1000/m3-batch21-code-r1/tree/` 只读，攻在它的拷贝上；只跑 harness 档，GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望。下文「副本」都指我草稿目录里的拷贝，副本上的数不算入库装置上的数；我提的改法（G5、G7）只在我的副本上量过、被攻过零轮。

## 复跑

```
# 空目录里从头跑（每步一条前台命令，单步 4 分钟以内；全部约 13 分钟）
SINGLEFS_REPO=/home/fy5090/code/singlefs FROZEN_TREE=/tmp/claude-1000/m3-batch21-code-r1/tree \
  bash research/prompts/m3-batch21-code-r1-opus-model/rerun.sh <空目录> <步骤>
# 步骤次序：prepare；build tree / c2cap32 / w1 / w2；n7-tests；n7-probe；n7-fix；n5-tests；n5-fix；n5-w2-targets；sweep；shapes；badimages；count；
#           n5-probe tree|w1|w2 0,1,2,3；n5-probe tree|w1|w2 4,5,6；compare（摘要与 expected-summary.txt 逐字比）。all 依次跑全部，给主 agent 放后台用。
```

交回前在空目录 `/tmp/claude-1000/m3-batch21-code-r1-opus/rerun2` 里照上面的次序逐步跑通一次（n5-probe 每批三件并行，退出码各写一个文件、件数对上、全 0），末步原样输出：`SUMMARY_MATCHES lines=552`。开发时那一遍（不同目录）与这一遍在 sweep、shapes、n5-probe w2 0,1,2,3、n7-probe 四步上另比过一次，同样逐行相同。

估时与取样：先跑小段估——sweep 前 40 段 1.2 秒（10 线程），按 466 段推 14 秒，实测 32 秒；N5 探针 K = 3 一个值 31 秒，推 K = 0..6 三份变体约 12 分钟。估全量不到 40 分钟，没缩取样。编译经内存包装 6G、跑 4G，线程上限 10（`capped.sh 10`，原型 `M3B21_THREADS=10`）。开工时 `ps` 看到同机另有别的会话的原型在跑（内存包装报「键 /tmp/claude-1000/m3-delete-checker-r2-opus/…」排在前面），没有性能测量在跑；之后每条命令带 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 并行放行，没有等锁。

层 0 崩溃状态：一条都没跑。`count` 步算了五条候选小流的全量状态数，最小的也有 6.8 × 10⁷（每次删除类发布的单元写那一段 26 个写），每段都超过约 10⁶，照原型规则不跑（第七节）。

模型目录 `research/prompts/m3-batch21-code-r1-opus-model/` 每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，共 47 个）：

```
efa0ab431ce99f3fcaff66c9449e3715f31e908416505f070a3bfc79aa563aa8  expected-summary.txt
ce876b8494679115d62925f01bb6ddae641063c60ae78da6be3520ac621f4280  n5summary.py
097ec9780a8c382a4f7fd24cd54d92e0b181f13bb39b6147c3f889fbdc3a97b2  out/badimages.out
27b5d9507170e173fdfa86e2186ed9c760e9e9d70e0ce518d718031f2e450a6c  out/count.out
7b9ddc31e3bed68f5204989d08c8d1c6fa115342d4e87f362d13ad628aceefa7  out/n5-fix-g5-tree.out
f1df1e0e33178529256b63e37954277684124907f138d0c8d2282dea71d4faff  out/n5-fix-g5-w1.out
f197c16b84ca5b32e9f7a6de06c09799b57247421f0eb0939e9f038e1dd4221e  out/n5-fix-g5-w2.out
95a4634c83b4fb65fcd2e66d74cde1d4d074f4788c1126b2f93bc216001d3eb2  out/n5-probe-tree-k0123.out
4e785dd6887d052937e8c8e60492900e0de7848eddfc57327256e7a74ba3869e  out/n5-probe-tree-k456.out
88095403f37cfdd9acea38149667b263ca3c2ec9ec8b1103e3559aa2153a871c  out/n5-probe-w1-k0123.out
c32faf1a22b7f7b9756f4e8a9f368de8293c4105524046f6658259d8b452a2f3  out/n5-probe-w1-k456.out
a485d69ccde8c9ef2f0de56b1d4afdea31c70b075dec8d2885c2dfde39e69f84  out/n5-probe-w2-k0123.out
f9f397cb7e80cac027b5d8bfda524c21895710fb083bbeb164946a271e4b4680  out/n5-probe-w2-k456.out
af7ae76a03bf59cdb17acbc7f34b18c266ca8c259a3d131f88de896fd89bed73  out/n5-tests-tree.out
faf21f77708df8a330d81ca5e0edad2a234eaf5e33ac73b260977769bc54c9f4  out/n5-tests-w1.out
797cc12d2a9ab2d06edbe07353343ebe30687d1f7998b01b4af789f3ff80b810  out/n5-tests-w2.out
1bc18d61f614d704feced36662c6be4ec13d328d7ae121d13166b25b5722b4f8  out/n5-tree-ignored-rollback.out
b7356c023d8eb4f5d2dcf18b0bda01e9958a11ee336de0b35e70b9055660292c  out/n5-w2-ignored.out
d0a2f819cbf87cb5549a5e28cdee6ad5c6d4123640f2ffd70e0c8c1f5ed7e4ad  out/n5-w2-unmount-targets.out
4c89b9e7e574a7a7e56da9ff5839c16bc3f0df320ef9ef1864fb11af38072aba  out/n7-c2cap32-core-lib.out
297a870ec1290f1e62b8b794ce30b5b74cb67e39b687ced64481e85518f1a3e0  out/n7-c2cap32-other-targets.out
4d10a4ee4aaf4d8974d76c6d781740c317d826f40af7c73a86fab90992bdd69d  out/n7-c2cap32-tests.out
46c0e230a13723e93bfec52ca85cbe9d94bb29ec0a957596885b9679f3346c95  out/n7-fix-g7-c2cap32.out
aac7374262b90fa81cf6c9448301e95efebf8df398766fd1431be11220c56ef4  out/n7-fix-g7-tree.out
deb9a145c65ef8f8650177a5c0246b685f8eca405917fa418bec8ad68280fd83  out/n7-probe-c2cap32.out
1693dd829ef503661b77ddb84734e5636ce4bf202bc807c8d122711dd35ea215  out/n7-probe-tree.out
2d916398d4ecbdd065a27eb414301f2f98d5957266c184a467602cfa277b8b1b  out/n7-tree-tests.out
c4891a375d7f607aa0632448e59a970e1a289d672b846cdc3e6fe55df9761a09  out/shapes.out
c72fed6853e131b0eafb844e64875cbb4c9bc0b9ad0b9213aada7c85f3667a6d  out/sweep.out
dda53548e1a31accbc6b1ebe5c05aea19844f83ef03c3209ede89860a602e702  patches/c2cap32.patch
2d993a844c25aff77263af7ec0fff59297ff148f93a07a3db62830b6630801a2  patches/w1.patch
87d734de882705aab3bc6696ffc2444f0dd3b8cd7e9af652ec133e876f7198ad  patches/w2.patch
738794b5d59e1c977e6fdbd8a6878b34d6c1a00300d8378886f9d6c26b29dc07  patches/workspace.patch
d92985345df4963bf3bb570a66272281621b26094965c6aab4ec858a14cb10b1  probes/n7_fix_test.rs
32ffffdddf2ddcfc77e2327d5ecfed422670e596443c61db51065986b001f11b  probes/probe_n5_fix_tail.rs
4ffce21936f784f2a5ffa1b2a495ff91cda2c0e55c9c1db54063d99ccc6ab3f3  probes/probe_n5_tail.rs
de25c59fb13cda351e0f6f5f24eb43bcf88a7df719f262b63ba0328376557567  probes/probe_n7_tail.rs
9392dd08423959c5693c421dc6a1cc27fa03ec669dc9f72cf29c9be3a328e7b8  proto/Cargo.toml
1af7ae940507df1f0661cb57879df1a4e6995ad9ecf6c1c8f580773935ffb5dd  proto/src/badimages.rs
234f9d5edc5bca5cba2afcbd74e766a7cc8ea9f82bd381f5c5c682b9a1f1ea81  proto/src/crash.rs
fc6ae8270148cf49dbe308cd80d48d3355289866776e2af57aa651e986322c69  proto/src/judge.rs
01b4c6f4ae214c689acb8173afcb7cfaa50beacc632cb2f8b10c11da028e5e25  proto/src/main.rs
6be9751a3dfc219a19901a61530a14eca6326ba9633d6c8080be5cfc764337cd  proto/src/pool.rs
c3f6f441a768ef0dc933117401c568e113d9a75b92e622c42dd4a6faa9bca9d3  proto/src/shapes.rs
023279062622c5979a6e3130bf2a5ca44cf2fe8fc3a010cfe09de139d53419d8  proto/src/sweep.rs
13709e81ce8a3446f5295ae7964b835b63e635f8de37364fd460669d90050078  rerun.sh
96ba5e4afc21004d0c263c6e7b6197515b877298185d997e074d8b10ab366753  summarize.py
```

`out/` 是 rerun2 那一遍各步的原始输出（带耗时，摘要 `expected-summary.txt` 去掉了耗时与编译目录）。`patches/` 里是三份错实现与工作区加成员那一份；`probes/` 里是追加到冻结副本测试文件后面的探针与两份改法用例；`proto/` 是原型（只把 `singlefs-checker-tier` 当库调，`crash` 模式在全量状态数超过 10⁶ 时不跑，rerun.sh 只调它的 `count`）。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| N7 每批换下数随规模变的错实现 | **打中** | 「每批取 max(144, ⌈剩下 ÷ 32⌉)」（批数封顶 32）在快档 1、144、145、432、433 与慢档 2304、4608 每一点上都与产品逐项相同，快档、慢档两条用例都绿；4609 个单元第一批换下 145、9216 个单元第一批换下 288 | 一 |
| N7 写调用数随规模变 | 没单独打中（附数） | 上面那份错实现在 9216 个单元上最多一次 79 次写调用（产品 73）；② 的上界从 W 式子算出来是「一个单元的文件 + 748」，罩得住它；产品自己 1、2304、4608、4609、9216 个单元最多一次 31、45、53、55、73 次 | 一 |
| N5 卸载崩点与覆盖写次数放开扫 | **打中** | 「当场放开到 min(F_生效, max(每块盘最新有效根最小者, F_生效 − 1))」（只扣现行那一版换下的槽）让这份测试文件 7 条全绿、另 17 个挂载卸载相关目标全绿；覆盖写 3 次、卸载崩在第 1–16 个写之后再挂、再挂途中再崩丢一块盘，212 个状态剩下那块盘读文件报 `DataUnitChecksumMismatchEverywhere`，合法实现 0 个 | 二 |
| N5 题面那一刀（第一次暖机之后就放开） | 没打中（新用例抓得住） | 那份错实现新用例红；扫 K = 0..6、卸载崩点全扫，它在 K = 2、3、5、6 各 26 个坏态，新用例取的 K = 3、崩点 15 落在会红的区里 | 二 |
| N1 两个方向都配上而违例漏判 | 没打中（按字面） | 交叉指、两条条目指一棵树在头 ID 定义下都是另一种配法，不是漏判；种类码对调（真 extent 树标成 livelist 或未登记的 13）两个方向都配上、I-4.7 报不适用、整池只红 I-9.16，但字面按种类码认 extent 树，那张镜像上 extent 树没有单元，不算漏判；列给主 agent 当线索 | 三 |
| N1 合法删除中间态误报（第 20 批那一格的第二次抽样） | 没打中 | 466 段合法历史、3508 份镜像，判不了、部分没判、违例都是 0；轴与第 20 批那一份不同（删到一半建 inode、挂载续做从意图与第 1 批之后起、多片 inode 叶） | 三 |
| N3 删完之后 CPU 答没有文件而 GPU 不答的别的形 | 没打中 | 读 wgsl 逐支对 `recovery.rs`；同一批 3508 份镜像上按 wgsl 推的五种会拒形状一份都没有（CPU 答 Z 的 2040 份） | 四 |
| 第 20 批 N3 坏镜像换底子 | 没打中 | 改法表原样 38 份里 32 份只红目标；另 3 份 I-9.6 墓碑那一刀不红、3 份改法 panic，都是改法把底子上的号写死（水位 2、叶里恰好两条），镜像本身不违例；换成不依赖号的改法后 7 份全只红目标 | 五 |
| 第 20 批 N5 其余几形 | 没打中 | nlink 偏移两内核同读一份常量 36；同号「后读到的盖掉先读到的」两边同；多片叶 470 个 inode 的历史无分叉 | 五 |
| 第 20 批 N9 | 没做完 | H3 那三份源文件与第 20 批快照逐字节相同；导出 bin 在 checker 档、没起，没逐字节比导出 | 五 |
| 第 19 批 N3、N4 合法实现误红 | 没打中 | 合法实现上 7 条全绿；N5 扫的 24434 个再挂崩溃态上合法实现 0 个坏态 | 五 |
| 第 19 批 N7 产品路径与关开关零违例 | 没打中 | 非测试代码里装开关 0 处、`PoolWriter::new` 缺省产品那一臂；3508 份镜像上 I-9.6 全成立 | 五 |
| 附带：冻结副本上 3 条慢档红 | 观察 | `rollback_by_a_forward_publish` 带 `--ignored` 跑，合法实现上 3 条红（错实现上同样 3 条），不在这一批的文件里 | 六 |

## 一、N7：每批换下数随规模变的错实现让三点照样过

### 错实现（只改产品那一臂的批大小）

`patches/c2cap32.patch`：`next_unlink_step` 里产品批大小从「一片下段叶」改成「max(一片下段叶, ⌈剩下 ÷ 32⌉)」，测试旋钮那一臂不动。冻结副本原来那一句是「let data_units_per_batch = batch_size.data_units_per_batch();」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/unlink.rs:119`），产品那一臂取「UnlinkBatchSize::OneExtentLowerLeaf => EXTENT_TREE_LOWER_LEAF_DATA_UNITS,」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/unlink.rs:95`）。它与变异表第 21 批那一刀（÷ 3）同一族，只是封顶的批数换成 32。

为什么三点照样过：剩下 r ≤ 4608 时 ⌈r ÷ 32⌉ ≤ 144，批大小仍是 144；用例取的规模最大是慢档的「let thirty_two_batches = writes_of_every_publish_of_an_unlink_in_product_batches(4608);」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:269`），快档最大是「let three_batches_and_one_unit = writes_of_every_publish_of_an_unlink_in_product_batches(433);」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:205`）。走到的每一个「剩下」都不超过 4608，每批换下数、写调用逐项与产品相同。

量的（副本 c2cap32，`out/n7-c2cap32-tests.out`，快档、慢档一起带 `--include-ignored` 跑）：

```
test unlink_of_files_of_one_to_four_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_does_not_grow_with_the_file ... ok
test unlink_of_files_of_sixteen_and_thirty_two_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_stays_within_a_constant_of_a_one_unit_file ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.84s
```

core 库 `unlink` 那 7 条、`unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches`、`container_trees_grow_a_level_when_the_root_is_full` 在这份错实现上也全绿（`out/n7-c2cap32-core-lib.out`、`out/n7-c2cap32-other-targets.out`）；仓里引产品批大小的测试只有这几处（`grep -rln 'OneExtentLowerLeaf\|next_unlink_step\|every_unlink_step_from' crates --include=*.rs`，另两处是 checker 档 bin 与 C479 用例，都只删空 inode 或小文件）。

它错在哪（探针 `probes/probe_n7_tail.rs`，追加在冻结副本那份测试文件后面，同一套搭建、规模取 4609 与 9216；整行抄自 `out/n7-probe-c2cap32.out` 与 `out/n7-probe-tree.out`）：

- 错实现：「N7_PROBE size=4609 publishes=33 first_released=[0, 145, 144, 144] clause_first=[0, 144, 144, 144] released_equals_clause=false largest_write_calls=53 largest_written_bytes=836096」（`research/prompts/m3-batch21-code-r1-opus-model/out/n7-probe-c2cap32.out:6`）
- 错实现：「N7_PROBE size=9216 publishes=55 first_released=[0, 288, 279, 271] clause_first=[0, 144, 144, 144] released_equals_clause=false largest_write_calls=79 largest_written_bytes=1262080」（`research/prompts/m3-batch21-code-r1-opus-model/out/n7-probe-c2cap32.out:7`）
- 产品：「N7_PROBE size=9216 publishes=65 first_released=[0, 144, 144, 144] clause_first=[0, 144, 144, 144] released_equals_clause=true largest_write_calls=73 largest_written_bytes=1163776」（`research/prompts/m3-batch21-code-r1-opus-model/out/n7-probe-tree.out:7`）

文件长到 N × 144 个单元时这份错实现每批换下 ⌈N ÷ 32⌉ × 144 上下，单次发布换下的单元随文件长；D8（核心索引结构） 已定项 16 第 2 条（`.claude/kb/decisions/08-核心索引结构.md:465`）定的是每批尾部 K 个、K 产品取一片下段叶。

### 写调用那一侧（附数，不单算打中）

② 的上界从 W 式子现算，探针打出来是「N7_PROBE one_unit_largest=31 bound_add=748」（`research/prompts/m3-batch21-code-r1-opus-model/out/n7-probe-tree.out:5`）：写调用最多那次只要不超过 31 + 748 = 779 都过。上面那份错实现 9216 个单元时最多一次 79 次，产品 73 次，都远在里面；③ 只钉 1、144、145、432、433、2304、4608 这几点的绝对值。所以「写调用随规模变、批数也变」的那一族，只要在这几点上与产品相同，② ③ 都拦不住；我没另造一份只在写调用上变、换下数不变的错实现（推的：③ 在几点上钉绝对值，这一族要在这几点上逐项重合才过，比换下数那一族更做作）。

产品自己（副本 tree，同一探针与用例）最多一次写调用：1 个单元 31、2304 个 45、4608 个 53、4609 个 55、9216 个 73，随规模在长；「斜率为 0」怎么读这一轮不判（正文第一节「这一轮不判的」第二条，进 C2 欠账），这几个数只当那一问的材料。

### 四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 这一轮没有对立臂；它分辨的是「产品批大小」与「批数封顶 N（N ≥ 32）」那一族，用例在取样点上分不开这两者 |
| 被判的系统当时看得到判别它的东西吗 | 用例看不到：判别它的观测是剩下多于 4608 个单元时那一批换下几个，用例取的规模一个都没到；纯算的 `next_unlink_step` 在任意 r 上都看得到（G7） |
| 满足判据字面的哪一个分句 | 正文第五节「漏红：一种违反条款或判决的实现上用例照样绿。」（`research/prompts/_m3-batch21-code-r1-body.md:68`） |
| 跑前条款给的改法在打中的格上还中不中 | 跑前条款只写「打中：主 agent 现查坐实，改法进下一批」，没给具体改法；下面 G7 在这一格上量过会红 |

### 改法 G7（只在我的副本上量过、被攻过零轮）

`probes/n7_fix_test.rs`：不建池，直接比 `next_unlink_step(r, OneExtentLowerLeaf)` 在 r = 145、433、4609、10⁶、2⁴⁰ 上都是「留下 r − 144」。产品路径上那一步就是它：「let step = match next_unlink_step(data_units_remaining, batch_size) {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/transaction.rs:6045`）。

| 改法 | 修哪一格 | 副本 tree | 副本 c2cap32 |
|---|---|---|---|
| G7 纯算比 2⁴⁰ 以内几点 | 批数封顶 N 那一族（N < 2⁴⁰ ÷ 144） | 量过：「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s」（`research/prompts/m3-batch21-code-r1-opus-model/out/n7-fix-g7-tree.out:8`） | 量过：红在 4609 那一点，「left: Batch { data_units_kept: 4464, size_after_the_batch_in_bytes: 145678176 }」（`research/prompts/m3-batch21-code-r1-opus-model/out/n7-fix-g7-c2cap32.out:14`） |
| G7 | 批大小不走 `next_unlink_step` 而在发布路径另算的错实现 | 推的：碰不到，那一形还得靠建池的几点 | 没量 |

推翻条件：在入库装置上把这份错实现打上，快档那条或慢档那条用例红——那就是我对取样点的读法错了。

## 二、N5：卸载崩点与覆盖写次数放开扫

### 打中的那一形：只扣「现行那一版换下的」那一截

新用例比的槽是「let released_before_the_unmount = slots_released_by_the_current_version(&pool);」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:697`），只是卸载之前现行那一版换下的。那个函数的文档说「盘 1 上最新的根早它一个 txg、还引用这几个槽，重建分配器时回收、扣到这次挂载的暖机落满每块盘（mount::rebuilt_allocator）。」（同一文件第 606 行，整行见下），而新用例那一形里盘 1 上最新的根早两个 txg：根环三个区域按「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/root_ring.rs:114`）落、区域归属「[DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/make_filesystem.rs:63`），覆盖写 3 次之后现行那一版 txg 6（区域 0、盘 0），txg 5 在区域 2（盘 0），盘 1 上最新的是 txg 4。重建分配器时扣住的是释放代在 (4, 6] 的两截——txg 5 那一版换下的与 txg 6 那一版换下的；用例只看后一截。

`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:606`：「/// 盘 1 上最新的根早它一个 txg、还引用这几个槽，重建分配器时回收、扣到这次挂载的暖机落满每块盘（mount::rebuilt_allocator）。」

错实现 W2（`patches/w2.patch`）：重建分配器当场放开的那一截从「effective_floor.min(newest_valid_root_on_every_device),」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/mount.rs:2038`）改成 `effective_floor.min(newest_valid_root_on_every_device.max(F_生效 − 1))`——当成「另一块盘上最新的根至多早 F_生效 一个 txg」，于是释放代 ≤ F_生效 − 1 的那一截当场经「allocator.reclaim_released_up_to(released_immediately_up_to, ReclaimedReuse::Immediately);」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/mount.rs:2044`）放开，只有现行那一版换下的那一截扣着。这违反 D16（发布语义） 已定项 1 表里「正常卸载」那一行（`.claude/kb/decisions/16-发布语义.md:41`）定的当场可用上限 min(F_生效, 每块盘上最新有效根的 txg 最小者)。

这份测试文件 7 条在 W2 上全绿（`out/n5-tests-w2.out`）：「test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-tests-w2.out:14`）。挂载、卸载相关的另 17 个 harness 目标在 W2 上也全绿（`out/n5-w2-unmount-targets.out`，17 个 `test result: ok`）；其中两个目标带 `--ignored` 跑的那几条，W2 与合法实现红的是同样 3 条（第六节），不是 W2 引起的。

### 放开扫（探针 `probes/probe_n5_tail.rs`）

只固定前缀（不补环窗口、准入关着造的窄盘，单元区 176 槽，与新用例同一个起法）与两次故障（卸载崩、再挂崩），用户决定的覆盖写次数 K 取 0..6，卸载落下 c 个写就崩的 c 取 1 到卸载写数减 1 的每一个；每个 (K, c) 上：① 不崩地再挂一次，数写行与暖机写进「现行那一版换下的槽」几个（新用例那一判）；② 再挂落下 r 个写又崩（r 取 0 到再挂写数减 1 的每一个）、丢盘 0 或盘 1，剩下那块盘只读挂载整读每个活文件：卸载之前那块盘单独读得出、而崩溃之后读不出或读出的不是任何一版发过的内容，记坏态。每份变体 279 个 (K, c)、24434 个再挂崩溃态。

整行抄自 `expected-summary.txt`：

- 合法实现：「n5-probe-tree-k0123 N5SUM variant=tree k=3 current_txg=6 mod3=0 crash_points=35 refused=0 warm_up_twice_c=15..31 remount_crash_states=3238 bad_states=0 crash_points_with_bad=- test_check_nonzero_c=17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35」（`research/prompts/m3-batch21-code-r1-opus-model/expected-summary.txt:76`）
- W2：「n5-probe-w2-k0123 N5SUM variant=w2 k=3 current_txg=6 mod3=0 crash_points=35 refused=0 warm_up_twice_c=15..31 remount_crash_states=3238 bad_states=212 crash_points_with_bad=1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16 test_check_nonzero_c=17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35」（`research/prompts/m3-batch21-code-r1-opus-model/expected-summary.txt:90`）
- W2：「n5-probe-w2-k456 N5SUM variant=w2 k=6 current_txg=9 mod3=0 crash_points=35 refused=0 warm_up_twice_c=15..31 remount_crash_states=3238 bad_states=90 crash_points_with_bad=15,16 test_check_nonzero_c=17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35」（`research/prompts/m3-batch21-code-r1-opus-model/expected-summary.txt:93`）

七个 K 合起来：合法实现 0 个坏态，W2 302 个（只在 K = 3、6，盘 1 上最新的根早现行那一版两个 txg 的那两格），题面那一刀 W1 104 个（`python3` 把 `N5SUM` 行的 `bad_states` 按变体加起来）。坏态的样子（整行抄自 `out/n5-probe-w2-k0123.out`）：

「N5_STATE k=3 c=15 newest_dev0=Some(6) newest_dev1=Some(4) warm_ups=2 remount_writes=55 taken_into_released_by_current=0 remount_crash_states=110 unreadable_before=0 bad_states=43 first_bad=r=10 lost=0 after=Err("读 inode 1：DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(1414) }")」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-probe-w2-k0123.out:150`）

就是新用例那一个点（K = 3、c = 15，暖机推两次）：`taken_into_released_by_current=0`——新用例那一判看到的是 0，照样绿；而写行那次把单元写进 1414、1415（txg 5 那一版换下、盘 1 上 txg 4 的根还引用），再挂写到第 10 个就崩、丢盘 0，盘 1 读文件全盘校验和对不上。

题面那一刀（W1，`patches/w1.patch`，第一次暖机之后就放开、预演同步放开）：新用例红（`out/n5-tests-w1.out` 第 28 行 6 过 1 红）；放开扫里它在 K = 2、5 的 c = 32、33 与 K = 3、6 的 c = 15、16 各有坏态，新用例取的 (3, 15) 在里面——这一刀新用例钉得住，不算打中。

### 四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 这一轮没有对立臂；它分辨「扣住那一截的下沿取每块盘最新有效根」与「只扣现行那一版换下的」，新用例在它取的那一点上分不开这两者 |
| 被判的系统当时看得到判别它的东西吗 | 用例看得到、没去看：txg 5 那一版换下的槽在覆盖写第 2 次之后的 `current_file_version().released` 里，用例只取第 3 次之后那一份 |
| 满足判据字面的哪一个分句 | 正文第五节「漏红：一种违反条款或判决的实现上用例照样绿。」（`research/prompts/_m3-batch21-code-r1-body.md:68`） |
| 跑前条款给的改法在打中的格上还中不中 | 跑前条款没给具体改法；下面 G5 在这一格上量过会红 |

### 改法 G5（只在我的副本上量过、被攻过零轮）

`probes/probe_n5_fix_tail.rs` 的 `fix_g5_…`：与新用例同一形（K = 3、c = 15），比的槽从「现行那一版换下的」放宽成「盘 1 上（单独恢复择到的）最新的根之后每一版换下的」（这一形里是 txg 5、6 两版各 12 个槽）；断言写行、两次暖机都不写进它们。

| 改法 | 修哪一格 | 副本 tree | 副本 W1 | 副本 W2 |
|---|---|---|---|---|
| G5 | 只扣现行那一版（W2）与第一次暖机之后就放开（W1） | 量过：「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-fix-g5-tree.out:9`） | 量过：「left: [[], [], [1414, 1415, 1416, 1417]]」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-fix-g5-w1.out:10`） | 量过：「left: [[1414, 1415], [], []]」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-fix-g5-w2.out:10`） |
| G5 | 扣住那一截的下沿取错到「盘 1 最新的根」以下（多扣） | 推的：碰不到，多扣不写坏数据，只会让准入或落点更紧 | — | — |
| 另一条路（推的，没实现）：新用例改成比行为——再挂途中逐写崩、丢盘 0 读文件 | W1、W2 与别的扣法错 | 推的：上表探针就是这一判，合法实现 0 坏态 | 推的：会红（探针 26 个坏态） | 推的：会红（探针 212 个坏态） |

那份函数文档里「盘 1 上最新的根早它一个 txg」只在 K = 0、2、5 时对（探针 c = 1 那一行：盘 1 上最新的根分别是 txg 2、4、7，现行那一版 3、5、8）；K = 1、4 时盘 1 上就是现行那一版；K = 3、6 时盘 1 上最新的根早两个 txg（txg 4、7，现行那一版 6、9）——新用例取的 K = 3 正在这一格。文档改准进 G5 一起。

推翻条件：在入库装置上把 W2 打上，这份测试文件或另 17 个目标里有一条红；或者在合法实现上用同一份探针扫出坏态——那就是我的再挂崩溃判据错了。

## 三、N1：配对改了之后

### 3.1 两个方向都配上、违例照旧漏判：按字面没打中，种类码对调那一形列给主 agent

原型 `shapes`（`proto/src/shapes.rs`）：底子与第 21 批 S1 用例同（新池新建文件之后建一个空 inode），先拿掉 1 号记录、留着单元（I-4.7 那一形坏镜像），再改树表。树表里 inode 树、`deleted_inodes`、extent 树各一条，配对要两个方向都恰好一条；新加的那一遍只看种类是 extent 的条目：「if inode_trees_named_by_the_extent_tree != 1 {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker/src/walk.rs:8488`）。整行抄自 `expected-summary.txt`（长行只抄到判定种类为止的那一段在 `out/shapes.out` 里，整行见那个文件）：

| 形 | 改法 | 整池违例 | I-4.7 / I-4.6 |
|---|---|---|---|
| drop-record | 只拿掉记录 | `["I-4.7"]` | V / N |
| drop-record+S1 | 第 21 批那一形（空根 livelist 冒充 extent、真 extent 头 ID 0） | `["I-9.16", "I-9.27"]` | G / G（新判法记判不了） |
| drop-record+swap-extent-and-livelist-kinds | 真 extent 树那一条种类改 6、头 ID 改 0；空根 livelist 那一条种类改 1、头 ID 指 inode 树。树 ID、根指针不动 | `["I-9.16"]` | N / N |
| drop-record+extent-as-unregistered-13 | 同上，真 extent 树那一条种类改成未登记的 13 | `["I-9.16"]` | N / N |

后两形里 deleted_inodes ↔ inode 树、inode 树 ↔ extent 树（种类 1 那一条）两个方向都恰好一条，新判法不记判不了，I-4.7 报「走读到的每一版里都没有处于删除中的 inode」，而 1 号的单元还在那棵被标成 livelist（或 13）的树里、inode 树里没有它的记录、号也不在队列。

为什么不算打中：I-4.7 对象的字面是「extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号」（`.claude/kb/invariants.md:183`），哪一棵是 extent 树按树表种类码认；那张镜像上种类 1 的那棵树是空根，字面上没有对象，报不适用不满足正文「报不适用而镜像上有它的对象」那一分句。整池仍红 I-9.16，与 `checker_known_bad_images.rs` 里登记的「互换两条的种类」那份同形（`invariants.md` I-9.16 那一行：「那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）」（`.claude/kb/invariants.md:303`））。要不要让删除态扫描在「种类码对不上树里装的东西」时记判不了，是条款问题，列给主 agent 当线索，不当打中。

交叉指（两棵 inode 树各配一棵 extent 树、头 ID 互换）与「一棵树被两条条目指着」（两条 extent 条目同头 ID、两条 inode 条目同树 ID）：读代码推的，没造镜像——前者在头 ID 的定义下就是另一种配法，每一棵 extent 树的单元按它头 ID 指的那棵 inode 树判，字面上的「inode 树」就是那一棵，不算漏判；后者落进「不是恰好一条」，三条（或两条）记判不了。这两形都要树表里同一种类两条，I-9.16 先红；I-9.27 对多条种类 2 放宽成「等于其中某一条」（`invariants.md` I-9.27 那一行），收严留给多头那一批。

### 3.2 合法删除中间态误报判不了：没打中（第 20 批那一格的第二次抽样）

原型 `sweep`（`proto/src/sweep.rs` 的 `histories`）只固定前缀（mkfs → 取号 → 暖机 → 新池新建文件 3000 字节）、没有故障，用户决定的几步放开扫，轴与第 20 批攻方那一份不同：

| 轴 | 取值 |
|---|---|
| 文件单元数 | 1、3、145（145 个单元下段两片叶） |
| 删之前建的空 inode | 0、1、3；另 240、470 个（多片 inode 叶） |
| unlink 对象与次序 | 只删 1 号；先删最后建的、再删 1 号；先删 1 号、再删最后建的 |
| 批大小 | 1、2、144（145 个单元取 100、144） |
| 删 1 号进行到一半（意图之后、第 1 批之后） | 不做；建一个 inode；不卸载重挂续做（`mount_writable_with_test_only_switches_and_deletion_switches`，批大小同上，挂载里续做到空） |
| 删完之后 | 不做；再建 2 个、删号最大的那个 |

每次发布、挂载之后的整池镜像跑池级 checker。判定种类 H 成立、N 不适用（一次都没判到）、G 不适用而理由是某一版判不了、P 部分没判、V 违例（`proto/src/judge.rs` 的 `kind`）。整行抄自 `expected-summary.txt`：

「sweep SWEEP_TOTALS histories=466 {"I-3.4H": 3508, "I-4.6H": 2342, "I-4.6N": 1166, "I-4.7H": 2342, "I-4.7N": 1166, "I-9.5H": 3508, "I-9.6H": 3508, "cpuFF": 1468, "cpuZZ": 2040, "images": 3508}」（`research/prompts/m3-batch21-code-r1-opus-model/expected-summary.txt:551`）

「sweep SWEEP_FLAGGED_HISTORIES 0」（`research/prompts/m3-batch21-code-r1-opus-model/expected-summary.txt:552`）

466 段全走得到（被拒的一段都没有），3508 份镜像上 G、P、V 与删除线之外的违例一个都没有。新加的 extent 反方向那一遍只读树表条目，合法发布写出的树表里 extent 树恰好一条、头 ID 指那棵 inode 树，扫到的全部树表都过了这一遍。

崩溃状态没扫：`count` 步算的五条候选小流全量状态数 6.8 × 10⁷ – 9.3 × 10⁷（`expected-summary.txt` 第 50–54 行），每次删除类发布单元写那一段 26 个写，单段就超过约 10⁶，照原型规则不跑。

## 四、N3：删完之后 CPU 答「没有文件」而 GPU 不答的别的形——没打中

读冻结副本的 wgsl，逐支对 CPU 冷走读（GPU 不跑）：

| 一支 | GPU（`crash_judge_common.wgsl`） | CPU（`recovery.rs`） | 删完之后的那一版 |
|---|---|---|---|
| extent 上段根零条目 | 「if (n.entry_count == 0u && !(is_the_root_of_the_tree && expected_level == 0u)) {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2328`） | 「if files.is_empty() {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/recovery.rs:4182`） | 根兼叶层级 0，两边都收 |
| 上段一条叶条目都没有 | 「if (upper_count == 0u) {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2688`） | 同上那一支，`data_pointers` 空 | 两边数据单元数都是 0 |
| 树表条目根指针全零 | 「if (pointer.all_zero) {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3061`） | 「if entry.root == NodePointer::empty_root() {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/recovery.rs:4868`） | `deleted_inodes` 删空后根指针置零，两边都跳过、映射条目数里都不算它 |
| 映射条目数里的删除树 | 「let deletion_tree_nodes = select(0u, 1u, have_deleted_inodes) + container_tree_units;」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3193`） | 「let deletion_tree_nodes = usize::from(by_kind.contains_key(&TREE_KIND_DELETED_INODES))」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/recovery.rs:4986`） | 同一个式子 |
| inode 树根 | 「if (inode_root.level != 1u) {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3218`），零条目照收 | 根恒层级 1、零条目照收 | 只剩空 inode 或一条都不剩时都是层级 1 |
| 没找到 1 号 | 「if (!found_inode) {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3282`）→ 没有文件 | 冷走读交回 `NoFile` | 同答 |

在同一批合法历史上按这几支推 GPU（`proto/src/judge.rs` 的 `gpu_shape_divergence`：CPU 两遍恢复答 Z 或 F 的每一份镜像，读最新那条根的树表，查 extent 条目根指针全零、extent 上段根零条目而层级不是 0、上段叶条目多于一条或不是 1 号、inode 树根层级不是 1 这五种 GPU 会走读失败的形状）：3508 份镜像（2040 份两遍都答 Z）上一份都没有（`SWEEP_FLAGGED_HISTORIES 0` 已含这一项）。删完之后还能做的只有建空 inode、删它们（D8（核心索引结构） 已定项 16 第 5 条与它的射程里 K-U5 那一句，`.claude/kb/decisions/08-核心索引结构.md` 那一节），空 inode 不进 extent 树，上段始终只有零条目根。

推翻条件：GPU 对拍「删到最后一批」那条流（提交时由主 agent 跑）在最后一批之后的某一版上 GPU 结局不是没有文件，或建空 inode 的发布往 extent 上段写了条目。

## 五、上一轮「没打中（一次抽样）」那几格的第二次抽样

### 5.1 第 20 批 N1 合法态误报

见 3.2：没打中。与第 20 批那一份共用的只有「每次发布之后整池镜像跑池级 checker」这一个判法；历史生成、取样轴、池的搭建代码都是这一份原型自己写的。

### 5.2 第 20 批 N3：坏镜像换造法，是不是仍只红目标

原型 `badimages`：`common_deletion_in_progress` 那四张改法表（意图之后、第一批之后、建一个空 inode 之后、记账标签 4）原样，底子换成四段别的合法历史上的同一类版本——3 个单元不建 inode（批 1）、145 个单元建 3 个（批 100）、3 个单元建 470 个（多片叶，批 1）、1 个单元建 1 个（批 144）。改法表原样的 38 份里 32 份只红目标；另 6 份：

- I-9.6 墓碑那一刀在建过 inode 的三份底子上一条不红，例：「badimages BADIMAGE base=u145-n3-b100 version=intent mutation=2 target=I-9.6 only_target=false violated=[]」（`research/prompts/m3-batch21-code-r1-opus-model/expected-summary.txt:12`）。改法把墓碑对象 ID 写成常数 2（`INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE`），这几份底子的水位是 5、472、3，镜像本身不违反 I-9.6，不红是对的。
- 拿掉 1 号记录那一刀在叶里多于两条的底子上 panic（它断言叶里恰好 1、2 号两条），第一批那一刀在 1 个单元的底子上 panic（第一批就是最后一批，记录已删）：改法的前提不成立，不是判定。

换成不依赖底子上号的改法（攻方自己写的 `raise_the_first_tombstone_far_past_the_watermark`、`drop_the_first_record_of_the_first_leaf`）之后 7 份全只红目标（`expected-summary.txt` 里 `_generic` 那 7 行；连同上面 32 份，`only_target=true` 共 39 行，`BADIMAGE` 共 45 行）。没打中。

### 5.3 第 20 批 N5 其余几形（同号多条、多片 inode 叶、nlink 偏移）

- nlink 偏移：第 21 批把两份常量合成一份，共用那一半「const INODE_RECORD_NLINK_OFFSET: u32 = 36u;」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:136`），扫描内核读同一份：「if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + INODE_RECORD_NLINK_OFFSET),」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2198`）；池级 checker「const INODE_RECORD_NLINK_OFFSET: usize = 36;」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker/src/walk.rs:171`）。共用那一半拼在三个内核前面（`crash_judge_gpu.rs` 第 185、194、203 行的拼法）。
- 判的位置：GPU「if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3287`）在找到 1 号之后、核单元数之前，与 CPU 同序。
- 同号多条：GPU 在每片叶里逐条遇到 1 号就盖掉前一条，CPU 同样后读到的赢；这一批没改那一段。
- 多片叶：sweep 里 240、470 个空 inode 的 18 段历史（3 个单元、删到一半建 inode、挂载续做）在上面五种 GPU 形状上一份都没中。
没打中。

### 5.4 第 20 批 N9（H3 形状与导出逐字节）

没做完：`scenario.rs`、`deletion_write_dump.rs`、`deletion_scenario_writes.rs` 三份在冻结副本里与第 20 批快照 `research/prompts/m3-batch20-code-r1-snapshot/crates-sha256.txt` 第 4、9、11 行的 sha256 逐字节相同；`deletion_scenario_writes` 那 5 条在副本 tree 上全绿。导出本身由 checker 档 bin 出，我没起，没与入库的 E172 第 2 次逐字节比；第 21 批动到的 `transaction.rs` 只有 N9 的 panic 一支与注释，产品那一臂交回的值没变（推的，没导出核）。

### 5.5 第 19 批 N3、N4 合法实现误红

合法实现（副本 tree）上这份测试文件 7 条全绿：「test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-tests-tree.out:14`）。第二节那张放开扫在合法实现上 K = 0..6、279 个卸载崩点、24434 个再挂崩溃态，坏态 0。新用例的两句断言（暖机推两次、三次发布都不写进现行那一版换下的槽）在合法实现上只在它取的那一点成立：c ≥ 17 时合法实现照样写进那几个槽（`test_check_nonzero_c=17,…`，那时带新 F 的根已落到盘 1，扣住那一截本该放开），用例把崩点写死在 15，不误红。没打中。

### 5.6 第 19 批 N7：产品路径读到开关、关开关零违例

- 产品路径：`grep` 全部 `crates/` 非测试代码，装开关的 `set_inode_number_watermark_source` 只在定义处，调用方只在两份 harness 用例；`PoolWriter::new` 缺省「InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,」（`/tmp/claude-1000/m3-batch21-code-r1/tree/crates/singlefs-core/src/transaction.rs:340`）。
- 关开关零违例：3.2 那 3508 份镜像（建 inode、删除、删到一半建、挂载续做、删完再建再删）I-9.6 全部成立（`I-9.6H: 3508`）。
没打中。

## 六、附带观察（不在这一批的文件里，不算打中）

`rollback_by_a_forward_publish` 带 `--ignored` 跑（慢档），冻结副本原样（副本 tree）上 3 条红，W2 上红的是同样 3 条：

- 「test plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced ... FAILED」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-tree-ignored-rollback.out:16`）
- 「test the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash ... FAILED」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-tree-ignored-rollback.out:24`）
- 「test the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash ... FAILED」（`research/prompts/m3-batch21-code-r1-opus-model/out/n5-tree-ignored-rollback.out:26`）

第一条的 panic 是恢复之后重开、池级 checker 报 I-3.1（记账的已分配与遍历全部有效根得到的对不上，`out/n5-tree-ignored-rollback.out` 里那一段）。release 档、线程上限 10 下跑的；另两条的 panic 同样是重开之后池级 checker 报 I-3.1（`out/n5-tree-ignored-rollback.out` 第 39、44 行）。这份测试文件不在第 21 批被判名单里；合入验证脚本里没有 `--ignored`（`grep -n ignored scripts/gate/merge-and-verify.sh` 0 行），这三条合入时不跑，提交时的整轮门禁跑不跑、上一次绿在哪一天，我没查。交主 agent 去处，我没追。

## 七、没打中的形状

| 攻击面 | 试过的形状 | 取样范围 |
|---|---|---|
| N1 两向配上而漏判 | 第 21 批 S1 那一形（记判不了，照预期）；真 extent 树种类改 6（与空根 livelist 对调）；真 extent 树种类改未登记的 13；对调形不拿掉记录的干净底子 | 一份底子（建一个空 inode）、四种树表改法；后两种按字面不算漏判（3.1） |
| N1 交叉指、一树两条 | 读代码推，没造镜像 | — |
| N1 合法态误报 | 3.2 那张轴表 | 466 段、3508 份镜像 |
| N3 删完之后 GPU 分叉 | 读 wgsl 逐支；按 wgsl 推的五种形状在合法镜像上查 | 同 3508 份；2040 份 CPU 两遍都答 Z |
| N5 其他错实现 | W1（题面那一刀）在 K = 0..6、全部卸载崩点上 | 新用例抓得住 |
| N6 | 没攻 | — |
| N7 写调用那一族 | 没另造 | 只量了 4609、9216 两点 |
| 层 0 崩溃状态 | 五条候选小流只算了全量状态数，一个都没跑 | 6.8 × 10⁷ – 9.3 × 10⁷，单段 2²⁶ |

## 这条腿自己的限度

- N5 的「坏态」判据是我自己定的：卸载之前那块盘单独读得出、两次崩溃之后同一块盘读不出或读出的不是任何一版发过的内容。合法实现上 0 坏态是它的反向自证（三份变体同一个判据：合法 0、W1 104、W2 302），没有第二条独立判据（例如按盘上最新根走树比引用集合）。
- 再挂途中的崩溃只取「前 r 个写落下」的前缀，不是层 0 的整写子集；卸载那一次同样只取前缀（与这份测试文件自己的造法一致）。
- W2 是我按「另一块盘至多早一个 txg」这个想当然写出来的错实现，它在别的入口（准入那一串、卸载那一串自己的当场放开）上的样子没管；我只说明新用例与这份文件、另 17 个目标拦不住它。
- N7 的错实现与 G7 都只在副本上量；G7 碰不到「在发布路径另算批大小」的错实现（推的）。
- sweep 的 GPU 形状判据是我读 wgsl 摘出来的五种，wgsl 里别的拒法（映射条目数、分配记录几何等）没逐一用 CPU 推；那几支删完前后都不变（推的）。
- 第二次抽样各格都是同一个模型同一份提示里的一次抽样；我读过第 20 批攻方报告的结论与轴表、第 19 与第 20 批判决，取样轴有意避开了第 20 批那一份，但不能说与它独立。

## 没做什么

- 没跑任何 checker 档的测试目标与 bin（GPU 对拍、`deletion_write_dump` 导出、层 0 用例），没跑重型测试；GPU 那一侧只读代码。
- 层 0 崩溃状态一个都没跑（单段超过约 10⁶，第七节）；这一条腿与接手的腿原型跑的全量合计 0 个状态。
- N6（同一次挂载两次 unlink）没攻；N2、N8、N9、N10 与 K1、K3 不归这条腿。
- 没改任何仓文件；副本上的数不算入库装置上的数，要引的由主 agent 在入库装置上重做。
- 没判 G5、G7 该不该采；它们算零轮。

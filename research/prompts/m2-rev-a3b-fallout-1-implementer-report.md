# A3b 善后一实现员报告：单元区起点随环长之后，8 份单外测试跟上

写于 2026-09-27。规格 `/tmp/claude-1000/impl-a3b-fallout-1/spec.md`。在副本 `work/`（主工作区现状 + A3b 补丁）里改，交 `patch/`。
收到主 agent 消息（证红第二批做完之后）：A3b 与 C577 已进主工作区。补丁已对主工作区 `git apply --check` 过；另拷一份主工作区现状 `check/`，打上本补丁跑了 8 份（第六节）。

## 一、结论

- 8 份里 6 份改了，2 份跑了、绿、没改（第 7、8 份）。
- 在主工作区现状（含 A3b、C577）加本补丁上，8 个测试二进制 7 个全绿。剩下 1 条红：`second_transaction_admission_raises_the_floor_before_refusing.rs` 的 `root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise`。它与几何无关，是 A3b 第 4 条和 D16「根槽这一次读坏」那一行（用户定案）冲突，属于设计问题，没改，停下交主 agent（第五节 P1）。
- `core_review_geometry_back_chain_and_empty_inode.rs` 的 `a_journal_ring_shorter_than_one_record_is_refused_before_any_write`，在不带 A3b 的主工作区副本（`main-only/`）上本来就红，与 A3b 无关：代码审阅第 38 条那一判加上之后，「一条记录长的环放行」就不成立了。这次一并改成它今天该钉的那一形。
- 变异表：追加 12 行（10 行新的，2 行是改名重加），替换 1 行，删 3 行（2 行是改名删旧名）。证红 20 条，全部抓到：新行 10 条，另 10 条是点着改过的用例的已有行（第四节）。
- 按主 agent 的定（Q6：不为测试开口子），段边界外的小环一条都没留，全换成 1 MiB 整数倍的环。一次发布记录条数上限的用例改在 1 MiB 环的在飞上限 85 上测：85 条放行，86 条拒；写行那一格是 85 片实例表加上分配记录树的节点。没有「内存或时长撑不住」的格。

**推翻条件**：
- 在 `check/` 上施加第四节任一行变异，它点名的测试不红。
- 1 GiB 那一格（159 MiB 环）量到的多扣块数不是 132–134，或者跨叶那两次不在 txg 18–19。
- 384 槽那一格 80 次覆盖写里，第 78 次之前段外不是 0 对。
- 256 槽那一格先覆盖写 34 次之后，写回不是被拒 3 次、第 4 次做成。

## 二、这一轮写过的文件

补丁 `patch/crates.patch` 动的 6 份（都在 `crates/singlefs-harness/tests/`）：

- `core_review_geometry_back_chain_and_empty_inode.rs`：第 15 条那一节 4 条用例重写（3 条改名），模块文档跟着改；观察结构多带 mkfs 之后的镜像。
- `a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`：读次数 `(5, 3)` → `(4, 2)`，两处文档注释跟着改。
- `core_review_unit_area_start_and_publish_limits.rs`：第 26 条那一节 3 条用例从 6 条、3 条记录长的环换成 1 MiB 环（名字没变）。
- `core_review_tree_table_duplicates_and_slot_one_search.rs`：「恰好三条放行」那一格改成「恰好三条被段边界那一判拒、1 MiB 环做成」（名字没变）。
- `second_transaction_admission_raises_the_floor_before_refusing.rs`：4 条用例的前提或钉值跟着改（名字没变）；`root_slot_already_unreadable…` 没动。
- `admission_checkpoint_cost_per_device_paths.rs`：1 GiB 那一格的环 128 → 159 MiB，钉值 36–38 → 132–134；探针历史那一格按新起点算盘宽，枚举成员改名 `SlotsFromTheDefaultStart` → `SlotsFromTheUnitAreaStartOfTheSmallDeviceRing`；文档注释跟着改。

`crates/mutations.tsv` 的改动放在补丁目录的三个文件里：
- `mutations-append.tsv`，12 行：
  - 新的 10 行，名字以「A3b 善后一：」开头；
  - 改名重加 2 行：「实审 A4d C545：落得下那一判不判数据单元（会话里第 78 次…）」，「实审 A4d（D16 已定项 1「准入」那一行；C545 准入先拒之后）：…256 槽那一格第 10 次覆盖写被式子拒就原样交回」。
- `mutations-replacements.tsv`，1 行：「实审 A2a 第 15 条：短于一条记录的环不拒」，只把点名的测试换成改名之后的那条。
- `mutations-delete.txt`，3 行：
  - 「实审 A2a 第 15 条：单元区越界改回按环长现算的起点判（实例表写在盘外才撞上块设备越界）」：A3b 之后产品代码本来就按环长现算起点，这一行在槽对齐的环上与原式同值。它的判别力改由新的两行接着（「…只算到实例表…」「…起点按环长向下取整算…」），用例 D 的环也改成了不对齐槽边界的。
  - 上面改名的两行的旧名。旧名里的「第 62 次」「第 9 次」是 A3b 之前的几何；证红日志里是第 78 次、第 10 次（第四节）。

副本 `work/` 相对「主工作区 + A3b」的 `git diff --stat -- crates litmus`（副本里起了一个草稿 git 仓，打完 A3b 提交成基线，只用来出 diff）：

```
 crates/mutations.tsv                               |  17 +-
 ...ewer_root_rereads_once_then_refuses_writable.rs |  14 +-
 .../admission_checkpoint_cost_per_device_paths.rs  |  55 ++++--
 ...e_review_geometry_back_chain_and_empty_inode.rs | 212 ++++++++++++++++-----
 ...ew_tree_table_duplicates_and_slot_one_search.rs |  37 +++-
 ...re_review_unit_area_start_and_publish_limits.rs |  99 +++++++---
 ...n_admission_raises_the_floor_before_refusing.rs |  59 +++---
 7 files changed, 354 insertions(+), 139 deletions(-)
```

补丁目录 `sha256sum`：
```
34924468b451408af843f8aa23d3a9fb2c7abf01e28389b78dfb265284bde038  patch/crates.patch
1ac263f779c9a956fa78f4d486e9f25a349c1989554ab438308b434e83deb61a  patch/mutations-append.tsv
1eb24cf0e025049721a2cda0bd238b90fc85abadbd170ab48b27f043dd094730  patch/mutations-delete.txt
62a5f708f6cc3594d82e894dc0052fffeb55f39122710e67f8d34fe841c4d545  patch/mutations-replacements.tsv
```

## 三、逐份改了什么、新值怎么算

基线（主工作区 + A3b，`work/` 上跑，改之前）：第 1 份编不过（引了删掉的成员，只红在这一份，`cargo build --all-targets --keep-going` 核过）；第 2 份红 1 条，第 3 份红 3 条，第 4 份红 1 条，第 5 份红 5 条，第 6 份红 1 条；第 7、8 份绿。

**第 1 份 `core_review_geometry_back_chain_and_empty_inode.rs`**（环末尾的下一个槽，用例一侧自己算：`1024 + ⌈环长 ÷ 16384⌉`）
- A `a_journal_ring_reaching_past_784_mebibytes_is_made_with_the_instance_table_at_the_slot_after_the_ring_instead_of_inside_it`，由原 `…reaching_past_the_compiled_unit_area_start_is_refused…` 改来：
  - 1 GiB 环，起点 1024 + 65536 = 66560（= 1040 × 64）；
  - 实例表写在 66560、树表第 0 版在 66562，两块盘读回来的字节等于 `instance_table_unit`；
  - 784 MiB 那处在环里，读回全 0：mkfs 清环时连花样一起清掉了。
  - 原用例守的是「越过编译期起点的环不许盖住实例表」。A3b 把起点改成现算，删掉了那一判，这条性质现在由起点现算那一处保证，由这条用例钉着。
- B `a_journal_ring_one_record_longer_than_the_default_ends_off_the_cluster_segment_boundary_and_is_refused_and_the_default_ring_is_not`：
  - 默认环加 4096 字节，下一个槽是 49152 + 1024 + 1 = 50177，拒成 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported { 768 MiB + 4096, 50177 }`，录制流 0 步、两盘逐字节不变；
  - 默认环做成，起点 50176。
- C `a_journal_ring_shorter_than_one_record_is_refused_before_any_write_and_one_record_long_passes_on_to_the_safety_factor_check`：
  - 半条记录照旧拒成 `JournalRingShorterThanOneRecord`；
  - 一条记录长的环：在飞上限 4096 ÷ 4096 ÷ 3 = 0，拒成 `JournalRingHoldsFewerRecordsThanTheSafetyFactor { 4096, 12288 }`，在任何写之前。
  - 这条在不带 A3b 的主工作区上本来就红（`main-only/` 上 `test result: FAILED. 13 passed; 1 failed`）。
- D `make_filesystem_units_ending_past_the_device_are_refused_before_any_write_and_a_sixteen_mebibyte_ring_on_a_hundred_mebibyte_device_is_made`：
  - 环 5 MiB − 4096，末尾落在槽 1343 里，起点向上取整到 1344 = 21 × 64。
  - 盘 1346 槽，树表那一槽（1346）在盘外，拒成 `UnitAreaBeyondDevice { 1344 × 16384, 1346 × 16384 }`，录制流 0 步。
  - 盘 1347 槽，三个单元正好放得下，做成。环 ≤ 盘 ÷ 4：5 MiB − 4 KiB ≤ 1346 × 16 KiB ÷ 4 ≈ 5.26 MiB。
  - 100 MiB 盘、16 MiB 环：起点 1024 + 1024 = 2048，做成（A3b 报告第四节说的那一形）。
  - 环长不取槽宽的整数倍，是为了让「起点向下取整」的变异也会红（第四节 M6）。

**第 2 份 `a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:623`**：实例表那一片的读次数 `(N + 3, 3)` → `(N + 2, 2)`，N = 2（重建分配器之前那两读）。算法：
- A3b 之后 `rebuilt_allocator` 先经 `instance_table_of_the_newest_root_read_at_most_twice` 读最新根的实例表（`check/` 上 `crates/singlefs-core/src/mount.rs:1301`），生效 F 用同一张表算（`:1306`），不再自己读一遍。
- 盘 0 那一份从第 3 读起坏：影子账第一遍是第 3 读，重读是第 4 读，所以是 N + 2 = 4。
- 盘 1 那一份只在盘 0 读坏时读，第 3、4 读各一次，所以是 2。
- 与 A3b 报告实测的 `left: (4, 2)` 对得上。已有两行变异（第 22 条那两行）点着这条用例，都不管读次数；新加 M1 管它。

**第 3 份 `core_review_unit_area_start_and_publish_limits.rs`**（1 MiB 环：64 槽，下一个槽 1088 = 17 × 64；256 条记录，在飞上限 256 ÷ 3 = 85）
- 顺序写的切分：从偏移 0 写 L 字节，切成 ⌈L ÷ 32634⌉ 个数据单元、同样多条记录。末条点名的项远少于 67，不再跨记录。
- 多于上限：L = 85 × 32634 + 1，86 条，拒成 `{ 86, 85 }`，两盘逐字节不变、分配器的记录不动。
- 等于上限：L = 84 × 32634 + 1，85 条，照发，`earlier_records + 1 = 85`，txg 4。
- 写行那一格：
  - 行数 85 × 369，写满 85 片（断言 `pages_after_this_publish() == 85`）；
  - 一条记录点名一项时，点名项 = 85 片 + 分配记录树的节点，至少 86 项、至少 86 条，拒成 `{ ≥ 86, 85 }`。
  - 证红日志里这一格实测 90 条（85 片 + 5 个节点，第四节 009）。
- 三条用例在 debug 下整个二进制 6.65 s，没有撑不住的格。

**第 4 份 `core_review_tree_table_duplicates_and_slot_one_search.rs:821`**：
- 恰好三条记录（12288 字节）的环过了在飞上限那一判（上限 1），由段边界那一判拒成 `{ 12288, 1025 }`，录制流 0 步、两盘不变。
- 1 MiB 环做成。
- 1、2 条与差一个字节够三条的环照旧拒成 `JournalRingHoldsFewerRecordsThanTheSafetyFactor`（mkfs 里这一判在段边界那一判之前，`make_filesystem.rs:327` 在 `:363` 之前）。

**第 5 份 `second_transaction_admission_raises_the_floor_before_refusing.rs`**（小盘三档照 A3b 的 `history.rs`：环 6 MiB，单元区从 1408 起）
- `:427` 树表 0 条那一版准入不够：
  - 小盘那一档的 6 MiB 环放不进「起点 + 16 槽」的盘：1424 槽 ÷ 4 ≈ 5.56 MiB。
  - 这一格的环改成 1 MiB，起点 1088（用例里断言），盘 1104 槽。
  - 单元区照旧 16 槽，落在叶 1（[812, 1624)）里，与改之前（落在叶 61 里）一样只罩一片叶，c_max 那一项不变，所以断言不动。
- 384 槽那一格（原 70 次，推过抬 F 的是 18、32、47、62）：
  - A3b 之后 70 次里走不到「段外 0 对」那一判：推过的是 19(37)、34(6)、48(1)、63(5)，最后一项是 5 对。
  - 草稿探查跑到 200 次：段外 0 对第一次出现在第 78 次。
  - 改成 80 次，钉 `[19(37), 34(6), 48(1), 63(5), 78(0)]`，判别力照旧落在最后一项。
- 256 槽截断写回那一格（原先先覆盖写 20 次，钉 6 个单元、被拒 3 次）：
  - A3b 之后 20 次那一格写回一次都不被拒（3 个单元、第 1 次就做成），界 3 那一格走不到。
  - 先覆盖写 0–45 次各跑一遍（草稿探查），写回被拒 3 次的是 11、34、43 次；34 次那一格长到 6 个单元、被拒 3 次、第 4 次做成，与原钉值逐项相同。
  - 改成 34 次，其余断言一个字没动。
- 240 槽「推满仍不够」那一格：第 10 次挂载取号之前短 36 → 30 槽，推两串之后仍短 34 → 28 槽。F 的上限 20、23，各推 3 次，停止原因都不变。
- 这几处新值怎么来的：
  - 都是历史跑出来的，靠手算推不出，是实测。
  - 变的原因是推的：分配记录树按绝对槽号罩叶（一片 812 槽）。
    - 384 槽的单元区原来是 [50176, 50560)，跨叶 61 的后 168 槽与叶 62 的前 216 槽；现在是 [1408, 1792)，跨叶 1 的后 216 槽与叶 2 的前 168 槽。
    - 240 槽原来是 168 / 72，现在是 216 / 24。
    - 每次发布重写几片叶跟着变，defer 与准入扣的量也跟着变。
  - 这一推断没有单独量过。
- 旁记（不归这一件）：256、240 两档上，先覆盖写几次的好几格里，截断本身（覆盖写成 0 字节）就在上限处被空间拒（`NoSpaceAfterRaisingTheFloor`）；240 档还有「写回被拒之后建 inode 也被拒」的格（草稿日志 `scratch-explore-3.log`）。这些格与 `.claude/rules/fs-design.md`「记账是事务的副产品」一节里「释放空间这个操作本身不需要申请空间」那句有没有冲突，没判，交主 agent。

**第 6 份 `admission_checkpoint_cost_per_device_paths.rs`**
- `:59` 1 GiB 那一格：
  - 128 MiB 环的起点变成 9216，离叶 11 的末槽 528 槽，60 次空发布走不到跨叶，「至少一次跨叶」那条断言红。
  - 环改成 159 MiB：起点 1024 + 159 × 64 = 11200 = 50176 − 3 × 12992。12992 = lcm(812, 64)，所以起点在叶内、在段内的位置与 50176 相同：离叶 13 的末槽 11367 是 168 槽，同默认环离叶 61 末槽 50343 的 168 槽。159 MiB ≤ 1 GiB ÷ 4。
  - 钉值按式子算（`admission.rs` 里 `checkpoint_cost_of_the_version_to_build_on` 的文档）。树高 2，K1 = 1 + Σ_盘 叶数 + 中央映射树 1 + 记账树 1 + 树表 1。
    - 单元区 [11200, 65536) 罩叶 13–80，每块盘 68 片，ckpt_cost = 1 + 136 + 3 = 140。
    - 每次空发布每块盘实写 6 块，跨叶那两次 8 块，多扣 134 / 132，钉 `over_reserved(132, 134)`。
    - 改之前 [50176, 65536) 罩叶 61–80，每块盘 20 片，ckpt_cost 44，多扣 38 / 36，与原钉值相符，这个式子对得上。
  - 实测：`checkpoint_cost=140`，`difference_min=-134 difference_max=-132`，跨叶在 `empty#13@txg18`、`empty#14@txg19`，与 A4 量表的 txg 18–19 相同。
- `:861` 探针历史那一格（`SlotsFromTheDefaultStart(2600)`）：
  - A3b 之后盘宽 (50176 + 2600) 槽配 6 MiB 环，单元区其实有 51368 槽，不是「2600 槽、逼满」。改之前这条用例照样绿，是碰巧。
  - 改成按参数的环现算起点：(1408 + 2600) × 16384，单元区 [1408, 4008)，罩叶 1–4。
  - 实测：K1 下 144 次空发布里一块盘改叶多于两片的有 4 次。K0 变异下少扣 4 次，在第 22 步崩了再挂之后的暖机、58、137、154 步；第 22 步是实写 12 / ckpt_cost 8，与 A4c 报的数相同。文档注释照这些改。
- 其余四格（默认环）与压小容量那三格，量到的与钉的逐项相同，没改。

**第 7 份 `second_transaction_allocation_record_tree_geometry_of_writer_and_reader.rs`**：跑了、绿、没改。它只比两侧按盘字节数算的几何，不看起点。第 36 行注释「取小盘那一档的 128 MiB」已经不是事实（小盘那一档现在是 6 MiB），没改，交主 agent。
**第 8 份 `second_transaction_supplement_two_root_ring_turn_in_one_mount.rs`**：跑了、绿、没改。

## 四、证红：改坏哪一行 → 哪条断言红

用 `research/scripts/prove-red.sh --copy work --memory 8G singlefs-harness …`，分两批，线程上限 4。原样末行：第一批 `✓ 点名 12 条：跑了 12 条，跳过 0 条，跑的都抓到了`，第二批 `✓ 点名 8 条：跑了 8 条，跳过 0 条，跑的都抓到了`。日志在 `prove-logs-1/`、`prove-logs-2/`。
基线红集：第 5 份里的 `root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise`（第五节 P1）。所以第 5 份的变异行都带 `--` 过滤到点名的那条，基线绿。其余各份整个二进制跑，基线绿。

| # | 变异（文件：改坏哪一行） | 红的断言 | 同时红的 |
|---|---|---|---|
| M1 新 | `mount.rs`：`rebuilt_allocator` 在读影子账那张表之前先调一次 `effective_rollback_floor` | 第 2 份读次数那条 `left == right`（回到 (5, 3)） | 无（8 条里 1 条红） |
| 912 换 | `make_filesystem.rs:320` `< JOURNAL_RECORD_BYTES` → `< … / 4` | C「半条记录长的环」（报成 SafetyFactor） | 无 |
| M2 新 | `make_filesystem.rs:363` 起点改回默认环的 | A「单元区起点是环末尾的下一个槽」 | B、D |
| M3 新 | 同 M2 | B「默认环长加一条记录：None」 | A、D |
| M4 新 | `make_filesystem.rs:320` `<` → `<=` | C「一条记录长的环过了…被在飞上限那一判拒」（报成 ShorterThanOneRecord） | 无 |
| M5 新 | `make_filesystem.rs` 判越界只算到实例表（`tree_table_genesis_placement_at` → `instance_table_placement_at`） | D「树表那一槽在盘外」（交回 `BlockDevice(OutOfRange { offset: 22052864 … })`，写到一半才撞上） | 无 |
| M6 新 | `make_filesystem.rs:371` 起点按环长向下取整（原 913 的替换文） | D 同一条，同一个 `OutOfRange` | 无 |
| M7 新 | 同 M2 | D（`UnitAreaBeyondDevice { unit_area_start: 822083584 … }`） | A、B |
| 1061 已有 | `transaction.rs` 上限 + 1 | 第 3 份「多于上限」的 `journal_record_limit… == 85` | 写行那一格（`{ 90, 86 }` 对不上 `record_limit: 85`） |
| 1062 已有 | `transaction.rs` `>` → `>=` | 第 3 份「等于上限照发」（`{ 85, 85 }`） | 无 |
| 1063 已有 | 写行数记录条数只数一个角色 | 第 3 份写行那一格 `expect_err`（照发了） | 无 |
| M8 新 | 同 M2 | 第 4 份「恰好三条…被段边界那一判拒：None」 | 无 |
| M9 新 | `mount.rs` 建空闲图改回 `DeviceFreeMap::new`（默认起点） | 第 5 份 `:427` 那条（panic：`DeviceEndsBeforeTheUnitAreaStart { device_bytes: 18087936 … }`） | 过滤到这一条 |
| 765 已有 | 树表 0 条那一版不在取号之前拒 | 同一条，「准入不够在取号之前拒：None」 | 过滤 |
| 1132 改名 | 落得下那一判不判数据单元 | 384 槽那条：第 78 项变成 `PlacementRefused` | 过滤 |
| 1138 改名 | 会话被准入拒时不推抬 F | 256 槽那条：「第 10 次覆盖写」被原样交回（旧名写的是第 9 次） | 过滤 |
| 764 已有 | 推满仍不够就拒可写挂载 | 240 槽那条：「第 8 次崩了再挂做成」拿到 `SpaceAdmissionRefusedBeforeAcquisition` | 过滤 |
| 1125 已有 | 分配记录树那一项退回 K0 | 第 6 份五格那条：`two-1GiB 量到 {0, 2}`、`two-4GiB 量到 {2, 4}` 等对不上钉值 | 过滤 |
| 1124 已有 | 同 1125 | 第 6 份探针历史：2600 那一段少扣 4 次，4 GiB 那一段第 132 步 14 / 12 | 过滤 |
| M10 新 | 同 M2 | 第 6 份五格那条：`two-1GiB 量到 {36, 38}`，钉的是 {132, 134} | 过滤 |

每个改过的用例至少证过一行：A（M2）、B（M3）、C（912、M4）、D（M5、M6、M7）、第 2 份（M1）、第 3 份三条（1061、1062、1063）、第 4 份（M8）、第 5 份四条（M9 与 765、1132、1138、764）、第 6 份两条（1125、M10、1124）。变异表里没有另留一行给 59 号：追加的 12 行这两批都证过，改名重加的两行就是证过的 1132、1138。

## 五、停下交主 agent 的设计问题

**P1 `root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise` 红，是 A3b 第 4 条的形态与 D16 的用户定案冲突，没改。**
- 现象：主工作区 + A3b（有没有 C577 都一样）上，`raise_rollback_floor_to_the_admission_ceiling` 交回 `Err(Recovery(RootRingSlotStillUnreadableAfterOneReread { ring_slot: RootRingSlot { region: 0, slot: 0 } }))`。用例钉的是「挂载时就坏的槽不挡抬 F」（`check/` 上这份测试文件第 711 行那条 panic）。
- 来由：A3b 在抬 F（`check/` 上 `crates/singlefs-core/src/mount.rs:2228`）改成调 `recovery::readable_roots_rereading_unreadable_root_ring_slots_once`（`recovery.rs:1181`）。它对每一个读不出的槽都重读一次、仍读不出就拒，不分这个进程知不知道那一槽住着根。
- D16 那一行（`.claude/kb/decisions/16-发布语义.md:37`）逐字写着：「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根；挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F」（用户 2026-09-26 定）。那一行说的是「算准入抬 F 的上限时」，A3b 改的是抬 F 里「回收门槛与影子账」那一遍读根环；这两遍要不要同一种分法，条款没写。
- 同一件事在变异表上的样子：已有变异 774「实五 445：挂载那一刻就读不出的槽也重读、仍坏就拒（挡住了条款说当没有根的那一格）」，描述的正是 A3b 现在的产品行为。59 号跑它时，基线就红。
- 要定的：回收门槛与影子账那一遍读根环，是照 D16 那一行分两类（这个进程的根环表里没有的槽当没有根），还是照 C554 乙 Q6 一律重读、仍坏就拒。前者要改 A3b 的 `raise_the_floor_through` 与 `readable_roots_rereading_unreadable_root_ring_slots_once`（不在我的 8 份里）；后者要改这条用例，并碰用户定案那一行。

**P2（旁记，不是这一件要定的）**：
- 第三节第 5 份那条旁记：小盘上截断本身被空间拒。
- 第 7 份第 36 行那句过时的注释。
- A3b 报告第四节列的「用 `HistoryDeviceWidth::UnitAreaOf*` 的其余几份」不在我的单里，这次没跑。

## 六、主工作区现状（A3b + C577 已打）上跑 8 份：C577 之后还差什么

照主 agent 的消息（证红第二批之后收到）办：
- `git apply --check patch/crates.patch` 直接对主工作区跑，退出码 0。
- `research/scripts/apply-writer-patch.py patch --dry-run` 对主工作区的输出：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1350 行`。
- 另拷主工作区现状到 `check/`（09-27），用 `apply-writer-patch.py --root check/` 打上补丁，33 号 0。
- 本节与第四节的源码行号都在 `check/` 上现取。

8 份各跑一遍（内存上限 8G，线程上限 4），`test result` 行原样：

```
core_review_geometry_back_chain_and_empty_inode: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.49s
core_review_unit_area_start_and_publish_limits: test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.72s
core_review_tree_table_duplicates_and_slot_one_search: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
second_transaction_admission_raises_the_floor_before_refusing: test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 78.25s
admission_checkpoint_cost_per_device_paths: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 90.14s
second_transaction_allocation_record_tree_geometry_of_writer_and_reader: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
second_transaction_supplement_two_root_ring_turn_in_one_mount: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.71s
```

C577 之后，这 8 份没有新的钉值要变：唯一的红就是 P1 那条，C577 之前（`work/` 上）就红。

## 七、交回前的验证（`check/` 上：主工作区现状 + 本补丁；末尾原样）

- `cargo fmt --all -- --check`：`fmt exit 0`（输出为空）。
- `cargo build --offline --all-targets`：`    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 34.60s`，`build exit 0`。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 七条（照 `.claude/singlefs-ai-sop/scripts/check.sh:72-80` 现抄）：`    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.75s`，`clippy exit 0`。
- 动到的测试二进制：第六节那 8 行。
- 登记给我的门禁阶段（`stage-owners.tsv` 现取 7 道），末行与退出码：
  - 33 号 退出码 0：`✓ … crates/mutations.tsv 1345 条的原文各命中源码一次；…`
  - 53 号 退出码 0：`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
  - 74 号 退出码 1：`test result: FAILED. 21 passed; 3 failed; 2 ignored; …`，`✗ 随机历史的测试二进制判红`。红的三条都在 `second_transaction_supplement_three_random_history.rs`，不在我的单里：
    - `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`
    - `seed_4000000204_raising_the_floor_on_narrow_devices_is_refused_before_any_write_instead_of_leaving_the_new_floor_on_one_device`
    - `writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged`
    - A3b 报告第四节说的「基线红 1 条 + Q1 新增红 2 条」就是这三条，没修。
  - 92 号 退出码 0：`✓ 布局清单 1 套布局、6 条路径都在…这次改动比 HEAD，111 个格式常量里变了 0 个…`
  - 94 号 退出码 0：`✓ checker 与实现只共享常量模块 \`singlefs-format\`…`
  - 93 号 退出码 0：`✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（…扫了 57 个 .rs…）`
  - 89 号 退出码 77（本次未跑，不是通过）：末行是「「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐」。与这次改动无关。
- 负载：开跑前 `ps` 看到别的会话的 `cargo test`（`second_transaction_supplement_three_crash_injection`、`…three_random_history`）与 `e162-crash-verdict-block-store`，没有 qemu、fio、vm-bench。都用 `nice -n 19` 照跑，没等锁。

## 八、没做什么

- 没走三方对抗；层 0、QEMU、herd7、crates 变异整表（59 号）都没跑，归 `crash-verifier`；没提交，没碰主工作区（只做了 `git apply --check` 与 `--dry-run`）。
- P1 那条用例没改（设计问题）；第 7 份那句过时注释没改（规格：绿的不改）。
- A3b 报告第四节「静态推的」其余几份（`HistoryDeviceWidth::UnitAreaOf*` 的其他用户、崩溃枚举用例）不在我的单里，没跑。
- 第 5 份几处新钉值是历史跑出来的实测值；「为什么变」写的是推断（叶的切法变了），没有单独量每次发布重写几片叶。
- 草稿探查用的测试文件 `work/crates/singlefs-harness/tests/zz_scratch_a3b_fallout_one.rs` 只在副本里建过，出补丁之前删了，不在补丁里；探查日志 `scratch-explore-1.log`、`-2`、`-3` 留在草稿目录。
- 副本 `work/`、`check/` 里各起过一个草稿 git 仓（`git init` 与 commit，只在副本里），用来出 diff 与给 `apply-writer-patch.py --root` 用；主仓没有任何 git 写操作。

## 九、草稿目录清理

删了（删之前 `du -sh`）：
- `/tmp/claude-1000/impl-a3b-fallout-1/work`：17G，其中 target 17G。
- `/tmp/claude-1000/impl-a3b-fallout-1/check`：18G，其中 target 18G。
- `/tmp/claude-1000/impl-a3b-fallout-1/main-only`：2.2G，其中 target 1.9G。

留着：
- `patch/`（交给主 agent 打）；
- `report.md`、`progress.md`、`spec.md`；
- 各次日志：`base-runs/`、`prove-logs-1/`、`prove-logs-2/`、`prove-red-*.out`、`scratch-explore-*.log`、`check-*.log`、`gate-*.log`；
- 草稿 `file1-section15.rs`、`file3-section26.rs`、`mutations-append-draft.tsv`。
都不是编译目录，也不是仓副本。

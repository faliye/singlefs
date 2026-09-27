# 实审 A3-checker 实现员报告：checker 那一侧与 core 同漏的几处判定（代码审阅第 29、38 条）

写于 2026-09-27（时刻都是 UTC；东京 JST = UTC + 9）。规格 `/tmp/claude-1000/impl-rev-a3-checker/spec.md`。底座是主工作区 2026-09-27T04:24:51Z 的快照（`snapshot-sha256.txt`：checker `lib.rs` 6223df4d…、`image.rs` 1ac23367…、`walk.rs` 07ec2b47…）；在副本 `b/` 里改，交补丁 `patch/`。

## 一、结论

- 规格第 1、2、4 项做了：`check_unit` 判单元格式版本 = 1 与 29 字节预留位恒 0；`index_node_view` / `packed_unit_view` 判宽 0 而条数非 0；`check_system_configuration_slot` 判系统配置格式版本 = 1、加密类型 = 0（关），`image::geometry_of` 判固定结构槽距、`physical_block_size`、journal 环长的上下界。新测试 11 条，变异 14 行，prove-red 14/14 抓到（第五节）。
- 规格第 3 项（指针头部 MAC 16 / nonce 12）**没做，停下交主 agent**（第四节第 1 条）：checker 今天确实不判（`image.rs:378` / `:391` 的 `parse_node_pointer` / `parse_data_pointer` 只从偏移 34 起读）；要判就得在 `walk.rs` 的每个跟随指针的调用点上判，而 `walk.rs` 不在我的文件清单；另外还没有一条不变量编号装它。
- 第 1 项在池级只接上了一半（第四节第 2 条）：池级走读判 I-2.4 的是 `walk.rs:618` 自己那份 `judge_unit_header`，不经 `check_unit`。码 2 节点因为随后调 `index_node_view` 会被拒，池级报成 I-7.2（最新根走不完），**不是 I-2.4**；码 1 数据单元、码 3 打包容器在池级仍不判预留位与格式版本。
- checker 与实现不共享判定函数：新加的判定全在 `singlefs-checker` 里各写一份，偏移、版本号、登记值自己写常量，只从 `singlefs-format` 取 `ROOT_RECORD_BYTES`、`JOURNAL_RECORD_BYTES`、`JOURNAL_SAFETY_FACTOR`、`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`、`SLOT_BYTES`、`SYSTEM_CONFIGURATION_SLOT_BYTES` 这几个标量。门禁 94 号在副本上绿（第六节）。
- 点 checker 的 44 个非层 0 测试目标跑了：34 个绿，10 个红；10 个红在原样副本 `c/` 上逐条同红（第八节），不是这一件带来的。

**什么现象会推翻**：主工作区打上补丁后，第五节任一行在门禁 59 号里没红；或层 0、崩溃注入、随机历史里某个合法崩溃状态上 checker 的判定跟着变了（第七节说为什么推断不会变）。

## 二、这一轮写过的文件

补丁 `patch/crates.patch` 里：

- `crates/singlefs-checker/src/lib.rs`：`Verdict` 加 8 个成员（`FormatVersionNotRecognized`、`EncryptionReservedBytesNotZero`、`EntryWidthZeroWithEntries`、`RecordWidthZeroWithRecords`、`EncryptionTypeNotOff`、`FixedStructureSlotSpacingOutsideTheFormatRange`、`PhysicalBlockSizeOutsideTheRootSlotBounds`、`JournalRingBytesOutsideTheSupportedRange`）；常量 `FORMAT_VERSION_OFFSET`、`UNIT_FORMAT_VERSION_THIS_CHECKER_READS`、`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`、`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET`（= 155 + 32 + 16 + 12 + 4 = 219）、`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`；`check_system_configuration_slot`、`check_unit`、`index_node_view`、`packed_unit_view` 各加判定；后两个的切法去掉 `.max(1)` / `.take(count)`。
- `crates/singlefs-checker/src/image.rs`：`geometry_of` 加三道判定与三个私有判定函数（`fixed_structure_slot_spacing_lies_in_the_format_range`、`physical_block_size_fits_a_root_slot`、`journal_ring_bytes_lie_in_the_supported_range`）；`use singlefs_format` 多引三个常量。
- `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`：新建，11 条用例。

补丁之外：`patch/mutations-append.tsv` 追加 14 行，名字都以「A3-checker」起头（第五节逐行列）。没有 replacements、没有 delete。checker 自己的测试文件没建（新测试都放在 harness 那一份里，要用实现写出来的真单元、真系统配置槽）。

副本里临时建过又删掉的：`crates/singlefs-harness/tests/zz_scratch_probe_checker_today.rs`（改之前的探针，输出在 `probe-baseline.log`），不在补丁里。

`git diff --stat -- crates litmus` 没法给：改动在副本里做，副本没有 `.git`。补丁的等价统计（`git apply --stat patch/crates.patch`，在主工作区上跑）见第九节。

## 三、规格表四行各自怎么做的

改之前的现象都是副本里探针实测（`probe-baseline.log`，第一个文件版本的池，改一处、按类重封或重算整槽校验和）。

| # | 改之前（实测） | 改成什么（副本 `b/` 的行号） | 用例 |
|---|---|---|---|
| 1 | 数据单元、inode 树根、inode 叶容器三类：预留位第 0 / 14 / 28 字节改 1、或格式版本改 2，`check_unit` 全交 `Ok(类标签)`；inode 树根预留位改 1 并沿引用链补校验和的池，`check_pool_image` 42 条成立、0 违例 | `check_unit`（`lib.rs:361`）在两道校验和之后判格式版本 = 1（`:402` 报 `FormatVersionNotRecognized`）、预留位 29 字节全 0（`:411` 报 `EncryptionReservedBytesNotZero`）。放在校验和之后：没重封的坏字节照旧报 `ChecksumMismatch`，已有用例钉的成员不变 | `a_unit_whose_format_version_…`、`a_unit_with_a_non_zero_byte_among_the_twenty_nine_…`、池级 `an_inode_tree_root_with_a_non_zero_encryption_reserved_byte_breaks_the_walk_of_the_newest_root`（钉 I-7.2 红、说明含「条目区解不开」） |
| 2 | key 宽 0、条目宽 0、条目数 3、声明长度 0 的码 2 节点：`index_node_view` 交 `Ok`，切出 0 条；记录宽 0、记录数 3 的码 3：`packed_unit_view` 交 `Ok`，切出 0 条 | `index_node_view` `:461` 报 `EntryWidthZeroWithEntries`，`packed_unit_view` `:620` 报 `RecordWidthZeroWithRecords`；切法改成「条数 0 ⇒ 空，否则 `chunks(宽)`」，宽 0 只剩条数 0 一种 | `a_code_two_node_whose_entry_width_is_zero_…`、`a_code_three_unit_whose_record_width_is_zero_…`（两条都另钉「条数也是 0 的空单元照收」） |
| 3 | checker 不判指针头部 MAC / nonce | **没做**，见第四节第 1 条 | — |
| 4 | 系统配置四槽都改成格式版本 2 / 加密类型 1 / 环长 12287 / 环长 768 MiB + 4096：`check_system_configuration_slot` 与 `geometry_of` 都收，池级 42 条成立；槽距 4095 或 1044481：池级 25 条成立、0 违例；pbs 456 或 4097：池级只红 I-7.1 | `check_system_configuration_slot` 在整槽校验和之后判格式版本 = 1（`lib.rs:246`），incompat 之后判加密类型 = 0（`:258`）；`geometry_of`（`image.rs:207`）在 R、S 之后依次判槽距（`:265`）、pbs（`:279`）、环长（`:287`），越界这一槽不可择，与 R、S 越界同一个处置 | 格式版本、加密类型、槽距、pbs、环长各一条（每条含界内界外两侧与 0 / 最大值），池级 `a_pool_whose_every_system_configuration_slot_carries_a_refused_value_gets_no_invariant_vouched_for`（五种值各一份镜像，改后一条「成立」都不报） |

第 4 项的界与出处（条款现查，界从条款取，不从 core 抄）：

| 字段 | 界 | 出处 |
|---|---|---|
| 格式版本（偏移 4） | = 1 | D22（单元原子性怎么合成） 已定项 9 的字段表 = `.claude/kb/layout/01-first-txn.md` 一，第 110 行「自举头 \| 格式版本 \| 2 \| 1 \| 骨架」；单元头那一份是第 172 行「单元头 \| 格式版本 \| 2 \| 1」。「读者遇到别的版本拒」没有条款（第四节第 4 条） |
| 加密类型（偏移 219） | = 0 | D9（加密） 已定项 10（加密不进第一个可运行版本）；D22 已定项 17 登记表「0 未加密、1 ChaCha20-Poly1305、2 AES-256-GCM」；layout 第 122 行「0（未加密…）」 |
| 固定结构槽距（429） | ≥ 4096，且 槽距 + 4096 ≤ 同一槽「根环起点」字段 × 16384 | D2（RAID 条带策略） 已定项 19「槽距 = 4096 向上取整到 io_min 的整数倍」给下界；上界是 D22 已定项 16「槽 i 的设备内偏移 = i × 固定结构槽距」与第 1 句「基址 = 系统配置『根环起点』字段」推出来的（槽 1 整槽不伸进根环），没有条款直写 |
| `physical_block_size`（317） | ∈ [457, 槽距] | D22 已定项 2「根槽宽等于 physical_block_size」、已定项 7 根记录 457、已定项 16 第 2 句根槽 j 在区域起点 + j × 槽距；读者一侧的判法没有条款直写 |
| journal 环长（333） | 环长 ÷ 4096 ÷ F ≥ 1（F 取 `JOURNAL_SAFETY_FACTOR` = 3），且 环起点字段 × 16384 + 环长 ≤ 单元区起始槽号字段 × 16384 | D23（journal 的角色与格式） 已定项 18「在飞记录数上限 = 环槽数 ÷ F」、已定项 19 ③「单元区起始槽号随环长走」；读者一侧的判法没有条款直写 |

两个槽号字段（根环起点 371、环起点 325、单元区起点 417）都是盘上 8 字节：乘 16384 溢出时，根环起点 / 单元区起点当成在任何设备偏移之外（上界不挡），环末端溢出当成越界。

## 四、停下交主 agent 的几处

1. **规格第 3 项（指针头部 MAC 16 / nonce 12 在加密关着时恒 0）没做。** 条款今天有了：D19（块指针的结构与宽度预算） 已定项 3 射程（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:64`，主 agent 第五批写回 04:3x UTC 加的）「加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法」。卡在两处：
   - **调用点在 `walk.rs`**：checker 解指针的是 `image.rs:378` `parse_node_pointer` / `:391` `parse_data_pointer`，只交 `PointerView`、不交判定；每条被跟随的指针在 `walk.rs` 里判（`judge_location_order` 的 11 个调用点，另有根记录四条指针、journal 新根段两条、树表条目根指针）。`walk.rs` 不在这一件的文件清单里。
   - **报在哪条不变量下没定**：`Judgements::judge` 只收 `IMPLEMENTED_INVARIANTS` 里的编号（`image.rs:102` 起那一段断言）。「判该指针所在的结构损坏」在 checker 里没有对应的一条：借 I-2.4 就要把 I-2.4 的射程从「单元头」扩到「指针头部」（`invariants.md` 第 122 行要改），另立一条就要加 `IMPLEMENTED_INVARIANTS` 并让 `invariants.md` 的「已实现」计数跟着加 1。这是 kb 的事，不归我。
   - 建议的改法（交下一件）：`PointerView` 加一个成员记 MAC 16 + nonce 12 是否全 0（`parse_*` 里填，`image.rs`）；`walk.rs` 在上面那些调用点按主 agent 定的编号判；D9 已定项 10 射程那句「池级 checker 仍不判这些字段」随之改。
2. **第 1 项在池级只接上码 2。** 池级走读判 I-2.4 的是 `walk.rs:618` `judge_unit_header`（`:656` 只看头校验和），不经 `check_unit`。码 2 节点随后在 `walk.rs:720` 等处调 `index_node_view`，现在会被拒，池级报成 I-7.2 走读失败（「…的条目区解不开」）；码 1 数据单元（`walk.rs:1829`）与码 3 打包容器（`walk.rs:1777`）只过 `judge_unit_header`，预留位非 0、格式版本不认得而两道校验和都重封了的，池级仍不报。要报成 I-2.4，改法是 `judge_unit_header` 在 `header_holds` 之外再判这两样、并入 `:656` 那一次判定——`walk.rs` 不在清单，没改。另外 I-2.4 那一行没写格式版本，把格式版本也报在 I-2.4 下要不要，同一处定。改了之后，我那条池级用例钉的说明「条目区解不开」会变成「头用不了」，要跟着改一句。
3. **系统配置越界的处置与 core 不同**：checker 照自己 R、S 的先例，越界这一槽不可择（全部槽都越界时整片报不适用）；core（实审 A3a）是整池拒挂载（`recovery.rs` 的 `SystemConfigurationValueRefused`）。只有一部分槽带越界值时两边结局不同：core 拒，checker 用别的槽照判。要对齐哪一边，归主 agent 定（A3a 报告第四节第 2 点是同一个岔路）。
4. **读者一侧的判法条款没写，建议补的原句**（交主 agent / 用户定）：
   - D22 已定项 9 补：「读者遇到系统配置或单元头的格式版本不是 1，一律拒收那一槽 / 那一个单元，不按今天的字段表往下解。」（D15（格式冻结政策） `15-格式冻结政策.md:90` 写着「今天的读者不核版本号」，随这一件与 A3a 过时。）
   - D22 已定项 16 补：「读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前；`physical_block_size` ∈ [根记录宽 457, 槽距]。越界的槽不可择（checker）/ 整池拒挂载（实现）。」
   - D23 已定项 18 / 19 补：「读者判环长：环槽数 ÷ F ≥ 1，且环起点 + 环长不越过单元区起始槽号（checker 取同一槽的这两个字段；实现今天取编译期常量，C475）。」
   - D9 已定项 10 射程（`09-加密.md:231`）「池级 checker 仍不判这些字段，checker 那一半实现在做」改成：「池级 checker 判单元头 29 字节全 0（`check_unit`，池级经码 2 节点解码报在 I-7.2；码 1 / 码 3 的池级判定与指针头部 MAC / nonce 仍欠）、系统配置格式版本 = 1 与加密类型 = 0（越界这一槽不可择）。」
5. **宽 0 而条数非 0 归哪条不变量**：码 3 已有 I-1.7（`invariants.md` 第 30 行「记录宽 == 登记表为该打包记录类型登记的宽」，宽 0 不是任何登记宽），池级 `walk.rs:1788` 本来就判；这一件只是让 `packed_unit_view` 这个单元级函数不再交出与头不符的视图。码 2 只有 I-1.10（第 33 行，登记树的条目宽 = 字段表宽），但它**不罩中央映射树（C307）与树表条目**，没有一条不变量写「条目宽 0 而条目数非 0 判损坏」。建议补进 I-1.10 或另立一行：「任一码 2 索引节点，条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它）；否则判该节点损坏。」池级今天这一格经 `index_node_view` 报成 I-7.2 走读失败。
6. **第 2 项两行变异红在 panic 上，不在断言上**：变异把判定改成 `if false` 之后，新的切法 `chunks(宽)` 在宽 0 时 panic（「chunk size must be non-zero」，`lib.rs:473` / `:633`），用例因此红。判定就是挡这个 panic 的那一道；改之前的原样（`.max(1).take(count)`）交的是切出 0 条的 `Ok`，用例的断言对它红——那一格由 `probe-baseline.log` 的 `PROBE width-0 node index_node_view = Ok((0, 0, 0))` 与 `PROBE width-0 packed packed_unit_view = Ok((0, 0))` 实测过。

没加、只列出的（条款没写的不是分支）：`PointerView` 新成员与访问器（第 1 条）、`Verdict` 的 `Display`（今天没有）。

## 五、新测试与证红

证红一律 `research/scripts/prove-red.sh --copy b/ --memory 8G singlefs-harness <14 个名字>`（线程上限 4），原样输出在 `prove-red-run-1.log`，逐条日志 `prove-red-logs/001.log`–`014.log`。基线（不改源码跑一次）是 prove-red 自带的第 ④ 步，没红。每行的参数都是 `-p singlefs-harness --test checker_judges_reserved_bytes_and_widths_it_used_to_skip -- <测试名>`，所以同一个二进制里别的测试被名字过滤掉了，「同时红了哪些」只看得到点名那一条。14 行全部证过，没有留给 59 号的。

| 变异名（`mutations-append.tsv`） | 改坏哪一行 | 哪条断言红 |
|---|---|---|
| A3-checker 第 29 条：check_unit 不判单元格式版本 | `lib.rs:401` 判定改 `if false` | `a_unit_whose_format_version_…` 第 139 行：数据单元格式版本 0，`Ok(1)` ≠ `Err(FormatVersionNotRecognized)` |
| A3-checker 第 29 条：check_unit 不判 29 字节预留位恒 0 | `lib.rs:411` 改 `return Ok(unit_class)` | `a_unit_with_a_non_zero_byte_…` 第 167 行：数据单元预留位第 0 字节 1，`Ok(1)` |
| A3-checker 第 29 条：check_unit 不判预留位，池级走读照走完预留位非 0 的 inode 树根 | 同上一行 | `an_inode_tree_root_…` 第 238 行：I-7.2 是 `Holds` |
| A3-checker 第 38 条：index_node_view 收条目宽 0 而条目数非 0 的节点 | `lib.rs:460` 改 `if false` | `a_code_two_node_…`：panic 在 `lib.rs:473` `chunks` 宽 0（第四节第 6 条） |
| A3-checker 第 38 条：packed_unit_view 收记录宽 0 而记录数非 0 的单元 | `lib.rs:619` 改 `if false` | `a_code_three_unit_…`：panic 在 `lib.rs:633`（同上） |
| A3-checker 第 29 条：系统配置槽不判格式版本 | `lib.rs:243–245` 改 `if false` | `a_system_configuration_slot_whose_format_version_…` 第 355 行：版本 0，`Ok(())` |
| A3-checker 第 29 条：系统配置槽不判加密类型 | `lib.rs:256–257` 改 `if false` | `…encryption_type_is_not_off…` 第 370 行：加密类型 1，`Ok(())` |
| A3-checker 第 29 条：系统配置不判加密类型，池级照替加密的卷作保 | 同上一行 | `a_pool_whose_every_system_configuration_slot_…` 第 549 行：「加密类型 1」42 条成立 ≠ 0 |
| A3-checker 第 38 条：固定结构槽距不设下界 | `image.rs:270` 改 `true` | `…slot_spacing…` 第 406 行：槽距 4095，`Ok(())` |
| A3-checker 第 38 条：固定结构槽距不设上界（槽 1 伸进根环照收） | `image.rs:273` 闭包恒真 | 同一条第 406 行：槽距 1044481，`Ok(())` |
| A3-checker 第 38 条：physical_block_size 不设下界（装不下根记录照收） | `image.rs:280` 去掉 `>= 457` | `…physical_block_size…` 第 442 行：456，`Ok(())` |
| A3-checker 第 38 条：physical_block_size 不设上界（宽过槽距照收） | `image.rs:280` 去掉 `<= 槽距` | 同一条第 442 行：4097，`Ok(())` |
| A3-checker 第 38 条：journal 环长不设下界（装不下 F 条记录照收） | `image.rs:296` 改 `true` | `…journal_ring…` 第 481 行：12287，`Ok(())` |
| A3-checker 第 38 条：journal 环长不设上界（环末端越过单元区起点照收） | `image.rs:300` 闭包恒真 | 同一条第 481 行：805306369，`Ok(())` |

`prove-red-run-1.log` 末两行原样：

```
✓ 点名 14 条：跑了 14 条，跳过 0 条，跑的都抓到了
exit 0
```

新测试文件整个二进制（最终那一版，含 clippy 要的 `type` 别名）：`new-tests-2.log` 末行 `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s`。

「改之前的 checker 不报」那一半没法拿这 11 条直接在原样代码上跑（它们点名了新成员，原样编不过），由探针实测：`probe-baseline.log` 全部 `PROBE` 行（第三节表第二列）。

## 六、第 4 步那几样的末尾原样输出（副本 `b/`，线程上限 4，内存上限 8G）

`final-checks.log` 原样：

```
### fmt
Diff in /tmp/claude-1000/impl-rev-a3-checker/b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs:3059:
fmt exit 1
### clippy checker
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
exit 0
### clippy harness new test
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.94s
exit 0
### build checker all targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
exit 0
### build harness lib + bins
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
exit 0
```

- fmt 那一处 `first_transaction_on_device.rs:3059` 不是我的文件，主工作区同一处同样报（同一次 `cargo fmt --all -- --check` 在主工作区跑出同一行）。我的三份文件 fmt 干净。
- clippy 带 `-D warnings` 加 `check.sh` 的 `CODE_DISCIPLINE_LINTS` 七条（`.claude/singlefs-ai-sop/scripts/check.sh:72` 起现抄）。照规格不用 `--all-targets` 跑全工作区（E161 那份 bin），改成 `-p singlefs-checker --all-targets` 与 `-p singlefs-harness --test <新目标>`；E161 在副本里编得过，没删。
- 登记给我的门禁阶段（`stage-owners.tsv` 里 implementation-writer 那几道），都以 `b/` 为根跑（`bash .claude/gate.d/<阶段> /tmp/claude-1000/impl-rev-a3-checker/b`），日志在 `gates/`：

| 阶段 | 退出码 | 末行原样（截到 300 字） |
|---|---|---|
| 33-mutation-tables | 1 | `crates/mutations.tsv:659 E158 root_choice_repair session s10：…原文在 crates/singl…`——红在第 659 行 E158 那一行（副本快照里的 E158 bin 比主工作区 14:1x JST 打上的旧），不是我的行；我那 14 行没被点名 |
| 53-format-const-placeholders | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` |
| 74-model-differential | 1 | `✗ 随机历史的测试二进制判红`；`test result: FAILED. 21 passed; 3 failed; 2 ignored`——三条红与改后目标那一轮同名，基线见第八节 |
| 89-closeout-row27-preconditions | 77 | 本次未跑：`「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上…`，与这一件无关 |
| 92-layout-checker-sync | 77 | 本次未跑：`! /tmp/claude-1000/impl-rev-a3-checker/b 不是 git 仓，本阶段跳过` |
| 93-feature-bits | 0 | `✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 57 个 .rs…）` |
| 94-checker-implementation-disjoint | 0 | `✓ checker 与实现只共享常量模块 singlefs-format（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个…）；checker 的 4 份源码零处引 singlefs_core` |

92 号在副本上判不了：补丁打进主工作区之后由主 agent 那一轮跑（这一件改的是 checker 判定路径、没改格式常量，推断不会红）。

## 七、受影响的层 0 流与崩溃枚举用例

改的是 checker 这几条的判定集合：`check_unit`（多拒两种）、`index_node_view` / `packed_unit_view`（多拒宽 0 而条数非 0）、`check_system_configuration_slot`（多拒格式版本 ≠ 1、加密类型 ≠ 0）、`geometry_of`（多拒槽距、pbs、环长越界）。池级表现：码 2 节点被拒 ⇒ 走读失败（I-7.2、I-4.8、I-7.4 这类按走读判的）；系统配置槽被拒 ⇒ 这一槽不可择。

**推断（没跑层 0 与崩溃枚举）：层 0 八条流、`stage-inputs.tsv` 里七个 `crash-case:` 用例的钉值都不跟着变。** 理由：

- 这些流与用例写出的单元全来自实现的写者：格式版本写 1（`crates/singlefs-core/src/unit.rs` 的 `write_common_prefix` 写 `FORMAT_VERSION` = 1）、29 字节预留位 `skip` 成 0；条目宽 / 记录宽都是字段表的宽，非 0。系统配置来自 mkfs：格式版本 1、加密类型 0；几何取 `common::parameters` / `segments.rs:261` 起那几份 / `scenario::e142_parameters`：槽距 4096 或 `slot_spacing_for(io_min)`、pbs 512、环长 768 MiB，都在界内（环末端恰好等于单元区起点，界是 `<=`）。
- 崩溃态里撕裂、没落盘的单元与系统配置槽，先在校验和那一步被拒（新判定都排在两道校验和 / 整槽校验和之后），交回的成员与改之前相同。
- 层 0 用的读者 `MemoryPool` 给了 journal 与单元的候选槽，环长字段在这些流里不参与扫环。

会变的只有人为改坏又重封过的镜像。受影响的已有用例我逐个跑了（第八节 44 个目标）：改后与原样副本逐条同结局。推翻条件：提交时层 0 快档 / 全量、崩溃注入快档的计数有一格变了。

## 八、点 checker 的非层 0 测试目标：改后 vs 原样副本

目标清单 `checker-targets.txt`（`crates/singlefs-harness/tests/` 里直接引 `singlefs_checker`、名字不带 layer0 的 44 份），外加 `-p singlefs-checker --lib`。改后在 `b/` 上逐个跑（`run-checker-targets.sh`，整条经 `run-with-memory-cap.sh 8G`、线程上限 4），逐目标日志与汇总在 `targets-after-change/`：45 行里 35 行退 0（lib 5 条、34 个目标全绿），10 行退 101。10 个红的目标在原样副本 `c/`（`b/` 拷一份、checker 两份源码与变异表换回快照、删掉新测试）上重跑（`run-baseline-reds.sh`、`run-baseline-reds-2.sh`、tree_split 单跑），日志在 `targets-baseline/`。逐目标比「每条测试的 ok / FAILED / ignored」集合：10 个都 `same-per-test`。

| 目标 | 改后 `test result` | 原样副本 `test result` |
|---|---|---|
| checker_known_bad_images | FAILED. 37 passed; 2 failed | FAILED. 37 passed; 2 failed |
| crash_injection_record_checker_sees_every_publish_across_a_second_crash | FAILED. 7 passed; 1 failed | FAILED. 7 passed; 1 failed |
| second_transaction_step_five_reuse | FAILED. 13 passed; 1 failed | FAILED. 13 passed; 1 failed |
| second_transaction_step_three_second_instance | FAILED. 5 passed; 1 failed | FAILED. 5 passed; 1 failed |
| second_transaction_supplement_three_bad_disk_input | FAILED. 8 passed; 1 failed; 1 ignored | FAILED. 8 passed; 1 failed; 1 ignored |
| second_transaction_supplement_three_crash_injection | FAILED. 7 passed; 3 failed; 2 ignored | FAILED. 7 passed; 3 failed; 2 ignored |
| second_transaction_supplement_three_fault_injection | FAILED. 12 passed; 1 failed; 1 ignored | FAILED. 12 passed; 1 failed; 1 ignored |
| second_transaction_supplement_three_random_history | FAILED. 21 passed; 3 failed; 2 ignored | FAILED. 21 passed; 3 failed; 2 ignored |
| second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version | FAILED. 15 passed; 2 failed | FAILED. 15 passed; 2 failed |
| second_transaction_supplement_two_tree_split | FAILED. 4 passed; 1 failed | FAILED. 4 passed; 1 failed |

这些红出在快照那一刻的 core / harness 上（多是可写挂载报 `NewerStateStillUnreadableAfterOneReread`、`InvariantViolated`，或模型说该成、实现拒了；tree_split、bad_disk_input 那两条是 A3a 报告第七节点名要跟着改的），与 checker 这一件无关；主工作区 15:0x JST 之后又打了乙-配置续与模型跟上乙，今天的红集可能已经不同，我没在主工作区现状上重跑。同名同结局只比到「每条测试红没红」，没逐条比红的说明文字。

## 九、补丁与打法

`patch/` 里：`crates.patch`（3 个文件：`lib.rs`、`image.rs` 修改，新测试文件新建）、`mutations-append.tsv`（14 行，六段）、`report.md`（本文的一份拷贝）。没有 `mutations-replacements.tsv`、`mutations-delete.txt`。

对主工作区现状（`crates/mutations.tsv` 1286 行时）核过：`git apply --check patch/crates.patch` 通过；14 个变异名在主工作区的表里都没有；checker `lib.rs` / `image.rs` / `walk.rs` 的 sha256 与快照相同（6223df4d… / 1ac23367… / 07ec2b47…）。主工作区 `tests/checker_known_bad_images.rs` 在快照之后改了 128 行，里面没有一处碰 `check_unit`、`index_node_view`、`packed_unit_view`、`check_system_configuration_slot`、`geometry_of`、预留位、宽 0 或系统配置这几个字段；`bad_disk_input.rs`、它的测试与 `fault_injection.rs` 的改动同样查了，没有交集。`git apply --stat` 原样：

```
 crates/singlefs-checker/src/lib.rs                 |  103 +++-
 crates/singlefs-checker/src/image.rs               |   81 +++
 ...es_reserved_bytes_and_widths_it_used_to_skip.rs |  555 ++++++++++++++++++++
 3 files changed, 719 insertions(+), 20 deletions(-)
```

`python3 research/scripts/apply-writer-patch.py <补丁目录> --dry-run` 原样：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1300 行`。

14 行变异的原文锚在我改的两份 checker 源码上，打完之后在主工作区各恰好命中一次（副本里核过，那两份文件主工作区没变）。

## 十、没做什么

- 规格第 3 项（指针头部 MAC / nonce）没做；第 1 项池级码 1 / 码 3 那一半没接（第四节第 1、2 条，都要改 `walk.rs`）。
- 没跑层 0（快档、全量）、崩溃枚举用例、门禁 54 / 55 / 57 / 59 与变异整表、全量 `cargo test`、`check.sh`（重型，归提交时的崩溃验证员与整轮门禁）。
- 没用 `--all-targets` 跑全工作区的 clippy / build（规格要求避开 E161 那份 bin），改成按 crate 与目标跑。
- 没在主工作区现状上重跑那 44 个目标（第八节末）。
- 门禁 92 号在副本上是 77（不是 git 仓），89 号 77，都没判。
- 没走三方对抗；没提交；没写 kb（第四节第 4、5 条的原句交主 agent）。

## 十一、草稿与删掉的东西

草稿目录 `/tmp/claude-1000/impl-rev-a3-checker/` 里留着：`report.md`、`progress.md`、`spec.md`、`patch/`、各轮日志（`probe-baseline.log`、`new-tests-*.log`、`prove-red-run-1.log`、`prove-red-logs/`、`targets-after-change/`、`targets-baseline/`、`gates/`、`final-checks.log`、`clippy-*.log`、`build-baseline-1.log`、`prebuild-targets.log`）与几份跑用的脚本（`run-checker-targets.sh`、`run-baseline-reds*.sh`）。仓副本与编译目录在交回前删掉，大小见交回消息。

# 实审 A3-checker-2 实现员报告：池级走读补齐 I-2.4（单元头与指针头部）、宽 0 报 I-1.10、系统配置越界报违例

写于 2026-09-27。规格 `/tmp/claude-1000/impl-rev-a3-checker-2/spec.md`。底座是主工作区 2026-09-27 的现状（`snapshot-sha256.txt`：checker `lib.rs` 7d7e4d95…、`image.rs` b3a314d2…、`walk.rs` 07ec2b47…、测试文件 19daca56…；复核过没变）。在副本 `b/` 里改，交补丁 `patch/`。

## 一、结论

- 规格四项都做了，15 行新变异经 `prove-red.sh` 逐条抓到（第五节）。
- **有一处红要主 agent 处置（第四节第 1 条）**：第 3 项要「报违例」，可 `invariants.md` 里没有一条罩得住「系统配置池级字段越界」，我按 I-1.11 的先例在 `image.rs` 暂立了一个具名常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS = "I-7.13"`，并把它加进 `IMPLEMENTED_INVARIANTS`（46 → 47）。这样一来，`checker_known_bad_images` 的 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 就红了：它要求清单里每条不变量在那份文件里都有一份坏镜像。那份文件不在我的清单里，我没碰。要么补一份坏镜像，要么换编号或换处置，由主 agent 定。
- 点 checker 的 44 个非层 0 测试目标加 checker `--lib` 都跑了：39 个绿、6 个红。6 个红的放到原样副本上重跑，其中 5 个逐条同结局，不是这一件带来的；只有上面那一条是这一件引起的（第七节）。
- 门禁：见第六节。

**什么现象会推翻**：主工作区打上补丁之后，第五节任一行在门禁 59 号里没红；或者层 0、崩溃注入、随机历史里某个合法崩溃状态上，除 I-7.13 多出的那一格评估之外，checker 的判定也跟着变了（第八节说明为什么推断不会变）。

## 二、这一轮写过的文件

补丁 `patch/crates.patch`（`git apply --stat` 原样，在主工作区上跑）：

```
 crates/singlefs-checker/src/walk.rs                |  159 +++++++-
 crates/singlefs-checker/src/image.rs               |  248 ++++++++++--
 crates/singlefs-checker/src/lib.rs                 |    6 
 ...es_reserved_bytes_and_widths_it_used_to_skip.rs |  413 ++++++++++++++++++++
 4 files changed, 760 insertions(+), 66 deletions(-)
```

- `image.rs`：新增常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`（:46），`IMPLEMENTED_INVARIANTS` 由 46 条变 47 条（:54），注释里写明 I-2.4 射程已扩；新增 `SystemConfigurationSlotReading`（:320）、`value_refused_by_this_reader`（:339，对 `Verdict` 穷举匹配）、`system_configuration_slot_readings`（:400）、`judge_system_configuration_values_the_reader_accepts`（:454）；`verified_system_configuration_slots` 改为在 readings 基础上筛出可用的槽，读法不变。`PointerView` 加成员 `is_mac_and_nonce_all_zero`（:575），两个 `parse_*` 负责填它；新增 `judge_pointer_mac_and_nonce_are_zero`（:619）。`geometry_of` 的文档注释跟着改了。
- `walk.rs`：`judge_unit_header`（:649）加判两格 I-2.4：格式版本（:695）与 29 字节预留位（:730），任一不过就判「头用不了」；新增 `EntryFieldTableRegistration`（:193）与 `index_node_view_judging_a_zero_entry_width`（:747），`read_index_node` 多一个参数，全部调用点都接上；四处跟随指针的地方调用 `judge_pointer_mac_and_nonce_are_zero`（:790、:1440、:1766、:1916）；`check_pool_image` 在择系统配置之后判 I-7.13，任一槽带越界值就整池不作保（:5573 起）。
- `lib.rs`：`FORMAT_VERSION_OFFSET`（:203）与 `UNIT_FORMAT_VERSION_THIS_CHECKER_READS`（:206）改成 `pub(crate)`，给走读用。
- 测试文件：新增 4 条池级用例，改了原有 2 条（第五节）。

补丁之外还交了三样：`patch/mutations-append.tsv` 追加 15 行，名字都以「A3-checker-2」开头；`patch/mutations-delete.txt` 删 1 行，即「A3-checker 第 29 条：check_unit 不判预留位，池级走读照走完预留位非 0 的 inode 树根」，理由见第五节末；`patch/report.md` 是本文的拷贝。没有 replacements。

`git diff --stat -- crates litmus` 给不出：改动在副本里，副本没有 `.git`，上面那份 `--stat` 是等价的统计。

## 三、规格表四行各自怎么做的

| # | 改之前 | 改成什么 |
|---|---|---|
| 1 | 指针头部 MAC 16 / nonce 12 不判 | `PointerView.is_mac_and_nonce_all_zero` 记指针偏移 0..28 是否全 0。走读跟随指针的四个入口都判 I-2.4：`read_index_node`（所有码 2 节点都从这里读，包括根记录的树表指针、映射根、分配记录树根、journal 新根段的两条、树表条目里的根指针，以及各树的子指针）；inode 内部条目的子指针；实例表链（根记录里的实例表指针和链指针）；extent 数据指针。这些入口也都经过 `note_references_of_a_pointer_followed_by_the_walk`。只判不断：指针指向的单元照样走下去。中央映射条目不判，它的 value 只有位置条目，没有指针头部 |
| 2 | 池级 `judge_unit_header` 只看头校验和；码 2 在解码时被拒，报成 I-7.2；码 1、码 3 不判 | 三种码一视同仁。头校验和过了之后判格式版本 = 1（I-2.4）；载荷 CRC 也过了之后判 29 字节预留位全 0（I-2.4），这个次序与 `check_unit` 一致。任一不过就记走读失败「头用不了（…）」并返回不能用，于是码 1、码 3 的 I-7.2 也会跟着红（照旧由它自己的判据报）。载荷 CRC 不过时不判预留位，那一格 I-2.3 已经红了 |
| 3 | 越界的槽当「这一槽不可择」，拿别的槽照判 | 槽读下来分三类：可择、「自证过但带越界值」、不可择。格式版本、加密类型、槽距、pbs、环长这五种越界归第二类，报 I-7.13 违例（说明里点名盘和偏移），其余不变量一律报「不适用」，理由是挂不上，与实现 `SystemConfigurationValueRefused` 整池拒挂载是同一个结局。R、S 越界照旧归不可择，没动 |
| 4 | 码 2 条目宽 0 而条目数非 0：解码被拒，只记走读失败 | 解码被拒的理由若是 `EntryWidthZeroWithEntries`，而这棵树登记了条目字段表（extent、inode、分配记录、记账），就报 I-1.10。树表和中央映射树不在 I-1.10 的射程里，照旧只记走读失败（第四节第 3 条） |

## 四、停下交主 agent 的几处

1. **第 3 项报在哪条不变量下（条款没写，我暂立了编号）。** 系统配置那几条（I-1.5、I-2.2、I-7.6、I-7.7、I-7.12、I-8.1）没有一条罩得住「池级字段越界」；实现那边的 `recovery.rs:190`（行号现查）也写着「这几样都是盘上读来的值、不是不变量」。可规格要「报违例」，而 `Judgements::judge` 只收 `IMPLEMENTED_INVARIANTS` 里的编号，所以我照 I-1.11 的先例暂立了 `"I-7.13"`，要改号只改 `image.rs:46` 这一处。带来的连锁：
   - `checker_known_bad_images.rs:2344` 那条断言红了（清单里每条都要有一份坏镜像）。要在那份文件的某个 `known_bad_images_*` 里给 I-7.13 补一份坏镜像，写法可以直接抄我那条用例：盘 1 一槽改加密类型 1、重算整槽校验和。那份文件不归我，没改。
   - 建议 kb 第六批补的不变量原句：「**I-7.13 系统配置池级字段在读者收的范围里**：任一自证过（magic 与整槽校验和对）、incompat 位认得的系统配置槽，格式版本 = 1、加密类型 = 0（第一版）、固定结构槽距 ≥ 4096 且槽 1 整槽落在根环起点之前、`physical_block_size` ∈ [457, 槽距]、journal 环长 ÷ 4096 ÷ F ≥ 1 且环末端不越过单元区起点；任一盘任一槽不成立即判红，整池拒绝挂载（checker 其余不变量报不适用）。R、S 越界这一槽不可择，不在这一条里。」`invariants.md` 第 12 行的「判 46 条」随之变成 47。
   - 我另外定了一件事，请主 agent 确认：只要有一槽越界，**其余不变量全部报不适用**（不拿别的槽照判）。理由是实现整池拒挂载，而 `check_pool_image` 对「挂不上的镜像」的现成处置就是不作保。
2. **I-2.4 的射程**（主 agent 已定，由 kb 第六批改 `invariants.md` 第 122 行）。现在这一行没写「格式版本」，也没写「指针头部」。建议加一句：「池级走读在单元头上另判偏移 4 的格式版本 = 1（不认得就不按今天的字段表往下解），在走读跟随的每条指针上判头部 MAC 16 + nonce 12 全 0（D19 已定项 3 射程），读者遇到不成立判该结构损坏」。另外，D9 已定项 10 射程里「池级 checker 仍不判这些字段」那一句也过时了。
3. **宽 0 而条目数非 0，在树表和中央映射树上只记走读失败、不报违例**：I-1.10 那一行明写不罩这两棵，我没往外扩。要不要扩，归 C307 与 C476。
4. **指针 MAC / nonce 非 0 时只判不断**：规格只说报 I-2.4，没说要不要断走读。我让走读照走，所以 I-7.2、I-4.8、I-7.4 不跟着红。如果要按 D19「判该指针所在的结构损坏」理解成走读也要断，改法是在四个入口判完后返回；这一点没有用例钉住，改起来不牵动测试。
5. **单元头格式版本或预留位不过时判「头用不了」**：这让码 1、码 3 的 I-7.2 也会红，与码 2 一致；这一点用例钉住了。A3-checker 报告第四节第 2 条预言过这一改法。

没加、只列出的（条款没写，也不是分支）：没有。

## 五、新测试与证红

新用例 4 条，都在 `checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`，从实现写出的第一个文件版本池出发：先改一处、按类重封，再把新的整单元校验和沿引用链补到根槽（根记录里的指针直接改根槽、重算自证校验和）：

| 用例 | 场景 |
|---|---|
| `every_unit_class_with_an_unrecognized_format_version_or_a_non_zero_reserved_byte_reddens_the_header_invariant_in_the_pool_walk` | 数据单元、inode 树根、inode 叶容器三种码，各做三处改动：格式版本改 2、预留位第 0 字节、第 28 字节。要求 I-2.4 违例（说明里点名「格式版本」或「预留位」），I-7.2 违例（说明里有「头用不了」） |
| `a_followed_pointer_whose_mac_or_nonce_is_not_zero_reddens_the_header_invariant` | 根记录的树表指针、根记录的实例表指针、inode 树根的子指针、extent 树根的内联数据指针四处，各把头部第 0、15、16、27 字节改成 1。要求 I-2.4 违例，说明里点名那一处 |
| `a_node_of_a_registered_tree_whose_entry_width_is_zero_while_its_entry_count_is_not_reddens_the_entry_width_invariant` | 用 key 宽 0、条目宽 0、条目数 3 的码 2 节点顶替 inode 树根（树 ID、层级、fsid 抄原样）。要求 I-1.10 违例，说明里有「条目宽 0 而条目数不是 0」 |
| `a_pool_where_one_system_configuration_slot_carries_a_refused_value_is_reported_and_not_vouched_for` | 只改盘 1 的槽 0 或槽 1，五种越界值各试一次。要求 I-7.13 违例、说明里点名「盘 1 偏移 …」，且一条「成立」都不报 |

改过的原有用例 2 条：`an_inode_tree_root_with_a_non_zero_encryption_reserved_byte_breaks_the_walk_of_the_newest_root` 的说明钉值从「条目区解不开」改成「头用不了」；`a_pool_whose_every_system_configuration_slot_carries_a_refused_value_gets_no_invariant_vouched_for` 多一句断言：I-7.13 违例。

「改之前 checker 不报」这一半没法拿新用例直接在原样代码上跑：它们点名了新常量，原样代码编不过。下表每行变异都把判定改回改之前的样子（不判、恒成立、退回「这一槽不可择」），由它们证明改之前不报。

证红一律用 `research/scripts/prove-red.sh --copy b/ --memory 8G singlefs-harness <名字…>`（线程上限 4），每行参数都是 `-p singlefs-harness --test checker_judges_reserved_bytes_and_widths_it_used_to_skip -- <测试名>`，所以同一个二进制里别的测试被名字过滤掉了。15 行全部亲自证过，没有留给 59 号的：

| 变异名（`mutations-append.tsv`） | 改坏哪里 | 哪条用例红 |
|---|---|---|
| A3-checker-2 第 29 条：池级走读不判单元头格式版本 | `walk.rs:695` 比较改成恒真 | 单元头那条 |
| A3-checker-2 第 29 条：池级走读不判单元头 29 字节预留位 | `walk.rs:730` 前面加 `true \|\|` | 单元头那条 |
| A3-checker-2 第 29 条：预留位非 0 的单元头照当能用往下走 | `walk.rs` 的 `if !reserved_bytes_are_zero` 改成 `if false`（第一版替换文只删 `return false`，编译通过但没红：`prove-red-run-2.log` 第 3 行；改用这一版后在 `prove-red-run-3.log` 抓到） | 单元头那条（I-7.2 不红） |
| A3-checker-2 第 29 条：池级走读不判预留位，预留位非 0 的 inode 树根走到解条目区才断 | 同第 2 行 | `an_inode_tree_root_…`（说明变回「条目区解不开」） |
| A3-checker-2 第 29 条：解指针不记 MAC / nonce 是否全 0（恒记成全 0） | `image.rs` 的 `mac_and_nonce_are_all_zero` 恒真 | 指针那条 |
| A3-checker-2 第 29 条：指针 MAC / nonce 判定恒成立 | `image.rs:624` 的判定改成 `true` | 指针那条 |
| A3-checker-2 第 29 条：读码 2 节点不判指针 MAC / nonce | 删掉 `walk.rs:790` 那一次调用 | 指针那条（树表指针那一格） |
| A3-checker-2 第 29 条：inode 内部条目子指针不判 MAC / nonce | 删掉 `walk.rs:1440` | 指针那条 |
| A3-checker-2 第 29 条：实例表链指针不判 MAC / nonce | 删掉 `walk.rs:1766` | 指针那条 |
| A3-checker-2 第 29 条：extent 数据指针不判 MAC / nonce | 删掉 `walk.rs:1916` | 指针那条 |
| A3-checker-2 第 38 条：登记树的码 2 节点条目宽 0 而条目数非 0 不报 I-1.10 | `walk.rs:756` 条件前加 `false &&` | 宽 0 那条 |
| A3-checker-2 第 38 条：树表条目指的登记树当成不在 I-1.10 射程里 | `walk.rs` 树表条目那一处把 `Registered` 改成 `NotRegistered` | 宽 0 那条 |
| A3-checker-2 第 29 条：系统配置越界值退回「这一槽不可择」 | `image.rs:345` 的 `=> true` 改成 `=> false` | 系统配置那条 |
| A3-checker-2 第 29 条：带越界值的系统配置槽判成立 | `image.rs` 越界那一臂的 `false` 改成 `true` | 系统配置那条 |
| A3-checker-2 第 29 条：一部分系统配置槽越界时拿别的槽照判、照作保 | `walk.rs:5578` 条件前加 `false &&` | 系统配置那条 |

原样输出：`prove-red-run-2.log`（14 条里 13 条抓到，第 3 行没红，已换替换文）、`prove-red-run-3.log` 与 `prove-red-run-4.log` 末行 `✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了`。基线（不改源码先跑一遍）是 prove-red 自带的第 ④ 步，没红。

**删掉的旧行**：「A3-checker 第 29 条：check_unit 不判预留位，池级走读照走完预留位非 0 的 inode 树根」。在新代码上复证它没红（`prove-red-run-a3.log`，A3-checker 那 14 行里其余 13 行照样抓到）：池级现在在 `judge_unit_header` 就拒了这个节点，改 `check_unit` 已经碰不到这条池级用例。同一条用例改由上表第 4 行守。

**顺带修的锚点**：我重写 `verified_system_configuration_slots` 之后，变异表第 990 行「实审 B2 追加件（A2a 第 37 条对齐）：槽 0 无效的盘不借别的盘记的槽距，照 4096 找槽 1」的原文命中 0 次（33 号红）。我把新代码里的变量名恢复成 `spacing_recorded_by_the_first_valid_slot_zero`，缩进也对上了，锚点回到恰好命中 1 次。那一行在新代码上复证抓到（`prove-red-run-5.log`），33 号复跑绿。

## 六、第 4 步那几样的末尾原样输出（副本 `b/`，线程上限 4，内存上限 8G）

`final-checks-2.log`（锚点改名之后的最终一版）原样：

```
### fmt
fmt exit 0
### clippy checker all targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
exit 0
### clippy harness new test
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.44s
exit 0
### build all targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.08s
exit 0
### new test binary
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s
exit 0
```

- clippy 带 `-D warnings`，另加 `check.sh:72` 起现抄的 `CODE_DISCIPLINE_LINTS` 七条。改完之后 `touch image.rs` 强制重查过一次，输出里有 `Checking singlefs-checker`，退出码 0。harness 这一边只查了新测试目标；`cargo build --offline --all-targets` 覆盖整个工作区。
- 登记给我的门禁阶段，都以 `b/` 为根跑（`bash .claude/gate.d/<阶段> <b>`），日志在 `gates/`：

| 阶段 | 退出码 | 末行原样（截断） |
|---|---|---|
| 33-mutation-tables | 0（第二次跑） | `✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.tsv 1309 条的原文各命中源码一次…`。第一次跑退 1，红在第 990 行锚点，是我造成的，已修（第五节末） |
| 53-format-const-placeholders | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` |
| 74-model-differential | 1 | `✗ 随机历史的测试二进制判红`；`test result: FAILED. 23 passed; 1 failed; 2 ignored`。红的是 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`，原样副本上同名同红（第七节） |
| 89-closeout-row27-preconditions | 77 | 本次未跑：`「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上…`，与这一件无关 |
| 92-layout-checker-sync | 77 | 本次未跑：`! …/b 不是 git 仓，本阶段跳过`。这一件没动格式常量，推断不红，打进主工作区之后由主 agent 那一轮跑 |
| 93-feature-bits | 0 | `✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（…扫了 57 个 .rs…）` |
| 94-checker-implementation-disjoint | 0 | `✓ checker 与实现只共享常量模块 singlefs-format（…checker 闭包 1 个、实现闭包 2 个…）；checker 的 4 份源码零处引 singlefs_core` |

## 七、点 checker 的非层 0 测试目标：改后 vs 原样副本

目标清单 `checker-targets.txt`：`crates/singlefs-harness/tests/` 里引了 `singlefs_checker`、名字不带 layer0 的 44 份，外加 `-p singlefs-checker --lib`。在 `b/` 上逐个跑（`run-checker-targets.sh`，整条经 `run-with-memory-cap.sh 8G`，线程上限 4），汇总在 `targets-after-change/summary.txt`：45 行里 39 行退 0，6 行退 101。6 个红的目标又在原样副本 `c/` 上重跑（主工作区现状拷的，拷时核过 checker 源码与快照相同），结果在 `targets-baseline/`，逐条比每个测试的 ok / FAILED / ignored：

| 目标 | 改后 | 原样 | 比对 |
|---|---|---|---|
| checker_known_bad_images | 37 passed; 2 failed | 38 passed; 1 failed | **不同**：`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 原样绿、改后红（`checker_known_bad_images.rs:2344`：right 集合多了 I-7.13）。归这一件，见第四节第 1 条。另一条红 `an_allocation_generation_past_its_unit_birth_…` 两边同红 |
| second_transaction_step_three_second_instance | 5 passed; 1 failed | 同 | same-per-test |
| second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version | 15 passed; 2 failed | 同 | same-per-test |
| second_transaction_supplement_three_random_history | 23 passed; 1 failed; 2 ignored | 同 | same-per-test |
| second_transaction_step_five_reuse | 13 passed; 1 failed | 同 | same-per-test |
| second_transaction_supplement_three_crash_injection | 7 passed; 3 failed; 2 ignored | 同 | same-per-test |

同结局只比到「每条测试红没红」，没逐条比红的说明文字。那 39 个绿的目标里包括 `checker_narrow_invariants_and_abandoned_roots`、`system_configuration_slots_per_region`、`second_transaction_step_three_formatted_pool` 这些点名 checker 判决的目标。

## 八、受影响的层 0 流与崩溃枚举用例

判定集合改了的：`judge_unit_header` 多拒两种（格式版本、预留位）；走读在指针上多判一格 I-2.4；登记树的码 2 节点宽 0 时多报 I-1.10；系统配置槽多出「带越界值」一类，报 I-7.13 并整池不作保；`IMPLEMENTED_INVARIANTS` 由 46 条变 47 条。

**推断（没跑层 0 与崩溃枚举）：层 0 八条流（`crates/singlefs-harness/tests/` 下名字带 layer0 的 8 份）、`stage-inputs.tsv` 里 7 个 `crash-case:` 用例的违例计数都不变；唯一跟着变的，是每条流的逐条不变量计数多出 I-7.13 这一格。**

- 合法崩溃态上新判定都成立：实现写出的单元格式版本 1、预留位 `skip` 成 0；实现写出的指针 MAC / nonce 恒 0（A3a 的 `pointer.rs` 读者也判它）；条目宽都是字段表宽；系统配置由 mkfs 写，格式版本 1、加密类型 0，几何在界内（A3-checker 报告第七节逐项核过）。
- 撕裂、没落盘的系统配置槽在整槽校验和那一步就被拒，归「不可择」，不会被误判成「带越界值」。走读里的单元都先过了位置条目上的整单元校验和（`read_referenced_unit`），撕裂的单元到不了 `judge_unit_header`。
- **会变的钉值**：层 0 各流按 `IMPLEMENTED_INVARIANTS` 逐条核「评估过的状态数 / 违例状态数」（例如 `first_transaction_step_seven_layer0.rs:160` 起的循环，除例外清单外都要求等于 `every_state`）。I-7.13 在「至少一槽可择」的每个状态上都评估，评估集合与 I-7.6 相同（都在择到系统配置之后才判），所以按推断它落在 `every_state` 这一档、不用进例外清单。打印出来的 `checker_line` 会多一段 `I-7.13=N/0`。如果某条流的判据写死了条数或整行字符串，那里会红，我没逐条查。
- 推翻条件：提交时层 0 快档 / 全量、崩溃注入快档里，任一格违例数变了，或 I-7.13 的评估数不等于 `every_state`。

## 九、补丁与打法

`patch/` 里有：`crates.patch`（4 个文件）、`mutations-append.tsv`（15 行，六段）、`mutations-delete.txt`（1 行）、`report.md`（本文的拷贝）。没有 `mutations-replacements.tsv`。

- 对主工作区现状核过：`git apply --check patch/crates.patch` 退 0；四份被改文件的 sha256 与底座快照相同（复核）。
- 这一件依赖的现有变异行，锚点都落在打补丁之后的源码上：第 990 行靠变量名恢复（第五节末）；A3-checker 那 13 行在副本上复证过。
- `apply-writer-patch.py --dry-run` 的输出见第十一节末。

## 十、没做什么

- 没跑层 0（快档、全量）、崩溃枚举用例、门禁 54 / 55 / 57 / 59 与变异整表、全量 `cargo test`、`check.sh`：这些是重型，归提交时的崩溃验证员与整轮门禁。
- 没改 `checker_known_bad_images.rs`（不在我的文件清单），它那条红留给主 agent（第四节第 1 条）。
- 没写 kb；第四节给的原句交 kb 第六批。
- 门禁 92 号在副本上退 77（副本不是 git 仓），89 号退 77，都等于没判。
- 没走三方对抗，没提交。

## 十一、草稿与删掉的东西

`apply-writer-patch.py <patch> --dry-run` 在主工作区上的原样输出：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1321 行`。主工作区的表在我开工之后又被别的会话追加过几行，所以不是 1300 + 15 − 1。

交回之前删掉了：仓副本 `/tmp/claude-1000/impl-rev-a3-checker-2/b`（连同它的编译目录）、原样副本 `/tmp/claude-1000/impl-rev-a3-checker-2/c`（2.9G）、补丁用的原文件拷贝 `orig/`。`b` 与 `orig` 的大小见交回消息。

草稿目录 `/tmp/claude-1000/impl-rev-a3-checker-2/` 里留着：`report.md`、`progress.md`、`spec.md`、`patch/`、`snapshot-sha256.txt`、各轮日志（`new-tests-1.log`、`final-checks*.log`、`prove-red-run-*.log`、`prove-red-logs*/`、`targets-after-change/`、`targets-baseline/`、`gates/`），以及跑用的脚本（`run-checker-targets.sh`、`run-gates.sh`、`final-checks.sh`、`clippy-checker.sh`）。

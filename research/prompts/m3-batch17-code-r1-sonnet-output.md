# 第 17 批代码轮第一轮：云端正推腿（Sonnet）报告，格 K1、K3

轮名 m3-batch17-code-r1。立场：从用户定案、第 16 批判决行与 kb 条款原文，推到冻结副本 `/tmp/claude-1000/m3-batch17-code-r1/tree/` 里的行，再推到钉它的测试与变异行。引代码写冻结副本里的文件:行号；引 kb 写 kb 文件自己的行号（逐句 `grep -nF` 现查过）。草稿 `/tmp/claude-1000/m3-batch17-code-r1-sonnet/`（只有一个数变异锚点命中数的小脚本 `count.py`，不建模型目录）。

## 0 先核的事实（命令与原样输出）

冻结副本四份文件的 sha256 与快照文件逐行比：

```
$ diff <(sha256sum crates/mutations.tsv crates/singlefs-core/src/mount.rs crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs) research/prompts/m3-batch17-code-r1-snapshot/crates-sha256.txt && echo SAME
SAME
```

背景材料里的两份补丁（`_m3-batch17-code-r1-diff.md` 第 6–477 行 P、第 492–695 行 Q）在冻结副本拷贝上反向打（`patch -R -p1 --dry-run`），上下文全部对得上：

```
P reverse dry-run exit 0
Q reverse dry-run exit 0
```

变异追加行：冻结副本 `crates/mutations.tsv` 第 2472–2475 行与背景材料 diff 第 483–486 行逐字相同（`diff` 无输出，回 `P rows frozen 2472-2475 = diff.md 483-486 identical`）；第 2476–2477 行与 diff 第 701–702 行逐字相同（回 `Q rows 2476-2477 identical`）。

六行变异（第 2472–2477 行）的「原文」在被改源文件里各恰好命中几次（`count.py` 原样输出，每行 6 列）：

```
2472 6
  file crates/singlefs-core/src/mount.rs orig count 1 rep count 0 test after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_m
2473 6
  file crates/singlefs-core/src/mount.rs orig count 1 rep count 1 test after_a_crashed_unmount_of_a_four_gibibyte_pool_the_next_wri
2474 6
  file crates/singlefs-core/src/mount.rs orig count 1 rep count 0 test after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_
2475 6
  file crates/singlefs-core/src/mount.rs orig count 1 rep count 2 test after_a_crashed_unmount_of_a_four_gibibyte_pool_remounted_on
2476 6
  file crates/singlefs-core/src/transaction.rs orig count 1 rep count 0 test a_session_unlink_given_a_time_earlier_than_the_previous_vers
2477 6
  file crates/singlefs-core/src/transaction.rs orig count 1 rep count 0 test the_batches_of_a_mount_continuation_after_an_intent_given_a_time_earlier_than_the_previous_version_copy_that_earlier_time_on_disk
```

六行原文都恰好命中一次（`rep count` 只说明替换文在源里本来出现几次，不影响锚点）。

**复核不了（没编译、没跑变异）**：定义不让子 agent 在没有明写的情况下编译，派发提示也没点名要跑。实现员 P、Q 的「prove-red 全抓到」（`/tmp/claude-1000/m3-batch17/p/prove-red.out` 末行 `✓ 点名 4 条：跑了 4 条，跳过 0 条，跑的都抓到了`；`q/prove-red.out` 末行 `✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了`）与合入验证 harness 档 151 个二进制全绿（`test.log` 里 `test result` 共 151 行），只当线索。下面每个「红不红」都是读代码推的，写明「推的」。

## 1 K1　条款对实现

### 1.1 N1　甲：m 取不到那一支（`mount.rs` `rebuilt_allocator`）

条款（D16 已定项 1「正常卸载」那一行，一整行；这一轮要比的是下面三句，都在 `.claude/kb/decisions/16-发布语义.md` 第 41 行）：

- 「重建分配器时当场可用的只到释放代不高于 max(min(F_生效, 「每块盘上最新有效根的 txg」最小者), 环里最旧有效根) 的槽」（`.claude/kb/decisions/16-发布语义.md:41`）
- 「某块盘一条有效根都读不出时只到环里最旧有效根（挂载时读不出的槽按它能住的最小 txg 并进这个最小值，与回收门槛同一个读数），其余扣到这次挂载的暖机在每块盘上都落了根（用户 2026-10-04 定，零轮）」（`.claude/kb/decisions/16-发布语义.md:41`）
- 「有一块盘一条有效根都读不出时这一截为空、整截扣到生效，这一遍读不出、自证不过的槽当没有根」（`.claude/kb/decisions/16-发布语义.md:41`，这是卸载那一格）

代码今天（冻结副本）三种情形各是什么：

| 情形 | 代码 | 当场放到 | 条款 | 判定 |
|---|---|---|---|---|
| m 取到 | 「Some(newest_valid_root_on_every_device) => Some(reclaim_floor(」（`crates/singlefs-core/src/mount.rs:1880`）后接第 1881 行「effective_floor.min(newest_valid_root_on_every_device),」与第 1882 行 `oldest_valid_root_in_the_window` | max(min(F_生效, m), o)，o 取不到时 F 那一项自己 | 第 41 行第一句 | 一致 |
| m 取不到、o 取得到 | 「None => oldest_valid_root_in_the_window,」（`crates/singlefs-core/src/mount.rs:1884`），第 1886 行「if let Some(released_immediately_up_to) = floor_released_immediately {」进去回收（Immediately） | o | 第 41 行第二句 | 一致（措辞差见下） |
| m 取不到、o 也取不到 | 同一行 `None`，第 1886 行 `if let` 不进，当场一个都不放 | 空 | 条款没写这一格 | 规则没说（代码读法是「只到 o」那一截为空） |
| 扣住那一遍 | 「ReclaimedReuse::HeldUntilFloorTakesEffect,」（`crates/singlefs-core/src/mount.rs:1891`）前一行 `floor_of_the_reclaim`，门槛是第 1873 行「let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_window);」 | F 与 o 的较大者，减去已当场放开的 | 第 41 行第二句后半「其余扣到……」 | 一致 |

几处要点：

- 当场放开的上界恒不高于扣住的上界：m 取到时 max(min(F,m),o) ≤ max(F,o)；m 取不到时 o ≤ max(F,o)。所以先 Immediately 后 Held 两遍，分配器用 `reclaimed` 集合去重（`crates/singlefs-core/src/allocator.rs:1741` 「if !self.reclaimed.insert(key) {」），第二遍只收第一遍没收的。这是读代码推的，没跑。
- 「窗口里最旧有效根与回收门槛是不是同一个读数」：是。`oldest_valid_root_in_the_window` 只在第 1847 行定义一次（「let oldest_valid_root_in_the_window = roots」），第 1873、1882、1884 行三处用同一个变量；挂载时读不出的槽按 `lowest_txg_it_can_hold` 并进它（第 1851–1855 行）。与条款「与回收门槛同一个读数」一致。
- 卸载那一格没有被碰：卸载里 m 取不到的处置在第 3095 行「None => Vec::new(),」，后接第 3097–3100 行把整截按 Held 收；P 补丁四个 hunk 的 `@@` 头在原文件上是 `-1737`、`-1860`、`-1874`、`-2471`（背景材料 diff 第 10、21、33、53 行），改后文件上是 1737、1860、1877、2476 行附近，都不在第 3085–3101 行；反向打补丁无冲突已核（第 0 节）。与条款「整截扣到生效」一致。
- 放开只发生在暖机之后：第 4756 行「allocator.release_reclaim_holds();」在暖机循环（第 4733–4749 行）之后。`establish_instance` 里调抬 F 的第一处在第 4806 行（`push_floor_raises_after_the_row_publish`），在 4756 之后；`rebuilt_allocator` 只被第 5628 行调用、`establish_instance` 只被第 5648 行调用（`grep -n` 现查）。所以走到 4756 时分配器上的扣住位全是重建那一处置的，与第 4750–4755 行注释的说法一致。另一处 `release_reclaim_holds` 在第 3212 行，属抬 F 那一串。

**N1 里条款与代码不一样的一处，两边整行并排（冲突，归属不在第 17 批）**

- 条款（kb，`.claude/kb/decisions/16-发布语义.md` 第 34 行，是「环里最旧有效根」那一格的后半；该格列的「同取这个值的六处」里第一处是分配器的回收门槛含抬 F 那一遍）：「六处都不再按一圈窗口滤根，窗口之外的有效根（崩在记录之后、根槽之前那一形留在槽里的旧根）也在候选集里、也压着门槛」（`.claude/kb/decisions/16-发布语义.md:34`）。
- 代码（冻结副本）：重建分配器算 `oldest_valid_root_in_the_window`（第 1847 行起）时仍按一圈窗口滤根，那一行是「.filter(|root| !is_abandoned(root) && is_inside_the_window(root.checkpoint_txg))」（`crates/singlefs-core/src/mount.rs:1849`）。

- 条款这一句今天是 kb 工作区里的改动（`git show HEAD:` 那一份里没有「同取这个值的六处」，工作区那份有），对应的用户定案在 `.claude/kb/decisions/16-发布语义.md:73` 的「用户定案 2026-10-04（Q1 RGall、Q2 下一轮就改），原话在变更史」。`.claude/kb/decisions/16-发布语义.md:79` 自己把这层关系写明了：「再挂时某块盘一条有效根都读不出只放到最旧有效根（该判决写作一圈窗口里最旧有效根，随回收门槛退回环里最旧有效根）」。
- 代码里 RGall 还没落：`grep -rn RGall crates/`（冻结副本）零命中；`records/2026-10-01-删除与并发事务.md:769` 写根环线的 core 修订「还没派」，要动 `mount.rs`。所以这条冲突是「RGall 没进代码」，不是第 17 批 N1 引入的，也不是 N1 的错：第 17 批写的是用户弹窗选的「只放到窗口里最旧有效根」，注释、用例文档都跟着用户的原话写「一圈窗口里最旧有效根」。
- 对 N1 的影响：RGall 落地时 `oldest_valid_root_in_the_window` 要改成不滤窗口（取值只会更小或相等），m 取到、m 取不到两支的当场放开都会少放或不变；N1 的结构（`None` 那一支取 o、`Some` 那一支取 max(min(F,m), o)）不用变。N2 的用例（盘 1 整环读坏）里 o 本来就被读不出的槽的 `lowest_txg_it_can_hold` 拉到窗口下沿（第 1851–1855 行），RGall 前后同向。这一句是推的，没跑。
- 这一格我记「冲突」，交主 agent 判是否并进 RGall 那条线；不替它采纳。

**判定**：m 取到、m 取不到而 o 取到、扣住那一遍、卸载未碰、放开时点，五格与 D16 第 41 行一致；m 与 o 都取不到那一格条款没写（规则没说），代码读成「空」，主 agent 在 `records/2026-10-01-删除与并发事务.md:701` 已判这个读法零轮；o 的定义（窗口滤不滤）与 D16 第 34 行冲突，归 RGall。

**什么现象会推翻「一致」**：(a) 找到一个起点使第 1873 行的 `floor_of_the_reclaim` 与第 1882/1884 行的 o 取到不同的值（两处各自重算了）；(b) 卸载第 3095 行被改成不是整截扣到生效；(c) 第 4756 行之前出现置扣住位的抬 F 调用；(d) 当场放开的槽里有释放代高于 o（m 取不到那一支）的。**什么现象会推翻「o 的定义冲突」**：RGall 的补丁已在冻结副本以外的主工作区落地，且 `mount.rs:1849` 那一行在其中已去掉窗口滤。

### 1.2 N2　再挂期间盘 1 根环每槽读错的读回用例（`the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs`）

- 参数与题面逐项对：264 槽（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:499` 「264,」）、覆盖写 6（同文件第 494 行 `overwrites_before_the_file_grows = 6`）、4 单元（第 495 行）、卸载刀 15（第 548 行「15,」）、再挂落下 6 个写（第 544 行「let remount_writes_landed = 6;」）、丢盘 0（`DeviceIdentity(0)` 在调用实参里）、盘 1 根环每槽每次读都报错（第 550 行「RootRingDuringTheRemount::EverySlotOfTheDeviceFailsEveryRead(DeviceIdentity(1)),」）。
- 判的是用户看得到的读回结局：末项调的是第 353 行的 `assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount`，它比的是第 370 行「survivor.after_the_crashes, survivor.before_the_unmount,」；两边都是第 70 行 `whole_read_of_the_live_files`（只读挂载、对每个活文件 `open_file` 加 `read_at` 整读，`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:71-84`）。池级 checker 没用（符合背景材料第五节）。
- 防空转的两项：第 555 行「survivor.reads_failed_during_the_remount > 0,」（读注错真拦下过）、第 559 行「survivor.write_calls_of_the_remount > remount_writes_landed,」（再挂走到了第 7 个写，不是在任何写之前被拒）。
- 注错只罩再挂那一次挂载：读计划只装在 `remount_devices`（第 302–315 行），之后的恢复与整读用的是镜像拷贝（第 324–336 行 `after_the_remount`），不经读注错层——与用例文档「挂载交回之后的恢复与整读不经这一层」一致。
- 红不红（推的，没跑）：变异 2472 把第 1884 行换成 `None => Some(floor_of_the_reclaim),`，就是第 16 批以前的「照原门槛整截当场放开」（第 16 批 `None => floor_of_the_reclaim`）；这一用例里盘 1 整环读坏、m 取不到，整截放开时写行那次的单元写会进盘 1 最新的根引用的槽，剩下盘 1 读不出。用例文档（第 487–489 行）写了攻方原型上量过 `MappingStillUnreadable { slot: SlotNumber(1410) }`，实现员 P 自报改前红在同一个错；我没复跑，当线索。
- **判定**：一致（判的是读回结局，题面参数逐项对上）。**推翻条件**：用例在 `None => oldest_valid_root_in_the_window` 这一支被改成 `None => Some(floor_of_the_reclaim)` 之后仍然绿（变异没咬住）；或整读比较的两边其实一个读不出另一个也读不出（`before_the_unmount` 在第 365 行已断言 `is_ok()`，已防这一条）。

### 1.3 N3　`newest_valid_root_txg_on_every_device` 文档

- 文档第 2479 行：「有一块盘上一条有效根都没读出来：交回 `None`（不按 txg 0 去比，说清是哪一格），两格各自处置：」（`crates/singlefs-core/src/mount.rs:2479`）；第 2480 行：「卸载那一格这一串回收的槽整截扣到生效；重建那一格当场只放到一圈窗口里最旧有效根」（`crates/singlefs-core/src/mount.rs:2480`，同行后接括号说明与回收门槛同一个读数）；第 2481 行：「其余扣到这次挂载的暖机落满每块盘」（`crates/singlefs-core/src/mount.rs:2481`，后接出处）。
- 卸载那一格：第 3095 行「None => Vec::new(),」后接第 3097–3100 行 Held 收，整截扣到生效，对得上。重建那一格：第 1884 行 `None => oldest_valid_root_in_the_window,`，对得上。
- 文档没写 o 也取不到的子格；那一格写在 `crates/singlefs-core/src/mount.rs:1866`「这一遍一条有效根都没读出、窗口里最旧有效根也取不到时」（该行后半是「只到窗口里最旧有效根」的那一截是空的……，带 「」 引号，没法整行引进来）。两处合起来没矛盾。
- 上一轮判决第二节 N3 那一行：「新理由推得通；冻结副本 mount.rs 第 2474 行」（`research/prompts/m3-batch16-code-r1-main-verification.md:35`），行里接着写的是「`None` 整截扣到生效」只对卸载那一格为真、那一句随第四节的选择一起改（该行带嵌套引号，这里不整行引）。现在已拆成卸载、重建两格，就是改的这一句。
- **判定**：一致。**推翻条件**：卸载或重建任一格的代码行为与上面对应的文档句不符（例如第 3095 行换成当场放开而文档仍写整截扣到生效）。

### 1.4 N4　暖机之后放开那一句，用例按扣住位计数钉

- 钉的东西：第 596–597 行逐盘比 `(device_map.isolated_slots(), device_map.free_slots())` 与 `(0, device_map.unblocked_slots())`（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:596`「(device_map.isolated_slots(), device_map.free_slots()),」，下一行是「(0, device_map.unblocked_slots()),」）。`unblocked_slots` 是「已分配、影子账隔离、抬 F 扣住」三种位都没置的槽数，`free_slots` 是没分配的槽数（`crates/singlefs-core/src/allocator.rs:433` 起的 `is_unblocked_at_index` 与 `set_blocking_bit` 增量维护），影子账关着（第 590 行「ShadowLedger::Off,」）隔离恒 0，所以相等就等于一个扣住位都没留。
- 造法：4 GiB 盘，覆盖写、崩了再挂、再覆盖写，卸载只落下第 1 个写（第 584 行「let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);」），两块盘都在、可写挂载做成（第 586 行起），不再崩。
- 判的**不是**用户看得到的读回结局，是分配器内部状态。用例自己写明了这一点：「用户看得到的结局没找到（推的，没量过）」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:572`）。
- 够不够：
  - 钉住「第 4756 行那一句被删」够：变异 2473 删的就是那一句，扣住位留下，第 597 行红（推的；实现员自报每盘留 14 个扣住位，没复跑）。
  - 钉不住「放开挪到暖机之前」（背景材料第一节已排除，进第 18 批）。
  - 用户看得到的结局有一条可走的线，用例没钉：扣住位挡着提交内生块的落点，`crates/singlefs-core/src/admission.rs:741`「slots_for_commit_generated_units: device_map.unblocked_slots(),」把它读进落点空间那一判；扣着的槽没放开时这一判会比放开后小，落点取不到就先推抬 F 那一串（D16 第 42 行「准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判」），抬 F 那一串做完才放开（第 3212 行 `release_reclaim_holds`）。可观察的是「挂载交回之后第一次写要不要多推一串空发布、F_生效 被多抬多少」。这一条是推的，我没造，K2（攻方腿）能不能造出来不归我。
- 条款侧：D16 第 41 行只写「其余扣到这次挂载的暖机在每块盘上都落了根」，没有写「挂载交回时无扣住位」这一句，用例钉的是从那句推出的读法。
- **判定**：钉住了「那一句在不在」（红在计数上），没钉用户看得到的结局；条款侧对这个读法没有逐字的句子（规则没说）。**推翻条件**：造出一个让「那一句被删」而第 597 行仍绿的挂载（例如重建时没有任何槽被扣住：释放代全不高于 min(F,m)），则这条用例在那种起点上分辨不了；或造出扣住位残留而用户读回、写入全无差别且连多推一串空发布都没有，则「用户看得到的结局没有」成立。

### 1.5 N5　变异：当场放开不取 min(F_生效, m)，补点 4 GiB 用例

- 改坏的是名字说的那一处：变异第 2475 行把「effective_floor.min(newest_valid_root_on_every_device),」换成 `effective_floor,`；原文在冻结副本里恰在 `crates/singlefs-core/src/mount.rs:1881`（上面 0 节已核命中一次）。换完 m 取到那一支的门槛变成 max(F, o)，即第 16 批以前的「照原门槛整截当场放开」，名字写的「当场放开不取 min(F_生效, m)」对得上。
- 点名的用例：`after_a_crashed_unmount_of_a_four_gibibyte_pool_remounted_once_the_next_writable_mount_crashing_after_its_thirty_third_write_keeps_what_device_one_reads`（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:458`）。它的文档（第 453–456 行）写着整截放开时盘 1 只读挂载走 extent 树走不下去（攻方原型上量过 `ExtentTreeWalk(MappingStillUnreadable { slot: SlotNumber(50253) })`）。
- 与第 16 批第 2380 行同原文同替换（只换点名的用例）：`awk -F'\t'` 现取，第 2475 行与第 2380 行的第 2–5 列（文件、原文、替换文、命令）逐字相同，只有第 6 列（点名的用例）不同。这样同一刀有两行、分别点 h2 窄盘与 4 GiB 盘，上一轮判决第二节「N2 形 R 两条读回」那一行要的「变异行补点 4 GiB 那一条」就是这个。
- 红不红：推的（用例的前提是 m 低于 F_生效 的挂载，卸载只落第 1 个写就是为了造出来），没跑；实现员自报该行抓到。**判定**：一致。**推翻条件**：这一刀下点名用例绿，或第 2475 行原文在源里命中数不是 1（现查为 1）。

### 1.6 N6　Q-F：两条用例各加「给的时间早于上一版」一形；变异两行

- 条款侧：D8 第 188 行「| 64 | atime / mtime / ctime 秒，各 8（unlink 的意图那次三个共写调用方给的一个值，各批照抄） | 24 |」（`.claude/kb/decisions/08-核心索引结构.md:188`）；第 195 行「**unlink 的意图那次**写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值」（`.claude/kb/decisions/08-核心索引结构.md:195`）；第 470 行「意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间」（`.claude/kb/decisions/08-核心索引结构.md:470`）。三处都没写时间只进不退，也没写「取较大者」，所以「给早于上一版的时间就写那个较早的」是字面读法。
- 代码侧：意图写调用方的值，在 `crates/singlefs-core/src/transaction.rs:5882` 的 `write_time_seconds,`（上一行 5881 是「change_count: change_count_of_the_intent,」）；各批抄上一版这条记录的值，在 `crates/singlefs-core/src/transaction.rs:5945` 的「write_time_seconds: record.write_time_seconds,」。
- 变异第 2476、2477 行是同一刀：把 5881–5882 两行换成带 `.max(record.write_time_seconds)` 的写法（原文在 transaction.rs 命中一次，已核；`record` 在第 5864–5870 行的作用域里）。名字说的「意图取给的时间与上一版里较大的那个」就是这一处。两行点不同的用例：会话那一形（第 668 行）与续做那一形（第 731 行）。
- 用例读的是盘：`time_seconds_of_the_first_file_on_each_device`（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:507`）读第 539、540、541 行三个偏移 64、72、80 各 8 字节，按盘各一组，与 D8 第 188 行的字段表逐项对。
- 红不红（推的，没跑）：「早于上一版」那一形里 given = 上一版 − 600，`.max` 把意图写成上一版的值，两条新用例（第 668、731 行）比 `[given; 3]` 全红；「晚于上一版」两条（第 649、714 行）给的比上一版大，`.max` 取到 given，仍绿——与用例文档「这一条红、晚于上一版那一条照旧绿」一致。反过来，`.min` 一类错实现被晚于一形抓。
- 「给的时间」的造法：`Earlier` 用 `checked_sub(600)`，前提是测试池写的固定时间大于 600（第 595–596 行 `expect`）；实现员 Q 自报读回 1788000060、期望 1787999460，说明前提成立。
- **判定**：一致。**推翻条件**：这一刀下第 668 或 731 行的用例绿；或四条用例给的时间在上一版的哪一边写反（`Later`/`Earlier` 对调）。「有没有错实现四条都过」归 K2，不在这里答。

## 2 K3　被判的文件名单（`research/prompts/m3-batch17-code-r1-snapshot/crates-sha256.txt` 四份，冻结副本里 sha256 逐行相同，见第 0 节）

| 文件（冻结副本） | 行数 | 这一轮的改动落在哪 | 判到的格 |
|---|---|---|---|
| `crates/singlefs-core/src/mount.rs` | 6861 | P 补丁四个 hunk：函数文档第 1740–1741 行；`rebuilt_allocator` 的注释与分支第 1857–1888 行；`newest_valid_root_txg_on_every_device` 文档第 2470–2482 行；暖机之后放开那一句第 4750–4756 行（这一句不在 P 补丁的 hunk 里，是第 16 批以前就在的，N4 的变异 2473 锚在它上面，所以连带判了）。其余不是这一轮的改动 | N1（1.1）、N3（1.3）、N4 的锚（1.4） |
| `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | 602 | 辅助函数（第 106–350 行，读注错计划、两层注错盘、拆出 `image_after_an_unmount_crashing_after`）、旧两条用例调用处加 `ReadAsUsual`（第 446、474 行）、新增两条用例（第 491、574 行） | N2（1.2）、N4（1.4）；旧两条用例的调用处改动见下 |
| `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | 1194 | Q 补丁一个 hunk：第 576–743 行（常量、`GivenTimeAgainstThePreviousVersion`、两个取数函数、四条用例） | N6（1.6） |
| `crates/mutations.tsv` | 2485 | 第 2472–2477 行（P 四行、Q 两行）。第 2478–2485 行是形式化线 25-1、25-3a 的 8 行（O1 跨叶容器 3 行、E171R1 5 行），按背景材料第一节「这一轮不判的」不判 | N1（行 2472）、N4（行 2473、2474）、N5（行 2475）、N6（行 2476、2477）；锚点命中数第 0 节 |

- 第 2472 行（N1）：名字说「m 取不到照原门槛整截当场放开、不只放到一圈窗口里最旧有效根」，替换文 `None => Some(floor_of_the_reclaim),` 把第 1884 行换回第 16 批以前的取值，名实相符；点名的用例是 N2 那一条（第 491 行）。
- 第 2473 行（N4）：删第 4756 行那一句，名实相符；点名第 574 行那一条。
- 第 2474 行（N4 同批，P 报告里算「扣住那一遍回收」）：把第 1889–1892 行的 `HeldUntilFloorTakesEffect` 换成 `Immediately`，等于 m 取到时也整截（到 max(F,o)）当场放开；点名的是旧的 h2 窄盘用例（第 381 行）。名实相符。
- 旧两条用例调用处的改动：第 446、474 行各加一个 `RootRingDuringTheRemount::ReadAsUsual` 实参；`devices_failing_every_write_from` 仍由第 253 行（卸载那一次）用，原来再挂那一次的单层写注错盘改成「外层读计划（未装弹，只数）、里层写计划」的两层（第 302–315 行），写计划的参数（`remount_writes_landed + 1`）没变。读起来语义不变（推的，没编译没跑；合入验证里这两条用例随 151 个二进制一起绿，是线索）。
- 形式化线 25-1/25-3a 的 8 行变异、`mount.rs` 里与 N1–N6 无关的差异（冻结副本对 HEAD 有 35 处 diff hunk，来自第 14–16 批等已合入改动）、`unlink_mount_continuation_and_the_emptied_version.rs` 第 165 行附近那一处（对 HEAD 共 2 个 hunk，另一个是 Q 的），都不在这一轮 diff 里，不判。
- **判定**：四份各有一格判到，逐份见上。**推翻条件**：背景材料 diff 以外还有对这四份的改动落在 N1–N6 之外（我用反向打补丁核过 P、Q 两份补丁都能干净反向打下，说明背景材料里的补丁与冻结副本一致，但「补丁之外是否还有这一轮的别的改动」只能由 `merge-and-verify.sh` 的四份补丁清单担保，我没有第二条独立路径）。

## 3 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-N1 m 取到 | 一致 | 第 1880–1883 行 max(min(F,m), o) 与 D16 第 41 行第一句相同 |
| K1-N1 m 取不到、o 取到 | 一致 | 第 1884 行只放到 o，扣住那一遍第 1889–1892 行，与第 41 行第二句相同 |
| K1-N1 m 与 o 都取不到 | 规则没说 | 条款没写这一格；代码当场一个都不放（第 1886 行 `if let` 不进），主 agent 已判这个读法零轮 |
| K1-N1 窗口里最旧有效根与回收门槛同一个读数 | 一致 | 第 1847 行一个变量，第 1873、1882、1884 行三处用 |
| K1-N1 卸载那一格有没有被碰 | 一致（未碰） | 第 3095 行整截扣到生效不变；P 补丁四个 hunk 都不在卸载段 |
| K1-N1 o 的定义（窗口滤不滤） | 冲突 | D16 第 34 行写六处不再按一圈窗口滤根，`mount.rs:1849` 仍滤；归 RGall 那条线，不是第 17 批引入 |
| K1-N2 | 一致 | 判的是只读挂载加整读的读回结局，参数逐项对上题面，带防空转两项 |
| K1-N3 | 一致 | 文档两格与第 3095、1884 行对得上；o 也取不到的子格写在第 1866 行注释 |
| K1-N4 | 钉住「那一句在不在」；没钉用户看得到的结局；条款侧规则没说 | 钉的是扣住位计数；`admission.rs:741` 给出一条可走的用户可见线（多推一串抬 F），用例没钉，我没造 |
| K1-N5 | 一致 | 变异改的就是 `min(F,m)` 那一处（第 1881 行），点名 4 GiB 用例；红不红是推的 |
| K1-N6 | 一致 | 变异改的是意图写时间那处（`transaction.rs:5881-5882`），早于上一版两条用例会红、晚于上一版两条仍绿；红不红是推的 |
| K3 mount.rs | 判到 | N1、N3，连带 N4 的锚 |
| K3 the_mount_after…reads.rs | 判到 | N2、N4，旧两条用例调用处改动语义不变（推的） |
| K3 unlink_mount…version.rs | 判到 | N6 |
| K3 mutations.tsv | 判到 | 第 2472–2477 行六行锚点都命中一次，名实相符；第 2478–2485 行不判 |

## 4 没做什么

- 没编译、没跑任何测试或变异：所有「红不红」是读代码推的，实现员 P、Q 的 prove-red 自报与合入验证的 151 个二进制绿只当线索。
- 不判 K2（攻方的格）：m 与 o 都取不到那一格造不造得出、窄盘被拒段数、第二次抽样各格、有没有错实现让四条 Q-F 用例照样过，都不归这条腿。
- 不替主 agent 采纳或出判决：「o 的定义冲突」是否并进 RGall、N4 够不够，交主 agent。
- 没读禁读清单里的两处（攻方腿的报告与模型目录）。
- 没建模型目录（没有需要入库的脚本；`count.py` 只在草稿目录里）。
- 第 16 批判决第三节第 5 条「四个固定点单元」旧说法不在 N1–N6 里，冻结副本里仍有 9 份文件含这个词（`grep -rl '四个固定点单元' crates/`），`records/2026-10-01-删除与并发事务.md:678` 写了挪到第 18 批，这里不判。
- 形式化线 25-1、25-3a 与根环线的改动不判（背景材料第一节）。

## 5 补充的现查（命令与原样输出）

```
$ cd /tmp/claude-1000/m3-batch17-code-r1/tree && grep -rn 'RGall' crates | wc -l
0
$ grep -n 'push_floor_raises_after_the_row_publish(\|rebuilt_allocator(\|establish_instance(' crates/singlefs-core/src/mount.rs | grep -v '///\|//'
4806:            push_floor_raises_after_the_row_publish(
5628:    let rebuilt = rebuilt_allocator(
5648:    establish_instance(
```

cite-check 末行（交回前原样）：`✓ 核了 20 处引文，对上 20 处，没判 0 处（逐处列在上面）`

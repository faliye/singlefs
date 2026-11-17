# m4-close-fixes-code-r2：里程碑四收口，代码轮第二轮，攻 r1 判决第四节那一批修补（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

`research/prompts/m4-close-fixes-code-r1-main-verification.md` 判决第四节五条（四处读计数、三类主动修复、查不到 key 不改记没坏、T3 改名与 newtype、GPU 下界判法补流）已经打进主工作区，被攻过零轮。判决第五节定这一题开第二轮，攻这批改动。

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。不重复 r1 已攻的角度（r1 判决第二节那些格），也不重攻 `m4-close-code` r1–r3 已判过的改动。

改动全文：`research/prompts/_m4-close-fixes-code-r2-diff.md`（`crates/singlefs-core` 7 个文件块）。测试文件只当上下文读：`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs`、`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs`。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 被判的产品代码：`crates/singlefs-core/src/` 的 `allocator.rs`、`compaction_plan.rs`、`mount.rs`、`mounted_read.rs`、`mounted_session.rs`、`recovery.rs`、`transaction.rs`（开工快照 `research/prompts/m4-close-fixes-code-r2-snapshot/crates-sha256.txt`）。这几份里不是这批的块（别的会话的改动，含 `mount.rs` 删开关 `FirstTxgOfTheNewInstance`）当上下文读，腿只攻 diff 里有的块。
- 实现员报告：`~/.local/share/singlefs/work/m4-close-fixes-writer/result.md`（五条与「停下交主 agent 的设计问题」七条）、`m4-close-fixes-writer-3/result.md`（两条用例的成因）、`m4-close-fixes-writer/result-checker-on-restored-bytes.md`（放回字节再判）。
- 验证到的数（量的）：主工作区带 gpu：`unit_read_…` 11 条过、`crash_points_…` 1 条过、`states_judged_by_the_gpu_here=182`（任务 11006）；GPU 新流缩域到 17 过（任务 11002）；验证包不带特性判绿 849 条（任务 10855 第 4 次）；变异在副本上证红 19 行（任务 10758、10662、10800、10801），带 `--item` 的门禁任务 11017 要记 45.6 GiB、等整台腾空，取消，整表归提交时；余下 5 行证红任务另交，判决写下时还没出结局。全量域 GPU 与不指定范围的门禁都没跑。
- 压着的条款：`.claude/kb/decisions/04-校验和位置.md` D4（校验和位置） 已定项 4；`.claude/kb/checks-owed.md` C946（只坏一份的读没有计数也没有搬迁修复）（正文过期，随这批写回）；用户 2026-11-09 定案（r1 判决第三节）：「别的读」按字面全接上，三类豁免现在补主动修复。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. **映射 key 被换、封得对的盘**：实现员读码（`crates/singlefs-core/src/code_two_tree.rs` 第 656–661 行）与用例 `a_disk_with_a_key_replaced_in_the_mapping_is_mounted_and_the_mount_puts_the_key_back`：重建只数条目数（`mapping_entry_count_is_one_per_mapped_unit`）就收下，第一次发布按指针重算、把 key 盖回，不计不拒。映射树是派生态还是权威态，这一形该不该拒，条款有没有写（D21（权威态与派生态的分界））。只读路径上它走不走得到错读。
2. 实现员七条设计问题（报告「停下交主 agent 的设计问题」）逐条判：① 没被读到的坏副本被换下时不核；② 攒批会话读不到待发写；③ 账的口径「一组位置条目等下一次改动的那一程只计一次」，第四种口径；④ 释放之前读盘核读到的单元就是这次换下的，「下一次改动之后搬」落空，发布在读盘核之后才被拒那一形没有用例；⑤ `prepare_the_version_publish_relocating_units` 里 deadlist 条目核不过提前返回时已读到的只坏一份没进账；⑥ `Relocated.mapping_keys` 含本来就重写的节点；⑦ 实例表读不出整批拒（新成员 `RelocationBatchRefused::InstanceTableOfTheVersionUnreadable`）。
3. 主动修复只试一次、被空间拒推抬 F 再试的路径：账什么时候清（发成、被拒、挂载结束），崩溃点上下一次挂载账还在不在、会不会把同一组搬两次。
4. 读实例表、释放前读盘核、搬留存读源各接上计数之后：有没有路径把同一份读重复计入，或读失败时账与「进这次挂载别的读」那个数不一致；测试断言的 2 次（推抬 F 沿链读实例表一次、搬留存单元读源一次）对不对。
5. 产生结论的方法：两个新测试目标改成「放回弄坏的字节再判池级 checker」（`assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back`、`put_the_damaged_bytes_back_on_the_baseline`），`version_references_the_location` 钉现行版本不再引用坏落点：这一组判法够不够替代放回之前看不到的东西，树表那条断言有没有判别力（报告写了偏弱）。
6. 从未被看过的地方：T3 改名之后的 `units_exempt_from_the_mapping_replaced` 成员在别的调用点的语义；newtype 改动。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」原样：

- crates/singlefs-core/src/transaction.rs:10935 另发那一批两样都空不发：唯一调用点 mounted_session.rs 的 publish_the_relocation_batch_once 只由 relocate_the_units_read_with_one_copy_failing_after_the_change 经推抬 F 那一层进来，那个函数在 planned 与 repair 都空时已交回 NotNeeded

没有新加 `todo!`、`unreachable!`，没有以 `Undecided`、`Unsupported`、`OutsideTheFirstVersion` 收尾的成员。

### 检查单：类型与名字

新加的名字由腿看 diff；`inode_record_is_not_below_the_separator_key_and_the_container` 三个裸 u64 已换 newtype（`InodeNumber`、`SeparatorKeyOfAnInodeLeaf`、`ContainerNumberOfAnInodeLeaf`）；`expect` 依赖的不变量逐处看。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；腿现跑 `grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r2-diff.md`。

### 检查单：不新建判定库

本轮改动没有这一类：diff 全在 `crates/singlefs-core/src/`，产品代码不写判定。

### 检查单：所有任务多机可用

产品代码没有这一类；两个新测试目标由腿看读不读本机状态（报告称盘都在进程里造）。

### 检查单：测试优先多线程

`unit_read_…` 各条单池单场景，经 GPU 判时带 `--test-threads 1`（`.claude/rules/verification.md`）；`crash_points_…` 状态交给定状态表判。

### 检查单：带 --item 的命令与项

补丁行改动后共 28 行；带 `--item` 的门禁任务 11017 取消；GPU 全量域那 1 行（`m4-close-fixes GPU 判退坏叶流：…`）只在副本上缩域证红（任务 10800），整表归提交时。崩溃放量入口 `scripts/crash/crash-point-replay.sh --item` 带上全包测试函数，这批改走门禁 `--check crash-case-replay --crash-cases-only --item crash-case:compaction-relocating-a-block-shared-with-a-snapshot-stream`（任务 11004）。

### 检查单：新崩溃流多罩了什么

`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping` 182 个状态：主动修复那两批（映射树叶路径重写、实例表整链重写）的写与它们的前缀，已有的流都不发这两批。GPU 新流（分隔 key 改高、打开读通）多罩下界判法，缩域到 17。腿核这两样是不是真比已有的多。

## 五、格

1. 代码做的是不是条款说的：D4（校验和位置） 已定项 4 与用户定案。
2. 第三节六条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。

# 崩溃一致性验证：m2-wave2（54 / 55 / 57 / 59）

改动范围：`research/prompts/m2-wave2-crash-verifier-scope.txt`（已核对与当前 `git diff HEAD --stat -- crates litmus` + 2 个未跟踪文件一致，见下方指纹）。

## 环境旁证（不单独下判断）

```
$ command -v herd7; echo exit=$?
exit=1
$ ls -l /dev/kvm
crw-rw---- 1 root kvm 10, 232 Sep 17 19:53 /dev/kvm
$ command -v qemu-system-x86_64; echo exit=$?
/usr/bin/qemu-system-x86_64
exit=0
```

## 一张表

| 阶段 | 退出码 | 耗时（UTC） | 判定 |
|---|---|---|---|
| 54 层 0 崩溃点重放（两条流全量） | 0 | 3:50:49（2026-09-18 04:22:54 → 08:13:43） | ✓，两条流各一行，见下 |
| 55 QEMU 真设备第一个事务 | 0 | 17s（08:20:47 → 08:21:04） | ✓，见下 |
| 57 herd7 内存序 | 0 | 1s（08:21:17 → 08:21:18） | ✓，见下 |
| 59 crates 变异表复跑 | 1 | <1s（08:21:46，锚点腐化，预扫阶段即中止，没进入任何一条变异的真实复跑） | ✗ + →，见下 |

## 54 号原样输出

```
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=4 no_file=262158 file_read=1842255 failed=0 journal_differing=24 verification_ran=42 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 I-1.3=2104413/0/0 I-1.4=2104413/0/0 I-1.6=2104413/0/0 I-1.7=2104413/0/0 I-2.1=2104413/0/0 I-2.3=2104413/0/0 I-2.4=2104413/0/0 I-2.5=2104413/0/0 I-3.1=1842252/0/262161 I-3.8=2104413/0/0 I-3.9=1842252/0/262161 I-4.8=2104413/0/0 I-5.1=2104413/0/0 I-5.2=1842252/0/262161 I-5.4=1842252/0/262161 I-7.1=2104413/0/0 I-7.2=2104413/0/0 I-7.4=2104413/0/0 I-7.6=2104413/0/0 I-7.7=2104413/0/0 I-7.8=2104413/0/0 I-9.1=1842252/0/262161 I-9.2=1842252/0/262161 I-9.4=1842252/0/262161 I-9.7=1842252/0/262161 I-9.10=1842252/0/262161 I-9.13=1842252/0/262161 I-9.14=1580105/0/524308 first_violation=none
```

第一条流的这一句只报 `LAYER0 …` 十项计数，不含逐条不变量的 `evaluated/violated` 明细——`.claude/gate.d/54-layer0-replay.sh` 第 25 行只 `grep '^LAYER0 '`，第 26 行紧接着 `rm -f "$log"` 把带明细的 `CHECKER …` 那一行连同整份 cargo 输出一起删了。第二条流的用例把 `tally.checker_counts_by_invariant()`直接拼进同一行 `LAYER0B …` 里，所以门禁echo 出来的这一句自带全部 29 条不变量的明细。两条流用的是同一个 `Layer0Tally` 结构，只是两个测试文件各自选择打印成一行还是拆两行——这是打印形态的差异，不是判定口径的差异。

## 55 号原样输出

```
  ✓ QEMU 真设备上的第一个事务：3 次虚机跑、18 项检查全过；direct 设备侧逐项对得上、两个对照都红在该红的地方
```

## 57 号原样输出

```
  ✓ 内核树 /home/fy5090/linux-bug-fix/linux
  ✓ herd7  7.58, Rev: exported

  ✓ litmus/commit-publish-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/commit-publish.litmus  Never（符合声明）
  ✓ litmus/first-txn-journal-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-journal-implies-units.litmus  Never（符合声明）
  ✓ litmus/first-txn-root-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-root-implies-units.litmus  Never（符合声明）

  ✓ LKMM 通过（3 条 Never：每条都有内容对得上的对照组，2 条绑到代码、1 条声明不对应代码；共 3 条 Sometimes）
  ✓ 内存序：litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符
```

`command -v herd7` 在我这次会话的 PATH 下 exit=1（见环境旁证），但 `.claude/scripts/lkmm.sh` 自己找到了 `/home/fy5090/linux-bug-fix/linux` 下的内核树与其中的 herd7（7.58），不受影响；这条不算「缺 herd7」。


## 59 号原样输出（✗ + →）

```
  ✗ 变异表的锚点腐化：
      crates/mutations.tsv:32 步 4：只写被退回的实例那一行、不写中间实例行：原文在 crates/singlefs-core/src/mount.rs 里命中 0 次，要恰好 1 次
      crates/mutations.tsv:37 步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根：原文在 crates/singlefs-checker/src/walk.rs 里命中 0 次，要恰好 1 次
      crates/mutations.tsv:41 步 5：复用时追加记录而不改写（同盘同槽两条）：原文在 crates/singlefs-core/src/allocator.rs 里命中 0 次，要恰好 1 次
      crates/mutations.tsv:42 步 5：checker 的候选集不按 F 收（F 之下的根照走）：原文在 crates/singlefs-checker/src/walk.rs 里命中 0 次，要恰好 1 次
      crates/mutations.tsv:69 步 3：树表 0 条的一版上要写行时，拒绝挪到取号之后（超级块已写进新号才返回）：原文在 crates/singlefs-core/src/mount.rs 里命中 0 次，要恰好 1 次
      crates/mutations.tsv:107 增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉）：原文在 crates/singlefs-core/src/mount.rs 里命中 0 次，要恰好 1 次
     → 怎么办：改代码时把变异表里的原文一起改到今天的写法（锚点腐化的那条变异等于没跑过，mutation-sampling.md 第七类）。
```

**这 6 行都不在这一轮追加的第 121–128 行里，都是这一轮之前就登记的旧行**，但它们指的文件（`mount.rs`、`walk.rs`、`allocator.rs`）都在这一轮的改动范围内（见 `research/prompts/m2-wave2-crash-verifier-scope.txt`）。是不是这一轮改坏的、该不该由这一轮的实现员回去把这 6 行的原文改到今天的写法，不归我判（`.claude/agents/crash-verifier.md`「没做什么」：不判红是不是这一轮的改动造成的）。

**关键的一点：`.claude/gate.d/59-crates-mutation-replay.sh` 第 50–64 行先对全表 128 行统一做锚点预扫，任何一行不恰好命中一次就在第 64 行 `sys.exit(1)`，预扫之后（第 65 行起）才真的复制仓、逐条改坏、跑 cargo test。这次预扫在第 6 行不合格的地方就整体退出，全表 128 行没有一行真的进了后面的复制 / 改坏 / cargo test 流程。** 也就是说，这一轮实现员追加的第 121–128 行（Z1-d 两条、I-5.4 两条、Z1-a 四条）**这次一条都没有被真的复跑过**——不是它们红了，是它们连跑都没跑到。实现员报告 `research/prompts/m2-wave2-fixes-implementer-report.md` 第 13 行写的推翻条件之一「门禁 59 号在 `gate-triage` 那一轮复跑，追加的 8 行有一行没红」，**这次没有被证实也没有被证伪**：这次的 exit=1 是因为另外 6 行旧的锚点腐化，不是新 8 行里任何一行没红。这一条要另找一次不撞上这 6 行旧锚点腐化的复跑才能真正验到。


## 主 agent 要的两个数

**层 0 两条流上 I-5.4（分配记录罩住的槽互不相交） 评估过的状态数与违例数：**

| 流 | 评估过的状态数 | 违例数 |
|---|---|---|
| 第一个事务那条流（`first_transaction_step_seven_layer0` 全量） | 4 | 0 |
| 两次发布那条流（`second_transaction_step_zero_layer0` 全量） | 1 842 252 | 0 |
| 两条流合计 | 1 842 256 | 0 |

第一条流的 4/0 不是从 54 号的 echo 行里直接读到的（它的 `LAYER0 …` 行不含逐条明细，见上），是我另跑同一条 cargo test 命令（与 54 号第 17 行完全相同的命令，二进制已经是 54 号那次跑过的 release 产物，不用重编）、
用 `--nocapture` 把 `CHECKER …` 那一行也留下来现读到的：

```
LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-1.3=262165/0 I-1.4=262165/0 I-1.6=262165/0 I-1.7=262165/0 I-2.1=262165/0 I-2.3=262165/0 I-2.4=262165/0 I-2.5=262165/0 I-3.1=4/0 I-3.8=262165/0 I-3.9=4/0 I-4.8=262165/0 I-5.1=262165/0 I-5.2=4/0 I-5.4=4/0 I-7.1=262165/0 I-7.2=262165/0 I-7.4=262165/0 I-7.6=262165/0 I-7.7=262165/0 I-7.8=262165/0 I-9.1=4/0 I-9.2=4/0 I-9.4=4/0 I-9.7=4/0 I-9.10=4/0 I-9.13=4/0 I-9.14=0/0
```

第二条流的 1842252/0 直接原样出现在 54 号自己 echo 出来的那一行里（`I-5.4=1842252/0/262161`，三段依次是评估过/判违例/不适用；1842252 + 262161 = 2104413 = 状态总数，一个状态都没漏记）。

**`first_transaction_step_seven_layer0` 全量里 I-5.4 评估过的状态数：4。**
实现员报告第 220 行写的是「这个数我只在 22 个状态的快用例上验过，`assert_checker_counts(&tally, 22, 4)` 通过了」；上面这次是在 262 165 个状态的全量流上直接现读到的 `I-5.4=4/0`，与快用例上量到的 4 一致，且 54 号里这条用例本身（内含同一条 `assert_checker_counts` 对全量调用的断言）已经 `exit=0` 通过。**这条确认成立，不是外推**：`I-3.1=4/0`、`I-3.9=4/0`、`I-5.2=4/0` 等同组的另外九条不变量在全量下也都是 4，与用例注释「只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树与分配记录树，那 10 条只在这些状态上评估」一致（`crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` 第 84 行附近的文档注释）。


## 判读纪律要求核的一条：kb 状态列与代码不符

`.claude/kb/invariants.md` 第 173 行，I-5.4 那一行的「checker 状态」列写的是「未实现（2026-09-18 立……）」，
但 `crates/singlefs-checker/src/image.rs` 第 36 行的 `IMPLEMENTED_INVARIANTS` 已经把 `"I-5.4"` 排在第 29 个成员里，
`crates/singlefs-checker/src/walk.rs` 里 `judge_allocation_records_disjoint` 真的在判它，54 号两条流跑出来的也是非零的评估计数（不是「不适用」也不是「没被评估到」）。
这一条我不改 kb（不归我写范围），只按「判读纪律」要求把它记下来：**这次的 I-5.4 计数不是「代码没跑到」，是真被评估过的**——如果只看 kb 那一行的「未实现」就以为这两个数没有意义，会判反。
实现员报告第 216 行自己也已经标出这处 kb 要改（「要改成已实现」），这不是我这次新发现的缺口，只是复核一遍确认它现在还没改。

## 指纹：54 号跑的过程中源码指纹变过

| 时点 | HEAD | diff sha256 | untracked sha256 |
|---|---|---|---|
| 54 开跑（04:22:54Z） | 502ba80e | 3f8c687a… | 8007246… |
| 54 结束（08:13:43Z） | 502ba80e | 0fffa176… | 6510e5b4… |
| 55/57/59 开跑与结束（08:20:47Z–08:21:46Z） | 502ba80e | 0fffa176…（四次全部一致） | 6510e5b4…（四次全部一致） |

**54 号开跑与结束这两个指纹不是同一份，55/57/59 用的是「54 号结束」那一份，彼此一致。** 查过之后确认这不是源码内容变了，是 git 索引状态变了：
`git status --porcelain` 里有 19 个文件从 `A` / `AM`（已暂存的新文件）变成了 `??`（未跟踪），具体是
`crates/singlefs-core/src/{instance_table,mount,write_accounting}.rs`、`crates/singlefs-harness/src/{bin/first_transaction_region_bytes,first_transaction_regions,hexadecimal,sha256}.rs`、
以及 `crates/singlefs-harness/tests/` 下 12 个文件（`first_transaction_region_bytes.rs`、`second_transaction_step_{five_reuse,four_rollback,three_formatted_pool,three_formatted_pool_layer0,three_second_instance}.rs`、
`second_transaction_supplement_{one_write_accounting,two_accounting_node_full,two_commit_generated_fallback,two_row_publish_admission,two_unequal_devices,two_warm_up_counter}.rs`）。
这 19 个文件现在的 `mtime` 全部 ≤ 2026-09-18T04:10:13Z（最晚那个是 `mount.rs` 的 03:53:28Z 与 `reused_record_overlap.rs`/`instance_table_page_full.rs` 的 04:03:15Z/04:10:13Z），都早于 54 号开跑的 04:22:54Z；
仍然留在 `git diff HEAD --stat` 里的那 26 个已跟踪文件，每个文件的改动行数（`insertions`/`deletions`）与开跑前记的那份逐字相同。
两条合起来：**这 19 个文件的字节内容在 54 号跑的这将近 4 个小时里没有被改过，只是 git 索引把它们从「已暂存」翻成了「未跟踪」**——是别的会话或流程做了一次 `git reset` 一类的索引操作，没有动工作区里的字节。
54 号里两条流跑的、以及 55/57/59 跑的，是同一份源码；只是「这份源码眼下在 git 里算暂存还是未跟踪」在 54 号跑的过程中变了。这次是我查过的结论，不是没查就当一样处理。


## 没做什么

- 没修 59 号那 6 处锚点腐化，也没改 `mutations.tsv` 或代码；不判它是不是这一轮的改动造成的，交主 agent 或 `gate-triage`。
- 没能验证实现员报告第 13 行的推翻条件之一（「追加的第 121–128 行有一行没红」）：59 号这次在预扫阶段就整体退出，128 行一条都没有真的进入复制 / 改坏 / cargo test 那一段（见上「59 号原样输出」一节的说明）。要验这一条，得先有一次不撞上那 6 行旧锚点腐化的 59 号复跑。
- 没有改 `.claude/kb/invariants.md` 第 173 行 I-5.4 那一行的「未实现」；只核实并记下它与代码现状不符，改不改、怎么改不归我。
- 没有跑门禁 89 号（阶段自检）或除 54/55/57/59 之外任何一个阶段；`.claude/gate.d/stage-owners.tsv` 里登记给 `crash-verifier` 的就是这四个。
- 54 号全绿只说明这两条流的证据要求满足；层 0 覆盖到哪几条流、哪些流没进来，看 54 号自己的文件头两行（`# gate-stage:`、`# gate-covers:`），不由我外推。
- 没有等或复跑「之前那趟跑旧代码的层 0」——按派发说明，那一趟已由主 agent 停掉。

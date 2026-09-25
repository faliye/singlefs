# 三方论证正推腿（Sonnet）：`m2-final-code-r4`，判 Z20 / Z23

代码读冻结副本 `/tmp/claude-1000/m2-final-code-r4/tree/crates/`；定义、检测器与两道门禁读冻结副本 `/tmp/claude-1000/m2-final-code-r4/defs/`；kb 引文行号取自 `/tmp/claude-1000/m2-final-code-r4/kb-snapshot/`，逐条 `grep -nF` 核过（未核过的没写进本报告）。判定只出「一致 / 冲突 / 规则没说」三种之一。

## Z20　抬 F 被拒退回分配器、核出对不上先重读

**代码位置**：`crates/singlefs-core/src/mount.rs` 的 `raise_rollback_floor`（955–1109 行）；`crates/singlefs-core/src/transaction.rs` 的 `copies_failing_the_release_checksum_check`（2700–2771 行）。
**压着的条款**：C546（抬 F 被拒时扣住的槽不退回，`checks-owed.md:478`）；D19（块指针的结构与宽度预算） 已定项 5「硬规则 1 的读盘核读不出、核出对不上时怎么办」（`19-块指针的结构与宽度预算.md:112`）。

### 问 1：换回的那一份分配器与抬 F 之前逐项相同，是不是在每一种被拒点（第一次、第二次、第 n 次空发布）上都成立

**判定：一致**（与 C546 记述的范围一致；不是「每一种被拒点都成立」，只在第一次成立，条款本身也只要求到这一步）。

代码：`mount.rs:979` 在动分配器之前 `let allocator_before_the_raise = allocator.clone();`；出错分支 `mount.rs:1073–1093`：

```
1079:                if publishes.is_empty() && allocator.frozen_publish().is_none() {
1080:                    *allocator = allocator_before_the_raise.clone();
1081:                }
```

只在 `publishes.is_empty()`（这一串一次都没落盘，即第一次空发布就被拒）时整份换回；`publishes` 非空（第二次起被拒）不换回。这与 C546 的最新记述逐字对应（`checks-owed.md:478`）：

> | C546 | 抬 F 被拒时扣住的槽不退回 | 抬 F 那一串发布被落点拒时，`mount.rs` 的 `release_reclaim_holds` 走不到、分配器不退回，被扣住的槽计数上是空闲、实际发不出去，而 D28（挂载期承诺量） 式子把它们算成可用 | 造抬 F 第一次空发布就被落点拒的历史，断言被拒之后扣住的槽已放开、分配器与被拒之前逐项相同；判别力：今天的代码必须红 | 实现缺口，归实二五。2026-09-25 实二五还了第一次那一半：抬 F 第一次空发布在任何写之前被拒时，分配器整个换回抬 F 之前那一份（用例按 `Debug` 整份比对）；第二次起被拒时扣住的槽仍算作可用，条款没写，照旧欠着。

测试 `crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs`：`raising_the_floor_with_no_free_slot_outside_the_hold_fails_before_any_write_and_hands_the_allocator_back_exactly_as_before_the_raise`（354–430 行）只造「第一次空发布即被拒」的历史，断言 `format!("{:?}", pool.allocator) == allocator_before_the_raise`（416 行，逐项 `Debug` 整份比对）；`a_raise_whose_second_empty_publish_is_refused_reports_that_one_publish_of_the_sequence_persisted`（442–527 行）造「第二次被拒」的历史，只断言 C516（`checks-owed.md:506`）要的已落盘账，**没有**任何断言要求分配器换回或空槽计数复位。两份用例分别对应 C546 描述的「还了第一次那一半」与「第二次起…照旧欠着」，代码、注释、测试三者一致，且都点名了各自对应的账（C546 / C516），不是把两笔账混成一笔。

**推翻条件**：若在 `publishes` 非空的分支里也发现 `*allocator = allocator_before_the_raise.clone()`（或等效的整份复位），或若 `checks-owed.md` C546 一行改写为「两种情形都已还清」而代码仍只处理第一次，则本判定作废。

### 问 2：第二次起没换回的那一半，会不会让准入读数与盘上对不上

**判定：一致**（会，且这正是 C546 明写的现象，不是这一轮新发现；背景材料「已知、不算打中的」已列此项）。

机制link（均在冻结副本内核实）：

- `allocator.rs:1204–1254` 的 `reclaim_released_up_to`/`reclaim_released_records_up_to`：`floor` 用的是 `new_floor`（`mount.rs:1040` `reclaim_floor(new_floor, oldest_valid_root)`），对每条 `record.generation <= floor` 的已释放记录调用 `device_map.mark_reclaimed(...)`（1243 行），随后按 `reuse` 参数（这里传的是 `ReclaimedReuse::HeldUntilFloorTakesEffect`，`mount.rs:1041`）额外调用 `hold_until_floor_takes_effect`（1246–1249 行）。
- `allocator.rs:370–388` 的 `mark_reclaimed`：无论 `reuse` 是哪一种，都先执行 `self.allocated_slots -= span; self.deferred_slots -= span; self.free_slots += span;`（386–388 行）——即先把这些槽记成「空闲」，`hold_until_floor_takes_effect` 是随后另加的一层限制。
- `allocator.rs:267–273` 的 `is_free`：`!self.allocated[...] && !self.isolated[...] && !self.held_until_floor_takes_effect[...]`——held 位一旦置位，`is_free` 立即为 false，分配器实际发不出去这个槽。
- `allocator.rs:211–213`（结构体字段的文档注释）与 `allocator.rs:338`（`hold_until_floor_takes_effect` 方法的文档注释）分别写着：

  ```
  211:    /// 抬 F 回收、但 F 还没在每块盘上生效的槽：记账已经算它空闲，分配器却不许发出去，直到带新 F 的根落满每块盘
  ```
  ```
  338:    /// 抬 F 回收的落点先扣住：空闲计数照加，位图照清，但 `is_free` 与开段都绕开它，直到 `release_holds`。
  ```

- `admission.rs:323` 的 `AllocationBasis`（记账读数结构）：`allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots())`——D28（挂载期承诺量） 已定项 1 式子（`admission.rs:262`「一块盘的可用(d)：容量减去式子里另外八项之后剩下的字节」）里的「已分配」项直接读 `allocated_slots()`，这个量在 `mark_reclaimed` 里已经被减掉（不管有没有 held），式子的八项里没有任何一项单独扣「held 但未生效」的槽。

四点连起来：第二次起被拒时，`allocator_before_the_raise` 没有被换回，held 位与 `free_slots`/`allocated_slots` 的改动都留在进程里；此时读 D28 准入式子会把这些槽算进「可用」，但 `is_free` 判它们不可发——即 Z20 问 2 所问的「准入读数与盘上（实际可发放状态）对不上」确实发生。这与 C546（`checks-owed.md:478`）「被扣住的槽计数上是空闲、实际发不出去，而 D28 式子把它们算成可用」逐字一致，也是背景材料第 41 行明写「已知、不算打中的」条目，本报告只是把它落到具体代码行，不作为新打中项。

**推翻条件**：若准入式子（`admission.rs`）另有一项专门扣「held_until_floor_takes_effect 的槽数」，或若 `mark_reclaimed` 在 held 分支下不提前增加 `free_slots`，则本判定作废。

### 问 3：两次读结果不同类时的处置，是不是与条款写的一致

**判定：一致**。

`transaction.rs:2729–2743`：

```
2729:                // 第一次读不出、或读出来核出对不上，都先重读一次（只一次，D19（块指针的结构与宽度预算） 已定项 5
2730:                // 「硬规则 1 的读盘核读不出、核出对不上时怎么办」）：一次瞬时坏读不许让单元永久隔离。
2731:                // 重读那一次对得上就算这一份对得上；两次都没对上，按重读那一次的样子报。
2732:                let reading = match check_the_copy_against_its_location_entry(read_the_copy()) {
2733:                    CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
2734:                    CopyCheck::Unreadable | CopyCheck::ChecksumMismatch => {
2735:                        match check_the_copy_against_its_location_entry(read_the_copy()) {
2736:                            CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
2737:                            CopyCheck::Unreadable => {
2738:                                QuarantinedCopyReading::UnreadableAfterOneReread
2739:                            }
2740:                            CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,
2741:                        }
2742:                    }
2743:                };
```

无论第一次失败的类型是 `Unreadable` 还是 `ChecksumMismatch`（外层 match 把这两类合并成同一分支），最终上报的 `reading` **只看第二次（重读）的结果**：重读 `Intact` 记「另一份失败」（这一份算通过）；重读仍失败则按重读那次的类型上报（`UnreadableAfterOneReread` 或 `ChecksumMismatch`）。这与 D19 已定项 5（`19-块指针的结构与宽度预算.md:112`）的字面完全对应：

> 两次读结果不同类（先读不出、重读对不上，或反过来）按两次都没对上处置，报重读那一次的样子（实二五取，被攻过零轮）。

代码不是分两条路径分别处理「同类」与「异类」，而是根本不判第一次失败的具体类型，直接以第二次的类型作为最终结果——这自动覆盖了条款要求的「异类按未对上处置、报重读那一次的样子」，也覆盖了「同类」的两种情形（各自返回同类型），没有额外分支需要另证。

**推翻条件**：若发现某处对 `reading` 的取值不是取自第二次读的结果（例如把第一次的类型也纳入判定），或该条款被改写为要求区分「同类/异类」两种上报格式，则本判定作废。


## Z23　定义、检测器第五种拒绝、门禁 55 与 74

**材料**：`/tmp/claude-1000/m2-final-code-r4/defs-r3-to-r4.diff`、`gates-vs-head.diff`，冻结副本 `defs/.claude/agent-common.md`、`defs/.claude/main-agent.md`、`defs/.claude/hooks/bash-command-detector.sh`、`defs/.claude/gate.d/55-qemu-first-transaction.sh`、`defs/.claude/gate.d/74-model-differential.sh`。

### 问 1：定义之间、定义与检测器之间有没有说反话，有没有写为什么的句子

**判定：冲突**（按 `rules-discipline.md` 的判据字面审，检测器新增注释里有一句「为什么」；但该判据的射程本身不覆盖 hook 脚本，这一层限制一并写明）。

`rules-discipline.md`（`kb-snapshot/.claude/singlefs-ai-sop/rules/rules-discipline.md:4`）的射程句：

> **管的是**：`rules/*.md`、项目本地的 `.claude/rules/*.md`，也就是**照着执行的规矩**。

第 1 条的表（`rules-discipline.md:14`）：

> | 做什么、不做什么：射程与例外 | 这条怎么来的：实测、日期、谁定的、踩过的坑 |

`.claude/hooks/bash-command-detector.sh` 不在 `rules/*.md`、`.claude/rules/*.md`、agent 定义或 skill 正文之列，字面上不是这条规则的射程对象；但背景材料的小节清单明写「Z23 直接以该文件的判据审『定义之间、定义与检测器之间有没有说反话，有没有写为什么的句子』」，即这一格要求把 `rules-discipline.md` 的判据也套到检测器注释上。按这个判据核，检测器新增的 ⑤ 号注释块（`defs/.claude/hooks/bash-command-detector.sh:73–74`）：

```
73	#   以 `/` 结尾的，写的是 目录/源的文件名，源里的通配按这一刻的文件展开）。2026-09-25 E158 第九段执行员用 `cp` 把 s8 同名的三份未跟踪产物整份覆盖，
74	#   旧字节找不回（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 24 行）；write-guard.sh 只拦 Write / Edit 工具，拦不到 shell。
```

这一句带日期（2026-09-25）、带事件（E158 第九段执行员的一次事故）、带出处（records 文件的具体行），是「这条怎么来的」的典型形态，正是第 1 条表里写明不许出现在正文的那一类。**这不是「说反话」**（没有两句互相矛盾的判据），是「写了为什么」。

**推翻条件**：若这句被证明其实是「射程与例外」的一部分（说明这条规则**不**管什么，而不是它**为什么**存在），或若主 agent 认定 hook 脚本注释确实不在这条规则的射程内、这一句不算违规，则本判定应改判「规则没说」。

### 问 2：第五种拒绝在两份定义里的说法与检测器判据是不是同一条

**判定：冲突**（两份定义的措辞不是同一条：`main-agent.md` 缺 `install`、缺「不带 `-a` 的」限定、缺「已存在」限定，比检测器实际判据窄）。

两处并排：

- `defs/.claude/agent-common.md:57`：「…用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下**已存在又没进 git 的**产物（要换就按日期另存新文件名，旧的留着）、…」
- `defs/.claude/main-agent.md:29`：「…用 shell 的 `>`、`tee`、`cp`、`mv` 整份覆盖 `research/results/` 下**未跟踪的**产物、…」

检测器实际判据（`defs/.claude/hooks/bash-command-detector.sh:71–72`）：

```
71	# ⑤ 命令位置上会整份覆盖 `research/results/` 下一个已存在、又没进 git（`git ls-files --error-unmatch` 失败）的文件：重定向 `>`、`>|`、`&>`、`>& 文件`
72	#   （前面带 fd 号的 `2>` 一样截断，一样算）、不带 `-a` / `--append` 的 `tee` 写的每个文件、`cp` / `mv` / `install` 的目标（`-t 目录`、或目标是已有目录、
```

三点差异：
1. `main-agent.md` 的列举里没有 `install`，而 `agent-common.md` 与检测器都把 `install` 算进这一种拒绝；`install` 的判定逻辑在检测器源码里也确有实现（`COPY_SHORT_OPTIONS_WITH_VALUE = {"cp": "tS", "mv": "tS", "install": "tSgmo"}`，`bash-command-detector.sh:127`）。
2. `main-agent.md` 写「`tee`」不写「不带 `-a` 的 `tee`」——检测器与 `agent-common.md` 都明确只拦不带 `-a`/`--append` 的 `tee`（放行行：`defs/.claude/hooks/bash-command-detector.sh:76`「放行：`>>`、`&>>`、`tee -a` 追加」），`main-agent.md` 的字面读法会让人以为 `tee -a` 也在拒绝之列。
3. `main-agent.md` 写「未跟踪的产物」，`agent-common.md` 与检测器都写「已存在又没进 git 的」——检测器的判据是「目标不存在（新文件名）」一律放行（同一行「放行：…目标不存在（新文件名）」），只说「未跟踪」而不带「已存在」，字面上不排除对新文件名的误读。

三处差异都是 `main-agent.md` 比检测器的实际判据**窄**（少列一种会被拒绝的写法、少了两个限定词），不是检测器比定义宽或反过来，也不是两份定义直接互相矛盾（没有一份说「允许」另一份说「拒绝」的同一件事），但按「引用要连它的定义一起带上」（`evidence-discipline.md`）与「限定词没了」的判据，这不算「同一条」的转述。

**推翻条件**：若 `main-agent.md:29` 那一整句本来就是跨多个 hook 的粗略汇总（这句里同时列了 `bash-command-detector.sh`、`heavy-test-guard.sh`、path-moves 与 session-wrapup 四类不同 hook 的拒绝项），且主 agent 认定这类汇总句不需要逐字对齐单个 hook 的判据，则「冲突」应改判为「规则没说」（这份定义本来就没打算精确复述第五种拒绝的判据）。

### 问 3：55 号新加的判据能不能在 F 没抬的镜像上判红（与第二轮 Z10 那一格对照）

**判定：一致**（能判红；这是对第二轮 Z10 那一格缺口的修复，代码已验证）。

第二轮 Z10 的判决（`research/prompts/m2-final-code-r2-main-verification.md` 第 75 行，本轮材料未冻结此文件，转述并标出处）：55 号原判据只读 `requested_floor=`（进程内存里的值），冷重开的字段表没有 `rollback_floor`，checker 只在 F 真变大时才判上界，因此「F 没抬的镜像」当时能判绿——改法是「让冷重开之后打印盘上读回的 `rollback_floor_on_disk=`，55 号改判那一行」。

这一轮冻结副本里，`crates/singlefs-harness/src/bin/first_transaction_on_device.rs:1066–1078` 的 `rollback_floor_read_back_from_the_chosen_root_line`：

```
1066	fn rollback_floor_read_back_from_the_chosen_root_line(reader: &dyn PoolReader) -> String {
1067	    let chosen_root = choose_system_configuration(reader)
1068	        .ok()
1069	        .and_then(|system_configuration| choose_root(reader, &system_configuration));
1070	    match chosen_root {
1071	        Some(root) => format!(
1072	            "name=recover_cold_rollback_floor chosen_root={}:{} rollback_floor_on_disk={}",
1073	            root.instance.0, root.checkpoint_txg.0, root.rollback_floor.0
1074	        ),
```

`root.rollback_floor.0` 取自 `choose_root(reader, ...)` 对 `reader`（真实的 `PoolReader`，冷重开之后现读）的调用结果，不是内存里那一版的 `requested_floor`——这是一次真的盘上读回。

`defs/.claude/gate.d/55-qemu-first-transaction.sh` 新增的必检行（`required_lines_of` 对 `raise-rollback-floor` 模式）：

```
name=recover_cold_rollback_floor chosen_root=2:11 rollback_floor_on_disk=3$
```

判定机制（`55-qemu-first-transaction.sh:219–223`）：`grep -aq "$pattern" "$out" || fail ...`——若盘上实际读回的 F 不是 3（例如「F 没抬」的镜像上仍是 0），`root.rollback_floor.0` 打印出的值就不是 `3`，这一行 `grep` 找不到该模式，`fail` 触发，整个阶段判红。这与第二轮 Z10 判决要求的改法（打印盘上读回的 F、55 号改判那一行）完全对应，缺口已补。

**推翻条件**：若 `choose_root` 在某条路径上会回退读取内存缓存的 `requested_floor` 而非真正的盘上字段（例如某种缓存命中路径），或若「F 没抬」的镜像恰好仍然打印出 `rollback_floor_on_disk=3`（例如择根择到了另一条带旧 F=3 的根），则本判定作废。**复核不了**：本轮未实际起 QEMU 复跑 55 号或构造一份「F 没抬」的坏镜像去实测，只静态核对了打印函数与门禁判据的源码；构造反例镜像属云端攻方（Opus，Z19/Z21/Z22）或本地攻方的算术格分工范围。

### 问 4：74 号改名之后，段名表与测试打的段名逐字对不对得上

**判定：一致**（改名后的段名与测试打印的段名逐字相同）。

`defs/.claude/gate.d/74-model-differential.sh:39`：

```
SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：越过原分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点")
```

`crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:346`：

```
346	        "── 随机历史：越过原分配记录墙的取样点 ──\n{rendered}用时 {:.1} 秒\n",
```

门禁的匹配机制（`74-model-differential.sh:64`）：`awk -v header="── ${section} ──" '$0 == header {...}'`——`header` 拼出来正是 `"── 随机历史：越过原分配记录墙的取样点 ──"`，与测试打印的这一行逐字节相同（含冒号、破折号数量与两侧空格）。逐一核对 74 号 `SECTIONS` 数组的另外四项（`随机历史快档`、`偏向抬 F 之后复用的取样点`、`偏向抬 F 之后回退的取样点`、`小盘上逼近单元区墙的取样点`），测试文件里对应的四行（242、271、308、391 行）同样逐字匹配。

**附带观察（超出问 4 本身，供参考）**：`second_transaction_supplement_three_random_history.rs:446` 这一轮新增了第六个模型对拍分段「`── 随机历史：小盘上逼近单元区墙的取样点（空间准入判着） ──`」（第三轮冻结树 `/tmp/claude-1000/m2-final-code-r3/tree/` 里的同一个测试文件没有这一段，`grep -n '── 随机历史' m2-final-code-r3/tree/…/second_transaction_supplement_three_random_history.rs` 只有 6 处、m2-final-code-r4 里有 7 处，多出的正是这一段），而 `defs/.claude/gate.d/74-model-differential.sh:39` 的 `SECTIONS` 数组仍然只有 5 项，没有把这个新分段列进去——这个新分段目前不在 74 号的必检范围内。这一点不在问 4 的射程内（问 4 只问改名那一项对不对得上），不作为本格的判定依据，仅记录供主 agent参考。

**推翻条件**：若测试文件的段标题字符串与门禁数组的对应项之间任何一处标点、空格或字数不同，本判定作废；附带观察部分若 74 号在别处（例如另一条门禁阶段或另一份 `SECTIONS` 定义）另行覆盖了这第六段，附带观察应删去。

**复核不了**：`--selftest` 未能在冻结副本 `defs/` 上跑通——`defs/.claude/hooks/` 目录只冻结了 `bash-command-detector.sh` 一份文件，缺它依赖的共用切词模块 `lib_shell_words.py`；直接跑 `bash defs/.claude/hooks/bash-command-detector.sh --selftest` 得到：

```
  ✗ 自检：读不到共用切词模块 /tmp/claude-1000/m2-final-code-r4/defs/.claude/hooks/lib_shell_words.py（FileNotFoundError(2, 'No such file or directory')）
    → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份，别在 hook 里再抄一份），再跑 --selftest
```

退出码 0（自检本身没有真正执行到任何一组用例，不是「跑过判绿」）。没有从别处补一份 `lib_shell_words.py` 进冻结副本——那样测的就不是「这一份冻结副本」本身，而是冻结副本加一个不属于这一轮快照的依赖。这一项的 `--selftest` 是否真能通过，留给主 agent 用完整的冻结树（或工作区当前版本）复核。


## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z20 问 1（换回是否逐点成立） | 一致 | 只在第一次空发布被拒时整份换回（`mount.rs:1079-1081`），与 C546 最新记述、测试覆盖范围一致，第二次起本就没有条款要求换回 |
| Z20 问 2（第二次起的账对不对得上盘） | 一致 | held 槽先加进 `free_slots`、`allocated_slots` 先减（`allocator.rs:386-388`），`is_free` 却判它不可发（`allocator.rs:267-273`），D28 式子读 `allocated_slots()`（`admission.rs:323`）不扣 held，确实对不上，这正是 C546 记的已知欠账 |
| Z20 问 3（两次读结果不同类的处置） | 一致 | 代码只按第二次（重读）的结果上报（`transaction.rs:2732-2743`），与 D19 已定项 5「报重读那一次的样子」逐字对应 |
| Z23 问 1（说反话 / 为什么句） | 冲突 | 按 `rules-discipline.md` 判据，检测器 ⑤ 号注释里带日期与事件的一句（`bash-command-detector.sh:73-74`）是「为什么」句；但该规则字面射程不含 hook 脚本，射程限制已写明 |
| Z23 问 2（第五种拒绝两份定义与检测器判据） | 冲突 | `main-agent.md:29` 比 `agent-common.md:57` 与检测器判据窄：缺 `install`、缺「不带 `-a` 的」、缺「已存在」三处限定 |
| Z23 问 3（55 号新判据能否在 F 没抬的镜像判红） | 一致 | `rollback_floor_on_disk=` 打印的是 `choose_root` 现读的盘上字段（`first_transaction_on_device.rs:1066-1078`），门禁按 `grep -aq` 必检该行（`55-qemu-first-transaction.sh:219-223`），F 没抬则该行不出现、判红；对齐第二轮 Z10 的改法要求 |
| Z23 问 4（74 号改名后段名逐字对得上） | 一致 | `SECTIONS` 数组第四项与测试打印的标题行（`second_transaction_supplement_three_random_history.rs:346`）逐字节相同；另五项同样核对通过 |

## 没做什么

- 没判 Z19、Z21、Z22（分给云端攻方 Opus）与算术格 ①–④（分给本地攻方）。
- Z20：没有实际起测试二进制复跑 `second_transaction_supplement_two_commit_generated_fallback` 与 `second_transaction_supplement_two_release_checksum_quarantine` 这两份用例（本轮只静态核对源码与测试断言，判定所需的事实都能从源码逐字确认，未额外跑 `cargo test`）。
- Z23 问 3：没有实际起 QEMU 复跑 55 号，也没有构造一份「F 没抬」的坏镜像去实测门禁真的判红——判定基于静态读代码与门禁脚本的判据链，标注「复核不了」的部分已在正文列出。
- Z23：`bash-command-detector.sh --selftest` 因冻结副本缺依赖文件 `lib_shell_words.py` 没能真正跑通，已如实报告、未从别处补依赖。
- 没有对 Z20、Z23 之外任何格给出判定或替主 agent 采纳结论。

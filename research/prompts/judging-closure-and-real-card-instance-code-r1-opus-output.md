# 攻方腿：judging-closure-and-real-card-instance-code-r1（2026-11-18）

被判的四份文件在开工时与开工快照逐字相同（`sha256sum -c research/prompts/judging-closure-and-real-card-instance-code-r1-snapshot/crates-sha256.txt` 四行全 OK）。没写计数模型；在草稿目录的仓副本里做了两处改动（取线程名改成 `expect`、删掉拷数据文件那个循环）、加了两条探针测试（`attack-probe.patch`），一条命令都没起，副本上的结局全部「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1a 三个只打日志的单元不改判定字 | 打中（今天的代码成立，键守不住它） | `gpu_lifecycle_line` 取线程名改成 `expect`：判器与 GPU 服务的派活线程都没名字，一派活就 panic，键不变、守登记的测试全绿 |
| 1b 四个单元只经那三个够得到 | 成立（性质有测试守），实现员的边解释有一半不对 | 方法名 `name` 由留下的入口 `judge_one_dispatch` 的 `pass.name()` 就算见过，不是靠 `thread.name()` |
| 1c 登记会不会切掉留下的单元真要的东西 | 第二批不会（它对键零作用，只是记账）；第一批见 1a | — |
| 2 带行号的登记名 | 实现员那句成立；陈旧后键不宽不窄；这条测试是噪声源 | `lib.rs` 第 78 行之上是 `pub mod` 表；2026-10-18 起改 `lib.rs` 的提交 2 次，一次是整份建文件，另一次在第 78 行之上加了 2 行（量的，git log） |
| 3 两步配对漏报 | 没打中 | 配对只拿掉（种类，指纹）相同的一对，剩下的差与键的差逐个相等；附一处措辞：空清单的红句说「组成清单是空表」 |
| 4a 线程数前提 | 对 `:31746` 那一处成立；实现员「唯一读」那句不成立 | `checking_threads()` 在产品代码里有 4 处调用 |
| 4b 漏拷就拒算还有没有守 | 打中（实现员已认一半） | 拷数据文件那个循环没有任何测试会红；变异行 `crates/mutations.tsv:8208` 名字写「漏拷」，弄坏的不是拷，是清单 |
| 4c (b)(c) 两条是警报还是噪声 | 警报 | 它们红的时候正是拷循环开始有用的时候 |
| 5a OnceLock 与「不用单例」 | 推不出（条款没有出路），算打中 | 先例同样是单例、没有豁免，「照先例」不是独立依据 |
| 5b 三条不要卡的测试 | 打中两处没罩到 | 薄壳换成每次新格（实现员已认）；建实例 panic 后 `OnceLock` 下一次再建 |
| 6 八格检查单 | 七格成立，一格推不出 | 见末节 |

## 面一：过度外推与越位

### 1a 打中：「不改判定字」只靠交值检查守，panic 与副作用一条都不守

登记的理由写着「调它们的派活、起卡、编管线、还卡与按块判的那几处都只当语句调」（`crates/singlefs-verification/src/judging_units_entries.rs:1169`）。今天的文本是这样；但守它的东西只有一条交值检查：「（留下的单元读不到登记的单元交的值）与 `an_edit_of_the_listed_gpu_lifecycle_trace_keeps_the_gpu_judge_key` 一族三条（改其中一个，键不变）；」（`crates/singlefs-verification/src/judging_units_entries.rs:1176`）。后一族三条断言的是登记生效后的结果（改登记的单元键不变），是登记的推论，不是登记安全的前提（见面二）。

具体改动（只改登记了的 `gpu_lifecycle_line`）：

- 哪一行：「        thread.name(),」（`crates/singlefs-verification/src/gpu_unit_checks.rs:1055`）
- 改成：`thread.name().expect("GPU 阶段现场那一行只在有名字的测试线程上打"),`（像是收紧 `test=` 那一格的写法，看不出会改结局）。
- 谁会在没名字的线程上调它：判器内核编译是 「let recovery = scope.spawn(recovery);」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:1308`）这类 `scope.spawn`；GPU 服务每张卡一个工作线程，「scope.spawn(move || {」（`gpu-verification-server/src/service_loop.rs:270`），里面跑 `card_worker_loop` 与判器；服务自己的日志也照没名字的线程写，「thread.name().unwrap_or("<unnamed>")」（`gpu-verification-server/src/service_log.rs:149`）。`std::thread::scope` 起的线程没有名字（Rust 标准库的行为，推的，没在本项目量）。
- 本该看到：判器的单元一改、判过的块要现判。
- 实际会怎样：这个单元不在闭包里，键不变，库里旧判定照复用；守登记的五条里只有交值那一条读登记单元的源码，而它只看签名里有没有 `->`，键不变的三条只比键，都照绿；单测 `the_gpu_lifecycle_line_names_its_time_keys_in_full` 交的是测试线程「&std::thread::current(),」（`crates/singlefs-verification/src/gpu_unit_checks.rs:3247`），有名字，照绿。真去判时每次派活 panic。GPU 服务按判器那一组的键认活（推的：键不变，服务不必重编，旧二进制照跑），下一次别的改动让键变、服务重编时才全部炸，红会记到那个无关改动头上（推的）。
- 同一类的别的改动（推的，没量）：`gpu_lifecycle_trace` 收的是整张卡「pub(crate) fn gpu_lifecycle_trace(card: &GpuCard, phase: &str, detail: std::fmt::Arguments<'_>) {」（`crates/singlefs-verification/src/gpu_unit_checks.rs:1007`），卡上有「pub fn device_condition(&self) -> &DeviceConditionOfACard {」（`crates/singlefs-verification/src/gpu_unit_checks.rs:1121`），判器在等卡失败时就是经它「condition.record_the_device_lost(」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:2059`）。在登记的单元里加一句记设备丢失，下一次派活的结局就变，键照样不变。

怎么量：草稿目录副本里已经做了这处改动，另加两条探针（`attack-probe.patch`）：
- `crash_identity::tests::an_edit_making_the_listed_gpu_lifecycle_line_panic_on_threads_without_a_name_keeps_the_gpu_judge_key`：用现成的 `assert_an_edit_of_a_listed_unit_that_only_prints_the_gpu_lifecycle_line_keeps_the_gpu_judge_key` 把 `expect` 改回 `thread.name()`，断言 GPU 判器那一组与镜像判定那一把都不变。预期绿。
- `gpu_unit_checks::tests::the_gpu_lifecycle_line_on_a_thread_without_a_name_does_not_panic`：`std::thread::scope` 起一条没名字的线程拼这一行。预期在副本上红。
- 同一次点名守登记的五条与单测：预期全绿。
命令在「要主 agent 跑的命令」第 1 条（同一趟另点了 4b 用的一条）。预期结局「8 条过、1 条失败（没名字的那条）」；若守登记的任何一条红，这一格撤回。

四句：分不分辨臂——这一轮没有臂，打中的是「登记安全」这一句本身；系统当时看不看得到——判别它要读登记单元的函数体，键与守登记的测试都不读（看不到）；满足的是哪一分句——正文第 1 条「有没有副作用（写全局、改卡的状态、panic 的路径）会改判定结局」里的「panic 的路径」与「改卡的状态」两个分句；改法——正文与实现员报告没给改法。

自提的改法（只在我的模型上量过、被攻过零轮；其实连模型都没有，只有副本上的探针，待主 agent 跑）：

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 给登记了的三个单元加一条文本白名单测试：函数体里出现的标识符只许是 `writeln`、`stderr`、`lock`、`format`、`format_args`、`Instant`、`SystemTime`、`OnceLock`、`process::id`、`thread` 的 `id`/`name`、`card.index`、`card.pci_bus_identifier` 与参数名，不许 `expect`、`unwrap`、`panic`、`card.` 的其余字段与方法 | 1a 两种改动都红 | 推的，待主 agent 跑 |
| 把没名字线程的探针（上面第二条）收进正式测试 | 只修 `expect` 那一种 | 推的，待主 agent 跑 |
| 不登记，让三个单元进键，日志一改就现判 | 1a 全部 | 推的；代价是改一个日志字全组现判，正文说过这是登记的原因 |

### 1b 成立，但实现员给的第二条边的解释要撤回一半

登记说「`gpu_lifecycle_line` 里 `thread.name()` 让方法名 `name` 算见过」（`crates/singlefs-verification/src/judging_units_entries.rs:1189`）。可留下的入口 `GpuCrashJudge::judge_one_dispatch`（登记表第 68 行，函数从 `crash_judge_dispatch.rs` 第 1633 行起、第 1956 行是下一个函数）自己就写着「self.submit_one_pass_and_wait(&passes, pass.name(), pipeline, None),」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:1726`）。R-方法是「R-方法  `follow_methods`（RTA）与 `name_type` 里后点名的类型补上之前见过的方法」（`crates/singlefs-unit-graph/src/closure.rs:7`）：方法名 `name` 不论有没有 `thread.name()` 都已见过，`<impl RecordedOperationKind>::name` 够不够得到只看 `RecordedOperationKind` 有没有被点名，而点名它的只有 `use std::io::Write` 撞到的那次宏调用。推的（读 `closure.rs` 文件头，没跑闭包打边）。

后果：四个单元「只经登记的单元够得到」这一性质由 `the_units_listed_as_reached_only_through_the_units_that_print_the_gpu_lifecycle_line_are_reached_through_them_and_by_no_unit_the_group_keeps` 守着（主 agent 给的 19725 读数里它不在 FAILED 那 27 条里，我没复核），成立；但它比登记写的更脆：哪个留下的单元写一次 `RecordedOperationKind`，`name` 那一条立刻连上，因为方法名早已见过。这是该有的警报，不是漏洞。

### 1c 第二批登记对键零作用

第一批登记生效时，四个单元没有一个被沿着收的单元连到（第一格的测试正是断言这一点），登不登它们，闭包都一样、键都一样。它们被登记只因为 `not_following_the_listed_units_drops_only_the_listed_ones` 要「少掉的恰好是登记的」。旁证：变异行「判器键：只经只打 GPU 阶段现场那一行的单元够到的那一批登记整批不交（少掉的四个没登记）」（`crates/mutations.tsv:8199`）点名红的只有这条记账测试，没有一条键测试。所以第二批切不掉留下的单元要的东西；唯一能让它真切东西的情形（留下的单元够到其中一个）同一条测试当场红。没打中。

### 1d 「只当语句调」字面不对

登记写「调它们的派活、起卡、编管线、还卡与按块判的那几处都只当语句调」（`crates/singlefs-verification/src/judging_units_entries.rs:1169`）；起卡那一处是把函数当值交出去，「                &gpu_work_trace,」（`crates/singlefs-verification/src/gpu_unit_checks.rs:745`），交给 `device_started_with_a_lifecycle_line` 的 `&dyn Fn(&str, Arguments)`。它返回 `()`，不影响交值检查的结论；只是这句话要改成「只当语句调或当返回 `()` 的回调交出去」。小。

### 4a 线程数前提：对被改的那一处成立，「唯一」那句撤回

实现员写「`:31742` `fn images_after_the_observers_cut_into_pieces_for_the_cpu(` 是唯一读 `checking_threads()` 的地方（`:31746`），只用它算 `pieces`。」（`/home/fy5090/.local/share/singlefs/work/ledger-topdown-watched-data-files-repair/result.md:16`）。主工作区此刻 `grep -nF 'checking_threads()' crates/singlefs-verification/src/crash_amplification.rs` 的产品代码调用有四处：「        checking_threads(),」（`crates/singlefs-verification/src/crash_amplification.rs:2494`，`check_blocks_where_until`）、「    let threads = checking_threads();」（`crates/singlefs-verification/src/crash_amplification.rs:23949`，`check_selected_blocks`）、「    let threads = checking_threads();」（`crates/singlefs-verification/src/crash_amplification.rs:30826`，`record_the_observers_of_the_flow`）、「crate::crash_library::InnerThreadsTaken::take(checking_threads().min(image_count).max(1));」（`crates/singlefs-verification/src/crash_amplification.rs:31746`）；另有两处在单测里（18022、18288）。

- `:31746` 那一处：读 `judge_the_images_after_the_observers_on_the_cpu`，每段一条线程、每张盘只由自己判、按段次序接回，前提成立（推的，读代码，与实现员同一条路，不算独立校验）。
- 另三处：不带特性那一趟被监视的清单是空的，说明没有一组的闭包够到 `checking_threads`（推的），所以这三处线程数不论改不改结论都不在任何键里；「线程数不改判定结论」对它们要另核，实现员没核。我也没核出反例，记推不出、不记打中。

## 面二：内部矛盾与循环依赖

### 打中：登记、守它的测试、变异行三者绕成一圈

GPU 阶段现场那一批的守护（登记文档列的「两条前提」与变异行）逐条看它到底守什么：

| 测试 | 变异行 | 它红在什么时候 | 守的是 |
|---|---|---|---|
| `an_edit_of_the_listed_gpu_lifecycle_trace_keeps_the_gpu_judge_key` 等三条 | `crates/mutations.tsv:8193`、`:8194`、`:8195`（登记里拿掉一个或整批） | 登记被拿掉、键变宽 | 登记还在（键不变宽） |
| `not_following_the_listed_units_drops_only_the_listed_ones` | `:8199` 等五行 | 少掉的与登记的对不上 | 记账 |
| `the_units_listed_as_reached_only_through_…` | `:8198` | 留下的单元够到第二批 | 第二批的可达性 |
| `no_unit_the_gpu_judge_group_keeps_reads_a_value_returned_by_…` | `:8196`（混进交值的单元） | 留下的单元写了交值单元的名字 | 交值不被读，唯一守「不改判定字」的一条 |

文档把「改其中一个，键不变」写成前提（`crates/singlefs-verification/src/judging_units_entries.rs:1176`），可它是登记生效的结果：登记在、键就不变，测试红只在有人把键改宽（更安全）的时候。三方互相当依据：登记的理由是「不改判定字」，守它的测试断言「键不随它变」，变异行证明这条测试在登记被撤时会红。独立核「不改判定字」的只有交值那一条，它不看 panic 与副作用（面一 1a）。

### 打中：门禁注释与登记互相矛盾

门禁注释写「crates 里这几行在判器的单元闭包里，动它们换键，所以滤在这里」（`.claude/gate.d/crash-point-replay.sh:314`）。这一批之后打 `GPU_LIFECYCLE` 的三个单元已不在 GPU 判器那一组的闭包里，这句话不再是现状；登记文档又反过来引这道过滤当理由之一（`judging_units_entries.rs` 第 1172 行起那两句）。门禁文件不在这一轮的写范围，交主 agent。

### 2 带行号的登记名：实现员那句成立，陈旧后键不宽不窄，但它是噪声源

- 成立：变异行「判器键：只经只打 GPU 阶段现场那一行的单元够到的那一批登记里录制流操作种类成员表那次宏调用的行号是陈旧的」（`crates/mutations.tsv:8202`）点名 `not_following_the_listed_units_drops_only_the_listed_ones`，19748 的「✓ 点名的 17 条变异都抓到了」（`task-server/task log 19748` 第 182 行，我读过）。`every_listed_unit_of_the_memory_ledger_names_exactly_one_unit_of_the_workspace` 读的是全部登记，也会同时红（推的）。
- 陈旧之后、测试红之前键怎样：宏调用挪到第 79 行，登记的 `@78` 一个单元都对不上，等于没登；而 `@79` 只被 `gpu_work_trace` 连到、它不沿着收，照样不进闭包。键与挪行之前逐字相同，不宽也不窄（推的，依据 1c）。
- 会变窄的只有一种：另一次 `enum_listing_every_member!` 调用恰好落在 `lib.rs` 第 78 行、又被留下的单元用到。那时登记切掉它，键变窄；同时 `the_units_listed_as_reached_only_through_…` 与 `not_following_…` 都红（推的）。窗口只在这两条挂 `gpu` 特性的测试跑之前。
- 噪声：`lib.rs` 第 78 行之前是 `pub mod` 表，加删一个模块就挪行。2026-10-18 起改 `lib.rs` 的提交两次：`baefc645a` 是整份建文件（`@@ -0,0 +1,969 @@`），`490168e59` 在 `@@ -13,0 +14 @@` 与 `@@ -58,0 +60 @@` 各加一行（量的，`git log --since=2026-10-18` 与 `git show --unified=0`）。按这个频率，下一次加模块这条测试就红，而键一个字不变，修法只是改行号。别的会话合一批让它红，是噪声，不是警报。
- 自提的改法（被攻过零轮，推的）：记账测试对「只被登记单元连到」的单元自动豁免（算出每个少掉的单元的全部入边是不是都来自登记的单元），第二批整批不必登记，行号问题随之消失。

### 2b 带行号的名字与去行号的比对：口径没有对不上

登记按带行号的名字逐字比（收闭包），红句按去行号的名字配对（`units_changed_between`）。两处各管各的：前者决定键，后者只决定红句里点不点名，后者不回流进键。查了 `crash_identity.rs` 里 `unit_name_without_line_numbers` 的唯一调用者「        &unit_name_without_line_numbers,」（`crates/singlefs-verification/src/crash_identity.rs:497`），只在 `units_changed_between` 里。没打中。

## 面三：哪一条最脆弱

### 3 两步配对：没打中（附证明）

键只看（种类，指纹）的多重集，「不带路径、名字、行号、手写版本号。」（`crates/singlefs-unit-graph/src/closure.rs:3260`）；红句里每个单元的名字是「<种类> <限定名> @ <文件>」，种类在名字里。两步配对每拿掉一对，两边的名字（或去行号的名字）相同、指纹相同，于是种类与指纹都相同。记两边的（种类，指纹）多重集为 S、N，拿掉的配对为 P（P 同时是 S 与 N 的子多重集），剩下的是 S−P 与 N−P，它们的差就是 S−N 与 N−S：键的差里的每一项都留在剩下的里、被点名，红句不会因为配对变空，也不会漏掉让键变了的那一项。逐个看正文点的三种：

- 挪了行又改了内容：旧名在 S 里、新名在 N 里，指纹不同，两步都配不上，旧名新名各报一个（新测试 `units_named_by_their_line_that_only_moved_are_not_named_as_changed_and_edited_ones_are` 正是这一种）。
- 两条同名单元互换内容（同一份文件里同一个宏的两次调用对调、两个 cfg 臂对调）：多重集不变，键不变，这条红句根本不打。
- 去行号后撞名：只在指纹也相同时配，配上的两个逐字相同，拿掉不丢差。
- 唯一会「点错名」的：同一个宏在一份文件里调三次，三段内容轮换一圈，又另有别处真改了。轮换那三个被配掉，只报真改的那一处。按键的口径轮换不算变，不算漏报。

附一处措辞（不是这一批带的，这一批让它多了一种触发）：清单为空时红句写「"变了的单元：（组成清单比不出：库里那一把或这一趟的组成清单是空表）".to_string()」（`crates/singlefs-verification/src/crash_identity.rs:566`）。键里还有条件原子、数据单元、工具链与依赖行，这几样变了而单元没变时清单同样是空的，红句却说组成清单是空表。配对之后多了一种情形会走到这一句：只挪了行的单元加上工具链或依赖变了，以前会列出挪行的单元，现在说「空表」。建议空清单时写「单元没变，变的是工具链、依赖、条件或数据行」（推的，被攻过零轮）。

### 4b 打中：拷数据文件那个循环没有测试守，变异行名字写的是它、弄坏的不是它

实现员已写「所以删掉 `RepositoryCopy::new` 里拷数据文件那个循环，今天两种编法下都没有测试红。」（`/home/fy5090/.local/share/singlefs/work/ledger-topdown-watched-data-files-repair/result.md:81`）。我补两点：

- 原断言（清单里有 `.claude/heavy-thread-default`）丢掉的守护：原来不带特性那一趟清单里真有一份 `crates/` 之外的文件，读活树的拒算那一批（`the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree` 等）每跑一次都真拷它一次，「for data_file in data_files_outside_the_crates_the_build_read() {」（`crates/singlefs-verification/src/crash_identity.rs:2221`）是有正例的活代码。今天不带特性时清单为空；带 `gpu` 时清单是 `prune-server/src/` 那一批，而「for crate_directory in crate_directories_outside_the_crates_directory(&repository) {」（`crates/singlefs-verification/src/crash_identity.rs:2215`）已整个拷过 prune-server。两种编法下这个循环都只拷已经在的东西，是死代码。
- 变异行「拷树重算 漏拷带 gpu 特性编时构建读到的 crates 之外的文件」（`crates/mutations.tsv:8208`）把交给纯函数的清单截成空的。截空之后拷树一份都不漏（prune-server 已由上面那一路拷进去），测试红只因为它断言了清单的内容。行名说「漏拷」，弄坏的是「清单」：「漏拷就拒算」这件事没有一行变异、也没有一条测试。
- 怎么量：在副本里删掉 `RepositoryCopy::new` 里 `for data_file in …` 那整个循环，跑拒算那一批里任一条（例 `the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree`，不带特性与带 `gpu` 各一次），两次都绿就坐实。草稿目录的同一份副本里已删掉这个循环（`attack-probe.patch` 后半），命令在「要主 agent 跑的命令」第 1、2 条，推的，待主 agent 跑。

### 4c (b)(c) 两条：是警报

`the_build_without_the_gpu_feature_reads_no_file_outside_the_crates` 红在「有一组闭包新够到 `crates/` 之外」，`the_build_with_the_gpu_feature_reads_the_prune_server_library_files_outside_the_crates` 红在「带 `gpu` 时够到 prune-server 之外」。这两种情形正是拷数据文件那个循环重新变成活代码、要有人补正例的时候，红了该看。会让它们误红的只有 prune-server 之外的 crate 被收进来，那同样是新输入进键，该看。不算噪声。没打中。

### 5a OnceLock 与「不用单例」：条款推不出，算打中

条款：「不用全局可变状态；不用单例」（`.claude/rules/code-discipline-by-review.md:54`），在「类型、数据与封装」那一节，没有例外分句。`static VULKAN_INSTANCE_OF_THIS_TEST_PROCESS` 是写一次的进程级单例；正文给的依据是先例「static CARDS_LEASED_ON_DEMAND_FOR_THIS_PROCESS: std::sync::OnceLock<String> =」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:664`），而先例本身同样违反这一条、也没有登记豁免。拿一处没被判过的违例当依据，是循环。

我的判断：这一处该有，条款该补出路。libtest 的同一个测试二进制里，各条 `#[test]` 之间没有别的共享位置（没有进程级的建立钩子），「一个进程只建一个实例」又是驱动的外部约束；判法已经把单例挤到薄壳（核心 `handed_over_gpus_through_the_instance_kept_for_this_process` 收 `&OnceLock`，见 `mod.rs` 第 702 行起），与「副作用挤到边上，核心写成纯函数」（`.claude/rules/code-discipline-by-review.md:20`）的写法一致。第三条路：每条真卡测试一个进程（一个测试一个二进制，或换按进程隔离的测试驱动），那时「一个进程一个实例」不需要静态量；代价是现有真卡测试目标拆分，我没算。要留它，就在条款里加一句出路（外部资源一个进程只许一份、核心经参数收、薄壳持有），连同先例一起登记；不加就是这一批与先例一起违反条款。

### 5b 三条不要卡的测试没罩住的两处

- 薄壳交的不是进程级那一格（实现员已认）：「        &VULKAN_INSTANCE_OF_THIS_TEST_PROCESS,」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:759`）改成 `&std::sync::OnceLock::new()`，三条照绿，真卡进程里每次取卡建一个实例。
- 建实例 panic 之后：「    let vulkan_instance = instance_kept.get_or_init(instance_created).clone();」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:721`）。`OnceLock::get_or_init` 的初始化闭包 panic 时格子留空，下一次调用再建（标准库文档的行为，推的，没在本项目量）。真卡进程里头一条用例建实例中途 panic（驱动没起来、超时），同一个进程的下一条用例会再建一个，正是「一个进程只建一个」要防的。三条测试的替身建实例从不 panic，罩不到。补一条：替身头一次建时 panic、`catch_unwind` 接住，第二次取卡时断言建的次数（零轮，推的，待主 agent 跑）；或在薄壳里把「建过一次」单独记下，第二次照拒（零轮）。

## 第 6 条：八格检查单逐格

| 格 | 判定 | 证据 |
|---|---|---|
| 走不到的理由 | 成立 | `crash_identity.rs` 第 483 到 560 行（`units_changed_between` 与两个新私有函数）里 `expect(`、`unwrap(`、`unreachable!`、`panic!`、`todo!` 命中 0 处（`awk` 加 `grep -cE` 数的）；`mod.rs` 第 702 到 738 行的三处 `panic!` 都在文档「# Panics」里写了，是测试包的取卡判红，不是产品路径 |
| 类型与名字 | 推不出，算打中（轻） | 这一轮给单元名字符串加了第二个解析者：「let base = format!("{} {} @ {file}", unit.kind, unit.qualified_name);」（`crates/singlefs-verification/src/crash_identity.rs:787`）与「format!("{base} #{occurrence}")」（`crates/singlefs-verification/src/crash_identity.rs:793`）拼出的格式，由 `unit_name_without_line_numbers` 按「`@` 后的数字」拆。格式只在字符串里约定，哪天同名后缀改成 `@<n>`、或种类、限定名里出现别的 `@<数字>`，配对会把不同单元当成同一个，没有类型与测试拦。今天 `singlefs-unit-graph/src` 里拼 `@` 加行号的地方只有 `rust_items.rs` 与 `wgsl.rs` 的九处（`grep` 核的），所以今天不错 |
| 不兼容旧代码与旧格式 | 成立 | 四份文件的 diff 里没有认旧版本的分支；第二步配对不是为旧库留的读法，两边都是这一版的名字 |
| 不新建判定库 | 成立 | 新测试经 `keys_before_and_after`（`RepositoryCopy::new` 建一次性临时目录）重算键，不存判定 |
| 所有任务多机可用 | 成立 | (b)(c) 两条断言的是编法决定的清单，路径是仓内相对路径；新测试文件的地址用文档地址段 `192.0.2.1`、`192.0.2.2`；没有主机名与绝对路径进断言 |
| 测试优先多线程 | 成立 | 拷活树的各用自己的标签建临时目录；装工作区照 `workspace_loading_threads_from_the_test_process_share` 取线程，没有写死的线程数 |
| 带 --item 的命令与项 | 成立 | 读了 `task-server/task log 19748`：第 182 行「✓ 点名的 17 条变异都抓到了」；同一份第 180 行写这一趟「现判 1 条、复用 6 条、拒算 10 条」，17 行里只有一行是这一次现跑的，其余按键复用或拒算照跑。19724 的 `--list-items` 输出我没读 |
| 新崩溃流多罩了什么 | 成立 | 没有新的崩溃点重放流与崩溃枚举用例 |

## 要主 agent 跑的命令

副本 `/home/fy5090/.local/share/singlefs/work/code-r1-judging-closure-attack/copy`（`minimal-copy.py` 从主工作区建，`.deliverables` 写成交接）上做了 `attack-probe.patch` 里的两处改动与两条探针测试。副本上的数是副本的数。两条都点名验证包的库测试，没登记成崩溃枚举用例，按不走崩溃枚举写；被范围声明闸拒、说明点名要缩域或全域理由的，照说明改。

1. 带 `gpu` 特性的九条（不碰卡）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 带 gpu 特性的测试目标 lib 里的 9 条测试; 范围=9 个测试; 为什么=攻方腿副本上 GPU 阶段现场那一行取线程名改成 expect、删掉拷数据文件的循环：守登记的四条、键不变的两条、单测与读活树重算键那一条照绿，没名字的线程上那一条红' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/code-r1-judging-closure-attack/copy -- cargo test -p singlefs-verification --features gpu --lib -- crash_identity::tests::not_following_the_listed_units_drops_only_the_listed_ones crash_identity::tests::no_unit_the_gpu_judge_group_keeps_reads_a_value_returned_by_a_listed_unit_that_only_prints_the_gpu_lifecycle_line crash_identity::tests::the_units_listed_as_reached_only_through_the_units_that_print_the_gpu_lifecycle_line_are_reached_through_them_and_by_no_unit_the_group_keeps crash_identity::tests::every_listed_unit_of_the_memory_ledger_names_exactly_one_unit_of_the_workspace crash_identity::tests::an_edit_of_the_listed_gpu_lifecycle_line_keeps_the_gpu_judge_key crash_identity::tests::an_edit_making_the_listed_gpu_lifecycle_line_panic_on_threads_without_a_name_keeps_the_gpu_judge_key crash_identity::tests::the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree gpu_unit_checks::tests::the_gpu_lifecycle_line_names_its_time_keys_in_full gpu_unit_checks::tests::the_gpu_lifecycle_line_on_a_thread_without_a_name_does_not_panic --exact`
   - 在哪起：副本根（上面那个路径）。
   - 看：`TASK_SERVER_RESULT` 的 `exit=` 非 0 是预期；`task-server/task log <号> --all` 里 `test result: FAILED. 8 passed; 1 failed`，失败的只有 `gpu_unit_checks::tests::the_gpu_lifecycle_line_on_a_thread_without_a_name_does_not_panic`，panic 消息里有「GPU 阶段现场那一行只在有名字的测试线程上打」。前面四条或 `an_edit_making_…` 有任何一条红，1a 撤回；`the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree` 红，4b 撤回。
2. 不带特性的一条
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 lib 里的 1 条测试; 范围=1 个测试; 为什么=攻方腿副本上删掉拷树重算里拷数据文件的循环，核不带特性时读活树重算键那一条照绿、没有测试守这个循环' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/code-r1-judging-closure-attack/copy -- cargo test -p singlefs-verification --lib -- crash_identity::tests::the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree --exact`
   - 在哪起：副本根。
   - 看：`exit=0`，log 里 `test result: ok. 1 passed`；红了 4b 撤回。

## 没打中的形状

- 两步配对：试了挪行又改内容、同名互换、去行号撞名、三次调用轮换、插一次相同内容的调用、`#<n>` 同名后缀挪位，六种都按上面的多重集论证走过一遍，没有一种让键的差里的单元不被点名。没写模型，取样是这六个手造的形状。
- 第二批登记切东西：四个单元在第一批登记生效时没有被沿着收的入边，登不登键一样；唯一会切到的情形同一趟测试红。
- 线程数：`:31746` 那一路逐张盘独立判、按次序接回，切法变不了结论；另三处产品调用我没核出反例，也没核成立。
- 读活树、挂 `gpu` 特性的五条没进拒算表：拒算是 `admission.py` 现算的（读 `CARGO_MANIFEST_DIR` 拼出的仓根的闭包判拒算），表只登不带特性那一趟。它们在带特性那一趟照跑、不复用（推的，没跑 `admission.py test-plan` 带 `gpu` 的那一份核），不构成键过窄。

## 这条腿自己的限度

- 1a 的「判器线程没名字就 panic」只在副本上用一条没名字的线程模拟，没上卡、没在 GPU 服务里跑；`scope.spawn` 线程没名字是标准库的行为，我没在本项目量。
- 1b 的边解释读 `closure.rs` 文件头推的，没打出闭包的边。
- 两步配对只做了证明与手造形状，没写枚举模型。
- 实现员报告与正文里给的任务读数（19725、19764）我没复核，只读了 19748 的两行。

## 没做什么

- 没交任务、没编译、没跑测试；两条命令在上一节。
- 不判正推那一边；不替主 agent 采纳；副本上的数不算入库装置的数。
- 门禁注释（`.claude/gate.d/crash-point-replay.sh:314`）与条款补出路不在写范围，交主 agent。
- 副本 `/home/fy5090/.local/share/singlefs/work/code-r1-judging-closure-attack/copy`（57M）留作交接，没删；草稿目录没有编译目录。

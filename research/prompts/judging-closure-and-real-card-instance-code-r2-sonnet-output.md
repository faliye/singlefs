# 辩方腿：judging-closure-and-real-card-instance-code-r2（2026-11-18）

复核对象：`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md`（下称「判决」）。依据的腿报告：`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md`（攻方，下称「攻方报告」）、`research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md`（正推，下称「正推报告」）。实现员原报告两份：`~/.local/share/singlefs/work/code-r1-judging-closure-repair/result.md`（修补）、`~/.local/share/singlefs/work/ledger-topdown-judging-closure-writer-second/result.md` 与 `~/.local/share/singlefs/work/ledger-topdown-watched-data-files-repair/result.md`（被判的那一批）。

开工核了三件事：
- 开工快照三份清单 `sha256sum -c`：六行（`crates-sha256.txt` 三份、`judged-sha256.txt` 两份里的正文与判决、`kb-sha256.txt` 一份）全 OK（量的）。
- 判决里的任务号 19825、19832 用 `task-server/task log <号> --all` 与 `task-server/task status <号>` 读过（只读）。
- 没读禁读清单里的文件。一轮没写模型、没起项目代码。

## 判定一览

| 判决条目 | 辩方判定 | 一句话 |
|---|---|---|
| 判决:17、:18 探针读数 | 站得住 | 任务号、条数、红的那一条与调度记录:262 逐处对得上；但读数在攻方腿副本上量，量的只有「panic」那半 |
| 判决:27 守「只打日志不改判定结局」缺 panic 与可变借用两条路 | panic 一半站得住（量的）；借用一半推不出（没量）；只对「登记」这一种写法成立，不分辨臂地打中别的写法 | 留在闭包里就没有这个缺口；同一形状在更早登记的显存记账那一批同样存在，判决没提 |
| 判决:33 拷数据文件的循环没有守 | 站得住（量的），但「没有东西守」说宽了 | 摘要比对是有条件的守，今天没有正例；清单里一旦有 `crates/` 之外又不在已拷目录里的文件，现有两条会红 |
| 判决:29 带行号的登记名记成已知代价 | 站得住 | 键不看名字与行号，陈旧登记不改键；先例已有三处；判决没回应 kb 条款里「行号不进键、不当复用依据」那句 |
| 判决:30 红句「空表」措辞不改 | 站得住，但「先前就有」不全：配对改动让触发更常见 | 要改得先分清「清单比不出」与「清单两边相同」，要动正在被别的会话大改的文件 |
| 判决:38、:52 单元名只在字符串里约定不改 | 站得住一半 | 判决只拿「立 newtype 是另一件」回应；更轻的一条钉格式的测试没被考虑 |
| 判决:32、:52 `checking_threads()` 另外 3 处没人核、转出去 | 站得住，理由比判决写的更强 | 线程数本来就由环境变量改而不进键；这三处今天推得出不在任何一组闭包里 |
| 判决:28、:32 撤回实现员两句 | 撤回对；两句各有一半其实成立 | 「name 算见过」作为「也」成立；「唯一读」没有范围限定时不成立 |
| 判决:30 两步配对不漏报、这一格停 | 停成立（照规则字面），不需要再派同立场腿 | 只有一条腿覆盖到的几种形状逐个列在节「两步配对」，停不靠它们 |
| 判决「三、改法」1 到 5 的落实 | 做到了字面；两处少做、一处多做 | 见节「修补对照」逐条文件:行 |

下面五节对应派发的五条。每节：结论、证据、替另一方写的最强理由与判定。

## 节「坐实的两处与探针读数」（对应派发第 1 条）

### 结论

- 任务号与读数对得上：判决:17、:18 与调度记录:262 说的 19825、19832，与任务服务里的状态与日志逐处相同（量的，下面「证据」）。
- 判决:27 把「panic」与「经借用改卡的状态」两条路并在「坐实（量的，任务 19825）」下面；量到的只有 panic 那一条。改卡状态那一条没有读数（推的）。判「判决站得住」限于 panic 一条，借用一条判「推不出」。
- 两处缺口对「别的写法」是否同样成立：
  - 缺 panic 与借用守护：只对「把单元登成不沿它收」成立。把这三个单元留在闭包里，改它们的函数体就改键，缺口不存在。所以这一打中分辨得出臂（不是「打中不分辨臂」那一形态），判决据此保留登记、补守护，方向没错。
  - 同一形状在更早登记的显存记账那一批同样存在，判决没提（下面「同样打中别的批」）。
  - 拷数据文件循环没有守：对「今天清单为空或被整目录拷贝罩住」成立，与登记无关；对「删掉这段死代码」这一替代方案不成立（下面）。

### 证据

- 任务 19825：`task-server/task status 19825` 打「退 101（command-failed）」；`task-server/task log 19825 --all` 里 `running 9 tests`、红的一条是 `gpu_unit_checks::tests::the_gpu_lifecycle_line_on_a_thread_without_a_name_does_not_panic`，收尾 `test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 1055 filtered out; finished in 145.86s`；panic 点 `gpu_unit_checks.rs:1055:23`，线程名 `<unnamed>`（以上是任务日志原样，量的）。
  - 守登记的四条（`not_following_the_listed_units_drops_only_the_listed_ones`、`no_unit_the_gpu_judge_group_keeps_reads_a_value_returned_by_a_listed_unit_that_only_prints_the_gpu_lifecycle_line`、`the_units_listed_as_reached_only_through_…`、`every_listed_unit_of_the_memory_ledger_names_exactly_one_unit_of_the_workspace`）、`an_edit_of_the_listed_gpu_lifecycle_line_keeps_the_gpu_judge_key`、探针 `an_edit_making_the_listed_gpu_lifecycle_line_panic_on_threads_without_a_name_keeps_the_gpu_judge_key`、`the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree`、`the_gpu_lifecycle_line_names_its_time_keys_in_full` 都是 `ok`：8 条，与判决:17 的列法一致。
- 任务 19832：`task-server/task status 19832` 打「退 0（succeeded）」；日志里 `the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree ... ok`、`test result: ok. 1 passed`（量的）。删循环的那处改动在攻方腿草稿的 `attack-probe.patch`（`~/.local/share/singlefs/work/code-r1-judging-closure-attack/attack-probe.patch`，77 行，最后一块 `@@ -2218,18 +2230,6 @@` 删 `for data_file in data_files_outside_the_crates_the_build_read() {`；仓外文件，引文核对不判）。
- 调度记录:262：「19825 退 101」（`records/2026-11-17-subagent账本清欠调度记录.md:262`），同行写 19832 退 0。
- 判决:27：「守这件事的检查不够，坐实（量的，任务 19825）」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:27`）。判决:33：「「漏拷 `crates/` 之外的数据文件就拒算」没有东西守，坐实」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:33`）。
- 借用一半没量：攻方报告只给了推的例子——「同一类的别的改动（推的，没量）」（`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:34`）。例子是在登记的单元里加一句经 `card.device_condition()` 记设备丢失。这条路确实走共享借用加内部可变：`gpu_lifecycle_trace` 收 `card: &GpuCard`（「pub(crate) fn gpu_lifecycle_trace(card: &GpuCard, phase: &str, detail: std::fmt::Arguments<'_>) {」，`crates/singlefs-verification/src/gpu_unit_checks.rs:1007`），卡上「pub fn device_condition(&self) -> &DeviceConditionOfACard {」（`crates/singlefs-verification/src/gpu_unit_checks.rs:1121`），记丢失的方法收 `&self`（「pub fn record_the_device_lost(&self, why: String, cause: CauseOfTheDeviceLoss) {」，`crates/singlefs-verification/src/gpu_memory_ledger.rs:2228`）。
- 判决:27 的「panic」一半在「没有名字的线程」上才红：判器的 `scope.spawn` 与 GPU 服务的卡工作线程确实没用 `Builder` 起（「let recovery = scope.spawn(recovery);」，`crates/singlefs-verification/src/crash_judge_dispatch.rs:1308`；「scope.spawn(move || {」，`gpu-verification-server/src/service_loop.rs:270`）；那一带没有 `thread::Builder`（`grep -n 'thread::Builder'` 对这两份文件 0 命中）。标准库里 `scope.spawn` 起的线程无名是标准库的行为（推的，没在本项目量）。这与判决:19 写的「核不动」一致，不影响缺口本身：缺口是「没有测试读登记单元的函数体」，与线程有没有名字无关。

### 同样打中别的批（判决没提）

- 登记文档写「判据与显存记账那一批里打设备丢失标记行的单元相同：不改崩溃状态的判定字」（`crates/singlefs-verification/src/judging_units_entries.rs:1168`）。显存记账那一批是常量 `UNITS_OF_THE_MEMORY_LEDGER_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`（`crates/singlefs-verification/src/judging_units_entries.rs:848`），里面有 `record_the_device_lost` 这类改 `DeviceConditionOfACard` 的单元。
- 新守护只遍历新常量：「let listed_units = crate::judging_units_entries::UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW;」（`crates/singlefs-verification/src/crash_identity.rs:2011`）。更早那一批的前提靠「够到的恰好是登记的」（`the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones`）与「少掉的恰好是登记的」（`not_following_the_listed_units_drops_only_the_listed_ones`）守，登记文档自己也这么写（`crates/singlefs-verification/src/judging_units_entries.rs:848` 上方的文档），没有 panic 与借用守护。
- 这不是说那一批有错（那一批登记时按什么判的，我没读它的三方判决）；说的是判决的打中形状（登记的「不改判定结局」前提没有读函数体的检查）对它同样成立，判决与修补都只落在新三个单元。要不要扩到更早那一批，是主 agent 的取舍，我不替它判。

### 替「该留在闭包里、不登」那一方写的最强理由，与判定

- 最强理由：留在闭包里，键跟着函数体走，panic、借用、内部可变全是闭包算出来的，不需要任何文本判法；登记本身是用「文本判法」换「少现判」，而文本判法认不全（实现员自己列了认不出的六类，见 `crates/singlefs-verification/src/crash_identity.rs:1810`、`:1856` 的文档）。登记前提的证明责任在登记一方。
- 对它的压力：登记的理由是日志每改一个字、这一组的键就变、每一块都跟着现判——「它们进键时改一个字，这一组的键就变」（`crates/singlefs-verification/src/judging_units_entries.rs:1173`）。判决没给这个代价的量，我也没量。
- 判定：压不过，也没被判决压过。判决的做法（保留登记、补文本守护）是在「保键稳」这一边选了更省的一条，缺的是代价对照；它没有证明「不登」更差，也没有证明登记加守护比「不登」更省。判决写的是「守的检查不够」，没写「该不登」，我判判决「站得住」限于它实际写的那句。
- 推翻它的现象：量一次「不登」时改一个日志字让 GPU 判器那一组现判多少块、多长时间；若比「文本守护加登记」的维护成本低一个量级，「不登」更优。

### 替「删掉拷数据文件的循环」那一方写的最强理由，与判定

- 最强理由：清单今天不带特性是空的、带 gpu 时被整目录拷贝罩住（正推报告：不带特性是 `[]`，见 `research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md:178`），循环在两种编法下都拷不出新东西，是死代码；给死代码补测试是加负担。
- 对它的压力：现有的摘要比对是有条件的守——「它是有条件的守：只要有一份被监视的文件拷贝里没有，那两条就红」（`research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md:203`）。比对函数 `assert_the_watched_files_are_the_ones_the_build_read`（`crates/singlefs-verification/src/crash_identity.rs:2732`）在 `crash_identity.rs:2835` 与 `:3270` 两处调。将来清单里真出现 `crates/` 之外又不在已拷目录里的文件，循环又被删，这两条会红，但红句是「树在编译后变了」（`crates/singlefs-verification/src/crash_identity.rs:2739`），不是「漏拷」；删掉循环等于把一个能给出准确红句的位置换成一个误导的红句。
- 判定：压得过。判决保留循环、补正例（判决:46）比删掉更稳；但判决:33 的「没有东西守」说宽了，应写「今天没有正例，将来有数据文件时摘要比对会红、红句不准」。
- 推翻它的现象：清单里将来永远只有被整目录罩住的文件，循环永远不用；那时删掉更省。判不了的是「将来」，所以保留。

## 节「判决认下而没改的四件」（对应派发第 2 条）

判决「三、改法」第 8 条把四件列进「不改、记账」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:52`）。判决给理由的只有其中三处半句：带行号的登记名「不算打中，记成已知代价」（:29）；红句「这是先前就有的措辞」（:30）；单元名「立 newtype 是另一件」（:52）；`checking_threads()`「转给今天改了那一处的会话」（:52）。下面逐件把理由补全再判。

### 带行号的登记名的代价

- 结论：判决「不改」站得住。
- 证据：
  - 键不看名字与行号：「不带路径、名字、行号、手写版本号。」（`crates/singlefs-unit-graph/src/closure.rs:3260`）；收闭包按限定名逐字比：「if self.units_not_followed.contains(&unit.qualified_name) {」（`crates/singlefs-unit-graph/src/closure.rs:1389`）。陈旧的登记名一个单元也对不上，等于没登；那次宏调用本来就只经不沿它收的 `gpu_work_trace` 够到，没登它也不进闭包，键不变。登记名在 `crates/singlefs-verification/src/judging_units_entries.rs:1200`，宏调用在 `crates/singlefs-verification/src/lib.rs:78`，今天两边对得上（量的：`grep -nF`）。
  - 红的是挂 `gpu` 特性的测试；两条腿都点了窗口与代价：「窗口只在这两条挂 `gpu` 特性的测试跑之前」（`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:96`）、「这是噪声，不是错」（`research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md:114`）。
  - 先例：同一份登记里包装设备那一批已有三处带行号的宏调用名，行号更大（「"singlefs_verification::history::enum_listing_every_member!@134",」`crates/singlefs-verification/src/judging_units_entries.rs:1081`）。`lib.rs` 自 2026-10-18 起被改 2 次（量的：`git log --since=2026-10-18 -- crates/singlefs-verification/src/lib.rs` 两行；两次各改了什么是攻方报告写的，我没复核）。
- 替「该在这一题里改」那一方写的最强理由：kb 写「一律不进键、也不许当复用依据：……路径（仓根也算）、名字、种子串、行号、资源上限、限时」（`.claude/kb/decisions/13-验证路线.md:436`）；用行号决定哪个单元不沿它收，是行号在决定键的内容，字面上碰了这句；而陈旧只有挂 `gpu` 特性的测试才看得见，不挂特性的开发循环里看不见，红会落在改 `lib.rs` 的无关改动头上。
- 判定：压不过。那句管的是键的输入；键里没有行号，登记名决定的是「哪个单元被排除」，排除之后键仍只由（种类，指纹）组成。先例三处在登记里已有（那一批登记的文档点名了一份三方判决，管的是「按组登记」；它有没有专门判过带行号的名字，我没读，不下结论）。判决没回应这句 kb 条款，应补一句「键里没有行号，登记名按行号只管排除」，不是补改法。
- 推翻它的现象：陈旧登记在红之前让键变窄或变宽（攻方报告写的唯一变窄路径是另有一次同名宏调用恰好落在 78 行又被留下的单元用到，同一趟的 `not_following_the_listed_units_drops_only_the_listed_ones` 与 `the_units_listed_as_reached_only_through_…` 一起红）。

### 红句「空表」措辞

- 结论：判决「不改、记账」站得住；它给的理由「先前就有」不全。
- 证据：
  - 句子：「"变了的单元：（组成清单比不出：库里那一把或这一趟的组成清单是空表）".to_string()」（`crates/singlefs-verification/src/crash_identity.rs:566`）。它有两种来历：库里读不到组成清单，调用方 `.unwrap_or_default()` 成空表（「.unwrap_or_default();」`crates/singlefs-verification/src/crash_amplification.rs:27882`，传给 `units_changed_between` 的调用在 `:27888`），这时「是空表」是真话；两边都有清单而且相同、键因工具链、依赖、条件原子、数据单元或入口行变了，这时是错话。
  - 配对改动让后一种更常见：「改成两步配对之后更常见」（`research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md:154`）；「配对之后多了一种情形会走到这一句」（`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:115`）。判决:30 只写「先前就有」，没写这一点。
- 替「该在这一题里改」那一方写的最强理由：两步配对是这一批带的，它让这句错话多一种触发（只挪了行、又恰好工具链变了：以前列出假名字，现在说「空表」）；红句的用处是给人判哪几个单元变了，说错原因会把人引去查库。一行字符串的事。
- 判定：压不过。改它不是改一行字符串：要分清「读不到」与「两边相同」，现在 `units_changed_between` 交的是 `Vec<String>`，两种来历都交空表；要改签名或让调用方先判有没有读到，落在 `crates/singlefs-verification/src/crash_amplification.rs`，而这份文件今天有别的会话的大批未提交改动（`git diff --stat HEAD` 这份文件：3757 行加、1503 行减，量的），这一轮不该动它。测试只钉前缀（「changed_units_sentence(&units_changed_between(&[], &[])).starts_with("变了的单元：")」，`crates/singlefs-verification/src/crash_identity.rs:3574`），改措辞不会被现有测试拦，也不会被它守住。
- 判决要补的：记账时写明「两步配对让它更常见」，并把「分清读不到与相同」写成要改什么，不写成措辞。
- 推翻它的现象：改措辞不需要动调用方（例如「清单是空表或与库里的清单相同」两种都包在句子里）；那样一行改完，「不改」站不住。

### 单元名只在字符串里约定

- 结论：判决「不改」站得住一半。它拿「立 newtype 是另一件」挡掉了重的一条，轻的一条没被考虑。
- 证据：
  - 判决自己把它判成打中：「单元名的格式（限定名里的 `@<行号>`）只在字符串里约定」「算打中」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:38`）；同一份判决:52 又把它列进「不改」。「推不出算打中」与「坐实的就改」之间，这一件靠「立 newtype 是另一件」过渡。
  - 今天产出格式的地方（量的，`grep -rnE '@\{' crates/singlefs-unit-graph/src/rust_items.rs crates/singlefs-unit-graph/src/wgsl.rs` 8 行）：`rust_items.rs` 7 处（例：「format!("{}!@{}", invocation.macro_name, self.line_of(header)),」`crates/singlefs-unit-graph/src/rust_items.rs:2574`，同一句另在 `:2920`）、`wgsl.rs` 1 处（「let name = format!("{keyword}@{}", scanner.line_of(index));」`crates/singlefs-unit-graph/src/wgsl.rs:521`）。攻方报告写「九处」（`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:145`），今天数得 8 处。
  - 今天按格式拆它的地方：验证包里 `unit_name_without_line_numbers`（「fn unit_name_without_line_numbers(unit: &str) -> String {」`crates/singlefs-verification/src/crash_identity.rs:549`）；单元图自己里也有一处（「.split_once("!@")」`crates/singlefs-unit-graph/src/workspace.rs:1292`）。先例：这种按字符串约定的耦合在产品代码里早就有。
  - 测试里的名字是手写的（「unit(&format!("use singlefs_verification::history::use@108 @ {history}"), 1),」`crates/singlefs-verification/src/crash_identity.rs:3644`），不是从单元图产出的：产出格式变了（例如 `use@L108`），这条测试照绿，真库里去行号这一步静默失效。
- 替「该在这一题里改」那一方写的最强理由：newtype 是重的一条，另有轻的一条：拿编进二进制的真组成清单（带 gpu 编时的 GPU 判器那一组、不带特性时别的组）断言「每个 `@` 之后紧跟的要么是空格（名字与文件之间）要么全是数字」，格式一变就红；它不读活树，在判决「三、改法」第 8 条里没被考虑。
- 判定：不改站得住，理由要换：今天正确（两条腿都核过，我再读了产出处与拆处，没有 `@` 后接非行号数字的写法），先例已有，改 newtype 要动单元图与它的 python 孪生（孪生里也有产出与拆分：「unit = self.add("item_macro", f"{name}!@{self.lines[name_index]}", start, end, context, module_path, container, conditions, True)」`scripts/gate/lib_function_units.py:3125`、「.partition("!@")[0]」`scripts/gate/lib_function_units.py:4398`），不是这一轮的事。但「立 newtype 是另一件」没回应轻的那一条；记账时应写成两件：newtype 另立一题，钉格式的真清单断言记「零轮」候选。
- 推翻它的现象：产出处或拆处出现 `@` 后接非行号的数字（例如以后某种单元名带 `@2` 表示序号）；今天 `grep` 到的 8 处与 1 处拆分都不是。

### `checking_threads()` 另外 3 处调用没人核

- 结论：判决「转给今天改了那一处的会话」站得住，理由比判决写的更强；判决漏记两件事：行号已漂移、剩下没核的只是一件事。
- 证据：
  - 判决:32：「攻方腿数出产品代码里有 4 处调用，另外 3 处两条腿都没核出结论」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:32`）。今天产品代码里仍是 4 处，但这份文件已不是腿读的那份：被改的那处从 `:31746` 移到 `:32438`（「crate::crash_library::InnerThreadsTaken::take(checking_threads().min(image_count).max(1));」`crates/singlefs-verification/src/crash_amplification.rs:32438`），另三处在 `:2495`（「        checking_threads(),」）、`:24648`、`:31518`（两处都是「    let threads = checking_threads();」）；单测里 `:18719`、`:18985` 两处，`crates/singlefs-verification/tests/` 下另有两个测试文件各一处（`crash_amplification_fixed_script_through_rollback.rs:318`）。这份文件没在两轮的开工快照里，今天 `git diff --stat HEAD` 列它 3757 行加、1503 行减（量的）；接手的会话要拿今天的行号，不拿判决里的。
  - 另三处只是把数交给带 `_with_threads` 的同名函数：「核对线程数由调用方给（崩溃枚举用例的判段照 `SINGLEFS_CRASH_POINT_REPLAY_THREADS`）」（`crates/singlefs-verification/src/crash_amplification.rs:2499`）。也就是同一份库里的判定本来就被不同线程数的调用方复用。
  - 线程数本来就不在任何键里：它由环境变量定（「pub const CHECKING_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_THREADS";」`crates/singlefs-verification/src/crash_amplification.rs:2382`），没设取每台默认重活线程数、再没设取登记文件（「没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取」`crates/singlefs-verification/src/crash_amplification.rs:2384`）。登记文件原来能进键，只因为它经 `include_str!` 编进来、闭包够到它（「pub const REGISTRATION_FILE_TEXT: &str = include_str!(concat!(」`crates/singlefs-verification/src/heavy_threads.rs:22`）；环境变量一直改线程数而键不动。所以「登记文件进键」从来不是线程数进键，只是巧合。
  - 今天没有一组的闭包够到它：不带特性的清单是空的、带 gpu 的 34 份都在 `prune-server/src/` 底下（正推报告：「带 `gpu` 是 `prune-server/src/` 底下 34 份」，`research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md:178`），登记文件不在里面。若某一组的闭包够到 `checking_threads`，它的登记文件会出现在清单里。这条推理攻方报告已写（「另三处：不带特性那一趟被监视的清单是空的，说明没有一组的闭包够到 `checking_threads`（推的）」，`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:71`）；判决:32 没把它写进去。我核了链上两环（登记文件经 `include_str!` 进、默认值取它），没核「闭包够到 `checking_threads` 就一定带上 `REGISTRATION_FILE_TEXT`」那一环（要单元图 `why`，写在末节「没读到的」）。
- 替「该在这一题里核」那一方写的最强理由：线程数前提是「删掉读登记文件那句」能成立的支点；四处调用只核了一处，其余三处的调用方把数交给哪个入口、入口的结论会不会随条数变，没人读。「推不出算打中」，转出去不等于核过。
- 判定：压不过，但要把「没核」写成两件：键过窄这一件今天推得出不成立（推的：线程数从来不靠登记文件进键、三处今天都不在任何一组闭包里）；剩下没核的是「这三处喂的入口，结论是否随线程数变」，那是入口自己的性质，与这一轮改的拷树重算无关，也与登记文件进不进键无关。转给改那一处的会话合理，但要同时给它今天的行号。
- 推翻它的现象：单元图 `why` 查出任一组闭包里有 `checking_threads` 或 `heavy_threads` 的 `REGISTRATION_FILE_TEXT`；或三处喂的入口里有一个把条数写进结论（例：「第几段」进了块判定字）。

## 节「撤回实现员的两句」（对应派发第 3 条）

判决撤回的两句：「实现员对边的解释撤回一半」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:28`）、「实现员「唯一读 `checking_threads()`」那一句撤回」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:32`）。

### 第一句：方法名 `name` 由 `thread.name()` 算见过

- 结论：撤回对，但被撤回的是「靠它」，不是「它也让 `name` 算见过」；实现员那句里带的括注是对的，而且是判决后面「更脆」那条结论的真正支点。
- 实现员原话：「`name` 由 `gpu_lifecycle_line` 的 `thread.name()`（`gpu_unit_checks.rs:1055`）算见过（别的留下的单元写没写 `.name()` 不影响：类型不被点名就不推）」（`~/.local/share/singlefs/work/ledger-topdown-judging-closure-writer-second/result.md:24`，仓外文件，引文核对不判）。
- 证据：
  - 留下的入口里有同名的方法调用：「self.submit_one_pass_and_wait(&passes, pass.name(), pipeline, None),」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:1726`，另有 `:1811` 一处），在 `judge_one_dispatch` 里（「fn judge_one_dispatch(」`crates/singlefs-verification/src/crash_judge_dispatch.rs:1633`），它是登记的入口（「"singlefs_verification::crash_judge_dispatch::GpuCrashJudge::judge_one_dispatch",」`crates/singlefs-verification/src/judging_units_entries.rs:68`）。
  - 「见过」是全局累积的：`follow_methods` 对每个访问到的单元把方法名记进 `methods_seen`（「if !self.methods_seen.insert(method_name.clone()) {」`crates/singlefs-unit-graph/src/closure.rs:1970`）。所以 `name` 不靠 `gpu_lifecycle_line` 就已见过。判决:28 说这条是攻方腿读的、主 agent 没另核；我读了 `:1726`、`:1811` 与 `closure.rs:1970`，对上。
  - 正推腿读成「`name` 是「见过」的因为 `gpu_lifecycle_line` 里写了 `.name(`」（`research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md:88`），也是「也」的读法，没写「只有」。
- 被撤回的那句有没有哪半句成立：「`thread.name()` 让 `name` 算见过」作为充分条件是真话：`gpu_lifecycle_line` 在不带登记收闭包时被访问，`thread.name()` 就把 `name` 记进去。不成立的是它是唯一路径。实现员括注「别的留下的单元写没写 `.name()` 不影响：类型不被点名就不推」成立：边的另一端（类型被点名）才是门，攻方报告「哪个留下的单元写一次 `RecordedOperationKind`，`name` 那一条立刻连上」（`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:56`）说的是同一件事。
- 判决少交代的一处：登记文档今天仍写「`gpu_lifecycle_line` 里 `thread.name()` 让方法名 `name` 算见过」（`crates/singlefs-verification/src/judging_units_entries.rs:1189`）。判决「三、改法」没有一条叫改它，所以这句撤回的话还在代码文档里。它不是错话，是不全的话；补一句「留下的入口 `judge_one_dispatch` 里 `pass.name()` 同样让它算见过」即可。归写回员或主 agent，我不改。
- 推翻它的现象：`closure.rs` 的 `follow_methods` 只在「单元在闭包里」时记方法名而 `judge_one_dispatch` 不在闭包里（它是登记的入口，`judging_units_entries.rs:68`）；或 `pass.name()` 的 `name` 被解成别的方法名（不会，方法名按字面）。

### 第二句：「唯一读 `checking_threads()`」

- 结论：撤回对。没有范围限定时这句话是假的；加上范围限定后它是否成立，没有人核过。
- 实现员原话：「`:31742` `fn images_after_the_observers_cut_into_pieces_for_the_cpu(` 是唯一读 `checking_threads()` 的地方（`:31746`），只用它算 `pieces`。」（`~/.local/share/singlefs/work/ledger-topdown-watched-data-files-repair/result.md:16`，仓外文件，引文核对不判）。
- 证据：同一份文件里产品代码调用 4 处（前面「`checking_threads()` 另外 3 处」一件里列了今天的行号）。句子里的「唯一」没有范围限定，字面是假的。
- 被撤回的那句有没有哪半句成立：「只用它算 `pieces`」对被改的那一处成立：「images_after_the_observers_cut_by_the_threads_taken(image_count, threads_taken.threads());」（`crates/singlefs-verification/src/crash_amplification.rs:32440`；这份文件今天与腿读的不同，切段函数的文字也变了，切段前另有核「段恰好罩全盘的下标」一句，`crates/singlefs-verification/src/crash_amplification.rs:32467` 的文档写的）。实现员报告紧接着的一段把切段函数不在那一组的闭包里单独交给了主 agent，说明他自己心里的范围是「那一组入口的调用链」；在那个范围内「唯一读」也许成立，但这要单元图证，字面没有写。
- 判决:32 的取舍对：撤回「唯一」，保留对被改那一处的前提（两条腿读码都判成立，正推报告 `:190` 起的切段与按段次序收回）。
- 推翻它的现象：没有（它是个措辞问题）；若主 agent 要「改限定后的句子成立」，要单元图 `why` 证那一组闭包里没有别的 `checking_threads` 读者。

## 节「两步配对不漏报、这一格停」（对应派发第 4 条）

判决:30：「两条腿都没打中（攻方腿给了「每次只拿掉种类与指纹都相同的一对，剩下的差与键的差一项不少」的证明，正推腿正推四种输入都不漏报）。这一格停。」（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:30`）。

### 结论

- 「停」成立（照规则字面）：这一格两条腿都答了、都没打中。不用再派同立场腿。
- 「停」不靠任何一条腿独有的形状：独有的几种形状都是两条腿各自给过的同一个多重集证明的特例（下面逐个列）。
- 但「两条腿各覆盖到什么」判决没写，写进下面。有一处两条腿的措辞不同、判决没记：名字归属可能点到另一个内容逐字相同的单元（不是漏报）。

### 规则怎么说

- 「两条腿都没打中，就记「没打中」、这一题停，不为它再派同立场腿」；「只有一条腿覆盖到的范围（分工只分给其中一条的格、只有一条读了的文件），要拿那一条腿的「没打中」当依据时，用同一份提示再派一条同立场腿」（`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`）。
- 同一节另写「几份样本一不一致，按每一格答的是哪件事判」：先把两条腿的答案对到同一件事上，只有一份答到的件照只有一份办。
- 分工：正文要腿判的第 3 条（「`units_changed_between` 的两步配对……漏报真变了的单元」，`research/prompts/_judging-closure-and-real-card-instance-code-r1-body.md:37`）两条腿都收到；攻方报告的对应节是「3 两步配对：没打中（附证明）」（`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:106`），正推报告的是「问题 2」。这一格不是「分工只分给其中一条」的格。

### 两条腿各覆盖到的范围

表里与后文的 `opus-output.md:N`、`sonnet-output.md:N` 是简写：指攻方报告（`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md`）与正推报告（`research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md`）的第 N 行，行号我都用 `grep -nF` 取过。

| 形状或前提 | 攻方腿 | 正推腿 | 只有一条覆盖？ |
|---|---|---|---|
| 一般性证明：每拿掉一对，两边的（种类，指纹）各少一个，剩下的差与键的差相等 | 有（`opus-output.md:108`「键只看（种类，指纹）的多重集」起一段） | 有（`sonnet-output.md:152`「所以「键的单元部分变了而这个函数交空表」的情形推不出来」） | 否，两份是同一个证明 |
| 挪了行又改内容 | 有（`opus-output.md:110`） | 有（`sonnet-output.md:138` 表行与 3167 一带的测试） | 否 |
| 两条同名单元互换内容 | 有 | 有（`sonnet-output.md:147` 一带） | 否 |
| 去行号后撞名 | 有 | 有（`sonnet-output.md:149` 一带） | 否 |
| 删一个 / 加一个按行号起名的单元 | 一般性证明里包含，没单列 | 有（表行「删一个单元」「加一个单元」，`sonnet-output.md:139`、`:140`；并写出测试里没有这样的输入，`sonnet-output.md:158`） | 只有正推腿单列 |
| 同一个宏调用三次、三段内容轮换 | 有（「唯一会「点错名」的」，`opus-output.md:113`），判不算漏报 | 没单列；最近的是「名字归属不定」（`sonnet-output.md:150`） | 攻方腿单列 |
| 插一次相同内容的调用 | 有（`opus-output.md:168`） | 没有 | 只有攻方腿 |
| `#<n>` 同名后缀挪位 | 有（`opus-output.md:168`） | 没有 | 只有攻方腿 |
| 唯一调用者核实：去行号只在 `units_changed_between` 里用 | 有（`opus-output.md:100` 起的「2b」） | 没有 | 只有攻方腿（今天我核到：`crash_identity.rs` 非测试处只有 `:497` 一处） |
| 前提「键的单元部分 = （种类，指纹）多重集」是从代码读的还是从注释读的 | 引的是注释「不带路径、名字、行号、手写版本号。」（`crates/singlefs-unit-graph/src/closure.rs:3260`） | 读的代码，`reuse_key` 里每个单元取（种类，指纹）（正推报告引 `closure.rs:3282` 起） | 代码层只有正推腿；我今天读了 `closure.rs:3282` 到 `:3303`，对上（见下） |
| 键以外的变化让清单为空时红句措辞 | 有（`opus-output.md:115`） | 有（`sonnet-output.md:153`） | 否，两条腿都提；判决:30 已记 |
| 第二步配对里「指纹也相同」没有变异行 | 没有 | 有（`sonnet-output.md:159`） | 只有正推腿；判决「三、改法」第 4 条已采 |
| 去行号函数会把不是行号的数字也去掉 | 在「类型与名字」格里提（`opus-output.md:145`） | 有（推翻条件，`sonnet-output.md:166`） | 否 |

两条腿都没覆盖的（我读代码得到，推的）：同一个单元挪到另一份文件、指纹不变——名字里带文件（「let base = format!("{} {} @ {file}", unit.kind, unit.qualified_name);」`crates/singlefs-verification/src/crash_identity.rs:787`），两步都配不上，旧名新名各报一个：这是多报，不是漏报，键也变不了。

### 我今天读的前提

- 键的单元部分：「unit_lines.extend(outcome.wgsl_units.iter().map(|wgsl_index| {」（`crates/singlefs-unit-graph/src/closure.rs:3294`）之前，普通单元也是 `unit\t<种类>\t<指纹>` 一行一个、排序后进哈希；名字、行号、文件都不在里面。
- 组成清单：名字是种类、限定名、文件，同名另加 `#<第几个>`（「format!("{base} #{occurrence}")」`crates/singlefs-verification/src/crash_identity.rs:793`）；指纹照单元自己的。
- 两步：先按整个名字加指纹配（「pair_off_units_with_the_same_compared_name_and_fingerprint(」`crates/singlefs-verification/src/crash_identity.rs:489`），再按去行号的名字加指纹配（`:494`）；每一对两边各拿掉一个，剩下的合并去重排序交回（`crates/singlefs-verification/src/crash_identity.rs:499` 起）。
- 由此：设 S、N 为两边单元的（种类，指纹）多重集，两步拿掉的每一对在 S、N 里各对应同一个（种类，指纹）（名字相同则种类相同，指纹相同是配对条件），所以剩下的多重集差正好是 S−N 与 N−S 的并集；键里单元部分变了，这两个差至少一个非空，至少交出一个名字。这一条我读码核了一遍，与两条腿同一个证明，不算独立校验。

### 「停」成不成立

- 规则字面：两条腿都答了这一格、都没打中；两条腿的措辞对到同一件事（「不漏」）上一致。「几份样本一不一致」那一段要求先按内容对格：我对了，对不上的只有「名字归属」一件——正推腿写「个数对、名字归属不定」（`sonnet-output.md:150`），攻方腿写轮换时「按键的口径轮换不算变，不算漏报」（`opus-output.md:113`），是同一现象的两种说法，都判不算漏报。
- 只有一条腿覆盖到的几种形状（上表的「只有……」几行），停不靠它们：都是一般性证明的特例；判决:30 引的是两条腿各自的一般性证明，不是形状清单。不需要「同一份提示再派一条同立场腿」。
- 这条「停」可以被推翻的唯一路径是前提：「键的单元部分是（种类，指纹）多重集」。两条腿都没跑，我也没跑；之后实现员补的变异行（`crates/mutations.tsv:8226`）与现有测试（`crates/singlefs-verification/src/crash_identity.rs:3636`）在 19876 里「抓到」（量的：19876 日志「判法单元红句：去掉行号之后配对不看指纹……：抓到：units_named_by_their_line_that_only_moved_are_not_named_as_changed_and_edited_ones_are 红了」），是配对条件的证红，不是不漏报的证明。

### 替「该再派一条腿」那一方写的最强理由，与判定

- 最强理由：两份证明都是读码、都没跑；一条腿只抽一次样不算一次观测；而这个函数的用处是给人点名「哪几个单元变了」，名字归属不稳定本身是正推腿提的小毛病，判决没记。
- 判定：压不过。「一次抽样」的门槛是为「没打中」写的，防的是一次答复没抽到反例；这里的两条腿给的是同一个可检验的证明，我再读码核了一遍，前提在代码里。再派一条腿读同一段码，边际价值低于加一条把它跑起来的检验。
- 判决要补记的两件：一，名字归属可能点到内容逐字相同的另一个单元（只出现在按行号起名的单元里，只会多点名一个同样内容的、不会少点），记账；二，正推腿写的「按行号起名的单元被删、或新加」没有输入（转述，原句在 `sonnet-output.md:158`）今天仍成立：测试 `units_named_by_their_line_that_only_moved_are_not_named_as_changed_and_edited_ones_are` 里没有这样的输入（读 `crash_identity.rs:3636` 至 `:3714`），判决「三、改法」第 4 条只补了变异行没补输入。
- 推翻它的现象：手造一组（库里、现在）两份清单，（种类，指纹）多重集不同而 `units_changed_between` 交空表。若要一次扫清，在草稿里写一个小模型对随机清单比「函数交空表 ⇔ 多重集相等」；这一轮没写（见「没做什么」）。

## 节「修补对照判决「三、改法」第 1 到 5 条」（对应派发第 5 条）

判决「三、改法」第 1 到 5 条在 `research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md:45` 到 `:49`。下面逐条对到今天文件的行。变异行号是今天 `crates/mutations.tsv` 的行；证红读数取任务日志（19873：「test result: ok. 4 passed」，含守护测试与拷树新测试，量的；19876、19921 各「跑了 4 条，抓到 4 条」，量的；19955「计数：抓到 9 条」，量的）。

### 改法 1：只打日志的单元加「不 panic、不借用改东西」的测试（判决:45）

| 判决要求 | 今天的落点 | 判 |
|---|---|---|
| 登记的每个单元函数体里没有 `expect`、`unwrap`、`panic!`、`assert!` 一族、下标取值 | 测试 `crates/singlefs-verification/src/crash_identity.rs:1887`；认法 `panicking_forms_in_the_function_body`（「fn panicking_forms_in_the_function_body(body: &str) -> Vec<String> {」`crates/singlefs-verification/src/crash_identity.rs:1812`）；遍历登记 `:2011` | 做到 |
| 参数里没有可变借用 | `parameters_that_can_change_what_the_caller_holds`（「fn parameters_that_can_change_what_the_caller_holds(parameters: &str) -> Vec<String> {」`crates/singlefs-verification/src/crash_identity.rs:1857`） | 做到字面 |
| 对照：认得出探针那一处 `expect` | `crash_identity.rs:1897` 至 `:1908` 现造 `gpu_lifecycle_line`，断言认出 `.expect(` | 做到 |
| 配变异行，同一处改动上红 | `crates/mutations.tsv:8223`，19876 抓到（量的） | 做到 |

- 多做（判决没要求）：整数算术 `+ - * / %`（「if matches!(character, '+' | '-' | '*' | '/' | '%')」`crates/singlefs-verification/src/crash_identity.rs:1844`）、`RefCell` 的 `borrow` 与 `borrow_mut` 及 `unwrap_err`、`expect_err`（`crash_identity.rs:1793`）、`unreachable!`、`todo!`、`unimplemented!`（「const MACROS_THAT_PANIC: [&str; 4] = ["panic", "unreachable", "todo", "unimplemented"];」`crates/singlefs-verification/src/crash_identity.rs:1803`）、`*mut` 与 `FnMut` 参数（`crash_identity.rs:1869`、`:1870`）。
  - 整数算术的依据：「overflow-checks = true」（`Cargo.toml:26`，`[profile.release]` 下），溢出与除零都 panic；是「不 panic」的延伸，不是偏离。
  - 对「误红会让人为过测试改登记」的回答（攻方的第 1 问，我只答与判决对照相关的一半）：把单元从登记里拿掉，它回到闭包里，键跟着函数体走（收法按限定名逐字比，`crates/singlefs-unit-graph/src/closure.rs:1389`）。误红的出路之一是往「更稳」那边改，代价是现判变多，不是漏判。所以多认的几类误红成本是搅动，不是不稳。
- 少做/做偏（判决字面够不着它自己的诊断）：
  - 判决:27 把缺口说成「panic 与经借用改卡的状态这两条路」，改法 1 只写「参数里没有可变借用」。今天 `gpu_lifecycle_trace` 收的是共享借用 `&GpuCard`（`crates/singlefs-verification/src/gpu_unit_checks.rs:1007`），改卡状态走 `device_condition()` 加 `&self` 方法（`gpu_unit_checks.rs:1121`、`crates/singlefs-verification/src/gpu_memory_ledger.rs:2228`），`&mut` 判据看不见。实现员写明认不出（「认不出的形态：内部可变的类型（`Cell`、`RefCell`、`Mutex`、原子类型）经共享借用改；参数之外的静态项。」`crates/singlefs-verification/src/crash_identity.rs:1856`）。这是判决字面窄于判决诊断，不是实现员做偏；今天这条路在三个单元里没有写法（量的：三个单元只读 `card.index` 与 `card.pci_bus_identifier`，`crates/singlefs-verification/src/gpu_unit_checks.rs:1012` 一带）。
  - 判决没有回应攻方腿提的另一条路：给登记的单元做标识符白名单（攻方报告表里「给登记了的三个单元加一条文本白名单测试」，`research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md:48`）。白名单一并关掉 panic 与共享借用两条，代价是改日志内容常常误红。判决选了黑名单，没写为什么；我不判谁对，记「推不出」。
  - 认不出的写法实现员列了六类，写进文档（`crash_identity.rs:1810` 起、`:1856`）：移位、调到的函数内部的 panic、宏展开、分配失败与栈溢出、共享借用加内部可变、参数之外的静态项。

### 改法 2：拷树重算拷 `crates/` 之外数据文件的那一段配带正例的测试（判决:46）

| 判决要求 | 今天的落点 | 判 |
|---|---|---|
| 「要另拷哪些文件」做成拷树那一步收的参数 | `RepositoryCopy::of_the_repository(label, repository, data_files_outside_the_crates)`（「fn of_the_repository(」`crates/singlefs-verification/src/crash_identity.rs:2680`）；薄壳 `new`（「fn new(label: &str) -> Self {」`crates/singlefs-verification/src/crash_identity.rs:2661`）交活树与构建时清单（「&data_files_outside_the_crates_the_build_read(),」`crates/singlefs-verification/src/crash_identity.rs:2670`） | 做到 |
| 用例交含一份 `crates/` 之外文件的清单，核它被拷到 | 测试 `crash_identity.rs:2762`，清单只含 `.claude/heavy-thread-default`，核同一路径、逐字相同 | 做到 |
| 删掉拷贝循环这条要红 | 循环在 `crates/singlefs-verification/src/crash_identity.rs:2712`；变异行 `crates/mutations.tsv:8224`，19876 抓到 | 做到 |
| 原变异行名字与弄坏的地方对不上，改名或改弄坏的地方 | 旧行删掉；`crates/mutations.tsv:8225` 改名成「……清单取成空的（弄坏的是清单，不是拷贝）」，弄坏的地方不变 | 做到（两样都做：改名，另加 8224 真删循环） |

- 多做：断言「不在清单里的不拷」（「"不在清单里的 crates/ 之外的文件不拷"」`crates/singlefs-verification/src/crash_identity.rs:2793`）、`crates/`、根上两份清单照拷。无害。
- 少做：薄壳 `new` 把清单交对了没有，没有东西判得出：把 `:2670` 换成 `&[]`，不带特性清单本来是空，带 gpu 时 `prune-server` 整目录已被拷（正推报告 `sonnet-output.md:201`），读活树的测试照绿。判决允许「等价的可注入形态」，没要求罩住薄壳；实现员自己写了这个缺口（不带特性时判不出）。与改法 2 同类的「今天没有行为差别」缺口，不是做偏。

### 改法 3：真卡共用模块（判决:47）

| 判决要求 | 今天的落点 | 判 |
|---|---|---|
| 两行打印各配测试与变异行（至少 `NOT_RUN_NO_GPU_ADAPTER` 那一行的文字与何时打） | 拼行函数 `not_run_line_of_the_test`（「format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")」`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:751`）、`handed_over_gpus_line_of_the_test`（`mod.rs:756`）、`line_reporting_the_gpus_taken`（`mod.rs:769`）；测试在新测试文件 `:227`、`:248`、`:272`；变异行 `crates/mutations.tsv:8227`（记号）、`:8228`（何时打）、`:8229`（原因）、`:8230`（序号） | 做到 |
| 取实例与读 `nvidia-smi` 的先后：判能不能在不要卡的测试里罩住 | 测试 `crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs:203`；变异行 `crates/mutations.tsv:8231`，19921 抓到 | 做到（判能罩） |
| 薄壳交进去的是不是那一个静态项：罩不住的写明并列进「合入后要在真卡上量」 | 薄壳 `mod.rs:791`、交静态项处 `mod.rs:803`；写在实现员报告「合入后要在真卡上量的」一节 | 做到，但只写在实现员报告里；仓里没有一份「合入后要在真卡上量」的清单可指（我没读到这样的登记位，见「没读到的」） |
| 建实例 panic 之后再建：写明是不是要守，是配测试，不是写文档 | 文档 `mod.rs:678`：「建它的那一次 panic 了，这一格照旧是空的」，写「这一条不守」，理由 `mod.rs:679` 至 `:680` | 做到；理由里「走到 panic 只有 wgpu 自己倒下」靠「驱动不在也交回实例、不报错」（`crates/singlefs-background-gpu/src/background_gpu.rs:1027`）；`OnceLock::get_or_init` 初始化 panic 后格子仍空是标准库行为，实现员自己写「没另查文档」，我也没查 |

- 少做（同一形状，判决没点名）：`println!` 自己不在任何不要卡的测试里：「println!("{}", line_reporting_the_gpus_taken(test_name, &taken));」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:809`）。这是判决「罩不住、并列进真卡上量」那一类，实现员已认。
- 少做（判决没点名、同一形状）：同一模块里还有两处直接打标记行的 `println!`，没改用拼行函数：「NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=nvidia-smi 列的计算进程里没有这个进程」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:634`）、「NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=读不成这个进程在哪几张卡上：{reason}"」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:642`）。判决:37 把「标记行那一行改坏了，门禁就读不到」算打中，这两行是同一个标记、同一个读法，今天只有返回值有测试（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:13479` 起两条断言只比返回值），打出来的字没人钉。变异行 `crates/mutations.tsv:7937` 的锚点把这两行尾部包在里面，所以实现员不改它们（「被两行旧变异锚住」，见调度材料「实现今天的样子」）。结果：标记行的拼法有三处，改拼行函数的字，这两处不会跟着变，没有东西守它们相同。判决没点这两处，所以实现员没做不算违令；但同一形状的缺口在这里同样成立，应记账，由主 agent 取舍要不要改锚点后让这两处也走拼行函数。

### 改法 4：第二步配对「指纹也相同」补一行变异（判决:48）

- 做到：`crates/mutations.tsv:8226`「判法单元红句：去掉行号之后配对不看指纹」，点名已有测试 `crates/singlefs-verification/src/crash_identity.rs:3636`，测试文档加了判别力一句（「去掉行号之后配对不看指纹（只按名字配）」`crates/singlefs-verification/src/crash_identity.rs:3634`）；19876 抓到（量的）。不多不少。

### 改法 5：带 gpu 的清单测试去掉「都在 `prune-server/src/` 底下」（判决:49）

- 做到：`crates/singlefs-verification/src/crash_identity.rs:2494` 到 `:2500` 只留「data_files.contains(&"prune-server/src/lib.rs"),」（`crates/singlefs-verification/src/crash_identity.rs:2497`）；点它的变异行 `crates/mutations.tsv:8225`，19876 抓到（量的）。不多不少。
- 范围外一句（不归这一条）：判决「三、改法」第 6 条要改的门禁注释，今天 `.claude/gate.d/crash-point-replay.sh` 那一段已写成「GPU_LIFECYCLE 行由 gpu_lifecycle_line、gpu_lifecycle_trace、gpu_work_trace 三个单元打，它们登记成 GPU 判器那一组不沿它收的单元」，我读到的是这一句（`.claude/gate.d/crash-point-replay.sh:315`）；这一条不是我复核的对象。

## 要主 agent 跑的命令

无。我没写模型、没起项目代码、没往任务服务交任务。判决要引的数（19825、19832、19873、19876、19921、19955）我只读了任务日志与状态，要引进判决的话照原样引任务服务的末行。

## 没读到的

- 禁读清单里的文件没读。
- 单元图 `why` 能答而我没跑：任一组闭包里有没有 `checking_threads` 或 `heavy_threads::REGISTRATION_FILE_TEXT`（「`checking_threads()` 另外 3 处」一件里「闭包够到就一定带上登记文件」那一环）。
- 没读：显存记账那一批与包装设备那一批登记时的三方判决（`research/prompts/m3-close-unit-graph-wrapper-impls-r3-main-verification.md` 与显存记账那一批登记的判决），所以「更早那一批同样缺 panic 与借用守护」只判到「判决的打中形状对它同样成立」，不判它是否有错。
- 没读：`scripts/mutation/crates-mutation-rows.py` 的 `NOT_RUN_LINE` 与 `task-server/task_server.py` 的 `gpu_not_run_test` 认这一行的正则（攻方腿攻修补的第 3 问，不归我）。
- 没读：仓里是否有一份登记「合入后要在真卡上量」的位置（`grep` 只在实现员报告与调度材料里见到这个说法，我没全仓搜）。
- 没量：不登记、让三个单元留在闭包里时，改一个日志字让 GPU 判器那一组现判多少块（节「坐实的两处」里「推翻它的现象」要的数）。
- 没查：`OnceLock::get_or_init` 初始化 panic 后格子仍空（标准库文档，没联网、没读源码）。
- 两处仓外文件（实现员报告 `~/.local/share/singlefs/work/ledger-topdown-judging-closure-writer-second/result.md`、`~/.local/share/singlefs/work/ledger-topdown-watched-data-files-repair/result.md`）引文核对不判，我用 `grep -n` 现取过行号。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。没往统一任务服务交任务，判决要引的数没复跑（没有命令要跑）。
- 不攻修补本身（攻方腿的第 1 到 4 问），只在节「修补对照」里写到与判决字面对照所需的程度。
- 没写模型、没跑「配对函数交空表 ⇔ 多重集相等」的随机检验；节「两步配对」的前提是读码核的，与两条腿同一个证明，不算独立校验。
- 没读不在那份判决里的新代码：`crates/singlefs-verification/src/crash_amplification.rs` 今天有别的会话的大批未提交改动（`git diff --stat HEAD` 3757 行加、1503 行减），我只读了与 `checking_threads()` 相关的几处。

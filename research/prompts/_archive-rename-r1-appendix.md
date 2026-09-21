**出处 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:49-73`（整段抄，未转述）**

```markdown
## 原样保存的证据不许事后改

发给模型的提示、跑出来的原始输出、当时的日志——这些和产物一一对应。
**改一个字，产物就不再对应它的输入**，而外面看不出来：文件还在，日期还在，
只有「这份输出是这份输入跑出来的」这句话悄悄不成立了。

所以改提示只能连同重跑一起改。要补说明就另开一份文件，别动原件。

**冻结只在这一轮内有效，出了轮归档进版本库。** 一轮的证据（提示、腿的报告、原始输出、留存产物）在这一轮里不许改一个字；
这一轮提交之后，下一次提交把**上一次留下的那批**删掉；本次的照旧留着，本次的验证照旧做得了。
要查删掉的那些，去版本库的历史里找。

⇒ **旧证据是参考，不是依据。** 对早先的结论有疑问时，重新验证它，不去翻旧证据：
旧数绑在当时那个构建上，而「今天还成不成立」只有今天跑一遍才知道（同篇「所有旧数据都只是参考」）。
要推翻早先的结论，按项目自己的推论纪律重走一轮，不是拿旧文件对质。

⇒ **正文引产物的检查按「这次改动」判，不按全仓判。** 当天写进正文的数要和当天的产物逐字对得上；
早先抄下的行是历史参考，它的产物已经归档，再判就是判一个不存在的对照。

⇒ **门禁也得绕开这批目录。** 一道要求人回去改原件的检查，逼出来的只有两种结果：
证据链断掉，或者这道检查被整个绕过去——两种都比不查更糟。
`scripts/doc-lint.sh` 从项目根的 `.claude/doc-lint-exclude` 读这份清单，
一行一条，**每条都要写明为什么**。指向不存在的目录、或者一个文件都没排到，都判红：
不起作用的排除项会让人以为那批文件已经被绕开了。

```

**出处 `.claude/singlefs-ai-sop/rules/show-me-test.md:97-121`（整段抄，未转述）**

```markdown
## 门禁不许假装通过

没实现的门禁阶段要**明说没实现**，不许悄悄跳过。
一个偷偷没跑崩溃测试的绿门禁，比红门禁危险得多。

**项目本地阶段这一轮无对象可判，退出码写 77**：`gate.sh` 记「本次未跑」，不记通过，也不算覆盖。
`exit 0` 的跳过在汇总里与「判过了」一模一样，而门禁分不出这两种——这一半靠写阶段的人。

同理，**跑批脚本不许把单轮的失败吞掉**（`|| true` 那种），输出路径也不许跨轮复用。
这两样凑一起，失败的那轮会安静地拿上一轮的输出顶上——看着一切正常，只是数字不动。
判据是「这份输出能不能证明是这一轮产生的」：跑之前删掉旧输出，跑完认一个本轮才会出现的完成标记。

**扫到 0 项也不是通过。** 判据的搜索范围写窄一点，对象就会全被第一步跳过——
既不算通过也不算失败，末尾照样报绿，而没人看得出来。
⇒ 扫一批对象的检查，成功那句里要报出**检查了多少项**；`gate-lint.sh` 判这一条。

**报了「查了多少」还不够，「没查的是哪些」也要逐个列出，而且清单要现算。**
一个阶段可以老老实实报出检查了多少项，同时把另一半对象整批漏掉：两句话都为真，
而读的人看不出后一句存在。
`gate-lint` 全绿，而那张表里 9 个计时行一个都没跑，其中 6 个既没跑、也没出现在任何一行里。
被掩住的是两处真问题：一个实验的留存产物早就对不上源码，另一个在 release 下直接 panic 跑不起来。
根因是跳过清单手抄——写死成 4 个编号，而真正没跑的是「表里全部计时行」加上那 4 个。
**所以跳过清单要与被扫集合出自同一份数据、现算**；成功那句同时报「跑了 N 个；没跑 M 个：逐个列名」。
这一条还没做成检查：`gate-lint` 现在不看报了数的脚本有没有同时列出跳过项，靠写阶段的人自己守。

```

**出处 `.claude/singlefs-ai-sop/rules/show-me-test.md:122-147`（整段抄，未转述）**

```markdown
# 一条改动不同步铺满全仓时，排除的每一份都要登记

「跳过清单要与被扫集合出自同一份数据、现算」那一条说的是**一道检查**的跳过清单。同一件事对**一次改动**同样成立，而且更容易被绕过去：
一个规范、一次操作、一次改动只铺了一部分文件时，**没铺到的要逐条登记进一张排除表**，
一行一条、`#` 后面写为什么不铺。**没有排除表就意味着全仓都要改。**

判据不是「改起来麻烦」，是「改了这句话就成假的，或者改了某道闸就失效」：
别家项目的术语与原文引文、某道检查自己的输入、工具自己的模式表、说这次改动本身的那句话——
这几类改了会出错，登记；其余一律改。

排除表照跟检查一样办：**指向不存在的路径判红**——不起作用的排除项会让人以为那批文件已经被绕开了。
表本身要能被一条命令读，别让「排除了什么」只活在做这件事的人的记忆里。

**成功那句里报的数，本身也要有东西钉住。** 它是成功行，永远不会红；它报了数，
满足「扫一批对象的检查要报出检查了多少项」那一条；它不是拒绝，`gate-lint` 另外三条都够不着它。
三样加起来，一个算错的统计量可以在门禁里一直绿着，而它正是给人看的那个结论。

**项目本地阶段与共享阶段同规矩。** 它们一样会拒绝提交者，一样受 `gate-lint`
与 `shell-lint` 管——`gate.sh` 把 `.claude/gate.d/` 一并交给这两个 lint
（singlefs 的本地阶段第一次被扫，就是 7 条没有出路的拒绝）。

⚠️ **射程只到 `.claude/gate.d/`。** 项目别处的脚本（研究脚本、hook）一样会拒绝人，却不在这两个 lint 的射程里。
⇒ 项目有这类目录，就在 `.claude/gate.d/` 里接一个本地阶段，把它们交给这两个 lint：调用时分别设 `GATE_LINT_DIR`、`SHELL_LINT_DIR` 指到目标目录——
不设的话共享脚本默认还会扫 SOP 自己的包，拿样本目录判红绿时就混进了真仓的脚本。
只接 gate-lint 等于只补了一半

```

**出处 `.claude/kb/layout/01-first-txn.md:380-406`（整段抄，未转述）**

```markdown
## 八、根槽写路径的段序列登记表（C316（提交步骤的登记位有四处且互不相同） 的登记位，2026-09-13 立）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 1（mkfs 种根）与步 5（发布：暖机与第一个事务）；恢复那一步不写字节，只吃这张表切出的段。

一条线的提交协议由它全部根槽写路径的录制流段序列定（D13（验证路线） 已定项 4 切段：屏障与 FUA 写切段，段内任意整写子集），等价类按这张表的同构分：段边界的位置相同、每段里出现的步骤种类**集合**相同（D17（实现分层与第三方管道） 已定项 2）；一步重复几次（设备数、单元数）是参数不进判据。别处引提交步骤一律链到这一节，不另抄。步骤种类只有五种：写单元（含码 3 容器、实例表单元）、写 journal 记录、根槽 FUA 写、系统配置槽原地覆写、屏障。表里的段序列逐字来自 E142（第一个事务的干跑） 产物的 `name=segments` 五行（`path=mkfs / instance_acquisition / warm_up / transaction / post_mkfs_stream`；取号那一行不是根槽写路径，登记在这里是因为整条流按录制流切、它的两个写落在流的开头一段），单测 `registered_segment_sequences_match_every_recorded_path` 钉住它们；表与产物之间的逐字比对由门禁 52 号做（C316（提交步骤的登记位有四处且互不相同） 2026-09-13 已还清）。

⚠️ **E142（第一个事务的干跑） 第九次跑（2026-09-16）已对**：产物 `e142-first-txn-dry-run-2026-09-16-instance-boundary.out` 的 `name=segments` 五行与第七次、第八次跑逐字相同（第九次只改恢复的前缀规则，写清单一个字不动）——树表条目加宽只改一个单元里装什么字节，不改写请求的条数、次序与屏障位置；`name=width` 28 行 expected == actual，层 0 仍 262165 个状态零违例。

| 根槽写路径 | 录制流的段序列（每段的写数与种类；「种类」那串是产物 `kinds=` 字段的原样，每段一个步骤种类多重集，段之间用 `\|` 隔开，门禁 52 号逐字比对） | 出处 | 层 0 枚举 |
|---|---|---|---|
| mkfs 种根 | [m1 实例表单元 × 2 盘 + m2 树表单元 × 2 盘 = 4 单元写] 屏障 [m3 根 FUA 区域 0] [m3 根 FUA 区域 1] [m3 根 FUA 区域 2] [m4 系统配置槽 × 2 盘 × 2 槽 = 4，都是世代号 1] 屏障 ⇒ `4+1+1+1+4`，13 次操作、34 个崩溃状态，种类 `[unit_write×4,barrier]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[system_configuration_slot×4,barrier]` | 一；E142（第一个事务的干跑） `fn mkfs`、产物 `name=segments path=mkfs` | **不在**（G19：装置从 mkfs 之后的池起枚举；段序列由单测钉住） |
| 第一次可写挂载取号（实例代号 0 → 1，不是根槽写路径） | [a1 系统配置槽 × 2 盘，世代号 2] ⇒ `2`，2 次操作、4 个崩溃状态，种类 `[system_configuration_slot×2]`；取号两写之后一道屏障（D23（journal 的角色与格式） 已定项 16 要取号两写之后、本实例第一个非系统配置写之前一道完成了的屏障；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：`crates/singlefs-core/src/transaction.rs` 的 `acquire_instance` 写完两份系统配置自己发一道（`CommitStep::Barrier`），首次挂载上它与暖机第一次空发布开头那道背靠背、登记的段序列不变；第二次以后的挂载靠它把取号与写行那次发布的单元写隔开（2026-09-17 按代码改写）；世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1，首次挂载写出的是 2 | 一（a1）；D23（journal 的角色与格式） 已定项 16；产物 `name=segments path=instance_acquisition` | 在（第六次跑起，整条流开头那一段） |
| 空发布（暖机，第一版 2 次） | 屏障 [w1 空记录 × 2 盘] 屏障 [w2 根 FUA] [w3 系统配置槽 × 2 盘]（第二次同型，w4–w6）⇒ 两次合起来 `2+1+2+2+1+2`，种类 `[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[system_configuration_slot×2]` | 零；D16（发布语义） 已定项 8；产物 `name=segments path=warm_up` | 在（第四次跑起） |
| 普通发布（第一个事务） | [t1–t8 单元 × 2 盘 = 16] 屏障 [t9 记录 × 2 盘] 屏障 [t10 根 FUA] [t11 系统配置槽 × 2 盘] ⇒ `16+2+1+2`，种类 `[unit_write×16,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[system_configuration_slot×2]` | 零；D16（发布语义） 已定项 7；产物 `name=segments path=transaction` | 在（E142（第一个事务的干跑） 主臂） |
| 实例切换 / 管理员回退 | 实例切换那一半 2026-09-16 已写成字节，就是「第一次之后的可写挂载（写行）」那一行（切换 = 挂载内做一次恢复再写行，D23（journal 的角色与格式） 已定项 14）；管理员回退那一半 2026-09-17 也写成字节（**装置钉住**，里程碑「第二个事务」步 4，孤立形状从第二条流推得）：[取号系统配置槽 × 2 盘，世代号 4] 屏障 [实例表单元（回退行 + 中间实例行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障 [记录 × 2 盘，jsn 接在 R_old 那条之后] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `2+10+2+1+2`，与写行那一行同型、多的是内容（回退行带回退位、中间实例行、影子账只住内存不写字节），**之后接新实例的暖机**（这条脚本上一次：txg 9 落盘 0、txg 10 落盘 1）；取号之后那道屏障是 D23（journal 的角色与格式） 已定项 16（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款））（D16（发布语义） 已定项 8：新实例的根覆盖两块盘之前连推空发布，与第一次挂载同型）——与普通发布 + 空发布同型，多的是内容不是步骤 | D23（journal 的角色与格式） 已定项 14；D16（发布语义） 已定项 8 | 不在 |
| 第一次之后的可写挂载（写行） | **装置钉住**（里程碑「第二个事务」步 3 2026-09-16 落地，孤立形状从第二条流推得）：[取号系统配置槽 × 2 盘，世代号 3] 屏障 [实例表单元（写行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `2+10+2+1+2`；整条流里取号的 2 个写与上一次发布的 2 个系统配置槽写合成 4 写一段、末尾的 2 个系统配置槽写与暖机第一次的 8 个单元写合成 10 写一段（第二条流第 13–17 段 `4+10+2+1+10`）；取号之后那道屏障由取号自己发（D23（journal 的角色与格式） 已定项 16：写行那次发布有单元写，等不到空发布开头那道）；每次可写挂载都写行（实例 0 不写，第一次可写挂载要写的区间是空的），写行那次是新实例的第一次发布，之前不推抬 F 的空发布，元数据走切换预留（D18（块里携带什么信息） 已定项 11、D28（挂载期承诺量） 已定项 3；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方） | D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 16；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
| 空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元） | **装置钉住**（步 3 落地，孤立形状从第二条流推得）：[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `8+2+1+2`，与首次挂载的暖机不同型：记账树已经存在，空发布也重写记账行连带四个固定点单元（D16（发布语义） 已定项 9），这条脚本上 c_max = 4；整条流里 8 个单元写与上一次发布的 2 个系统配置槽写合成 10 写一段；实例 2 从 txg 5 起推 2 次（txg 6 落盘 0、txg 7 落盘 1），次数按「本实例的根覆盖全部区域盘」现算、上限 3——D16（发布语义） 已定项 8 只给第一次可写挂载定了常量 2，后续挂载的次数没有条款，实做的取法等用户定 | D16（发布语义） 已定项 8 / 已定项 9；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
| 抬 F 的空发布（D16（发布语义） 已定项 1：准入不够时先推空发布抬回退下界） | **装置钉住**（里程碑「第二个事务」步 5 2026-09-17 落地，孤立形状从第二条流推得）：与后续挂载的暖机空发布同型——[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] ⇒ 孤立看 `8+2+1+2`，根记录的 F 写成目标值、记账行是回收过释放代 ≤ F 之后的账；推到每块盘上都有一条带新 F 的根为止（这条脚本上两次：txg 15 落盘 0、txg 16 落盘 1）；第一版只有测试的强制入口，准入不够的正常触发没做 | D16（发布语义） 已定项 1；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
| 只做过 mkfs 的池的可写挂载（取号、零单元写行、零单元暖机）再发第一个文件版本 | **装置钉住**（里程碑「第二个事务」步 3 2026-09-17 落地，2026-09-17 用户定案允许只做过 mkfs 的池可写挂载）：重开之后取号 [系统配置槽 × 2 盘] 屏障，树表 0 条 ⇒ 写行那次发布与暖机都写零个单元，与 mkfs 同一个进程里的取号、两次暖机同型，之后第一个文件版本同第一个事务 ⇒ 整条流 `2+2+1+2+2+1+18+2+1+2`，与第一个事务那条流逐段相同、闭式 262165；树表 0 条而要写实例表行、回退到树表 0 条的根都在写之前拒绝 | D16（发布语义） 已定项 9（树表 0 条时空发布写零个单元）；用例 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs` | 与第一个事务那条流的基线镜像、写表与段序列逐项相同（快用例钉住），不另枚举；崩溃状态上重开可写挂载不在层 0 里 |

⚠️ 发布与下一次发布之间没有屏障：上一次发布的系统配置槽写与下一次发布的单元写落在同一段（第一个事务的第六次跑里暖机第二次的 2 个系统配置写与 16 个单元写合成 18 个写的一段，取号的 2 个系统配置写与暖机第一次开头的屏障合成开头一段，整条流 `2+2+1+2+2+1+18+2+1+2`、262165 个状态，种类 `[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]`）；mkfs 末尾有屏障、空发布开头有屏障，那两道接缝是切开的。段序列按整条录制流切，不按路径切，所以登记表按路径给的段序列是「孤立看」的形状，整条流那一行才是层 0 枚举吃的。

⚠️ 第二条流（里程碑「第二个事务」步 0 的固定脚本，2026-09-17 做到发布 E）：取号 → 暖机 × 2 → A → B → 进程退出、重开取号 → 写行 → 暖机 × 2 → C → 进程退出、重开回退到 A 的根、取号 → D（回退行）→ 暖机 × 1 → 覆盖写 × 4 → 抬 F 的空发布 × 2 → E，第二条流的段序列 `2+2+1+2+2+1+18+2+1+18+2+1+4+10+2+1+10+2+1+10+2+1+18+2+1+4+10+2+1+10+2+1+18+2+1+18+2+1+18+2+1+18+2+1+10+2+1+10+2+1+18+2+1+2`、279 次写、2104413 个状态，没有干跑产物，装置钉住：`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 把这个数组与闭式钉死，门禁 54 号在 release 下全量跑它、52 号核这句与用例里的数组、闭式、写数相符；到 C（26 段、789555）与到 D（33 段、791624）两个前缀各由同一份用例里只跑准备的一条钉住。前 12 段与第一条流相同，第 13 段是 B 的 2 个系统配置槽写与重开取号的 2 个系统配置槽写合成的 4 写一段（进程退出与重开之间没有屏障，录制流按设备连着记）；第 26 段同型（C 的 2 个系统配置槽写与回退取号的 2 个）。

<!-- format-const: WARM_UP_EMPTY_PUBLISHES = 2 -->
<!-- format-const: FIRST_TRANSACTION_TXG = 3 -->

```

**出处 `.claude/kb/term-renames.md:1-8`（整段抄，未转述）**

```markdown
# 术语改名对照表

全仓改过名的概念，旧名与新名的唯一权威对照。**别处一律用新名**——`research/scripts/sweep-term.py --check` 扫全仓，
除 `.claude/term-rename-exempt` 登记的几处之外出现旧名就判红。

这一份自己登记在那张豁免表里：写「X 改名成 Y」这句话必须写得出 X，而**旧名要有一处查得到，历史产物与旧提交里的字段才对得上**。
要查某个旧名今天叫什么，只看这一份；要查某次改名当天改了哪些文件、为什么改，看决策变更史当天那一条。

```

**出处 `.claude/kb/term-renames.md:9-41`（整段抄，未转述）**

```markdown
## 超级块 → 系统配置（2026-09-20 起，英文标识符与冻结目录 2026-09-21）

用户 2026-09-20 定案：盘上那块 481 字节的结构不叫「超级块」——本工程的索引树是普通的树，丢了能重建，没有「超级」的东西；
那块结构是**系统配置**。2026-09-21 定案：英文标识符、文件名与冻结目录一并改，「直到全工程都搜不出来」。

<!-- term-renames:table -->
一行一对，`research/scripts/sweep-term.py` 按这张表换、按这张表查；**加一行就自动受门禁管**。
「匹配」写 `整串` 或 `词边界`（后者用在缩写上，免得打中别的词里的那几个字母）。长的写在前面，免得短的先把长的吃掉一半。

| 旧 | 新 | 匹配 | 是什么 |
|---|---|---|---|
| 超级块 | 系统配置 | 整串 | 中文术语 |
| SUPER_BLOCK | SYSTEM_CONFIGURATION | 整串 | 常量名，带下划线 |
| SUPERBLOCK | SYSTEM_CONFIGURATION | 整串 | 常量名 |
| Super_Block | SystemConfiguration | 整串 | 大驼峰带下划线 |
| SuperBlock | SystemConfiguration | 整串 | 大驼峰，类型名 |
| Superblock | SystemConfiguration | 整串 | 大驼峰，b 小写的写法 |
| superBlock | systemConfiguration | 整串 | 小驼峰 |
| super_block | system_configuration | 整串 | 蛇形带下划线 |
| super-block | system-configuration | 整串 | 连字符形态（二进制名、产物文件名） |
| superblock | system_configuration | 整串 | 蛇形标识符、模块名、文件名 |
| sb_ | system_configuration_ | 词边界 | 缩写前缀（`sb_mac`） |
| _sb | _system_configuration | 词边界 | 缩写后缀（`tail_sb`） |
| sb | system_configuration | 词边界 | 裸缩写。`sb` 本是为 superblock 设的，概念没了缩写也去掉，同日从 SOP 的缩写表里删行 |

**跟着改的文件名**：`crates/singlefs-core/src/superblock.rs` → `system_configuration.rs`；五个实验装置、五张变异表、五个留存产物
（E100 / E115 / E124 / E126 / E147）。搬迁怎么做见 [.claude/rules/path-moves.md](../rules/path-moves.md)「怎么做」那一节。

**没改的**：`SUPERBLOCK_MAGIC` 的**值** `*b"SFSB"` 不动——那是写进盘上的魔数，改它等于改磁盘格式。常量名跟着改了。

**改名前的原件**：在 git 里。中文那一批的前一个状态见提交 `495bede` 之前；英文标识符与冻结目录那一批见 `4105e70` 之后的工作区改动。
留存产物与源码是同一次一起改的，所以 `replay.sh` 的逐字节比对改名前后都成立——这一条由改完的全量复跑证明，不是声称。

```

**出处 `.claude/kb/term-renames.md:42-47`（整段抄，未转述）**

```markdown
## 历史版本

### 2026-09-21

- 立本文件。此前新旧对照散在决策变更史 2026-09-20（其二十）那一条里，而那一条自己也在清扫范围内，改完就读不出旧名是什么。

```

**出处 `.claude/kb/checks-owed.md:384-387`（整段抄，未转述）**

```markdown
| C428 | 路径回写的自检与被检共用同一条跳过规则 | `research/scripts/rewrite-moved-paths.py` 的 `statement_line()` 把带「搬到」「改前」「改名为」这类词的行当成「说搬迁这件事本身」的行跳过不改，**而它的自检 `remaining()` 调的是同一个 `statement_line()`** ⇒ 凡是它误跳的行，它自己永远报不出来，成功句照样写「回读都没有旧路径」。2026-09-21 实测：`crates/singlefs-core/src/system_configuration.rs` 改名成 `system_configuration.rs` 之后，C414（挂载时不比对 io_min 与记录的槽距） 那一行的现状句因为含「搬到」二字被跳过，旧路径留在正文里，脚本报绿；主 agent 逐条分类全仓剩余旧名时才发现 | 自检不复用 `statement_line()`：凡是文件里还出现旧路径整串的行一律报出来，被有意跳过的另列一节、逐行写明为什么跳，人确认过才算过；判别力自证喂一个「含『搬到』且含旧路径的现状句」的小仓，必须判红 | 无 | 2026-09-21 全仓术语改名第二批的核验轮 |
| C429 | 路径回写不保形，也不核新路径指不指得到 | `rewrite-moved-paths.py` 把「纯文件名」「相对所在 crate 根的路径」一律升级成仓库根起的路径，而有些位置要的不是那种形状。2026-09-21 实测两处：`research/e7-index-bench/Cargo.toml` 的 `path =`（Cargo 按 crate 根解析）拼成 `src/bin/research/e7-index-bench/src/bin/…`，`cargo build` 报 can't find bin；`research/scripts/replay.sh` 登记表第 4 列（要 `results/` 底下的纯文件名）拼成 `results/research/results/…`，`diff` 报「对不上，127 行不同」——看着像产物变了，其实是登记表坏了。**而它报「✓ 改写了 34 个文件，回读都没有旧路径」：回读只证明旧路径消失，不证明新路径指得到东西** | 回写之后对每个写进去的新路径按它所在文件的解析口径做存在性断言（Markdown 链接按文件位置、Cargo 的 `path` 按 crate 根、其余按仓库根），指不到就判红并列出文件与那一行；第二处已另立会红的闸（`replay.sh` 在构建之前判第 4 列带斜杠就退 2，坏表退 2 / 真表退 0 双向证过） | 无 | 2026-09-21 全仓术语改名第二批 |
| C430 | 以数据形式存在的名字没人管命名纪律 | `naming-lint` 的射程写明只查**我们声明的名字**（变量、字段、类型、常量……），而一类名字活在**字符串字面量与表格里**：实验装置的字段表标签（`("system_configuration_mac", 16)`，E100 / E115 / E124）、臂名（`Tail::SystemConfiguration => "tail_system_configuration"`，E23）、段序列的步骤种类串（`StepKind::SystemConfigurationSlot => "system_configuration_slot"`）。它们逐字落进留存产物与 kb 正文，是**人真正读到的那批名字**，却一条命名纪律都罩不到——2026-09-21 全仓术语改名时才发现 `system_configuration` 这个缩写在这一层活了下来，而同期 `naming-lint` 报全仓 0 违规 | 扫产物与 kb 表格里的「名字形态的串」（`key=` 的键名、`("名字", 数)` 的首列、`=> "名字"` 的右侧），按同一份缩写词表与单字母规则判；判别力自证喂一个含 `("system_configuration_mac", 16)` 的装置源码，必须判红 | 无 | 2026-09-21 用户看到 `system_configuration_mac` / `tail_system_configuration` 问「不是明确了不允许缩写么」，逐处现查后立 |
| C431 | 腿把背景材料的行号当成 kb 文件自己的行号 | 三方的腿读的是拼好的背景材料（正文 + 清单 + 附录，上千行），顺手记下的行号是**材料里的行号**，贴在 kb 文件名后面就指向一个不存在的位置。`.claude/rules/three-way-inference.md` 已明令「引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数」，派发提示里也逐字写了，**2026-09-21 增补 1 第二轮里两条云端腿仍各犯一次**（`23-journal的角色与格式.md:669` 实际是背景材料第 669 行，kb 文件只 614 行；`invariants.md:1202-1208` 同理，那份只 458 行）——靠提示里写一句拦不住 | 核查员那一步机械判：腿报告里每个 `<路径>:<行号>` 形态的引用，先看那份文件有没有那么多行（超了直接判红、报出文件实际行数），再看引文是不是逐字出现在那一行附近；判别力自证喂一份「行号超过文件行数」的报告，必须判红 | 无 | 2026-09-21 增补 1 第二轮核查员交回，76 处里 2 处是这个形态 |
```

**出处 `.claude/term-rename-exempt:1-10`（整段抄，未转述）**

```markdown
# 全仓术语改名扫不到的地方。一行一条：<相对路径或目录>  # 为什么
# 判据不是「改起来麻烦」，是「改了这句话就成假的，或者改了某道闸就失效」。
# 改一个全仓术语怎么做见 .claude/rules/path-moves.md「改一个全仓术语：正文之外还有五处会红」。
# 指向不存在的路径判红：不起作用的豁免项会让人以为那批文件已经被绕开了。
.claude/kb/prior-art.md  # 他家方案调研：btrfs、F2FS、ZFS 等自己的术语与原文引文。把别家的 superblock 改成我们的名字，引文就成了假的（用户 2026-09-21 定）
.claude/warnings/  # 警告记录按日期冻结，而且它记的就是「这个词被改掉了」这件事本身，词换掉句子就没意义了
.claude/singlefs-ai-sop/  # 上游 SOP 的副本，只能在上游仓改（CLAUDE.md「怎么改」那一行），就地改下次同步就没
research/scripts/sweep-term.py  # 清扫工具自己：替换表与检测正则里必须写得出旧名，扫了它就再也换不动东西
.claude/term-rename-exempt  # 这张表自己：豁免理由里要写得出旧名才说得清豁免的是什么
.claude/kb/term-renames.md  # 新旧对照的唯一权威记录：写「X 改名成 Y」必须写得出 X，旧名要有一处查得到，历史产物与旧提交的字段才对得上
```

**出处 `research/scripts/archive-past-rounds.py:163-224`（整段抄，未转述）**

```markdown
def selftest():
    import tempfile
    with tempfile.TemporaryDirectory() as work:
        os.makedirs(os.path.join(work, "research/results"))
        os.makedirs(os.path.join(work, "research/prompts"))
        os.makedirs(os.path.join(work, ".claude/kb"))
        git("init", "-q", root=work)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
        open(os.path.join(work, "research/results/e1-old.out"), "w").write("old\n")
        open(os.path.join(work, "research/prompts/r1-main-verification.md"), "w").write("判决\n")
        open(os.path.join(work, ".claude/kb/page.md"), "w").write(
            "产物 `research/results/e1-old.out`，判决 `research/prompts/r1-main-verification.md`\n")
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "round one"], cwd=work, check=True)
        open(os.path.join(work, "research/results/e2-new.out"), "w").write("new\n")
        if run(work, False) != 1:
            print("  ✗ 自检失败：有上一轮记录时 --check 没判红")
            print("     → 怎么办：看 past_round_files() 的 ls-files 与 diff 组合。")
            return 1
        run(work, True)
        if os.path.exists(os.path.join(work, "research/results/e1-old.out")):
            print("  ✗ 自检失败：上一轮的产物没删掉")
            print("     → 怎么办：看 run() 的 os.remove 那一段。")
            return 1
        if not os.path.exists(os.path.join(work, "research/prompts/r1-main-verification.md")):
            print("  ✗ 自检失败：判决被删了，它该留")
            print("     → 怎么办：看 KEEP 那条正则。")
            return 1
        if not os.path.exists(os.path.join(work, "research/results/e2-new.out")):
            print("  ✗ 自检失败：本轮新产生的被删了")
            print("     → 怎么办：未跟踪的文件不在 ls-files 里，看 past_round_files() 为什么把它算进去了。")
            return 1
        # 「删完有没有门禁失去输入」这条自己也要证明会红：造一道点名了被删产物的检查
        gate_dir = os.path.join(work, ".claude", "gate.d")
        os.makedirs(gate_dir, exist_ok=True)
        open(os.path.join(gate_dir, "99-sample.sh"), "w").write(
            '#!/usr/bin/env bash\n# 点名 research/results/e1-old.out 当输入\n')
        if not gates_losing_input(work, ["research/results/e1-old.out"]):
            print("  ✗ 自检失败：有检查点名了被删的产物，gates_losing_input 却一条都没报")
            print("     → 怎么办：看它扫的目录与 doomed_names 的匹配。")
            return 1
        if gates_losing_input(work, ["research/results/e2-new.out"]):
            print("  ✗ 自检失败：没有检查点名 e2-new.out，gates_losing_input 却报了")
            print("     → 怎么办：它在拿文件名做子串匹配，看是不是打中了别的词。")
            return 1
        page = open(os.path.join(work, ".claude/kb/page.md"), encoding="utf-8").read()
        if "research/results/e1-old.out" in page:
            print("  ✗ 自检失败：指向被删文件的引用还留着指空的路径")
            print("     → 怎么办：看 rewrite_links() 的 LINK 正则与 doomed 集合。")
            return 1
        if "research/prompts/r1-main-verification.md" not in page:
            print("  ✗ 自检失败：指向判决的引用被误改了，判决没删就不该改它的链接")
            print("     → 怎么办：rewrite_links() 只改 doomed 里的路径，看它为什么碰了别的。")
            return 1
        if run(work, False) != 0:
            print("  ✗ 自检失败：删完 --check 仍判红")
            print("     → 怎么办：看 past_round_files() 删除之后还认出了什么。")
            return 1
    print("  ✓ 自检：有旧记录判红、删完判绿、判决与本轮新产物不被删、指向被删文件的引用改成不带路径的说法、"
          "点名被删产物的检查报得出来且不误报")
    return 0
```

**出处 `research/scripts/sweep-term.py:171-228`（整段抄，未转述）**

```markdown
def selftest():
    import tempfile
    with tempfile.TemporaryDirectory() as work:
        os.makedirs(os.path.join(work, ".claude"))
        os.makedirs(os.path.join(work, "keep"))
        open(os.path.join(work, "keep", "quoted.md"), "w", encoding="utf-8").write("btrfs 的 superblock\n")
        # 每一种变体各一行：漏掉哪一种，下面「换完判绿」那一步就红
        open(os.path.join(work, "code.rs"), "w", encoding="utf-8").write(
            "const SUPERBLOCK_BYTES: u64 = 481;\n"
            "const SUPER_BLOCK_BYTES: u64 = 481;\n"
            "struct SuperBlock;\nstruct Superblock;\nstruct Super_Block;\n"
            "let superBlock = 1;\nlet super_block = 2;\nlet superblock = 3;\n"
            "// super-block 与 超级块 两种写法\n"
            "let sb_mac = 4;\nlet tail_sb = 5;\nlet sb = 6;\n")
        open(os.path.join(work, ".claude", "term-rename-exempt"), "w", encoding="utf-8").write("keep/  # 别家术语\n.claude/kb/term-renames.md  # 表自己写着旧名\n%s  # 脚本自己写着旧名\n"
            % os.path.relpath(os.path.abspath(__file__), work))
        os.makedirs(os.path.join(work, ".claude", "kb"))
        open(os.path.join(work, ".claude", "kb", "term-renames.md"), "w", encoding="utf-8").write(
            "<!-- term-renames:table -->\n"
            "| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n"
            "| 超级块 | 系统配置 | 整串 | 中文 |\n"
            "| SUPER_BLOCK | SYSTEM_CONFIGURATION | 整串 | 常量带下划线 |\n"
            "| SUPERBLOCK | SYSTEM_CONFIGURATION | 整串 | 常量 |\n"
            "| Super_Block | SystemConfiguration | 整串 | 大驼峰带下划线 |\n"
            "| SuperBlock | SystemConfiguration | 整串 | 大驼峰 |\n"
            "| Superblock | SystemConfiguration | 整串 | 大驼峰 b 小写 |\n"
            "| superBlock | systemConfiguration | 整串 | 小驼峰 |\n"
            "| super_block | system_configuration | 整串 | 蛇形带下划线 |\n"
            "| super-block | system-configuration | 整串 | 连字符 |\n"
            "| superblock | system_configuration | 整串 | 蛇形 |\n"
            "| sb_ | system_configuration_ | 词边界 | 缩写前缀 |\n"
            "| _sb | _system_configuration | 词边界 | 缩写后缀 |\n"
            "| sb | system_configuration | 词边界 | 裸缩写 |\n")
        if sweep(work, False) != 1:
            print("  ✗ 自检失败：带旧名的小仓 --check 没判红")
            print("     → 怎么办：看 walk() 与 MAP 的匹配。")
            return 1
        sweep(work, True)
        if sweep(work, False) != 0:
            print("  ✗ 自检失败：--apply 之后 --check 仍判红")
            print("     → 怎么办：看 MAP 的替换顺序，大小写形态是不是漏了一种。")
            return 1
        if "superblock" not in open(os.path.join(work, "keep", "quoted.md"), encoding="utf-8").read():
            print("  ✗ 自检失败：豁免目录里的旧名被换掉了")
            print("     → 怎么办：看 exempt() 的目录前缀判法。")
            return 1
        open(os.path.join(work, ".claude", "term-rename-exempt"), "w", encoding="utf-8").write("nowhere/  # 指不到\n")
        code = None
        try:
            sweep(work, False)
        except SystemExit as stop:
            code = stop.code
        if code != 2:
            print("  ✗ 自检失败：豁免登记表指不到文件时没判红")
            print("     → 怎么办：看 load_exempt() 的存在性断言。")
            return 1
    print("  ✓ 自检：带旧名判红、换完判绿、豁免目录不被换、豁免指不到文件判红")
    return 0
```

**出处 `research/scripts/replay.sh:445-451`（整段抄，未转述）**

```markdown
  # 留存产物已按「每次提交删上一次的实验记录」归档进版本库时，这一档不比对。
  # 不报成「对不上」：那与「装置真的改坏了、复跑出不同字节」长得一模一样，读的人会以为实验坏了
  # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
  if [[ ! -f "results/$stored" ]]; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 产物已归档 "$stored 不在树里；本次跑得出来，逐字节这一档不比对"
    archived=$((archived+1)); CLAIM_QUEUE+=("$exp|$fresh"); continue
  fi
```

**出处 `research/scripts/replay.sh:476-484`（整段抄，未转述）**

```markdown
echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad ／ 产物已归档 $archived"
echo "本轮输出：$OUT_DIR"
if [[ $archived -ne 0 ]]; then
  echo "  ! 「产物已归档」$archived 行：留存产物按「每次提交删上一次的实验记录」归档进了版本库，逐字节这一档没有对照物。"
  echo "     这不是判红——这几行本次都跑得出来，结论区间断言照常判。要看当时的产物："
  echo "         git log --all --diff-filter=D --name-only -- \"*<产物文件名>\"     # 找到删它的那次提交"
  echo "         git show <提交>^:research/results/<产物文件名>                      # 读回当时的内容"
  echo "     读到的是当时的数、不是今天的结论；要拿它支撑新结论就重新跑一遍（evidence-discipline.md）。"
fi
```

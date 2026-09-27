# m2-safety-r3 本地攻方提示：逐句核对表

英文项 / 原文文件:行 / 首稿缺的 / 定稿。原文表格用竖线分格，转写时把「格」拆成句子逐条核。

## 1. Rule T（正文 `_m2-safety-r3-body.md:13`）

原文（整行，三格）：`T（今天） | 容量(d) − 已分配(d) − 不可回收(d) − defer 待释放(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − [待删 + 已承诺预留 + checkpoint 保留池] ÷ R（crates/singlefs-core/src/admission.rs 的 available_on_each_device） | Σ 这次发布重写的角色里 space_budget_of_role 为 Demand 的 span_slots（admission.rs 的 demand_of_the_roles_on_each_device） | 报拒（发布 PublishError::SpaceAdmissionRefused，挂载 MountError::SpaceAdmissionRefusedBeforeAcquisition）`

英文项（提示 Part 1 第一段）：available(d) = capacity(d) minus already-allocated(d) minus unreclaimable(d) minus deferred-pending-release(d) minus mount-time-commitment(d) minus abandoned-root-exclusive(d) minus [pending-delete + committed-reservation + checkpoint-reserve-pool] divided by R；demand(d) = Σ span_slots (Demand 角色)；不过时报拒两种错误码。

首稿缺的：无——逐项都译了，函数名 `available_on_each_device`、`demand_of_the_roles_on_each_device`、两个错误码原样保留。

多出的（首稿即定稿，本条要单列原因）：demand(d) 定义句里加了括注 "(ordinary allocation: user data units, and extent-tree/inode-tree nodes rewritten because the user changed content)"。原文这一格本身没有这句解释，它出自同一份正文旁边 admission.rs 的角色定义注释（`SpaceBudgetOfARole::Demand` 那一段：「普通分配：用户数据单元，与因为用户这次改了内容才重写的 extent 树、inode 树的节点」，`crates/singlefs-core/src/admission.rs:514`）。加它的原因：本地模型看不到 `crates/` 源码，「Demand 角色」四个字不解释就是个不可查的黑箱记号，模型会当成需要它自己去查的外部事实而卡住；这句解释来自同一颗决策链条上的另一处已定义源码注释，不是我自己编的例子。

定稿：保留这句括注，按上面写明原因。

## 2. Rule P2（正文 `_m2-safety-r3-body.md:15`）

原文（整行）：`P2（ND+A1n） | T 的式子去掉「− defer 待释放(d)」那一项 | Σ 这次发布重写的**全部**角色的 span_slots（不按 space_budget_of_role 过滤：普通分配加这次写出的固定点），不加换下的槽 | 报拒`

英文项：available(d) = T's formula with "minus deferred-pending-release(d)" removed；demand(d) = Σ span_slots over every role (not filtered)，含普通分配与固定点，不加换下的槽；不过时报拒。

首稿缺的：无。

多出的：无——「不加换下的槽」直接译成 "without adding the slots being replaced (released) by this publish"，加的「(released)」只是给「replaced」补一个与 Part 3 的 "released" 用词一致的同义词，不改变原句意思，不算多加限定词。

定稿：与首稿相同。

## 3. Rule P1（正文 `_m2-safety-r3-body.md:14`）

原文（整行）：`P1（T + C283） | 同 T | 同 T | 先推一次空发布抬 F，F 取 D16（发布语义） 已定项 1「抬 F 的上限」那一行的准入上限（.claude/kb/decisions/16-发布语义.md 第 36 行），推完重判；一次准入最多 8 次发布（同文件第 41 行，B = 4 + 2 k_tol，k_tol = 2），做满仍不过才报拒；写行那次发布之前不推（同一行的括注）`

英文项：available(d)/demand(d) 同 T；不过时先推一空发布抬 F 到 fact B2 的上限，重判；最多推 8 次（fact B1）；做满 8 次仍不过才拒；写行那次发布之前不推。

首稿缺的：「写行那次发布之前不推」这一句的限定词「新实例的第一次发布」在正文本行只用括注指过去（「同一行的括注」），没有整句抄在这一行——括注本身在 D16 原文第 41 行内（就是 fact B1 那一句的括号部分）。译文里我把这句限定词直接整段搬进了 fact B1 的引文（"this is never pushed before the publish that writes the row: that publish is the new instance's first publish, its metadata goes through the instance-switch reserve..."），P1 定义那一句只留了一个指针式的短语 "never pushed before the publish that first writes a row into a brand-new instance's table"。核对时发现：P1 定义那句的英文比正文这一格单独看少了「元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC」这半句——但这半句在 fact B1 的整段引文里逐字都有，两处合起来没有漏。

定稿：不改——P1 定义句保留短版指针，理由写在此处；核对者需要那半句限定词时去读 fact B1 整段。

## 4. Rule P3（正文 `_m2-safety-r3-body.md:16`）

原文（整行）：`P3（ND+A1n + C283，用户定的组合） | 同 P2 | 同 P2 | 同 P1`

英文项：available(d)/demand(d) 同 P2；不过时同 P1（同样的最多 8 次推空发布与 fact B2 的上限）。

首稿缺的：无——「用户定的组合」这半句括注是身份说明，不影响可算的规则本身，没译入 Part 1（Part 1 只保留可执行的规则定义），略去的原因写在此。

定稿：与首稿相同。

## 5. Fact B1（D16（发布语义） 已定项 1「准入」行，`.claude/kb/decisions/16-发布语义.md:41`）

原文（整行，`grep -n` 现查确认）：`准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, df)；准入不够时先推空发布抬 F（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC），一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC`

英文项（整段引文）：Admission: allocatable = min(reallocatable + live metadata minus reserve pool, df); when admission is insufficient, first push an empty publish to raise F (this is never pushed before the publish that writes the row: that publish is the new instance's first publish, its metadata goes through the instance-switch reserve, and if the user-data redo within that same publish is insufficient it returns ENOSPC directly); at most B = 4 + 2 k_tol publishes are pushed within one admission attempt (k_tol = 2, so B = 8); only report ENOSPC after doing the full amount and still being insufficient.

首稿缺的：原文「同一次发布里的用户数据重做照走准入、不够返回 ENOSPC」直译是「同一次发布里的用户数据重做仍然要走准入检查，不够时返回 ENOSPC」——首稿写成 "if the user-data redo within that same publish is insufficient it returns ENOSPC directly"，把「仍然要走准入检查」（照走准入）压缩成了默认前提，没有单独成句。核对后认为：下一句直接说「不够返回 ENOSPC」已经蕴含了「要走准入检查」（不检查就不会有「不够」这个判断），压缩没有丢限定词本身，只是没有把「照走准入」四个字单列。

定稿：保留首稿译法，本条记录压缩点、不改写——因为把「照走准入」单独译成一句会让英文读起来像是新加了一个检查步骤，反而引入歧义。

## 6. Fact B2（D16（发布语义） 已定项 1「抬 F 的上限」行，`.claude/kb/decisions/16-发布语义.md:36`）

原文（整行）：`抬 F 的上限 | 准入抬 F：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。卸载抬 F：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小）`

英文项（整段引文）：Ceiling for raising F: admission raise-F uses min(the newest persisted valid root on that disk, the 4th-newest non-empty persisted valid root); when there are fewer than 4 non-empty valid roots, the oldest valid root is used instead. The target of one reclaim pass = min(the release-generation being freed this time, the 4th-newest non-empty root). Unmount raise-F (a different, non-admission path) instead raises F to the txg of the currently active version, with no separate ceiling check.

首稿缺的：无——「等价于上限取最新的持久有效根，不按盘取小」这半句是解释卸载抬 F 与准入抬 F 上限公式的等价关系，与 Part 1/Part 4 的问题无关（问题只用准入抬 F 那一半），核对后判定：略去不改变任何一道题的可解性，但为了「整行抄」的纪律，这半句本该保留却被首稿删掉了。

多出的：无。

定稿：本条判定为**漏译**，需要补——英文引文补回这半句。（见下方「本轮改动」。）

## 7. Fact A5 的解释句（`m2-safety-r2-opus-output.md:76`）

原文（整句）：`21 − 20 = 1，实际掉了 28：多出的 8 槽 = 这次新写的 14 槽里不算需求的那 8 槽（分配记录树、记账树、映射树的节点与树表，space_budget_of_role 归到保留池那一格）。`

英文项："21 minus 20 equals 1, but the actual drop was 28: the extra 8 slots are the 8 out of this write's 14 newly-written slots that are not counted in demand (the allocation-record tree, the accounting tree, the mapping tree, and the tree table)."

首稿缺的：原文括注最后半句「space_budget_of_role 归到保留池那一格」（这几个角色被 `space_budget_of_role` 归类为 `CheckpointReservePool` 那一档，不是 `Demand`）没有译出。

定稿：判定为漏译，需要补——按下面「本轮改动」处理，加回这半句的等价英文。

## 本轮改动（核对之后对提示文件做的两处补丁）

1. Fact B2 引文补回「等价于上限取最新的持久有效根，不按盘取小」，英文追加一句："(this is equivalent to taking the ceiling as the newest persisted valid root without taking the minimum across disks)"。
2. Fact A5 解释句补回「这几个角色被 space_budget_of_role 归到 CheckpointReservePool 那一档」，英文追加一句："these four roles are classified by space_budget_of_role as CheckpointReservePool, not Demand"。

两处都已经写回 `research/prompts/m2-safety-r3-local-attack.md`（用 `research/scripts/replace-once.py` 定点改），改后逐字节核对过与本表定稿一致。

## 8. 工具闸导致的格式改动（不是转述，是给自动检查器让路）

`research/scripts/ask-local.sh` 对提示文件本身也跑一遍 `corruption-check.py`（不止查模型的答复）。第一次提交时判红：`cjk=12 粘连=7`。

粘连的 7 处都是 `[:;,]\w`（标点前面不是字母数字、后面紧跟单词字符）命中的假阳性，逐处都是 Rust 的 `::` 命名空间分隔符或本轮日志格式里 `+:`/`-:` 这种记法，不是丢字：

- `PublishError::SpaceAdmissionRefused`、`MountError::SpaceAdmissionRefusedBeforeAcquisition`（正文 Rule T 的判不过时那句）
- `SparseDevice::default()`（fact A2 的代码整行引用）
- `publish+:av[` `mount-:av[`（fact A1 引用的 e7.log 第 88 行）
- `publish+:av[` `mount+:av[`（fact A6 引用的 e7.log 第 87 行）

CJK=12 是三处 `.claude/kb/decisions/16-发布语义.md`（文件名本身含中文）。这三处虽然没有单独触发判红（`cjk<100` 时命中率算 0），但违反「给本地腿的提示一律用英文」，一并改掉。

处理：不改事实、不改数字、不改标识符本身的拼写，只做两处纯格式改动，且在提示文件开头加了一句说明这两处改动（`research/scripts/replace-once.py` 定点加的那一句，紧跟在「Answer format. Answer in English.」后面）：

1. 三处 `.claude/kb/decisions/16-发布语义.md` 改写成不含汉字的 `decision file D16`（`research/scripts/replace-batch.py`，count=3，一次性核对三处都命中）。
2. 六处 `::`、`+:`、`-:` 后补一个空格（`PublishError:: SpaceAdmissionRefused`、`MountError:: SpaceAdmissionRefusedBeforeAcquisition`、`SparseDevice:: default()`、`publish+: av[`×2、`mount-: av[`、`mount+: av[`），逐处用 `research/scripts/replace-batch.py` 定点改，改前 `--dry-run` 核过 6 处都恰好命中 1 次。

改完复核：`corruption-check.py` 对提示文件判绿（`cjk=0 words=2176 粘连=0`）。这一节记录的是「为什么这几个字符看起来和原文不一样」，不是转述错误，因此单列在转述核对表之外、放在同一份文件里存档。

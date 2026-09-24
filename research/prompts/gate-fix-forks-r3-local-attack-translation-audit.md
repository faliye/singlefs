# gate-fix-forks-r3 本地攻方提示 逐句核转述

核对对象：`research/prompts/gate-fix-forks-r3-local-attack.md`（英文提示，交给本地模型）。
下表每行：英文项（提示里的原句或段落，按内容摘一段定位，不摘句改写）/ 原文文件:行 / 首稿缺的 /
定稿。原文行号在下列各源文件里现查（`grep -n` 现取，非从背景材料数）。

## 一、T4 部分（80 号射程与 5 份 .rs 的事实表）

| 英文项（提示里，定位摘要） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| "every experiment binary must carry at least one assertion that is pinned to an absolute value, not only a relative comparison between several arms" | `.claude/gate.d/80-absolute-assertions.sh:2` | 无 | 与原文「每个实验二进制都要有钉绝对值的断言」及测试纪律「只让多条臂互相比」对齐，未丢限定词 |
| "every file directly inside one fixed directory, research/e7-index-bench/src/bin, plus, in any other crates/*/src/bin or research/*/src/bin directory, every file whose name starts with the letter e, followed by one or more digits, followed by an underscore" | `.claude/gate.d/80-absolute-assertions.sh:18` | 无 | 与原文「射程：research/e7-index-bench/src/bin 下的每一份，加上别处 crates/\*/src/bin、research/\*/src/bin 下文件名以 e<数字>_ 开头的」逐项对齐 |
| "(the written form for an experiment number, registered in a project file named .claude/abbreviations as e<digits>); recognition goes by whether the file is an experiment, not by which directory it sits in, so an experiment living under crates/ follows the same rule as one under research/" | `.claude/gate.d/80-absolute-assertions.sh:19` | 首稿把「`.claude/abbreviations` 登记的」简化成「registered elsewhere in the project」，丢了登记位这个具体文件名 | 补回「registered in a project file named .claude/abbreviations」 |
| "elsewhere, a file whose name does not start with that e<digits>_ pattern is treated as an apparatus tool, for example one that compares a device log against ground truth field by field; such a file is not judged at all, it is only listed by name on the stage's success line, with that list computed fresh on every run" | `.claude/gate.d/80-absolute-assertions.sh:20-21` | 无 | 与「别处不以 e<数字>_ 开头的是装置工具（例：拿设备日志逐项比 ground truth 的），不判，成功行逐个列名，清单现算」逐项对齐 |
| Row 1 doc-comment quote: "E156 rerun, second time, first stage: the cost numbers for the four alloc-basis forks, doing only fork 7 (the defer check) and its prerequisite S1 and anchor (pre registered in research/prompts/e156-r2-prereg.md, part five, 5.7, first paragraph)." | `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:1-2`（快照树） | 无 | 「S1/锚点」的「/」按并列项译成 and（两者都是前置条件，非二选一），与上下文「与它前置的」一致 |
| Row 1 fact: "this row's source file was rewritten, wholesale, by a different, unrelated working session, after this project's previous round of three-way review of this same stage 80 question had already finished and handed back its verdict" | `research/prompts/_gate-fix-forks-r3-body.md:19` | 无 | 多出「this project's previous round of three-way review of this same stage 80 question」「unrelated」两处，为让缺上下文的读者看懂「第二轮交回」指什么、那个会话与本轮无关，属补充说明，不改变事实本身 |
| Row 2 doc-comment quote: "E158, root choice and repair, four forks, pre registration, first stage: the apparatus to be checked into the results store." | `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:1`（快照树） | 无 | 与「E158（择根与修复四岔路）跑前登记第一段：入库装置。」逐项对齐 |
| Row 3 doc-comment quote: "Host side: take the two device-side logs recorded by QEMU blklogwrites, and compare them field by field, disk by disk, against the recorded stream obtained by replaying the same write path on the host, same parameters, same bytes." | `crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs:1`（快照树） | 无 | 「逐盘逐项」原文序是先盘后项，英文写成 field by field, disk by disk（先项后盘），纯语序调整，不丢限定词 |
| Row 4 doc-comment quote: "Virtual-machine tier, QEMU or KVM: run the whole write path of the first transaction on two real virtio disks, cold-reboot, then recover and read the file back." | `crates/singlefs-harness/src/bin/first_transaction_on_device.rs:1`（快照树） | 首稿把「虚机档」（虚拟机档位）误译成「Real-machine tier」，方向译反；「两块真 virtio 盘」的「真」本来修饰盘，首稿被误挪去修饰了机 | 改译「Virtual-machine tier」，「real virtio disks」的「real」仍准确落在盘上；此处 Row 5 里同一个「虚机档」有第二处引用，一并改译为 virtual-machine tier |
| Row 4 fact: "row 4's own doc comment states, in its own words, that it is fed into a virtual machine by a script named research/scripts/vm-bench.sh ... That same later line also states, in its own words, that this binary only judges what it can judge by itself (recovering the file correctly, and the sequence of on-disk segments), and that a separate question, whether what actually arrived on disk matches what the program believed it sent, is left to row 3's binary, first_transaction_device_log_check, to judge afterward on the host side." | `crates/singlefs-harness/src/bin/first_transaction_on_device.rs:5-7`（快照树） | 首稿完全漏掉「这个二进制只判它自己判得了的（恢复读回文件、段序列）」这半句，只译了后半句「盘上实际收到的是不是程序以为的……判」 | 补回「this binary only judges what it can judge by itself (recovering the file correctly, and the sequence of on-disk segments)」；「VM_DISKS=2，设备路径排在参数前面」与「vm-bench.sh 的完整性闸」两处操作细节判定与任务判据无关，不补，理由见下节 |
| Row 5 doc-comment quote: "E142, first-transaction dry run, the implementation side of measurement 5: run scenario:: run_first_transaction on two in-memory disks ..., same parameters as the virtual-machine tier and the E142 apparatus: same fsid, same write timestamp, same 3000-byte content, two 4 GiB disks, and print the bytes written to those 21 regions by the first transaction, one result line per region." | `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs:1-3`（快照树） | 首稿同样把「虚机档」误译成「real-machine tier」（小写 r，未被首次 grep 用大写 R 抓到，第二遍全文核对时才发现） | 改译「virtual-machine tier」；「scenario::run_first_transaction」按提示自己定的格式第 6 条，在双冒号后插一个原文没有的空格，并在同一句里说明这处插入不是丢字 |
| Row 5 fact: "yes, but only as one step inside a function named driver_e142, not as a dedicated function of its own the way rows 1 and 2 have; driver_e142 runs ... and captures its output into a temporary file, which is then handed as a command-line argument to a different binary that performs the actual comparison and prints the assertions, if any; no assertion about absolute values lives inside first_transaction_region_bytes.rs itself" | `research/scripts/replay.sh` 里 `driver_e142` 函数体（快照树，函数名定位，不写行号） | 无 | 按函数体现读复述，未加未减 |

## 二、T8 部分（clip 函数机制说明）

`Exhibit 1` 的五步描述基于 `.claude/scripts/gen-decision-items.py:23-70`（快照树，与提示里 `clip-extract.py` 逐字节核对一致，见运行记录）。这一段是把 Python 正则与控制流改写成英文操作步骤，不是逐句译 Chinese 散文，因此下面按语义块核，不按逐句核：

| 提示里的步骤 | 源码位置（快照树） | 首稿缺的 | 定稿 |
|---|---|---|---|
| Step one（截断落在 ASCII 词中间，半截整个去掉） | 第 32-35 行（含注释「按长度截断落在一个 ASCII 词中间...半截整个去掉」） | 无 | 与条件、动作逐项对齐 |
| Step two（未闭合括注回退） | 第 36-37 行 | 无 | 与 while 循环逐项对齐 |
| Step three（没有真截断就直接返回） | 第 46-47 行 | 无 | 与 `if t == text: return` 逐项对齐 |
| Step four 里编号形状与左边界 | 第 42、49 行（含注释「编号按 doc-lint 认编号的同一个形状剥：整词...前一个字符不是字母、数字、.、_、-」） | 无 | 与正则 `(?<![A-Za-z0-9._-])([A-Z]+-?\d+(?:\.\d+)*)\s*$` 逐项对齐 |
| Step four 里登记词不剥 | 第 44-45、50 行 | 无 | 与「kb 里登记过的领域词...不剥」逐项对齐 |
| Step four 里连接符收尾 | 第 51 行 | 具体六个字符（顿号、逗号、全角逗号、中点、的、与、和）没有直接贴汉字，改用英文描述其语义 | 提示里明说「六个具体的中文标点或连接字符」并逐一用英文解释各自大致含义（顿号=enumeration comma、逗号=ASCII comma、全角逗号=full-width comma、中点=middle dot、「的」与「与」各译一个连接词），未贴原字符本身，理由：避免模型在自己的答复里被迫复述生僻标点、触发字词损坏闸里的粘连检测（`research/scripts/corruption-check.py` 的 glue 规则） |
| not_number_tokens 读法（登记集合怎么建） | 第 62、66-69 行 | 无 | 「read every markdown file found anywhere under .claude/kb...read once and cached」与「按 cwd 下的 kb 读，读一次」逐项对齐；省略了正则里「`<!--` 后可选空白、捕获组前至少一个空白」这类纯语法细节，因为提示没有要求模型自己解析标记——每一格是否登记已经作为给定事实直接写在 Exhibit 2 里（例如「not a member of the registered set of domain words」），模型不需要自己重新解析标记正则 |

## 三、格式安全相关的改写（不是内容翻译，记录改写理由）

| 提示里的写法 | 为什么这样写 |
|---|---|
| "t = text[0:n]"（不写 "text[:n]"） | `research/scripts/corruption-check.py` 的 `splice_marks` 里 `glue` 规则会把「非字母数字紧跟着的冒号/分号/逗号，后面立刻接一个字母」判成掉字信号；`[:n]` 里的 `:` 前面是 `[`（非字母数字），后面是字母 `n`，会命中这条规则。这条规则查的是模型的**答复**，不是这份提示本身，但提示里出现这种写法，模型逐字复述进答复的概率很高，因此提示本身也照这条写法改写，并作为提示的格式规则第 6 条写明「双冒号紧跟字母时要插一个空格并说明」 |
| "scenario:: run_first_transaction"（双冒号后插一个原文没有的空格） | 原文是 `scenario::run_first_transaction`，双冒号第二个冒号前面是第一个冒号（非字母数字），后面紧跟字母 `r`，同样会命中上面那条 glue 规则；Row 5 的事实描述里显式声明这处空格是插入的，不是丢字 |

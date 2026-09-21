# m2-s1-r2 本地攻方：逐字核对表

列：英文项 / 原文文件:行 / 首稿缺的 / 定稿。行号为现查（`grep -n` 现取，非背景材料数出）。攻击面限于 I5、I6；本表只核这两格用到的转述。

## A 组：SECTION 1（对应 I5）用到的转述

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| FACT TABLE 1「Ruling」（持久顺序 + fsync 等根槽） | `.claude/kb/decisions/16-发布语义.md:169` | 缺：原文本句后半「fsync 返回条件多一句：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回、不向管理员确认回退（做法与次数在已定项 8）」整句未译入 | 有意删去，理由写在此处：那半句管的是新实例（挂载/恢复/切换/回退）第一个根的双盘确认，是另一个条件分支，不是稳态组提交场景，与 Section 1 的六问（都在稳态单实例内问等待时间）不相关；若判者认为需要该分支也应算作转述缺陷，此处如实记录未译 |
| FACT TABLE 1「Scope note」（屏障口径：两道 FLUSH+FUA=三个序点） | `.claude/kb/decisions/16-发布语义.md:174` | 无实质遗漏；英文把「D25（目标负载优先级）推导的两个」改写成「a separate workload-priority ruling」（不点决策编号） | 保留改写：给本地模型一个不依赖内部编号体系的说法，语义不变（仍是「比另一条推导多一个序点」），不是新增或删减限定词，是命名替换 |
| FACT TABLE 2（甲B 定义） | `research/prompts/_m2-s1-r2-body.md:85` | 无 | 直译，「甲0」译 Arm-A0、「甲B」译 Arm-A-batch，「不改任何条款」译 "No established ruling is changed by this arm"（加 "by this arm" 为语法补全，非新增事实） |
| GLOSSARY「wal_full」定义 | `research/prompts/_m2-s1-r2-body.md:86` | 有意删去：原文「一个间隔里写了又被换掉的每个中间版也写一条已释放的分配记录」（K9′ 中间版细节）未译入 | 因为 Section 1 六问都不问 wal_full 内部的中间版记账，只把它当「另一条被比较的臂」用；删去不影响六问的可答性，如实记录 |
| GLOSSARY「Arm-B-M」定义 | `research/prompts/_m2-s1-r2-body.md:88` | 有意删去：原文「施加时把叶换进内存里的父节点」「中间版照 K9′ 记」两处未译；「挂载后第一次发布」被泛化成「the normal publish path」（原文特指挂载后第一次发布，不是任意一次后续发布） | 泛化加一条待核记录：这一处英文比原文更宽（把「挂载后第一次发布」这个特定时点泛化成「later through the normal publish path」），本应逐字译出该限定词；因为 Section 1 六问不依赖这个时点细节，予以保留但在此列出「多出的宽度」——**更准确地说是英文比原文更窄的限定词被丢弃，不是多加**，此处按「缺」处理 |
| FACT TABLE 3 行 a（row7_fold_point 360 行的构成） | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:66` | 加了限定词 "first"（fold_point_zero/fold_point_ten_percent 译成 "the smallest k at which s first becomes/drops"）；原文只说「翻面 k」「s ≤ 0」「s 压到一成以内」，没有逐字说「最小的、第一次出现的那个 k」 | 保留添加：这是从「翻面点」（fold point）这个术语本身、以及行 c 给出的「wal_full 8 那一档 97 组、16 那一档 22 组、4 那一档 6 组」（多个不同 k 值分别计数，暗示每行只记一个特定的 k）反推出的必要澄清，不加则模型无法确定 fold_point 是否可以是「任意满足条件的 k」还是「最先满足条件的那个 k」；已在核对表单列 |
| FACT TABLE 3 行 b（s 的定义，泛化到 Arm-B-M） | `research/prompts/_m2-s1-r2-body.md:59` | 原句只对 "wal_full-K9′" 定义 s（`s = 1 − wal_full-K9′ 摊销 ÷ 甲摊销`），英文加了一句「the same row7_fold_point table reports an analogous s computed against Arm-B-M instead of wal_full」——这是推广，原文这一行没有逐字这样写 | 保留推广，理由已在英文本身写明是推广（"an analogous s"，不是逐字引用）；依据是行 c 原文本身把 wal_full 与 乙-M 两套 s 并列报数（`.claude/kb/experiments/155-...md:70`），可交叉验证这两套 s 用的是同一个定义换了分子 |
| FACT TABLE 3 行 c（360 组读数原文） | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:70` | 无 | 逐句直译，含「不是几乎全部」那句保留 |
| FACT TABLE 3 行 d（P=100 一格的字节数） | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:69` | 有意删去：原文「B12/B13/B14 三个手算格在单测与产物里逐字节一致」这半句（数值的验证出处说明）未译入 | 因为不影响这个数值本身的含义，只是它的验证方法说明，Section 1 的问题只需要这个数值，不需要它的验证过程；如实记录已删 |
| FACT TABLE 4（D25 用户答复原文） | `records/2026-09-19-里程碑二遗留收拢.md:143` | 「根据硬盘和电脑的性能来做」译成 "sized to the disk's and the machine's performance"——原文「来做」更宽泛（"design it based on"），英文「sized to」把它收窄成「按容量/规模定」这一种读法 | 保留收窄：因为上下文（"目标负载越高越好"）本身就是讲规模/容量，"sized to" 是这句话在本轮语境下最贴切的收窄，但确系比原文窄，在此列出 |

## B 组：SECTION 2（对应 I6）用到的转述

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| FACT TABLE 5（旧模型几何：128 叉、内部 4 层、块 4KiB） | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:217` | 有意删去：原文「记录 48+每项(24或24+32)字节；意图日志记录定长48字节（模型参数……）」与「叶=用户数据块，一次操作弄脏1(rand)/2(metaheavy)/8(seq)个叶」两段未译入，提示文件里已用括注明写「省略了不影响树高/扇出的字段宽度与每工作负载弄脏叶数条款」 | 有意删去且已在提示原文里自陈理由，不是无声消失 |
| FACT TABLE 6（56 格峰值表，7×8） | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:63-71` | 无；数字逐格核对：streams=2/4/8/16/32/64/128 各 8 列 + 峰值位置列，与源表逐格比对全部相符 | 整表照抄，未改一个数字 |
| FACT TABLE 6「峰值逐行落在批≈流数」引句 | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:73` | 无 | 直译 |
| FACT TABLE 7（预测公式与拟合误差） | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:78-80` | 英文把「甲每批」「乙每批」译成 "Arm intent's device block writes per batch" / "Arm wal_leaf's device block writes per batch"，加了 "device block writes" 这个单位澄清——原句「甲每批 ≈ 批 + min(流数,批)×树高 + 2」本身没有单位词 | 保留添加：依据是 FACT TABLE 6 表头本身写明这是「设备写块数 / 用户写次数」的写放大比（`research/prompts/_m2-s1-r2-appendix.md` 抄自 `.claude/kb/experiments/16-...md:19-20` 「写放大 = 设备写块数 / 用户写次数」），公式与表格算的是同一个量，加单位词是消歧不是改事实 |
| FACT TABLE 8「扇出公式」 | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:256` | 英文加了 floor()，原句只写「扇出 = (节点字节 − 64) / 40」没有写取整 | 保留添加：用 65536 那一行反推——(65536−64)/40=1636.8，表格给的扇出是 1636（整数），不可能不取整；已在提示原文里用这个反推过程说明为什么加 floor()，不是凭空加的 |
| FACT TABLE 8「树高规则」 | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:256` | 无 | 直译 |
| FACT TABLE 8「叶数固定」 | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:274` | 英文加了「128^4×4096=268435456×4096=2^40 字节，即恰好 1 TiB」这个算术推导——原句只说「≈2.68亿个4KiB数据块≈1TiB」，两处都是「≈」，没有断言恰好相等 | 保留添加，且已在提示原文里明写「this exact equality is arithmetic, not itself part of the quoted sentence」，把推导与引文分开标注；这条算术是笔者验证过的准确值（128^4=268435456，×4096=1099511627776=2^40），供 Section 2 问题 2 的「恰好 1 TiB」锚点用 |
| FACT TABLE 8「实测表」（512/1024/4096/16384/65536 五行） | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:268-272` | 有意删去两列：原表还有「批10多流祖先节点」「批10多流祖先字节」两列，未译入 | 因为 Section 2 的问题都不用这两列，只用扇出/树高两列；如实记录已删两列 |
| FACT TABLE 9 行 a/b（今天的数据单元 32768、索引节点 16384） | `.claude/kb/decisions/18-块里携带什么信息.md:215` | 有意删去：原文还提到「打包记录单元（码3）也是32768、对齐32768」，只保留数据单元与索引节点两档，未译打包记录单元 | 因为 Section 2 问题只用数据单元与索引节点两个宽度，打包记录单元不在攻击面内；如实记录已删 |
| FACT TABLE 9 行 c（inode 树内部节点扇出 135） | `.claude/kb/decisions/08-核心索引结构.md:160` | 英文加了一整句警告：「这个扇出/条目宽只属于 inode 树，别的树（extent、分配记录、记账、中央映射、树表）宽度未给，不许假设与 135 相同」，以及括注「树表自己的条目宽是 200 字节，不是 120」——原句本身只有 inode 树这一行的数字，没有这句警告 | 保留添加，且这条添加是关键：防止模型把 inode 树一棵树的扇出错当成全系统通用扇出去用；括注里的「树表 200 字节」现查自 `.claude/kb/decisions/08-核心索引结构.md:232`（已定项 8：「树表条目合计 200 字节」），是真实存在的另一个数字，不是编造的对照值 |
| FACT TABLE 9 行 d（journal 记录 4096/307/56/67） | `crates/singlefs-format/src/lib.rs:139,148,151,155` | 无；已现查代码逐行核对：139 行 `JOURNAL_RECORD_BYTES: u64 = 4096`、148 行 `JOURNAL_HEADER_BYTES: u64 = 307`、151 行起 `JOURNAL_NAMED_ENTRY_BYTES`（14×2+1+8+8+10+1=56）、155 行起 `JOURNAL_NAMED_ENTRIES_PER_RECORD`（(4096−307)/56=67，整除） | 直译成事实陈述，未改动数字；行号本身未写进提示文件（提示文件里只写数字，不写行号），此处核对表单独记录来源行号 |

## C 组：命名统一（非原文/译文分歧，记此备查）

GLOSSARY 把 Section 1 与 Section 2 的臂名分成两组、互不假设相同（Arm-A0/Arm-A-batch/wal_full/Arm-B-M 属 Section 1 的 E155 材料；intent/wal_leaf 属 Section 2 的 E16 材料）。这是笔者的编排选择，不是某一句原文的翻译；两组材料本身在各自来源文件里确实是两次独立的测量（E155 与 E16 是两份不同的实验页），未把「乙-M」与「wal_leaf」这两个概念上相近但定义不完全相同的臂强行合并成一个英文词——初稿曾把「wal_leaf」与「Arm-B-M」并作一个词写「wal_leaf/Arm-B-M」，复核时发现两者定义不逐字相同（乙-M 特指「挂载后第一次发布」落盘祖先，`research/prompts/_m2-s1-r2-body.md:88`；wal_leaf 只泛泛说「延到 checkpoint」，`.claude/kb/experiments/16-journal的角色WALvs意图日志.md:15`），定稿改成两个不互相别名的独立词。

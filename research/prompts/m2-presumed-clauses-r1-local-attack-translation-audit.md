# m2-presumed-clauses-r1 本地攻方 英文提示 逐句核对表

列：英文项 / 原文文件:行 / 首稿缺的 / 定稿。逐条核对 `research/prompts/m2-presumed-clauses-r1-local-attack.md` 里每一句转述自中文材料的内容。

## K1 部分

1. 英文项：Part K1 intro 段 "All three candidates below concern one specific event: the row-writing publish and the warm-up publishes that immediately follow it..."
   原文文件:行：`research/prompts/_m2-presumed-clauses-r1-body.md:28`（"写行与暖机那几次发布里，实例表单元排在全部提交内生块之前取落点"）
   首稿缺的：首稿的三个候选定义（K1-A/B/C）里没有这句范围限定，只写了「排在 t2–t8 之前 / 之后 / 之间」，没交代这个次序只在「写行与暖机那几次发布」这一件事里说——一处限定词漏了。
   定稿：加了这一整段，写清三个候选都只描述「写行与暖机」这一件事里的次序，并说明这是今天实现里唯一会写实例表单元的时机。

2. 英文项：Candidate K1-C 的具体定义（"between t7 ... and t8 ..."）
   原文文件:行：无对应原文——`_m2-presumed-clauses-r1-body.md:39` 只写「某两类角色之间」，没有指定是哪两类。
   首稿缺的：不适用（这不是一句转述，是我自己的具体化选择）。
   定稿：选「t7（中央映射树，已定为倒数第二）与 t8（树表单元，已定为最末）之间」这个切法，因为这两个角色在 D3 已定项 10 ⑤ 原文里本来就各自有一个命名的固定位置（倒数第二 / 最末），是「某两类角色之间」最贴着原文读法的一种具体化；提示里没有把这一句标成「引用」，就当成攻方自己的候选定义。

3. 英文项：K1-row-1 引文（D3 已定项 10 ⑤，bump 次序那句）
   原文文件:行：`.claude/kb/decisions/03-空间分配.md:198`
   首稿缺的：省略了「码 3 打包容器按「数据单元」那一档取落点」那半句（与 K1 的问题——实例表排第几——无关）；省略了「（D19（块指针的结构与宽度预算） 已定项 9）」那个书目括注（纯引用指针，不影响这句话本身的内容）。
   定稿：两处省略都保留省略，未回填——都不是这句话本身的限定词，是与本题无关的另一件事（打包容器落点）和一个不改变语义的交叉引用括注。

4. 英文项：K1-row-1 里 "This clause's own table enumerates exactly the seven roles t2 through t8..."
   原文文件:行：`.claude/kb/decisions/03-空间分配.md:204-210`（t2–t8 角色表）
   首稿缺的：不适用，这是我自己现读角色表之后加的一句事实陈述，不是转述某一句中文。
   定稿：保留，因为它是这一格要判的关键事实（零处提到实例表），有独立核实的行号支撑。

5. 英文项：K1-row-2 引文（D3 已定项 8 标题 + 第 2 条）
   原文文件:行：`.claude/kb/decisions/03-空间分配.md:154`（标题）、`159`（第 2 条）
   首稿缺的：省略了「已定项 5「用户数据块的落点不受此约束」原样成立、不推广」那半句；省略了「C146（无空段时的回落政策全仓无定义） 第 ② 条…由此有了政策」那句欠账交叉引用。
   定稿：两处省略都保留省略——前者是另一项（已定项 5）跟这一条的关系，不改变已定项 8 第 2 条本身对 K1 问题的答案；后者是欠账簿记的指针，不是这句定案本身的限定词。

## K3 部分

6. 英文项：Candidate K3-1 引文 "The first version has no clean-shutdown marker; every reopen goes through recovery."
   原文文件:行：`crates/singlefs-core/src/mount.rs:4`
   首稿缺的：无——逐字对应「第一版没有干净关闭标记，重开一律走恢复。」
   定稿：保留逐字翻译，不改。

7. 英文项：Candidate K3-1 里 "Every mount... runs the identical recovery path: full scan of the root ring, per-record verification, root selection, replay of the determined prefix."
   原文文件:行：`.claude/kb/decisions/16-发布语义.md:99`（已定项 1 射程段「先全环扫描、逐条验证、不许先信 tail」原样成立）与 `crates/singlefs-core/src/mount.rs:1-3`（模块头概述「先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根…」）
   首稿缺的：不适用，这是把两处原文合并转述成一句概述，不是逐句直译。
   定稿：保留合并转述，因为它只是给模型一个背景描述、不作为可判的事实表行（K3 的事实表行只有 K3-row-1 到 4 那四行，这句不在核判范围内）。

8. 英文项：Candidate K3-2 段（clean-shutdown marker 假设写法）
   原文文件:行：`research/prompts/_m2-presumed-clauses-r1-body.md:46`（"有标记"这一候选，正文只给了候选名，没有给出候选的具体机制）
   首稿缺的：不适用——这不是转述，是我自己对「有标记」这个候选的具体化（写在卸载时、下次挂载读、有效时可以跳过或缩短恢复路径的哪一部分留给每一行自己判）。
   定稿：保留具体化，并且在文中写明「跳过哪一部分是这个候选本身留白的地方，你在下面每一行里自己判要不要跳」，不替模型预设答案。

9. 英文项：K3-row-1 引文（D16 已定项 1 表格「准入」行）
   原文文件:行：`.claude/kb/decisions/16-发布语义.md:39`
   首稿缺的：无遗漏；把「回退下界 F」的全名代入了「F」这个符号（表格另一行「回退下界 F」定义在 `16-发布语义.md:29`）。
   定稿："raise the rollback floor F" —— 补全符号 F 的全名，来源是同一个已定项 1 表格里「回退下界 F」那一行，不是凭空加的。

10. 英文项：K3-row-2 引文（D16 已定项 8 标题 + 定案）
    原文文件:行：`.claude/kb/decisions/16-发布语义.md:185`（标题）、`187`（定案）
    首稿缺的：无遗漏；追加了一句 "described in K1-row-4 above" 作为读者向导。
    定稿：保留向导句，明确标成向导不是引文本身。

11. 英文项：K3-row-3 引文（D16 已定项 1 表格「回退候选集」行 + D23 已定项 14 回退例外段）
    原文文件:行：`.claude/kb/decisions/16-发布语义.md:38`、`.claude/kb/decisions/23-journal的角色与格式.md:363`
    首稿缺的：无遗漏；追加了一句 "This candidate set is computed freshly at every mount and at every raise of F..."。
    定稿：这句取自 `23-journal的角色与格式.md:363` 同一段后半句「每次挂载与每次抬 F 按当时的候选集重算」，忠实转述、非凭空加。

12. 英文项：K3-row-4 引文（D16 已定项 1 表格「生效」行）
    原文文件:行：`.claude/kb/decisions/16-发布语义.md:37`
    首稿缺的：无遗漏，逐句对应。
    定稿：不改。

## 补：K1-row-3、K1-row-4（漏编号，接续在这里）

13. 英文项：K1-row-3 引文（D16 已定项 9 标题 + 定案）
    原文文件:行：`.claude/kb/decisions/16-发布语义.md:205`（标题）、`207`（定案）
    首稿缺的：无遗漏；"whenever the accounting tree exists, it is rewritten on every publish, including an empty one" 里 "on every publish, including an empty one" 是从同句前半「空发布也是发布」带出来的衔接，不是凭空加。
    定稿：保留，逐句对应「记账树存在时就写——空发布也是发布…」。

14. 英文项：K1-row-4 引文（D22 已定项 16 第 5 句）
    原文文件:行：`.claude/kb/decisions/22-单元原子性怎么合成.md:322`（标题）、`330`（第 5 句）
    首稿缺的：省略了「写成常量比写成运行时谓词少一条只有一个取值的分支」这句设计理由（不改变第 5 句给出的两个常量本身，只是解释为什么写成常量）。
    定稿：保留省略，因为它是「为什么这样设计」不是「这句定案给出什么值」，与 K1 逐格填表要的答案无关。

## 小结

四条已定分项事实表行（K1 四行、K3 四行）的引文本身逐字核对，没有一处漏掉会改变「这个候选下这条分项给出什么答案」这件事的限定词；被省略的内容全部是与本题无关的另一件事、或纯粹的书目交叉引用括注。唯一一处真正补回来的限定词是条目 1（K1 三个候选的范围限定「写行与暖机那几次发布里」）。追加的内容（条目 2、4、7、8、9、11）均有各自的来源标注：3 处是我自己对模糊候选（K1-C、K3-2）的具体化并在文中标出、3 处是从同一份引文的相邻半句里带出来的忠实衔接，没有一处凭空加。

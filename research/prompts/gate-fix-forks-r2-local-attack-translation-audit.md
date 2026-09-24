# 本地攻方提示译文核对表（gate-fix-forks-r2）

每一句转述与原文并排，写完对照过一遍。栏位：英文项（提示文件里的位置）/ 原文文件:行 / 首稿缺的 / 定稿。

## 1. T4 场景说明段（提示文件第 19-30 行）

英文项：Today it only looks at every .rs file directly inside one fixed directory,
research/e7-index-bench/src/bin. The proposed new rule instead judges every .rs file
directly inside research/e7-index-bench/src/bin, plus every .rs file directly inside any
crates/*/src/bin or research/*/src/bin directory whose filename starts with the letter e
followed by one or more digits and then an underscore (for example e156_something.rs).
Every other .rs file under those src/bin directories is only listed by name in the
stage's success line; it is not judged at all under the proposed rule.

原文 research/prompts/_gate-fix-forks-r2-body.md:49：
「形态：判 `research/e7-index-bench/src/bin` 下全部，加上 `crates/*/src/bin`、
`research/*/src/bin` 下文件名以 `e<数字>_` 开头的；其余只在成功行逐个列名。」

首稿缺的：没有漏掉限定词。首稿比原文多三处：一是「directly inside」这个消歧（原文「下」
本身兼指「直接在里面」与「递归往下」两种读法，这里选了「直接在里面」，依据是
80-absolute-assertions.sh 的实现用的是 `for f in "$directory"/*.rs`，不递归，
是现查代码坐实的消歧，不是凭感觉加的）；二是「it is not judged at all under the proposed
rule」，是对原文「只在成功行逐个列名」的显式点破（原文靠「判 vs 只列名」的对比隐含这层意思，
英文补了一句挑明）；三是举例「(for example e156_something.rs)」，例子取自 T4 标题本身
（body.md:47 的 `e<数字>_*.rs`），不是凭空造的。

定稿：三处加的限定词都保留，理由已写在上一段；没有发现被删掉的限定词。

## 2. T4 问句（提示文件 T4-Q1 到 T4-Q7）

原文 research/prompts/_gate-fix-forks-r2-body.md:51：
「问：按文件名认实验，会漏掉哪一类实验、误罩哪一类工具（逐份判今天那 4 份，再给出以后会出现
的形状）。」

首稿缺的：原文一句里三层意思（逐份判今天 4 份 / 给出漏掉的实验形状 / 给出误罩的工具形状），
提示文件拆成了 T4-Q1 到 T4-Q4（逐份判）加 T4-Q6（漏掉哪类实验的形状）加 T4-Q7（误罩哪类工具
的形状），三层都落了地，没有丢。T4-Q5（数「判断与射程列不一致的格数」）是我自己按「按文件名
认实验」这个机制另外操作化出来的一问，原文没有这一句，不算转述，这里注明不算翻译对照项。

定稿：T4-Q1 到 T4-Q4、T4-Q6、T4-Q7，一一对应原文三层意思，见上。

## 3. T7 场景引入段（提示文件第 129-138 行）

英文项：Stage 52 ... used to print a message and exit with code 77 (meaning "skipped,
nothing to judge") when it could not find a script it depends on ... Stage 31 ... used
to behave the same way when it could not find a generator script it depends on ... A
change under review makes both of these cases exit with code 1 (meaning "failed")
instead of 77. Stage 31 also used to have a fallback ... That fallback line has been
deleted in the same change.

原文 research/prompts/_gate-fix-forks-r2-body.md:63（标题）：
「T7　52 号找不到脚本、31 号找不到生成器退 1」

首稿缺的：标题本身只有六个字概括这件事，没有具体展开「以前是 77、现在改成 1」「31 号去掉了
哪条后路」。这两处背景不是从 body.md 翻出来的，是我另外从这一轮代码 diff
（research/prompts/_gate-fix-forks-r2-diff.md 里 31-blocking-verdict.sh、
52-segment-registry.sh 两处的改动）现读出来、核对过的事实，不算对 body.md 某一句的转述，这里
注明来源不是同一处。

定稿：引入段照原样保留，事实来源已在这里注明。

## 4. T7 问句（提示文件 T7-Q1 到 T7-Q5，尤其 T7-Q5）

原文 research/prompts/_gate-fix-forks-r2-body.md:65：
「问：列出今天会走到这两个分支的全部场合（真仓整轮、样本自检、`gate.sh --staged` 的临时
worktree、单独跑），哪一个场合下退 1 是误红。31 号去掉了「按脚本位置取不到就退回按 cwd 取」
那条后路：有没有哪个场合原来靠那条后路才取得到生成器。」

首稿缺的：没有漏掉限定词。原文问「哪一个场合下退 1 是误红」是单数问法，提示文件把它拆成
T7-Q1 到 T7-Q8，对事实表里的每一行（A1、A2、B1-B4、C1-C2、D1-D8）逐行判「误红/对/走不到」，
是原问句的细化覆盖，不是窄化——原问句要的那个答案（哪个场合误红）能从这些逐行判断里直接读出。
「31 号去掉了……那条后路：有没有哪个场合原来靠那条后路才取得到生成器」对应 T7-Q5，逐字照译，
没有增减。

定稿：T7-Q1 到 T7-Q8，见上；T7-Q5 与原文第二句逐字对应。

## 5. T8 场景说明段与 Exhibit 1（提示文件第 296-334 行）

原文 research/prompts/_gate-fix-forks-r2-body.md:69：
「形态：`clip` 只在真截断过时剥尾，剥的编号是
`(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$`，与 doc-lint 认编号的形状一致（整词、前一个
字符不是字母数字 `.` `_` `-`）。截断正好停在领域词（`SHA256`、`RAID5`）之后时整词剥掉。」

首稿缺的：这一句本身没有直接译成英文散文——按这一轮「不许用文字描述机制」的要求，机制改用
Exhibit 1（从源文件用 ast 现抽出来的 clip 函数原文）与 Exhibit 2（逐格现跑出来的输入输出表）
代替，原文这句话里的每一层意思都能在 Exhibit 1（正则原文）与表格的第 1-11、12-14 行（截断点
落在哪、剥没剥）里现读出来，没有丢，只是载体从散文换成了代码与实测表。

定稿：Exhibit 1、Exhibit 2 与 T8 场景说明段，见上。

## 6. T8-Q3 引用的那句"不会剩半截"的断言

英文项："when it has a left boundary, a domain word sitting at the truncation point
either survives whole or gets stripped whole; it can never be left half-stripped."

首稿归错的出处：写这一题时最初以为这句话出自 body.md 第 69 行的「形态」描述，核对之后发现
body.md:69 只写了「截断正好停在领域词……之后时整词剥掉」，没有「不会剩半截」这个更强的保证
措辞。这句更强的措辞其实原样写在源码文件自己的注释里。

原文 .claude/scripts/gen-decision-items.py:40（clip 函数正上方的注释，现查行号）：
「带了左边界，截断尾巴上的领域词要么整个留着、要么整个剥掉，不会剩半截（三方判决
gate-fix-forks-r1 的 T8）。」

定稿：T8-Q3 引文文字不改（逐字译得准，「要么……要么……不会剩半截」对应「either survives whole
or gets stripped whole ... can never be left half-stripped」），这里把出处改记成
gen-decision-items.py:40，不是 body.md:69。提示文件本身没有点出处文件名，模型不需要知道这一
点也答得了题，这条只影响我这份核对表的记账，不影响提示文件的内容。

## 7. T8 问句（提示文件 T8-Q1、T8-Q2）

原文 research/prompts/_gate-fix-forks-r2-body.md:71：
「问：按这个正则逐格算一张输入表，找出剥成半截的、剥掉了不该剥的、该剥没剥的（剩下裸引用、
doc-lint 会判红的）。」

首稿缺的：原文三类失效（剥成半截的 / 剥掉了不该剥的 / 该剥没剥的、剩下裸引用）对应 T8-Q1 的
三个类目 half-stripped-word / dropped-whole-word-unnecessarily / left-a-bare-reference，
一一对应，没有丢。第四类 correct 是我自己补的完成项（原文三类是失效类目，「其余算对」在原文里
是隐含的，没有单独一句话），这里补成一个显式类目方便模型逐格填表，不算漏译，是操作化。

定稿：T8-Q1 四类目，T8-Q2 数每类几格，见上。

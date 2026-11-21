#!/usr/bin/env bash
# 门禁 code-source-discipline 的格函数库：unsafe-allowances、gpu-exemptions、crash-point-replay-segments、crate-sources-reachable、ordinal-names、data-names、verus-ghost-names、refused-tests。
# 由 .claude/gate.d/code-source-discipline.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# bad、howto 由入口设好，在调用时才取。

# ── 格 unsafe-allowances：crates/ 里放开 unsafe 的地方，与登记表逐处对上
#
# 登记表 .claude/gate.d/unsafe-allowances.tsv 一处放开一行（八段写法在表头）：每个根一组行。根是 crate 的库根（第 1 段只写 crate 目录名）或一个 bin 根
# （第 1 段写 <crate 目录名>/<bin 根相对 crate 目录的路径>）；没有放开的根一行、第 3 到第 8 段写 -；放开了的根每个放开的项一行。
# 项（第 4 段）三种写法：函数写函数名；直接写在 impl 块里的方法写 <类型>::<方法>（trait impl 与固有 impl 都按实现它的类型认，trait 名不进键：
#   两个类型各实现一次同一个 trait，方法同名也是两个键）；impl 块本身写 impl <Trait> for <类型>（固有 impl 写 impl <类型>）。类型与 trait 都取路径最后一节、去掉 <…>。
# 先判表本身，逐行：
#   - 八段都有内容；第 2 段只由 forbid、deny 组成；写了 deny 才有放开，第 3 到第 6 段（文件、项、项里 unsafe 几处、SAFETY 注释必须提到的词）
#     要么都写、要么都是 -；几处是正整数；必须提到的词按 | 切开，每个词都不空；
#   - 同一个（根，项）登两行判红（没有放开的那一行项写 -，同一个根写两行这种也红）；
#   - 同一个根的几行第 2 段不一样判红（各段写 - 的那一行与放开的行并存就是这一种）；
#   - 放开了的行，第 7 段理由不是 - 且不少于 20 个字，第 8 段出处是仓里真有的文件路径、或「用户 YYYY-MM-DD 弹窗定」加那个路径；history-kept: 出处栏认的写法本身，按格式占位写
#     没放开的行两段都写 -。只判理由在、够长，出处判文件存在。
# 再扫 crates/<crate>/ 下所有 .rs（跳过 target/），先抹掉注释与字符串（行号不变），按根分开（bin_roots_of）：
#   bin 根是 cargo 自动认的二进制目标根 src/main.rs、src/bin/<名>.rs、src/bin/<名>/main.rs；src/main.rs 只管它自己，
#   src/bin/<名>.rs 管它自己与 src/bin/<名>/ 下的文件（它 mod 引的子模块 cargo 就在那里找；那个目录里另有 main.rs 时整个目录归 src/bin/<名>/main.rs），
#   src/bin/<名>/main.rs 管 src/bin/<名>/ 下的整个目录；别的 .rs 都归库根（src/lib.rs）那一份，tests/、benches/ 也在里头；
#   include!("…") 与 #[path = "…"] 点到的文件（按写着它的那份文件所在目录解析）归写着它的那份文件的根，一并扫，不看扩展名；
#   点到的是哪个 crate 都当 .rs 扫的文件不扫第二遍，点到而不在、在仓根之外的跳过。每个根各自判 ①–④：
#   ① 根文件写了 `#![forbid(unsafe_code)]` 或 `#![deny(unsafe_code)]`，登记了的根什么都不写判红；登记表只许 forbid 的根写 deny 判红；
#      登记表里没有的 bin 根照 forbid 判：不写级别也行，写了 deny 判红；登记了的 bin 根在 crate 里不是 bin 根（文件不在、写法不对）判红；
#   ② 总数：`allow` / `expect` / `warn` 里带 `unsafe_code` 的属性（含 cfg_attr 里的）数一遍，根是 forbid 时必须 0 处，
#      根是 deny 时必须等于这个根登记的行数；`unsafe` 关键字（unsafe 块、unsafe fn 的签名、unsafe impl 的项头）同样数，根是 forbid 时 0 处，
#      根是 deny 时等于这个根各行第 5 段之和；不带 unsafe 写的外层属性 #[no_mangle]、#[export_name = …]、#[link_section = …]（含 cfg_attr 里的）
#      unsafe_code 这条 lint 照样报，各算一处 unsafe，位置记在它后面紧挨着的项头上（写成 #[unsafe(no_mangle)] 的由里面的 unsafe 关键字数，不重数）；
#   ③ 位置：每一处放开紧挨着的项是某一行登记的（第 3 段那个文件里的第 4 段那个项；属性与项头之间隔着别的属性与注释照认），不是整文件的内层属性 `#![allow(...)]`；
#      登记了放开的根里，每个 `unsafe` 关键字落在某一行登记的项里：项的范围从项头起（unsafe、pub 这类限定词算进去，所以 unsafe fn 签名与 unsafe impl 项头
#      里的 unsafe 都在里头）到项尾；几行都罩住同一处的（方法在它所在的 impl 块里）归范围最小的那一行；
#      根是 deny 时逐行再判：登记的项上恰好一处放开，项里 `unsafe` 关键字恰好第 5 段那么多处（这两条另随 ② 或 ③ 的弄坏开关一起不判）；
#   ④ 每个 `unsafe` 所在那条语句（或项）的开头一行上方，连着的 `//` 注释里有一行以 `// SAFETY:` 起头（中间隔着独占几行的外层属性照往上找，
#      所以 SAFETY 注释写在 #[allow(unsafe_code, …)] 上方或它与项头之间都认）；往上找读的是只留 // 注释、抹掉字符串与块注释的那一份，
#      按 \n 切行（与 line_of 数行号同一个切法；U+2028 这类不算换行），字符串里顶格写的 // SAFETY: 不算；落在登记项里的每一处，从那一行到语句开头上一行的注释文字
#      要提到那一行第 6 段登记的每个词（不分大小写按字面认；文件描述符、偏移、长度三个词另认同义写法，见 SAFETY_WORD_SYNONYMS）；
#   ⑤ crates/ 下出现登记表里没有库根那一行的 crate 目录判红；登记表里的 crate 目录不在仓里，跳过那几行并在成功那句里点名。
# 成功那句报扫了几个 crate、几个 .rs、登记表几行、放开几处、unsafe 几处、bin 根几个（登记了的几个），再逐个根列级别、放开、unsafe、.rs 与登记的行数
#   （没登记的 bin 根按 crate 合成一项）。
# 射程：tests/、benches/ 各是自己的 crate 根，lib.rs 的 forbid 管不到它们（编译器也不拦），这一格把它们算进库根那一份数；src/main.rs 用 mod 引的 src/ 下的文件也算库根那一份。
# 抹字符串认 `"…"`（连同 b、c 前缀）、原始字符串（r、br、cr 前缀）与字符字面量；research/ 不在射程。
# 按设计不展开宏，只认写出来的 unsafe 关键字（理由写在登记表表头）：宏定义里一个 unsafe 块展开几次都只算定义处一处，SAFETY 注释也只在定义处找。
# 项怎么认（items_of）：impl 块认 `unsafe impl` / `impl` 起头、前一个非空白字符是 { } ; ] 或文件开头的（`-> impl Trait` 这类类型位置上的不算），
#   项头到 { 为止；函数认 `fn <名>`，签名到圆括号、方括号之外的第一个 { 或 ; 为止。trait 定义里的方法照函数名认。
#   同一份文件里几个方法的 <类型>::<方法> 一样（同一个类型的几个固有 impl、或几个 trait impl 里同名的方法）时，这几个都改用全写（写法在 items_of 上方的注释与登记表表头）。
# 语句开头怎么找（statement_start_line）：从 `unsafe` 那一行往上，上一行抹掉注释与字符串之后以 STATEMENT_CONTINUES 列的符号结尾（没收尾），
#   或这一行以 CHAIN_CONTINUES（单个 .）起头（方法链的续行，中间只隔着注释与空行的照跳过），就算同一条语句的续行，一并跳过接着往上；
#   rustfmt 把 `let x =` 与 `unsafe { … }` 折成两行、把 `let x = a()` 与 `.map(|_| unsafe { … })` 折成几行时，写在 `let` 上方的注释照样认。
#   SAFETY 只按语句开头认，不按 unsafe 所在那一行认：写在链中间、续行上方的注释不认。哪几种结尾算没收尾、逗号与 { 为什么不算，写在 STATEMENT_CONTINUES 上方。
#   同一条语句里几处 unsafe（续行接着的第二个 unsafe 块）共用语句开头上方那一段注释，按设计不要求各写一段（理由写在登记表表头）。
#   注释与语句开头之间隔着一条已经收尾的别的语句（以 ; 结尾），那段注释不算这一处的。
# 不判的：理由与 SAFETY 前提写得对不对、`unsafe` 块里的代码对不对（靠 review）。
# 样本（fixtures/code-source-discipline.sh/ 下，各自带登记表与它的出处文件 records/sample-decision.md）：
#   unsafe-allowances-green               core 根 forbid 零处放开、另一个 crate 也 forbid
#   unsafe-allowances-deny-one-green      deny 恰一处
#   unsafe-allowances-several-rows-green  core 登两行：打洞那个函数一处 unsafe、读内存堆那个函数两处，各提到自己那一行登记的词；harness 根 forbid
#   unsafe-allowances-two-allows-red      两处 allow 叠在同一个函数上
#   unsafe-allowances-wrong-function-red  allow 与 unsafe 在别的函数上
#   unsafe-allowances-no-safety-red       没有 SAFETY 注释
#   unsafe-allowances-root-blank-red      core 根什么都不写
#   unsafe-allowances-other-crate-red     harness 的 tests/ 里 allow 加 unsafe，根仍是 forbid
#   unsafe-allowances-reason-empty-red    放开了而理由栏写 -
#   unsafe-allowances-source-missing-red  出处指向仓里没有的文件
#   unsafe-allowances-safety-vague-red    有 SAFETY 注释但没提描述符、偏移、长度
#   unsafe-allowances-place-count-wrong-red        两行的处数登反了（登 2 处的函数里 1 处、登 1 处的函数里 2 处），总数照样对得上
#   unsafe-allowances-one-place-no-safety-red      登 2 处的读内存堆那个函数里，读堆（read_the_memory_heap）那一处 unsafe 上方的注释不以 // SAFETY: 起头
#   unsafe-allowances-one-place-word-missing-red   登 2 处的读内存堆那个函数里，读堆那一处的 SAFETY 注释没提到登记的词 hal
#   unsafe-allowances-allow-count-short-red        登了两行，只有一个函数上有 allow
#   unsafe-allowances-allows-on-one-function-red   登了两行，两处 allow 叠在同一个函数上、另一个没有，总数照样对得上
#   unsafe-allowances-same-function-two-rows-red   同一个函数登了两行
#   unsafe-allowances-crate-rows-disagree-red      core 一行各段写 -（只许 forbid）、一行放开（forbid,deny）
#   unsafe-allowances-statement-continued-green    rustfmt 折行：// SAFETY: 写在 `let return_code =` 上方，`unsafe { … }` 在下一行；只有 core 一个 crate
#   unsafe-allowances-statement-finished-between-red  同一段 // SAFETY: 与 `let return_code =` 之间隔着一条以 ; 收尾的 `let checked_length = length;`
#   下面七份的库根 crates/singlefs-verification/src/lib.rs 写 forbid；前三份的 bin 根是 src/bin/e900_thread_clock.rs（读线程钟），unsafe-impl 那三份是 src/bin/e900_counting_allocator.rs：
#   unsafe-allowances-bin-root-green               bin 根写 deny、单独登一行（一个函数一处 unsafe）；另一个 bin 根 e901_plain_report.rs 没登记、不写级别、零处
#   unsafe-allowances-bin-root-unregistered-red    bin 根没登记、不写级别，写了一处带 SAFETY 注释的 unsafe
#   unsafe-allowances-bin-root-forbid-red          表里给 bin 根只登 forbid 一行，bin 根却写 deny、放开一个函数
#   unsafe-allowances-bin-root-not-found-red       表里登记的 bin 根 e999_removed_allocator.rs 不在（crate 里只有没登记的 e901_plain_report.rs）
#   unsafe-allowances-unsafe-impl-green            bin 根里计数分配器：unsafe impl 一行（1 处）、两个 unsafe fn 方法各一行（签名加函数体各 1 处）；
#                                                  SAFETY 注释写在放开属性上方，有一处放开属性折成四行
#   unsafe-allowances-unsafe-impl-no-safety-red    同上，unsafe impl 上方的注释不以 // SAFETY: 起头
#   unsafe-allowances-unsafe-fn-no-safety-red      同上，alloc 签名上方的注释不以 // SAFETY: 起头（函数体里那一处照写着）
#   unsafe-allowances-same-method-two-types-green  core 库根 deny：两个类型各实现一次 GlobalAlloc、各有 alloc 与 dealloc，按 <类型>::<方法> 各登一行；
#                                                  SAFETY 注释写在放开属性与项头之间
#   下面十五份是 m4-gpugc-unsafe-gate-r1 判决 Q1 那八处判法与两样按设计的钉样，库根 forbid、bin 根都是 src/bin/e900_thread_clock.rs：
#   unsafe-allowances-line-separator-in-a-comment-red          注释里夹一个 U+2028，SAFETY 与 unsafe 之间隔着一条收尾的语句（按 \n 切行才判红）
#   unsafe-allowances-same-text-without-the-line-separator-red 同一份源码，U+2028 换成空格，照样红
#   unsafe-allowances-safety-line-inside-a-string-red          // SAFETY: 那一行在字符串字面量里，不是注释
#   unsafe-allowances-raw-c-string-hides-unsafe-red            两个 cr"\" 中间夹一处 unsafe；bin 根没登记，照 forbid 红
#   unsafe-allowances-include-of-a-non-rs-file-red             bin 根 include! 一份 .inc，unsafe 在 .inc 里；bin 根没登记
#   unsafe-allowances-path-attribute-to-a-non-rs-file-red      bin 根 #[path] 引一份 .inc 当模块，unsafe 在 .inc 里；bin 根没登记
#   unsafe-allowances-no-mangle-without-the-keyword-red        不带 unsafe 的 #[no_mangle]；bin 根没登记
#   unsafe-allowances-no-mangle-registered-green               bin 根 deny，登记放开导出符号的函数（1 处），SAFETY 注释写在放开属性上方
#   unsafe-allowances-bin-helper-directory-registered-green    放开写在 src/bin/e900_thread_clock/ 下的子模块里，登记在 bin 根名下
#   unsafe-allowances-bin-helper-directory-unregistered-red    同样的子模块里写 unsafe，bin 根没登记，红在 bin 根名下
#   unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green  unsafe 在 .map(…) 那一行，SAFETY 写在 let 上方
#   unsafe-allowances-same-method-name-in-two-inherent-impls-green         impl Reader<u8> 与 impl Reader<u16> 各有 read，登 Reader<u16>::read
#   unsafe-allowances-same-method-name-registered-without-type-arguments-red  同样两个 read，登记仍写 Reader::read，红在放开那一处、点名全写
#   unsafe-allowances-macro-expands-three-unsafe-for-one-keyword-green     按设计：宏定义里一处 unsafe 调三次，登 1 处
#   unsafe-allowances-one-comment-covers-the-second-unsafe-of-a-chain-green  按设计：一条语句两处 unsafe 共用语句开头上方的一段 SAFETY，登 2 处
# 弄坏开关 GATE_UNSAFE_ALLOWANCES_BREAK 各项在文件头。
cell_unsafe_allowances() {
  [[ -d crates ]] || { echo "  ! 没有 crates/ 目录，这一格无对象可判"; exit 77; }

  python3 - <<'PY'
import os, re, sys

BREAK = os.environ.get("GATE_UNSAFE_ALLOWANCES_BREAK", "")
REGISTRY_PATH = ".claude/gate.d/unsafe-allowances.tsv"
REGISTRY_FIELD_COUNT = 8
REASON_MINIMUM = 20
PLACE_COUNT = re.compile(r"[1-9][0-9]*")
# 登记表第 6 段里这三个词另认同义写法：打洞那一行（singlefs-core）登的是这三个词，它的 SAFETY 注释写的是「描述符」「offset」这类说法；别的词按字面认。
SAFETY_WORD_SYNONYMS = {"文件描述符": re.compile(r"描述符|file descriptor|\bfd\b", re.I),
                        "偏移": re.compile(r"偏移|offset", re.I),
                        "长度": re.compile(r"长度|length|\blen\b", re.I)}
# 弄坏开关 registered-words-ignored 下每一处都照这三个词判，不读登记表第 6 段。
HOLE_PUNCHING_WORDS = ("文件描述符", "偏移", "长度")
# 一行抹掉注释与字符串之后以这几种符号结尾，就是语句没收尾、下一行是它的续行：=（连同 +=、== 这类以 = 结尾的）、( 与 [、
# 运算符 + - * / % & | ^ ! < >（=> 与 -> 的末字符也在里头）。逗号与 { 不算：match 的一个分支、结构体字面量的一个字段、
# 函数头或块的头一行以它们结尾时，下一行是另一处的代码，跳过去会把写给别处的 SAFETY 注释认成这一处的。
STATEMENT_CONTINUES = re.compile(r"[=(\[+\-*/%&|^!<>]\s*$")
# 一行抹掉注释与字符串之后以单个 . 起头（不是 .. 区间），是方法链接着上面那一段：rustfmt 把 `let x = a()` 与 `.map(|_| unsafe { … })` 折成几行时，
# unsafe 所在那一行不是语句开头，写在 `let` 上方的注释照样认。SAFETY 只按语句开头认，写在链中间那几行上方的注释不认。
CHAIN_CONTINUES = re.compile(r"\s*\.(?!\.)")
problems = []


def reject(summary, details, howto):
    problems.append((summary, details, howto))


def finish():
    for summary, details, howto in problems:
        print(f"  ✗ {summary}")                      # gate-lint:summary
        for detail in details:
            print(f"     {detail}")                  # gate-lint:detail
        print(f"     → 怎么办：{howto}")
    if problems:
        sys.exit(1)


def blank(piece):
    return re.sub(r"[^\n]", " ", piece)


RAW_STRING_OPEN = re.compile(r'r(#*)"')
# 原始字符串 r"…" 前面可以紧挨着的一个前缀字母：b（字节串 br"…"）与 c（C 字符串 cr"…"）。弄坏开关 c-string-prefix-unknown 下只认 b：
# cr"\" 里的 \ 被当成转义，抹字符串失步、吞掉后面的代码。
RAW_STRING_PREFIXES = "b" if BREAK == "c-string-prefix-unknown" else "bc"


def strip_comments_and_strings(text, keep_line_comments=False):
    """注释整段抹成空格，字符串只抹里面（引号留着），换行都留着，行号与每个字符的位置都不变。
    keep_line_comments 为真时 // 注释原样留着（块注释照抹）：找 SAFETY 注释读这一份，字符串里顶格写的 // SAFETY: 不当注释。"""
    out, index, length = [], 0, len(text)
    while index < length:
        char = text[index]
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            out.append(text[index:end] if keep_line_comments else blank(text[index:end])); index = end
        elif text.startswith("/*", index):
            depth, end = 1, index + 2
            while end < length and depth:
                if text.startswith("/*", end):
                    depth += 1; end += 2
                elif text.startswith("*/", end):
                    depth -= 1; end += 2
                else:
                    end += 1
            out.append(blank(text[index:end])); index = end
        elif char == "r" and RAW_STRING_OPEN.match(text, index) and (
            index == 0 or not (text[index - 1].isalnum() or text[index - 1] == "_")
            or (text[index - 1] in RAW_STRING_PREFIXES and (index == 1 or not (text[index - 2].isalnum() or text[index - 2] == "_")))
        ):
            opening = RAW_STRING_OPEN.match(text, index)
            closing = '"' + opening.group(1)
            close_at = text.find(closing, opening.end())
            if close_at < 0:
                out.append(blank(text[index:])); index = length
            else:
                out.append(text[index:opening.end()] + blank(text[opening.end():close_at]) + closing)
                index = close_at + len(closing)
        elif char == '"':
            end = index + 1
            while end < length and text[end] != '"':
                end += 2 if text[end] == "\\" else 1
            end = min(end, length)
            out.append('"' + blank(text[index + 1:end]) + ('"' if end < length else ""))
            index = min(end + 1, length)
        elif char == "'":
            if index + 1 < length and text[index + 1] == "\\":
                end = text.find("'", index + 3)
                end = length if end < 0 else end + 1
                out.append(blank(text[index:end])); index = end
            elif index + 2 < length and text[index + 2] == "'" and text[index + 1] != "'":
                out.append("'" + blank(text[index + 1]) + "'"); index += 3
            else:
                out.append(char); index += 1
        else:
            out.append(char); index += 1
    return "".join(out)


def line_of(text, position):
    return text.count("\n", 0, position) + 1


ATTRIBUTE_OPEN = re.compile(r"#\s*(!?)\s*\[")


def attributes(cleaned):
    """逐个 #[...] / #![...]：(起点, 终点, 是不是内层属性, 里面的文本)。"""
    found, index = [], 0
    while True:
        opening = ATTRIBUTE_OPEN.search(cleaned, index)
        if not opening:
            return found
        depth, end = 1, opening.end()
        while end < len(cleaned) and depth:
            depth += {"[": 1, "]": -1}.get(cleaned[end], 0)
            end += 1
        found.append((opening.start(), end, opening.group(1) == "!", cleaned[opening.end():end - 1]))
        index = end


RELAX = re.compile(r"\b(allow|expect|warn)\s*\([^)]*\bunsafe_code\b")
LEVEL = re.compile(r"\b(forbid|deny)\s*\([^)]*\bunsafe_code\b")
# 项头：函数的限定词连 fn 与函数名；impl 块的 unsafe impl / impl。属性后面跟着的项从 ITEM_AFTER_ATTRIBUTES 收尾处起。
FUNCTION_ITEM = re.compile(
    r"(?:\bpub(?:\s*\([^)]*\))?\s+)?(?:\b(?:const|async|unsafe|safe|default)\s+|\bextern\s+\"[^\"]*\"\s+)*\bfn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
IMPL_ITEM = re.compile(r"\b(?:unsafe\s+)?impl\b")
ITEM_AFTER_ATTRIBUTES = re.compile(r"\s*(?:#\s*\[[^\]]*\]\s*)*")
UNSAFE_WORD = re.compile(r"\bunsafe\b")
# 2021 版不带 unsafe 写的这三种外层属性，unsafe_code 这条 lint 照样报（2024 版要写成 #[unsafe(no_mangle)]，那里的 unsafe 由 UNSAFE_WORD 数）：
# 各算项里的一处 unsafe，位置记在它后面紧挨着的项头上。
EXPORTED_SYMBOL_ATTRIBUTE = re.compile(r"\b(?:no_mangle|export_name|link_section)\b")
# include!("…") 与 #[path = "…"]：在抹过注释与字符串的那一份里认，路径从原文同一位置读（字符串里只抹内容、引号留着）。
FILE_REFERENCE = re.compile(r'(?:\binclude!\s*\(\s*|#\s*\[\s*path\s*=\s*)"')
# cargo 自动认的二进制目标根，相对 crate 目录。
BIN_ROOT_FILE = re.compile(r"src/main\.rs|src/bin/[^/]+\.rs")
BIN_ROOT_DIRECTORY_MAIN = re.compile(r"src/bin/[^/]+/main\.rs")


def bin_roots_of(relative_paths):
    """crate 目录下（相对路径）的 bin 根：{根文件: 这个根管的文件}。src/main.rs 只管它自己；src/bin/<名>.rs 管它自己与 src/bin/<名>/ 下的全部
    （它的 mod 照 cargo 的规矩从那个目录找；同一个目录里另有 src/bin/<名>/main.rs 时那个目录归后者）；src/bin/<名>/main.rs 管 src/bin/<名>/ 下的全部。
    弄坏开关 bin-roots-in-library 下一个都不认（全算进库根）；bin-helper-directory-in-library 下 src/bin/<名>.rs 只管它自己。"""
    if BREAK == "bin-roots-in-library":
        return {}
    roots = {}
    for relative in relative_paths:
        if BIN_ROOT_FILE.fullmatch(relative):
            members = [relative]
            helper_directory = relative[:-len(".rs")] + "/"
            if (relative.startswith("src/bin/") and BREAK != "bin-helper-directory-in-library"
                    and helper_directory + "main.rs" not in relative_paths):
                members += [other for other in relative_paths if other.startswith(helper_directory)]
            roots[relative] = members
        elif BIN_ROOT_DIRECTORY_MAIN.fullmatch(relative):
            directory = relative[:-len("main.rs")]
            roots[relative] = [other for other in relative_paths if other.startswith(directory)]
    return roots


def closing_brace(cleaned, opening):
    """从 opening 处的 { 起，配对的 } 之后那个位置。"""
    depth, index = 0, opening
    while index < len(cleaned):
        depth += {"{": 1, "}": -1}.get(cleaned[index], 0)
        index += 1
        if depth == 0:
            break
    return index


def signature_end(cleaned, index):
    """从 index 往后，圆括号、方括号之外的第一个 { 或 ;：交回（位置, 那个字符）；到文末都没有交回（文末, ""）。"""
    depth = 0
    while index < len(cleaned):
        char = cleaned[index]
        if char in "([":
            depth += 1
        elif char in ")]":
            depth -= 1
        elif char in "{;" and depth <= 0:
            return index, char
        index += 1
    return index, ""


def skip_generic_parameters(text):
    """impl 后面紧跟的 <…>（-> 里的 > 不算）去掉，交回余下的。"""
    stripped = text.lstrip()
    if not stripped.startswith("<"):
        return stripped
    depth = 0
    for index, char in enumerate(stripped):
        if char == "<":
            depth += 1
        elif char == ">" and stripped[index - 1] != "-":
            depth -= 1
            if depth == 0:
                return stripped[index + 1:]
    return ""


TYPE_LEADING_QUALIFIERS = re.compile(r"^(?:\s+|[&!]|'[A-Za-z_][A-Za-z0-9_]*|\bmut\b|\bdyn\b)*")
INLINE_MODULE = re.compile(r"\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{")


def path_name(text):
    """类型或 trait 的写法里认名字：去掉打头的 &、!、生命周期、mut、dyn，取 < 之前那段路径的最后一节。"""
    text = TYPE_LEADING_QUALIFIERS.sub("", text)
    return "".join(text.split("<", 1)[0].split("::")[-1].split())


def written_type(text):
    """类型或 trait 连同 <…> 里的实参：去掉打头的 &、!、生命周期、mut、dyn 与全部空白，< 之前那段路径只留最后一节（实参原样留着）。"""
    compact = "".join(TYPE_LEADING_QUALIFIERS.sub("", text).split())
    head, angle, arguments = compact.partition("<")
    return head.split("::")[-1] + angle + arguments


def items_of(cleaned):
    """一份抹过注释与字符串的文件里的 impl 块与函数，每项 {"key", "start", "body", "end"}：key 是登记表第 4 段的写法，start 是项头起点，
    body 是 { 的位置（没有函数体的声明是 None），end 是项尾之后。弄坏开关 methods-keyed-by-name 下方法也只按方法名认。
    同一份文件里几个方法的 <类型>::<方法> 一样（同一个类型的几个 impl 块里同名的方法）时，这几个都改用全写：
    <内联模块路径::><类型连同实参>::<方法>，trait impl 里的写成 <<内联模块路径::><类型连同实参> as <Trait 连同实参>>::<方法>；
    弄坏开关 colliding-methods-keyed-alike 下不改（撞了的几个同一个键，登记表那一行只认得到最靠前的那个）。"""
    modules = []
    for match in INLINE_MODULE.finditer(cleaned):
        modules.append((match.group(1), match.end() - 1, closing_brace(cleaned, match.end() - 1)))

    def module_path_at(position):
        return "".join(f"{name}::" for name, opening, closing in modules if opening < position < closing)

    impls = []
    for match in IMPL_ITEM.finditer(cleaned):
        before = match.start() - 1
        while before >= 0 and cleaned[before].isspace():
            before -= 1
        if before >= 0 and cleaned[before] not in "{};]":
            continue
        body, char = signature_end(cleaned, match.end())
        if char != "{":
            continue
        header = re.split(r"\bwhere\b", skip_generic_parameters(cleaned[match.end():body]), maxsplit=1)[0]
        parts = re.split(r"\bfor\b(?!\s*<)", header, maxsplit=1)
        type_name = path_name(parts[-1])
        key = f"impl {path_name(parts[0])} for {type_name}" if len(parts) == 2 else f"impl {type_name}"
        full_type = module_path_at(match.start()) + written_type(parts[-1])
        full_owner = f"<{full_type} as {written_type(parts[0])}>" if len(parts) == 2 else full_type
        impls.append({"key": key, "type": type_name, "full_owner": full_owner,
                      "start": match.start(), "body": body, "end": closing_brace(cleaned, body)})
    functions = []
    for match in FUNCTION_ITEM.finditer(cleaned):
        boundary, char = signature_end(cleaned, match.end())
        body = boundary if char == "{" else None
        end = closing_brace(cleaned, boundary) if char == "{" else boundary + 1
        functions.append({"name": match.group(1), "start": match.start(), "body": body, "end": end})
    for function in functions:
        enclosing = [item for item in impls + functions
                     if item["body"] is not None and item["body"] < function["start"] < item["end"]]
        innermost = min(enclosing, key=lambda item: item["end"] - item["body"], default=None)
        is_method = innermost is not None and "type" in innermost and BREAK != "methods-keyed-by-name"
        function["key"] = f"{innermost['type']}::{function['name']}" if is_method else function["name"]
        if is_method:
            function["full_key"] = f"{innermost['full_owner']}::{function['name']}"
    if BREAK != "colliding-methods-keyed-alike":
        method_key_counts = {}
        for function in functions:
            if "full_key" in function:
                method_key_counts[function["key"]] = method_key_counts.get(function["key"], 0) + 1
        for function in functions:
            if "full_key" in function and method_key_counts[function["key"]] > 1:
                function["key"] = function["full_key"]
    return impls + functions


def item_span(item):
    """登记的一项管的范围 [起, 止)：从项头起到项尾。弄坏开关 signature-outside-body 下从 { 起（签名与 impl 项头落在外面）。"""
    if BREAK == "signature-outside-body":
        return None if item["body"] is None else (item["body"], item["end"])
    return item["start"], item["end"]


def attribute_lines(cleaned):
    """独占几行的外层属性（同一行上属性前后没有别的代码）占的行号（从 1 数）：找 SAFETY 注释时隔着它们往上找。
    弄坏开关 attributes-not-skipped 下一行都不算（碰到属性就停）。"""
    lines = set()
    if BREAK == "attributes-not-skipped":
        return lines
    for start, end, inner, body in attributes(cleaned):
        line_start = cleaned.rfind("\n", 0, start) + 1
        line_end = cleaned.find("\n", end)
        line_end = len(cleaned) if line_end < 0 else line_end
        if not inner and not cleaned[line_start:start].strip() and not cleaned[end:line_end].strip():
            lines.update(range(line_of(cleaned, start), line_of(cleaned, end - 1) + 1))
    return lines


def statement_start_line(cleaned_lines, unsafe_line):
    """unsafe 所在那条语句的开头一行（从 0 数）：从 unsafe 那一行往上，上一行抹掉注释与字符串之后以 STATEMENT_CONTINUES 结尾的，
    或这一行以 CHAIN_CONTINUES 起头的（方法链接着上面那一段；中间只有注释与空行的照跳过），算同一条语句的续行，一并跳过接着往上。
    弄坏开关 continuation-lines-ignored 下两种都不跳；chain-lines-not-continued 下不认以 . 起头的那一种。"""
    start = unsafe_line - 1
    while BREAK != "continuation-lines-ignored" and start > 0:
        if STATEMENT_CONTINUES.search(cleaned_lines[start - 1]):
            start -= 1
        elif BREAK != "chain-lines-not-continued" and CHAIN_CONTINUES.match(cleaned_lines[start]):
            above = start - 1
            while above > 0 and not cleaned_lines[above].strip():
                above -= 1
            if not cleaned_lines[above].strip():
                break
            start = above
        else:
            break
    return start


def safety_comment(comment_lines_of_the_file, cleaned_lines, unsafe_line, skipped_lines):
    """unsafe 所在那条语句（或项）的开头一行上方，连着的 // 注释里有一行以 // SAFETY: 起头：返回从那一行往下到语句开头之间的注释文字；没有返回 None。
    comment_lines_of_the_file 是只留 // 注释、抹掉字符串与块注释的那一份按 \\n 切的行（strip_comments_and_strings 带 keep_line_comments），
    与 cleaned_lines 行号一一对上；字符串里顶格写的 // SAFETY: 在这一份里已经抹掉，不当注释。
    往上找注释时 skipped_lines（attribute_lines 给的属性行）照跳过，不算注释也不打断。"""
    start = statement_start_line(cleaned_lines, unsafe_line)
    comment_lines = []
    number = start - 1
    while number >= 0:
        stripped = comment_lines_of_the_file[number].lstrip()
        if stripped.startswith("//"):
            comment_lines.append(comment_lines_of_the_file[number])
            if stripped.startswith("// SAFETY:"):
                return "\n".join(reversed(comment_lines))
        elif number + 1 not in skipped_lines:
            break
        number -= 1
    return None


def safety_word_pattern(word):
    """登记表第 6 段的一个词怎么认：SAFETY_WORD_SYNONYMS 里的三个词认同义写法，别的词不分大小写按字面认。"""
    return SAFETY_WORD_SYNONYMS.get(word) or re.compile(re.escape(word), re.I)


def registered_row_holding(allowance_rows, spans, path, position):
    """unsafe 关键字（path 文件里的 position 处）落在哪一行登记的项里：几行都罩住它的取范围最小的那一行（方法先于它所在的 impl 块）；
    不在任何一行里返回 None。弄坏开关 outermost-holder 下取表里靠前的那一行。"""
    covering = []
    for row in allowance_rows:
        span = spans.get((row["file"], row["function"]))
        if path == row["file"] and span is not None and span[0] <= position < span[1]:
            covering.append((span[1] - span[0], row))
    if not covering:
        return None
    if BREAK == "outermost-holder":
        return covering[0][1]
    return min(covering, key=lambda pair: pair[0])[1]


# ── 登记表 ──
registry = {}           # 根（第 1 段：库根写 crate 目录名，bin 根写 <crate 目录名>/<bin 根>）→ {项（没有放开的那一行是 -）: 登记的那一行}
row_of_function = {}    # （根，项）→ 表里的行号
if not os.path.isfile(REGISTRY_PATH):
    reject(f"没有登记表 {REGISTRY_PATH}", [], f"照 {REGISTRY_PATH} 的表头写法建这张表：crates/ 下每个 crate 至少一行；表是这一格判的依据，不能没有。")
    finish()
for number, raw in enumerate(open(REGISTRY_PATH, encoding="utf-8").read().splitlines(), 1):
    if not raw.strip() or raw.lstrip().startswith("#"):
        continue
    fields = raw.split("\t")
    if len(fields) != REGISTRY_FIELD_COUNT or not all(field.strip() for field in fields):
        reject(f"{REGISTRY_PATH}:{number} 不是八段（根、根允许的级别、放开的文件、放开的项、项里 unsafe 几处、SAFETY 注释必须提到的词、理由、出处），段间用制表符，每段都有内容",
               [raw], "照表头写法改这一行；没有放开的行，第 3 到第 8 段都写 -。")
        continue
    root_name, levels, allowed_file, allowed_function, place_count, words, reason, source = (field.strip() for field in fields)
    level_list = levels.split(",")
    word_list = [word.strip() for word in words.split("|")]
    row_allows = allowed_file != "-"
    if (any(level not in ("forbid", "deny") for level in level_list)
            or ("deny" in level_list) != row_allows
            or any((field == "-") == row_allows for field in (allowed_function, place_count, words))
            or (row_allows and (not PLACE_COUNT.fullmatch(place_count) or not all(word_list)))):
        reject(f"{REGISTRY_PATH}:{number} 这一行自相矛盾", [raw],
               "级别只写 forbid 或 forbid,deny；写了 deny 才有放开，第 3 到第 6 段（文件、项、项里 unsafe 几处、SAFETY 注释必须提到的词）都写上，"
               "没有 deny 这四段都写 -；几处写正整数，必须提到的词用 | 分隔、每个词都不空。")
        continue
    function_key = (root_name, allowed_function)
    if function_key in row_of_function and BREAK != "duplicate-rows-allowed":
        subject = "没有放开的那一行" if allowed_function == "-" else f"函数 {allowed_function} "
        reject(f"{REGISTRY_PATH}:{number} {root_name} 的{subject}登了两行（另一行在第 {row_of_function[function_key]} 行）", [raw],
               "一个函数只登一行：函数里的几处 unsafe 合写在那一行的第 5 段（几处）与第 6 段（每一处都要提到的词）；没有放开的 crate 只写一行。"
               "方法写 <类型>::<方法>、impl 块写 impl <Trait> for <类型>，两个类型的同名方法是两个键。")
        continue
    if not row_allows:
        if reason != "-" or source != "-":
            reject(f"{REGISTRY_PATH}:{number} {root_name} 没有放开，理由与出处两段却写了内容", [raw],
                   "没有放开的行，理由与出处都写 -。")
            continue
    else:
        if BREAK != "reason-ignored" and (reason == "-" or len(reason) < REASON_MINIMUM):
            reject(f"{REGISTRY_PATH}:{number} {root_name} 放开了 unsafe，理由栏{'是 -' if reason == '-' else f'只有 {len(reason)} 个字'}（不少于 {REASON_MINIMUM} 个字）", [raw],
                   "理由栏写为什么非放开不可、不放开的替代为什么不行；理由写不出来的放开不登记。")
            continue
        if BREAK != "source-ignored":
            source_words = source.split()
            source_path = source_words[-1]
            source_prefix = " ".join(source_words[:-1])
            if source == "-":
                source_problem = "出处栏是 -，放开了的行要写谁哪天定的"
            elif source_prefix and not re.fullmatch(r"用户 \d{4}-\d{2}-\d{2} 弹窗定", source_prefix):
                source_problem = f"出处栏「{source}」的写法不对：只写仓里的路径，或「用户 YYYY-MM-DD 弹窗定 <路径>」"
            elif os.path.isabs(source_path) or ".." in source_path.split("/") or not os.path.isfile(source_path):
                source_problem = f"出处栏指向的 {source_path} 不是仓里的文件（要写仓内的相对路径、文件存在）"
            else:
                source_problem = None
            if source_problem:
                reject(f"{REGISTRY_PATH}:{number} {root_name} 的出处不成立：{source_problem}", [raw],
                       "出处写记录这次定案的仓内文件路径，或「用户 YYYY-MM-DD 弹窗定」加那份记录的路径；记录文件要真在仓里。")
                continue
    row_of_function[function_key] = number
    registry.setdefault(root_name, {})[allowed_function] = {
        "number": number, "levels": level_list, "file": allowed_file, "function": allowed_function,
        "count": int(place_count) if row_allows else 0, "words": word_list if row_allows else []}
if BREAK != "crate-levels-disagreement-allowed":
    for root_name, rows in registry.items():
        if len({",".join(row["levels"]) for row in rows.values()}) > 1:
            numbers = "、".join(str(row["number"]) for row in rows.values())
            spelled = "、".join(f"第 {row['number']} 行 {','.join(row['levels'])}" for row in rows.values())
            reject(f"{REGISTRY_PATH}:{numbers} {root_name} 的几行第 2 段（根允许的级别）不一样：{spelled}", [],
                   "同一个 crate 的几行第 2 段写成一样：有放开的 crate 每一行都写 forbid,deny，不再留各段写 - 的那一行；没有放开的 crate 只写各段写 - 的那一行。"
                   "bin 根与库根各是一个根，各写各的一组。")
finish()


def crate_of(root_name):
    """登记表第 1 段属于哪个 crate 目录：库根就是它自己，bin 根取第一个 / 之前。"""
    return root_name.split("/", 1)[0]

crate_names = sorted(entry for entry in os.listdir("crates") if os.path.isdir(os.path.join("crates", entry)))
# scanned 一个根一项：（第 1 段的写法, 是不是没登记的 bin 根, 所属 crate, .rs 几个, 根的级别, 放开几处, unsafe 几处, 登记几行）
file_count, crate_count, scanned = 0, 0, []


def judge_root(label, crate_name, root_path, files, rows, registered):
    """判一个根（库根或 bin 根）的 ①–④：label 是登记表第 1 段的写法，files 是这个根管的 {路径: (原文, 抹过注释与字符串的)}，
    rows 是这个根登记的行；没登记的 bin 根（registered 为假）rows 是空的，级别照 forbid、不写也行。"""
    allowance_rows = [row for row in rows.values() if row["file"] != "-"]
    levels = [level for level in ("forbid", "deny") if any(level in row["levels"] for row in rows.values())] if registered else ["forbid"]
    has_allowance = bool(allowance_rows)
    relaxes, unsafes, level, items_by_path, skipped_by_path, comment_lines_by_path = [], [], None, {}, {}, {}
    for path, (original, cleaned) in files.items():
        places = []
        for start, end, inner, body in attributes(cleaned):
            if (not inner and BREAK != "exported-symbol-attributes-unseen"
                    and EXPORTED_SYMBOL_ATTRIBUTE.search(body) and not UNSAFE_WORD.search(body)):
                places.append((path, line_of(cleaned, start), ITEM_AFTER_ATTRIBUTES.match(cleaned, end).end()))
            if RELAX.search(body):
                if re.match(r"\s*(?:pub(?:\s*\([^)]*\))?\s+)?assume_specification\b", cleaned[end:]):
                    continue
                if path not in items_by_path:
                    items_by_path[path] = items_of(cleaned)
                following_start = ITEM_AFTER_ATTRIBUTES.match(cleaned, end).end()
                following = next((item for item in items_by_path[path] if item["start"] == following_start), None)
                relaxes.append((path, line_of(cleaned, start), None if inner else (following["key"] if following else None)))
            if path == root_path and inner and LEVEL.search(body):
                found = LEVEL.search(body).group(1)
                level = "forbid" if (found == "forbid" or level == "forbid") else "deny"
        for match in UNSAFE_WORD.finditer(cleaned):
            places.append((path, line_of(cleaned, match.start()), match.start()))
        unsafes.extend(sorted(places, key=lambda place: (place[1], place[2])))
    scanned.append((label, not registered, crate_name, len(files), level, len(relaxes), len(unsafes), len(rows)))

    if root_path not in files:
        reject(f"{label} 没有 crate 根 {root_path}", [], "登记表与判法都按 src/lib.rs 当库根；crate 改了布局就先改这一格。")
        return
    if BREAK != "root-level-ignored":
        if level is None and registered:
            reject(f"{root_path} 没写 #![forbid(unsafe_code)] 也没写 #![deny(unsafe_code)]", [],
                   "根上写 #![forbid(unsafe_code)]；登记了放开的 crate 写 #![deny(unsafe_code)]。")
        elif level is not None and level not in levels:
            unregistered_note = "" if registered else f"这个 bin 根在登记表里没有行，照 forbid 判；要在这里放开，先给它登记（第 1 段写 {label}）。"
            reject(f"{root_path} 的级别是 {level}，登记表只许 {'、'.join(levels)}", [],
                   f"根改回 forbid；要放开 unsafe 先改 {REGISTRY_PATH} 的行并走三方。{unregistered_note}")
    if level is None and not registered:
        level = "forbid"
    if level is None or level not in levels:
        return
    level_text = level if registered else "forbid（没登记的 bin 根照 forbid 判）"
    allowance_in_force = level == "deny"
    if allowance_in_force:
        expected_relaxes = 1 if BREAK == "allow-count-single" else len(allowance_rows)
        expected_unsafes = sum(row["count"] for row in allowance_rows)
        relax_basis = f"（登记表 {len(allowance_rows)} 行，一行一处）"
        unsafe_basis = f"（登记表 {len(allowance_rows)} 行第 5 段之和）"
    else:
        expected_relaxes, expected_unsafes, relax_basis, unsafe_basis = 0, 0, "", ""
    spans = {}
    for row in allowance_rows:
        if row["file"] in files:
            if row["file"] not in items_by_path:
                items_by_path[row["file"]] = items_of(files[row["file"]][1])
            item = next((item for item in items_by_path[row["file"]] if item["key"] == row["function"]), None)
            spans[(row["file"], row["function"])] = None if item is None else item_span(item)
    holders = [registered_row_holding(allowance_rows, spans, path, position) for path, line, position in unsafes]
    registered_items = "、".join(f"{row['file']} 的 {row['function']}" for row in allowance_rows)
    if BREAK != "count-ignored":
        if len(relaxes) != expected_relaxes:
            reject(f"{label} 放开 unsafe_code 的地方 {len(relaxes)} 处，根是 {level_text}、应当 {expected_relaxes} 处{relax_basis}",
                   [f"{path}:{line}" for path, line, function in relaxes] or ["（没有）"],
                   f"根写 forbid 就一处都不许放开；根写 deny 就登记的每一行恰好放开一处。新增一处放开要改 {REGISTRY_PATH} 的登记并走三方，不是加一个 allow。")
        if len(unsafes) != expected_unsafes:
            reject(f"{label} 的 unsafe 关键字 {len(unsafes)} 处，根是 {level_text}、应当 {expected_unsafes} 处{unsafe_basis}",
                   [f"{path}:{line}" for path, line, position in unsafes] or ["（没有）"],
                   f"tests/、benches/ 的文件算进库根那一份数，src/bin/ 下的 bin 根各自判、没登记的照 forbid；不要在那里写 unsafe"
                   f"（不带 unsafe 的 #[no_mangle]、#[export_name]、#[link_section] 各算一处，include! 与 #[path] 点到的文件算进写着它的那个根）。"
                   f"要新增走 {REGISTRY_PATH} 的登记（bin 根第 1 段写 <crate 目录名>/<bin 根>）与三方。")
    if BREAK != "placement-ignored":
        registered_pairs = {(row["file"], row["function"]) for row in allowance_rows}
        for path, line, function in relaxes:
            if function is None:
                reject(f"{path}:{line} 是整文件的内层属性或后面没有函数，不是登记的那几处",
                       [], f"放开只许是 #[allow(unsafe_code)] 紧挨着登记的函数、方法或 impl 块（{registered_items or '这个根没有登记放开'}）；要换地方改 {REGISTRY_PATH} 的登记并走三方。")
            elif (path, function) not in registered_pairs:
                registered_text = f"登记的是 {registered_items}" if has_allowance else f"{label} 在登记表里没有放开"
                reject(f"{path}:{line} 放开 unsafe_code 的函数是 {function}，{registered_text}",
                       [], f"放开只许在登记的那几处；要新增或换地方改 {REGISTRY_PATH} 的登记并走三方（方法写 <类型>::<方法>，impl 块写 impl <Trait> for <类型>）。")
        if has_allowance:
            for (path, line, position), holder in zip(unsafes, holders):
                if holder is None:
                    reject(f"{path}:{line} 的 unsafe 不在登记的 {registered_items} 函数体里", [],
                           f"unsafe 只许写在登记的那几项里（项从项头算起，unsafe fn 签名与 unsafe impl 项头都在里头）；要换地方改 {REGISTRY_PATH} 的登记并走三方。")
    if allowance_in_force and BREAK not in ("count-ignored", "placement-ignored"):
        for row in allowance_rows:
            if BREAK != "per-row-allows-ignored":
                beside = [f"{path}:{line}" for path, line, function in relaxes if (path, function) == (row["file"], row["function"])]
                if len(beside) != 1:
                    reject(f"{REGISTRY_PATH}:{row['number']} 登记的 {row['file']} 的 {row['function']} 上放开 unsafe_code 的属性 {len(beside)} 处，应当恰好 1 处",
                           beside or ["（没有）"],
                           "登记的每个函数上恰好写一处 #[allow(unsafe_code, reason = \"…\")]：多出的删掉，缺的补上（reason 照登记表这一行第 7 段）。")
            if BREAK != "per-function-count-ignored":
                inside = [f"{path}:{line}" for (path, line, position), holder in zip(unsafes, holders) if holder is row]
                if len(inside) != row["count"]:
                    reject(f"{REGISTRY_PATH}:{row['number']} 登记的 {row['file']} 的 {row['function']} 里 unsafe 关键字 {len(inside)} 处，登记表这一行第 5 段写的是 {row['count']} 处",
                           inside or ["（没有）"],
                           f"函数里多出的 unsafe 删掉；确要加减，改 {REGISTRY_PATH} 这一行的第 5 段并走三方。")
    judged_rows = set()
    for (path, line, position), holder in zip(unsafes, holders):
        function_already_judged = holder is not None and holder["number"] in judged_rows
        if holder is not None:
            judged_rows.add(holder["number"])
        if path not in skipped_by_path:
            skipped_by_path[path] = attribute_lines(files[path][1])
        if path not in comment_lines_by_path:
            # 弄坏开关 safety-read-from-raw-lines 下读原文（字符串里顶格的 // SAFETY: 也当注释）；
            # lines-split-at-unicode-separators 下按 splitlines() 切（U+2028 这类也算换行，与 line_of 按 \n 数的行号错开）。
            text = files[path][0] if BREAK == "safety-read-from-raw-lines" else strip_comments_and_strings(files[path][0], keep_line_comments=True)
            comment_lines_by_path[path] = text.splitlines() if BREAK == "lines-split-at-unicode-separators" else text.split("\n")
        comment = safety_comment(comment_lines_by_path[path], files[path][1].split("\n"), line, skipped_by_path[path])
        if comment is None:
            if BREAK != "safety-comment-ignored" and not (BREAK == "safety-one-place-per-function" and function_already_judged):
                reject(f"{path}:{line} 的 unsafe 上方没有 // SAFETY: 注释", [],
                       "在 unsafe 上方连着写 // SAFETY: 起头的注释，说清调用的前提为什么成立；语句折成几行的，写在语句开头那一行上方，中间不隔别的语句；"
                       "unsafe fn 与 unsafe impl 写在项头上方，中间隔着 #[allow(unsafe_code, …)] 这类独占几行的属性照认。")
        elif holder is not None and BREAK != "safety-premise-ignored" and not (BREAK == "words-one-place-per-function" and function_already_judged):
            words = HOLE_PUNCHING_WORDS if BREAK == "registered-words-ignored" else holder["words"]
            absent = [word for word in words if not safety_word_pattern(word).search(comment)]
            if absent:
                reject(f"{path}:{line} 的 // SAFETY: 注释没写出它依赖的前提：没提到{'、'.join(absent)}", comment.splitlines(),
                       f"SAFETY 注释要写出调用依赖的前提，{REGISTRY_PATH}:{holder['number']} 第 6 段登记的词（{'、'.join(holder['words'])}）这个函数里每一处都要提到；"
                       "这一格只查这几个词有没有出现，写得对不对靠 review。")


def read_source(path):
    original = open(path, encoding="utf-8", errors="replace").read()
    return original, strip_comments_and_strings(original)


def referenced_files(path, original, cleaned):
    """path 这份文件里 include!("…") 与 #[path = "…"] 点到的文件（相对仓根、规范化过），都按 path 所在的目录解析；
    内联 mod 块里的 #[path] 照同一个目录解析（cargo 在那里还要拼上模块路径，这一格不拼）。"""
    targets = []
    for match in FILE_REFERENCE.finditer(cleaned):
        closing = cleaned.find('"', match.end())
        if closing >= 0:
            targets.append(os.path.normpath(os.path.join(os.path.dirname(path), original[match.end():closing])))
    return targets


rust_files_by_crate = {}
for name in crate_names:
    found = []
    for directory, subdirectories, names in os.walk(os.path.join("crates", name)):
        subdirectories[:] = sorted(entry for entry in subdirectories if entry != "target" and not entry.startswith("."))
        found += [os.path.join(directory, file_name) for file_name in sorted(names) if file_name.endswith(".rs")]
    rust_files_by_crate[name] = found
scanned_by_some_crate = {path for found in rust_files_by_crate.values() for path in found}
included_count = 0

for name in crate_names:
    if name not in registry:
        reject(f"crates/{name} 没在登记表里", [f"{REGISTRY_PATH} 没有 {name} 这一行"],
               f"新 crate 先在 {REGISTRY_PATH} 登记一行（根要写 forbid；要放开 unsafe 的走三方、级别写 forbid,deny，放开的每个函数一行，写明文件、函数、unsafe 几处与 SAFETY 注释必须提到的词）。")
        continue
    crate_roots = {root_name: rows for root_name, rows in registry.items() if crate_of(root_name) == name}
    if BREAK == "other-crates-ignored" and not any(row["file"] != "-" for rows in crate_roots.values() for row in rows.values()):
        continue
    crate_directory = os.path.join("crates", name)
    files = {path: read_source(path) for path in rust_files_by_crate[name]}
    file_count += len(files)
    crate_count += 1
    bin_roots = bin_roots_of(sorted(os.path.relpath(path, crate_directory) for path in files))
    # 每份文件归哪个根（库根是 None）；include! 与 #[path] 点到的文件归写着它的那份文件的根，一并扫：
    # 点到的是哪个 crate 都扫的 .rs 不再扫第二遍，点到而不在的跳过（编译过不了的写法归编译器）。弄坏开关 included-files-unscanned 下一份都不追（只扫 .rs）。
    owner = {os.path.join(crate_directory, member): root for root, members in bin_roots.items() for member in members}
    pending = [] if BREAK == "included-files-unscanned" else sorted(files)
    while pending:
        including = pending.pop(0)
        for target in referenced_files(including, *files[including]):
            if (target in files or target in scanned_by_some_crate or os.path.isabs(target)
                    or target.split(os.sep)[0] == ".." or not os.path.isfile(target)):
                continue
            files[target] = read_source(target)
            owner[target] = owner.get(including)
            included_count += 1
            pending.append(target)
    for root_name in sorted(crate_roots):
        if root_name != name and root_name[len(name) + 1:] not in bin_roots and BREAK != "registered-bin-root-unchecked":
            numbers = "、".join(str(row["number"]) for row in crate_roots[root_name].values())
            reject(f"{REGISTRY_PATH}:{numbers} 登记的根 {root_name} 不是 crates/{name}/ 下的 bin 根",
                   [f"crates/{name}/ 下的 bin 根：{'、'.join(sorted(bin_roots)) or '（没有）'}"],
                   "第 1 段写 bin 根时写 <crate 目录名>/src/main.rs、<crate 目录名>/src/bin/<名>.rs 或 <crate 目录名>/src/bin/<名>/main.rs，那个文件要在；"
                   "bin 根改了名或删了，改这几行或删掉。")
    judge_root(name, name, os.path.join(crate_directory, "src", "lib.rs"),
               {path: texts for path, texts in files.items() if owner.get(path) is None}, crate_roots[name], True)
    for root in sorted(bin_roots):
        root_name = f"{name}/{root}"
        if root_name not in crate_roots and BREAK == "unregistered-bin-roots-ignored":
            continue
        root_files = {path: texts for path, texts in files.items() if owner.get(path) == root}
        judge_root(root_name, name, os.path.join(crate_directory, root), root_files, crate_roots.get(root_name, {}), root_name in crate_roots)

finish()
missing = sorted({crate_of(root_name) for root_name in registry} - set(crate_names))
row_total = sum(len(rows) for rows in registry.values())
relax_total = sum(entry[5] for entry in scanned)
unsafe_total = sum(entry[6] for entry in scanned)
bin_entries = [entry for entry in scanned if entry[0] != entry[2]]
pieces = []
for crate_name in dict.fromkeys(entry[2] for entry in scanned):
    entries = [entry for entry in scanned if entry[2] == crate_name]
    pieces += [f"{label} 根 {level}、放开 {relaxes} 处、unsafe {unsafes} 处、{count} 个 .rs、登记 {rows} 行"
               for label, unregistered, _crate, count, level, relaxes, unsafes, rows in entries if not unregistered]
    unregistered_entries = [entry for entry in entries if entry[1]]
    if unregistered_entries:
        pieces.append(f"{crate_name} 没登记的 bin 根 {len(unregistered_entries)} 个（照 forbid 判）、放开 {sum(entry[5] for entry in unregistered_entries)} 处、"
                      f"unsafe {sum(entry[6] for entry in unregistered_entries)} 处、{sum(entry[3] for entry in unregistered_entries)} 个 .rs")
summary = "；".join(pieces)
print(f"  ✓ crates/ 里放开 unsafe 的地方与登记表对上：扫了 {crate_count} 个 crate、{file_count} 个 .rs、登记表 {row_total} 行、放开 {relax_total} 处、unsafe {unsafe_total} 处、"
      f"bin 根 {len(bin_entries)} 个（登记了的 {sum(1 for entry in bin_entries if not entry[1])} 个）（{summary}）；登记了但仓里没有的 crate {len(missing)} 个：{'、'.join(missing) or '无'}；"
      f"include! 与 #[path] 点到、没有哪个 crate 当 .rs 扫的文件另扫了 {included_count} 份（算进写着它的那份文件的根，上面各根的个数里含着）")
PY
}

# ── 格 gpu-exemptions：不走 GPU 的计算与豁免表逐项对上（欠账 C626（走 GPU 的计算类没有登记））
# 判法只在 scripts/crash/gpu-exemptions-check.py check：豁免表 .claude/gate.d/gpu-exemptions.tsv 每一行四段（截名、代码标记名、为什么上不了 GPU、
# 用户点头日期），crates/ 下 .rs 里每一处 // gpu-exempt: <截名> 都有表行、每一行都有标记。被判的根里没有 crates/：这一格本次无对象可判。
# 管不到的：一截计算真在不在 GPU 上跑、标记写没写在真不走 GPU 的那一截上、为什么写得对不对，靠 review 与用户点头。
cell_gpu_exemptions() {
  if [[ ! -d "$ROOT/crates" ]]; then
    echo "  ! 没有 crates/：豁免表这一格本次无对象可判"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/scripts/crash/gpu-exemptions-check.py" check "$ROOT"
}

# ── 格 crash-point-replay-segments：崩溃点重放的按段登记成形
# 判法只在 scripts/crash/lib_crash_point_replay_items.py segments（两个入口 scripts/crash/crash-point-replay.sh、scripts/crash/crash-amplification.sh 开跑前调同一份）：
# .claude/gate.d/crash-point-replay-segments.tsv 每一行五段（项名、流名、段号、段写数、段写表摘要），项名在合表 .claude/gate.d/crash-point-replay-items.tsv 里，
# 流名匹配 ^[A-Za-z][A-Za-z0-9_]*$（引擎按它取这条流的行），段号与段写数是非负整数且段写数大于 0，摘要是 64 位小写十六进制，同一（流名，段号）只一行。
# 被判的根里合表与按段登记文件都没有：这一格本次无对象可判；合表在而按段登记文件不在判红（入口开跑前要读它）。
# 管不到的：摘要是不是那一段写表真的 sha256、段号与段写数对不对得上那条流（引擎跑起来才知道），靠引擎那一侧与 review。
cell_crash_point_replay_segments() {
  if [[ ! -f "$ROOT/.claude/gate.d/crash-point-replay-segments.tsv" && ! -f "$ROOT/.claude/gate.d/crash-point-replay-items.tsv" ]]; then
    echo "  ! 没有崩溃点重放的合表与按段登记文件：这一格本次无对象可判"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/scripts/crash/lib_crash_point_replay_items.py" segments "$ROOT"
}


# ── 格 crate-sources-reachable：没有 cargo 不编的落空源文件
# 判据与射程在 scripts/gate/crate-sources-reachable.py 文件头：scripts/experiment-apparatus 与 crates 下每一份 .rs，要么是某个 crate 的目标
# （[lib]、[[bin]]、[[test]]、自动发现的那几类），要么从目标经 #[path]、mod、include! 引得到，include_str!、include_bytes! 当文本读的也算。
# 管不到的：引到了却是死代码（cfg 永远不开的模块）、引它的那个目标自己从来不跑。
# red：一份平铺 crate 里多一份谁都没引的 .rs；green：同一份 crate 里每份都登记或被引。
cell_crate_sources_reachable() {
  local directories=() directory
  for directory in "$ROOT/scripts/experiment-apparatus" "$ROOT/crates"; do
    [[ -d "$directory" ]] && directories+=("$directory")
  done
  if ((${#directories[@]} == 0)); then
    echo "  ! 本次无对象可判：$ROOT 下没有 scripts/experiment-apparatus/ 与 crates/"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/scripts/gate/crate-sources-reachable.py" "${directories[@]}"
}

# ── 格 ordinal-names：代码里我们自己的名字不带序号与位置词（records/2026-10-13-复用键与全仓改名.md）
# 判据：git 列出的 crates/、scripts/、.claude/gate.d、.claude/hooks 下每一份 .rs、.sh、.py（这几处里的样本也算），下面三样名字按词边界含了
#   词表 .claude/naming-ordinal-tokens 里的一个词，判违规，逐处列「文件:行 种类 名字（含 词）」：
#   ① 文件名与它上面每一级目录名（目录名的违规记在那一级目录的路径上，同一个目录只报一次）；
#   ② 源码里的名字：.rs 把注释与字符串字面量（含原始串、字节串、字符字面量）抹成空白之后的每一个标识符（函数、变量、常量、模块、类型、字段、
#      测试、生命周期都在内；同一份文件同一个名字只报第一处）；.sh 的函数名、赋值（含 local / export / declare / readonly 与命令前的 X=…）、
#      for / read 的变量与 $ 展开里的变量名（# 注释与单引号串去掉，双引号串里只留 $ 展开，引号可以跨行）；.py 经 tokenize 出来的每一个标识符
#      （关键字除外）；.sh 里起 python 的 heredoc 正文按 .py 判，别的 heredoc 正文当数据，只认界符不带引号时里面的 $ 展开；
#   ③ 环境变量名：.rs 里 env::var / var_os / env! / option_env! / set_var / remove_var / .env(…) 的字符串参数，.py 里 os.environ.get / [] / setdefault / pop、
#      getenv / putenv / unsetenv 的字符串参数（.sh 里的环境变量就是变量，归 ②）。
# 词边界：名字与词都先摊平——驼峰拆开（FooBar 拆成 foo、bar 两段）、大小写不分、字母数字之外的字符（_、-、.、/）一律当边界——
#   词前后紧挨的不是字母或数字就算命中，复合名里夹着的也算；词后面紧跟一位数字的不算（那是另一个号）。
# 判法是纯函数（ordinal_tokens_in 输入一个名字、输出命中的词；names_in 输入一份源文件的路径与文本、输出它的名字表），不起 cargo、不编译。
# 词表一行一个小写词（字母起头，只含字母、数字与 _），# 后写为什么它是序号或位置词；写坏的行、同一个词登两行、没写理由的行判红，词表不在判红。
# 还没改完的逐处登记在 .claude/naming-ordinal-exclude（一行 <路径>  # 理由；路径是文件，或目录名本身带词的那一级目录），登记的那一处的违规不判；
#   排除项指向不存在的路径、那一处已经一个违规都没有了、或没写理由，判红（排除只缩不涨）。排除表不在等于一条都没排。
# tokenize 认不出的 .py（与 .sh 里认不出的 python heredoc）逐个列在「扫了…」那一行，不算判过；一份 .rs / .sh / .py 都没列出来：这一格本次无对象可判。
# 管不到的：注释、文档与输出文字里的序数与位置代称（不归这一格判，靠 review；改过的名字登记在 .claude/kb/term-renames.md）；
#   词表外的写法（别的序数词、位置词，要判就往词表里加）；词在名字里其实是别的意思（说「流里最早那一个下标」的名字也照判，改名或登记排除）；
#   .sh 里命令的参数与数组下标、Rust 宏展开出来的名字。
# red：一份 .rs 带 enumerate_ 接层序号的函数、一份 .sh 给带流序号的变量赋值，排除表一行指向已经改好名的文件、一行指向不存在的文件；
# green：同样的词只出现在 .rs / .sh / .py 的注释与字符串里、词后面紧跟一位数字的名字、登记在排除表里还没改完的那一份 .rs。
cell_ordinal_names() {
python3 - <<'PY'
import bisect, io, keyword, os, re, subprocess, sys, tokenize

TOKENS_FILE = ".claude/naming-ordinal-tokens"
EXCLUDE_FILE = ".claude/naming-ordinal-exclude"
SCANNED_DIRECTORIES = ("crates", "scripts", ".claude/gate.d", ".claude/hooks")
SUFFIXES = (".rs", ".sh", ".py")
BROKEN = os.environ.get("GATE_ORDINAL_NAMES_BREAK", "")


def registry(path):
    """一行一条 `<条目>  # 理由` 的登记表：[(行号, 条目, 理由)]；文件不在返回 None。"""
    if not os.path.isfile(path):
        return None
    entries = []
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        body = line.split("#", 1)[0].strip()
        if body:
            entries.append((number, body, line.split("#", 1)[1].strip() if "#" in line else ""))
    return entries


def normalized_words(name):
    """名字按词边界摊平：驼峰拆开、全部小写、字母数字之外的一律当成 `_`。"""
    name = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    name = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def token_patterns(tokens):
    patterns = []
    for token in tokens:
        flat = re.escape(normalized_words(token))
        if BROKEN == "compound-names-unseen":
            patterns.append((token, re.compile("^" + flat + "$")))
        else:
            patterns.append((token, re.compile(r"(?<![a-z0-9])" + flat + r"(?![a-z0-9])")))
    return patterns


def ordinal_tokens_in(name, patterns):
    """纯函数：名字里按词边界含了词表里的哪几个词（次序照词表）。"""
    flat = normalized_words(name)
    return [token for token, pattern in patterns if pattern.search(flat)]


# ── 各语言里我们自己的名字：(行号, 种类, 名字) ──

RUST_ENVIRONMENT = re.compile(r"(?:\benv::var(?:_os)?|\bvar(?:_os)?|\benv!|\boption_env!|\bset_var|\bremove_var|\.env|\.env_remove)\s*\(\s*$")
RUST_LEXEME_START = re.compile(r"//|/\*|(?<![A-Za-z0-9_])b?r#*\"|(?<![A-Za-z0-9_])b\"|\"|'")
RUST_BLOCK_COMMENT_EDGE = re.compile(r"/\*|\*/")
RUST_STRING_BODY = re.compile(r"(?:\\.|[^\"\\])*\"", re.DOTALL)
RUST_CHARACTER_LITERAL = re.compile(r"'(?:\\(?:u\{[0-9A-Fa-f]+\}|x[0-9A-Fa-f]{2}|.)|[^\\'\n])'")
IDENTIFIER = re.compile(r"(?<![A-Za-z0-9_])(?:r#)?([A-Za-z_][A-Za-z0-9_]*)")


def line_of(line_starts, offset):
    return bisect.bisect_right(line_starts, offset)


def rust_names(text):
    """.rs：注释与字符串字面量抹成空白后的每一个标识符（含生命周期与标签），加读写环境变量处字符串里的变量名。"""
    names, code, index, length = [], [], 0, len(text)
    line_starts = [0] + [match.end() for match in re.finditer("\n", text)]
    blank = lambda piece: re.sub(r"[^\n]", " ", piece)
    while index < length:
        start = RUST_LEXEME_START.search(text, index)
        if not start:
            code.append(text[index:]); break
        code.append(text[index:start.start()])
        lexeme, index = start.group(0), start.start()
        if lexeme == "//":
            end = text.find("\n", index)
            end = length if end < 0 else end
        elif lexeme == "/*":
            depth, end = 1, index + 2
            while depth:
                edge = RUST_BLOCK_COMMENT_EDGE.search(text, end)
                if not edge:
                    end = length; break
                depth += 1 if edge.group(0) == "/*" else -1
                end = edge.end()
        elif lexeme == "'":
            literal = RUST_CHARACTER_LITERAL.match(text, index)
            if not literal:
                code.append("'"); index += 1; continue
            end = literal.end()
        else:
            opening = start.end()
            hashes = lexeme.count("#") if "r" in lexeme else None
            if hashes is None:
                body = RUST_STRING_BODY.match(text, opening)
                end = body.end() if body else length
            else:
                closing = text.find('"' + "#" * hashes, opening)
                end = length if closing < 0 else closing + 1 + hashes
            content = text[opening:end - 1 - (hashes or 0)]
            if RUST_ENVIRONMENT.search("".join(code[-8:])[-40:]) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", content):
                names.append((line_of(line_starts, index), "环境变量", content))
        if BROKEN == "comments-and-strings-judged":
            code.append(text[index:end])
        else:
            code.append(blank(text[index:end]))
        index = end
    stripped = "".join(code)
    seen = set()
    for match in IDENTIFIER.finditer(stripped):
        name = match.group(1)
        if name in seen:
            continue
        seen.add(name)
        names.append((line_of(line_starts, match.start()), "名字", name))
    return names


SHELL_HEREDOC = re.compile(r"<<(-?)\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
SHELL_ASSIGNMENT = re.compile(r"(?:^|[\s;&|(`{!])([A-Za-z_][A-Za-z0-9_]*)(?:\[[^\]]*\])?\+?=")
SHELL_FUNCTION = re.compile(r"^\s*(?:function\s+([A-Za-z_][A-Za-z0-9_:.-]*)|([A-Za-z_][A-Za-z0-9_:.-]*)\s*\(\s*\))")
SHELL_EXPANSION = re.compile(r"\$\{?#?!?([A-Za-z_][A-Za-z0-9_]*)")
SHELL_FOR = re.compile(r"(?:^|[\s;&|(])(?:for|select)\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\b")
SHELL_READ = re.compile(r"(?:^|[\s;&|(])read((?:\s+[^\s;&|<>)]+)+)")


def read_variables(arguments):
    """read 的参数里哪几个是变量名：-a 后面那一个是数组名，-d -i -n -N -p -t -u 后面那一个是选项值。"""
    words, names, index = arguments.split(), [], 0
    while index < len(words):
        word = words[index]
        if word.startswith("-"):
            if word[-1] in "dinNptu":
                index += 1
            elif word[-1] == "a" and index + 1 < len(words):
                names.append(words[index + 1]); index += 1
        elif re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", word):
            names.append(word)
        index += 1
    return names


def shell_names(text, unjudged):
    """.sh：函数名、赋值（含 local / export / declare / readonly 与命令前的 X=…）的变量名、for / read 的变量、$ 展开里的变量名。
    逐字扫一遍：# 注释去掉，单引号串（与 $'…'）抹成空白，双引号串里只留 $ 展开，引号可以跨行；
    heredoc 起头那一行起 python 的，heredoc 正文按 python 判；别的 heredoc 正文是数据，只认界符不带引号时里面的 $ 展开。"""
    names, lines, number, quote = [], text.split("\n"), 0, None
    while number < len(lines):
        line = lines[number]
        number += 1
        out, index, heredocs = [], 0, []
        while index < len(line):
            character = line[index]
            if BROKEN == "comments-and-strings-judged" and quote is None:
                heredoc = SHELL_HEREDOC.match(line, index) if line.startswith("<<", index) and not line.startswith("<<<", index) else None
                if heredoc:
                    heredocs.append((heredoc.group(3), bool(heredoc.group(2)), heredoc.group(1) == "-"))
                    index = heredoc.end(); continue
                out.append(character); index += 1; continue
            if quote is None:
                if character == "#" and (index == 0 or line[index - 1] in " \t;|&("):
                    break
                if character == "\\":
                    out.append(line[index:index + 2]); index += 2; continue
                if line.startswith("<<", index) and not line.startswith("<<<", index):
                    heredoc = SHELL_HEREDOC.match(line, index)
                    if heredoc:
                        heredocs.append((heredoc.group(3), bool(heredoc.group(2)), heredoc.group(1) == "-"))
                        out.append(" " * (heredoc.end() - index)); index = heredoc.end(); continue
                if line.startswith("$'", index):
                    quote = "$'"; out.append("@ "); index += 2; continue
                if character in "'\"":
                    quote = character; out.append("@"); index += 1; continue
                out.append(character)
            elif quote in ("'", "$'"):
                if quote == "$'" and character == "\\":
                    out.append("  "); index += 2; continue
                if character == "'":
                    quote = None
                out.append(" ")
            else:
                if character == "\\":
                    out.append("  "); index += 2; continue
                if character == '"':
                    quote = None; out.append(" ")
                elif character == "$":
                    expansion = SHELL_EXPANSION.match(line, index)
                    if expansion:
                        out.append(" " + expansion.group(0)); index = expansion.end(); continue
                    out.append(" ")
                else:
                    out.append(" ")
            index += 1
        code = "".join(out)
        for match in SHELL_FUNCTION.finditer(code):
            names.append((number, "函数", match.group(1) or match.group(2)))
        for match in SHELL_ASSIGNMENT.finditer(code):
            names.append((number, "变量", match.group(1)))
        for match in SHELL_FOR.finditer(code):
            names.append((number, "变量", match.group(1)))
        for match in SHELL_READ.finditer(code):
            names.extend((number, "变量", name) for name in read_variables(match.group(1)))
        for match in SHELL_EXPANSION.finditer(code):
            names.append((number, "变量", match.group(1)))
        for delimiter, quoted, strip_tabs in heredocs:
            body_start = number
            while number < len(lines) and (lines[number].lstrip("\t") if strip_tabs else lines[number]).rstrip() != delimiter:
                number += 1
            body = lines[body_start:number]
            number += 1
            if re.search(r"(?<![A-Za-z0-9_./-])python3?(?![A-Za-z0-9_.-])", line):
                inner_names = python_names("\n".join(body))
                if inner_names is None:
                    unjudged.append(f"第 {body_start} 行起的 python heredoc")
                    continue
                names.extend((body_start + inner, kind, name) for inner, kind, name in inner_names)
            elif not quoted:
                for offset, body_line in enumerate(body, 1):
                    for match in SHELL_EXPANSION.finditer(body_line):
                        names.append((body_start + offset, "变量", match.group(1)))
    return names


PYTHON_ENVIRONMENT_CALLS = ("environ.get", "environ.setdefault", "environ.pop", "getenv", "putenv", "unsetenv", "environ[")


def python_names(text):
    """.py：tokenize 出来的每一个标识符（关键字除外，f 串花括号里的表达式也算），加读写环境变量处字符串里的变量名；认不出返回 None。"""
    names, previous, seen = [], [], set()
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    except (tokenize.TokenError, IndentationError, SyntaxError):
        return None
    for token in tokens:
        if token.type == tokenize.NAME and not keyword.iskeyword(token.string) and token.string not in seen:
            seen.add(token.string)
            names.append((token.start[0], "名字", token.string))
        elif token.type == tokenize.STRING:
            tail = "".join(previous[-4:])
            literal = re.fullmatch(r"[rRbBuU]?(['\"])([A-Za-z_][A-Za-z0-9_]*)\1", token.string)
            if literal and any(tail.endswith(call) or tail.endswith(call + "(") for call in PYTHON_ENVIRONMENT_CALLS):
                names.append((token.start[0], "环境变量", literal.group(2)))
        if BROKEN == "comments-and-strings-judged" and token.type in (tokenize.STRING, tokenize.COMMENT):
            for word in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", token.string):
                names.append((token.start[0], "名字", word))
        if token.type not in (tokenize.NL, tokenize.NEWLINE, tokenize.COMMENT, tokenize.INDENT, tokenize.DEDENT):
            previous.append(token.string)
    return names


def names_in(path, text, unjudged):
    """纯函数：一份源文件里我们自己的名字 [(行号, 种类, 名字)]；认不出的部分记进 unjudged（逐个列出，不算判过）。"""
    if path.endswith(".rs"):
        return rust_names(text)
    if path.endswith(".sh"):
        if BROKEN == "shell-unscanned":
            return []
        inner_unjudged = []
        names = shell_names(text, inner_unjudged)
        unjudged.extend(f"{path} {part}" for part in inner_unjudged)
        return names
    names = python_names(text)
    if names is None:
        unjudged.append(path)
        return []
    return names


def main():
    tokens = registry(TOKENS_FILE)
    if tokens is None:
        print(f"  ✗ 读不到序号与位置词表 {TOKENS_FILE}")
        print("     → 怎么办：从 git 恢复它；这一格没有词表就什么都判不了，不是通过。")
        return 1
    table_problems = []
    seen_tokens = {}
    for number, token, reason in tokens:
        if not re.fullmatch(r"[a-z][a-z0-9_]*", token):
            table_problems.append(f"{TOKENS_FILE}:{number}「{token}」不是小写字母起头、只含小写字母、数字与 _ 的词")
        elif normalized_words(token) in seen_tokens:
            table_problems.append(f"{TOKENS_FILE}:{number}「{token}」与第 {seen_tokens[normalized_words(token)]} 行登记的是同一个词")
        else:
            seen_tokens[normalized_words(token)] = number
        if len(reason) < 4:
            table_problems.append(f"{TOKENS_FILE}:{number}「{token}」没写为什么算序号或位置词（# 后面至少 4 个字）")
    patterns = token_patterns([token for _number, token, _reason in tokens])
    excluded = registry(EXCLUDE_FILE) or []

    listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *SCANNED_DIRECTORIES],
                            capture_output=True)
    if listed.returncode != 0:
        print(f"  ✗ git ls-files 退 {listed.returncode}：{listed.stderr.decode('utf-8', 'replace').strip()}")
        print("     → 怎么办：在 git 仓的根上跑这一道；列不出文件就判不了，不是通过。")
        return 1
    paths = sorted(path for path in listed.stdout.decode("utf-8", "replace").split("\0") if path.endswith(SUFFIXES) and os.path.isfile(path))
    if not paths:
        print("  ! 本次无对象可判：" + "、".join(SCANNED_DIRECTORIES) + " 下一份 .rs、.sh、.py 都没列出来")
        return 77

    violations = {}  # 违规记在哪一处（文件或目录的路径） -> [行]
    unparsed = []
    directories_seen = set()
    for path in paths:
        parts = path.split("/")
        for depth in range(1, len(parts)):
            directory = "/".join(parts[:depth])
            if directory in directories_seen:
                continue
            directories_seen.add(directory)
            hit = ordinal_tokens_in(parts[depth - 1], patterns)
            if hit:
                violations.setdefault(directory, []).append(f"{directory}/ 目录名「{parts[depth - 1]}」（含 {'、'.join(hit)}）")
        hit = ordinal_tokens_in(parts[-1], patterns)
        if hit:
            violations.setdefault(path, []).append(f"{path} 文件名（含 {'、'.join(hit)}）")
        text = open(path, encoding="utf-8", errors="replace").read()
        reported = set()
        for number, kind, name in names_in(path, text, unparsed):
            if (kind, name) in reported:
                continue
            hit = ordinal_tokens_in(name, patterns)
            if hit:
                reported.add((kind, name))
                violations.setdefault(path, []).append(f"{path}:{number} {kind} {name}（含 {'、'.join(hit)}）")

    excluded_paths = {entry for _number, entry, _reason in excluded}
    problems, exclusion_problems = [], []
    for owner in sorted(violations):
        if owner in excluded_paths and BROKEN != "exclusions-ignored":
            continue
        problems += violations[owner]
    for number, entry, reason in excluded:
        if not os.path.exists(entry):
            exclusion_problems.append(f"{EXCLUDE_FILE}:{number} 指向不存在的路径 {entry}")
        elif entry not in violations:
            exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 里已经一个带序号或位置词的名字都没有了，这一行删掉")
        elif len(reason) < 4:
            exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 没写为什么先不改（# 后面至少 4 个字）")

    unparsed_note = "无" if not unparsed else "、".join(unparsed)
    print(f"  · 扫了 {len(paths)} 份 .rs / .sh / .py 与它们所在的 {len(directories_seen)} 个目录，词表 {len(patterns)} 个词，排除 {len(excluded)} 条；"
          f"tokenize 认不出、没判的 {len(unparsed)} 处：{unparsed_note}")
    for problem in table_problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if table_problems:
        print(f"  ✗ {TOKENS_FILE} 有 {len(table_problems)} 行写坏了")  # gate-lint:summary
        print("     → 怎么办：一行一个小写词（字母起头，只含字母、数字与 _），同一个词只登一行，# 后面写为什么它是序号或位置词。")
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if problems:
        print(f"  ✗ {len(problems)} 处名字带序号或位置词（词表 {TOKENS_FILE}，按词边界认，复合名也算）")  # gate-lint:summary
        print("     → 怎么办：按 .claude/agent-common.md「写」一节「名字、注释、文档、输出文字都不用序数或位置代称」改成说内容的名字（例 enumerate_crash_states、new_pool_file_creation_stream），")
        print("                改过的名字登记进 .claude/kb/term-renames.md；一时改不完的文件登记进 " + EXCLUDE_FILE + "（一行 <路径>  # 理由），改完删那一行。")
    for problem in exclusion_problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if exclusion_problems:
        print(f"  ✗ {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由")  # gate-lint:summary
        print("     → 怎么办：指向不存在的与已经改完的那几行删掉；留着的每一行写明为什么先不改。排除只缩不涨。")
    if table_problems or problems or exclusion_problems:
        return 1
    print(f"  ✓ 名字里没有序号与位置词（扫了 {len(paths)} 份 .rs / .sh / .py、{len(directories_seen)} 个目录，词表 {len(patterns)} 个词，排除 {len(excluded)} 条）")
    return 0


sys.exit(main())
PY
}

# ── 格 data-names：以数据形式存在的名字也按命名纪律判（欠账 C430）
# 判法只在 scripts/gate/data-name-lint.py：naming-lint 只查我们声明的名字，而字符串字面量与 kb 表格里还活着一批人真正读到的名字——
#   scripts/experiment-apparatus/**/*.rs（顶层的装置与装置目录里的子模块）与 crates/*/src/**/*.rs 里的元组首列 ("名字", …)、=> "名字" 右侧（含 cargo fmt 折成 => { "名字" } 的分支）、格式串里 键= 的键名，
#   .claude/kb/**/*.md 表格单元格里反引号包着的名字（只认整串是 [a-z][a-z0-9_]*）。
# 按同一套判：_ 切词，单字母判红，词在 naming-lint.sh 的缩写词表里（脚本从那份读，不另抄一份）或在脚本的补充缩写里（mac）而没登记进 .claude/abbreviations 判红；
#   e<数字>、c<数字> 这类整类登记的编号放行。已经落进留存产物、改名要连产物一起重跑的存量逐条登记在 .claude/data-name-lint-exclude
#   （一行 <路径>:<串>  # 理由；指向不存在的路径、已经一个违规都没有了的行、没写理由的行判红，排除只缩不涨）。
# 成功句报：扫了几份文件、几个串，排除表放行了几条。扫不到一份 .rs 与 kb：这一格本次无对象可判（退 77）。
# 管不到的：不是这三种形态的名字（结构体字段赋的字符串、函数调用的实参、拼出来的名字），词表之外的缩写，名字起得好不好，靠 review。
# red：一份 .rs 里元组首列、=> 右侧、键= 各一个带缩写或单字母的名字，一份 kb 表格里一个带缩写的单元格，排除表一行指向已经改好名的串、一行指向不存在的文件；
# data-names-folded-arm-red：一条 match 分支折成「=> {」「"名字"」「}」三行、串里带单字母，判红；if / else 块里带缩写的串前面不是 =>，不判（扫 3 个串）；
# green：样本自己的 .claude/abbreviations 登记了缩写与整类编号，同样的词只出现在注释、函数调用实参、围栏代码块与表格外的正文里，排除表登记的那个串还在。
cell_data_names() {
  python3 "$STAGE_REPOSITORY/scripts/gate/data-name-lint.py" "$ROOT"
}

# ── 格 verus-ghost-names：被验 crate 里不自己定义名叫 Ghost / Tracked 的类型，也不把别的东西改名成这两个名字（records/2026-10-29-subagent拆分提案.md 第一百三十三节，取甲）
# 为什么：函数单元图的两份实现（crates/singlefs-unit-graph/src/verus_ghost_code.rs 与 scripts/gate/lib_function_units.py 的 VERUS_GHOST_WRAPPERS）按类型名认 Ghost / Tracked，
#   不分 vstd 的那两个与同名的自定义类型；被验 crate 里出现同名的自定义类型，单元图就把真代码当幽灵代码剔出键。两份实现不动，在源码这一侧拒同名。
# 对象：被判的根里 VERUS_PINS_TABLE（.claude/gate.d/verus-pins.tsv）每一行「crate<TAB><目录>…」登记的 crate，<目录>/src/ 下全部 .rs（去掉注释、字符串与字符字面量之后判，行号照原文）。
# 判据，判红的两样：
#   ① 类型定义：struct、enum、union、type、trait 后面紧跟的名字是 Ghost 或 Tracked（被验 crate 自己定义的，都不来自 vstd）；
#   ② 改名：use 语句里「<路径> as Ghost」「<路径> as Tracked」（花括号分组里的按分组前缀补全路径），路径不是 vstd:: 起头、以同名结尾的那一种（use vstd::prelude::Ghost as Ghost 放过）。
#   登记表不在、或一行 crate 都没有：退 77；登记的目录不在：红，点名那一行。
# 管不到的：宏展开出来的定义与改名；extern crate vstd as … 这类把整个 vstd 改名之后再取 Ghost 的写法；不叫这两个名字、却在别处被单元图当幽灵包装认的类型（单元图只认这两个名字）。
# 样本：verus-ghost-names-red（登记两个 crate：一个 src 里有 pub struct Ghost<T>、type Tracked = …，另一个有 use crate::holder::Holder as Tracked 与分组 use crate::{wrap::Wrapper as Ghost}，四处逐条点名判红；
#   另一行登记的目录不在，判红点名）；verus-ghost-names-green（use vstd::prelude::*、use vstd::prelude::Ghost as Ghost、注释与字符串里写的「struct Ghost」、
#   没登记的 crate 里的 struct Ghost 都不判，判绿报查了几份 .rs）。
# 弄坏开关 GATE_VERUS_GHOST_NAMES_BREAK=<项>，见文件头开关表：definitions-unseen 不判 ①；renames-unseen 不判 ②；comments-judged 不去注释与字符串；vstd-renames-judged 连 vstd 的同名改名也判红。
# 退役判据：两份单元图实现改成按解析出的路径认幽灵包装（拆分提案第一百三十三节的乙）时删掉这一格。
VERUS_PINS_TABLE=".claude/gate.d/verus-pins.tsv"
cell_verus_ghost_names() {
python3 - "$VERUS_PINS_TABLE" "${GATE_VERUS_GHOST_NAMES_BREAK:-}" <<'PY'
import glob, os, re, sys

table_path, broken = sys.argv[1], sys.argv[2]
NAMES = ("Ghost", "Tracked")
DEFINITION = re.compile(r"\b(struct|enum|union|type|trait)\s+(Ghost|Tracked)\b")
USE_STATEMENT = re.compile(r"\buse\s+([^;]*);")
RENAME = re.compile(r"((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*)\s+as\s+(Ghost|Tracked)\b")


def code_only(text):
    """注释、字符串与字符字面量换成空格（换行照留，行号不变）。"""
    out, index, length = [], 0, len(text)
    while index < length:
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            out.append(" " * (end - index))
            index = end
        elif text.startswith("/*", index):
            depth, cursor = 1, index + 2
            while cursor < length and depth:
                if text.startswith("/*", cursor):
                    depth, cursor = depth + 1, cursor + 2
                elif text.startswith("*/", cursor):
                    depth, cursor = depth - 1, cursor + 2
                else:
                    cursor += 1
            out.append(re.sub(r"[^\n]", " ", text[index:cursor]))
            index = cursor
        elif (re.match(r'(?:b?r(#*)")', text[index:index + 8])
              and not (index and (text[index - 1].isalnum() or text[index - 1] == "_"))):
            hashes = re.match(r'b?r(#*)"', text[index:index + 8]).group(1)
            end = text.find('"' + hashes, text.find('"', index) + 1)
            end = length if end < 0 else end + 1 + len(hashes)
            out.append(re.sub(r"[^\n]", " ", text[index:end]))
            index = end
        elif text[index] == '"':
            cursor = index + 1
            while cursor < length and text[cursor] != '"':
                cursor += 2 if text[cursor] == "\\" else 1
            out.append(re.sub(r"[^\n]", " ", text[index:cursor + 1]))
            index = cursor + 1
        elif text[index] == "'" and re.match(r"'(?:\\.|[^\\'\n])'", text[index:index + 4]):
            literal = re.match(r"'(?:\\.|[^\\'\n])'", text[index:index + 4]).group(0)
            out.append(" " * len(literal))
            index += len(literal)
        else:
            out.append(text[index])
            index += 1
    return "".join(out)


def renames(statement_body):
    """use 语句正文里改名成 Ghost / Tracked 的 → [(补全的路径, 新名)]；花括号分组按分组前缀补全。"""
    found = []
    for match in RENAME.finditer(statement_body):
        prefix_parts, depth = [], 0
        for position in range(match.start() - 1, -1, -1):
            character = statement_body[position]
            if character == "}":
                depth += 1
            elif character == "{":
                if depth:
                    depth -= 1
                else:
                    head = re.search(r"((?:[A-Za-z_][A-Za-z0-9_]*::)+)\s*$", statement_body[:position])
                    if head:
                        prefix_parts.insert(0, head.group(1))
        found.append(("".join(prefix_parts) + match.group(1), match.group(2)))
    return found


if not os.path.isfile(table_path):
    print(f"  ! 没有 {table_path}，没有登记被验的 crate，这一格无对象可判")
    sys.exit(77)
rows = []
with open(table_path, encoding="utf-8", errors="replace") as handle:
    for number, line in enumerate(handle, 1):
        columns = line.rstrip("\n").split("\t")
        if columns[0] == "crate" and len(columns) >= 2:
            rows.append((number, columns[1].strip()))
if not rows:
    print(f"  ! {table_path} 里一行 crate 都没有，这一格无对象可判")
    sys.exit(77)
problems, scanned = [], 0
for number, directory in rows:
    if not os.path.isdir(directory):
        problems.append(f"{table_path}:{number} 登记的被验 crate 目录 {directory} 不在")
        continue
    for path in sorted(glob.glob(os.path.join(directory, "src", "**", "*.rs"), recursive=True)):
        scanned += 1
        text = open(path, encoding="utf-8", errors="replace").read()
        code = text if "comments-judged" in broken else code_only(text)
        if "definitions-unseen" not in broken:
            for match in DEFINITION.finditer(code):
                line_number = code.count("\n", 0, match.start()) + 1
                problems.append(f"{path}:{line_number} 定义了名叫 {match.group(2)} 的 {match.group(1)}（被验 crate 里的同名类型，单元图会当 vstd 的幽灵包装剔出键）")
        if "renames-unseen" not in broken:
            for statement in USE_STATEMENT.finditer(code):
                for full_path, new_name in renames(statement.group(1)):
                    from_vstd = full_path.startswith("vstd::") and full_path.endswith("::" + new_name)
                    if from_vstd and "vstd-renames-judged" not in broken:
                        continue
                    line_number = code.count("\n", 0, statement.start()) + 1
                    problems.append(f"{path}:{line_number} 把 {full_path} 改名成 {new_name}（不是 vstd 的同名类型，单元图会当幽灵包装剔出键）")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 被验 crate 里有 {len(problems)} 处自己定义或改名出 Ghost / Tracked（查了 {len(rows)} 个登记的 crate、{scanned} 份 .rs）")
    print("     → 怎么办：给自定义的类型、改名的东西换一个说出它是什么的名字（Ghost / Tracked 只留给 vstd 的那两个）；登记的目录搬了家就照 .claude/rules/path-moves.md 改 verus-pins.tsv 那一行；"
          "判据在 code-source-discipline.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 被验 crate 里没有自己定义或改名出的 Ghost / Tracked：查了 {len(rows)} 个登记的 crate、{scanned} 份 .rs")
PY
}

# ── 格 refused-tests：验证包里拒算的测试函数与拒算登记表逐条对上
# 判法只在 scripts/gate/refused-test-registry.py check（它文件头写取法、完整性闸与弄坏开关）：照门禁 crash-point-replay 的 crash-case-replay 格同一套起法跑
# admission.py test-functions-request 与 test-plan（验证包不带特性那一趟、--exclude-registered），逐条拒算行（单元键算不出：闭包读活树、读运行时拼出的仓根路径这类，按条复用时
# 照跑、不复用、不入库）与登记表 .claude/gate.d/refused-tests.tsv 比：拒算的测试函数不在表里判红；表里登了而已经不拒算的判红（只缩不涨）；理由至少 8 个字，同一个测试函数不登两行。
# 登记由主 agent 在合入后验证之后二次确认再写，实现员不登记（.claude/main-agent.md「合入后验证之后判新出现的拒算」）。
# 样本根里放 .refused-tests-listing.tsv（照 test-plan 标准输出的写法造的文件）的，样本根（不是这道阶段所在的仓、也不是 --staged 的临时树）拿它当计划输出，不起 admission.py。
# 取不到清单（判定库服务连不上、test-plan 没打完、条数对不上）：这一格本次未跑（退 77），不当通过。
# 管不到的：该不该拒算（要不要二次确认通过）靠主 agent 的二次确认；挂特性的测试文件、产品 GPU 包与别的包里的 #[test] 的拒算不取。
cell_refused_tests() {
  local -a sample_plan=()
  if [[ -f "$ROOT/.refused-tests-listing.tsv" && "$(pwd -P)" != "$STAGE_CELLS_STAGE_REPOSITORY" && -z "${GATE_STAGED_FROM:-}" ]]; then
    sample_plan=(--plan "$ROOT/.refused-tests-listing.tsv")
  fi
  python3 "$STAGE_REPOSITORY/scripts/gate/refused-test-registry.py" check "$ROOT" ${sample_plan[@]+"${sample_plan[@]}"}
}

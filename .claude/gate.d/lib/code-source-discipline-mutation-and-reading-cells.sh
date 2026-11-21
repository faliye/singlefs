#!/usr/bin/env bash
# 门禁 code-source-discipline 的格函数库：mutation-tables、absolute-assertions、reading-discipline。
# 由 .claude/gate.d/code-source-discipline.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# bad、howto 由入口设好，在调用时才取。

# ── 格 mutation-tables：每个实验二进制都要有变异表
#
# research 那侧：scripts/experiment-apparatus/ 下每份实验装置在 scripts/mutation/tables/ 下有同名 .tsv，至少一条成形的变异
# （三段制表符分隔）、没有坏行；每条原文（\n 还原成换行）在源码里恰好命中一次（mutate.sh 把命中 0 次或多次的那一条记「⚓ 锚点失效」、别的照跑，整轮退 3）；
# 一份实验装置是一个单元：顶层的 <名>.rs，或装置目录 <名>/（根是 <名>/main.rs，子模块逐层往下，名叫 target 的子目录不进）；表名是单元名，
# 装置目录的原文命中数是目录下全部 .rs 的合计；顶层的 lib.rs、main.rs 是 crate 根，不是装置；扫到 0 个单元退 77（不算绿）。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=apparatus-directories-unscanned 退回只扫顶层的 <名>.rs：装置目录不进单元，缺表与锚点不对的行看不见：
#   样本 mutation-tables-red 里装置目录那三处（e13 没有变异表、e14 原文命中 0 次、e15 原文在两份子模块里各命中一次）找不到，
#   mutation-tables-green 里两个装置目录（e11 的锚点落在根、子模块、第二层子模块，e12 只有根）不进单元，「✓ 6 个实验二进制…8 条变异」那一行没有，两份样本都判错。
# 原文与替换文里只许 \n 这一种反斜杠转义。没有同名二进制的表（shell 探针的）不判锚点，成功行逐个列名。
# 重复行：每张表（有没有同名二进制都判）里除第一列变异名外各列逐字全同的几行判红，逐组列表名、行号与变异名（一处改法写了几行，条数看着多、真改法没多）；
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=research-duplicates-unchecked 不判这一样：red 样本 e1_sample.tsv 第 3、4 行那一组找不到。
# 为什么：kb 里每一句「N 条变异全部被抓」都靠这些表复现。
#
# crates/mutations.tsv（门禁 crates-mutation-replay 的表，六段）：那一道要跑几个钟头，改 crates/ 的实现员不跑它，
# 它派活之前判的几样在这里先静态判。判法只有一份，在 scripts/mutation/crates-mutation-rows.py 的 row_problems，这里经 static-check 调：
#   成形：六段；替换文可以空（删掉原文），其余五段不许空；
#   锚点：原文的 \n 还原成换行后，在「文件」那一段指的源码里恰好命中一次，文件不在、读不了也算（那一道把这样的行记「锚点失效」、别的行照判）；
#   测试名：必须红的测试名是一个用例名（不带正则符号与空白）；
#   转义：原文与替换文里只许 \n 这一种反斜杠转义（别的按字面写进源码，那条变异编不过、等于没跑）；
#   拷贝范围 ①：某一行改的文件不在 .claude/gate.d/stage-inputs.tsv 里那一道登记的路径之内（它只把这些路径拷进每片的源码副本）；
#   拷贝范围 ②：crates/ 下 .rs 里以 "../ 起头、解析之后逃出 crates/、又不在拷贝范围里的字符串字面量
#     （紧跟在 include_str!(、include_bytes!( 后面的字面量照源文件所在目录解析，别的照所在 crate 的根解析；#[path = …] 与注释里的不算）：运行时读它的测试在副本里没改坏就红，那一道记「基线不绿」。
#     管不到：路径拼在变量里、环境变量给的、format! 拼出来的、运行期再 join("..") 的；指到仓根本身的（"../../"）按不在范围里判。
#   替换文引的标识符：static-check 的 UNDEFINED 行（替换文施加之后新出现、被改文件与它所在的包的 .rs 里都一处没有定义的标识符，判法在
#     scripts/mutation/crates-mutation-rows.py 的 replacement_undefined_identifiers）判红：那条变异套上去编不过，crates-mutation-replay 记无效、整道红。
#     成功句报 IDENTIFIERS_CHECKED 那一行的两个数（判了几行、没判几行：文件不是 .rs 或锚点失效）。
#     弄坏开关 GATE_MUTATION_TABLES_BREAK=undefined-identifiers-ignored 不读 UNDEFINED 行：样本 mutation-tables-undefined-identifier-red（替换文引了哪里都没有的 halved_before）判绿、样本判错。
#   点名的测试够得着被改坏的函数：static-check 一律判（判法在 scripts/mutation/crates-mutation-rows.py 文件头「点名测试够不着被改坏的函数」，没有开关参数）。
#     UNREACHABLE（点名测试的闭包里根本没有被改坏的单元，或开 GPU 时被改坏的单元只在 GPU 路上不走的 CPU 判器入口后面：改了它测试照样过）与
#     REACHABILITY_UNJUDGED（判不了够不够得着：锚点失效、拒算、闭包算不成）进红句；REACHABILITY_CHECKED 进成功句（判了几行、没判几行、豁免几行、截过闭包几行、开没开 GPU）。
#     选路表、截断表、豁免表点名的函数在这棵树里一个都没有（门禁样本那种小根）时 static-check 打 REACHABILITY_NOT_JUDGED：成功句写「本次未判，不算判过」，不红；
#     有的在、有的不在是配置错（ERROR，static-check 退 2，整格没判完）。真仓上这一半的结论随多机配置（ENABLE_GPU）变，GPU 没开的机器只判 closure 那一路。
#     弄坏开关 GATE_MUTATION_TABLES_BREAK=reachability-lines-unread 不读这几行：样本 mutation-tables-reachability-red（第 3 行改的函数没有测试调到）判绿、
#     mutation-tables-reachability-green 成功句里没有可达性那半句，两份样本都判错。
#     样本 mutation-tables-reachability-green：照 mutation-tables-green 拷来，crates/mutations.tsv 换成一行（改 counted，点名的测试调得到它），
#     crates 下加 singlefs-verification 一个小包，放着选路表、截断表、豁免表点名的那几个函数（空函数体；函数落在表里点名的文件上：crash_amplification/ 与 crash_state_enumeration/ 下的子模块文件，
#     由 crash_amplification.rs、crash_state_enumeration.rs 与 lib.rs 的 mod 声明引到），所以这一格不是「本次未判」；
#     mutation-tables-reachability-red 是它的整份拷贝，变异表多第 3 行（改 never_called_by_the_test，点名的测试调不到它：closure）。
#   重测试目标只许 release：static-check 的 RELEASE 行（命中重测试目标而第五段没写 --release / --profile release 的行，判法在 scripts/mutation/lib_release_only_tests.py，
#     命中的两样是 gpu 特性与 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行登记的测试目标；登记表读本模块所在那一份仓的，样本根里没有）逐行进红句，点名行号、变异名与依据；
#     成功句报「命中重测试目标的行第五段都写了 --release（判了 N 行」，N 是表的行数（static-check 每行都判）。登记表读不了 static-check 打 ERROR、退 2，照「没跑完」判红。
#     弄坏开关 GATE_MUTATION_TABLES_BREAK=release-lines-unread 不读 RELEASE 行：样本 mutation-tables-release-red 判绿、mutation-tables-release-green 成功句里没有 release 那半句，两份样本都判错。
#     样本 mutation-tables-release-green：照 mutation-tables-green 拷来，crates/mutations.tsv 多第 8 行（-p demo --features gpu --release --lib，点名 lib_case）；
#     mutation-tables-release-red 是同一份拷贝，第 8 行第五段不写 --release（-p demo --features gpu --lib），只这一样判红。
# 另判四样那一道不判的：重复行——「文件、原文、替换文」三样逐字全同的几行判红（点名的测试、cargo 参数不同也算：一处改法留一行），
#   逐组列行号与变异名，组里 cargo 参数也全同的另注一句；变异名相同的两行另判（按名字分格的报告把两条并成一格）。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=duplicate-key-with-test 把点名的测试也算进重复的键：red 样本第 13、14 行那一组（点名的测试不同）找不到；
# 滞后表：两侧的重复行里还不能删的组（有装置或名单按名字引着、臂表与键表按臂或键登记）登记在 .claude/gate.d/mutation-duplicates-lag.tsv（写法在它的表头），
#   登记的组不判红，成功句前逐组列名、成功句报「重复行滞后 N 组」。判红的：一行不是四列或有空格子；表那一列不是 crates/mutations.tsv、也不是 scripts/mutation/tables/<表名>.tsv，
#   或那张表不在；名字在那张表第一列里找不到；名字此刻不是同一组（三样不全同 / 除变异名外各列不全同）；那一组里还有没登记的名字；名字在表里不唯一、对上不止一组；
#   两行罩同一组；只缩不涨——此刻的一行在基准那一版（scripts/gate/changed-paths.sh 的 gate_diff_base gate）同一张表里找不到名字是它超集的一行。
#   基准里还没有这张表时不比、成功句写明；git 取不到基准那一版判红。名字改了也算涨（按名字认组）。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=duplicates-lag-ignored 不读滞后表：mutation-tables-duplicates-lag-green 登记的两组照红，mutation-tables-duplicates-lag-red 里滞后表那四处不对也不报，两份样本都判错；
#   duplicates-lag-growth-unchecked 不和基准比：mutation-tables-duplicates-lag-red 里比基准多登记的那一组不红，mutation-tables-duplicates-lag-green 成功句里「没涨」那半句也不打，两份样本都判错。
#   样本 mutation-tables-duplicates-lag-green：crates 表与 e1_sample.tsv 各一组重复、滞后表两行罩住、基准那一版与此刻相同；
#   mutation-tables-duplicates-lag-red：基准那一版只登记 crates 那一组，此刻多登记 e1_sample.tsv 那一组（涨了），另有一行名字找不到、一行名字不是同一组、一行漏登记组里一个名字。
# 包的认法（#[ignore]、筛选词、点名的测试在不在选的目标里这三样都先认包）：cargo 参数在 `--` 之前点名的全部包，每个认成一个包目录，下面三样在包目录的 src/ 与 tests/ 里判：
#   -p <包>、--package <包>（空格隔开或写 =）只认工作区成员：成员从仓根 Cargo.toml 的 [workspace] members 现读（通配照 glob 展开，exclude 的不算），
#     包名取各成员目录的 Cargo.toml 的 [package] name，目录名与包名不必相同；点到不是成员的（例：被 exclude 掉的独立工程 prune-server）判红，出路写 --manifest-path。
#   --manifest-path <路径>（空格隔开或写 =）取那份 Cargo.toml 所在的目录当包目录，路径从仓根算起；解到仓根之外、文件名不是 Cargo.toml、文件不在的判红。
#   同一行既带 -p 又带 --manifest-path、带两次 --manifest-path、--manifest-path 后面没写路径的判红，那一行不再往下判。
#   成功句报两个数：-p 点名的包解出目录的行数、--manifest-path 解出目录的行数。
#   弄坏开关与各自的样本见文件头开关表（package-directory-from-name 等九条）：样本 mutation-tables-manifest-path-green 加了 Cargo.toml 工作区（成员 crates/demo、目录名与包名不同的 crates/renamed-dir），
#   与被 exclude 掉的独立工程 crates/standalone（自己的 Cargo.toml 带空的 [workspace]；测试在它的 src/ 与 tests/ 里），第 7 至 9 行判绿：--manifest-path 空格写法与等号写法各一行、-p renamed-package 一行；
#   mutation-tables-manifest-path-red 是它的整份拷贝，多第 11 至 18 行各判红一样：-p 点非成员、路径在仓根之外、文件名不是 Cargo.toml、文件不在、-p 与 --manifest-path 并写、--manifest-path 写两次、后面没写路径、
#   独立工程里找不到点名的测试。别的 mutation-tables 样本各加了一份只列自己用到的包的工作区 Cargo.toml 与成员的 Cargo.toml。
# #[ignore]、筛选词、点名的测试在不在选的目标里这三样的包取 cargo 参数在 `--` 之前点名的全部包（-p、--package，空格隔开或写 =；--manifest-path 同理）：一行写了几个 -p，点名的测试落在其中任一个包里就算在。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=one-package-per-row 只认最靠前的那一个 -p：样本 mutation-tables-ignore-comments-and-packages-green
#   第 9 行（-p demo -p retry --lib，点名的测试只在 retry 里）判红、整格退 1，mutation-tables-ignore-comments-and-packages-red 第 11 行的报告里没有 retry 那一个包。
# 点名的测试标了 #[ignore] 而参数里没有 --include-ignored：只认 fn 上面的属性行（每行去掉行尾 // 起的注释），夹在属性之间的注释行照样认得出下面的 fn，
#   但注释行与行尾注释里写的 #[ignore] 不算标了；管不到：属性的字符串值里写的 #[ignore]、跨行的属性。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=comment-lines-as-attributes 注释行也算属性：样本 mutation-tables-ignore-comments-and-packages-green
#   第 8 行（点名的 commented_ignore_case 只在注释里写着 #[ignore]）判红、整格退 1；
# `--` 之后的筛选词一个都筛不到点名的测试（libtest 比的是完整测试名：模块路径由点名的函数所在文件在目标模块树里的位置、再加它套在哪几层内联 mod 里算出；
#   目标模块树：从目标根（src/lib.rs、src/main.rs、src/bin/<名>.rs、src/bin/<名>/main.rs、tests/<名>.rs、tests/<名>/main.rs，加包的 Cargo.toml 里 [lib]、[[bin]]、[[test]] 写的 path）出发，
#   顺 `mod 名;` 往下找文件（认 #[path = "…"] 与内联 mod 里的 `mod 名;`；根与 mod.rs 的子模块找它所在的目录，别的 <名>.rs 的子模块找 <名>/），一个文件被几个目标引到、
#   或以几个名字引到就有几个前缀、各算一个全名；没被任何目标根引到的文件退回只看位置猜（src/ 下照文件路径，lib.rs、main.rs、src/bin/ 下的是空，别处空）。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=test-target-module-tree-unwalked 一律退回只看位置猜：tests/<名>/ 与 src/bin/<名>/ 下子模块文件里的用例全名少了前缀，
#   样本 mutation-tables-test-target-modules-green 里按真实全名写 --exact 的行筛不到、整格退 1，mutation-tables-test-target-modules-red 里写裸名的行反而筛得到、不红，两份样本都判错。
#   配 `mod 名 {` 与花括号之前先把注释、字符串、原始串与字符字面量抹成空白，串里的 `mod tests {` 与落单的花括号不算。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=inline-module-literals-counted 退回直接扫原文：样本 mutation-tables-inline-module-literals-green 的 banner.rs 里
#   串里的「mod tests { {」与字符字面量 '{' 把顶层的测试算成 banner::tests::…，按 --exact 点名的 banner::banner_length_counts_the_braces 筛不到、整格退 1）；
# 点名的测试在 cargo 参数选的目标里找不到（--lib：src/ 除 src/bin/ 与 src/main.rs；--bin <名>：src/bin/<名>.rs 与 src/bin/<名>/；
#   --test <名>：tests/<名>.rs、tests/<名>/（目录式目标）与它们 mod 进来的 tests/ 下文件（目录式目标里 main.rs 与子模块 #[path] 引的共用模块目录同样跟）；都没写或 --tests、--all-targets 这一类：src/ 与 tests/ 全部），
#   按 scripts/gate/lib_function_units.py 的 rust_lex 切词，名字是其中一个标识符就算找得到（宏展开出来的测试名在调用处是标识符，认得出；
#   只在注释、字符串里出现的不算）。测试改了名、删了，那一道照这一行跑时点名的测试不在输出里，变异永远报没红。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=named-test-unsearched 不判这一样：red 样本「点名的测试不存在」那几条找不到；
#   named-test-target-ignored 不看选的目标、整个包里找：red 样本里 --lib 点名只在 tests/ 里的 slow_heavy_case 那一条找不到；
#   named-test-definitions-only 只认 `fn <名>` 的定义：green 样本里宏展开出来的 generated_by_a_macro_case 那一行判红、整格退 1。
#
# 这一格不跑测试、不判抓不抓得到（逐条施加替换文、照跑测试，归 research-build-and-replay 的 mutation-tables 格），只保证装置在、形状对。
# research 那侧「表今天还会不会红」靠改实验代码时手跑 scripts/mutation/mutate.sh，门禁不判：C40（变异表没存）。
#
# 样本：red 里 research 那侧有一份没有同名表的实验二进制（e2_naked）、一条锚点命中 0 次、一条替换文带 \& 的、一张不是 UTF-8 的表（setup.sh 现造）、
# 除变异名外各列全同的两行（e1_sample.tsv 第 3、4 行）；
# crates/mutations.tsv 那侧有锚点腐化、转义写进源码、文件列是目录、点名标了 #[ignore] 的用例没带开关、两种筛选词对不上、文件在拷贝范围外、
# 测试名带符号、测试名空着、点名的测试不存在、--lib 点名只在 tests/ 里的测试、同一处改法点名两条不同测试的两行（后五样 setup.sh 现写）各一行，另有一份逃出 crates/ 的相对路径字面量；
# green 是同形的一套都对得上的（含替换文空着的删行变异、读 litmus/ 的测试、点名宏展开出来的测试），每个实验二进制各有一张锚点唯一的表。
# mutation-tables-test-target-modules-green 是 mutation-tables-inline-module-literals-green 的整份拷贝，多目录式测试目标 crates/demo/tests/split_target/（main.rs 引子模块 nested_cases、嵌套的 deeper、
# #[path] 引的共用模块 common_split_shared）与目录式二进制 crates/demo/src/bin/split_tool/（main.rs 引子模块 checks，用例在 checks 的内联 mod tests 里），crates/mutations.tsv 多四行判绿的，
# 筛选词都写目标模块树里的真实全名加 --exact（nested_cases::deeper::deeper_case、common_split_shared::shared_case、checks::tests::tool_case，目标根文件里的 root_case 写裸名）；
# mutation-tables-test-target-modules-red 是它的整份拷贝，多两行判红的：--exact 写裸名 deeper_case 与 tool_case（用例已经挪进子模块，全名多了前缀，筛不到）。
# mutation-tables-undefined-identifier-red 是 green 的整份拷贝，crates/mutations.tsv 多一行替换文引了哪里都没有定义的 halved_before（第 8 行），只这一样判红。
# mutation-tables-ignore-comments-and-packages-green 是 green 的整份拷贝，多一个包 crates/retry 与 crates/demo/tests/commented.rs，crates/mutations.tsv 多两行判绿的：
# 第 8 行点名注释里写着 #[ignore]、本身没标的 commented_ignore_case，不带 --include-ignored；第 9 行 -p demo -p retry --lib 点名只在 retry 里的测试。
# mutation-tables-ignore-comments-and-packages-red 是它的整份拷贝，多 crates/demo/tests/interleaved.rs 与两行判红的：第 10 行点名真标了 #[ignore]、
# 注释行夹在属性之间的 interleaved_slow_case，不带 --include-ignored；第 11 行 -p demo -p retry --lib 点名两个包里都没有的测试。
cell_mutation_tables() {
BIN_DIR=scripts/experiment-apparatus
MUT_DIR=scripts/mutation/tables
[[ -d "$BIN_DIR" && -d "$MUT_DIR" ]] || { echo "  ! 找不到 $BIN_DIR 或 $MUT_DIR，这一格跳过"; exit 77; }

# 一份实验装置是一个单元：顶层的 <名>.rs，或装置目录 <名>/（根是它底下的 main.rs，子模块逐层往下）；单元名就是变异表的表名
missing=(); malformed=(); apparatus_units=(); directory_units=0
for src in "$BIN_DIR"/*.rs "$BIN_DIR"/*/main.rs; do
  [[ -f "$src" ]] || continue   # 没有匹配时 glob 原样留着，不是一份实验
  case "$src" in "$BIN_DIR"/lib.rs|"$BIN_DIR"/main.rs) continue ;; esac   # crate 根的库与主二进制不是一份实验
  if [[ "$(basename "$src")" == main.rs ]]; then
    [[ "${GATE_MUTATION_TABLES_BREAK:-}" == *apparatus-directories-unscanned* ]] && continue
    stem="$(basename "$(dirname "$src")")"
    [[ "$stem" == target ]] && continue
    directory_units=$((directory_units+1))
  else
    stem="$(basename "$src" .rs)"
  fi
  apparatus_units+=("$stem")
  tsv="$MUT_DIR/$stem.tsv"
  if [[ ! -f "$tsv" ]]; then missing+=("$tsv 不存在（被测的是 $src）"); continue; fi
  # 至少一条成形的变异行：非注释、非空、恰好三段
  ok_rows=$(awk -F'\t' '!/^#/ && NF==3 && $1!="" && $2!="" {n++} END{print n+0}' "$tsv")
  bad_rows=$(awk -F'\t' '!/^#/ && NF!=3 && $0!="" {n++} END{print n+0}' "$tsv")
  if [[ "$ok_rows" -eq 0 || "$bad_rows" -gt 0 ]]; then malformed+=("$tsv：成形 $ok_rows 条、坏 $bad_rows 行"); fi
done

# ── research 变异表的锚点与转义：只做子串计数，不跑变异 ──
anchor_report="$(BIN_DIR="$BIN_DIR" MUT_DIR="$MUT_DIR" python3 - <<'PY'
import os, glob, re
bin_dir = os.environ["BIN_DIR"]; mut_dir = os.environ["MUT_DIR"]
bad = []; stray = []; unreadable = []; duplicates = []; checked = 0; tables_read = 0
check_duplicates = "research-duplicates-unchecked" not in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
scan_apparatus_directories = "apparatus-directories-unscanned" not in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
def unit_sources(stem):
    """一份实验装置的全部源文件：顶层的 <名>.rs（在就算，目录与读不了的留给下面判红），加上带根 main.rs 的装置目录 <名>/ 底下逐层往下的 .rs（名叫 target 的子目录不进）。"""
    paths = []
    single = os.path.join(bin_dir, stem + ".rs")
    if os.path.exists(single):
        paths.append(single)
    directory = os.path.join(bin_dir, stem)
    if scan_apparatus_directories and os.path.isfile(os.path.join(directory, "main.rs")):
        for current, subdirectories, names in os.walk(directory):
            subdirectories[:] = sorted(name for name in subdirectories if name != "target")
            paths.extend(os.path.join(current, name) for name in sorted(names) if name.endswith(".rs"))
    return paths
def stray_escapes(segment_name, segment):
    """mutate.sh 与 crates-mutation-replay 只把 \\n 还原成换行，别的反斜杠按字面写进源码（C427）。"""
    return [(segment_name, match.group(0)) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]
def duplicate_groups(stem, table_lines):
    """一张表里除第一列变异名外各列逐字全同的几行：每组一项 (表名, 「第 a、b 行（名；名）」, 组内各行的变异名用 \\x1f 连起来)。注释行与空行不算。"""
    groups = {}
    for lineno, line in enumerate(table_lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t")
        groups.setdefault(tuple(parts[1:]), []).append((lineno, parts[0]))
    return [(stem, "第 " + "、".join(str(lineno) for lineno, _name in rows) + " 行（" + "；".join(name for _lineno, name in rows) + "）",
             "\x1f".join(name for _lineno, name in rows))
            for rows in groups.values() if len(rows) > 1]
for tsv in sorted(glob.glob(os.path.join(mut_dir, "*.tsv"))):
    stem = os.path.basename(tsv)[:-4]
    source_paths = unit_sources(stem)
    # 重复行每张表都判（有没有同名二进制都算）；读不了的表在下面锚点那一段判红，没有同名二进制的读不了也判红
    try:
        table_lines = open(tsv, encoding="utf-8").read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        table_lines = None
        if not source_paths:
            unreadable.append((stem, f"{type(error).__name__}: {error}"))
    if table_lines is not None:
        tables_read += 1
        if check_duplicates:
            duplicates += duplicate_groups(stem, table_lines)
    # 没有同名二进制的表（shell 探针的变异表）不在锚点的射程：它们的被测对象不是 .rs；成功行逐个列名
    if not source_paths:
        print("NO_BINARY", tsv, sep="\t")
        continue
    # 读不了（不是 UTF-8、是个目录、没权限）的一份判红、接着判下一份：崩在这里会让整段报告一行不剩
    try:
        sources = [open(source_path, encoding="utf-8").read() for source_path in source_paths]
        table_lines = open(tsv, encoding="utf-8").read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        unreadable.append((stem, f"{type(error).__name__}: {error}"))
        continue
    for lineno, line in enumerate(table_lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 3:
            continue
        name, frm, to = parts
        checked += 1
        # 口径与 mutate.sh 一致：表里的 \n 先还原成换行，再数子串
        anchor = frm.replace("\\n", "\n")
        hits = sum(source.count(anchor) for source in sources)   # 装置目录：命中数是各份源文件的合计，原文跨不过两份文件
        if hits != 1:
            bad.append((stem, name, lineno, hits))
        for segment_name, segment in (("原文", frm), ("替换文", to)):
            for found in stray_escapes(segment_name, segment):
                stray.append((stem, name, lineno) + found)
print("CHECKED", checked)
print("TABLES_READ", tables_read)
for d in duplicates:
    print("DUP", *d, sep="\t")
for b in bad:
    print("BAD", *b, sep="\t")
for s in stray:
    print("STRAY", *s, sep="\t")
for u in unreadable:
    print("UNREADABLE", *u, sep="\t")
PY
)"
anchor_exit_code=$?
# ── crates/mutations.tsv 只在这里判的三样：两行重复、#[ignore]、筛选词；行经共用模块的 read_table 读（成形的口径只有它一份）──
MUTATION_ROWS_MODULE="$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py"
crates_report=""
if [[ -f crates/mutations.tsv ]]; then
  crates_report="$(MUTATION_ROWS_MODULE="$MUTATION_ROWS_MODULE" python3 - <<'PY'
import glob
import importlib.util
import os
import re
import sys
import tomllib
sys.dont_write_bytecode = True
specification = importlib.util.spec_from_file_location("crates_mutation_rows", os.environ["MUTATION_ROWS_MODULE"])
mutation_rows = importlib.util.module_from_spec(specification)
specification.loader.exec_module(mutation_rows)
first_line_of_change = {}
first_line_of_name = {}
attributes_by_package = {}
def path_based_module_path(package_directory, path):
    """只看位置猜的 libtest 测试名模块前缀：src/lib.rs、src/main.rs 与 src/bin/ 下的是空，src/a.rs 与 src/a/mod.rs 是 a，src/a/b.rs 是 a::b；tests/ 下的是空。
    没被任何目标根引到的文件用它当退路；弄坏开关 GATE_MUTATION_TABLES_BREAK=test-target-module-tree-unwalked 下所有文件都用它。"""
    relative = os.path.relpath(path, package_directory)
    parts = relative.split(os.sep)
    if parts[0] != "src" or parts[1:2] == ["bin"] or parts[1:] in (["lib.rs"], ["main.rs"]):
        return []
    parts = parts[1:]
    parts[-1] = parts[-1][:-len(".rs")]
    if parts[-1] == "mod":
        parts = parts[:-1]
    return parts
# 配花括号之前先抹掉的几样：行注释、块注释（可嵌套，找到开头再往后数）、原始串（找到开头再找收尾）、普通串与字符字面量
MASKED_START = re.compile(r"""//[^\n]*|/\*|(?<![A-Za-z0-9_])b?r(?P<hashes>#*)"|b?"(?:\\.|[^"\\])*(?:"|\Z)|b?'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]{1,6}\}|[^\n])|[^\\'\n])'""", re.S)
def masked_comments_and_literals(text):
    """注释、字符串、原始串与字符字面量抹成同样长度的空白（换行留着），偏移不变：配花括号与找 `mod 名 {` 只看代码。"""
    pieces, position = [], 0
    while True:
        found = MASKED_START.search(text, position)
        if not found:
            return "".join(pieces) + text[position:]
        end = found.end()
        if found.group(0) == "/*":
            depth = 1
            while end < len(text) and depth:
                depth += 1 if text.startswith("/*", end) else -1 if text.startswith("*/", end) else 0
                end += 2 if text.startswith(("/*", "*/"), end) else 1
        elif found.group("hashes") is not None:
            closing = '"' + found.group("hashes")
            close_at = text.find(closing, end)
            end = len(text) if close_at < 0 else close_at + len(closing)
        pieces += [text[position:found.start()], re.sub(r"[^\n]", " ", text[found.start():end])]
        position = end
def enclosing_inline_modules(text):
    """(偏移, 那一刻套着的内联 mod 名单) 的有序表：按 `mod 名 {` 与花括号配平；注释、字符串、原始串与字符字面量先抹掉再配
    （弄坏开关 GATE_MUTATION_TABLES_BREAK=inline-module-literals-counted 下直接扫原文，串里的 `mod tests {` 与落单花括号照配）。"""
    events, stack, depth = [], [], 0
    if "inline-module-literals-counted" not in os.environ.get("GATE_MUTATION_TABLES_BREAK", ""):
        text = masked_comments_and_literals(text)
    for token in re.finditer(r"\bmod\s+(\w+)\s*\{|[{}]", text):
        if token.group(1):
            depth += 1
            stack.append((token.group(1), depth))
        elif token.group(0) == "{":
            depth += 1
        else:
            if stack and stack[-1][1] == depth:
                stack.pop()
            depth -= 1
        events.append((token.end(), [name for name, _ in stack]))
    return events
# 外联 `mod 名;`（前面可以带属性与可见性）与花括号：配花括号只为知道一个 `mod 名;` 套在哪几层内联 mod 里
MODULE_ITEM = re.compile(r"((?:#\s*\[[^\]]*\]\s*)*)(?:pub(?:\s*\([^)]*\))?\s+)?(?<![A-Za-z0-9_])mod\s+(\w+)\s*([;{])|[{}]")
PATH_ATTRIBUTE = re.compile(r'#\s*\[\s*path\s*=\s*"((?:\\.|[^"\\])*)"\s*\]')
declared_modules_by_path = {}
def declared_modules(path):
    """一份 .rs 里的外联 `mod 名;`：[(名, #[path = "…"] 写的路径或 None, 它套着的内联 mod 名单)]；内联的 `mod 名 { … }` 不出项，只记进里面那些声明套着的名单。
    注释、字符串、原始串与字符字面量先抹掉再配（偏移不变，#[path] 的值从原文取）；读不了的文件交空。"""
    if path not in declared_modules_by_path:
        try:
            text = open(path, encoding="utf-8").read()
        except (OSError, UnicodeDecodeError):
            text = ""
        found, stack, depth = [], [], 0
        for token in MODULE_ITEM.finditer(masked_comments_and_literals(text)):
            if token.group(2) is None:
                if token.group(0) == "{":
                    depth += 1
                else:
                    if stack and stack[-1][1] == depth:
                        stack.pop()
                    depth -= 1
            elif token.group(3) == "{":
                depth += 1
                stack.append((token.group(2), depth))
            else:
                attribute = PATH_ATTRIBUTE.search(text[token.start(1):token.end(1)])
                found.append((token.group(2), attribute.group(1) if attribute else None, tuple(name for name, _ in stack)))
        declared_modules_by_path[path] = found
    return declared_modules_by_path[path]
def target_roots(package_directory):
    """包目录里 cargo 认的目标：[(种类, 名, 根源文件)]，种类是 lib、bin、test。认 src/lib.rs（lib）、src/main.rs（bin，名是包名）、src/bin/<名>.rs 与 src/bin/<名>/main.rs（bin）、
    tests/<名>.rs 与 tests/<名>/main.rs（test），加包的 Cargo.toml 里 [lib]、[[bin]]、[[test]] 的 path 写的那几份（名取 name，没写取文件名或 lib）。"""
    try:
        with open(os.path.join(package_directory, "Cargo.toml"), "rb") as handle:
            manifest = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError):
        manifest = {}
    roots = []
    for name in ("lib.rs", "main.rs"):
        path = os.path.join(package_directory, "src", name)
        if os.path.isfile(path):
            roots.append(("lib", None, path) if name == "lib.rs" else ("bin", manifest.get("package", {}).get("name"), path))
    for kind, directory in (("bin", os.path.join(package_directory, "src", "bin")), ("test", os.path.join(package_directory, "tests"))):
        for entry in sorted(os.listdir(directory)) if os.path.isdir(directory) else []:
            path = os.path.join(directory, entry)
            if entry.endswith(".rs") and os.path.isfile(path):
                roots.append((kind, entry[:-len(".rs")], path))
            elif os.path.isfile(os.path.join(path, "main.rs")):
                roots.append((kind, entry, os.path.join(path, "main.rs")))
    for kind, sections in (("lib", [manifest.get("lib", {})]), ("bin", manifest.get("bin", [])), ("test", manifest.get("test", []))):
        for section in sections:
            if isinstance(section, dict) and isinstance(section.get("path"), str):
                path = os.path.normpath(os.path.join(package_directory, section["path"]))
                if os.path.isfile(path):
                    roots.append((kind, None if kind == "lib" else section.get("name", os.path.basename(path)[:-len(".rs")]), path))
    return list(dict.fromkeys(roots))
module_trees_by_package = {}
def module_trees_of_package(package_directory):
    """{目标根源文件: {源文件: [模块前缀…]}}：从每个目标根出发顺 `mod 名;` 走出它的模块树（认 #[path]、内联 mod 里的外联声明），一个文件在一棵树里以几个名字被引到就有几个前缀。
    找文件照 Rust 的规则：根与 mod.rs 的子模块找它所在的目录（<名>.rs 或 <名>/mod.rs），别的 <名>.rs 的子模块找 <名>/ 下；#[path] 相对声明所在文件的目录（套着内联 mod 时再加上那几层名字），
    经 #[path] 引来的文件按 mod.rs 对待，它的子模块找它所在的目录。每个包目录只走一次。"""
    if package_directory not in module_trees_by_package:
        trees = {}
        for _kind, _name, root in target_roots(package_directory):
            if root in trees:
                continue
            reached, seen, pending = {}, set(), [(root, (), True)]
            while pending:
                path, prefix, owns_directory = pending.pop()
                if (path, prefix) in seen:
                    continue
                seen.add((path, prefix))
                reached.setdefault(path, []).append(list(prefix))
                directory = os.path.dirname(path)
                children_directory = directory if owns_directory else os.path.join(directory, os.path.basename(path)[:-len(".rs")])
                for name, attribute, inline in declared_modules(path):
                    base = os.path.join(children_directory, *inline)
                    if attribute is not None:
                        child, child_owns_directory = os.path.normpath(os.path.join(base if inline else directory, attribute)), True
                    else:
                        options = [(os.path.join(base, name + ".rs"), False), (os.path.join(base, name, "mod.rs"), True)]
                        child, child_owns_directory = next(((option, owns) for option, owns in options if os.path.isfile(option)), (None, False))
                    if child is not None and os.path.isfile(child):
                        pending.append((child, prefix + inline + (name,), child_owns_directory))
            trees[root] = reached
        module_trees_by_package[package_directory] = trees
    return module_trees_by_package[package_directory]
module_tree_unwalked = "test-target-module-tree-unwalked" in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
comment_lines_as_attributes = "comment-lines-as-attributes" in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
def attribute_code(lines_above):
    """fn 上面那一串（属性行与夹在中间的注释行）里只留属性行，每行去掉行尾 `//` 起的注释：注释行（`//`、`///`）与行尾注释里写的 `#[ignore]` 不算标了 ignore
    （弄坏开关 GATE_MUTATION_TABLES_BREAK=comment-lines-as-attributes 下整串原样交回，注释行里的 `#[ignore]` 也算）。"""
    if comment_lines_as_attributes:
        return lines_above
    return "\n".join(line.split("//", 1)[0] for line in lines_above.split("\n") if line.lstrip().startswith("#["))
def test_attributes_of_package(package_directory):
    """包目录的 tests/ 与 src/ 下每个 fn 名 → [(它上面那串属性行（注释去掉，见 attribute_code）, libtest 里的完整测试名, 编它的目标根源文件或 None)]；每个包目录只扫一次。
    一个文件被几个目标引到、或在一个目标里以几个名字引到，同一个 fn 有几项；没被任何目标根引到的文件交一项，目标根是 None。"""
    if package_directory not in attributes_by_package:
        table = {}
        for directory_name in ("tests", "src"):
            for root, _directories, files in os.walk(os.path.join(package_directory, directory_name)):
                for file_name in sorted(files):
                    if not file_name.endswith(".rs"):
                        continue
                    path = os.path.join(root, file_name)
                    try:
                        text = open(path, encoding="utf-8").read()
                    except (OSError, UnicodeDecodeError):
                        continue
                    # 这个文件在哪些（目标根, 模块前缀）下被编：没被任何目标根引到的退回只看位置猜，不属于哪个具体目标（根交 None）
                    placements = [] if module_tree_unwalked else [(target_root, tuple(prefix)) for target_root, reached in module_trees_of_package(package_directory).items()
                                                                  for prefix in reached.get(path, [])]
                    placements = placements or [(None, tuple(path_based_module_path(package_directory, path)))]
                    events = enclosing_inline_modules(text)
                    for found in re.finditer(r"((?:[ \t]*#\[[^\n]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(", text):
                        modules = []
                        for offset, stack in events:
                            if offset > found.start(2):
                                break
                            modules = stack
                        for target_root, prefix in placements:
                            table.setdefault(found.group(2), []).append((attribute_code(found.group(1)), "::".join(list(prefix) + modules + [found.group(2)]), target_root))
        attributes_by_package[package_directory] = table
    return attributes_by_package[package_directory]
# 点名的测试在选的目标里找不找得到：切词用 lib_function_units.py 的 rust_lex（与单元切法同一份词法，不另写一份去注释、去字符串）
named_test_break = os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
function_units_specification = importlib.util.spec_from_file_location(
    "lib_function_units", os.path.join(os.path.dirname(os.environ["MUTATION_ROWS_MODULE"]), "..", "gate", "lib_function_units.py"))
function_units = importlib.util.module_from_spec(function_units_specification)
function_units_specification.loader.exec_module(function_units)
words_by_path = {}
def words_of(path):
    """一份 .rs 里的标识符集合与紧跟在 fn 后面的标识符集合（注释、字符串、字符字面量里的不算）；读不了的两样都空。"""
    if path not in words_by_path:
        try:
            kinds, texts = function_units.rust_lex(open(path, encoding="utf-8").read())[:2]
        except (OSError, UnicodeDecodeError):
            kinds, texts = [], []
        identifiers = {text for kind, text in zip(kinds, texts) if kind == function_units.IDENT}
        defined = {texts[index + 1] for index in range(len(texts) - 1)
                   if kinds[index] == function_units.IDENT and texts[index] == "fn" and kinds[index + 1] == function_units.IDENT}
        modules = [texts[index + 1] for index in range(len(texts) - 2)
                   if texts[index] == "mod" and kinds[index + 1] == function_units.IDENT and texts[index + 2] == ";"]
        words_by_path[path] = (identifiers, defined, modules)
    return words_by_path[path]
def rust_files_under(directory):
    found = []
    for root, _directories, files in os.walk(directory):
        found += [os.path.join(root, name) for name in files if name.endswith(".rs")]
    return sorted(found)
def target_selections(arguments):
    """cargo test 参数在 `--` 之前选的编译目标：[(种类, 名)]，--lib 是 ("lib", None)，--bin、--test 是 ("bin", 名)、("test", 名)，--tests、--all-targets 这一类是 ("all", None)；一样都没写是空表。"""
    before_separator = arguments[:arguments.index("--")] if "--" in arguments else arguments
    selections = []
    for position, word in enumerate(before_separator):
        if word == "--lib":
            selections.append(("lib", None))
        elif word in ("--bin", "--test") and position + 1 < len(before_separator):
            selections.append((word[2:], before_separator[position + 1]))
        elif word.startswith(("--bin=", "--test=")):
            selections.append(tuple(word[2:].split("=", 1)))
        elif word in ("--tests", "--bins", "--all-targets", "--examples", "--benches"):
            selections.append(("all", None))
    return selections
def selected_target_roots(package_directory, arguments):
    """点名的目标（--lib、--bin <名>、--test <名>）在这个包里的目标根源文件的集合；一样都没点名、点了 --tests 这一类、或点名的目标在这个包里找不到根时交 None（不按目标收窄）。
    弄坏开关 GATE_MUTATION_TABLES_BREAK=named-test-target-ignored 与 test-target-module-tree-unwalked 下交 None。"""
    selections = target_selections(arguments)
    if not selections or ("all", None) in selections or "named-test-target-ignored" in named_test_break or module_tree_unwalked:
        return None
    roots = {root for kind, name, root in target_roots(package_directory) if (kind, name if kind != "lib" else None) in selections}
    return roots or None
def selected_target_files(package_directory, arguments):
    """cargo test 参数在 `--` 之前选的编译目标的源文件与一句说明。--lib、--bin、--test 认；都没写或别的选法按 src/ 与 tests/ 全部。"""
    base = package_directory
    selections = target_selections(arguments)
    if not selections or ("all", None) in selections or "named-test-target-ignored" in named_test_break:
        return rust_files_under(os.path.join(base, "src")) + rust_files_under(os.path.join(base, "tests")), f"{package_directory} 的 src/ 与 tests/"
    files, shown = [], []
    for kind, value in selections:
        if kind == "lib":
            files += [path for path in rust_files_under(os.path.join(base, "src"))
                      if not path.startswith(os.path.join(base, "src", "bin") + os.sep) and path != os.path.join(base, "src", "main.rs")]
            shown.append("--lib")
        elif kind == "bin":
            files += [os.path.join(base, "src", "bin", value + ".rs")] + rust_files_under(os.path.join(base, "src", "bin", value))
            shown.append(f"--bin {value}")
        else:
            target_files = [os.path.join(base, "tests", value + ".rs")] + rust_files_under(os.path.join(base, "tests", value))
            files += target_files
            for module in [name for path in (target_files[:1] if "directory-target-modules-unread" in named_test_break else target_files) for name in words_of(path)[2]]:
                files += [os.path.join(base, "tests", module + ".rs")] + rust_files_under(os.path.join(base, "tests", module))
            shown.append(f"--test {value}")
    return [path for path in dict.fromkeys(files) if os.path.isfile(path)], f"{package_directory} 的 {'、'.join(shown)}"
workspace_members_cache = []
def workspace_members():
    """仓根 Cargo.toml 的 [workspace] members 现读：{包名: 目录（相对仓根）}。members 里的通配照 glob 展开，exclude 里的目录不算，
    包名取各成员目录的 Cargo.toml 的 [package] name（目录名与包名不必相同）。仓根或成员的 Cargo.toml 读不了就抛，这一段 python 崩了，整格按「没跑完」判红。"""
    if not workspace_members_cache:
        with open("Cargo.toml", "rb") as handle:
            workspace = tomllib.load(handle).get("workspace", {})
        excluded = {os.path.normpath(path) for path in workspace.get("exclude", [])}
        members = {}
        for pattern in workspace.get("members", []):
            for directory in sorted(glob.glob(pattern)):
                directory = os.path.normpath(directory)
                if directory in excluded or not os.path.isdir(directory):
                    continue
                with open(os.path.join(directory, "Cargo.toml"), "rb") as handle:
                    package_name = tomllib.load(handle).get("package", {}).get("name")
                if package_name is not None:
                    members.setdefault(package_name, directory)
        workspace_members_cache.append(members)
    return workspace_members_cache[0]
repository_root = os.path.realpath(".")
def manifest_package_directory(path):
    """`--manifest-path <路径>` 指的那份 Cargo.toml 所在的目录（相对仓根）与问题句，两样里恰有一样是 None：路径从仓根算起（cargo 在仓根起），
    解到仓根之外、文件名不是 Cargo.toml、文件不在的回问题句。
    弄坏开关 GATE_MUTATION_TABLES_BREAK 的 manifest-path-outside-repo-unjudged、manifest-path-file-name-unjudged、manifest-path-missing-file-unjudged 各不判其中一样；
    manifest-path-not-a-package 三样都判，但判完不把那个目录当包目录用。"""
    target = os.path.realpath(os.path.join(repository_root, path))
    if not (target == repository_root or target.startswith(repository_root + os.sep)) and "manifest-path-outside-repo-unjudged" not in named_test_break:
        return None, f"--manifest-path {path} 解到仓根之外（{target}）；写仓内独立工程的 Cargo.toml（相对仓根的路径）"
    if os.path.basename(target) != "Cargo.toml" and "manifest-path-file-name-unjudged" not in named_test_break:
        return None, f"--manifest-path {path} 的文件名不是 Cargo.toml；写成指到那个独立工程的 Cargo.toml"
    if not os.path.isfile(target) and "manifest-path-missing-file-unjudged" not in named_test_break:
        return None, f"--manifest-path {path} 指的文件不在（仓根下 {os.path.relpath(target, repository_root)}）；写成那个独立工程的 Cargo.toml 的真实路径"
    if "manifest-path-not-a-package" in named_test_break:
        return None, None
    return os.path.relpath(os.path.dirname(target), repository_root), None
def packages_of(arguments):
    """cargo test 参数在 `--` 之前点名的包 → (包目录列表, 问题句列表, 经 -p 解出的目录, 经 --manifest-path 解出的目录)，目录照写的次序、去重、相对仓根：
      -p <包>、--package <包>、-p=<包>、--package=<包>   只认仓根 Cargo.toml 的 [workspace] members 里的包（workspace_members 现读，目录名与包名不必相同）；
                                                       不是成员的（被 exclude 掉的独立工程、哪里都没有的名字）判问题，出路是 --manifest-path
      --manifest-path <路径>、--manifest-path=<路径>    取那份 Cargo.toml 所在的目录当包目录（manifest_package_directory），独立工程用它点名
    同一行既带 -p 又带 --manifest-path、带两次 --manifest-path、--manifest-path 后面没写路径的判问题；有问题的写法不再往下解目录。
    弄坏开关 GATE_MUTATION_TABLES_BREAK：
      one-package-per-row  只认整串参数里最靠前的那一个 `-p <包>`
      package-directory-from-name  包目录不读仓根 Cargo.toml、按 crates/<包名> 硬拼（-p 点到非成员也照拼）
      non-member-package-unjudged  成员照读，-p 点到非成员的不判问题，退回按 crates/<包名> 硬拼
      package-and-manifest-path-together-allowed、manifest-path-twice-allowed、manifest-path-without-value-unjudged  各不判其中一样"""
    before_separator = arguments[:arguments.index("--")] if "--" in arguments else arguments
    names, manifests = [], []
    for position, word in enumerate(before_separator):
        following = before_separator[position + 1] if position + 1 < len(before_separator) else None
        if word in ("-p", "--package"):
            if following is not None:
                names.append(following)
        elif word.startswith(("-p=", "--package=")):
            names.append(word.split("=", 1)[1])
        elif word == "--manifest-path":
            manifests.append(following)
        elif word.startswith("--manifest-path="):
            manifests.append(word.split("=", 1)[1])
    names = list(dict.fromkeys(names))
    if "one-package-per-row" in named_test_break:
        names = [arguments[arguments.index("-p") + 1]] if "-p" in arguments[:-1] else []
    manifest_words = [f"--manifest-path {path}" if path is not None else "--manifest-path" for path in manifests]
    if names and manifests and "package-and-manifest-path-together-allowed" not in named_test_break:
        return [], [f"同一行既写了 {' '.join('-p ' + name for name in names)} 又写了 {' '.join(manifest_words)}；"
                    "二选一：工作区成员写 -p <包名>，不在工作区的独立工程写 --manifest-path <它的 Cargo.toml>"], [], []
    if len(manifests) > 1 and "manifest-path-twice-allowed" not in named_test_break:
        return [], [f"--manifest-path 写了不止一次（{'、'.join(path or '（没写路径）' for path in manifests)}）；一行只许一个，要测两个独立工程就拆成两行"], [], []
    problems, from_names, from_manifests = [], [], []
    for path in manifests:
        if path is None:
            if "manifest-path-without-value-unjudged" not in named_test_break:
                problems.append("--manifest-path 后面没写路径；写成 --manifest-path <那个独立工程的 Cargo.toml>")
            continue
        directory, problem = manifest_package_directory(path)
        if problem is not None:
            problems.append(problem)
        elif directory is not None:
            from_manifests.append(directory)
    for name in names:
        if "package-directory-from-name" in named_test_break:
            from_names.append(os.path.join("crates", name))
            continue
        members = workspace_members()
        if name in members:
            from_names.append(members[name])
        elif "non-member-package-unjudged" in named_test_break:
            from_names.append(os.path.join("crates", name))
        else:
            problems.append(f"-p {name} 不是工作区成员（仓根 Cargo.toml 的 [workspace] members 里没有叫这个名字的包；成员有：{'、'.join(sorted(members))}）；"
                            "不在工作区的独立工程用 --manifest-path <它的 Cargo.toml>")
    return list(dict.fromkeys(from_names + from_manifests)), problems, from_names, from_manifests
rows, _malformed = mutation_rows.read_table("crates/mutations.tsv")
# 弄坏开关 duplicate-key-with-test：把点名的测试也算进重复的键（同一处改法点名两条不同测试的几行就放过去了）
duplicate_key_with_test = "duplicate-key-with-test" in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
rows_with_member_packages = rows_with_manifest_paths = 0
for row in rows:
    line_number, fields = row.line_number, row.fields
    name, path = row.name, row.path
    # 几行的（文件, 原文, 替换文）逐字相同：同一处改法写了几行，表里的条数看着多、真改法没多；一处改法留一行
    change = (path, fields[2], fields[3]) + ((fields[5],) if duplicate_key_with_test else ())
    first_line_of_change.setdefault(change, []).append((line_number, name, fields[4]))
    # 变异名相同 ⇒ 按名字分格的报告把两条并成一格，其中一条的结论看不见。
    if name in first_line_of_name:
        print("CRATES_DUP", line_number, name, f"变异名与第 {first_line_of_name[name]} 行相同", sep="\t")
    else:
        first_line_of_name[name] = line_number
    # 点名的测试标了 #[ignore]（只留下的不是分档的那几条：要显卡、量显存与速度、要补丁表副本），cargo test 参数里却没有 --include-ignored / --ignored：
    # crates-mutation-replay 与 prove-red.sh 照这一行跑，点名的测试被 libtest 跳过，这条变异永远报「没红」，而它守的那一格其实没人看。
    arguments = fields[4].split()
    # 一行写了几个 -p：点名的测试落在其中任一个包里就算在（筛选词、点名的测试在不在目标里、#[ignore] 三样都按这几个包的并集判）；
    # 包目录：-p 点名的取仓根 Cargo.toml 的工作区成员，--manifest-path 点名的取那份 Cargo.toml 所在的目录；认不出的整行报 CRATES_PACKAGE，那一行不再往下判
    package_directories, package_problems, package_directories_from_names, package_directories_from_manifests = packages_of(arguments)
    for problem in package_problems:
        print("CRATES_PACKAGE", line_number, name, problem, sep="\t")
    rows_with_member_packages += bool(package_directories_from_names)
    rows_with_manifest_paths += bool(package_directories_from_manifests)
    test_name = fields[5].split("::")[-1].strip()
    # cargo test 参数里 `--` 之后的位置参数是 libtest 的筛选词：点名的测试一个都筛不到，那条测试根本不跑，变异永远报没红。
    libtest_arguments = arguments[arguments.index("--") + 1:] if "--" in arguments else []
    filters = [word for word in libtest_arguments if not word.startswith("-")]
    exact = "--exact" in libtest_arguments
    # libtest 拿筛选词去比完整测试名（模块路径 + 函数名）：完整名从点名的函数在哪个文件、套在哪几层内联 mod 里算出来；找不到那个函数的不判。
    # 点名了 --lib、--bin、--test 的，只收编进那些目标的同名函数（别的目标里的同名用例不是这一行跑的那条）；没被任何目标根引到的文件里的同名函数照收
    found_tests = []
    for package_directory in package_directories if test_name else []:
        selected_roots = selected_target_roots(package_directory, arguments)
        found_tests += [(attributes, full_name) for attributes, full_name, target_root in test_attributes_of_package(package_directory).get(test_name, [])
                        if selected_roots is None or target_root is None or target_root in selected_roots]
    candidates = [full_name for _attributes, full_name in found_tests]
    if candidates and filters and not any((full_name == word) if exact else (word in full_name) for word in filters for full_name in candidates):
        print("CRATES_FILTER", line_number, name, f"筛选词 {' '.join(filters)}{'（--exact）' if exact else ''} 筛不到点名的 {' 或 '.join(candidates)}", sep="\t")
    # 点名的测试在选的目标里找不到（改了名、删了）：那一道照这一行跑，点名的测试不在输出里，变异永远报没红
    if package_directories and test_name and "named-test-unsearched" not in named_test_break:
        target_files, target_shown = [], []
        for package_directory in package_directories:
            package_files, package_shown = selected_target_files(package_directory, arguments)
            target_files += package_files
            target_shown.append(package_shown)
        look_in = 1 if "named-test-definitions-only" in named_test_break else 0
        if not any(test_name in words_of(path)[look_in] for path in target_files):
            print("CRATES_MISSING", line_number, name, f"点名的测试 {test_name} 在 {'、'.join(target_shown)}（{len(target_files)} 份 .rs）里找不到", sep="\t")
    if package_directories and test_name and not ({"--include-ignored", "--ignored"} & set(arguments)):
        if found_tests and all("#[ignore" in attributes for attributes, _full_name in found_tests):
            print("CRATES_IGNORED", line_number, name, f"点名的 {test_name} 标了 #[ignore]，cargo test 参数「{fields[4]}」里没有 --include-ignored", sep="\t")
# 重复的改法逐组一行：「第 a、b 行（a 名；b 名）」，组里 cargo 参数也全同的另注一句
for (path, *_rest), group in first_line_of_change.items():
    if len(group) > 1:
        same_arguments = "；这几行的 cargo test 参数也全同" if len({arguments for _line, _name, arguments in group}) == 1 else ""
        print("CRATES_DUP_CHANGE", "第 " + "、".join(str(line) for line, _name, _arguments in group) + " 行（"
              + "；".join(f"{line} {name}" for line, name, _arguments in group) + f"），改的是 {path}{same_arguments}",
              "\x1f".join(name for _line, name, _arguments in group), sep="\t")
print("CRATES_PACKAGES_CHECKED", rows_with_member_packages, rows_with_manifest_paths, sep="\t")
print("CRATES_CHECKED", len(rows))
PY
)"
  crates_exit_code=$?
fi
# ── 门禁 crates-mutation-replay 派活之前判的那几样与拷贝范围：共用模块的 static-check 逐处打
# MALFORMED / ANCHOR / NAME / ROW / STRAY / LITERAL / UNDEFINED / RELEASE，末尾 COPIED、IDENTIFIERS_CHECKED 与 CHECKED；退 0 一处都没有、退 1 有、别的退出码是它自己没判完（ERROR 行是原因）
scope_report=""; scope_exit_code=0
if [[ -f "$MUTATION_ROWS_MODULE" ]]; then
  scope_report="$(python3 "$MUTATION_ROWS_MODULE" static-check "$ROOT")" || scope_exit_code=$?
else
  scope_exit_code=127; scope_report=$'ERROR\t'"找不到 $MUTATION_ROWS_MODULE"
fi
scope_copied="$(sed -n 's/^COPIED\t//p' <<<"$scope_report")"
scope_checked="$(sed -n 's/^CHECKED\t//p' <<<"$scope_report")"
mapfile -t crates_malformed < <(grep $'^MALFORMED\t' <<<"$scope_report")
mapfile -t crates_bad < <(grep $'^ANCHOR\t' <<<"$scope_report")
mapfile -t crates_names < <(grep $'^NAME\t' <<<"$scope_report")
mapfile -t scope_rows < <(grep $'^ROW\t' <<<"$scope_report")
mapfile -t crates_stray < <(grep $'^STRAY\t' <<<"$scope_report")
mapfile -t scope_literals < <(grep $'^LITERAL\t' <<<"$scope_report")
# 替换文引了 crates 里找不到定义的标识符（文件头这一格「替换文引的标识符」）；弄坏开关 undefined-identifiers-ignored：不读这种行
crates_undefined=()
if [[ "${GATE_MUTATION_TABLES_BREAK:-}" != *undefined-identifiers-ignored* ]]; then
  mapfile -t crates_undefined < <(grep $'^UNDEFINED\t' <<<"$scope_report")
fi
# 命中重测试目标而第五段没写 --release（文件头这一格「重测试目标只许 release」）；弄坏开关 release-lines-unread：不读这种行，成功句里也不打 release 那半句
crates_release=(); release_lines_read=""
if [[ "${GATE_MUTATION_TABLES_BREAK:-}" != *release-lines-unread* ]]; then
  mapfile -t crates_release < <(grep $'^RELEASE\t' <<<"$scope_report")
  release_lines_read=1
fi
scope_identifiers="$(sed -n 's/^IDENTIFIERS_CHECKED\t//p' <<<"$scope_report")"
# 点名的测试够不着被改坏的函数（文件头这一格「点名测试够不着被改坏的函数」）：static-check 在 scope_report 里一律判，UNREACHABLE 与 REACHABILITY_UNJUDGED 进红句，
# REACHABILITY_CHECKED 与 REACHABILITY_NOT_JUDGED 进成功句。弄坏开关 reachability-lines-unread 退回不读这几行：UNREACHABLE 与 REACHABILITY_UNJUDGED 不进红句、成功句里的可达性那半句也不打
scope_unreachable=(); scope_reachability_unjudged=(); scope_reachability_checked=""; scope_reachability_not_judged=""
if [[ "${GATE_MUTATION_TABLES_BREAK:-}" != *reachability-lines-unread* ]]; then
  mapfile -t scope_unreachable < <(grep $'^UNREACHABLE\t' <<<"$scope_report")
  mapfile -t scope_reachability_unjudged < <(grep $'^REACHABILITY_UNJUDGED\t' <<<"$scope_report")
  scope_reachability_checked="$(sed -n 's/^REACHABILITY_CHECKED\t//p' <<<"$scope_report")"
  scope_reachability_not_judged="$(sed -n 's/^REACHABILITY_NOT_JUDGED\t//p' <<<"$scope_report")"
fi
mapfile -t scope_errors < <(sed -n 's/^ERROR\t//p' <<<"$scope_report")
IFS=$'\t' read -r scope_row_count scope_source_count scope_literal_count <<<"$scope_checked"
crates_checked="$(sed -n 's/^CRATES_CHECKED //p' <<<"$crates_report")"
mapfile -t crates_dup < <(grep $'^CRATES_DUP\t' <<<"$crates_report")
mapfile -t crates_duplicate_groups < <(grep $'^CRATES_DUP_CHANGE\t' <<<"$crates_report")
mapfile -t crates_ignored < <(grep '^CRATES_IGNORED' <<<"$crates_report")
mapfile -t crates_filter < <(grep '^CRATES_FILTER' <<<"$crates_report")
mapfile -t crates_missing_tests < <(grep '^CRATES_MISSING' <<<"$crates_report")
mapfile -t crates_packages < <(grep $'^CRATES_PACKAGE\t' <<<"$crates_report")
IFS=$'\t' read -r rows_with_member_packages rows_with_manifest_paths < <(sed -n 's/^CRATES_PACKAGES_CHECKED\t//p' <<<"$crates_report")
anchor_checked="$(sed -n 's/^CHECKED //p' <<<"$anchor_report")"
tables_read="$(sed -n 's/^TABLES_READ //p' <<<"$anchor_report")"
mapfile -t research_duplicate_groups < <(grep $'^DUP\t' <<<"$anchor_report")
mapfile -t anchor_bad < <(grep '^BAD' <<<"$anchor_report")
mapfile -t stray_rows < <(grep '^STRAY' <<<"$anchor_report")
mapfile -t anchor_unreadable < <(grep '^UNREADABLE' <<<"$anchor_report")
mapfile -t tables_without_binary < <(sed -n 's/^NO_BINARY\t//p' <<<"$anchor_report")
# ── 重复行的滞后表：登记的组不判红，表只缩不涨（判法写在这一格的文件头注释「滞后表」那几行）──
DUPLICATES_LAG=.claude/gate.d/mutation-duplicates-lag.tsv
lag_base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  LIB_CHANGED_PATHS="$STAGE_REPOSITORY/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  lag_base="$(gate_diff_base gate)"
fi
duplicates_lag_judge="$(cat <<'PY'
import importlib.util, os, sys
lag_path, mutation_directory, base = sys.argv[1], sys.argv[2], sys.argv[3]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[4])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
broken = os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
CRATES_TABLE = "crates/mutations.tsv"
groups = []   # (表, 原样输出的那一行去掉名字那一段, 组内名字的列表)
for line in sys.stdin.read().split("\n"):
    fields = line.split("\t")
    if fields[0] == "CRATES_DUP_CHANGE" and len(fields) == 3:
        groups.append((CRATES_TABLE, "\t".join(fields[:2]), fields[2].split("\x1f")))
    elif fields[0] == "DUP" and len(fields) == 4:
        groups.append((f"{mutation_directory}/{fields[1]}.tsv", "\t".join(fields[:3]), fields[3].split("\x1f")))
def names_of_table(path):
    """那张表第一列的全部名字（注释行与空行不算）；读不了返回 None。"""
    try:
        text = open(path, encoding="utf-8").read()
    except (OSError, UnicodeDecodeError):
        return None
    return {line.split("\t")[0] for line in text.split("\n") if line.strip() and not line.startswith("#")}
def parse_rows(text, label):
    """滞后表的行：[(行号, 表, 名字列表)]，形状不对的进 problems。切行经 lib-lag-baseline.py 的 content_rows（注释与空行不算）。"""
    rows, problems = [], []
    for line_number, fields in lag_baseline.content_rows(text):
        if len(fields) != 4 or not all(field.strip() for field in fields):
            problems.append(f"{label}:{line_number} 不是四列、或有空格子（表、组内名字、为什么滞后、归哪一次处理）")
            continue
        rows.append((line_number, fields[0].strip(), fields[1].split("|")))
    return rows, problems
problems = []
rows = []
malformed_count = 0
if "duplicates-lag-ignored" not in broken and os.path.exists(lag_path):
    try:
        rows, problems = parse_rows(open(lag_path, encoding="utf-8").read(), lag_path)
        malformed_count = len(problems)
    except (OSError, UnicodeDecodeError) as error:
        problems.append(f"{lag_path} 读不了（{type(error).__name__}: {error}）")
covered = {}   # 组的下标 → 罩住它的那一行的行号
lag_lines = []
table_names = {}
for line_number, table, names in rows:
    where = f"{lag_path}:{line_number}"
    if table != CRATES_TABLE and not (table.startswith(mutation_directory + "/") and table.endswith(".tsv")):
        problems.append(f"{where} 的表「{table}」不是 {CRATES_TABLE}，也不是 {mutation_directory}/<表名>.tsv")
        continue
    if table not in table_names:
        table_names[table] = names_of_table(table)
    if table_names[table] is None:
        problems.append(f"{where} 指的表 {table} 不在或读不了")
        continue
    if len(set(names)) != len(names):
        problems.append(f"{where} 里同一个名字写了不止一次")
        continue
    missing = [name for name in names if name not in table_names[table]]
    if missing:
        problems.append(f"{where} 的这几个名字在 {table} 里找不到：" + "；".join(missing))
        continue
    exact = [index for index, (group_table, _shown, group_names) in enumerate(groups) if group_table == table and set(group_names) == set(names)]
    if len(exact) > 1:
        problems.append(f"{where} 的名字在 {table} 里对上 {len(exact)} 组（名字在表里不唯一），一行只罩一组")
        continue
    if not exact:
        holding = [group_names for group_table, _shown, group_names in groups if group_table == table and set(names) <= set(group_names)]
        if holding:
            problems.append(f"{where} 那一组里还有没登记的名字：" + "；".join(sorted(set(holding[0]) - set(names))))
        else:
            problems.append(f"{where} 的名字此刻不是同一组重复行（" + ("三样不全同" if table == CRATES_TABLE else "除变异名外各列不全同") + "）：" + "；".join(names))
        continue
    if exact[0] in covered:
        problems.append(f"{where} 与第 {covered[exact[0]]} 行罩的是同一组")
        continue
    covered[exact[0]] = line_number
    lag_lines.append(f"{table}\t" + "；".join(names))
# 只缩不涨：此刻的每一行，基准那一版同一张表里要有一行的名字是它的超集；基准里还没有这张表不比，git 取不到基准那一版判红
baseline_note = "不在 git 仓里，没和基准比"
if base and rows and "duplicates-lag-growth-unchecked" not in broken:
    # 取基准那一版：lib-lag-baseline.py 的 baseline_text，「基准里没有这张表」与「git 取不到」分开，后者判红
    shown, baseline_problem = lag_baseline.baseline_text(base, lag_path)
    if baseline_problem is not None:
        problems.append(f"{baseline_problem.sentence(200)}，只缩不涨没比")
    elif shown is None:
        baseline_note = f"基准 {base[:12]} 里还没有这张表，没比只缩不涨"
    else:
        baseline_rows, _baseline_problems = parse_rows(shown, f"基准里的 {lag_path}")
        grown = [(line_number, table, names) for line_number, table, names in rows
                 if not any(base_table == table and set(names) <= set(base_names) for _n, base_table, base_names in baseline_rows)]
        for line_number, table, names in grown:
            problems.append(f"{lag_path}:{line_number} 相对基准那一版多出一组（{table}：" + "；".join(names) + "）：这张表只缩不涨")
        baseline_note = f"与基准 {base[:12]} 那一版按组比，" + (f"多出 {len(grown)} 组" if grown else "没涨")
elif base and "duplicates-lag-growth-unchecked" in broken:
    baseline_note = "没和基准比"
elif base:
    baseline_note = "表里没有行，不用和基准比"
for index, (_table, shown, _names) in enumerate(groups):
    if index not in covered:
        print(shown)
for line in lag_lines:
    print("LAG", line, sep="\t")
for problem in problems:
    print("LAG_PROBLEM", problem, sep="\t")
print("LAG_BASELINE", baseline_note, sep="\t")
print("LAG_ROWS", len(rows) + malformed_count, sep="\t")
PY
)"
lag_report="$( { printf '%s\n' ${crates_duplicate_groups[@]+"${crates_duplicate_groups[@]}"} ${research_duplicate_groups[@]+"${research_duplicate_groups[@]}"}; } \
  | python3 -c "$duplicates_lag_judge" "$DUPLICATES_LAG" "$MUT_DIR" "$lag_base" "$LAG_BASELINE_LIBRARY")"
lag_exit_code=$?
mapfile -t crates_dup_changes < <(sed -n 's/^CRATES_DUP_CHANGE\t//p' <<<"$lag_report")
mapfile -t research_duplicates < <(grep $'^DUP\t' <<<"$lag_report")
mapfile -t lag_groups < <(sed -n 's/^LAG\t//p' <<<"$lag_report")
mapfile -t lag_problems < <(sed -n 's/^LAG_PROBLEM\t//p' <<<"$lag_report")
lag_baseline="$(sed -n 's/^LAG_BASELINE\t//p' <<<"$lag_report")"
lag_rows="$(sed -n 's/^LAG_ROWS\t//p' <<<"$lag_report")"
# 两段 python 自己崩了（退出码非 0、或没报出 CHECKED 那一行）：它们没判完，不许读成「没有 BAD 行」
script_failures=()
if [[ "${anchor_exit_code:-1}" -ne 0 || -z "$anchor_checked" ]]; then
  script_failures+=("research 变异表的锚点检查没跑完（python 退出码 ${anchor_exit_code:-?}，CHECKED 行「${anchor_checked}」）")
fi
if [[ -f crates/mutations.tsv ]] && [[ "${crates_exit_code:-1}" -ne 0 || -z "$crates_checked" ]]; then
  script_failures+=("crates/mutations.tsv 的重复、#[ignore]、筛选词、点名的测试在不在目标里这几样检查没跑完（python 退出码 ${crates_exit_code:-?}，CRATES_CHECKED 行「${crates_checked}」）")
fi
if [[ "${lag_exit_code:-1}" -ne 0 || -z "$lag_rows" ]]; then
  script_failures+=("重复行滞后表 $DUPLICATES_LAG 的判法没跑完（python 退出码 ${lag_exit_code:-?}，LAG_ROWS 行「${lag_rows}」），重复行哪几组算滞后没判")
fi
if [[ "$scope_exit_code" -ne 0 && "$scope_exit_code" -ne 1 ]] || [[ -z "$scope_checked" ]]; then
  script_failures+=("门禁 crates-mutation-replay 派活之前那几样与拷贝范围的检查没跑完（scripts/mutation/crates-mutation-rows.py static-check 退出码 ${scope_exit_code}，CHECKED 行「${scope_checked}」；${scope_errors[*]:-没报原因}）")
fi

if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]} + ${#crates_bad[@]} + ${#crates_dup[@]} + ${#crates_dup_changes[@]} + ${#research_duplicates[@]} + ${#lag_problems[@]} + ${#crates_ignored[@]} + ${#crates_filter[@]} + ${#crates_missing_tests[@]} + ${#stray_rows[@]} + ${#crates_stray[@]} + ${#crates_malformed[@]} + ${#crates_names[@]} + ${#anchor_unreadable[@]} + ${#script_failures[@]} + ${#scope_rows[@]} + ${#scope_literals[@]} + ${#crates_undefined[@]} + ${#crates_release[@]} + ${#crates_packages[@]} + ${#scope_unreachable[@]} + ${#scope_reachability_unjudged[@]})); then
  if ((${#script_failures[@]})); then
    echo "  ✗ 锚点或拷贝范围检查的 python 没跑完，这一段等于没判："
    printf '      %s\n' "${script_failures[@]}"
    echo "    → 怎么办：看上面 python 打出的报错（多半是某份文件读不了），修好再跑；没跑完不许当成锚点都对得上。"
  fi
  if ((${#anchor_unreadable[@]})); then
    echo "  ✗ 这些变异表或它的被测源码读不了（不是 UTF-8、是目录、没权限），表里的条目一条都没判："
    while IFS=$'\t' read -r _ stem why; do
      printf '      %s 或 %s：%s\n' "$MUT_DIR/$stem.tsv" "$BIN_DIR/$stem.rs" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_unreadable[@]}")
    echo "    → 怎么办：转成 UTF-8 文本（或把放错的目录挪走），再跑；mutate.sh 读它时同样会失败。"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 这些实验二进制没有同名变异表："   # gate-lint:detail
    printf '      %s\n' "${missing[@]}"
  fi
  if ((${#malformed[@]})); then
    echo "  ✗ 这些变异表不成形（要求每行三段制表符分隔，至少一条）："
    printf '      %s\n' "${malformed[@]}"
    echo "    → 每行写成「变异名<TAB>原文<TAB>替换文」三段，坏行补齐或删掉，至少留一条成形的。"
  fi
  if ((${#anchor_bad[@]})); then
    echo "  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 把这几条记「⚓ 锚点失效」、不跑，别的照跑，整轮退 3）："
    while IFS=$'\t' read -r _ stem name lineno hits; do
      if [[ -f "$BIN_DIR/$stem.rs" ]]; then
        printf '      %s:%s %s：原文在 %s 里命中 %s 次\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$BIN_DIR/$stem.rs" "$hits"   # gate-lint:detail
      else
        printf '      %s:%s %s：原文在 %s 里命中 %s 次（装置目录，按底下全部 .rs 合计）\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$BIN_DIR/$stem/" "$hits"   # gate-lint:detail
      fi
    done < <(printf '%s\n' "${anchor_bad[@]}")
    echo "    命中 0 次：源码改过而表没跟；命中多次：原文要多带一行上下文才唯一。"
  fi
  if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]})); then
    echo "  → 怎么办：缺表的写 scripts/mutation/tables/<bin名>.tsv（每行：变异名<TAB>原文<TAB>替换文）；"
    echo "    锚点对不上的把「原文」改成今天源码里逐字存在、且只出现一次的那一段，"
    echo "    改完跑 bash scripts/mutation/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。"
  fi
  if ((${#stray_rows[@]})); then
    echo "  ✗ 这些变异条目的原文或替换文里有 \\n 以外的反斜杠转义（mutate.sh 只还原 \\n，别的按字面写进源码）："
    while IFS=$'\t' read -r _ stem name lineno segment escape; do
      printf '      %s:%s %s：%s里的 %s\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${stray_rows[@]}")
    echo "    → 怎么办：去掉那个反斜杠。替换文按字面写进源码，\\& \\\" 这类会原样落进去、那份源码编不过，"
    echo "      变异既不算被抓也不算没红（.claude/rules/mutation-sampling.md 第八类：无效变异），而跑变异的脚本报的是「点名的测试没跑到」。"
  fi
  if ((${#crates_stray[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的原文或替换文里有 \\n 以外的反斜杠转义（门禁 crates-mutation-replay 只还原 \\n）："
    while IFS=$'\t' read -r _ lineno name segment escape; do
      printf '      crates/mutations.tsv:%s %s：%s里的 %s\n' "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_stray[@]}")
    echo "    → 怎么办：去掉那个反斜杠，改完单跑那一行证明点名的测试红（2026-09-20 第 180 行踩过：替换文写 \\&report.outcome，"
    echo "      写进源码是字面反斜杠、编不过，crates-mutation-replay 把它报成「点名的测试没跑到」，出路指向「先造一条会红的用例」，方向是反的）。"
  fi
  if ((${#crates_dup_changes[@]})); then
    echo "  ✗ crates/mutations.tsv 里有 ${#crates_dup_changes[@]} 组行的「文件、原文、替换文」三样逐字全同（同一处改法写了不止一行）："
    printf '      crates/mutations.tsv %s\n' "${crates_dup_changes[@]}"   # gate-lint:detail
    echo "    → 怎么办：一处改法留一行，多的删。"
  fi
  if ((${#crates_dup[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的变异名与前面某一行相同（按名字分格的报告会把两条并成一格，其中一条的结论看不见）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_dup[@]}")
    echo "    → 怎么办：给后面那一行起一个说它改了什么的新名字（名字写它是什么、做什么，不加序号）。"
  fi
  if ((${#research_duplicates[@]})); then
    echo "  ✗ 这些变异表里有 ${#research_duplicates[@]} 组行除变异名外各列逐字全同（同一处改法写了不止一行）："
    while IFS=$'\t' read -r _ stem where; do
      printf '      %s %s\n' "$MUT_DIR/$stem.tsv" "$where"   # gate-lint:detail
    done < <(printf '%s\n' "${research_duplicates[@]}")
    echo "    → 怎么办：一处改法留一行，多的删。"
  fi
  if ((${#lag_problems[@]})); then
    echo "  ✗ 重复行滞后表 $DUPLICATES_LAG 有 ${#lag_problems[@]} 处不对（指到不存在的名字、名字此刻不是同一组、或相对基准那一版涨了）："
    printf '      %s\n' "${lag_problems[@]}"   # gate-lint:detail
    echo "    → 怎么办：名字照那张表第一列逐字写、一行写全一组的名字（用 | 隔）；那一组已经不重复了（删了多的、或改了写法）就删掉这一行；"
    echo "      这张表只缩不涨，新出的重复行不往里加，照上面重复行那一条一处改法留一行。"
  fi
  if ((${#crates_packages[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行 cargo test 参数里点名的包认不出来（crates-mutation-replay 照这一行跑 cargo test，包点错了它起不来，或点名的测试不在包里）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_packages[@]}")
    echo "    → 怎么办：工作区成员（仓根 Cargo.toml 的 [workspace] members）写 -p <包名>；被 exclude 掉的独立工程写 --manifest-path <它的 Cargo.toml>（相对仓根的路径，只许仓内）；"
    echo "      一行只用其中一种、--manifest-path 只写一次；改完单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#crates_filter[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行 cargo test 参数里的筛选词筛不到点名的测试（crates-mutation-replay 跑它时那条测试不跑，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_filter[@]}")
    echo "    → 怎么办：把 \`--\` 后面的筛选词改成点名测试名的一段（测试改过名、拆过的多半是这个），或者删掉筛选词跑整个测试目标。"
  fi
  if ((${#crates_missing_tests[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行点名的测试在 cargo test 参数选的目标里找不到（crates-mutation-replay 跑它时输出里没有这条测试，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_missing_tests[@]}")
    echo "    → 怎么办：测试改了名就把最后一列改成新名字；测试挪了目标就改 cargo 参数里的 --lib / --test <名>；测试删了就换一条今天还在、改坏这一处会红的测试，"
    echo "      改完单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#crates_ignored[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行点名的测试标了 #[ignore]，cargo test 参数里却没有 --include-ignored（crates-mutation-replay 跑它时那条测试被跳过，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_ignored[@]}")
    echo "    → 怎么办：在那一行 cargo test 参数的 \`--\` 后面加 --include-ignored（没有 \`--\` 就补 \`-- --include-ignored\`），改完单跑那一行证明点名的测试红。"
  fi
  if ((${#crates_malformed[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行不成形（六段制表符分隔：变异名、文件、原文、替换文、cargo test 参数、必须红的测试名；替换文可以空，那是删掉原文，其余五段不许空；门禁 crates-mutation-replay 派活之前同样判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_malformed[@]}")
    echo "    → 怎么办：补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。"
  fi
  if ((${#crates_names[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「必须红的测试名」不是一个用例名（带了正则符号或空白，门禁 crates-mutation-replay 按字面在测试输出里永远找不到，派活之前同样判红）："
    while IFS=$'\t' read -r _ lineno name expected; do
      printf '      crates/mutations.tsv:%s %s：「%s」\n' "$lineno" "$name" "$expected"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_names[@]}")
    echo "    → 怎么办：那一列写成用例名本身（或带模块路径 tests::<名字>），去掉 \$、^ 这类符号；crates-mutation-replay 已经按「在输出里只许对上一个用例」判，不用再靠 \$ 锚定。"
  fi
  if ((${#crates_bad[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 crates-mutation-replay 把这几行记「锚点失效」、不跑，别的行照判，整道判红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_bad[@]}")
    echo "    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），改完单跑那几行证明点名的测试红。"
  fi
  if ((${#scope_rows[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行改的文件不在门禁 crates-mutation-replay 每片拷的范围里（crates-mutation-replay 派活之前就整道判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name path; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$path"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_rows[@]}")
    echo "    → 怎么办：把那个路径（或它所在的目录）登记进 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay.sh 那一行（拷贝范围就是那一行登记的路径，现在是：${scope_copied}），"
    echo "      或者把这条变异改到拷贝范围之内的文件上。"
  fi
  if ((${#scope_literals[@]})); then
    echo "  ✗ crates/ 下这些 .rs 里有以 \"../ 起头、解析之后逃出 crates/、又不在门禁 crates-mutation-replay 拷贝范围里的字符串字面量（紧跟在 include_str!(、include_bytes!( 后面的照源文件所在目录解析，别的照所在 crate 的根解析；crates-mutation-replay 的副本里没有那个路径，运行时读它的测试没改坏就红，crates-mutation-replay 记「基线不绿」）："
    while IFS=$'\t' read -r _ file lineno literal resolved; do
      printf '      %s:%s %s → %s\n' "$file" "$lineno" "$literal" "$resolved"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_literals[@]}")
    echo "    → 怎么办：把那个路径所在的目录登记进 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay 那一行（它同时是 crates-mutation-replay 的复用判定输入，登记之后 crates-mutation-replay 整张重跑一次），"
    echo "      或者让测试读 crates/ 之内的文件；#[path = …] 指的源文件不算，不用管。"
  fi
  if ((${#crates_undefined[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#crates_undefined[@]} 行的替换文引了 crates 里找不到定义的标识符（被改文件与它所在的包的 .rs 里一处都没有；套上去编不过，门禁 crates-mutation-replay 记无效变异、整道判红）："
    while IFS=$'\t' read -r _ lineno name path identifiers; do
      printf '      crates/mutations.tsv:%s %s（改 %s）：%s\n' "$lineno" "$name" "$path" "$identifiers"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_undefined[@]}")
    echo "    → 怎么办：替换文引了已删或改名的标识符：改那一行的替换文（换成今天源码里还在的名字，或照今天的代码重写这一处改法），"
    echo "      改完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 UNDEFINED，再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#crates_release[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#crates_release[@]} 行命中重测试目标（gpu 特性、登记的崩溃枚举用例目标）而第五段没写 --release（门禁 crates-mutation-replay 与证红照补 --release 起，表上写的参数与真跑的不一致）："
    while IFS=$'\t' read -r _ lineno name basis; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$basis"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_release[@]}")
    echo "    → 怎么办：第五段补上 --release（插在第一个 -- 之前；--profile release 也算写了），判法在 scripts/mutation/lib_release_only_tests.py；"
    echo "      改完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 RELEASE，再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#scope_unreachable[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#scope_unreachable[@]} 行点名的测试够不着被改坏的函数（closure：点名测试的单元闭包里根本没有被改坏的单元；gpu-route：开 GPU 时点名测试走 GPU 判段，被改坏的单元只在 GPU 路上不走的 CPU 判器入口后面；改了它测试照样过，这一行等于没判）："
    while IFS=$'\t' read -r _ lineno name route mutated test cargo_arguments; do
      printf '      crates/mutations.tsv:%s %s（%s）：被改坏的 %s，点名的测试 %s（门禁给这一行用的 cargo 参数 %s）\n' "$lineno" "$name" "$route" "$mutated" "$test" "$cargo_arguments"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_unreachable[@]}")
    echo "    → 怎么办：换成调得到被改坏函数的用例，或删掉这一行；换完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 UNREACHABLE，再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）；"
    echo "      判法与每一种的出路见 scripts/mutation/crates-mutation-rows.py 文件头「点名测试够不着被改坏的函数」。"
  fi
  if ((${#scope_reachability_unjudged[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#scope_reachability_unjudged[@]} 行判不了点名的测试够不够得着被改坏的函数（锚点失效、拒算、闭包算不成；判不了不算够得着）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_reachability_unjudged[@]}")
    echo "    → 怎么办：照每行写的原因改（锚点失效的先改原文；闭包拒算的照 scripts/gate/lib_function_units.py 文件头「拒算」改成字面写法）；改完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 REACHABILITY_UNJUDGED。"
  fi
  exit 1
fi
n=${#apparatus_units[@]}   # 与上面的循环同一个射程：顶层的 .rs 与带根 main.rs 的装置目录，crate 根的库与主二进制不算
((n)) || { echo "  ! $BIN_DIR 下一个实验装置都没扫到（顶层的 .rs 与带 main.rs 的装置目录），这一格无对象可判"; exit 77; }
crates_summary="；没有 crates/mutations.tsv"
[[ -f crates/mutations.tsv ]] && crates_summary="；crates/mutations.tsv ${scope_row_count} 条的原文各命中源码一次（成形：六段，替换文可以空、其余五段不空；点名的测试名都是用例名）"
crates_summary+="；门禁 crates-mutation-replay 的拷贝范围（${scope_copied}）：变异表 ${scope_row_count} 行改的文件都在里面，crates/ 下 ${scope_source_count} 份 .rs 里逃出 crates/ 的相对路径字面量 ${scope_literal_count} 处都在里面"
if [[ -n "$scope_identifiers" ]]; then
  IFS=$'\t' read -r identifiers_judged identifiers_unjudged <<<"$scope_identifiers"
  crates_summary+="；替换文引的标识符判了 ${identifiers_judged} 行、都找得到定义（没判 ${identifiers_unjudged} 行：文件不是 .rs 或锚点失效）"
fi
if [[ -n "$release_lines_read" && -f crates/mutations.tsv ]]; then
  crates_summary+="；命中重测试目标的行第五段都写了 --release（判了 ${scope_row_count} 行：gpu 特性、登记的崩溃枚举用例目标）"
fi
if [[ -n "$scope_reachability_checked" ]]; then
  IFS=$'\t' read -r reachability_judged reachability_unjudged reachability_exempt reachability_cut _reachability_rest <<<"$scope_reachability_checked"
  reachability_gpu="${scope_reachability_checked##*$'\t'}"   # 最后一段：ENABLE_GPU=1 或 GPU 没开（前一段多机配置路径可以是空的，不按位置读）
  crates_summary+="；点名的测试都够得着被改坏的函数（判了 ${reachability_judged} 行、没判 ${reachability_unjudged} 行，豁免 ${reachability_exempt} 行，截过闭包 ${reachability_cut} 行；${reachability_gpu}）"
elif [[ -n "$scope_reachability_not_judged" ]]; then
  crates_summary+="；点名的测试够不够得着被改坏的函数本次未判（${scope_reachability_not_judged}），不算判过"
fi
if ((${#tables_without_binary[@]})); then
  crates_summary+="；没有同名实验二进制、锚点不在这一格射程的表 ${#tables_without_binary[@]} 张：$(printf '%s ' "${tables_without_binary[@]}")"
fi
if ((${#lag_groups[@]})); then
  echo "  · 重复行滞后 ${#lag_groups[@]} 组（$DUPLICATES_LAG 登记、这一格不判红；逐组：表、组内各行的变异名）："
  printf '      %s\n' "${lag_groups[@]//$'\t'/ }"
fi
lag_summary="；重复行滞后 ${#lag_groups[@]} 组（$DUPLICATES_LAG 登记 ${lag_rows} 行，${lag_baseline}）"
[[ -f "$DUPLICATES_LAG" ]] || lag_summary="；重复行滞后 0 组（没有 $DUPLICATES_LAG）"
echo "  ✓ $n 个实验二进制都有成形的变异表，${anchor_checked} 条变异的原文各命中源码一次，${tables_read} 张研究变异表里没有两行除变异名外各列全同${lag_summary}${crates_summary}（这一格不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行改法相同、点名标了 #[ignore] 的测试都带 --include-ignored、筛选词筛得到点名的测试、点名的测试在选的目标里找得到、-p 点名的包都是仓根 Cargo.toml 的工作区成员（${rows_with_member_packages:-0} 行）、--manifest-path 指到的都是仓内的 Cargo.toml（${rows_with_manifest_paths:-0} 行）——crates 那张表的重复按「文件 + 原文 + 替换文」三项认，变异名另判）"
echo "    这 $n 个里装置目录 ${directory_units} 个（$BIN_DIR/<名>/ 带根 main.rs，锚点按目录下全部 .rs 合计命中），顶层单文件 $((n - directory_units)) 个"
}

# ── 格 absolute-assertions：每个实验二进制都要有钉绝对值的断言
#
# 依据：.claude/rules/test-discipline.md「只让多条臂互相比，测不出「所有臂一起错」」。
# 只判机器判得了的一半：这个实验有没有任何一条把某个量与数字字面量比死的断言；钉的是不是对的量、绝对值是不是独立算出来的，留给人。
# 射程：scripts/experiment-apparatus 下除 lib.rs、main.rs 之外的每一份（顶层的 .rs，与带根 main.rs 的装置目录 <名>/），加上 crates/*/src/bin 下名字以 e<数字>_ 开头的
# .rs 与目录（按是不是实验认，不按住在哪个包认）；别处不以 e<数字>_ 开头的是装置工具，不判，成功行逐个列名。
# 一份装置是一个单元：装置目录与旁边有同名目录的 .rs 都按底下全部 .rs 合起来数，有一条绝对值断言就算；名叫 target 的子目录不进；扫到 0 个单元退 77（不算绿）。
# 弄坏开关 GATE_ABSOLUTE_ASSERTIONS_BREAK=apparatus-directories-unscanned 退回只认顶层的 .rs 文件：装置目录与 .rs 旁边的子模块目录都不认：
#   样本 absolute-assertions-red 里两个装置目录（scripts 下的 e12、实验包式的 crates/demo/src/bin/e8）一条绝对值断言都没有那两行找不到，
#   absolute-assertions-green 里 e7 的 .rs 本身没有断言、断言在旁边同名目录的子模块里，不认子模块目录就判红，样本判错。
# 断言按语句认、不按行认：rustfmt 会把长断言拆成多行，逐行 grep 会把拆开的断言误判成没有。
# 样本：red 有只比相对值的（e7 目录单行 e1_relative、crates/ 下单行 e2_crates_relative）、e7 目录下拆成多行但比字符串的（e2_multiline_relative），
# 与注释掉的、只判非零的、消息文本里带「, 5)」的（e6_commented_and_structural），四份都要点名判红；
# green 里 e7 目录下每一份都钉了绝对值（e1_pinned 单行、e5_multiline_pinned 拆成多行用 `assert_eq!`），crates/ 下一份单行（e3_crates_pinned）、
# 一份拆成多行用 `assert!` 带消息（e4_crates_multiline_pinned），判绿；另放一份不以 e<数字>_ 开头的 crates/demo/src/bin/probe.rs，成功行要把它列成没判的。

# 绝对值断言：与数字字面量比死，assert_eq! 与 assert! 两种形态都认。
# assert! 那一支只认 ==、<=、>=、<、>，而且运算符前一个字不能是 !、=、<、>：`!= 0` 是结构性质（非空、非零），不是把量钉死在一个数上。
ABSOLUTE_ASSERTION_PATTERN='assert_eq!\([^;]*, *-?[0-9][0-9_]*(\.[0-9]+)?\)|assert!\([^;]*[^!<>=](==|<=|>=|<|>) *-?[0-9][0-9_]*(\.[0-9]+)?[,)]'

# 把拆成多行的 assert!()/assert_eq!() 按圆括号配平拼回一条条完整语句，逐条打到 stdout；配平之前挖掉字符串字面量与行注释。
# 每行先掐掉两端空白再首尾相接、不插分隔符：rustfmt 常把收尾的 `);` 单独放一行，原样拼会在数字和 `)` 之间垫出空白，判据就认不出了。
join_assertion_statements() {
  awk '
    function trim(s) {
      gsub(/^[ \t]+|[ \t]+$/, "", s)
      return s
    }
    function strip(line,    s) {
      s = line
      gsub(/"([^"\\]|\\.)*"/, "\"\"", s)
      sub(/\/\/.*/, "", s)
      return s
    }
    function paren_delta(s,    i, c, d) {
      d = 0
      for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "(") d++
        else if (c == ")") d--
      }
      return d
    }
    BEGIN { in_statement = 0; depth = 0; buffer = "" }
    {
      raw_line = $0
      if (!in_statement) {
        # 起点在挖掉字符串与行注释之后的那一行上找：注释掉的 `// assert_eq!(x, 5);`、字符串里的「assert!(」都不是一条断言
        code_line = strip(raw_line)
        if (!match(code_line, /assert(_eq)?!\(/)) next
        piece = trim(substr(code_line, RSTART))
        buffer = piece
        depth = paren_delta(piece)
        in_statement = 1
      } else {
        # 判据套在挖掉字符串与行注释之后的语句上：消息文本里的「, 5)」不算把量钉死
        piece = trim(strip(raw_line))
        buffer = buffer piece
        depth += paren_delta(piece)
      }
      if (depth <= 0) {
        print buffer
        in_statement = 0
        buffer = ""
        depth = 0
      }
    }
  ' "$1"
}

# 一份实验装置的源文件，每行一个路径：标签是顶层的 <名>.rs（旁边同名目录 <名>/ 底下的 .rs 是它的子模块，一并算）或装置目录 <名>/（根是 main.rs，子模块逐层往下）；
# 名叫 target 的子目录不进。弄坏开关 GATE_ABSOLUTE_ASSERTIONS_BREAK=apparatus-directories-unscanned 时只认标签本身那一份 .rs、不认任何目录。
apparatus_unit_files() {
  local label="$1"
  if [[ -d "$label" ]]; then
    find "$label" -name '*.rs' -not -path '*/target/*' | LC_ALL=C sort
    return
  fi
  echo "$label"
  if [[ -d "${label%.rs}" && "${GATE_ABSOLUTE_ASSERTIONS_BREAK:-}" != *apparatus-directories-unscanned* ]]; then
    find "${label%.rs}" -name '*.rs' -not -path '*/target/*' | LC_ALL=C sort
  fi
}

cell_absolute_assertions() {
BINS=scripts/experiment-apparatus
# $BINS 不在不等于无对象：住在 crates/ 下、以 e<数字>_ 开头的实验照判；两处都一份没有，才在末尾退 77
judged=() uncovered=()
directories_judged=1 directory_units=0
[[ "${GATE_ABSOLUTE_ASSERTIONS_BREAK:-}" == *apparatus-directories-unscanned* ]] && directories_judged=0
for f in "$BINS"/*.rs; do
  case "$(basename "$f")" in lib.rs|main.rs) continue ;; esac   # crate 根的库与主二进制不是一份实验
  [[ -f "$f" ]] && judged+=("$f")   # 目录是空的时候 glob 原样留着，不是一份实验
done
if ((directories_judged)); then
  for f in "$BINS"/*/main.rs; do
    if [[ -f "$f" && "$(dirname "$f")" != "$BINS/target" ]]; then
      judged+=("$(dirname "$f")/")   # 装置目录：标签带结尾的 /
      directory_units=$((directory_units+1))
    fi
  done
fi
in_bins=${#judged[@]}
for directory in crates/*/src/bin; do
  [[ -d "$directory" && "$directory" != "$BINS" ]] || continue
  for f in "$directory"/*.rs "$directory"/*/; do
    if [[ -f "$f" ]]; then
      if [[ "$(basename "$f")" =~ ^e[0-9]+_ ]]; then judged+=("$f"); else uncovered+=("$f"); fi
    elif [[ -d "$f" ]] && ((directories_judged)) && [[ "$(basename "$f")" =~ ^e[0-9]+_ ]] && [[ ! -f "${f%/}.rs" ]]; then
      judged+=("$f")   # 以 e<数字>_ 起头的装置目录（根是 main.rs）；旁边有同名 .rs 的是那份 .rs 的子模块目录，不另算一份
      directory_units=$((directory_units+1))
    fi
  done
done
bad=0; n=0
for f in "${judged[@]}"; do
  n=$((n+1))
  c=0
  while IFS= read -r source_file; do
    c=$(( c + $(join_assertion_statements "$source_file" | grep -cE "$ABSOLUTE_ASSERTION_PATTERN") ))
  done < <(apparatus_unit_files "$f")
  if (( c == 0 )); then
    echo "  ✗ $f 一条绝对值断言都没有"
    bad=$((bad+1))
  fi
done

if ((bad)); then
  echo "     → 怎么办：给它加一条把**被量的那个数**钉死的断言，绝对值要由**独立算术**给出，"
  echo "               不许从代码里读回来。加完用 scripts/mutation/mutate.sh 证明它会红——"
  echo "               实测教训：先加的断言可能一条变异都拦不住（E9 踩过），只有变异测试分得开。"
  exit 1
fi
((n)) || { echo "  ! $BINS 下一个 .rs 都没有、别处也没有 e<数字>_ 开头的，这一格无对象可判"; exit 77; }
echo "  ✓ $n 个实验二进制各自至少有一条绝对值断言（$BINS 下 $in_bins 份，别处 src/bin 下以 e<数字>_ 开头的 $(( n - in_bins )) 份）"
echo "    这 $n 个里装置目录 ${directory_units} 个（目录下全部 .rs 合起来至少一条）"
echo "    没判的 ${#uncovered[@]} 份（别处 src/bin 下不以 e<数字>_ 开头，按装置工具算）："
for f in ${uncovered[@]+"${uncovered[@]}"}; do echo "      $f"; done
}

# ── 格 reading-discipline：实验源码的两条读数纪律——种子不许折叠，读数不许恒为字面量 0
#
# 扫 scripts/experiment-apparatus 下顶层的 *.rs 与带根 main.rs 的装置目录（目录底下逐层往下的 .rs 都扫），与验证包、角色登记表 .claude/gate.d/crate-roles.tsv 里角色是
# 「实验装置」的包（实验包）的 src/bin 下名字以 e<号> 起头的 .rs 与目录（.rs 旁边同名目录里的 .rs 是它的子模块，一并扫；名叫 target 的子目录不进；顶层的 lib.rs、main.rs 是 crate 根，不是一份实验），只看注释与字符串之外的源码；两条都判完再退出。
# 一份装置是一个单元：种子折叠按文件判；整型字段在哪个文件里声明、豁免表与报告就记哪个文件，而「写过它、初始化过它、拿它进了断言」在整份装置的源文件里找
# （装置拆成子模块时字段与它的赋值常常不在一个文件里；单文件装置按整份文件判）；字段声明前的可见性认 pub 与 pub(super)、pub(crate) 这类带括号的。
#   弄坏开关 GATE_READING_DISCIPLINE_BREAK=apparatus-directories-unscanned 退回只扫 scripts/experiment-apparatus/*.rs 与 crates/singlefs-verification/src/bin/e[0-9]*.rs（只扫一层）：
#     样本 reading-discipline-red 里装置目录 e13 与登记为实验装置的包里 e14 的三处违规找不到，reading-discipline-green 里扫到的文件数对不上，两份样本都判错。
#   弄坏开关 GATE_READING_DISCIPLINE_BREAK=restricted-visibility-unread 字段可见性只认 pub、不认 pub(super)：样本 reading-discipline-red 里 pub(super) 声明的恒为 0 的字段找不到，
#     reading-discipline-green 里整型结构体字段数少一个。
#   ① C59（种子折叠成同一个状态）：名字里带 seed 的标识符后面直接跟 `| 1` 或 `& !1`，判红。`seed | 1` 把 2 与 3、4 与 5 折成同一个状态，
#      轮间变异系统性偏小。改法：先乘法混淆再置位，`seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1`（奇数乘子是双射）。
#   ② C60（恒定读数没有故障注入自证）：本文件里声明的整型结构体字段，从没出现在写入位置（`= 非零表达式`、`+=` 这类复合赋值、`&mut`），
#      而每一处结构体字面量初始化都是字面量 0，判红：这样的读数与「被测对象真的是 0」在产物里长得一模一样。
# 存量登记进两张豁免表（改一处就要重跑那个实验、逐个回对正文引的数），四列制表符分隔：文件路径、定位（①写行号，②写 `结构体.字段`）、欠账编号、为什么还没改：
#   .claude/gate.d/experiment-seed-fold-lag.tsv（挂 C59）
#   .claude/gate.d/experiment-constant-reading-lag.tsv（挂 C60）
# 豁免表的闸，任一条不成立判红：编号在 checks-owed.md 欠着那张表里（按 lib-owed.py 读，认不出「### 已还清」标题判红）；
# 每一行对得上这一轮扫出来的一处违规；只缩不涨：与基准提交比，一份文件拆成几份的按拆分族比之和（scripts/gate/lib_lag_split_family.py 的 family_growth：此刻每个文件先归到基准里的入口文件，
# 归到同一行的几个文件登记行数之和不许超过基准那一行），不在任何入口文件的族里的文件、基准里没有的文件不许出现。
#   弄坏开关 GATE_READING_DISCIPLINE_BREAK=lag-split-family-unrecognized 退回只认路径自己：装置拆成入口加子模块文件、登记行跟着挪过去的，子模块文件都成了族外；
#     样本 reading-discipline-split-family-green（拆后登记行之和与基准相同）判成红，reading-discipline-split-family-red 里族内之和涨了那一条的措辞变成族外那一句，两份样本都判错。
# 基准里还没有这张表算初次登记、成功行照实报；git 取不到基准那一版判红，不当初次登记。
#
# 射程：
#   ① 只认名字带 seed 的标识符。先把种子搬进别的名字再折叠（`let base = seed; let mut state = base | 1;`）认不出；
#      折叠之后再混淆（`(seed | 1) ^ K`、`(seed | 1).wrapping_mul(K)`）认得。只判表达式形态，不追这个值拿去做什么。
#   ② 只认本文件里 `struct X { 字段: 整型 }` 声明的字段；局部变量、非整型字段、别的 crate 的结构体、宏展开的字段罩不到。
#      用非字面量初始化（`field: total`）算写过。C60 的另一半（每个进结论的读数要有一次让它变号的故障注入自证）要人做。
#   只缩不涨按每个文件（拆分族按族）的登记行数比（行号会随无关改动漂），同一个文件里删一行再加一行看不出来。
#
# 样本：red 里 e8_seed_and_readings.rs 同时犯两条，另带一张挂在认不出的欠账表上的豁免表、
# 一份住在 crates/singlefs-verification/src/bin/ 下同样折叠了种子的装置，豁免表基准那一版的 blob 被 setup.sh 从对象库里删掉，必须判红；
# green 是同一份源码加上两张对得上的豁免表，必须判绿（别的格的实验二进制一处都不犯，扫到的文件数随之是 4）。
# reading-discipline-split-family-green：基准提交里 e8 还是 scripts/experiment-apparatus/e8_seed_and_readings.rs 一个文件、两张豁免表各一行登记在它名下；此刻它拆成 e8_seed_and_readings/main.rs 加
# readings.rs，两张表各一行跟着挪到拆出来的文件名下（setup.sh 先提交拆之前的样子再换上拆之后的）：族内之和与基准相同，必须判绿；
# reading-discipline-split-family-red 是它的拷贝，C60 那张表在 readings.rs 名下登记了两行（族内之和 2 超过基准的 1），另有一份不属于任何入口文件的族的 e12_unrelated_readings.rs 也登记了一行，两处都判红。
cell_reading_discipline() {
base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  LIB_CHANGED_PATHS="$STAGE_REPOSITORY/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
fi
SPLIT_FAMILY_LIBRARY="$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py"   # 拆分族的认法（一份文件拆成几份，登记行跟着挪）只在这一份
python3 - "$base" "$OWED_LIBRARY" "$STAGE_DIRECTORY/lib/rejection-blocks.py" "$LAG_BASELINE_LIBRARY" "$(stage_cells_default_jobs)" "$SPLIT_FAMILY_LIBRARY" <<'PY'
import glob, importlib.util, multiprocessing, os, re, subprocess, sys

base = sys.argv[1]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[4])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
try:
    split_family_spec = importlib.util.spec_from_file_location("lib_lag_split_family", sys.argv[6])
    split_family = importlib.util.module_from_spec(split_family_spec)
    split_family_spec.loader.exec_module(split_family)
except (OSError, SyntaxError, ImportError) as error:
    print(f"  ✗ 载不进 {sys.argv[6]}：{error}")
    print("     → 怎么办：它随仓走（scripts/gate/lib_lag_split_family.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；拆分族的认法只在那一份里写。")
    sys.exit(1)
owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[2])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)
rejection_blocks_spec = importlib.util.spec_from_file_location("rejection_blocks", sys.argv[3])
rejection_blocks = importlib.util.module_from_spec(rejection_blocks_spec)
rejection_blocks_spec.loader.exec_module(rejection_blocks)
BREAK_SWITCHES = os.environ.get("GATE_READING_DISCIPLINE_BREAK", "")
SCAN_DIRECTORIES = "apparatus-directories-unscanned" not in BREAK_SWITCHES
APPARATUS_ROOT = "scripts/experiment-apparatus"
if SCAN_DIRECTORIES:
    # 装置住在 scripts/experiment-apparatus/（顶层的 <名>.rs，或带根 main.rs 的装置目录 <名>/）与各包的 src/bin/（名字以 e<号> 起头的 <名>.rs 与目录 <名>/）
    BIN_GLOB = "scripts/experiment-apparatus 下顶层的 *.rs 与带根 main.rs 的装置目录、验证包与角色登记为实验装置的包的 src/bin 下名字以 e<号> 起头的 .rs 与目录"
else:
    BIN_GLOB = "scripts/experiment-apparatus/*.rs 与 crates/singlefs-verification/src/bin/e[0-9]*.rs"
OWED_PATH = ".claude/kb/checks-owed.md"
SEED_LAG = ".claude/gate.d/experiment-seed-fold-lag.tsv"
READING_LAG = ".claude/gate.d/experiment-constant-reading-lag.tsv"
# 只缩不涨的比法认不认拆分族；弄坏开关 lag-split-family-unrecognized 退回只认路径自己：拆成部分文件之后登记行挪过去的，部分文件都成了族外
LAG_FAMILY_DEFECTS = frozenset({split_family.UNRECOGNIZED}) if "lag-split-family-unrecognized" in BREAK_SWITCHES else frozenset()

INTEGER = r"(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize)"
# 名字带 seed 的标识符紧跟 `| 1` 或 `& !1`。`1` 后面允许一个整型后缀，
# 不允许再跟数字或下划线——`seed | 16` 与 `seed | 1_000` 不是这个形态。
SEED_FOLD = re.compile(r"(?<![\w.])(\w*[sS]eed\w*)\s*(?:\|\s*1|&\s*!\s*1)(?:" + INTEGER + r")?(?![\w.])")
SEED_IDENT = re.compile(r"(?<![\w.])\w*[sS]eed\w*(?![\w])")
STRUCT_HEAD = re.compile(r"\bstruct\s+(\w+)\s*(?:<[^{};]*>)?\s*\{")
# 字段前的可见性：pub，或 pub(super)、pub(crate) 这类带括号的（装置拆成子模块之后字段多半是 pub(super)）
VISIBILITY = r"pub\s+" if "restricted-visibility-unread" in BREAK_SWITCHES else r"pub(?:\s*\([^)]*\))?\s+"
STRUCT_FIELD = re.compile(r"(?:^|,)\s*(?:" + VISIBILITY + r")?(\w+)\s*:\s*" + INTEGER + r"\s*(?=,|$)", re.M)
LITERAL_ZERO = re.compile(r"^0(?:" + INTEGER + r")?$")

failures = []          # 每项是一块要打印的拒绝：(摘要, 明细列表, 出路列表)
def reject(summary, details, steps):
    failures.append((summary, details, steps))

BLANK_OUT_OPENERS = re.compile(r'//|/\*|"')
BLOCK_COMMENT_MARKS = re.compile(r"/\*|\*/")
STRING_MARKS = re.compile(r'\\|"')

def blank_out(text):
    """把注释与字符串字面量换成同样长度的空白，行号与列号都不变。注释与字符串之外的整段原样拷，不逐字符走。"""
    out, index, size = [], 0, len(text)
    while index < size:
        opener = BLANK_OUT_OPENERS.search(text, index)
        if opener is None:
            out.append(text[index:]); break
        out.append(text[index:opener.start()]); index = opener.start()
        if opener.group(0) == "//":
            end = text.find("\n", index)
            end = size if end < 0 else end
            out.append(" " * (end - index)); index = end
        elif opener.group(0) == "/*":
            depth, end = 1, index + 2
            while depth:
                mark = BLOCK_COMMENT_MARKS.search(text, end)
                if mark is None:
                    end = size; break
                depth += 1 if mark.group(0) == "/*" else -1
                end = mark.end()
            out.append("".join(c if c == "\n" else " " for c in text[index:end])); index = end
        else:
            end = index + 1
            while True:
                mark = STRING_MARKS.search(text, end)
                if mark is None:
                    end = size; break
                if mark.group(0) == '"':
                    end = mark.end(); break
                end = mark.start() + 2
            out.append("".join(c if c == "\n" else " " for c in text[index:end])); index = end
    return "".join(out)

# ── 被扫集合：一次扫描，两条判据与「没查的是哪些」都从它现算 ───────────────
def rust_files_below(directory):
    """目录底下逐层往下的全部 .rs（名叫 target 的子目录不进），排好序。"""
    found = []
    for current, subdirectories, names in os.walk(directory):
        subdirectories[:] = sorted(name for name in subdirectories if name != "target")
        found.extend(os.path.join(current, name) for name in sorted(names) if name.endswith(".rs"))
    return found

CRATE_ROLE_TABLE = ".claude/gate.d/crate-roles.tsv"

def apparatus_crate_bin_directories():
    """装置二进制住的包的 src/bin：验证包，加上角色登记表 CRATE_ROLE_TABLE（四列制表符分隔：包名、角色、目录、理由）里角色是「实验装置」的包（实验包）。表不在就只有验证包。"""
    directories = {"crates/singlefs-verification/src/bin"}
    if os.path.isfile(CRATE_ROLE_TABLE):
        for line in open(CRATE_ROLE_TABLE, encoding="utf-8").read().split("\n"):
            columns = line.split("\t")
            if len(columns) == 4 and not line.startswith("#") and columns[1] == "实验装置":
                directories.add(columns[2].strip("/") + "/src/bin")
    return sorted(directories)

def apparatus_units():
    """实验装置的单元：[[源文件…]]。一份装置的源文件是 <名>.rs 加它旁边同名目录 <名>/ 底下全部 .rs，或根在 <名>/main.rs 的装置目录底下全部 .rs；
    scripts/experiment-apparatus 顶层的 lib.rs、main.rs 是 crate 根的库与主二进制，不是一份实验。"""
    units = []
    if not SCAN_DIRECTORIES:
        for path in sorted(glob.glob(APPARATUS_ROOT + "/*.rs") + glob.glob("crates/singlefs-verification/src/bin/e[0-9]*.rs")):
            if os.path.basename(path) not in ("lib.rs", "main.rs"):
                units.append([path])
        return units
    for base, name_filter in ((APPARATUS_ROOT, None), *((directory, re.compile(r"e[0-9]")) for directory in apparatus_crate_bin_directories())):
        stems = {}
        for entry in sorted(os.listdir(base) if os.path.isdir(base) else []):
            stem = entry[:-3] if entry.endswith(".rs") else entry
            path = os.path.join(base, entry)
            if name_filter is not None and not name_filter.match(stem):
                continue
            if entry.endswith(".rs") and os.path.isfile(path):
                if base == APPARATUS_ROOT and entry in ("lib.rs", "main.rs"):
                    continue
                stems.setdefault(stem, [])
            elif os.path.isdir(path) and entry != "target" and (name_filter is not None or os.path.isfile(os.path.join(path, "main.rs"))):
                stems.setdefault(stem, [])
        for stem in sorted(stems):
            members = []
            single = os.path.join(base, stem + ".rs")
            if os.path.isfile(single):
                members.append(single)
            members.extend(rust_files_below(os.path.join(base, stem)))
            if members:
                units.append(members)
    return units

units = apparatus_units()
paths = [path for members in units for path in members]
if not paths:
    print(f"  ✗ {BIN_GLOB} 一个文件都没扫到")
    print("     → 怎么办：实验装置目录搬了家就同步改这一格里的 APPARATUS_ROOT 与 apparatus_units（.claude/rules/path-moves.md）；")
    print("               扫到 0 个对象而报绿，与判过了在门禁输出里一模一样。")
    sys.exit(1)

def examine_one_source(path, text, problem, unit_text):
    """一份实验源码的两条判据。text 是这份文件抹掉注释与字符串之后的文本，problem 是读不动的说明（读得动是 None），unit_text 是这份文件所在那份装置全部源文件的文本（单文件装置就是 text）。
    交回 (读不动的说明, 名字带 seed 的标识符数, 折叠写法, 整型字段数, 恒为字面量 0 的字段)。种子折叠按这份文件判；字段在这份文件里声明，写它、初始化它、拿它进断言在整份装置里找
    （装置拆成子模块之后，字段与它的赋值常常不在同一份文件里）。"""
    if problem is not None:
        return (problem, 0, [], 0, [])
    # ① 折叠写法：[(行号, 原样片段)]
    seed_identifiers = len(SEED_IDENT.findall(text))
    hits = []
    for line_number, line in enumerate(text.split("\n"), 1):
        hits.extend((line_number, match.group(0).strip()) for match in SEED_FOLD.finditer(line))
    # ② 恒为字面量 0 的整型字段：[(结构体.字段, 初始化处数)]
    declared = {}
    for head in STRUCT_HEAD.finditer(text):
        # 花括号要数着配对：拿「下一个 }」凑合，一行写完的结构体会一路吃到别的结构体的收尾，把别人的字段记到自己名下。
        depth, cursor = 1, head.end()
        while cursor < len(text) and depth:
            if text[cursor] == "{": depth += 1
            elif text[cursor] == "}": depth -= 1
            cursor += 1
        if depth:
            continue
        for field in STRUCT_FIELD.finditer(text[head.end():cursor - 1]):
            declared.setdefault(field.group(1), head.group(1))
    frozen = []
    # 每条带前缀的正则都要从文本里每个标识符起试一遍；先拿以字段名字面起头的必要条件筛（这种搜法按字面跳着找），筛不中的那一条不可能中，不跑
    has_mutable_borrow = re.search(r"&\s*mut\s", unit_text) is not None
    has_assertion = "assert" in unit_text
    for field, owner in sorted(declared.items()):
        name = re.escape(field)
        prefix = r"(?<![\w.])(?:\w+\s*\.\s*)*" + name
        plainly_assigned = re.search(name + r"\s*=(?!=)", unit_text) is not None
        compound = re.search(name + r"\s*(?:[-+*/|&^%]=|<<=|>>=)", unit_text) and re.search(prefix + r"\s*(?:[-+*/|&^%]=|<<=|>>=)", unit_text)
        assigned = [match.group(1).strip() for match in re.finditer(prefix + r"\s*=(?!=)\s*([^;,}\n]+)", unit_text)] if plainly_assigned else []
        borrowed = has_mutable_borrow and re.search(r"&\s*mut\s+(?:\w+\s*\.\s*)*" + name + r"\b", unit_text)
        # 经方法调用的结果写字段（`history_mut(…).名 = 值`）与结构体字面量里的字段简写（一行只有 `名,`，值是同名的局部变量）也算写过
        if "reading-writes-unseen" not in os.environ.get("GATE_READING_DISCIPLINE_BREAK", ""):
            if plainly_assigned:
                assigned += [match.group(1).strip() for match in re.finditer(r"\)\s*\.\s*" + name + r"\s*=(?!=)\s*([^;,}\n]+)", unit_text)]
            if re.search(name + r"\s*,", unit_text) and re.search(r"^\s*" + name + r"\s*,\s*$", unit_text, re.M):
                continue
        # 进了断言被拿来核的字段（钉值表的期望一侧，`assert_eq!(现算, 钉的.名)`）是钉值，不是没人算的读数
        if "asserted-fields-judged" not in os.environ.get("GATE_READING_DISCIPLINE_BREAK", "") and has_assertion and \
                re.search(r"assert(?:_eq|_ne)?!\s*\([^;]*?\.\s*" + name + r"\b", unit_text):
            continue
        if compound or borrowed or any(not LITERAL_ZERO.match(value) for value in assigned):
            continue
        initialised = [match.group(1).strip() for match in re.finditer(r"(?<![\w.])" + name + r"\s*:\s*([^,}\n]+)", unit_text)]
        initialised = [value for value in initialised if not re.fullmatch(INTEGER, value)]
        if initialised and all(LITERAL_ZERO.match(value) for value in initialised):
            frozen.append((f"{owner}.{field}", len(initialised)))
    return (None, seed_identifiers, hits, len(declared), frozen)

def examine_unit(members):
    """一份装置的全部源文件：交回每份文件一项 examine_one_source 的结果，次序同 members。文件之间用 `;` 加换行隔开拼成整份装置的文本，正则跨不过两份文件。"""
    texts, problems = {}, {}
    for path in members:
        try:
            texts[path] = blank_out(open(path, encoding="utf-8").read())
        except OSError as error:
            problems[path] = f"{path}：{error}"
    unit_text = ";\n".join(texts[path] for path in members if path in texts)
    return [examine_one_source(path, texts.get(path), problems.get(path), unit_text) for path in members]

# 按装置并行（进程数取登记的每台默认重活线程数，由这一格的 shell 交进来）；map 按 units 的次序交回，收的次序与一份接一份跑时相同。
worker_count = max(1, min(int(sys.argv[5]), len(units)))
if worker_count == 1:
    examined_units = [examine_unit(members) for members in units]
else:
    with multiprocessing.get_context("fork").Pool(worker_count) as pool:
        examined_units = pool.map(examine_unit, units)
examined = [result for unit_results in examined_units for result in unit_results]
if len(examined) != len(paths):
    print(f"  ✗ 派出去 {len(paths)} 份实验源码，收回 {len(examined)} 份的判定，对不上")
    print("     → 怎么办：这是这一格并行收束的缺陷；把上面的输出交给管门禁的一方，别当判过。")
    sys.exit(1)
unreadable = []
seed_folds, seed_identifier_count, files_without_seed = {}, 0, []
constant_readings, field_count, files_without_field = {}, 0, []
for path, (problem, seed_identifiers, hits, declared_count, frozen) in zip(paths, examined):
    if problem is not None:
        unreadable.append(problem)
        continue
    seed_identifier_count += seed_identifiers
    if not seed_identifiers:
        files_without_seed.append(os.path.basename(path))
    if hits:
        seed_folds[path] = hits
    field_count += declared_count
    if not declared_count:
        files_without_field.append(os.path.basename(path))
    if frozen:
        constant_readings[path] = frozen

if unreadable:
    reject(f"{len(unreadable)} 个实验源码读不动，这一轮对它们两条判据都没跑：",
           unreadable,
           ["怎么办：把读不动的原因修掉（权限、编码）再跑；读不动的文件在这一格里既不算判过也不算判红，",
            "          而一个整批读不动的目录会让两条判据都扫到 0 项，末尾照样报绿。"])

# ── 豁免表 ─────────────────────────────────────────────────────────────
owed_table = owed_library.read_owed_table(OWED_PATH)
owed_open = set(owed_table.open_names)
owed_unrecognised = owed_table.file_found and not owed_table.paid_heading_found

def read_lag(path, label):
    """读一张豁免表，返回 {(文件, 定位): (欠账编号, 行号)}；形状坏了的行当场登记成拒绝。"""
    rows, malformed = {}, []
    if not os.path.isfile(path):
        return rows
    for line_number, fields in lag_baseline.content_rows(open(path, encoding="utf-8").read()):
        if len(fields) != 4 or not all(field.strip() for field in fields):
            malformed.append(f"第 {line_number} 行：{chr(9).join(fields)}")
            continue
        source, locator, number, _reason = (field.strip() for field in fields)
        rows[(source, locator)] = (number, line_number)
    if malformed:
        reject(f"{label} 里 {len(malformed)} 行不是四列、或有空格子：", malformed,
               [f"怎么办：{path} 每行四列用制表符分隔：文件路径、定位、欠账编号、为什么还没改；四列都要有内容，理由不许省。"])
    return rows

def baseline_counts(path):
    """基准提交里这张表每个文件登记了几行；表在基准里不存在时返回 None（这次是初次登记）。"""
    if not base:
        return None
    # 「基准里没有这张表」与「git 取不到基准那一版」分开判（lib-lag-baseline.py 的 baseline_rows）：后者判红，不当成初次登记静默不比
    baseline, problem = lag_baseline.baseline_rows(base, path)
    if problem is not None:
        reject(f"{problem.sentence(None)}，「只缩不涨」这一条没比：",
               [problem.stderr.strip()[:200]],
               ["怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库没坏，多半是对象库缺了那一份）再跑；取不到、读不出基线都不是「初次登记」。"])
        return None
    if baseline is None:
        return None
    counts = {}
    for _line_number, fields in baseline:
        if len(fields) != 4:
            continue
        counts[fields[0].strip()] = counts.get(fields[0].strip(), 0) + 1
    return counts

def judge(label, owed_number, lag_path, violations, locator_of, what_to_fix, how_to_fix):
    """一条判据一段：没豁免的违规、指向空处的编号、对不上的登记行、涨了的登记行，四样都判完。"""
    lag = read_lag(lag_path, label)
    excused, unexcused = [], []
    for path, hits in sorted(violations.items()):
        for hit in hits:
            locator = locator_of(hit)
            if (path, locator) in lag:
                excused.append((path, locator))
            else:
                unexcused.append(f"{path}:{locator}    {hit[1] if isinstance(hit[1], str) else ''}".rstrip())
    if unexcused:
        reject(f"{label}：{len(unexcused)} 处违规没有登记在豁免表里：", unexcused,
               [f"怎么办：两条出路，别两边都不做。① 就地改掉——{how_to_fix}；",
                "               改了就要重跑这个实验并按新产物逐个回对正文引的数（门禁 research-build-and-replay 会逐字节判红直到重跑）。",
                f"               ② 今天改不动的，登记进 {lag_path}：四列写文件路径、定位、{owed_number}、为什么还没改。",
                f"               ⚠️ 这张表只缩不涨——新写的实验源码一律走出路 ①，{what_to_fix}。"])
    if lag and owed_unrecognised:
        reject(f"{label}：豁免表有 {len(lag)} 行要核欠账编号，而 {OWED_PATH} 里认不出「### 已还清」那一行标题，分不出哪些账还开着", [],
               ["怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；标题改了名或丢了就改回来，",
                "               别让这一格对着一张认不出的表判——认不出时连历史版本节里的表格行都会被算成开着的账。"])
    dangling = [f"{source}:{locator} 挂的是 {number}（第 {line_number} 行）"
                for (source, locator), (number, line_number) in sorted(lag.items())
                if number not in owed_open and not owed_unrecognised]
    if dangling:
        reject(f"{label}：豁免表里 {len(dangling)} 行挂的欠账编号不在 {OWED_PATH} 欠着那张表里：", dangling,
               ["怎么办：登记一条豁免，等于承认这一处今天还没改，那笔账要有人排期。",
                f"               编号写成欠着那张表里开着的一行（这一条是 {owed_number}）；",
                "               那笔账已经还清了，就把这些豁免行一起删掉——账还清了就不该再有豁免。"])
    live = {(path, locator_of(hit)) for path, hits in violations.items() for hit in hits}
    stale = [f"{source}:{locator}（第 {line_number} 行）"
             for (source, locator), (_number, line_number) in sorted(lag.items())
             if (source, locator) not in live]
    if stale:
        reject(f"{label}：豁免表里 {len(stale)} 行对不上这一轮扫出来的任何一处违规：", stale,
               ["怎么办：这一处已经改好了就删掉这一行——留着的豁免行会让人以为那个文件还欠着，",
                "               而它其实已经干净了（这张表只缩不涨，改好一处就删一行）；",
                "               只是定位挪了（行号随无关改动漂、字段改了名），就把定位那一列改成现在的值。"])
    baseline = baseline_counts(lag_path)
    grown = []
    if baseline is not None:
        current = {}
        for source, _locator in lag:
            current[source] = current.get(source, 0) + 1
        for item in split_family.family_growth(baseline, dict(sorted(current.items())), LAG_FAMILY_DEFECTS):
            if item.owner is None:
                grown.append(f"{item.path}：基准 {base} 里没有这个文件登记，它也不是基准里哪一份入口文件拆出来的部分文件，现在 {item.own} 行")
            else:
                grown.append(f"{item.path}：归基准 {base} 里的入口文件 {item.owner}（那一行基准登记 {item.allowed} 行），族内 {item.members} 个文件现在共 {item.total} 行")
    if grown:
        reject(f"{label}：豁免表涨了——{len(grown)} 个文件的登记行数比基准多：", grown,
               ["怎么办：这张表只缩不涨。新冒出来的违规一律就地改，不许加一行豁免把它按下去——",
                "               加得了一行，这一格对新写的实验源码就什么都拦不住了。",
                f"               真有非加不可的理由，那是一次规则变更：先改 {lag_path} 的表头注释与这一格的判据，说清为什么。"])
    return len(excused), len(lag), baseline is not None

seed_excused, seed_rows, seed_compared = judge(
    "C59（种子折叠成同一个状态）", "C59", SEED_LAG, seed_folds,
    lambda hit: str(hit[0]),
    "它们一处都不许出现在这张表里",
    "把 `seed | 1` 改成先乘法混淆再置位，例 `seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1`")
reading_excused, reading_rows, reading_compared = judge(
    "C60（恒定读数没有故障注入自证）", "C60", READING_LAG, constant_readings,
    lambda hit: hit[0],
    "它们一处都不许出现在这张表里",
    "要么让这个字段真的被写（把它算出来），要么删掉它——一个恒为 0 的读数不该进产物")

if failures:
    rejection_blocks.print_blocks(failures, continuation_indent="       ")
    sys.exit(1)

def listing(names):
    """一行塞几个名字，按显示宽度折行——名字本身不许被折断，折断的文件名 grep 不出来。"""
    lines, current = [], ""
    for index, name in enumerate(names):
        piece = name + ("、" if index + 1 < len(names) else "")
        if current and len(current) + len(piece) > 100:
            lines.append("        " + current); current = ""
        current += piece
    if current:
        lines.append("        " + current)
    return "\n".join(lines)

seed_hits = sum(len(hits) for hits in seed_folds.values())
reading_hits = sum(len(hits) for hits in constant_readings.values())
compared = "、".join(name for name, done in (("C59", seed_compared), ("C60", reading_compared)) if done)
compared = compared or "两张豁免表在基准里都还不存在，这一次是初次登记，没有基线可比"
print(f"  ✓ 实验源码纪律：扫了 {len(paths)} 个文件（读不动的 0 个）；"
      f"C59 查了 {seed_identifier_count} 处名字带 seed 的标识符，折叠写法 {seed_hits} 处（豁免表 {seed_rows} 行，放行 {seed_excused} 处）；"
      f"C60 查了 {field_count} 个整型结构体字段，恒为字面量 0 的 {reading_hits} 个（豁免表 {reading_rows} 行，放行 {reading_excused} 处）；"
      f"只缩不涨（基准 {base or '不是 git 仓，这一条没跑'}）：{compared}")
print(f"    这 {len(paths)} 个文件分属 {len(units)} 份装置，其中 {sum(1 for members in units if len(members) > 1)} 份有不止一个源文件")
print(f"    C59 对这 {len(files_without_seed)} 个文件没有对象可判（一处名字带 seed 的标识符都没有）：")
print(listing(files_without_seed) if files_without_seed else "        （没有）")
print(f"    C60 对这 {len(files_without_field)} 个文件没有对象可判（一个整型结构体字段都没有）：")
print(listing(files_without_field) if files_without_field else "        （没有）")
PY
}

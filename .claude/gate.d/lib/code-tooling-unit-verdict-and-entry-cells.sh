#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：reuse-selftests、function-units-two-implementations-agree、rust-name-collisions、mutation-list-items、break-switches-sample-only、entry-scripts、break-switch-headers。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# ── 格 function-units-two-implementations-agree：函数单元的两份实现对拍（crates/singlefs-unit-graph 的 unit-graph 与 scripts/gate/lib_function_units.py）
# 判据：被判的根上 unit-graph dump 与 python 那一份的 dump --language rust、--language wgsl 各一趟，按（文件，起行）配对，逐单元比种类、限定名、起止、指纹
#   （dump 的写法以 Rust 那一份为准，数据单元 data 行同样配对）；再对登记入口逐个比闭包成员与键。登记入口：被判的根 .claude/gate.d/stage-inputs.tsv 第二列、
#   scripts/gate/verus-proof-check.py 与 scripts/mutation/mutation-prune.py 的 JUDGING_ENTRIES 里写成 <文件>.rs::<限定名> 的那几个。python 那一份照 find_entry
#   定入口单元与目标，换成 unit-graph closure <crate>::<模块路径>::<名>（--target <种类>:<名>，lib 不写、build 只写种类；lib 与 bin 里按 cfg(test) 编的另给 --cfg test；
#   文档测试入口的路径是所挂条目或模块在 lib 模块树里的路径（lib_function_units.py 的 doctest_entry_paths 头一条），目标写不带名字的 --target doctest，
#   那个路径点到的全部文档测试一起当入口；lib 模块树之外的与挂在 crate 根上的写不出这一趟，列进解不成的），
#   两边同给一行工具链 host: x86_64-unknown-linux-gnu、不给依赖行；python 那一份的键照 unit-graph 的写法算（lib_function_units.py 的 unit_graph_key）。
#   差一处判红、点名单元（逐处至多列 12 行）。python 那一份解不成入口的（不在 crates/ 的工作区里，例 scripts/experiment-apparatus/ 下的实验装置）逐个列在成功那句里。
#   再走产品行为视图那一趟（lib_function_units.py 文件头「判定种类」）：unit-graph dump --judgment-kind product-behavior 与 python 那一份
#   dump_lines(…, judgment_kind=product-behavior, product_dump=<Rust 那一份交的 dump 行>) 照同样逐单元比，登记入口逐个比 closure 另带
#   --judgment-kind product-behavior 与 python 那一份 RustWorkspace(根, PRODUCT_BEHAVIOR, <同一份 dump 行>) 的闭包与键；差一处判红，点名的行带「产品行为视图：」。
#   python 那一份的剥法就是 Rust 交的这一份（不另写），这一趟比的是两份的配对、只给 Verus 看的单元停在哪、闭包的边与键。
# unit-graph 怎么来（function_units_build_unit_graph）：这一道所在仓的 crates/singlefs-unit-graph（连同它 include_str! 读的 .claude/heavy-thread-default）拷进
#   这一格自己的临时目录（${TMPDIR:-/tmp} 下 mktemp -d）里一个只有它的工作区，
#   起 cargo build --release --offline -p singlefs-unit-graph（编译目录是这一格自己的、跑完连同临时目录删，自己用完自己删，
#   不跨轮复用；不在仓的工作区里编，不碰仓的 Cargo.lock）；编出来的 unit-graph 每一趟直接起（内存上限归起这一道的 task-server，这一格不另套包装）。
#   两条判定都复用时不编（下面「按条复用」）；要编时这一格的程序经 export -f 起这个函数。
# 按条复用（共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：这一格两条判定——「两份 dump 逐单元相同」（原样与产品行为视图两趟的 dump）与「登记入口的闭包与键逐个相同」
#   （两趟的登记入口）；整格一条判不了复用三问的 ③（只一条时闭包内改动就是全部），按格里本来的两半分。种类 code-tooling.sh:function-units-two-implementations-agree
#   （prune-server 那一份库，unit-verdicts 族 singlefs-unit-verdicts/code-tooling.sh:function-units-two-implementations-agree，库根不进键）。
#   判的是这一道所在仓里的两份实现拿被判那棵树当数据跑，被判那棵树里没有程序入口：请求的入口写 -（scripts/gate/admission.py 文件头「判定请求」），键由这几样组成：
#   ① 判法（judge-root 是这一道所在的仓）：这一格的程序（function_units_agree_judge_code 打的 FUNCTION_UNITS_AGREE_PROGRAM）、python 那一份
#      这一格调到的那几个函数（PYTHON_SIDE_FUNCTIONS，各按定义它的那一份 scripts/gate/lib_function_units_*.py 登记）的闭包；
#   ② line:unit-graph=：crates/singlefs-unit-graph 整棵与 .claude/heavy-thread-default 的摘要（Rust 那一份；按源码文件折，注释也算，改一句就重编重比）；
#      工具链与依赖的版本不进键（D13（验证路线） 已定项 17：键只由代码函数、屏障与数据流组成）；
#   ③ 被判那棵树当数据读的：line:workspace=（仓根 Cargo.toml：工作区有哪几个成员）、line:sources=（python 那一份 discover_rust_crates 找到的每个 crate 目录整棵，
#      连同这些目录里 .rs 的字符串字面量写着的、落在 crate 目录之外的文件与 crates/ 之下的目录（.git 底下的不收），宁多收；两份实现要比的是这些文件原样，
#      不按哪一份切出的单元折）；
#   ④ 登记入口那一条另加 line:entries=（登记入口的条数与排好序的摘要）；样本根里只供样本的两样（.function-units-break、.unit-graph-recorded/rust-side.json）
#      与设着的 CODE_TOOLING_BREAK 各进一个 line:。
#   键相同、库里记着绿的那一半不比；两条都复用不编 unit-graph、一条都不比，报「全部复用」。判完照键记：判绿记绿，判红照同一把键记红、撤掉原有的绿；
#   每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（样本不抽）；失灵照全部现判、判红，prune-server 不可用（退 75）一条都不比、这一格退 75。
#   样本：function-units-two-implementations-agree-reuse-three-questions（green 的拷贝；复用三问：change.sh 从 stage-inputs.tsv 去掉一个登记入口，
#     affected 是登记入口那一条；unrelated.sh 在样本根加一份 crate 目录之外的文件）；
#     function-units-two-implementations-agree-red-over-green（key-red 的拷贝，.unit-verdict-sample/prior/ 是这一版逐字拷的一份、照判绿记两条；
#     red-over-green 下一律现判：登记入口那一条判红记红、撤绿，再算一次计划：复用 1 条、现判 1 条、原因「上次判红」）。
#   弄坏开关 CODE_TOOLING_BREAK=function-units-red-not-recorded：判红不交 --red-names（只记绿），-red-over-green 判错；
#     =function-units-plan-ignored：不按计划跳过复用的那一半、每趟两半都比，-reuse-three-questions 判错。
#   各份样本的 setup.sh 都 git init（判定库的名字取 git common-dir）。
# 只供样本（被判的根不是这一道所在的仓、也没设 GATE_STAGED_FROM 时才认）：
#   .unit-graph-recorded/rust-side.json  录好的 Rust 那一侧（dump 与每个入口那一趟 closure 的退出码与输出行，键是 unit-graph 在根之后的参数）：有它就不编、不起 unit-graph
#   .function-units-break                一行，当 python 那一份的 FUNCTION_UNITS_BREAK（真仓与 --staged 下这个变量一律置空）
#   重录（真 unit-graph 跑一趟）：先删掉样本根的 .unit-graph-recorded/，再
#     FUNCTION_UNITS_AGREE_RECORD=<样本根>/.unit-graph-recorded/rust-side.json bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree <样本根>
#     （key-red 那一份的录像重录之后把原样那一趟 lib.rs::entry 的 key 行改一位；product-view-red 的录像与 green 的相同）
# 样本：function-units-two-implementations-agree-green（一个小 crate：文件开头 #![…]、行内模块、include_str! 读的数据文件、bin 的 main、一块 verus!（spec 与 proof 函数、
#   带 requires / ensures 与 proof 块的执行函数），六个登记入口，两趟都判绿）；
#   -product-view-red（green 的拷贝，.function-units-break 是 product-view-kept-formal：python 产品行为视图照原样，dump 与闭包都点名只给 Verus 看的单元）；
#   -dump-red（.function-units-break 是 rust-module-heads-dropped：python 那一份不切模块头，dump 与闭包都点名 mod_head）；
#   -closure-red（rust-constants-dropped：dump 相同，闭包里 Rust 那一份多两个 const）；-key-red（录好的 entry 那一趟的键改了一位：闭包相同、键不同）；
#   -data-directory-green（两个登记入口：一个读 crates/ 之下的数据目录、一个 include_str! 读数据文件，Rust 那一侧是真 unit-graph 录下来的；闭包的数据行行首读目录的是 data-directory、读文件的是 data，两份逐字相同；
#   弄坏开关 function-units-data-kind-ignored 下读目录那一个入口的闭包两份不同、这份样本判红）；
#   -doctest-green（一个登记的文档测试入口：crates/fdocsample/src/lib.rs 里 documented 挂两个文档测试，入口登记在样本的 scripts/mutation/mutation-prune.py 的
#   JUDGING_ENTRIES（限定名带空格，stage-inputs.tsv 第二列按空白切、登记不了），Rust 那一侧录的是 closure fdocsample::documented --target doctest 两趟，两趟都判绿；
#   弄坏开关 function-units-doctest-target-named 下录好的输出里没有那一趟、点名那个入口判红）；
#   -cfg-one-side-red（crates/fsample 的 Cargo.toml 是 default = []，Rust 那一侧的录像是特性开着时录的：entry 里挂在 gpu 特性下的 gpu_boost 只在 Rust 闭包里，闭包两份不同）。
# 弄坏开关 CODE_TOOLING_BREAK：function-units-dump-unchecked 不比 dump（-dump-red 里「只有 Rust 那一份切出：mod_head」那条 want 找不到）；
#   function-units-closures-unchecked 不比闭包与键（-closure-red 与 -cfg-one-side-red 判绿）；function-units-keys-unchecked 闭包相同就不比键（-key-red 判绿）；
#   function-units-product-view-unchecked 不跑产品行为视图那一趟（-product-view-red 判绿，-green 成功那句里没有产品行为视图那一段）；
#   function-units-data-kind-ignored 闭包的数据行一律写 data（-data-directory-green 判红：读目录的入口，Rust 一侧行首是 data-directory）；
#   function-units-doctest-target-named 文档测试入口写 --target doctest:<名>（unit-graph 不认；-doctest-green 判红：「crates/fdocsample/src/lib.rs::documented::<doctest line 3>：unit-graph closure」那一行）。
cell_function_units_two_implementations_agree() {
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds function-units-two-implementations-agree function_units_agree_pass
  else
    function_units_agree_pass ""
  fi
}

# function_units_build_unit_graph <这一格的临时目录>：编一份 unit-graph（这一格文件头「unit-graph 怎么来」），成了末行打它的路径、退 0；
#   没成打 ✗ 与出路、退 1。这一格的程序有一条要现判时经 export -f 起它（读 STAGE_REPOSITORY，起它的那一句设进环境）
function_units_build_unit_graph() {
  local build_home="$1" crate_source manifest_text build_log kept_build_log
  crate_source="$STAGE_REPOSITORY/crates/singlefs-unit-graph"
  [[ -f "$crate_source/Cargo.toml" ]] || { echo "  ✗ 这一道所在的仓里没有 crates/singlefs-unit-graph"; echo "     → 怎么办：两份实现对拍要编 Rust 那一份；crate 被删了就从 git 找回来，改了名就改这一格的 crate_source。"; return 1; }
  mkdir -p "$build_home/workspace/crates" "$build_home/workspace/.claude" || { echo "  ✗ 建不了 $build_home"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; return 1; }
  cp -dR --preserve=timestamps,links "$crate_source" "$build_home/workspace/crates/" && cp --preserve=timestamps "$STAGE_REPOSITORY/.claude/heavy-thread-default" "$build_home/workspace/.claude/" \
    || { echo "  ✗ 拷不进 $build_home/workspace"; echo "     → 怎么办：看那个目录的空间与权限，修好再跑。"; return 1; }
  manifest_text=$'[workspace]\nmembers = ["crates/singlefs-unit-graph"]\nresolver = "2"\n\n[profile.release]\nopt-level = 3\noverflow-checks = true\ndebug-assertions = true\n'
  printf '%s' "$manifest_text" > "$build_home/workspace/Cargo.toml"
  build_log="$build_home/build.log"
  if ! CARGO_TARGET_DIR="$build_home/target" \
      cargo build --release --offline --manifest-path "$build_home/workspace/Cargo.toml" -p singlefs-unit-graph > "$build_log" 2>&1; then
    # build_home 连同里面的日志随这一格收尾删掉：判红时把日志拷到 TMPDIR 下留着，出路指那一份
    kept_build_log="$(mktemp "${TMPDIR:-/tmp}/singlefs-unit-graph-build.XXXXXX.log")" && cp -- "$build_log" "$kept_build_log" || kept_build_log="（没留下：拷不出来）"
    echo "  ✗ 编不出 unit-graph（cargo build --release -p singlefs-unit-graph，日志留在 $kept_build_log）：$(tail -n 3 "$build_log" | tr '\n' ' ' | clip_text 300)"
    echo "     → 怎么办：看那份日志，看完自己删；编译错就修 crates/singlefs-unit-graph。"
    return 1
  fi
  printf '%s\n' "$build_home/target/release/unit-graph"
}

# function_units_agree_pass <各条结局写到哪，空是不写>：这一格的一趟（建请求、查库、要现判时编 unit-graph、比没复用的那一半、记库、判）
function_units_agree_pass() {
  local units_library="$STAGE_REPOSITORY/scripts/gate/lib_function_units.py" python_break="" recording="" build_home pass_exit
  [[ -f "$units_library" ]] || { echo "  ✗ 找不到 $units_library"; echo "     → 怎么办：它随仓走（scripts/gate/lib_function_units.py），被删了就从 git 找回来；别在这一格另写一份。"; return 1; }
  if judged_root_is_sample; then
    [[ -f .function-units-break ]] && python_break="$(tr -d '[:space:]' < .function-units-break)"
    [[ -f .unit-graph-recorded/rust-side.json ]] && recording="$ROOT/.unit-graph-recorded/rust-side.json"
  fi
  build_home="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-unit-graph-build.XXXXXX")" || { echo "  ✗ 建不了这一格的临时目录"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; return 1; }
  export -f function_units_build_unit_graph
  if STAGE_REPOSITORY="$STAGE_REPOSITORY" FUNCTION_UNITS_BREAK="$python_break" python3 -c "$FUNCTION_UNITS_AGREE_PROGRAM" "$ROOT" "$units_library" "${CODE_TOOLING_BREAK:-}" \
       "${FUNCTION_UNITS_AGREE_RECORD:-}" "$recording" "$build_home" "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" \
       "$STAGE_NAME:function-units-two-implementations-agree" "$STAGE_REPOSITORY" "$(judged_root_is_sample && echo 1)" "$1" \
       "$FUNCTION_UNITS_AGREE_JUDGE_TOKEN" "$python_break"; then pass_exit=0; else pass_exit=$?; fi
  rm -rf -- "$build_home"
  return "$pass_exit"
}

# function-units-two-implementations-agree 格的程序（这一格文件头「按条复用」）：建两条请求、查库，要现判时取 Rust 那一侧（录好的或编出来的），
# 比没复用的那一半、记库、判。function_units_agree_judge_code 打它：键里的判法
FUNCTION_UNITS_AGREE_PROGRAM="$(cat <<'PY_FUNCTION_UNITS_AGREE'
import ast, collections, hashlib, importlib.util, json, os, re, shutil, subprocess, sys, tempfile, types
root, library, broken, record_path, recording_path, build_home = sys.argv[1:7]
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[7], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_root, sample, states_path, judge_token, python_break = (sys.argv[8], sys.argv[9], sys.argv[10], sys.argv[11] == "1", sys.argv[12],
                                                                              sys.argv[13], sys.argv[14])
HOST_LINE = "host: x86_64-unknown-linux-gnu"
ENTRY = re.compile(r"^(.+\.rs)::(.+)$")
LIST_LIMIT = 12
DUMP_ITEM, ENTRIES_ITEM = "两份 dump 逐单元相同", "登记入口的闭包与键逐个相同"
# python 那一份这一格调到的函数（键里的判法，按函数收闭包、不收整份程序：它的自证与命令行那几段不改这一格的结局），
# 各按定义它的那一份文件登记（scripts/gate/lib_function_units.py 只 import 再导出，那一份里没有这几个单元）
PYTHON_SIDE_FUNCTIONS = (("lib_function_units_closure_keys.py", "dump_lines"), ("lib_function_units_rust_workspace.py", "discover_rust_crates"),
                         ("lib_function_units_rust_workspace.py", "RustWorkspace.find_entry"), ("lib_function_units_rust_workspace.py", "RustWorkspace.lib_target"),
                         ("lib_function_units_rust_workspace.py", "doctest_entry_paths"), ("lib_function_units_rust_workspace.py", "rust_unit_is_test"),
                         ("lib_function_units_rust_closure.py", "rust_closure_of"), ("lib_function_units_closure_keys.py", "unit_graph_key"))
HOW = "bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree（unit-graph 与 lib_function_units.py 两份实现对拍）"
recording = None
if recording_path:
    with open(recording_path, encoding="utf-8") as handle:
        recording = json.load(handle)
binary = None   # 编出来的 unit-graph：有一条要现判、又没有录好的 Rust 那一侧时才编
recorded = {"dump": None, "closures": {}}


def rust(request):
    """unit-graph 的一趟 → (退出码, 输出行)；request 是根之后的参数（录好的输出照它认，不带根）。"""
    name = "\t".join(request)
    if recording is not None:
        entry = recording.get("dump") if request == ["dump"] else recording["closures"].get(name)
        if entry is None:
            return None, [f"录好的输出里没有这一趟：unit-graph {' '.join(request)}"]
        return entry["exit"], entry["lines"]
    completed = subprocess.run([binary, request[0], root, *request[1:]], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    lines = [line for line in completed.stdout.split("\n") if line]
    entry = {"exit": completed.returncode, "lines": lines}
    if request == ["dump"]:
        recorded["dump"] = entry
    else:
        recorded["closures"][name] = entry
    return completed.returncode, lines


specification = importlib.util.spec_from_file_location("lib_function_units_two_implementations", library)
units = importlib.util.module_from_spec(specification)
specification.loader.exec_module(units)
problems = {DUMP_ITEM: [], ENTRIES_ITEM: []}   # 这一趟比出来的不一致，按条记（这一格文件头「按条复用」）


def placed(lines):
    groups = collections.defaultdict(list)
    for line in lines:
        fields = line.split("\t")
        if len(fields) != 4 or fields[0] in ("refused", "key"):
            continue
        file, _colon, span = fields[2].rpartition(":")
        start = span.split("-")[0]
        groups[(file, int(start) if start.isdigit() else 0)].append(line)
    return groups


def compare_dumps(rust_lines, python_lines, label, sink):
    """两份 dump 按（文件，起行）配对逐单元比，不同的往 sink 记一条（label 是这一趟的前缀，原样那一趟是空的）→ (Rust 那一份的单元数, 各类的数)。"""
    rust_groups, python_groups = placed(rust_lines), placed(python_lines)
    rust_count = sum(len(lines) for lines in rust_groups.values())
    by_kind = collections.Counter("WGSL" if line.startswith("wgsl_") else "数据" if line.startswith("data\t") else "Rust"
                                  for lines in rust_groups.values() for line in lines)
    differences = []
    for place in sorted(set(rust_groups) | set(python_groups)):
        rust_only = sorted((collections.Counter(rust_groups.get(place, ())) - collections.Counter(python_groups.get(place, ()))).elements())
        python_only = sorted((collections.Counter(python_groups.get(place, ())) - collections.Counter(rust_groups.get(place, ()))).elements())
        if len(rust_only) == 1 and len(python_only) == 1:
            fields = [name for name, left, right in zip(("种类", "限定名", "起止", "指纹"), rust_only[0].split("\t"), python_only[0].split("\t")) if left != right]
            differences.append(f"{place[0]}:{place[1]} {'、'.join(fields)}不同：Rust「{rust_only[0][:150]}」python「{python_only[0][:150]}」")
            continue
        differences += [f"{place[0]}:{place[1]} 只有 Rust 那一份切出：{line[:200]}" for line in rust_only]
        differences += [f"{place[0]}:{place[1]} 只有 python 那一份切出：{line[:200]}" for line in python_only]
    if differences and broken != "function-units-dump-unchecked":
        sink.append(f"{label}dump 有 {len(differences)} 处单元两份不同（Rust {rust_count} 行、python {sum(len(lines) for lines in python_groups.values())} 行）")
        for line in differences[:LIST_LIMIT]:
            print(f"     {label}{line}")
        if len(differences) > LIST_LIMIT:
            print(f"     {label}…另 {len(differences) - LIST_LIMIT} 处")
    return rust_count, by_kind


# 登记入口
entries, sources = [], collections.Counter()
table = os.path.join(root, ".claude", "gate.d", "stage-inputs.tsv")
if os.path.isfile(table):
    with open(table, encoding="utf-8") as handle:
        for raw in handle:
            if raw.startswith("#") or "\t" not in raw:
                continue
            for token in raw.rstrip("\n").split("\t")[1].split():
                if ENTRY.match(token):
                    entries.append(token)
                    sources["stage-inputs.tsv"] += 1
for relative in ("scripts/gate/verus-proof-check.py", "scripts/mutation/mutation-prune.py"):
    path = os.path.join(root, relative)
    if not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8") as handle:
        tree = ast.parse(handle.read())
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "JUDGING_ENTRIES" for target in node.targets):
            for constant in ast.walk(node.value):
                if isinstance(constant, ast.Constant) and isinstance(constant.value, str) and ENTRY.match(constant.value):
                    entries.append(constant.value)
                    sources[relative] += 1
entries = list(dict.fromkeys(entries))
def compare_entries(workspace, label, view_request, sink):
    """登记入口逐个比两份的闭包成员与键，不同的往 sink 记一条（label 是这一趟的前缀，view_request 是 unit-graph closure 另带的选项）→ (比了几个, 解不成 Rust 入口的)。"""
    compared, unresolved = 0, []
    for entry in entries:
        path, qualified = ENTRY.match(entry).groups()
        try:
            unit, target_id = workspace.find_entry(path, qualified, None)
        except units.UsageError as error:
            unresolved.append(f"{entry}（{error}）")
            continue
        target = workspace.targets[target_id]
        module_path = list(target.files.get(unit.file, ((), []))[0])
        if target.kind == "doctest":
            # 文档测试入口照 unit-graph 的写法（crates/singlefs-unit-graph 的 resolve_entry_units）：路径是所挂条目或模块在 lib 模块树里的路径
            # （doctest_entry_paths 的头一条），那个路径点到的全部文档测试一起当入口，python 那一份的 rust_closure_of 照 doctest_entry_group 取同一组；
            # lib 模块树之外的与挂在 crate 根上的（unit-graph 的入口路径在 crate 名之后至少要一段）写不出这一趟，列进解不成的
            library = workspace.lib_target(target.crate)
            owner_paths = units.doctest_entry_paths(library, unit) if library is not None else []
            if not owner_paths or not owner_paths[0]:
                unresolved.append(f"{entry}（文档测试{'不在 lib 目标的模块树里' if not owner_paths else '挂在 crate 根上'}，unit-graph closure 写不出它的入口路径）")
                continue
            segments = list(owner_paths[0])
        elif unit.container is not None:
            container = unit.container
            type_name = sorted(container.self_types)[0] if container.kind == "impl" and container.self_types else container.short
            segments = module_path + list(unit.scope or ()) + [type_name, unit.short]
        else:
            segments = module_path + unit.name.split("::")
        request = ["closure", f"{target.crate.lib_name}::{'::'.join(segments)}"]
        if target.kind == "doctest":
            # unit-graph 的 TargetSelector::parse 只认不带名字的 doctest；弄坏开关 function-units-doctest-target-named：写 doctest:<名>
            request += ["--target", f"doctest:{target.name}" if broken == "function-units-doctest-target-named" else "doctest"]
        elif target.kind != "lib":
            request += ["--target", target.kind if target.kind == "build" else f"{target.kind}:{target.name}"]
        if target.kind in ("lib", "bin") and units.rust_unit_is_test(workspace, unit, target_id):
            request += ["--cfg", "test"]
        request += ["--toolchain-line", HOST_LINE] + list(view_request)
        exit_code, lines = rust(request)
        if exit_code not in (0, 3):
            sink.append(f"{label}{entry}：unit-graph {' '.join(request)} 退 {exit_code}（{' | '.join(lines[-2:])[:200]}）")
            continue
        try:
            closure = units.rust_closure_of(workspace, unit, target_id)
        except units.UsageError as error:
            sink.append(f"{label}{entry}：python 那一份收不了闭包（{error}）")
            continue
        compared += 1
        if broken == "function-units-closures-unchecked":
            continue
        python_members = [member.dump_text() for member in closure.order if not member.data]
        # 数据行的行首照读它的那一处的种类写（site[5]：读文件的 data，读目录的 data-directory，Rust 那一份 output.rs 的 data_placed_line 同样）；弄坏开关：一律写 data
        python_members += [f"{'data' if broken == 'function-units-data-kind-ignored' else site[5]}\t{site[2]}\t{site[0]}:{site[1]}-{site[1]}\t{site[3]}" for site in closure.data_sites]
        rust_members = [line for line in lines if not line.startswith(("refused\t", "key\t"))]
        rust_left = collections.Counter(rust_members) - collections.Counter(python_members)
        python_left = collections.Counter(python_members) - collections.Counter(rust_members)
        if rust_left or python_left:
            sink.append(f"{label}{entry}：闭包两份不同（Rust {len(rust_members)} 个、python {len(python_members)} 个；只在 Rust 里 {sum(rust_left.values())} 个、"
                            f"只在 python 里 {sum(python_left.values())} 个）")
            for line in sorted(rust_left.elements())[:LIST_LIMIT // 2]:
                print(f"     {label}{entry} 只在 Rust 闭包里：{line[:200]}")
            for line in sorted(python_left.elements())[:LIST_LIMIT // 2]:
                print(f"     {label}{entry} 只在 python 闭包里：{line[:200]}")
            continue
        if broken == "function-units-keys-unchecked":
            continue
        rust_key = next((line.split("\t", 1)[1] for line in lines if line.startswith("key\t")), None)
        python_key, python_refusals = units.unit_graph_key(closure, [HOST_LINE], [])
        if rust_key != python_key:
            sink.append(f"{label}{entry}：闭包相同、键不同（Rust {rust_key or '拒算'}，python {python_key or '拒算：' + '；'.join(refusal.text() for refusal in python_refusals[:2])}）")
    return compared, unresolved


STRING_LITERAL = re.compile(r'"((?:[^"\\\n]|\\.){1,300})"')


def sources_digest():
    """被判那棵树里两份实现当数据读的（这一格文件头「按条复用」③）：每个 crate 目录整棵，连同那些目录里 .rs 的字符串字面量写着的、
    落在 crate 目录之外（仓根之内）的文件与 crates/ 之下的目录（include_str! 一类与运行期读的字面路径，宁多收；.git 底下的不收）→ (摘要, crate 目录数, 目录之外另收的个数)。"""
    directories = sorted({crate.directory or "." for crate in units.discover_rust_crates(root)})
    inside = [directory for directory in directories if directory != "."]
    referenced = set()
    for directory in directories:
        crate_root = os.path.join(root, directory)
        for base, subdirectories, files in os.walk(crate_root):
            subdirectories[:] = sorted(name for name in subdirectories if name not in ("target", ".git"))
            for name in files:
                if not name.endswith(".rs"):
                    continue
                with open(os.path.join(base, name), encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
                for literal in STRING_LITERAL.findall(text):
                    if "/" not in literal and "." not in literal:
                        continue
                    for anchor in (base, crate_root, root):
                        candidate = os.path.normpath(os.path.join(anchor, literal.lstrip("/")))
                        relative = os.path.relpath(candidate, root)
                        if relative in (".", "..") or relative.startswith(".." + os.sep) or not os.path.exists(candidate) or relative.split(os.sep)[0] == ".git":
                            continue
                        # 目录只认 crates/ 之下的（两份实现只把那里的目录当数据目录读，lib_function_units.py 的 data_directory_unit）；别处的目录（.claude、scripts 这类）是字面量碰巧同名
                        if os.path.isdir(candidate) and relative.split(os.sep)[0] != "crates":
                            continue
                        if any(relative == place or relative.startswith(place + os.sep) for place in inside):
                            continue
                        referenced.add(relative)
    parts = [f"crate\0{directory}\0{rounds.content_digest(os.path.join(root, directory))}" for directory in directories]
    parts += [f"referenced\0{path}\0{rounds.content_digest(os.path.join(root, path))}" for path in sorted(referenced)]
    return hashlib.sha256("\n".join(parts).encode("utf-8", "surrogateescape")).hexdigest(), len(directories), len(referenced)


# 两条判定的请求（这一格文件头「按条复用」①–④）：入口写 -，判法在这一道所在的仓里算闭包
key_problem = ""
try:
    sources_key, crate_count, referenced_count = sources_digest()
except (OSError, ValueError, units.FunctionUnitsError, units.UsageError) as error:
    key_problem = f"被判那棵树的 crate 读不出（{error}）"
names = [DUMP_ITEM, ENTRIES_ITEM]
plan_scratch = tempfile.mkdtemp(prefix="function-units-agree-plan.")
verdicts = rounds.ItemVerdicts(admission, root, kind, plan_scratch, "code-tooling.sh 的 function-units-two-implementations-agree 格", sample, HOW,
                               broken != "function-units-red-not-recorded")
if key_problem:
    print(f"  ! 按条复用算不出键（{key_problem}）：两条都现判、都不记库", flush=True)
else:
    entries_text = "\n".join(sorted(entries))
    unit_graph = " ".join(rounds.content_digest(os.path.join(judge_root, *parts)) for parts in (("crates", "singlefs-unit-graph"), (".claude", "heavy-thread-default")))
    common = [judge_token, *(f"judge:scripts/gate/{file}::{name}" for file, name in PYTHON_SIDE_FUNCTIONS), f"judge-root:{judge_root}", f"line:unit-graph={unit_graph}",
              f"line:workspace={rounds.content_digest(os.path.join(root, 'Cargo.toml'))}",
              f"line:sources={sources_key}"]
    common += [f"line:{name}={value}" for name, value in (("break", broken), ("python-break", python_break)) if value]
    if recording is not None:
        common.append(f"line:recorded-rust-side={rounds.content_digest(recording_path)}")
    requests = {DUMP_ITEM: [DUMP_ITEM, "-", *common],
                ENTRIES_ITEM: [ENTRIES_ITEM, "-", *common, f"line:entries={len(entries)} {hashlib.sha256(entries_text.encode('utf-8', 'surrogateescape')).hexdigest()}"]}
    print(f"  · 键里被判那棵树的输入：crate 目录 {crate_count} 个整棵、目录之外字面量点到的 {referenced_count} 处、登记入口 {len(entries)} 个", flush=True)
    verdicts.plan(["\t".join(requests[name]) for name in names])
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print("  ✗ 两份实现对拍这一格没判：prune-server 不可用（上面那一句），两条都没比，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：两条都现判、都不记库", flush=True)
verdicts.report_plan(names, "（两份实现对拍的两半）")
judged = [name for name in names if verdicts.decision(name)[0] != rounds.REUSE]
ran_anyway = set()
if broken == "function-units-plan-ignored":   # 弄坏开关：每趟两半都比
    ran_anyway, judged = set(names) - set(judged), list(names)
judge_dump, judge_entries = DUMP_ITEM in judged, ENTRIES_ITEM in judged


def finish(exit_code):
    """收尾：各条结局写给复用三问、删算键的临时目录；比下来没红而记库时服务不可用退 75、按条复用失灵判红。"""
    verdicts.write_states(states_path, names, ran_anyway)
    shutil.rmtree(plan_scratch, ignore_errors=True)
    if exit_code == 0 and verdicts.unavailable:
        print("  ✗ 两份实现对拍：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree")
        sys.exit(75)
    if exit_code == 0 and verdicts.red:
        print("  ✗ 两份实现对拍判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）")
        print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree；"
              "不能复用就判红，不退回全部现判当绿")
        sys.exit(1)
    sys.exit(exit_code)


if not judged:
    print("  ✓ 两份实现对拍：两条判定都有键相同、判过绿的判定，全部复用，这一格没编 unit-graph、一条都没比")
    finish(0)
# Rust 那一侧：样本给了录好的就用它，不然编一份（经 export -f 交下来的 function_units_build_unit_graph）
if recording is None:
    done = subprocess.run(["bash", "-c", 'function_units_build_unit_graph "$1"', "function-units-build", build_home],
                          capture_output=True, text=True, errors="replace", stdin=subprocess.DEVNULL)
    built = done.stdout.strip().split("\n")[-1] if done.stdout.strip() else ""
    if done.returncode != 0 or not os.path.isfile(built):
        print(done.stdout.rstrip("\n") or f"  ✗ 编 unit-graph 那一步退 {done.returncode}、一行都没打（{done.stderr.strip()[-300:]}）")
        if "→ 怎么办" not in done.stdout:
            print("     → 怎么办：单跑这一格看报错：bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree；编译错就修 crates/singlefs-unit-graph")
        finish(1)
    binary = built
rust_count, by_kind = 0, collections.Counter()
if judge_dump:
    # dump：按（文件，起行）配对，逐单元比种类、限定名、起止、指纹
    exit_code, rust_lines = rust(["dump"])
    if exit_code != 0:
        problems[DUMP_ITEM].append(f"unit-graph dump 没跑成（退 {exit_code}）：{' | '.join(rust_lines[-3:])[:400]}")
    else:
        python_lines = units.dump_lines(root, "rust")[0] + units.dump_lines(root, "wgsl")[0]
        rust_count, by_kind = compare_dumps(rust_lines, python_lines, "", problems[DUMP_ITEM])
compared, unresolved = compare_entries(units.RustWorkspace(root), "", [], problems[ENTRIES_ITEM]) if judge_entries else (0, [])
# 产品行为视图那一趟（lib_function_units.py 文件头「判定种类」）：python 那一份的产品行为视图吃的就是 Rust 那一份交的 dump，比的是配对、只给 Verus 看的单元停在哪、闭包的边与键
product_summary = "产品行为视图那一趟没比（弄坏开关 function-units-product-view-unchecked）"
if broken != "function-units-product-view-unchecked":
    product_request = ["dump", "--judgment-kind", "product-behavior"]
    exit_code, rust_product_lines = rust(product_request)
    if exit_code != 0:
        for name in judged:
            problems[name].append(f"产品行为视图：unit-graph {' '.join(product_request)} 没跑成（退 {exit_code}）：{' | '.join(rust_product_lines[-3:])[:300]}")
        product_summary = "产品行为视图那一趟的 dump 没跑成"
    else:
        product_parts = []
        if judge_dump:
            python_product_lines = (units.dump_lines(root, "rust", judgment_kind=units.PRODUCT_BEHAVIOR, product_dump=rust_product_lines)[0]
                                    + units.dump_lines(root, "wgsl")[0])
            product_count, _product_kinds = compare_dumps(rust_product_lines, python_product_lines, "产品行为视图：", problems[DUMP_ITEM])
            # 只给 Verus 看的单元产品行为视图压根不切（lib_function_units.py 文件头「判定种类」），两份都不列的个数 = 形式化视图那一趟比的个数 − 这一趟比的个数
            product_parts.append(f" dump 比了 {product_count} 个单元（只给 Verus 看的 {rust_count - product_count} 个两份都不列）")
        if judge_entries:
            product_workspace = units.RustWorkspace(root, units.PRODUCT_BEHAVIOR, rust_product_lines)
            product_compared, _product_unresolved = compare_entries(product_workspace, "产品行为视图：", ["--judgment-kind", "product-behavior"], problems[ENTRIES_ITEM])
            product_parts.append(f"登记入口 {product_compared} 个，闭包与键逐个相同")
        product_summary = "产品行为视图那一趟" + "、".join(product_parts) + ("" if judge_entries else "，逐个相同")
if record_path and recording is None:
    os.makedirs(os.path.dirname(record_path) or ".", exist_ok=True)
    with open(record_path, "w", encoding="utf-8") as handle:
        json.dump(recorded, handle, ensure_ascii=False, indent=0, sort_keys=True)
    print(f"  · 录下了 Rust 那一侧：{record_path}（dump 一趟、产品行为视图的 dump 与各趟 closure 共 {len(recorded['closures'])} 趟）")
# 照键记库（这一格文件头「按条复用」）：现判的那几条，比出不一致的记红、撤掉同一把键原有的绿，没有的记绿
red_names = [name for name in judged if problems[name]]
verdicts.record([name for name in judged if not problems[name]], red_names, final=True)
if red_names:
    verdicts.reread_after_red()
named = "、".join(f"{source} {count} 个" for source, count in sorted(sources.items())) or "一个都没有"
unresolved_text = f"解不成 Rust 入口的 {len(unresolved)} 个" + (f"：{'；'.join(unresolved)}" if unresolved else "")
found = problems[DUMP_ITEM] + problems[ENTRIES_ITEM]
if found:
    compared_parts = ([f"比了 dump {rust_count} 个单元"] if judge_dump else []) + ([f"登记入口 {compared} 个（{named}）"] if judge_entries else [])
    print(f"  · {'、'.join(compared_parts)}；{product_summary.replace('，闭包与键逐个相同', '')}" + (f"；{unresolved_text}" if judge_entries else ""))
    for problem in found:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）有 {len(found)} 处不一致")  # gate-lint:summary
    print("     → 怎么办：照上面点名的单元查两边的切分与闭包规则，以 crates/singlefs-unit-graph 为准改 python 那一份（定的口径变了就两边一起改）；"
          "单跑这一格：bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree")
    finish(1)
dump_part = (f"dump 比了 {rust_count} 个单元（{'、'.join(f'{kind} {count}' for kind, count in sorted(by_kind.items()))}），逐个相同" if judge_dump
             else "dump 那一条复用（键相同、库里判过绿）")
entries_part = f"登记入口 {compared} 个（{named}），闭包与键逐个相同" if judge_entries else "登记入口那一条复用（键相同、库里判过绿）"
print(f"  ✓ 两份实现对拍：{dump_part}；{entries_part}；{product_summary}" + (f"；{unresolved_text}" if judge_entries else ""))
finish(0)
PY_FUNCTION_UNITS_AGREE
)"
function_units_agree_judge_code() { printf '%s\n' "$FUNCTION_UNITS_AGREE_PROGRAM"; }

# ── 格 rust-name-collisions：同一个 Rust 编译目标里不同模块各自定义的同名条目判红（撞名必须修复，不得撞名）
# 判法只在 scripts/gate/rust-name-collisions.py（用这一道所在仓里那一份，被判的根照传；它退 77（根下没有 Rust 编译目标）这一格就退 77）；
#   单元与模块树照 scripts/gate/lib_function_units.py 的 RustWorkspace 取，与单元图 R-裸名按名字连的那一份索引同源；算不算撞名的依据写在它的文件头。
# 为什么归这一道、不归 code-source-discipline：撞名坏的是判定的键（R-裸名在同一个目标里只按名字连，判器键收进流水线的私有 fn），
#   这一道已管单元图（function-units-two-implementations-agree），也已扫 crates/（heavy-thread-defaults、memory-bounded-parallelism）；
#   code-source-discipline 那四格是交回前的命名检查，判名字本身写得好不好，不比两个名字撞没撞。
# 样本：rust-name-collisions-red（工作区里一个 crate：ledger 与 journal 各一个私有 fn private_twin、journal::tests 与 inventory::tests 各一个 helper_twin、
#   lib.rs 里两个行内模块各一个 shared_inline）判红，三处与六个文件:行都点名；rust-name-collisions-green（两个 impl 里同名的固有方法与 trait 规定的 fmt、
#   同一个模块里 cfg 分开的两份、lib 与 src/bin 下各一份同名私有 fn）判绿。弄坏开关归判法自己（RUST_NAME_COLLISIONS_BREAK，自证自带开关轮，文件头逐条列）。
cell_rust_name_collisions() {
  local checker="$STAGE_REPOSITORY/scripts/gate/rust-name-collisions.py"
  [[ -f "$checker" ]] || { echo "  ✗ 找不到 $checker"; echo "     → 怎么办：它随仓走（scripts/gate/rust-name-collisions.py），被删了就从 git 找回来；别在这一格另写一份判法。"; exit 1; }
  python3 "$checker" "$ROOT"
}

# ── 格 reuse-selftests：登记表 .claude/gate.d/verdict-library-scripts.tsv 里每份带判定库的脚本，自证的复用三问都过；声称复用的都登记了
#
# 判法在 scripts/gate/lib_reuse_checks.py 的 audit（文件头「四」：登记表的写法、射程 # scan:、本次未跑与不算、只在临时服务上跑的包一层起）与它判一行过没过的
#   audit_row_judgement；这一格经 audit 的 verdicts 按条复用（下面「按条复用」），另只取并行度（每台默认重活线程数 heavy_thread_default，同 research-script-selftests）、
#   交被判的根。用的是这一道所在仓里那一份 lib_reuse_checks.py（样本根里没有它）；现跑的自证在被判的根里起，样本根里的是样本自己的假自证。
#   登记表与射程的静态几样（四列写对、同一路径两行、指到的路径在、漏登记、声称复用登记成无判定库、格名）只读文本，每趟照判，不入库。
# 按条复用（共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：登记表里要现跑自证的一行一条判定，名字是那一行的路径，种类 code-tooling.sh:reuse-selftests（prune-server 那一份库，
#   unit-verdicts 族 singlefs-unit-verdicts/code-tooling.sh:reuse-selftests，库根不进键）。键（scripts/gate/admission.py verdict-plan 算，不带行号）由这几样组成：
#   ① 那一行自证命令起的程序的单元闭包：入口是头一段的被测脚本，&& 后面几段的被测脚本各进一个 judge:；lib_reuse_checks.py sample <阶段> <样本> 那一段起的程序
#      是 .claude/gate.d/<阶段>，那份样本目录整棵的摘要另进 line:sample=（后面几段的 judge: 与判法同在 judge-root 那棵树里算：真仓与 --staged 下就是被判的根；
#      样本的自证命令只写一段、不写 sample）；
#   ② 判法：这一道所在仓里 lib_reuse_checks.py 的 audit_row_judgement、run_command（有 sample 那一段的另加 run_sample）与这一格的程序（reuse_selftests_judge_code
#      打的 REUSE_SELFTESTS_PROGRAM）；
#   ③ 那一行前四列各一个 line:（路径、自证命令、判定库放哪、复用三问格名；说明那一列 audit 不读，不进）；
#   ④ 自证只在临时 prune-server 上跑的另加 file:scripts/prune-server/prune_server_client_selftest.py（包装本身）。二进制路径不进键。
#   键相同、库里记着绿的不起、照过算（复用）；对不上的、记着红的（原因以「上次判红」起头）现跑；认不出起的程序的那一行算不出键，照跑、不记。
#   要临时服务的那几行有一行要现跑才取一次 prune-server 二进制（code_tooling_prune_server_binary，经 export -f 交给这一格的程序起；全部复用就不编）。
#   判完一趟记一次：判绿记绿，判红照同一把键记红、撤掉原有的绿（verdict-record --red-names）；每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（样本不抽）；
#   失灵（verdict-plan 退非 0、没打 PLAN、哨兵报红）照全部现跑、判红，prune-server 不可用（退 75）一份都不起、这一格退 75（判法在 admission.py 的 plan-failure）。
#   样本：reuse-selftests-reuse-three-questions（三行假自证；复用三问：change.sh 改 beta.sh 打的那一句，affected 是它那一行；unrelated.sh 只给 gamma 那一行加说明列）；
#     reuse-selftests-red-over-green（两行：alpha.sh 打三问都过、broken.sh 不打规范输出；.unit-verdict-sample/prior/ 是这一版逐字拷的一份、照判绿记两条，
#     red-over-green 下一律现跑：broken.sh 判红记红、撤绿，再算一次计划：复用 1 条、现判 1 条、原因「上次判红」）。
#   弄坏开关 CODE_TOOLING_BREAK=reuse-selftests-red-not-recorded：判红不交 --red-names（只记绿），reuse-selftests-red-over-green 判错；
#     =reuse-selftests-plan-ignored：不按计划跳过复用的行、要现跑的全部现跑（每趟整份重跑），reuse-selftests-reuse-three-questions 判错。
# 样本：reuse-selftests-green（一份假自证打三问都过的规范输出，一份「无判定库：待立」报本次未跑，一份「不算」）判绿；
#   reuse-selftests-red（一份假自证只退 0、不打规范输出，一份注释写了判定库却没登记，一份指到不存在的路径）判红、三样都点名。各份 setup.sh 都 git init（判定库的名字取 git common-dir）。
# 弄坏开关 CODE_TOOLING_BREAK=reuse-selftest-line-unchecked：折成 REUSE_CHECKS_BREAK=audit-line-unchecked，自证退 0 就算过：
#   reuse-selftests-red 里「输出里没有 REUSE_SELFTEST script=scripts/gate/no-line.sh」那条 want 找不到（另两样照红）。
cell_reuse_selftests() {
  local jobs
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），现跑自证的并行度定不了，一份都没判"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds reuse-selftests reuse_selftests_pass
  else
    reuse_selftests_pass ""
  fi
}

# reuse_selftests_pass <各条结局写到哪，空是不写>：这一格的一趟（建请求、查库、现跑没复用的那几行、记库、判）；jobs 取调用它的那一格的
reuse_selftests_pass() {
  local break_items="" needs_server=0 registered build_scratch pass_exit
  [[ "${CODE_TOOLING_BREAK:-}" == reuse-selftest-line-unchecked ]] && break_items=audit-line-unchecked
  # 登记表里有自证只在临时 prune-server 上跑的脚本（要临时服务的表里登记成 required，audit 包一层起）：有一行要现跑才取一次二进制
  temporary_service_table_load || return 1
  while IFS= read -r registered; do
    [[ -f "$registered" ]] && temporary_service_is_required "$registered" && needs_server=1
  done < <(awk -F'\t' '!/^#/ && NF >= 2 { print $1 }' .claude/gate.d/verdict-library-scripts.tsv 2>/dev/null)
  build_scratch="$(mktemp -d)" || { echo "  ✗ 建不了编 prune-server 的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格"; return 1; }
  # 程序要编二进制时起 bash -c code_tooling_prune_server_binary：函数经 export -f 交下去，它读的 ROOT 在起程序的那一句里设进环境
  export -f code_tooling_prune_server_binary code_tooling_prune_server_build_into
  if ROOT="$ROOT" REUSE_CHECKS_BREAK="$break_items" python3 -c "$REUSE_SELFTESTS_PROGRAM" "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" \
       "$STAGE_NAME:reuse-selftests" "$STAGE_REPOSITORY" "$ROOT" "$jobs" "${CODE_TOOLING_BREAK:-}" "$(judged_root_is_sample && echo 1)" "$1" "$needs_server" "$build_scratch" \
       "$REUSE_SELFTESTS_JUDGE_TOKEN"; then pass_exit=0; else pass_exit=$?; fi
  rm -rf -- "$build_scratch"
  return "$pass_exit"
}

# reuse-selftests 格的程序（这一格文件头「按条复用」）：建请求、经 ItemVerdicts 查库，交 lib_reuse_checks.py 的 audit 现跑没复用的那几行、记库，定这一格的退出码。
# reuse_selftests_judge_code 打它：键里的判法（改一句就作废这一格的全部判定）
REUSE_SELFTESTS_PROGRAM="$(cat <<'PY_REUSE_SELFTESTS'
import importlib.util, os, shlex, shutil, subprocess, sys, tempfile, types
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[1], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_root, root, jobs, broken = sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5], int(sys.argv[6]), sys.argv[7]
sample, states_path, needs_server, build_scratch, judge_token = sys.argv[8] == "1", sys.argv[9], sys.argv[10] == "1", sys.argv[11], sys.argv[12]
specification = importlib.util.spec_from_file_location("lib_reuse_checks_for_reuse_selftests", os.path.join(judge_root, "scripts", "gate", "lib_reuse_checks.py"))
checks = importlib.util.module_from_spec(specification)
specification.loader.exec_module(checks)
TEMPORARY_SERVICE_WRAPPER = "scripts/prune-server/prune_server_client_selftest.py"
HOW = "bash .claude/gate.d/code-tooling.sh --check reuse-selftests（现跑登记表那一行的自证命令，认规范输出那一行）"


class ServiceUnavailable(Exception):
    """prune-server 不可用：这一格一份自证都不起、退 75。"""


def programs_of(command):
    """自证命令起的程序（相对被判的根）与样本目录：逐段（&& 隔开）取被测脚本；lib_reuse_checks.py sample <阶段> <样本> 那一段取 .claude/gate.d/<阶段>
    与那份样本目录。认不出交 (None, None)。"""
    programs, samples = [], []
    for segment in command.split("&&"):
        script, _assigned = checks.command_script(segment)
        if script is None:
            return None, None
        words = shlex.split(segment)
        rest = next((words[index + 1:] for index, word in enumerate(words) if os.path.normpath(word) == script), [])
        if script == "scripts/gate/lib_reuse_checks.py" and rest[:1] == ["sample"] and len(rest) >= 3:
            programs.append(f".claude/gate.d/{rest[1]}")
            samples.append(f".claude/gate.d/fixtures/{rest[1]}/{rest[2]}")
        else:
            programs.append(script)
    return programs, samples


def request_line(path, command, library, cell, needs_service):
    """登记表一行的判定请求（这一格文件头「按条复用」①–④）；认不出起的程序交 None。"""
    programs, samples = programs_of(command)
    if not programs:
        return None
    judges = ["audit_row_judgement", "run_command"] + (["run_sample"] if samples else [])
    tokens = [f"judge:scripts/gate/lib_reuse_checks.py::{name}" for name in judges] + [judge_token, f"judge-root:{judge_root}"]
    tokens += [f"judge:{program}" for program in dict.fromkeys(programs[1:]) if program != programs[0]]
    tokens += [f"line:path={path}", f"line:command={command}", f"line:library={library}", f"line:cell={cell}"]
    tokens += [f"line:sample={rounds.content_digest(os.path.join(root, directory))}" for directory in samples]
    if needs_service:
        tokens.append(f"file:{TEMPORARY_SERVICE_WRAPPER}")
    return "\t".join([path, programs[0], *tokens])


class AuditVerdicts:
    """lib_reuse_checks.py 的 audit 收的 verdicts（audit 的文档串）：decide 建请求、查库，record 照键记库。"""

    def __init__(self, scratch):
        self.items = rounds.ItemVerdicts(admission, root, kind, scratch, "code-tooling.sh 的 reuse-selftests 格", sample, HOW,
                                         broken != "reuse-selftests-red-not-recorded")
        self.requests, self.names, self.ran_anyway, self.red_recorded = {}, [], set(), False

    def decide(self, rows):
        self.names = [row[0] for row in rows]
        for path, command, _script, library, cell, needs_service in rows:
            line = request_line(path, command, library, cell, needs_service)
            if line is not None:
                self.requests[path] = line
        if self.requests:
            self.items.plan(list(self.requests.values()))
            if self.items.unavailable:
                raise ServiceUnavailable()
            if not self.items.ok:
                print(f"  ! 按条复用算不出（{self.items.problem}）：这一趟要现跑的每一行都现跑、一条都不记库", flush=True)
        decisions = {path: self.items.decision(path) if path in self.requests else (rounds.JUDGE, "算不出键：自证命令里认不出起的程序")
                     for path in self.names}
        _judged, reused, _refused = self.items.report_plan(self.names, "要现跑的自证")
        if broken == "reuse-selftests-plan-ignored":
            self.ran_anyway = {path for path, (decision, _reason) in decisions.items() if decision == rounds.REUSE}
            return {}
        if reused == len(self.names):
            print("  · 全部复用，这一格一份自证都没起", flush=True)
        return decisions

    def record(self, green, red):
        red = [path for path in red if path in self.requests]
        self.items.record([path for path in green if path in self.requests], red, final=True)
        self.red_recorded = bool(red)


def server_binary():
    """audit 要现跑的里有要临时服务的才调（调一次）：经 export -f 交下来的 code_tooling_prune_server_binary 取一份 prune-server 二进制。"""
    if not needs_server:
        return None, "要临时服务的表里没有一行点到这张登记表里的脚本，这一格不取二进制"
    done = subprocess.run(["bash", "-c", 'code_tooling_prune_server_binary "$1"', "code-tooling-reuse-selftests", build_scratch],
                          capture_output=True, text=True, errors="replace", stdin=subprocess.DEVNULL)
    lines = done.stdout.strip().splitlines()
    if done.returncode != 0 or not lines:
        return None, (done.stderr.strip().splitlines() or [f"退 {done.returncode}"])[-1][:300]
    return lines[-1], ""


scratch = tempfile.mkdtemp(prefix="reuse-selftests-plan.")
verdicts = AuditVerdicts(scratch)
try:
    audit_exit = checks.audit(root, jobs, checks.DEFAULT_AUDIT_TIMEOUT_SECONDS, server_binary=server_binary, verdicts=verdicts)
except ServiceUnavailable:
    shutil.rmtree(scratch, ignore_errors=True)
    print("  ✗ reuse-selftests 这一格没判：prune-server 不可用（上面那一句），一份自证都没起，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check reuse-selftests")
    sys.exit(75)
if verdicts.red_recorded:
    verdicts.items.reread_after_red()
verdicts.items.write_states(states_path, list(verdicts.requests), verdicts.ran_anyway)
shutil.rmtree(scratch, ignore_errors=True)
if verdicts.items.unavailable:
    print("  ✗ reuse-selftests：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check reuse-selftests")
    sys.exit(75)
if verdicts.items.red:
    print("  ✗ reuse-selftests 判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）")
    print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check reuse-selftests；不能复用就判红，不退回全部现跑当绿")
    sys.exit(1)
sys.exit(audit_exit)
PY_REUSE_SELFTESTS
)"
reuse_selftests_judge_code() { printf '%s\n' "$REUSE_SELFTESTS_PROGRAM"; }

# ── 格 mutation-list-items：还 03-third-txn.md 待优化第 57 行（变异门禁 --list-items 在主工作区抛 PicklingError、取不出键对不上的行数）。
# 判据：在被判的根上跑这一道所在仓里的 scripts/mutation/crates-mutation-rows.py --list-items <根>（门禁 crates-mutation-replay.sh --list-items exec 的就是它，
#   不编、不跑，只起 git），要：退 0；输出里有「· 键对不上 N 条」那一行（键算出来了）；没有「键对不对得上没算出来」、没有 Traceback 与 PicklingError。
#   键算不出来时那份脚本照旧退 0、只打一句「没算出来」（列项照列），平常跑看不出来，提交前要取的数就取不到；这一格在真仓上跑它，把这一种当红。
#   PicklingError 的来源（算闭包的工作进程把按路径载入的模块里的异常往回传）在 crates-mutation-rows.py 里由它的自证（弄坏开关 closure-error-masked）证红；
#   这一格补的是「在真仓上跑」那一半。真仓上一趟 78 到 168 秒（在副本上量了两趟，机器同时在跑别的变异；5260 行）。
# 被判的根里没有 crates/mutations.tsv：退 77。
# 按条复用（共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：变异表的行按改的文件（第二列）分组，一组一条判定，名字是那份文件的路径；一行一条也行，按文件分是因为
#   一行的键里能分开的只有那一行的原文（源码那一项是 crates/ 整棵，见 ②），六千多行一行一条只是把查库、记库放大几十倍。种类 code-tooling.sh:mutation-list-items
#   （prune-server 那一份库，unit-verdicts 族 singlefs-unit-verdicts/code-tooling.sh:mutation-list-items，库根不进键）。
#   判的是这一道所在仓里的 crates-mutation-rows.py 拿被判那棵树当数据跑：请求的入口写 -（scripts/gate/admission.py 文件头「判定请求」），键由这几样组成：
#   ① 判法（judge-root 是这一道所在的仓）：这一格的程序（mutation_list_items_judge_code 打的 MUTATION_LIST_ITEMS_PROGRAM）、scripts/mutation/crates-mutation-rows.py 的
#      command_list_items（--list-items 那一路）的闭包；
#   ② 被判那棵树当数据读的：line:sources=（crates/ 下除变异表之外的每一项整棵）、line:shared=（仓根 Cargo.toml、
#      .claude/gate.d/stage-inputs.tsv、.claude/gate.d/crash-point-replay-items.tsv、.claude/heavy-thread-default、scripts/crash/crash-amplification.sh，宁多收）；
#      与变异那一族同口径，构建环境与版本不进键：.cargo/ 是构建环境、rust-toolchain(.toml) 与 rustc、cargo 的版本是工具链版本、Cargo.lock 是依赖版本；
#      这一格判的是「--list-items 在被判的根上算得出键」，那份脚本算键不读它们（行键只有被改坏的单元、原文、替换文与点名测试的闭包），仓根 Cargo.toml 的成员与特性决定闭包怎么收，所以留；
#   ③ line:rows=：这一组的行原文照表里的次序连起来的摘要（行号不进）；设着的 CODE_TOOLING_BREAK 里这一格自己的开关（MUTATION_LIST_ITEMS_OWN_SWITCHES）另进一个 line:。
#   这一格只认自己的开关：别的格的开关值不交给这一格的程序、不进键，这一格起的 python（连同它起的 admission.py 与 crates-mutation-rows.py --list-items）
#   环境里不带 CODE_TOOLING_BREAK（env -u）。样本档设上别的格的开关时，这一格的样本照不设时那样判，stage-break-switch-replay 比点名的与判错的样本才对得上。
#   键相同、库里记着绿的组不判；全部复用就不起 --list-items，报「全部复用」。有一组要现判就整张表跑一趟（--list-items 本来就整张表算），
#   结局记给要现判的那几组：没跑通都记红、撤掉同一把键原有的绿，跑通都记绿。每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（样本不抽）；
#   失灵照现判、判红，prune-server 不可用（退 75）不起、这一格退 75。
#   样本：mutation-list-items-reuse-three-questions（setup.sh 现写的小工作区两份源文件、各一行变异；复用三问：change.sh 改 second.rs 那一行的替换文，
#     affected 是 crates/tiny/src/second.rs 那一组；unrelated.sh 在样本根加一份 crates/ 之外的文件）；
#     mutation-list-items-red-over-green（red 的同一个小工作区，.unit-verdict-sample/prior/ 是这一版逐字拷的一份、照判绿记那一组；red-over-green 下一律现判：
#     没跑通、记红撤绿，再算一次计划：复用 0 条、现判 1 条、原因「上次判红」）。各份样本的 setup.sh 都 git init。
#   弄坏开关 CODE_TOOLING_BREAK=mutation-list-items-red-not-recorded：判红不交 --red-names（只记绿），-red-over-green 判错；
#     =mutation-list-items-plan-ignored：不按计划跳过、每趟都起 --list-items，-reuse-three-questions 判错；
#     =mutation-list-items-build-environment-keyed：键另收 .cargo/、rust-toolchain(.toml)、Cargo.lock 与 rustc -vV、cargo -V（构建环境与版本回到键里），
#       -build-environment-reuse-three-questions 判错（那份样本的 unrelated.sh 改 .cargo/config.toml 与 Cargo.lock，复用三问的「闭包外改动」要复用，开关下现判）。
#   样本：mutation-list-items-build-environment-reuse-three-questions（reuse-three-questions 的同一个小工作区，setup.sh 另写 .cargo/config.toml 与 Cargo.lock；
#     unrelated.sh 改这两份：键不变、复用）。
# 样本：mutation-list-items-red（setup.sh 现写一个小工作区，变异表一行点名只在 gpu 特性下编的测试、cargo 参数没带特性：键算不出来，那份脚本退 0，这一格判红）；
#   mutation-list-items-green（同一个小工作区，变异表一行点名普通测试：报「键对不上 1 条」）。
# 弄坏开关 CODE_TOOLING_BREAK=list-items-exit-only：只看退出码，mutation-list-items-red 判绿（期望退出 1，实测 0）。
cell_mutation_list_items() {
  [[ -f crates/mutations.tsv ]] || { echo "  ! 被判的根里没有 crates/mutations.tsv，这一格无对象可判"; exit 77; }
  [[ -f "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" ]] || { echo "  ✗ 找不到 $STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py"; echo "     → 怎么办：它随仓走（scripts/mutation/crates-mutation-rows.py），被删了就从 git 找回来。"; exit 1; }
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds mutation-list-items mutation_list_items_pass
  else
    mutation_list_items_pass ""
  fi
}

# 这一格自己的开关（文件头开关表里点名 mutation-list-items 样本的那三条）：设着的 CODE_TOOLING_BREAK 里只这几项交给这一格（这一格文件头「按条复用」③）
MUTATION_LIST_ITEMS_OWN_SWITCHES=(list-items-exit-only mutation-list-items-red-not-recorded mutation-list-items-plan-ignored mutation-list-items-build-environment-keyed)
# mutation_list_items_pass <各条结局写到哪，空是不写>：这一格的一趟（建请求、查库、有一组要现判就起 --list-items、记库、判）。
#   别的格的开关不交、不进键，起的 python 环境里不带 CODE_TOOLING_BREAK（这一格文件头「按条复用」③）
mutation_list_items_pass() {
  local given_switch own_switch own_switches=""
  local -a given_switches=()
  IFS=, read -r -a given_switches <<<"${CODE_TOOLING_BREAK:-}"
  for given_switch in ${given_switches[@]+"${given_switches[@]}"}; do
    for own_switch in "${MUTATION_LIST_ITEMS_OWN_SWITCHES[@]}"; do
      [[ "$given_switch" == "$own_switch" ]] && own_switches+="${own_switches:+,}$given_switch"
    done
  done
  env -u CODE_TOOLING_BREAK python3 -c "$MUTATION_LIST_ITEMS_PROGRAM" "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" "$STAGE_NAME:mutation-list-items" \
    "$STAGE_REPOSITORY" "$ROOT" "$own_switches" "$(judged_root_is_sample && echo 1)" "$1" "$MUTATION_LIST_ITEMS_JUDGE_TOKEN"
}

# mutation-list-items 格的程序（这一格文件头「按条复用」）：变异表按改的文件分组建请求、查库，有一组要现判就整张表起一趟 --list-items、照结局记那几组、判。
# mutation_list_items_judge_code 打它：键里的判法
MUTATION_LIST_ITEMS_PROGRAM="$(cat <<'PY_MUTATION_LIST_ITEMS'
import hashlib, os, re, shutil, subprocess, sys, tempfile, types
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[1], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_root, root, broken, sample, states_path, judge_token = sys.argv[2:6] + [sys.argv[6], sys.argv[7] == "1", sys.argv[8], sys.argv[9]]
TABLE = os.path.join("crates", "mutations.tsv")
SHARED_INPUTS = ("Cargo.toml", ".claude/gate.d/stage-inputs.tsv",
                 ".claude/gate.d/crash-point-replay-items.tsv", ".claude/heavy-thread-default", "scripts/crash/crash-amplification.sh")
BUILD_ENVIRONMENT_INPUTS = (".cargo", "rust-toolchain", "rust-toolchain.toml", "Cargo.lock")   # 弄坏开关 mutation-list-items-build-environment-keyed 才进键
NO_FILE_GROUP = "（第二列没写文件的行）"
HOW = "bash .claude/gate.d/code-tooling.sh --check mutation-list-items（crates-mutation-rows.py --list-items 在被判的根上整张表跑一趟）"


def digest_of(parts):
    return hashlib.sha256("\n".join(parts).encode("utf-8", "surrogateescape")).hexdigest()


# 变异表成形的行按改的文件分组（这一格文件头「按条复用」），照表里的次序
groups = {}
with open(TABLE, encoding="utf-8", errors="surrogateescape") as handle:
    for line in handle.read().splitlines():
        if line.strip() and not line.startswith("#"):
            fields = line.split("\t")
            groups.setdefault(fields[1] if len(fields) > 1 and fields[1].strip() else NO_FILE_GROUP, []).append(line)
names = list(groups)
plan_scratch = tempfile.mkdtemp(prefix="mutation-list-items-plan.")
verdicts = rounds.ItemVerdicts(admission, root, kind, plan_scratch, "code-tooling.sh 的 mutation-list-items 格", sample, HOW,
                               broken != "mutation-list-items-red-not-recorded")
if names:
    sources = digest_of(f"{name}\0{rounds.content_digest(os.path.join('crates', name))}" for name in sorted(os.listdir("crates")) if name != "mutations.tsv")
    build_environment_keyed = broken == "mutation-list-items-build-environment-keyed"   # 弄坏开关：构建环境与版本回到键里
    shared = digest_of(f"{path}\0{rounds.content_digest(path)}" for path in SHARED_INPUTS + (BUILD_ENVIRONMENT_INPUTS if build_environment_keyed else ()))
    common = [judge_token, "judge:scripts/mutation/crates-mutation-rows.py::command_list_items", f"judge-root:{judge_root}", f"line:sources={sources}", f"line:shared={shared}"]
    common += [f"line:toolchain={rounds.toolchain_line()}"] if build_environment_keyed else []
    common += [f"line:break={broken}"] if broken else []
    verdicts.plan(["\t".join([name, "-", *common, f"line:rows={digest_of(groups[name])}"]) for name in names])
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print("  ✗ mutation-list-items 这一格没判：prune-server 不可用（上面那一句），--list-items 没起，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check mutation-list-items")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：每一组都现判、一组都不记库", flush=True)
    verdicts.report_plan(names, f"（变异表 {sum(len(rows) for rows in groups.values())} 行按改的文件分组）")
judged = [name for name in names if verdicts.decision(name)[0] != rounds.REUSE]
ran_anyway = set()
if broken == "mutation-list-items-plan-ignored":   # 弄坏开关：每趟都起 --list-items
    ran_anyway, judged = set(names) - set(judged), list(names)


def finish(exit_code):
    """收尾：各条结局写给复用三问、删算键的临时目录；跑通了而记库时服务不可用退 75、按条复用失灵判红。"""
    verdicts.write_states(states_path, names, ran_anyway)
    shutil.rmtree(plan_scratch, ignore_errors=True)
    if exit_code == 0 and verdicts.unavailable:
        print("  ✗ mutation-list-items：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check mutation-list-items")
        sys.exit(75)
    if exit_code == 0 and verdicts.red:
        print("  ✗ mutation-list-items 判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）")
        print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check mutation-list-items；不能复用就判红，不退回全部现判当绿")
        sys.exit(1)
    sys.exit(exit_code)


if names and not judged:
    print(f"  ✓ crates-mutation-rows.py --list-items：变异表按改的文件分的 {len(names)} 组都有键相同、判过绿的判定，全部复用，这一格没起它")
    finish(0)
# 有一组要现判（或表里一行成形的都没有）：整张表跑一趟。脚本路径就地写成 os.path.join(<这一道所在的仓>, <字面尾巴>)：收闭包时照「<折不出的目录>/<仓里的相对路径>」
# 认到仓里那一份（写成模块级的名字再引，尾巴认不出、拒算 subprocess-path-unfolded）
done = subprocess.run(["python3", os.path.join(judge_root, "scripts", "mutation", "crates-mutation-rows.py"), "--list-items", root],
                      stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace", stdin=subprocess.DEVNULL)
output, list_exit = done.stdout, done.returncode
summary = next((line for line in output.split("\n") if re.match(r"^  · 键对不上 [0-9]+ 条", line)), "")
if broken == "list-items-exit-only":
    summary = summary or "（弄坏开关 list-items-exit-only：不看输出）"
elif list_exit == 0 and (not summary or "键对不对得上没算出来" in output or "Traceback" in output or "PicklingError" in output):
    list_exit = 1
verdicts.record([] if list_exit else judged, judged if list_exit else [], final=True)
if list_exit and judged:
    verdicts.reread_after_red()
if list_exit != 0:
    for line in output.rstrip("\n").split("\n")[-12:]:
        print(f"    {line}")  # gate-lint:detail
    print(f"  ✗ crates-mutation-rows.py --list-items 在 {root} 上没跑通（退 {list_exit}；键对不上的条数没报出来、或有「没算出来」、Traceback、PicklingError，末尾 12 行在上面）")
    print("     → 怎么办：照上面那一句的出路改（多半是变异表某一行点名的测试在 cargo 参数选的特性下不编，给那一行补特性），改完单跑")
    print("               python3 scripts/mutation/crates-mutation-rows.py --list-items . 到它报出「键对不上 N 条」；回溯是脚本自己的错，照回溯修 crates-mutation-rows.py。")
    finish(1)
listed = sum(1 for line in output.split("\n") if re.search(r"\t第 [0-9]* 行，改 ", line))
print(f"  ✓ crates-mutation-rows.py --list-items 在被判的根上跑通：列了 {listed} 行，{summary.removeprefix('  · ')}")
finish(0)
PY_MUTATION_LIST_ITEMS
)"
mutation_list_items_judge_code() { printf '%s\n' "$MUTATION_LIST_ITEMS_PROGRAM"; }

# ── 格 break-switches-sample-only：还 checks-owed.md C622（门禁的弄坏开关平常跑也认）门禁那一半。
# 判据：被判的根里 .claude/gate.d/ 顶层每道阶段（*.sh），代码里（注释之外）读了名字以 _BREAK 结尾的变量的，要经共用库那一判：
#   调了 stage_cells_run（它开头调 stage_cells_refuse_unsampled_break_switches），或自己调了 stage_cells_refuse_unsampled_break_switches（不经共用库跑格的那一道）；
#   共用库 .claude/gate.d/lib/stage-cells.sh 自己要定义这个函数、stage_cells_run 里调它。判法与「样本根」的认法只在共用库一处（它文件头「弄坏开关只在样本根上认」）。
# 射程只到门禁阶段：scripts/ 与 .claude/hooks/ 里读 *_BREAK 的研究脚本与钩子「只在自证起的子进程里认」那一半是 break-switch-outside-selftest 格（形态各家不同，没有共用的一判可认，那一格逐份起探针）。
# 一道读 *_BREAK 的阶段都没有：退 77。
# 样本：break-switches-sample-only-red（一道阶段读 SAMPLE_STAGE_BREAK、不调 stage_cells_run 也不调那一判；共用库的 stage_cells_run 没调那一判，两处点名判红）；
#   break-switches-sample-only-green（一道经 stage_cells_run、一道自己调那一判、一道不读开关，报「读开关的 2 道都经共用库那一判」）。
# 弄坏开关 CODE_TOOLING_BREAK=break-switch-guard-unchecked：不判，break-switches-sample-only-red 判绿（期望退出 1，实测 0）。
cell_break_switches_sample_only() {
  [[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
  python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
broken = sys.argv[2]
GUARD, RUNNER, LIBRARY = "stage_cells_refuse_unsampled_break_switches", "stage_cells_run", ".claude/gate.d/lib/stage-cells.sh"
SWITCH = re.compile(r"\b[A-Z][A-Z0-9_]*_BREAK\b")
problems, readers, routes = [], [], {}
for path in sorted(glob.glob(".claude/gate.d/*.sh")):
    code = code_text.code_of(open(path, encoding="utf-8", errors="replace").read())
    if not SWITCH.search(code):
        continue
    readers.append(path)
    if re.search(r"\b" + RUNNER + r"\b", code):
        routes[path] = RUNNER
    elif re.search(r"\b" + GUARD + r"\b", code):
        routes[path] = GUARD
    elif broken != "break-switch-guard-unchecked":
        problems.append(f"{path} 读了 {SWITCH.search(code).group(0)} 这类弄坏开关，不经 {RUNNER} 跑格、也没调 {GUARD}：环境里漏一个开关，平常跑也照弄坏的判法判")
if readers and broken != "break-switch-guard-unchecked":
    library = code_text.code_of(open(LIBRARY, encoding="utf-8", errors="replace").read()) if os.path.isfile(LIBRARY) else ""
    runner_body = re.search(RUNNER + r"\(\)\s*\{(.*?)\n\}", library, re.S)
    if not re.search(GUARD + r"\(\)\s*\{", library):
        problems.append(f"{LIBRARY} 里没有定义 {GUARD}（共用的那一判）")
    if not runner_body or not re.search(r"\b" + GUARD + r"\b", runner_body.group(1)):
        problems.append(f"{LIBRARY} 的 {RUNNER} 里没调 {GUARD}：经共用库跑格的阶段平常跑也认开关")
if not readers:
    print(f"  ! .claude/gate.d/ 顶层一道读 *_BREAK 的阶段都没有，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 弄坏开关只在样本根上认：{len(problems)} 处没经共用库那一判（读开关的阶段 {len(readers)} 道）")  # gate-lint:summary
    print(f"     → 怎么办：阶段 source {LIBRARY}，格经 {RUNNER} 跑；不经共用库跑格的，source 它取 {GUARD}、cd 到被判的根之后调它"
          "（写法在共用库文件头「弄坏开关只在样本根上认」，样子照 crash-point-replay.sh）")
    sys.exit(1)
by_runner = sum(1 for route in routes.values() if route == RUNNER)
print(f"  ✓ 读开关的 {len(readers)} 道都经共用库那一判（经 {RUNNER} {by_runner} 道、自己调 {GUARD} {len(readers) - by_runner} 道）")
PY
}

# ── 格 entry-scripts：还 checks-owed.md C641（两份入口脚本没按入口脚本格式核）的那一格。
# 登记表 .claude/gate.d/entry-scripts.tsv（被判的根里那一份）一行一份入口脚本（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」）：
#   <相对仓根的路径><TAB><每项一行或末行总计的前缀><TAB><那一行要带的字段，逗号连><TAB># 理由
#   前缀那一列写 pending 的是认了是入口脚本、还没改成这个格式的（字段那一列写 -）：不判，逐份报本次未判（往 $GATE_NOT_RUN_FILE 报一行），成功句里点名。
# 判据（逐份读源码，注释行与 python 的文档串不算；不起脚本）：文件在；代码里有 --list-items 与 --selftest 两个参数；代码里有那个前缀，且它那一句（前缀所在行的前 3 行到后 3 行）
#   带齐字段那一列的每个字段。项名写错退 2、列项不起重活、三条完整性闸会不会红，由各份自己的 --selftest 证（research-script-selftests 格跑），这一格只判带没带。
# 表不在、一行都没有：退 77。表里写坏的行（不是四段、理由不到 4 个字）与指到不在的文件判红（只读表，每趟照判，不入库）。
# 按条复用（共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：登记表里成形、文件在、不是 pending 的一行一条判定，名字是那一行的路径（同一路径登记两行的，后一行另接「（第 N 行）」），
#   种类 code-tooling.sh:entry-scripts（prune-server 那一份库，unit-verdicts 族 singlefs-unit-verdicts/code-tooling.sh:entry-scripts，库根不进键）。
#   判的是这一格的判法拿那份脚本的源码当数据读：请求的入口写 -（scripts/gate/admission.py 文件头「判定请求」），键由这几样组成：判法（这一格的程序
#   entry_scripts_judge_code 打的 ENTRY_SCRIPTS_PROGRAM，在这一道所在的仓里算）、line:prefix= 与 line:fields=（那一行的第二、三列；理由那一列不读，不进）、
#   file:<那份脚本>（源码原样：这一格认整行注释与文档串的写法与单元指纹去注释的写法不同，按原文进）；设着的 CODE_TOOLING_BREAK 另进一个 line:。
#   键相同、库里记着绿的不读（复用）；判完照键记：判绿记绿，判红照同一把键记红、撤掉原有的绿；每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（样本不抽）；
#   失灵照全部现判、判红，prune-server 不可用（退 75）一份都不读、这一格退 75。
#   样本：entry-scripts-reuse-three-questions（两份带齐；复用三问：change.sh 给 sample-beta.sh 加一行代码，affected 是它那一行；unrelated.sh 只改 sample-alpha.sh 那一行的理由）；
#     entry-scripts-red-over-green（一份带齐、一份每项那一句少 red=；.unit-verdict-sample/prior/ 是这一版逐字拷的一份、照判绿记两条；red-over-green 下一律现判：
#     少字段那一份判红记红、撤绿，再算一次计划：复用 1 条、现判 1 条、原因「上次判红」）。各份样本的 setup.sh 都 git init。
#   弄坏开关 CODE_TOOLING_BREAK=entry-scripts-red-not-recorded：判红不交 --red-names（只记绿），entry-scripts-red-over-green 判错；
#     =entry-scripts-plan-ignored：不按计划跳过、每份都读，entry-scripts-reuse-three-questions 判错。
# 样本：entry-scripts-red（一份没有 --list-items、一份每项那一句少 red=、一行指到不在的文件、一行写坏，逐条点名判红）；
#   entry-scripts-green（两份带齐、一份 pending，报「核了 2 份……pending 1 份本次未判」）。
# 弄坏开关 CODE_TOOLING_BREAK=entry-item-fields-unchecked：不核那一句的字段，entry-scripts-red 里「少了字段 red=」那条 want 找不到。
cell_entry_scripts() {
  local registry=".claude/gate.d/entry-scripts.tsv"
  [[ -f "$registry" ]] || { echo "  ! 没有 $registry，这一格无对象可判"; exit 77; }
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds entry-scripts entry_scripts_pass
  else
    entry_scripts_pass ""
  fi
}

# entry_scripts_pass <各条结局写到哪，空是不写>：这一格的一趟（读表、建请求、查库、读没复用的那几份、记库、判）
entry_scripts_pass() {
  python3 -c "$ENTRY_SCRIPTS_PROGRAM" ".claude/gate.d/entry-scripts.tsv" "${CODE_TOOLING_BREAK:-}" "${GATE_NOT_RUN_FILE:-}" "$UNIT_VERDICT_ROUNDS_LIBRARY" \
    "$STAGE_REPOSITORY/scripts/gate/admission.py" "$STAGE_NAME:entry-scripts" "$STAGE_REPOSITORY" "$ROOT" "$(judged_root_is_sample && echo 1)" "$1" \
    "$ENTRY_SCRIPTS_JUDGE_TOKEN"
}

# entry-scripts 格的程序（这一格文件头「按条复用」）：读表（写坏的行、不在的文件每趟照判）、建请求、查库，读没复用的那几份源码、记库、判。
# entry_scripts_judge_code 打它：键里的判法
ENTRY_SCRIPTS_PROGRAM="$(cat <<'PY_ENTRY_SCRIPTS'
import ast, os, re, shutil, sys, tempfile, types
registry, broken, not_run_file = sys.argv[1:4]
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[4], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_root, root, sample, states_path, judge_token = sys.argv[5], sys.argv[6], sys.argv[7], sys.argv[8], sys.argv[9] == "1", sys.argv[10], sys.argv[11]
HOW = "bash .claude/gate.d/code-tooling.sh --check entry-scripts（读登记的入口脚本的源码，核 --list-items、--selftest 与每项那一句的字段）"
problems, rows, pending = [], [], []
for number, line in enumerate(open(registry, encoding="utf-8"), 1):
    if not line.strip() or line.startswith("#"):
        continue
    columns = line.rstrip("\n").split("\t")
    reason = columns[3].lstrip("#").strip() if len(columns) == 4 else ""
    if len(columns) != 4 or not columns[3].startswith("#") or len(reason) < 4:
        problems.append(f"{registry}:{number} 不是「<路径><TAB><前缀><TAB><字段，逗号连><TAB># 理由>」（理由至少 4 个字）")
        continue
    path, prefix, fields = columns[0], columns[1], [field for field in columns[2].split(",") if field and field != "-"]
    if not os.path.isfile(path):
        problems.append(f"{registry}:{number} 指到的 {path} 不在")
        continue
    if prefix == "pending":
        pending.append((path, reason))
        continue
    rows.append((number, path, prefix, fields))


def script_problems(path, prefix, fields):
    """一份登记的入口脚本照这一格的判据判（这一格函数上方的注释）：交不对的那几句，判绿是空的。"""
    found = []
    lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    documentation = set()
    if path.endswith(".py"):
        # python 的文档串（模块、函数、类开头那一段字符串）照注释办：文件头的「输出」一段写着字段，代码里没打也会被认成打了
        try:
            tree = ast.parse("\n".join(lines))
        except SyntaxError as error:
            return [f"{path}：python 语法不过（{error}），判不了"]
        for node in ast.walk(tree):
            body = getattr(node, "body", None)
            if isinstance(body, list) and body and isinstance(body[0], ast.Expr) and isinstance(getattr(body[0], "value", None), ast.Constant) and isinstance(body[0].value.value, str):
                documentation.update(range(body[0].lineno - 1, body[0].end_lineno))
    code = [row for index, row in enumerate(lines) if not row.lstrip().startswith("#") and index not in documentation]
    text = "\n".join(code)
    missing = [argument for argument in ("--list-items", "--selftest") if argument not in text]
    if missing:
        found.append(f"{path}：代码里没有 {'、'.join(missing)}（入口脚本要能列项、要有自证）")
    # 打那一行的句子：带输出的词（echo、printf、print、say），不是 grep 它的那一种（自证里核输出的那几句也带前缀）
    starts = [index for index, row in enumerate(code) if prefix in row and re.search(r"\b(echo|printf|print|say)\b", row) and "grep" not in row]
    if not starts:
        found.append(f"{path}：代码里没有打「{prefix}」那一句（echo、printf、print、say）")
    elif fields and broken != "entry-item-fields-unchecked":
        best = min(([field for field in fields if field not in "\n".join(code[max(start - 3, 0):start + 4])] for start in starts), key=len)
        if best:
            found.append(f"{path}：「{prefix}」那一句少了字段 {'、'.join(best)}（判了、红了、复用了多少与秒，取不出的写取不出、不写 0）")
    return found


# 按条复用（这一格文件头「按条复用」）：一行一条判定
requests = {}
for number, path, prefix, fields in rows:
    name = path if path not in requests else f"{path}（第 {number} 行）"
    tokens = [judge_token, f"judge-root:{judge_root}", f"line:prefix={prefix}", f"line:fields={','.join(fields) or '（无）'}", f"file:{path}"]
    requests[name] = (path, prefix, fields, "\t".join([name, "-", *tokens] + ([f"line:break={broken}"] if broken else [])))
names = list(requests)
plan_scratch = tempfile.mkdtemp(prefix="entry-scripts-plan.")
verdicts = rounds.ItemVerdicts(admission, root, kind, plan_scratch, "code-tooling.sh 的 entry-scripts 格", sample, HOW, broken != "entry-scripts-red-not-recorded")
if names:
    verdicts.plan([requests[name][3] for name in names])
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print("  ✗ entry-scripts 这一格没判：prune-server 不可用（上面那一句），一份都没读，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check entry-scripts")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：每一份都现判、一份都不记库", flush=True)
    verdicts.report_plan(names, "登记的入口脚本")
judged = [name for name in names if verdicts.decision(name)[0] != rounds.REUSE]
ran_anyway = set()
if broken == "entry-scripts-plan-ignored":   # 弄坏开关：每份都读
    ran_anyway, judged = set(names) - set(judged), list(names)
if names and not judged:
    print("  · 全部复用，这一格一份都没读", flush=True)
green_names, red_names = [], []
for name in judged:
    found = script_problems(*requests[name][:3])
    problems += found
    (red_names if found else green_names).append(name)
verdicts.record(green_names, red_names, final=True)
if red_names:
    verdicts.reread_after_red()
verdicts.write_states(states_path, names, ran_anyway)
shutil.rmtree(plan_scratch, ignore_errors=True)
for path, reason in pending:
    print(f"  ! {path} 登记成入口脚本、还没改成入口脚本格式，本次未判：{reason}")
    if not_run_file and os.path.exists(not_run_file):
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"code-tooling.sh entry-scripts：{path} 还没改成入口脚本格式，本次未判\n")
if not rows and not pending and not problems:
    print(f"  ! {registry} 一行都没有，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 入口脚本格式：{len(problems)} 处不对（登记 {len(rows) + len(pending)} 份）")  # gate-lint:summary
    print("     → 怎么办：照上面逐条补：--list-items 逐行列可点名的项、不编不跑；项名写错退 2；--selftest 证三条完整性闸各会红；"
          "每项一行带项名、退出码、判了、红了、复用了多少与秒，末行总计（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」）；"
          f"还没改成的登记成 pending、写明还缺什么；表 {registry} 的写法在这一格函数上方的注释")
    sys.exit(1)
if verdicts.unavailable:
    print("  ✗ entry-scripts：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check entry-scripts")
    sys.exit(75)
if verdicts.red:
    print("  ✗ entry-scripts 判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）")
    print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check entry-scripts；不能复用就判红，不退回全部现判当绿")
    sys.exit(1)
print(f"  ✓ 入口脚本格式：核了 {len(rows)} 份（{'、'.join(path for _number, path, _prefix, _fields in rows)}），都有 --list-items、--selftest 与带齐字段的那一句；pending {len(pending)} 份本次未判"
      + (f"（{'、'.join(path for path, _ in pending)}）" if pending else ""))
PY_ENTRY_SCRIPTS
)"
entry_scripts_judge_code() { printf '%s\n' "$ENTRY_SCRIPTS_PROGRAM"; }

# ── 格 break-switch-headers：还 checks-owed.md C625（弄坏开关文件头三样没有门禁）的格式那一半。
# 判据：被判的根里 scripts/ 与 .claude/hooks/ 下（fixtures 目录除外）的 .sh、.py，代码里读了名字以 _BREAK 结尾的变量的
#   （shell 的 $X_BREAK / ${X_BREAK…}，python 的 environ.get("X_BREAK") / environ["X_BREAK"] / getenv("X_BREAK") 与 `= "X_BREAK"` 的常量，注释行不算），
#   文件头里要有至少一条 mutate.sh 写法的开关（读法在 BREAK_SWITCH_TABLE_LIBRARY），每一条三样写全：名字与值、弄坏哪一处、下一行起至少一行点名的项；
#   点名的项里「」要成对：复跑格按「」读点名的项，少一个收尾的括号就读不到它点名的是哪一格，只在复跑时才暴露。配不配平以复跑格读点名的项用的 quoted_items 读出的各项为准
#   （BREAK_SWITCH_HEADER_LIBRARY 的 quote_gap，与复跑格开跑前的预检同一份），不配平判红：点名这一条开关、第几行、怎么改（补上收尾的括号，或删掉多出来的那一个）。点名的项里没有「」的（门禁阶段写样本目录名）不判。
#   复跑登记表 .claude/gate.d/break-switch-replay-files.tsv 第一列点名的文件（放在 task-server/ 这类别处的同样）一起判：break-switch-replay 格判「文件头开关表没写全」的，这一格点名同一条。
#   立账时还没改成这种写法的存量逐份登记在 .claude/gate.d/break-switch-headers-exempt.tsv（「<相对仓根的路径><TAB># 理由>」），登记了的不判；
#   一行指到的文件不在、或那份已经写全了（或不再读开关）、或那份在复跑登记表里（复跑那一格不认豁免），判红：那一行删掉。表只缩不涨（不判涨，靠 review）。
# 管不到的：逐个打开开关、看自证红在点名的那一项（复跑方）——研究脚本与钩子那一半是 break-switch-replay 格（只复跑登记表里的），门禁阶段那一半是 stage-break-switch-replay 格。
# 一份读开关的都没有：退 77。
# 拆分族（写法与认法在 BREAK_SWITCH_HEADER_LIBRARY 的 SplitFamilies）：入口拆出的部分文件（同目录的 <名>_<部分>.py、<名>-<部分>.sh、<名>-<部分>.py，门禁阶段的 lib/ 下 <阶段名>-<部分>.sh，
#   且入口的代码里提到它）不单算一份：读开关、开关表写没写全按入口加部分文件整族算（开关表可以写在入口或部分文件的文件头，有表的每一份各按三样判，红句的文件:行是那一份的），
#   入口自己不读、部分文件读，入口那一份照判豁免表；部分文件读的每个开关变量要在整族的开关表里有，不在的点名「读了 X，所属入口 … 整族的文件头开关表里没有」；
#   豁免表与复跑登记表只登记入口，复跑登记表点名的文件各自是入口。成功句末尾报部分文件读开关的份数，不进「读开关的 N 份」的份数。
#   入口的代码里没提到的同族命名文件（gate.sh 与 gate-staged.sh 这种）是独立的脚本，照单份判。
#   样本 break-switch-headers-split-family-green（入口 scripts/gate/divided.py 不读开关、文件头写全 DIVIDED_BREAK，import 的 divided_core.py 读它；同目录 divided_other.py 入口没提到、自己写全了 OTHER_BREAK：
#   读开关的 2 份，另有 1 份部分文件）、break-switch-headers-split-family-red（部分文件读的 ORPHAN_BREAK 不在整族的开关表里：点名这份部分文件与入口）、
#   break-switch-headers-split-family-table-in-part-green（入口 scripts/gate/shared.py 的文件头没有开关表，开关表写全在 import 的 shared_core.py 的文件头：读开关的 1 份，另有 1 份部分文件）。
#   弄坏开关 CODE_TOOLING_BREAK=headers-split-family-unread：每份各判各的、不认部分文件，-split-family-green 判红（期望退出 0，实测 1：divided_core.py 读开关而文件头没有表）、
#   -split-family-red 里「所属入口」那条 want 找不到、-split-family-table-in-part-green 里「另有 1 份部分文件」那条 want 找不到；
#   =headers-split-family-table-entry-only：开关表只读入口那一份的文件头，-split-family-table-in-part-green 判红（期望退出 0，实测 1：shared.py 文件头没有表）；
#   =headers-split-family-by-name：部分文件只按命名认、不核入口的代码提没提到，-split-family-green 判红（「读开关的 2 份」那条 want 找不到：
#   divided_other.py 被当成部分文件、读开关的只剩 1 份、另有 2 份部分文件）。
# 样本：break-switch-headers-red（一份读开关却没有表、一份一条没写弄坏哪一处、一份一条没点名的项、豁免表一行指到写全了的、一行指到不在的；
#   复跑登记表点名 task-server/ 下两份，点名行都与开关行一样深，其中一份也登在豁免表里，逐条点名判红）；
#   break-switch-headers-green（一份写全、一份登记在豁免表、一份只在注释里提开关不算读、一份在复跑登记表里写全，报「读开关的 3 份：写全 2 份、豁免 1 份」）；
#   break-switch-headers-quote-unbalanced-red（一份点名的项里起头的括号没有收尾、一份多一个收尾的括号、一份「」成对且里面套着一层成对的；点名第 4 行与怎么改，报「2 处不对（读开关的 3 份：写全 1 份、豁免 0 份）」）。
# 弄坏开关 CODE_TOOLING_BREAK=switch-items-unchecked：不判点名的项，break-switch-headers-red 里「点名的项」那条 want 找不到；
#   headers-quotes-unpaired-unchecked：不判点名的项里「」配不配平，break-switch-headers-quote-unbalanced-red 判绿（期望退出 1，实测 0）；
#   开关 headers-replay-files-unread（文件头那一条）打开时不读复跑登记表，break-switch-headers-red 里 task-server/ 那几条 want 找不到，break-switch-headers-green 的份数对不上。
# 同一份文件头两格结论相同：读没读开关（SplitFamilies.switch_names）、写全没写全（SplitFamilies.family_gaps）、红句（gap_sentence）、复跑登记表点名的文件（replay_registry_paths）
#   都只在 BREAK_SWITCH_HEADER_LIBRARY 里写一份，这一格与 break-switch-replay 格开跑前的预检都调它；复跑格的红句是同一句前面加「复跑登记表:行号」。
#   样本 break-switch-header-gap-red 两格一起跑：登记表点名的 task-server/shallow-items.py 开关那一条的点名行与开关行一样深（缩进不够），两格都红、红句点名 SHALLOW_BREAK=skip；
#   弄坏开关 switch-items-unchecked、headers-replay-files-unread 打开时这一格的那条 want 找不到（复跑格另有 replay-header-gaps-unjudged）。
cell_break_switch_headers() {
  local exempt=".claude/gate.d/break-switch-headers-exempt.tsv"
  python3 - "$BREAK_SWITCH_TABLE_LIBRARY" "$exempt" "${CODE_TOOLING_BREAK:-}" "$BREAK_SWITCH_HEADER_LIBRARY" "$BREAK_SWITCH_REPLAY_JUDGE_LIBRARY" "$TREE_LISTING_LIBRARY" \
    "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import glob, os, sys, types
tree = types.ModuleType("lib_tree_listing")
exec(compile(sys.argv[6], "tree-listing-library", "exec"), tree.__dict__)
table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[1], "break-switch-table-library", "exec"), table.__dict__)
exec(compile(sys.argv[5], "break-switch-replay-judge-library", "exec"), table.__dict__)   # 复跑格读点名的项用的 quoted_items（点名的项里「」配不配平以它读出的为准）
exec(compile(sys.argv[4], "break-switch-header-library", "exec"), table.__dict__)   # 读没读开关、写全没写全、「」配没配平、复跑登记表的取法：与 break-switch-replay 格同一份
exempt_path, broken = sys.argv[2:4]
REPLAY_REGISTRY = table.REPLAY_REGISTRY_PATH   # break-switch-replay 格复跑的文件：它判没写全的，这一格同样判
registered = set() if broken == "headers-replay-files-unread" else set(table.replay_registry_paths(REPLAY_REGISTRY))
paths = sorted({path for directory in ("scripts", ".claude/hooks") for path in tree.files_under(directory, (".sh", ".py"), skipped=("target", "fixtures"))}
               | {path for path in registered if os.path.isfile(path)})   # 指到不在的那一行归 break-switch-replay 格点名
# 拆分族（这一格文件头「拆分族」）：登记表点名的文件各自是入口；弄坏开关 headers-split-family-unread 下每份各判各的，headers-split-family-by-name 下部分文件只按命名认，
# headers-split-family-table-entry-only 下开关表只读入口那一份的文件头
families = table.SplitFamilies(table.load_split_family(sys.argv[7]), independent=registered, alone=broken == "headers-split-family-unread",
                               by_name=broken == "headers-split-family-by-name", entry_table_only=broken == "headers-split-family-table-entry-only")
ignored_gaps = {"switch-items-unchecked": ("点名的项",), "headers-quotes-unpaired-unchecked": (table.QUOTES_PAIRED,)}.get(broken, ())
problems, exempt = [], {}
if os.path.isfile(exempt_path):
    for number, line in enumerate(open(exempt_path, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        path, _, reason = line.rstrip("\n").partition("\t")
        if len(reason.lstrip("#").strip()) < 4:
            problems.append(f"{exempt_path}:{number} 不是「<路径><TAB># 理由>」（理由至少 4 个字）")
            continue
        exempt[path] = number
readers, complete, exempted, part_readers = [], [], [], []
for path in paths:
    if families.entry_of(path) != path:
        # 部分文件：读没读开关、开关表写没写全归所属入口那一行（整族一起判），这里只数读开关的部分文件
        if table.read_switch_names(families.text_of(path)):
            part_readers.append(path)
        continue
    if not families.switch_names(path):
        continue
    readers.append(path)
    gaps = families.family_gaps(path, ignored=ignored_gaps)
    written = not gaps
    if path in exempt and path not in registered:
        exempted.append(path)
        if written:
            problems.append(f"{exempt_path}:{exempt[path]} {path} 的开关表已经三样写全了，豁免那一行删掉")
        continue
    problems.extend(table.gap_sentence(where, gap) for where, gap in gaps)
    problems.extend(f"{part}：读了 {'、'.join(missing)}，所属入口 {path} 整族的文件头开关表里没有这几条（部分文件读的开关写在入口或部分文件的文件头，每条三样写全）"
                    for part, missing in families.uncovered_reads(path))
    if written and not families.uncovered_reads(path):
        complete.append(path)
for path, number in sorted(exempt.items(), key=lambda item: item[1]):
    if not os.path.isfile(path):
        problems.append(f"{exempt_path}:{number} 指到的 {path} 不在，那一行删掉")
    elif families.entry_of(path) != path:
        problems.append(f"{exempt_path}:{number} {path} 是 {families.entry_of(path)} 的部分文件，开关写在入口的文件头开关表里，豁免只登记入口，这一行删掉")
    elif path not in readers:
        problems.append(f"{exempt_path}:{number} {path} 代码里不读弄坏开关了，那一行删掉")
    elif path in registered:
        problems.append(f"{exempt_path}:{number} {path} 在复跑登记表 {REPLAY_REGISTRY} 里，复跑那一格要判它的开关表，豁免不算数，那一行删掉、开关表写全")
if not readers and not problems:
    print("  ! scripts/ 与 .claude/hooks/ 里没有读弄坏开关的脚本，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 弄坏开关文件头：{len(problems)} 处不对（读开关的 {len(readers)} 份：写全 {len(complete)} 份、豁免 {len(exempted)} 份）")  # gate-lint:summary
    print("     → 怎么办：照 scripts/mutation/mutate.sh 文件头那张表写：一行「X_BREAK=<值>  <弄坏哪一处>」，下一行起缩进更深写打开后哪几项该红（自证的项名，括号里写失败行句首）；"
          f"一时改不完的存量登记进 {exempt_path}（只缩不涨），改完就删那一行；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 弄坏开关文件头：读开关的 {len(readers)} 份：写全 {len(complete)} 份、豁免 {len(exempted)} 份（{exempt_path}，只缩不涨）；"
      f"另有 {len(part_readers)} 份部分文件读开关，读的开关整族的文件头开关表里都有")
PY
}

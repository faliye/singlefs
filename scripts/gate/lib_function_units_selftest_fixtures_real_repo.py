# 算键库 scripts/gate/lib_function_units.py 的自证夹具的一份（总说明在 lib_function_units_selftest_fixtures.py 文件头）：
# 只由算键库的 load_selftest_fixtures 在自证开跑前按路径载入，读已载入的几份与自证各模块此刻的名字、不被 import。

NESTED_PROGRAM_WORDS_OUTER = "tool/nested_seam.sh::run_cells"
NESTED_PROGRAM_WORDS_INNER = "tool/nested_seam.sh::run_cells::run_inner_seam"

# TAIL_READ_SCOPES 的函数一格还认的模块级语句单元（内联 python 正文里的读多半写在模块级）：赋值的名字、<module-body>、<main-guard>
TAIL_READ_SCOPE_STATEMENT_KINDS = ("assign", "module-body", "main-guard")

TAIL_READS_RECORDS_REAL_CELL = "真仓：按尾巴整组收、没登记 TAIL_READ_SCOPES 的读法，收进来的文件里没有落在 records/ 或 research/ 下的（夹具里没登记的一处判得出来、登记之后不判）"
# 夹具同 TAIL_READ_SCOPES_FIXTURE，另在 records/、research/ 下各放一份同名的清单：scan 照尾巴收全仓的 Manifest.toml，没登记时这两份都收进来
TAIL_READS_RECORDS_FIXTURE = dict(TAIL_READ_SCOPES_FIXTURE, **{"records/round/Manifest.toml": "name = record\n", "research/round/Manifest.toml": "name = research\n"})
TAIL_READS_RECORDS_SCRIPT_SUFFIXES = (".py", ".sh", ".bash")

BUILD_OUTPUT_CELL = "python：拷 <临时目录>/target/release/<程序>，仓里 target 下有一份编出来的二进制：不收进键，改它键不变"
TOOL_SOURCE_CELL = "python：编工具程序的函数（BUILT_TOOL_SOURCES）进了闭包：它编的程序的源码目录收成数据目录，改里面一份键变、改编出来的二进制键不变"
BUILD_OUTPUT_FIXTURE = {
    "pkg/build.py": ("import os\nimport shutil\nimport tempfile\n\n\ndef build_program(destination):\n    build = tempfile.mkdtemp()\n"
                     "    shutil.copy2(os.path.join(build, \"target\", \"release\", \"tool\"), os.path.join(destination, \"tool-copy\"))\n"
                     "    return os.path.join(destination, \"tool-copy\")\n"),
    "target/release/tool": "binary one\n",
    "crates/sample-tool/Cargo.toml": "[package]\nname = \"sample-tool\"\n",
    "crates/sample-tool/src/main.rs": "fn main() {}\n",
}
BUILD_OUTPUT_ENTRY = "pkg/build.py::build_program"

BUILT_TOOL_LAUNCH_CELL = "shell 函数编工具、内联 python 起编出来的那份（BUILT_TOOL_SOURCES 与 BUILT_TOOL_LAUNCHES）：不登记时拒算，登记了不拒算、改工具源码里一个函数键变"
# 夹具：同 code-tooling.sh 的 function-units-two-implementations-agree：shell 函数编工具、export -f 交给内联 python，python 经 bash -c 起它、取它打的末行当程序路径再起
BUILT_TOOL_LAUNCH_FIXTURE = {
    "crates/sample-tool/src/main.rs": "fn main() {\n    println!(\"{}\", helper());\n}\n\nfn helper() -> u32 {\n    1\n}\n",
    "s/agree.sh": '''#!/usr/bin/env bash
build_sample_tool() {
  rustc -O -o "$1/sample-tool" crates/sample-tool/src/main.rs > /dev/null 2>&1 || return 1
  printf '%s\\n' "$1/sample-tool"
}
export -f build_sample_tool
python3 - "$1" <<'PY'
import subprocess
import sys

build_home = sys.argv[1]
binary = None


def rust(request):
    completed = subprocess.run([binary, request[0], *request[1:]], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return completed.returncode


if build_home:
    done = subprocess.run(["bash", "-c", 'build_sample_tool "$1"', "sample-build", build_home], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    built = done.stdout.strip().split("\\n")[-1] if done.stdout.strip() else ""
    binary = built
sys.exit(rust(["dump"]))
PY
''',
}
BUILT_TOOL_LAUNCH_ENTRY = "s/agree.sh"
BUILT_TOOL_LAUNCH_FIXTURE_SOURCES = {("s/agree.sh", "build_sample_tool"): "crates/sample-tool/src"}
BUILT_TOOL_LAUNCH_FIXTURE_LAUNCHES = {("s/agree.sh", "rust", "binary"): ("s/agree.sh", "build_sample_tool")}

CALLER_SUPPLIED_REGISTERED_CELL = "调用方登记：共用起法拿参数起子进程，登记了的调用处交的命令不拒算"
CALLER_SUPPLIED_UNREGISTERED_CELL = "调用方登记：同一个共用起法，没登记的调用处交的命令照拒算"

CALL_SITE_SHADOWED_CELL = ("python：调用方交进来的命令按调用处认，套在别的函数里的同名函数的调用 run(1, 5) 不算模块级 run 的调用处：不拒算，"
                           "模块级 run 真正的调用处给的 tool/child.sh 连上（改它键变）")
CALL_SITE_ATTRIBUTE_CELL = ("python：调用方交进来的命令按调用处认，import 进来的别的模块上的 other_tools.run(7) 不算 runner_lib.run 的调用处：不拒算，"
                            "runner_lib.run 真正的调用处给的 tool/child.sh 连上（改它键变）")
UNIT_GRAPH_TEXT_BUSY_CELL = "产品行为视图：起 unit-graph 的那一刻程序文件还有人开着写（ETXTBSY），写的那一方关掉之后再起得成，不记成起不了"
UNIT_GRAPH_FAILED_STEP_CELL = "产品行为视图：编 unit-graph 的一步抛 OSError（缓存目录的位置上是一份普通文件）：报错带上是哪一步（os.makedirs）与那个路径"
UNIT_GRAPH_PRESENT_PROGRAM_CELL = "产品行为视图：缓存目录里的 unit-graph 文件在、本机看不到执行位（共享盘上别的机器刚换上）：照用，不重编"
UNIT_GRAPH_SHARED_DIRECTORIES_CELL = "产品行为视图：缓存目录与这一份程序的目录已由别的进程建好：照用、换上编出来的程序，不报 EEXIST（编译换成假的）"
# 编 unit-graph 那几格的小仓：built_unit_graph 只看 crates/singlefs-unit-graph/Cargo.toml 在不在、照三样算缓存目录名，编译那一步换成假的（不起 cargo）
UNIT_GRAPH_STEP_FIXTURE = {
    "crates/singlefs-unit-graph/Cargo.toml": '[package]\nname = "singlefs-unit-graph"\nversion = "0.0.0"\nedition = "2021"\n',
    "crates/singlefs-unit-graph/src/main.rs": "fn main() {}\n",
    ".claude/heavy-thread-default": "4\n",
    "Cargo.lock": "# 自证的小仓\n",
}

CALL_SITE_PARTIAL_CELL = ("python：调用方交进来的命令按调用处认，functools.partial(runner_lib.run, \"tool/child.sh\") 算 runner_lib.run 的一处调用：不拒算，"
                          "它交的 tool/child.sh 连上（改它键变）")
CALL_SITE_UNDECIDED_CELL = "python：调用方交进来的命令按调用处认，判不了的 holder.run(7)（holder 是参数）照旧算 runner_lib.run 的调用处，照拒算"
# 夹具：run / runner_lib.run 拿参数起子进程（subprocess.call，名字不叫 run）；三份调用方各带一处同名的调用：套着的 run、别的模块上的 run、参数上的 run
CALL_SITE_FIXTURE = {
    "tool/shadowed_run.py": ('import subprocess\n\n\ndef run(program):\n    return subprocess.call([program, "--flag"])\n\n\n'
                             'def cells():\n    def run(count, seed):\n        return count + seed\n    return run(1, 5)\n\n\n'
                             'def main():\n    cells()\n    return run("tool/child.sh")\n'),
    "tool/runner_lib.py": 'import subprocess\n\n\ndef run(program):\n    return subprocess.call([program, "--flag"])\n',
    "tool/other_tools.py": "def run(count):\n    return count\n",
    "tool/attribute_caller.py": 'import other_tools\nimport runner_lib\n\n\ndef main():\n    other_tools.run(7)\n    return runner_lib.run("tool/child.sh")\n',
    "tool/holder_caller.py": 'import runner_lib\n\n\ndef main(holder):\n    holder.run(7)\n    return runner_lib.run("tool/child.sh")\n',
    "tool/partial_caller.py": ('import functools\n\nimport runner_lib\n\n\ndef main():\n'
                               '    group = functools.partial(runner_lib.run, "tool/child.sh")\n    return group()\n'),
    "tool/child.sh": "#!/usr/bin/env bash\necho child\n",
}

INLINE_REFUSAL_LINE_CELL = "内联 python 的正文出自 here-doc（直接喂的、装进变量再交 -c 的）：拒算落在原文那一行（正文头一行加偏移），不落在起它的那一行或头一处像起 python 的行"
# 夹具：起 python 的那一行之前另有一行提到 python3（word_line 退回去找的头一处）；一份 here-doc 直接喂给 python3 -，一份先经 $(cat <<词 …) 装进变量再交 -c，
# 两份正文里各有一处必拒算的 globals()
INLINE_REFUSAL_LINE_FIXTURE = {
    "s/inline_refusal_line.sh": '''#!/usr/bin/env bash
echo "python3 - checks the rows"
python3 - "$1" <<'PY'
import sys


def pick(name):
    return globals().get(name)


pick(sys.argv[1])
PY
LOOKUP_PROGRAM="$(cat <<'PY_LOOKUP'
import sys


def lookup(name):
    return globals().get(name + "_handler")


lookup(sys.argv[1])
PY_LOOKUP
)"
python3 -c "$LOOKUP_PROGRAM" "$1"
''',
}

STAGE_CELL_FUNCTIONS_REAL_CELL = "真仓：各道门禁 stage_cell 那一行第二列都是字面函数名（stage-cells.sh 起的 \"$function_name\" 靠写着名字的那一行进键）"
MODULE_BODY_COMMAND_CELL = "python：模块体里循环中赋的列表与 for 取的字面列表拼成 [*a, *b]：照函数体同一条折，连到那份脚本（改它键变）"
EXPORTED_VARIABLE_CELL = "shell：同一个单元里 export 过的变量到内联 python 的环境里：os.environ[…] 折得出，sys.path 与它起的脚本都认（改它键变）"
PYTHON_SPLIT_PART_ENVIRONMENT_CELL = ("python：拆成 shell 入口加同目录模块，拆出的模块读入口给内联 python export 的变量：os.environ[…] 照入口折，"
                                      "它起的脚本连上（改它键变）")
# 拆成入口加被 source 的库：库里另一个函数在命令前写 NAME="$NAME"（把入口赋的值原样交给那条命令），读 NAME 的函数照旧到入口那一处折
SHELL_SELF_ASSIGNMENT_FIXTURE = {
    "s/self_assigned.sh": ('#!/usr/bin/env bash\nSELF_ASSIGNED_ROOT="$(cd "$(dirname "$0")" && pwd)"\n'
                           'source "$SELF_ASSIGNED_ROOT/lib/self_assigned_cells.sh"\nrun_child\n'),
    "s/lib/self_assigned_cells.sh": ('show_root() {\n  SELF_ASSIGNED_ROOT="$SELF_ASSIGNED_ROOT" echo shown\n}\n\n'
                                     'run_child() {\n  bash "$SELF_ASSIGNED_ROOT/child.sh"\n}\n'),
    "s/child.sh": "#!/usr/bin/env bash\necho child\n",
}
SHELL_SELF_ASSIGNMENT_CELL = ("shell：被 source 的库里另一个函数在命令前写 NAME=\"$NAME\"（原样交给那条命令），库里的函数读 NAME 照旧到入口里赋它的那一处折，"
                              "起的脚本连上（改它键变）")
GUARD_NAME_PATH_CELL = "python：main-guard 里赋的名字给 sys.path.insert：折得出，from 那份模块连上（改它键变）"

NARROWED_VIEW_SHARED_CELL = "shell：几种字面参数剪出同一份正文的共用一个视图，剪出别的正文的是另一个"
OWN_PROGRAM_KEYED_CELL = "Rust：bin 的 main 起自己（路径原样交给 Command::new）：不拒算、不连边，改子进程那个角色跑的函数键变；库函数从 bin 的 main 连到的同样不拒算"
OWN_PROGRAM_ENTRY_CELL = "Rust：路径原样交给 Command::new 而入口不是 bin 的 main（库单测、bin 里的单测、单测调 main、按测试编的 main、bin 的辅助函数、模块里的 main）：拒算 current-exe"
OWN_PROGRAM_PATH_CELL = "Rust：入口是 bin 的 main 而路径改过或流向别处（同目录的程序、父目录、失败时交回别的路径、当参数交给别的程序、交给辅助函数、名字绑成别的路径、--exact 点名不存在的）：拒算 current-exe"
EXTERN_LISTED_CELL = "Rust：extern \"C\" 块只声明名单里的符号（#[link_name] 或函数名）：不拒算，改调用方键变"
EXTERN_REFUSED_CELL = "Rust：extern 块有名单外的符号、#[link]、别的 ABI、static、别的属性：拒算 extern-block"
OWN_PROGRAM_MANIFEST_PATH = "crates/frestart/Cargo.toml"
OWN_PROGRAM_MANIFEST = '[package]\nname = "frestart"\n'
OWN_PROGRAM_MAIN_FILE = "crates/frestart/src/main.rs"
OWN_PROGRAM_LIBRARY_FILE = "crates/frestart/src/lib.rs"
OWN_PROGRAM_MAIN_ENTRY = OWN_PROGRAM_MAIN_FILE + "::main"
OWN_PROGRAM_LIBRARY_RESTART = 'pub fn restart_from_library() {\n    let _ = std::process::Command::new(std::env::current_exe().expect("路径")).status();\n}\n'
# 夹具与 crates/singlefs-unit-graph 的 tests/unit_graph_own_program_restart_is_keyed_only_from_the_main_of_a_bin.rs 同形
OWN_PROGRAM_RESTARTS_KEYED = {
    "command-new": '''fn restart() {
    let mut command = std::process::Command::new(std::env::current_exe().expect("路径"));
    command.arg("role");
    command.status().expect("起得来");
}''',
    "bound-diverging-fallback-used-at-the-level-of-the-let": '''fn restart() {
    let this_program = std::env::current_exe().unwrap_or_else(|error| {
        eprintln!("取不到路径：{error}");
        std::process::exit(78)
    });
    eprintln!("起 {}", this_program.display());
    let status = std::process::Command::new(&this_program).arg("role").status();
    drop(status);
}''',
    "bound-by-match-with-a-diverging-block": '''fn restart() {
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => {
            eprintln!("取不到路径：{error}");
            std::process::exit(78)
        }
    };
    std::process::Command::new(&executable).arg("role").status().expect("起得来");
}''',
    "bound-by-match": '''fn restart() -> Result<(), String> {
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => return Err(error.to_string()),
    };
    let child = std::process::Command::new(executable).arg("role").spawn();
    child.map(drop).map_err(|error| error.to_string())
}''',
    "question-mark": '''fn restart() -> std::io::Result<()> {
    let program = std::env::current_exe()?;
    std::process::Command::new(program).arg("role").status().map(drop)
}''',
}
OWN_PROGRAM_RESTARTS_REFUSED = {
    "sibling-inside-command-new": '''fn restart() {
    let _ = std::process::Command::new(std::env::current_exe().expect("路径").with_file_name("other")).status();
}''',
    "sibling-through-binding": '''fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new(program.with_file_name("other")).status();
}''',
    "binding-of-parent": '''fn restart() {
    let program = std::env::current_exe().expect("路径").parent().expect("有上级").join("other");
    let _ = std::process::Command::new(program).status();
}''',
    "fallback-another-path": '''fn restart() {
    let program = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("/bin/other"));
    let _ = std::process::Command::new(program).status();
}''',
    "match-error-another-path": '''fn restart() {
    let program = match std::env::current_exe() {
        Ok(program) => program,
        Err(_) => std::path::PathBuf::from("/bin/other"),
    };
    let _ = std::process::Command::new(program).status();
}''',
    "match-ok-another-path": '''fn restart() -> Option<()> {
    let other = std::path::PathBuf::from("/bin/other");
    let program = match std::env::current_exe() {
        Ok(_program) => other,
        Err(_) => return None,
    };
    std::process::Command::new(program).status().ok().map(drop)
}''',
    "path-as-argument": '''fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new("sh").arg(&program).status();
}''',
    "path-to-helper": '''fn helper(_path: &std::path::Path) {}
fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new(&program).status();
    helper(&program);
}''',
    "rebound-name": '''fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new(&program).status();
    let program = program.with_file_name("other");
    let _ = std::process::Command::new(program).status();
}''',
    "exact-names-nobody": '''fn restart() {
    let _ = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::missing"]).status();
}''',
}
# 用法与 let 不在同一层花括号里的几种（OWN_PROGRAM_BLOCK_CELL）：用法进了更深的块（起子进程失败的闭包块里显示、if 块里起）、用法出了 let 所在的块
OWN_PROGRAM_RESTARTS_REFUSED_BY_BLOCK = {
    "displayed-inside-a-deeper-block": '''fn restart() {
    let this_program = std::env::current_exe().unwrap_or_else(|error| {
        eprintln!("取不到路径：{error}");
        std::process::exit(78)
    });
    let status = std::process::Command::new(&this_program).arg("role").status().unwrap_or_else(|error| {
        eprintln!("起不了 {}：{error}", this_program.display());
        std::process::exit(78)
    });
    drop(status);
}''',
    "use-in-an-if-block": '''fn restart() {
    let exe = std::env::current_exe().expect("路径");
    if std::env::args().count() == 1 {
        std::process::Command::new(&exe).arg("role").status().expect("起得来");
    }
}''',
    "use-after-the-block-of-the-let": '''fn restart() {
    let exe = std::path::PathBuf::from("/bin/worker");
    {
        let exe = std::env::current_exe().unwrap();
        eprintln!("launcher at {}", exe.display());
    }
    let _ = std::process::Command::new(&exe).status();
}''',
}
# 失败时的闭包以 if … else { panic!() } 收尾（OWN_PROGRAM_DIVERGENCE_CELL）：块前面是 else，不是闭包参数也不是 =>，别的分支可能交值
OWN_PROGRAM_RESTARTS_REFUSED_BY_DIVERGENCE = {
    "fallback-closure-ending-in-an-if-else": '''fn restart() {
    let this_program = std::env::current_exe().unwrap_or_else(|_| {
        if let Some(first_argument) = std::env::args_os().next() {
            std::path::PathBuf::from(first_argument)
        } else {
            panic!("拿不到程序路径")
        }
    });
    let _ = std::process::Command::new(&this_program).arg("role").status();
}''',
}
OWN_PROGRAM_BLOCK_CELL = "Rust：路径绑给 let 名字，用法进了更深的块或出了 let 所在的块（起子进程失败的闭包块里显示、if 块里起、内层块的 let 在块外用）：拒算 current-exe"
OWN_PROGRAM_DIVERGENCE_CELL = "Rust：取路径失败时的闭包以 if … else { panic!() } 收尾（块前面是 else）：拒算 current-exe"
# 夹具与 tests/unit_graph_extern_blocks_declaring_only_listed_system_library_calls_are_keyed.rs 同形
EXTERN_BLOCK_FILE = "crates/fextern/src/lib.rs"
EXTERN_BLOCK_ENTRY = EXTERN_BLOCK_FILE + "::entry_point"
EXTERN_LISTED_LINK_NAMES = '''extern "C" {
    #[link_name = "pwrite"]
    fn pwrite_raw(
        fd: i32,
        buf: *const core::ffi::c_void,
        count: usize,
        offset: i64,
    ) -> isize;
    #[link_name = "fdatasync"]
    fn fdatasync_raw(fd: i32) -> i32;
}
pub fn entry_point() -> i32 {
    unsafe { fdatasync_raw(3) }
}
'''
EXTERN_LISTED_OWN_NAME = 'extern "C" {\n    pub fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n'
EXTERN_BLOCKS_REFUSED = {
    "unlisted": 'extern "C" {\n    fn external_symbol() -> u32;\n}\npub fn entry_point() -> u32 {\n    unsafe { external_symbol() }\n}\n',
    "link-name-outside": 'extern "C" {\n    #[link_name = "my_native_write"]\n    fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n',
    "one-outside": 'extern "C" {\n    #[link_name = "fdatasync"]\n    fn sync_raw(fd: i32) -> i32;\n    fn external_symbol() -> u32;\n}\npub fn entry_point() -> i32 {\n    unsafe { sync_raw(3) }\n}\n',
    "link-attribute": '#[link(name = "mine")]\nextern "C" {\n    fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n',
    "another-abi": 'extern "system" {\n    fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n',
    "static": 'extern "C" {\n    fn fdatasync(fd: i32) -> i32;\n    static environ: *const u8;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n',
    "another-attribute": 'extern "C" {\n    #[cold]\n    fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n',
}
EXTERN_LINK_ATTRIBUTE_CELL = "Rust：extern 块外属性里任何位置有 link（#[cfg_attr(条件, link(…))]）：拒算 extern-block"
EXTERN_OTHER_BLOCK_LINK_CELL = "Rust：同一个目标里另一个 extern 块带 #[link]（那个块不在闭包里）：名单里的块也拒算 extern-block"
EXTERN_BUILD_SCRIPT_LINK_CELL = "Rust：包的构建脚本写了 rustc-link-lib：名单里的块也拒算 extern-block；构建脚本没写的不拒算"
WORKSPACE_DOT_MEMBER_CELL = "工作区成员：[workspace] members = [\".\"]：没有 [package] 的根收成叫 root 的 crate 且有库目标，有 [package] 的根只收一个 crate"
WORKSPACE_MULTILINE_CELL = "工作区成员：members 里写成多行基础串（三个双引号）的：没有反斜杠的去掉开头的换行照收，含反斜杠的报读不出（FunctionUnitsError）"
WORKSPACE_BRACKET_CELL = "工作区成员：仓根路径带 [ ] 时成员照样展开"
WORKSPACE_SAME_NAME_CELL = "工作区成员：两个成员目录同名（crates/common、tools/common）各有 tests/smoke.rs，两个目标都算得出键且键不同"
EXTERN_OTHER_TARGET_LINK_CELL = "Rust：同一个包或依赖的测试目标里有 #[link] 的 extern 块（那个目标不是入口的目标）：名单里的块也拒算 extern-block"
EXTERN_DEPENDENCY_LINK_CELL = "Rust：顺依赖（含开发依赖、构建依赖）够到的包带 #[link]、构建脚本打 rustc-link-lib、构建依赖的库代码打 cargo:rustc-link-lib 指令：名单里的块也拒算 extern-block"
EXTERN_LINK_ARGUMENT_CELL = "Rust：构建脚本与构建依赖的库代码打 cargo:rustc-link-arg=-l…、cargo:rustc-flags=-l …、cargo::rustc-link-lib=…（两个冒号）：名单里的块也拒算 extern-block"
EXTERN_EXTERNAL_DEPENDENCY_CELL = "Rust：块所在的包、入口所在的包与它们顺依赖够到的包有工作区外的依赖（普通、开发、构建任一种，cc 这一类）：名单里的块也拒算 extern-block；依赖都在仓里的、没有依赖关系的包里的不拒算"
EXTERN_INSTRUCTION_LITERAL_CELL = "Rust：只有带 cargo: 或 cargo:: 前缀的字面串算打指令：构建脚本里的注释与裸词、不是构建依赖的库里的字面串都不算，名单里的块不拒算"
EXTERN_SYSTEM_LIBRARY_LIST_CELL = "Rust：extern 块放行的 C 库调用名单与 crates/singlefs-unit-graph 的 SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS 逐项相同（读那份源码比）"
EXTERN_LINK_ATTRIBUTE_SOURCE = ('#[cfg_attr(target_os = "linux", link(name = "fakesync"))]\nextern "C" {\n    fn fdatasync(fd: i32) -> i32;\n}\n'
                                'pub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n')
EXTERN_OTHER_BLOCK_LINK_SOURCE = ('#[link(name = "fakesync")]\nextern "C" {}\n\nextern "C" {\n    fn fdatasync(fd: i32) -> i32;\n}\n'
                                  'pub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n')
EXTERN_BUILD_SCRIPT_LINKING = {"crates/fextern/build.rs": 'fn main() {\n    println!("cargo:rustc-link-lib=fakesync");\n}\n'}
EXTERN_BUILD_SCRIPT_QUIET = {"crates/fextern/build.rs": 'fn main() {\n    println!("cargo:rerun-if-changed=build.rs");\n}\n'}
# 下面几组：名单里的块（EXTERN_LISTED_OWN_NAME，包 fextern）所在的产物链进来别的目标、依赖、构建依赖带来的库。每组 REFUSED 的每一棵树拒算、KEYED 的每一棵树算得出键
# 工作区成员那几格的夹具：members = ["."]（没有 [package] 与有 [package] 两种）、members 通配（放进名字带 [ ] 的目录里用）、两个同名的成员目录
WORKSPACE_DOT_UNNAMED_FIXTURE = {"Cargo.toml": '[workspace]\nmembers = ["."]\n', "src/lib.rs": "pub fn root_value() -> u32 {\n    1\n}\n"}
WORKSPACE_DOT_NAMED_FIXTURE = {"Cargo.toml": '[package]\nname = "rootpkg"\n\n[workspace]\nmembers = ["."]\n', "src/lib.rs": "pub fn root_value() -> u32 {\n    1\n}\n"}
# members 里一个成员写成多行基础串：开头的换行去掉；串里有反斜杠（行尾反斜杠连行）的，这一份与 crates/singlefs-unit-graph 的 workspace.rs 都报读不出
WORKSPACE_MULTILINE_PLAIN_FIXTURE = {"Cargo.toml": '[workspace]\nmembers = [\n    """\ncrates/alpha""",\n]\n', "crates/alpha/Cargo.toml": '[package]\nname = "alpha"\n'}
WORKSPACE_MULTILINE_BACKSLASH_FIXTURE = {"Cargo.toml": '[workspace]\nmembers = ["""crates/\\\n    alpha"""]\n', "crates/alpha/Cargo.toml": '[package]\nname = "alpha"\n'}
WORKSPACE_MEMBERS_FIXTURE = {"Cargo.toml": '[workspace]\nmembers = ["crates/*"]\n', "crates/alpha/Cargo.toml": '[package]\nname = "alpha"\n',
                             "crates/alpha/src/lib.rs": "pub fn alpha_value() -> u32 {\n    1\n}\n"}
# 两个 smoke.rs 各调自己包的库函数 value（返回 1 与 2）：键只由闭包里各单元的内容定，不带目标号、路径与名字，两份一字不差的测试会得同一把键
WORKSPACE_SAME_NAME_FIXTURE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/common", "tools/common"]\n',
    "crates/common/Cargo.toml": '[package]\nname = "commonlib"\n', "crates/common/src/lib.rs": "pub fn value() -> u32 {\n    1\n}\n",
    "crates/common/tests/smoke.rs": "#[test]\nfn smoke() {\n    assert_eq!(commonlib::value(), 1);\n}\n",
    "tools/common/Cargo.toml": '[package]\nname = "commontool"\n', "tools/common/src/lib.rs": "pub fn value() -> u32 {\n    2\n}\n",
    "tools/common/tests/smoke.rs": "#[test]\nfn smoke() {\n    assert_eq!(commontool::value(), 2);\n}\n",
}
EXTERN_LINKING_BLOCK = '#[link(name = "fakesync")]\nextern "C" {}\n'
EXTERN_QUIET_BUILD_SCRIPT = 'fn main() {\n    println!("cargo:rerun-if-changed=build.rs");\n}\n'
EXTERN_PACKAGE_WITH_DEPENDENCY = '[package]\nname = "fextern"\n\n[dependencies]\nflinkdep = { path = "../flinkdep" }\n'
EXTERN_PACKAGE_WITH_BUILD_DEPENDENCY = '[package]\nname = "fextern"\n\n[build-dependencies]\nflinkdep = { path = "../flinkdep" }\n'
EXTERN_DEPENDENCY_PACKAGE = '[package]\nname = "flinkdep"\n'
EXTERN_OTHER_TARGET_LINK_REFUSED = {
    "test-target": {"crates/fextern/tests/linked.rs": EXTERN_LINKING_BLOCK + "#[test]\nfn it_links() {}\n"},
    "test-target-of-a-dependency": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_DEPENDENCY, "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
                                    "crates/flinkdep/src/lib.rs": "pub fn quiet() {}\n",
                                    "crates/flinkdep/tests/linked.rs": EXTERN_LINKING_BLOCK + "#[test]\nfn it_links() {}\n"},
}
EXTERN_OTHER_TARGET_LINK_KEYED = {
    "test-target-without-link": {"crates/fextern/tests/quiet.rs": "#[test]\nfn it_runs() {}\n"},
}
EXTERN_DEPENDENCY_LINK_REFUSED = {
    "dependency-link-attribute": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_DEPENDENCY, "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
                                  "crates/flinkdep/src/lib.rs": EXTERN_LINKING_BLOCK},
    "dependency-build-script": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_DEPENDENCY, "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
                                "crates/flinkdep/src/lib.rs": "pub fn quiet() {}\n",
                                "crates/flinkdep/build.rs": 'fn main() {\n    println!("cargo:rustc-link-lib=fakesync");\n}\n'},
    "build-dependency-library": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_BUILD_DEPENDENCY, "crates/fextern/build.rs": "fn main() {\n    flinkdep::emit();\n}\n",
                                 "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
                                 "crates/flinkdep/src/lib.rs": 'pub fn emit() {\n    println!("cargo:rustc-link-lib=fakesync");\n}\n'},
    "dev-dependency-of-a-dependency": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_DEPENDENCY,
                                       "crates/flinkdep/Cargo.toml": '[package]\nname = "flinkdep"\n\n[dev-dependencies]\nflinklib = { path = "../flinklib" }\n',
                                       "crates/flinkdep/src/lib.rs": "pub fn quiet() {}\n",
                                       "crates/flinklib/Cargo.toml": '[package]\nname = "flinklib"\n', "crates/flinklib/src/lib.rs": EXTERN_LINKING_BLOCK},
}
EXTERN_DEPENDENCY_LINK_KEYED = {
    "dependency-without-link": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_DEPENDENCY, "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
                                "crates/flinkdep/src/lib.rs": "pub fn quiet() {}\n"},
    "build-dependency-library-mentioning-the-instruction-in-a-comment": {
        "crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_BUILD_DEPENDENCY, "crates/fextern/build.rs": "fn main() {\n    flinkdep::emit();\n}\n",
        "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
        "crates/flinkdep/src/lib.rs": '// 不打 cargo:rustc-link-lib 指令\npub fn emit() {\n    println!("cargo:rerun-if-changed=build.rs");\n}\n'},
    "unrelated-package-with-every-kind-of-instruction": {
        "crates/funrelated/Cargo.toml": '[package]\nname = "funrelated"\n\n[dependencies]\nserde = "1"\n', "crates/funrelated/src/lib.rs": EXTERN_LINKING_BLOCK,
        "crates/funrelated/build.rs": 'fn main() {\n    println!("cargo:rustc-link-lib=fakesync");\n}\n'},
}
EXTERN_LINK_ARGUMENT_REFUSED = {
    "link-arg": {"crates/fextern/build.rs": 'fn main() {\n    println!("cargo:rustc-link-arg=-lfakesync");\n}\n'},
    "rustc-flags": {"crates/fextern/build.rs": 'fn main() {\n    println!("cargo:rustc-flags=-l fakesync");\n}\n'},
    "build-dependency-library-link-arg": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_BUILD_DEPENDENCY, "crates/fextern/build.rs": "fn main() {\n    flinkdep::emit();\n}\n",
                                          "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
                                          "crates/flinkdep/src/lib.rs": 'pub fn emit() {\n    println!("cargo:rustc-link-arg=-lfakesync");\n}\n'},
    "two-colon-link-lib": {"crates/fextern/build.rs": 'fn main() {\n    println!("cargo::rustc-link-lib=fakesync");\n}\n'},
    "two-colon-link-arg-in-a-build-dependency-library": {
        "crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_BUILD_DEPENDENCY, "crates/fextern/build.rs": "fn main() {\n    flinkdep::emit();\n}\n",
        "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
        "crates/flinkdep/src/lib.rs": 'pub fn emit() {\n    println!("cargo::rustc-link-arg=-lfakesync");\n}\n'},
}
EXTERN_LINK_ARGUMENT_KEYED = {
    "build-script-without-link": {"crates/fextern/build.rs": EXTERN_QUIET_BUILD_SCRIPT},
}
EXTERN_EXTERNAL_DEPENDENCY_REFUSED = {
    "external-build-dependency": {"crates/fextern/Cargo.toml": '[package]\nname = "fextern"\n\n[build-dependencies]\ncc = "1"\n',
                                  "crates/fextern/build.rs": EXTERN_QUIET_BUILD_SCRIPT},
    "external-build-dependency-without-a-build-script": {"crates/fextern/Cargo.toml": '[package]\nname = "fextern"\n\n[build-dependencies]\ncc = "1"\n'},
    "external-normal-dependency": {"crates/fextern/Cargo.toml": '[package]\nname = "fextern"\n\n[dependencies]\nserde = "1"\n'},
    "external-development-dependency": {"crates/fextern/Cargo.toml": '[package]\nname = "fextern"\n\n[dev-dependencies]\nproptest = "1"\n'},
    "external-dependency-of-a-dependency": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_DEPENDENCY,
                                            "crates/flinkdep/Cargo.toml": '[package]\nname = "flinkdep"\n\n[dependencies]\nlibc = "0.2"\n',
                                            "crates/flinkdep/src/lib.rs": "pub fn quiet() {}\n"},
    "path-dependency-outside-the-repository": {"crates/fextern/Cargo.toml": '[package]\nname = "fextern"\n\n[dependencies]\nfaraway = { path = "../../../faraway" }\n'},
}
EXTERN_EXTERNAL_DEPENDENCY_KEYED = {
    "build-dependency-in-the-repository": {"crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_BUILD_DEPENDENCY, "crates/fextern/build.rs": EXTERN_QUIET_BUILD_SCRIPT,
                                           "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE, "crates/flinkdep/src/lib.rs": "pub fn quiet() {}\n"},
}
# 只有带 cargo: 或 cargo:: 前缀的字面串算打指令；每棵树里名单里的块（EXTERN_LISTED_OWN_NAME，包 fextern）都不拒算
EXTERN_INSTRUCTION_LITERAL_KEYED = {
    "build-script-mentioning-the-instruction-in-a-comment": {
        "crates/fextern/build.rs": '// 不打 rustc-link-lib 指令\nfn main() {\n    println!("cargo:rerun-if-changed=build.rs");\n}\n'},
    "build-script-with-the-bare-word-in-a-string": {
        "crates/fextern/build.rs": 'const INSTRUCTION: &str = "rustc-link-lib";\nfn main() {\n    println!("cargo:rerun-if-changed=build.rs");\n}\n'},
    "normal-dependency-library-with-the-literal": {
        "crates/fextern/Cargo.toml": EXTERN_PACKAGE_WITH_DEPENDENCY, "crates/flinkdep/Cargo.toml": EXTERN_DEPENDENCY_PACKAGE,
        "crates/flinkdep/src/lib.rs": 'pub fn text() -> &\'static str {\n    "cargo:rustc-link-lib=fakesync"\n}\n'},
}

FAKE_TOOLCHAIN_SCRIPT = '#!/bin/sh\necho "{name} $(cat "{version}")"\n'

# 登记表点名的程序（文件头「登记表点名的程序」）那一格的夹具：run_probe 起的脚本路径来自登记行，登记表写在 REGISTRY_LISTED_PROGRAMS 里
REGISTRY_FIXTURE = {
    "pkg/probes.py": '''import os
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REGISTRY = "pkg/registry.tsv"
REGISTRY_LISTED_PROGRAMS = {"run_probe": (REGISTRY, r"probe=(\\S+)")}


def run_probe(root, parts):
    return subprocess.run(["bash", os.path.join(root, parts["script"])], check=False).returncode


def check_all():
    with open(os.path.join(ROOT, REGISTRY), encoding="utf-8") as handle:
        rows = handle.read().splitlines()
    return [run_probe(ROOT, {"script": row.split("probe=")[1]}) for row in rows]
''',
    "pkg/registry.tsv": "alpha\tprobe=pkg/probe_alpha.sh\nbeta\tprobe=pkg/probe_beta.sh\n",
    "pkg/probe_alpha.sh": "#!/usr/bin/env bash\necho alpha\n",
    "pkg/probe_beta.sh": "#!/usr/bin/env bash\necho beta\n",
}

# 按行进键的登记表（文件头「按行进键的登记表」）那一格的夹具：guard.py 按 ROW_KEYED_TABLES 登记的取用函数 registered_selftest 查一行，audit 不在登记里、读整张
ROW_KEYED_FIXTURE = {
    ".claude/hooks/selftest-registry.tsv": "scripts/a.py\tselftest\t# 甲\nscripts/b.py\tselftest\t# 乙\n.claude/gate.d/x.sh\tsamples:tools\t# 丙\n",
    "pkg/guard.py": '''import os

REGISTRY = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), ".claude", "hooks", "selftest-registry.tsv")


def selftest_registry_at(path):
    rows = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle.read().split("\\n"):
            fields = line.split("\\t")
            if len(fields) > 1:
                rows.setdefault(fields[0], fields[1])
    return rows


def read_selftest_registry(path):
    return selftest_registry_at(path)


def registered_selftest(key):
    return read_selftest_registry(REGISTRY).get(key)


def judge(key):
    return registered_selftest(key) is not None


def audit():
    return sorted(read_selftest_registry(REGISTRY))
''',
}

LOADER_FILES = (".claude/scripts/preflight.sh", ".claude/scripts/project_preflight.py")
CANONICAL_FILES = ("preflight.sh", "preflight.py", "proc.py")   # preflight.py 按名字载入同目录的 proc.py（load_package_module）

IMPORT_TIME_FIXTURE = {
    "tool/entry.py": ("import sys\n\nimport helper\n\n\ndef judge():\n    return helper.verdict()\n\n\n"
                      "if __name__ == \"__main__\":\n    sys.exit(judge())\n"),
    "tool/helper.py": ("import os\n\nUNUSED = 0\n\n\nclass Settings:\n    threshold = 0\n\n    def __init__(self):\n        self.value = 1\n\n\n"
                       "def verdict():\n    return 0\n\n\ndef unused(value=0):\n    return value\n"),
}
IMPORT_TIME_ENTRY = "tool/entry.py::judge"
# 每一种改动：(名, 文件, 原文, 改成) —— 前四种是 import 时就执行、让 python3 tool/entry.py 退非 0 的模块级语句，末一种是对照：
# 改的是程序入口（main-guard）、不在 judge 的闭包里，退出码变了而键不变，这一格的判法必须认出它
IMPORT_TIME_EDITS = (
    ("模块级 import", "tool/helper.py", "import os\n", "from os import missing_name_for_selftest\n"),
    ("模块级赋值", "tool/helper.py", "UNUSED = 0\n", "UNUSED = int(\"selftest\")\n"),
    ("类头", "tool/helper.py", "    threshold = 0\n", "    threshold = int(\"selftest\")\n"),
    ("没人调的 def 的缺省值", "tool/helper.py", "def unused(value=0):", "def unused(value=int(\"selftest\")):"),
)
IMPORT_TIME_CONTROL = ("程序入口（main-guard）", "tool/entry.py", "    sys.exit(judge())\n", "    sys.exit(judge() + 3)\n")
IMPORT_TIME_EXIT_CELL = "python：import 时执行的模块级 import、赋值、类头与 def 缺省值改了让程序退出码变的，入口的键都变（键不变而退出码变的必红，对照那一种必须认出）"
IMPORT_TIME_CLASS_CELL = "python：只因 import 时执行收进来的类头不算被点名，类里的 __init__ 改了不作废"
EXCLUDED_EDGE_PYTHON_CELL = "不走的子进程边（夹具）：登记了的 python 单元起的那一条不连、同一个单元起的另一个程序照连、不登记时那一条照连"
EXCLUDED_EDGE_SHELL_CELL = "不走的子进程边（夹具）：登记了的 shell 函数起的那一条不连、同一个函数起的另一个程序照连、不登记时那一条照连"
# 每个入口单元起两个程序：hint 是自证期间登记成不走的那一条，kept 不登记；EXCLUDED_PROGRAM_EDGES 平时的内容不进这一格
EXCLUDED_EDGE_FIXTURE = {
    "tool/entry.py": ("import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
                      "def judge():\n    subprocess.run([sys.executable, os.path.join(HERE, \"kept.py\")], check=False)\n"
                      "    return subprocess.run([sys.executable, os.path.join(HERE, \"hint.py\")], check=False)\n"),
    "tool/kept.py": "def work():\n    return 1\n\n\nif __name__ == \"__main__\":\n    work()\n",
    "tool/hint.py": "def work():\n    return 2\n\n\nif __name__ == \"__main__\":\n    work()\n",
    "tool/run.sh": '''#!/usr/bin/env bash
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
launch() {
  bash "$here/kept.sh"
  bash "$here/hint.sh"
}
launch
''',
    "tool/kept.sh": "#!/usr/bin/env bash\necho kept\n",
    "tool/hint.sh": "#!/usr/bin/env bash\necho hint\n",
}
EXCLUDED_EDGE_PYTHON_ENTRY = "tool/entry.py::judge"
EXCLUDED_EDGE_SHELL_ENTRY = "tool/run.sh::launch"
EXCLUDED_EDGE_FIXTURE_EDGES = {
    ("tool/entry.py", "judge", "tool/hint.py"): "夹具：python 单元起的程序，只打提示",
    ("tool/run.sh", "launch", "tool/hint.sh"): "夹具：shell 函数起的程序，只打提示",
}
ROOT_IGNORED_SCRIPTS_CELL = ("仓根被 .gitignore 挡着的私有脚本：列仓根整层起的 multi-host-quiesce-*.sh 不进闭包、不拒算；"
                             "同一层没挡着、同样起仓外路径的那一份照连、照拒算 shell-path-outside-root")
# 入口照 os.path.join(<根>, <折不出的名字>) 起 bash（lib_reuse_checks.py 的 run_sample 那一种），列到仓根整层的两份脚本；两份都起同一个仓外路径，
# 只有 ROOT_IGNORED_SCRIPT 被仓根的 .gitignore 挡着
ROOT_IGNORED_SCRIPT = "multi-host-quiesce-sample.sh"
ROOT_VISIBLE_SCRIPT = "visible-outside-root.sh"
ROOT_IGNORED_SCRIPTS_FIXTURE = {
    ".gitignore": "# 私有清场脚本\n/multi-host-quiesce-*.sh\n",
    ROOT_IGNORED_SCRIPT: "#!/usr/bin/env bash\nbash /singlefs-function-units-outside-root/stop.sh\n",
    ROOT_VISIBLE_SCRIPT: "#!/usr/bin/env bash\nbash /singlefs-function-units-outside-root/stop.sh\n",
    "tool/launch.py": ("import os\nimport subprocess\n\nROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))\n\n\n"
                       "def launch(name):\n    return subprocess.run([\"bash\", os.path.join(ROOT, name)], check=False)\n"),
}
ROOT_IGNORED_SCRIPTS_ENTRY = "tool/launch.py::launch"
REAL_ENTRIES_CELL = "真仓：判法入口从登记现取（registered_judging_entries），形式化入口在里面、都认得出"
REAL_TREE_IGNORED = shutil.ignore_patterns("__pycache__", "fixtures", "kb", "target")
REAL_FORMAL_SCRIPT = "scripts/gate/verus-proof-check.py"
# 真仓上的三格，入口都是形式化入口（check_proof 经 reuse_checks.ReuseSentinel 等点到 lib_reuse_checks.py，那份模块因此被载入）：
# (格名, 文件, 原文, 改成)；原文在那份文件里要恰好一次，对不上这一格红、出路是照新原文改这里
REAL_IMPORT_TIME_EDITS = (
    ("真仓：改 scripts/gate/lib_reuse_checks.py 一个模块级 import，形式化入口的键变", "scripts/gate/lib_reuse_checks.py", "\nimport re\n", "\nimport re, json\n"),
    ("真仓：改 scripts/gate/lib_reuse_checks.py 的一个类头（ReuseSentinel），形式化入口的键变", "scripts/gate/lib_reuse_checks.py",
     "\nclass ReuseSentinel:\n", "\nclass ReuseSentinel:\n    IMPORT_TIME_SELFTEST = 1\n"),
    ("真仓：改一个没人调的 def 的缺省值（scripts/gate/lib_reuse_checks.py 的 write_target_last_run），形式化入口的键变", "scripts/gate/lib_reuse_checks.py",
     'def write_target_last_run(script_key, target, exit_code, test_filter="", target_file=""):',
     'def write_target_last_run(script_key, target, exit_code, test_filter=None, target_file=""):'),
)

_REAL_IMPORT_TIME_RESULTS = {}

# 「键不变而退出码变」必红那一格（夹具）：quiet.py 与 leak.py 都起 hint.py，两条边自证期间都登记成不走；改坏 hint.py 之后
# quiet.py 吞掉它的退出码、leak.py 把它的退出码当自己的。两个的键都不变，退出码只有 leak.py 变，判法必须认出 leak.py
EXCLUDED_EDGE_EXIT_CELL = "不走的子进程边（夹具）：切了边的程序改坏后包装退出码变了而键不变的必红（对照 leak.py 必须认出），吞掉退出码的 quiet.py 不红"
EXCLUDED_EDGE_EXIT_HEAD = "import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
EXCLUDED_EDGE_EXIT_LAUNCH = "subprocess.run([sys.executable, os.path.join(HERE, \"hint.py\")], check=False)"
EXCLUDED_EDGE_EXIT_FIXTURE = {
    "tool/hint.py": "def work():\n    return 0\n\n\nif __name__ == \"__main__\":\n    work()\n",
    "tool/quiet.py": EXCLUDED_EDGE_EXIT_HEAD + f"def run():\n    {EXCLUDED_EDGE_EXIT_LAUNCH}\n    return 0\n\n\nif __name__ == \"__main__\":\n    sys.exit(run())\n",
    "tool/leak.py": EXCLUDED_EDGE_EXIT_HEAD + f"def run():\n    return {EXCLUDED_EDGE_EXIT_LAUNCH}.returncode\n\n\nif __name__ == \"__main__\":\n    sys.exit(run())\n",
}
EXCLUDED_EDGE_EXIT_WRAPPERS = ("tool/quiet.py", "tool/leak.py")
EXCLUDED_EDGE_EXIT_EDGES = {(wrapper, "run", "tool/hint.py"): "夹具：自证期间登记成不走" for wrapper in EXCLUDED_EDGE_EXIT_WRAPPERS}
EXCLUDED_EDGE_EXIT_EDIT = ("tool/hint.py", "    work()\n", "    raise RuntimeError(\"selftest\")\n")

# 登记的准入边（ADMISSION_EDGE）的三格，都在本仓上量（准入异常那一格起本仓的跑法脚本，只走到计划那一步、编译换成假的）
ADMISSION_EDGE_KEY = "E183/h2"
ADMISSION_EDGE_ANOMALY_CELL = ("不走的子进程边（真仓 E183 准入）：准入这条路上的异常（崩、被杀、退 77、退 3、退 0 不打行、打别的行）"
                               "只让计划那一步退非 0、不起编译、不写计划，读数出不来")
ADMISSION_EDGE_STRIP_CELL = "不走的子进程边（真仓 E183 准入）：准入往标准输出打的只有 E7INPUT 行（键算出、拒算两样），判法读产物前去掉这几行之后读数一行不差"
ADMISSION_EDGE_REACH_CELL = "不走的子进程边（真仓 E183 准入）：切边前后少掉的单元都在 admission.py 当程序起的闭包里（只经这条边可达），且确实少了 admission.py"
# 准入异常那一格的假准入：(名, 正文)。每个都先在 ADMISSION_EDGE_MARK 指的文件里记一笔，证明计划那一步真问到了它
ADMISSION_EDGE_ANOMALIES = (
    ("崩（抛异常）", "raise RuntimeError(\"selftest\")\n"),
    ("被 SIGKILL 杀", "import signal\nos.kill(os.getpid(), signal.SIGKILL)\n"),
    ("退 77", "import sys\nsys.exit(77)\n"),
    ("退 3", "import sys\nsys.exit(3)\n"),
    ("退 0 什么都不打", "pass\n"),
    ("退 0 打别的行", "print(\"selftest junk\")\n"),
)
# 弄坏开关 admission-edge-admitted-sample 下多跑的那一个：放行，计划往下走、起编译，这一格必须红
ADMISSION_EDGE_ADMITTED = ("放行（弄坏开关 admission-edge-admitted-sample 加的）",
                           "print(\"E7INPUT name=input_fingerprint key=E183/h2 sha256=\" + \"0123456789abcdef\" * 4 + \" units=2\")\n")
ADMISSION_EDGE_MARK_LINE = "import os\nopen(os.environ[\"ADMISSION_EDGE_MARK\"], \"w\").write(\"asked\")\n"
ADMISSION_EDGE_FAKE_CARGO = "#!/usr/bin/env bash\necho \"$*\" >> \"$ADMISSION_EDGE_CARGO_CALLS\"\nexit 1\n"
ADMISSION_EDGE_DROPPED_VARIABLES = ("CARGO_TARGET_DIR", "MULTI_HOST_RUN_PEER_SCRATCH")
ADMISSION_EDGE_READINGS = ("E7RESULT name=selftest_reading value=1", "E7RESULT name=done emitted=1")

# 不拒算的程序词里领卡客户端那两行（UNKEYED_PROGRAM_WORDS 里 device_card_lease.py 的 take_card、return_card 起的 client；文件头「不拒算的程序词」）配的三格，
# 都在本仓上量。异常与去行两格在子进程里载入本仓 device_card_lease.py 的原文，照 row_driver.py 的 run_device_on_a_leased_card 的次序调 take_card、装置、
# return_card，领卡客户端是 CARD_LEASE_CLIENT_WORDS_FAKE_CLIENT；可达那一格只读源码
CARD_LEASE_CLIENT_WORDS_LAUNCHER = "scripts/experiment-apparatus/e171_three_oracle_collusion/device_card_lease.py"
CARD_LEASE_CLIENT_WORDS_FUNCTIONS = ("take_card", "return_card")
CARD_LEASE_CLIENT_WORDS_WORD = "client"
# 经 device_card_lease.py 领卡、还卡的调用方（可达那一格逐份读源码判）
CARD_LEASE_CLIENT_WORDS_CALLERS = ("scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py",
                                   "scripts/experiment-apparatus/e171_three_oracle_collusion/shared_steps.py")
CARD_LEASE_CLIENT_WORDS_ANOMALY_CELL = ("不拒算的程序词（真仓领卡客户端）：device_card_lease.py 的 take_card 那一回的异常（抛异常、被 SIGKILL 杀、退 3、退 0 不打卡串、"
                                        "退 0 只打空白）抛 CardLeaseRefused、装置不起、标准输出空着；return_card 那一回的异常（抛异常、被 SIGKILL 杀、退 1、退 3）"
                                        "装置照起一回、退出码与标准输出就是装置的")
CARD_LEASE_CLIENT_WORDS_STRIP_CELL = ("不拒算的程序词（真仓领卡客户端）：device_card_lease.py 的 return_card 那一回客户端往标准输出打的行转到标准错误，"
                                      "调用方的标准输出与装置打的读数一行不差")
CARD_LEASE_CLIENT_WORDS_REACH_CELL = ("不拒算的程序词（真仓领卡客户端）：row_driver.py 与 shared_steps.py 领卡、还卡都经 device_card_lease.py 的 take_card、return_card，"
                                      "不经别处起领卡客户端；device_card_lease.py 里起子进程的只有这两个函数，起的头一个词是登记的 client")
# return_card 把客户端的标准输出转走的原文（左边，在 device_card_lease.py 里要恰好一次）；弄坏开关 card-lease-client-words-return-output-kept 下换成右边
CARD_LEASE_CLIENT_WORDS_RETURN = ('subprocess.run([client, "lease", "return", cards], cwd=tree_root, stdout=subprocess.PIPE)',
                                  'subprocess.run([client, "lease", "return", cards], cwd=tree_root, stdout=None)')
CARD_LEASE_CLIENT_WORDS_CARD = "selftest-card"

CARD_LEASE_CLIENT_WORDS_READINGS = ("E7RESULT name=selftest_reading value=1", "E7RESULT name=done emitted=1")
CARD_LEASE_CLIENT_WORDS_REFUSED_EXIT = 90   # 起法接到 CardLeaseRefused（装置没起）时子进程退这个码
# 领卡那一回、还卡那一回各演的异常：(说法, 假客户端的结局)
CARD_LEASE_CLIENT_WORDS_TAKE_ANOMALIES = (("抛异常", "raised"), ("被 SIGKILL 杀", "killed"), ("退 3", "refused"), ("退 0 不打卡串", "silent"),
                                          ("退 0 只打空白", "blank"))
CARD_LEASE_CLIENT_WORDS_RETURN_ANOMALIES = (("抛异常", "raised"), ("被 SIGKILL 杀", "killed"), ("退 1", "unanswered"), ("退 3", "refused"))
CARD_LEASE_CLIENT_WORDS_DEVICE_EXITS = (0, 3)
# 弄坏开关 card-lease-client-words-granted-sample 下领卡那几个异常里多跑的那一个：领到了，装置照起，这一格必须红
CARD_LEASE_CLIENT_WORDS_GRANTED_SAMPLE = ("领到了（弄坏开关 card-lease-client-words-granted-sample 加的）", "granted")
# 假领卡客户端：每一回往 CARD_LEASE_CLIENT_WORDS_EVENTS 记「lease:<take|return>:<卡串，没给写 all>」，标准错误照真客户端打一行在等；
# 领卡那一回照 CARD_LEASE_CLIENT_WORDS_TAKE、还卡那一回照 CARD_LEASE_CLIENT_WORDS_RETURN 演结局（granted 是领到了打卡串、还了退 0）
CARD_LEASE_CLIENT_WORDS_FAKE_CLIENT = f'''#!/usr/bin/env python3
import os
import signal
import sys

verb = sys.argv[2] if len(sys.argv) > 2 else ""
with open(os.environ["CARD_LEASE_CLIENT_WORDS_EVENTS"], "a", encoding="utf-8") as handle:
    handle.write("lease:" + verb + ":" + (" ".join(sys.argv[3:]) or "all") + "\\n")
print("TASK_SERVER_CARD_LEASE state=waiting sequence=1 cards='' why=selftest", file=sys.stderr, flush=True)
mode = os.environ["CARD_LEASE_CLIENT_WORDS_TAKE" if verb == "take" else "CARD_LEASE_CLIENT_WORDS_RETURN"]
if mode == "granted":
    if verb == "take":
        print({CARD_LEASE_CLIENT_WORDS_CARD!r}, flush=True)
    sys.exit(0)
if mode == "raised":
    raise RuntimeError("selftest")
if mode == "killed":
    os.kill(os.getpid(), signal.SIGKILL)
if mode == "noisy":
    print({CARD_LEASE_CLIENT_WORDS_NOISE!r}, flush=True)
    print({CARD_LEASE_CLIENT_WORDS_NOISE!r}, file=sys.stderr, flush=True)
    sys.exit(0)
if mode == "blank":
    print("   ", flush=True)
    sys.exit(0)
sys.exit({{"refused": 3, "unanswered": 1, "silent": 0}}[mode])
'''
# 起法：载入 $1 那一份 device_card_lease.py，领卡、装置（记一笔、打读数）、还卡，退装置的码；领卡没成退 CARD_LEASE_CLIENT_WORDS_REFUSED_EXIT、装置不起
CARD_LEASE_CLIENT_WORDS_HARNESS = '''import importlib.util
import os
import sys

specification = importlib.util.spec_from_file_location("device_card_lease_for_program_words_selftest", sys.argv[1])
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
client, root = sys.argv[2], sys.argv[3]
try:
    cards = module.take_card(client, root, "selftest")
except module.CardLeaseRefused:
    sys.exit(int(os.environ["CARD_LEASE_CLIENT_WORDS_REFUSED_EXIT"]))
try:
    with open(os.environ["CARD_LEASE_CLIENT_WORDS_EVENTS"], "a", encoding="utf-8") as handle:
        handle.write("device:" + cards + "\\n")
    print(os.environ["CARD_LEASE_CLIENT_WORDS_READINGS"], flush=True)
    device_exit = int(os.environ["CARD_LEASE_CLIENT_WORDS_DEVICE_EXIT"])
finally:
    module.return_card(client, root, cards, "selftest")
sys.exit(device_exit)
'''
# 可达那一格的绕道样本：(说法, 文件, 原文, 改成)；原文在本仓那一份里要恰好一次。每个都要判出点名那份文件的问题；
# 弄坏开关 card-lease-client-words-other-launches-unchecked 下不核绕道，这几个判不出、这一格红
CARD_LEASE_CLIENT_WORDS_BYPASSES = (
    ("row_driver.py 拿 client_of 给的客户端自己起 lease take", CARD_LEASE_CLIENT_WORDS_CALLERS[0],
     "    client = device_card_lease.client_of(arguments.repo_root)\n",
     "    client = device_card_lease.client_of(arguments.repo_root)\n    subprocess.run([client, \"lease\", \"take\"], cwd=arguments.repo_root)\n"),
    ("shared_steps.py 照 <树根>/task-server/task 自己拼路径起 lease return", CARD_LEASE_CLIENT_WORDS_CALLERS[1],
     "    client = device_card_lease.client_of(repo_root)\n",
     "    client = device_card_lease.client_of(repo_root)\n"
     "    subprocess.run([os.path.join(repo_root, \"task-server\", \"task\"), \"lease\", \"return\", \"all\"], cwd=repo_root)\n"),
    ("device_card_lease.py 里 take_card、return_card 之外的函数起客户端", CARD_LEASE_CLIENT_WORDS_LAUNCHER,
     "\n\ndef never_broken(item: str) -> bool:\n",
     "\n\ndef peek_card(client: str, tree_root: str) -> int:\n    return subprocess.run([client, \"lease\", \"status\"], cwd=tree_root).returncode\n"
     "\n\ndef never_broken(item: str) -> bool:\n"),
)

# main-guard 的两种写法与几段合一（文件头「Python 闭包：import 时执行的模块级语句」；欠账 C947）那三格
MAIN_GUARD_IMPORT_CELL = ("python：被 import 的模块里 if __name__ == \"__main__\" and …: 那一段不进闭包（自证分支不收），"
                          "块外的模块级语句与认不出的形态（带 else、__name__ 那一项不在最左）照收")
MAIN_GUARD_PROGRAM_CELL = "python：被起的程序里每一段 main-guard 都收（if __name__ == \"__main__\" and …: 那一段与后面另一段），起它的参数不是字面量也连自证分支"
MAIN_GUARD_REAL_CELL = ("真仓：以 scripts/hosts/memory_peaks_lib.py 的 command_key 为判法入口，闭包不收它的自证（run_self_test、self_test_cells），"
                        "改 self_test_cells 的函数体判法单元指纹不变")

MAIN_GUARD_REAL_LIBRARY = "scripts/hosts/memory_peaks_lib.py"
MAIN_GUARD_REAL_ENTRY = f"{MAIN_GUARD_REAL_LIBRARY}::command_key"   # 峰值表的键：库里不碰自证的一个函数；做入口时库的 module-body 照收、main-guard 不收
MAIN_GUARD_REAL_SELFTEST_UNITS = ("run_self_test", "self_test_cells")
MAIN_GUARD_REAL_EDITED = "self_test_cells"   # 改它的函数体：它只经 main-guard 里的 run_self_test 到，闭包不收它，指纹就不变

REAL_JUDGING_CLOSURES_CELL = "真仓：登记的每个判法入口的闭包都不拒算（拒算的那一路判定一条都写不进库），夹具里的 globals() 查找判得出来"
JUDGING_REFUSAL_FIXTURE = {"tool/judge.py": 'def verdict(row):\n    original = globals().get("verdict")\n    return original is not None and row\n\n\ndef judge(row):\n    return verdict(row)\n'}

E171_CLOSURE_CELL = ("真仓：E171 重跑的两份入口脚本（名单核对 list_check.py、变异驱动 mutation_driver.py）的闭包都不拒算；没被单测点名的自证格（reuse_library 的 cell_rp1_layers_end_to_end、"
                     "cell_reuse_three_questions）与 admission.py 的自证（run_selftest）不在闭包里；名单核对的闭包里没有 reuse_library.py 的单元")
# 每份入口脚本的闭包里不许出现的 (文件, 单元名)：自证格只有自证走到。admission.py 经 long_run_item_verdicts.py 在进程里 import 进来，
# 它的子命令函数随模块级的 COMMANDS 表（import 时执行的赋值）进闭包，自证 run_selftest 不在那张表里、不该进来
E171_CLOSURE_ABSENT = (("reuse_library.py", "cell_rp1_layers_end_to_end"), ("reuse_library.py", "cell_reuse_three_questions"))
E171_ADMISSION_ABSENT = ("run_selftest",)
E171_ENTRY_SCRIPTS = ("list_check.py", "mutation_driver.py")

# 跨轮缓存的登记（文件头「跨轮缓存的登记」）那两格
CROSS_RUN_CACHE_CELL = ("真仓：跨轮缓存登记表（cross_run_cache_registry）里的每一份，在 TMPDIR 指到一个临时目录、GATE_CROSS_RUN_TMPDIR 没设、在 git 仓里"
                        "（.git 是目录，或 git worktree 里 .git 是文件）时现取的位置都不落在 TMPDIR 之下；worktree 里落在 git 目录底下的几份与主仓共用 common-dir 那一处")
CROSS_RUN_CACHE_SCAN_CELL = ("真仓：提到 GATE_CROSS_RUN_TMPDIR 的脚本都登记进跨轮缓存表或在不登记表里写明为什么，不登记表里的每一份都在、都还提到它；"
                             "夹具里没登记的一份判得出来")
CROSS_RUN_CACHE_PROBE_SECONDS = 300
CROSS_RUN_CACHE_MENTION = "GATE_CROSS_RUN_TMPDIR"
CROSS_RUN_CACHE_SCANNED_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/scripts", ".claude/hooks", "task-server", "prune-server")
CROSS_RUN_CACHE_SCANNED_SUFFIXES = (".py", ".sh", ".bash")
CROSS_RUN_CACHE_SKIPPED_DIRECTORIES = frozenset(("fixtures", "__pycache__", "target", ".git", "node_modules"))
# 提到 GATE_CROSS_RUN_TMPDIR 而不往那里放跨轮缓存的脚本：{路径: 为什么不登记}；只缩不涨，指不到的、不再提到它的这一格判红
CROSS_RUN_CACHE_NOT_CACHES = {
    "scripts/hosts/test-environment-check.py": "只把它当 gate.sh 开跑前的临时目录、在那里扫残留，不往里放跨轮的东西",
    "scripts/kb/verify-citations.sh": "文本缓存每趟自己 mktemp -d、跑完连目录删；这个名字只出现在弄坏开关（改前的样子）与自证里",
    "scripts/gate/lib_function_units_selftest_fixtures_real_repo.py": "算键库的自证夹具，自证开跑前才载入；这个名字只出现在格名、扫描用的常量与夹具文本里，不放缓存",
    "scripts/gate/lib_function_units_closure_keys.py": "算键库里跨轮缓存登记表与现取位置的函数所在：这个名字只出现在说明与读 test-environment 脚本取位置的函数里，不往那里放缓存",
    "scripts/gate/lib_function_units_selftest_real_repo.py": "算键库的自证格：拷真仓起命令行 cross-run-caches、摘掉这个变量、扫提到它的脚本；这个名字只出现在格函数与说明里，不放缓存",
    "scripts/gate/lib_function_units_selftest_program_registries.py": ("算键库的自证格（编 unit-graph 那几格，with_unit_graph_cache）：把这个变量换成格自己临时目录底下的目录、"
                                                                      "跑完还原，程序缓存落在那一格的临时目录里随它删，不往真的跨轮缓存放东西"),
    "scripts/mutation/crates-mutation-rows.py": ("crates 变异的入口：这个名字只出现在文件头说明编译目录放哪的那一段里（编译目录放这一趟的运行目录，不读这个变量），"
                                                 "不往那里放缓存"),
}
CROSS_RUN_CACHE_SCAN_FIXTURE = {
    "scripts/sample/keeps-cache.sh": '#!/usr/bin/env bash\ncache_directory="${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/sample-cache"\nmkdir -p "$cache_directory"\n',
    "scripts/sample/plain.sh": "#!/usr/bin/env bash\necho plain\n",
}

# 两个互相收进对方闭包的入口（键里的 entry 行，entry_key_lines）：一个 trait impl 里的两个方法（方法连 impl 头、impl 头点名类型、类型照 R-隐式收它的
# trait impl 全部成员，两个方法收到的单元一模一样；core 的 impl BlockDevice for FileBackedBlockDevice 的样子），与 python 里互相调用的两个 def
ENTRY_UNIT_KEY_CELL = "键：两个互相收进对方闭包的入口，键不同（一个 trait impl 里的两个方法、python 里互相调用的两个 def：闭包的单元多重集相同，键各是各的）"
ENTRY_UNIT_KEY_RUST_FILE = "crates/fsiblings/src/lib.rs"
ENTRY_UNIT_KEY_FIXTURE = {
    "crates/fsiblings/Cargo.toml": '[package]\nname = "fsiblings"\n',
    ENTRY_UNIT_KEY_RUST_FILE: '''pub trait SampleStore {
    fn fill(&self, count: u64) -> u64;
    fn width(&self) -> u64;
}

pub struct SampleDevice {
    pub width: u64,
}

impl SampleStore for SampleDevice {
    fn fill(&self, count: u64) -> u64 {
        count.min(self.width)
    }

    fn width(&self) -> u64 {
        self.width
    }
}
''',
    "tool/pair.py": "def ping(count):\n    return pong(count - 1) if count > 0 else 0\n\n\ndef pong(count):\n    return ping(count - 1) if count > 0 else 1\n",
}
ENTRY_UNIT_KEY_PAIRS = (
    ("Rust", (f"{ENTRY_UNIT_KEY_RUST_FILE}::<impl SampleStore for SampleDevice>::fill", f"{ENTRY_UNIT_KEY_RUST_FILE}::<impl SampleStore for SampleDevice>::width")),
    ("python", ("tool/pair.py::ping", "tool/pair.py::pong")),
)

# unit_graph_key 的 entry 行（照 crates/singlefs-unit-graph 的 reuse_key 与 entry_key_lines）：一个 trait impl 里的方法入口（entry 行连 impl 头），
# 一个挂了两个文档测试的条目（文档测试入口记所挂条目的全部文档测试，照 Rust 那一份 resolve_entry_units 文档测试那一支）
ENTRY_LINES_SHAPE_CELL = ("键：unit_graph_key 照 crates/singlefs-unit-graph 的 reuse_key 带 entry 行（数据行之后、工具链行之前，排序去重；impl 方法入口连 impl 头；"
                          "文档测试入口记所挂条目的全部文档测试、各一行，闭包从它们一起收）")
ENTRY_LINES_RUST_KEYS_CELL = ("键：夹具上 unit_graph_key 与 crates/singlefs-unit-graph 的 unit-graph closure 打的键逐字相同"
                              "（一个 impl 方法入口、一个文档测试入口；Rust 那一侧是录下的键 ENTRY_LINES_RUST_KEYS）")
ENTRY_LINES_FILE = "crates/fentrylines/src/lib.rs"
ENTRY_LINES_METHOD = "<impl Shape for Square>::area"
ENTRY_LINES_HOST_LINE = "host: x86_64-unknown-linux-gnu"
ENTRY_LINES_FIXTURE = {
    "crates/fentrylines/Cargo.toml": '[package]\nname = "fentrylines"\nversion = "0.1.0"\nedition = "2021"\n',
    ENTRY_LINES_FILE: '''pub trait Shape {
    fn area(&self) -> u64;
}

pub struct Square {
    pub side: u64,
}

impl Shape for Square {
    fn area(&self) -> u64 {
        self.side * self.side
    }
}

/// Twice the value.
///
/// ```
/// assert_eq!(fentrylines::documented(2), 4);
/// ```
///
/// ```
/// assert_eq!(fentrylines::documented(3), 6);
/// ```
pub fn documented(value: u64) -> u64 {
    value * 2
}
''',
}
# Rust 那一侧录下的键（unit-graph closure 打的 key 行）：ENTRY_LINES_FIXTURE 用 write_tree 写出成一棵树（<根>），带 entry 行的那一版 crates/singlefs-unit-graph 编的
#   unit-graph closure <根> fentrylines::Square::area --toolchain-line 'host: x86_64-unknown-linux-gnu'               → "impl-method"
#   unit-graph closure <根> fentrylines::documented --target doctest --toolchain-line 'host: x86_64-unknown-linux-gnu'  → "doctest"
# 还没录的写 None（那一格判红、说明里打出 python 这一边的键）。夹具或 Rust 那一份的键的写法改了就重录。
ENTRY_LINES_RUST_KEYS = {
    "impl-method": "e74ceaa6abb0198ba10e62a909dccfc1e321557f4775cae362a3481a5a8762e7",
    "doctest": "3597d88a25cc2c3fad7d6f222e704d2ea205ccd9fe3c41ab81451c81391e6670",
}

# 标准写法的 main-guard 带 else：if 那一支是 main-guard，else 那一支在被 import 时执行、进 module-body
MAIN_GUARD_ELSE_CELL = ("python：if __name__ == \"__main__\": … else: … 被 import 时 else 那一支照收、if 那一支不收；被当程序起时 if 那一支照收")
MAIN_GUARD_ELSE_FIXTURE = {
    "tool/either.py": ("import sys\n\n\ndef main_work():\n    return 1\n\n\ndef imported_work():\n    return 2\n\n\ndef value():\n    return 0\n\n\n"
                       "if __name__ == \"__main__\":\n    sys.exit(main_work())\nelse:\n    imported_work()\n"),
    "tool/importer.py": "import either\n\n\ndef judge():\n    return either.value()\n",
    "tool/runner.py": ("import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
                       "def run():\n    return subprocess.run([sys.executable, os.path.join(HERE, \"either.py\")], check=False).returncode\n"),
}

# 判法入口拒算的并行进程数（judging_entry_refusal_workers）
JUDGING_ENTRY_WORKERS_CELL = ("判法入口拒算的并行进程数：没设 SINGLEFS_THREAD_CAP 取登记的每台默认重活线程数（头一行是注释的登记文件照样取到），"
                              "设了照它，不退回按核数算")

# verus! 里的 assume_specification（与 crates/singlefs-unit-graph 的
# tests/unit_graph_verus_assume_specifications_are_units_in_the_formal_view_and_blanked_in_the_product_view.rs 同一份夹具、同一套期望）
ASSUME_SPECIFICATION_FILE = "crates/fassume/src/lib.rs"
ASSUME_SPECIFICATION_FIXTURE = {
    "crates/fassume/Cargo.toml": '[package]\nname = "fassume"\nversion = "0.1.0"\nedition = "2021"\n',
    ASSUME_SPECIFICATION_FILE: '''pub const LIMIT_IN_BYTES: u64 = 32;

pub fn width_of(value: u64) -> u64 {
    value + 1
}

pub fn judge_limit() -> u64 {
    LIMIT_IN_BYTES
}

verus! {

#[allow(unsafe_code, reason = "夹具：Verus 把 assume_specification 展开成 unsafe fn 声明")]
pub assume_specification[ crate::LIMIT_IN_BYTES ] -> (limit: u64) returns 32u64;

assume_specification[ width_of ](value: u64) -> (width: u64)
    ensures width == value + 1;

pub assume_specification<Element>[ Vec::<Element>::len ](vector: &Vec<Element>) -> (length: usize);

pub open spec fn spec_limit() -> u64 {
    32
}

}
''',
    "Cargo.lock": "# 夹具不解析依赖\n",
}
ASSUME_SPECIFICATION_FORMS_CELL = ("verus!：assume_specification 带属性带 pub、不带 pub 带参数表与 ensures、带泛型参数表三种写法各切成一个单元"
                                   "（种类、名字、起止、按点名路径末段的名字找得到），dump 一条拒算都没有，后面的 spec fn 照旧自成单元")
ASSUME_SPECIFICATION_KEY_CELL = "verus!：点到常量的入口按名字连上它的 assume_specification，改它交回的值形式化键变"

# 只在 dev 依赖里开的特性（crates/singlefs-unit-graph 的 resolve_features：入口 crate 照目标种类顺 dev 依赖，别的 crate 只顺普通依赖）：
# fbin 的普通依赖不开 counters、dev 依赖开；fhelper 同样只在 dev 依赖里开，fvia 只经普通依赖链到 fhelper
FEATURE_SECTIONS_FIXTURE = {
    "crates/fcore/Cargo.toml": '[package]\nname = "fcore"\n\n[features]\ncounters = []\n',
    "crates/fcore/src/lib.rs": '''pub fn count() -> u32 {
    counted_value()
}
#[cfg(feature = "counters")]
fn counted_value() -> u32 {
    1
}
#[cfg(not(feature = "counters"))]
fn counted_value() -> u32 {
    0
}
''',
    "crates/fbin/Cargo.toml": ('[package]\nname = "fbin"\n\n[dependencies]\nfcore = { path = "../fcore" }\n\n'
                               '[dev-dependencies]\nfcore = { path = "../fcore", features = ["counters"] }\n'),
    "crates/fbin/src/main.rs": "fn main() {\n    let _ = fcore::count();\n}\n",
    "crates/fbin/tests/counting.rs": "#[test]\nfn counting_sees_the_counters() {\n    assert_eq!(fcore::count(), 1);\n}\n",
    "crates/fhelper/Cargo.toml": ('[package]\nname = "fhelper"\n\n[dependencies]\nfcore = { path = "../fcore" }\n\n'
                                  '[dev-dependencies]\nfcore = { path = "../fcore", features = ["counters"] }\n'),
    "crates/fhelper/src/lib.rs": "pub fn helped() -> u32 {\n    fcore::count()\n}\n",
    "crates/fvia/Cargo.toml": '[package]\nname = "fvia"\n\n[dependencies]\nfhelper = { path = "../fhelper" }\n',
    "crates/fvia/tests/via.rs": "#[test]\nfn via_the_helper() {\n    let _ = fhelper::helped();\n}\n",
}
FEATURE_SECTIONS_COUNTED_LINE = 4   # crates/fcore/src/lib.rs 里 cfg(feature = "counters") 那一份 counted_value 起的行（单元从属性起）
FEATURE_SECTIONS_UNCOUNTED_LINE = 8   # cfg(not(feature = "counters")) 那一份起的行
FEATURE_SECTIONS_ENTRY_CRATE_CELL = "R-目标：入口 crate 的 dev 依赖里才开的特性，不按 cfg(test) 编的 bin 当它没开，集成测试当它开着"
FEATURE_SECTIONS_LINKED_CRATE_CELL = "R-目标：经普通依赖链进来的 crate，它的 dev 依赖里开的特性哪个目标都不顺"

# verus! 里 } 之后紧跟的 ;（结构体字面量当初值的 const、impl 里的关联 const）：unit-graph 的 verus_item_starts 不拿 ; 当起头，; 归前一个条目、进它的指纹
VERUS_SEMICOLON_FILE = "crates/fsemicolon/src/lib.rs"
VERUS_SEMICOLON_CONSTANT = "pub const ORIGIN: Pair = Pair { left: 0, right: 0 };"
VERUS_SEMICOLON_ASSOCIATED = "pub const UNIT: Pair = Pair { left: 1, right: 1 };"
VERUS_SEMICOLON_FIXTURE = {
    "crates/fsemicolon/Cargo.toml": '[package]\nname = "fsemicolon"\n',
    VERUS_SEMICOLON_FILE: (f"verus! {{\npub struct Pair {{\n    pub left: u64,\n    pub right: u64,\n}}\n{VERUS_SEMICOLON_CONSTANT}\n"
                           f"impl Pair {{\n    {VERUS_SEMICOLON_ASSOCIATED}\n}}\npub fn total(pair: Pair) -> u64 {{\n    pair.left + pair.right\n}}\n}}\n"),
    "Cargo.lock": "# 夹具不解析依赖\n",
}
VERUS_SEMICOLON_CELL = ("verus!：} 之后紧跟的 ; 归前一个条目（结构体字面量当初值的 const 与 impl 里的关联 const），指纹是连 ; 在内的词法单元以 0x1f 连的 sha256，"
                        "后面的条目照旧切开")

PRODUCT_VIEW_SOURCE = """verus! {
pub open spec fn doubled(x: u64) -> u64 {
    x * 2
}
proof fn lemma_doubled_is_even(x: u64)
    ensures
        doubled(x) % 2 == 0,
{
    assert(doubled(x) == x * 2);
}
pub fn scaled(x: u64) -> (r: u64)
    requires
        x < 1000,
    ensures
        r == doubled(x),
{
    proof {
        lemma_doubled_is_even(x);
    }
    helper(x) * 2
}
fn helper(x: u64) -> u64 {
    x + 0
}
} // verus!
pub fn caller() -> u64 {
    scaled(3)
}
"""
# 只给 Verus 看的那几段：spec 与 proof 函数整条、具名返回值的「(r:」与「)」、requires 与 ensures、函数体里的 proof 块
PRODUCT_VIEW_GHOST_PIECES = (
    "pub open spec fn doubled(x: u64) -> u64 {\n    x * 2\n}",
    "proof fn lemma_doubled_is_even(x: u64)\n    ensures\n        doubled(x) % 2 == 0,\n{\n    assert(doubled(x) == x * 2);\n}",
    "(r: ",
    ")\n    requires\n        x < 1000,\n    ensures\n        r == doubled(x),",
    "proof {\n        lemma_doubled_is_even(x);\n    }",
)
PRODUCT_VIEW_FIXTURE = {
    "crates/fverus/Cargo.toml": '[package]\nname = "fverus"\n',
    PRODUCT_VIEW_FILE: PRODUCT_VIEW_SOURCE,
    "crates/fplain/Cargo.toml": '[package]\nname = "fplain"\n',
    "crates/fplain/src/lib.rs": "pub fn plain() -> u32 {\n    7\n}\n",
}
PRODUCT_VIEW_LEMMA_CELL = "产品行为视图：引理里加一句 assert(true);，产品行为键不变、形式化键变"
PRODUCT_VIEW_ENSURES_CELL = "产品行为视图：verus! 里执行函数的 ensures 改了，产品行为键不变"
PRODUCT_VIEW_EXECUTABLE_CELL = "产品行为视图：verus! 里被调到的执行代码改了，产品行为键变"

# 只在规格里点到的执行侧条目（HeaderModel 只出现在 ghost 字段的类型、ensures、let ghost、proof 块与循环头里）：unit-graph 剥完再切、再收闭包，
# 闭包里没有它；手工剥过的拷贝照原样切出来的 dump 当 Rust 那一份，照原样收它的闭包当 Rust 那一份的闭包
PRODUCT_VIEW_EDGES_FILE = "crates/fghostedges/src/lib.rs"
PRODUCT_VIEW_EDGES_ENTRY = f"{PRODUCT_VIEW_EDGES_FILE}::caller"
PRODUCT_VIEW_EDGES_SOURCE = """verus! {
pub struct HeaderModel {
    pub width: nat,
}
pub open spec fn spec_header(width: u64) -> HeaderModel {
    HeaderModel { width: width as nat }
}
pub struct Recorded {
    pub width: u64,
    pub history: Ghost<Seq<HeaderModel>>,
}
pub fn parse_header(width: u64) -> (parsed: Recorded)
    ensures
        spec_header(width) == (HeaderModel { width: parsed.width as nat }),
{
    let ghost before = spec_header(width);
    proof {
        assert(before.width == width as nat);
    }
    let mut total = 0u64;
    while total < width
        invariant
            total <= width,
            spec_header(total) == if total == 0 { HeaderModel { width: 0 } } else { spec_header(total) },
        decreases width - total,
    {
        total = total + 1;
    }
    Recorded { width: total, history: Ghost(Seq::empty()) }
}
} // verus!
pub fn caller() -> u64 {
    parse_header(3).width
}
"""
# 照 crates/singlefs-unit-graph 的剥法要剥的那几截：spec 函数整条、ghost 字段连逗号、具名返回值的「(名:」与「)」、ensures、let ghost、proof 块、
# 循环头的 invariant 与 decreases、字面量里列尾的 Ghost(…) 字段连它前面那个逗号
PRODUCT_VIEW_EDGES_GHOST_PIECES = (
    "pub open spec fn spec_header(width: u64) -> HeaderModel {\n    HeaderModel { width: width as nat }\n}",
    "pub history: Ghost<Seq<HeaderModel>>,",
    "(parsed: ",
    ")\n    ensures\n        spec_header(width) == (HeaderModel { width: parsed.width as nat }),",
    "let ghost before = spec_header(width);",
    "proof {\n        assert(before.width == width as nat);\n    }",
    "invariant\n            total <= width,\n            spec_header(total) == if total == 0 { HeaderModel { width: 0 } } else { spec_header(total) },\n"
    "        decreases width - total,",
    ", history: Ghost(Seq::empty())",
)
PRODUCT_VIEW_EDGES_CELL = ("产品行为视图：只在规格里点到的执行侧条目（ghost 字段的类型、ensures、let ghost、proof 块、循环头里点到的结构体）不进闭包，"
                           "闭包与 unit-graph 剥完再收的那一份逐个相同")
PRODUCT_VIEW_UNVERIFIED_CELL = "产品行为视图：这一份剥出来的与 Rust 那一份交的核不上，照原样那一份收（闭包只多不少，规格里点到的结构体照收）"

# 外属性 #[verifier::…]：Rust 那一份剥掉它（verus! 里外都剥），打头的属性一剥，单元的起始行往下挪；这一份照同一套词法剥了再切，两边的起止才对得上
PRODUCT_VIEW_ATTRIBUTE_LINE = "#[verifier::external_body]"
PRODUCT_VIEW_ATTRIBUTE_INSIDE_CELL = "产品行为视图：verus! 里的执行函数只多一行 #[verifier::external_body]，产品行为视图配得上、产品行为键不变"
PRODUCT_VIEW_ATTRIBUTE_OUTSIDE_CELL = "产品行为视图：没有 verus! 块的文件里的函数只多一行 #[verifier::external_body]，产品行为视图配得上、产品行为键不变"

FORKED_GATHER_CELL = ("自证并行：每条在自己 fork 出来的进程里跑，收回的按派出去的次序排、各带自己的输出；没收回结果的（退出码 0 却没写结果、退出码 3）"
                      "逐条点名，收回的条数与派出去的对不上不吃掉")

NESTED_POSITIONALS_CELL = ("shell：bash -c '…' 里套着的里层 bash -c \"…\\$1…\"：外层只换没转义的 $1…$9，转义的 \\$1 留给里层照它自己的参数折"
                           "（里层的 $1 是外层的 $2），连到它 import 的仓内模块")

# Rust 词法单元缓存的目录名（算键库文件头「Rust 词法单元的缓存」）那三格
RUST_LEX_NAMESPACE_CHANGED_CELL = "Rust 词法单元缓存的目录名：改格式号 RUST_LEX_CACHE_FORM、改一个词种名常量（IDENT 的值）、改 load_rust_lex 的读法，目录名各换"
RUST_LEX_NAMESPACE_KEPT_CELL = ("Rust 词法单元缓存的目录名：改 rust_lex 里一行注释、这份库别处一行注释、一个不在 rust_lex_cached 闭包里的函数（match_brackets），"
                                "目录名都不换")
RUST_LEX_NAMESPACE_CLOSURE_CELL = ("真仓：rust_lex_cached 的闭包里没有算键库整份的 source-text 与 data-file，读写缓存的两个函数、格式号与词种名常量在里面；"
                                   "只改算键库一行注释它的键不变，改 load_rust_lex 键变")
# 弄坏开关 rust-lex-cache-namespace-reads-own-file 在拷贝上给 rust_lex_cache_namespace 加回的那一句（读算键库自己的原文）：(原文, 改成)
RUST_LEX_NAMESPACE_OWN_FILE_EDIT = ('    label = ",".join(sorted(breaks))\n',
                                    '    label = ",".join(sorted(breaks))\n    with open(os.path.realpath(__file__), "rb") as handle:\n'
                                    '        label += hashlib.sha256(handle.read()).hexdigest()\n')
# 真仓那一格在拷贝上改的两处：(说明, 原文, 改成, 键该不该变)
RUST_LEX_NAMESPACE_CLOSURE_EDITS = (
    ("算键库一行注释", "# ── Rust 与 WGSL 的词法 ─", "# ── Rust 与 WGSL 的词法单元 ─", "same"),
    ("load_rust_lex 的读法", "        texts = [table[code] for code in text_codes]", "        texts = [table[code][:3] for code in text_codes]", "changed"),
)
# (说明, 原文, 改成)：原文在算键库里要恰好命中一次
RUST_LEX_NAMESPACE_CHANGED_EDITS = (
    ("格式号", 'RUST_LEX_CACHE_FORM = "singlefs-rust-lex-cache-v1"', 'RUST_LEX_CACHE_FORM = "singlefs-rust-lex-cache-v1-changed"'),
    ("词种名常量", 'IDENT, LITERAL, LIFETIME, PUNCT = "ident", "literal", "lifetime", "punct"',
     'IDENT, LITERAL, LIFETIME, PUNCT = "identifier", "literal", "lifetime", "punct"'),
    ("load_rust_lex 的读法", "        texts = [table[code] for code in text_codes]", "        texts = [table[code][:3] for code in text_codes]"),
)
RUST_LEX_NAMESPACE_KEPT_EDITS = (
    ("rust_lex 里一行注释", "                # 原始标识符的词按源码原样取、带着 r#", "                # 原始标识符的词照源码原样取、带着 r#"),
    ("算键库别处一行注释", "# ── Rust 与 WGSL 的词法 ─", "# ── Rust 与 WGSL 的词法单元 ─"),
    ("不在闭包里的函数", "    pairs, stack = {}, []", "    pairs, stack = dict(), []"),
)

# 文档串的读法（算键库文件头「文档串的读法」）那四格
DOCSTRING_WRITE_CELL = "python：模块级给 namedtuple 写 X.__doc__ = … 只写不读，别的文档串不跟着进闭包（改它键不变）"
DOCSTRING_AUGMENTED_CELL = "python：X.__doc__ += … 先读后写，照算读（改那段文档串键变）"
DOCSTRING_GETATTR_CELL = "python：getattr(<函数>, \"__doc__\") 读另一个函数的文档串（改那段文档串键变）"
DOCSTRING_GETDOC_CELL = "python：inspect.getdoc(<函数>) 读另一个函数的文档串（改那段文档串键变）"
DOCSTRING_READS_FIXTURE = {
    "pkg/docs.py": '''import collections
import inspect

Record = collections.namedtuple("Record", "name value")
Record.__doc__ = """Record 的文档串"""


def described():
    """described 的文档串"""
    return 1


def other():
    """other 的文档串"""
    return 2


def plain():
    return 3


def by_getattr():
    return getattr(described, "__doc__")


def by_getdoc():
    return inspect.getdoc(other)


def augmented():
    described.__doc__ += "（补一句）"
    return 4
''',
}

# 不走的调用边（算键库文件头「不走的调用边」）那三格
EXCLUDED_CALL_EDGE_CELL = "python：EXCLUDED_CALL_EDGES 登记的调用边不走：只经它读的数据文件改了键不变，同一个单元调的另一个函数读的改了键变"
EXCLUDED_CALL_EDGE_FIXTURE = {
    "pkg/tool.py": '''import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def locate():
    with open(os.path.join(ROOT, "pkg", "locator.txt"), encoding="utf-8") as handle:
        return handle.read()


def judge_input():
    with open(os.path.join(ROOT, "pkg", "judged.txt"), encoding="utf-8") as handle:
        return handle.read()


def entry():
    return locate() + judge_input()
''',
    "pkg/locator.txt": "locator\n",
    "pkg/judged.txt": "judged\n",
}
EXCLUDED_CALL_EDGE_FIXTURE_EDGES = {("pkg/tool.py", "entry", "locate"): "夹具：只在 EXCLUDED_CALL_EDGE_CELL 跑的那一会儿登记"}
UNIT_GRAPH_INPUTS_CELL = ("真仓：unit_graph_directory 的闭包里没有 built_unit_graph、build_unit_graph、unit_graph_inputs_digest；Cargo.lock 里头一个包的版本、"
                          ".claude/heavy-thread-default、crates/singlefs-unit-graph/src 里一份文件各改一处，键都不变")
UNIT_GRAPH_DIGEST_CELL = "unit-graph 程序的缓存目录名：Cargo.lock、.claude/heavy-thread-default、crates/singlefs-unit-graph/src 里一份文件各改一处，目录名各换"
UNIT_GRAPH_DIGEST_FIXTURE = {
    "crates/singlefs-unit-graph/Cargo.toml": '[package]\nname = "singlefs-unit-graph"\n',
    "crates/singlefs-unit-graph/src/main.rs": "fn main() {}\n",
    "Cargo.lock": "version = 4\n",
    ".claude/heavy-thread-default": "# 每台默认重活线程数（自证夹具）\n8  # 自证夹具\n",
}

# 自证夹具放在这一份（算键库文件头「自证夹具」）那两格
SELFTEST_FIXTURES_OUTSIDE_CELL = ("真仓：lib_function_units_selftest_fixtures.py 的单元不进 rust_lex_cached 的闭包（算键库 import 时执行的语句都在里面），"
                                  "那一份末尾加一句常量键不变")
SELFTEST_FIXTURE_NAMES_CELL = ("真仓：lib_function_units_selftest_fixtures.py 定义的名字，算键库运行期那一边（import 时执行的语句、selftest 够不到的函数与类、"
                               "别的脚本经属性调的函数）一个都不读，算键库自己也不再定义")
# 弄坏开关 selftest-fixtures-loaded-at-import 在拷贝上给算键库加的一句模块级赋值：(原文, 改成)
SELFTEST_FIXTURES_IMPORT_EDIT = ("\n\n\ndef match_brackets(kinds, texts):\n",
                                 "\n\nfrom lib_function_units_selftest_run import load_selftest_fixtures\nSELFTEST_FIXTURES_PROBE = load_selftest_fixtures\n\n\ndef match_brackets(kinds, texts):\n")
SELFTEST_FIXTURES_APPENDED = '\nSELFTEST_FIXTURES_APPENDED_PROBE = "自证在这一份末尾加的一句"\n'
# 算键库入口不在 import 时载入自证模块（算键库文件头「自证模块」）那一格；弄坏开关 selftest-modules-loaded-at-import 在拷贝上给入口末尾加的那一句
SELFTEST_MODULES_OUTSIDE_CELL = ("真仓：按路径载入算键库的入口（lib_mutation_units.py 的 UnitKeys 那几个方法）的闭包里没有 lib_function_units_selftest*.py 的单元、"
                                 "没有拒算（算键库入口 import 时不载入自证模块）")
# 按路径载入算键库的入口（UnitKeys 经 function_units() 载入 lib_function_units.py：被改坏的单元、点名的测试、覆盖落进的单元都从这里调它）
SELFTEST_MODULES_OUTSIDE_ENTRIES = ("scripts/mutation/lib_mutation_units.py::UnitKeys.mutated_units", "scripts/mutation/lib_mutation_units.py::UnitKeys.resolve_test",
                                    "scripts/mutation/lib_mutation_units.py::UnitKeys.executed_units")
SELFTEST_MODULES_IMPORT_LINE = "\nfrom lib_function_units_selftest_run import selftest  # noqa: E402,F401\n"
# 登记的函数在登记的文件里有定义（算键库文件头「不拒算的程序词」等几张登记表）
REGISTERED_FUNCTIONS_REAL_CELL = ("真仓：按（文件, 函数）登记的几张表（UNKEYED_PROGRAM_WORDS、LAUNCHED_REPOSITORY_PROGRAMS、BUILT_TOOL_SOURCES、BUILT_TOOL_LAUNCHES、"
                                  "CALLER_SUPPLIED_COMMANDS、REGISTRY_COMMAND_TEXTS、TAIL_READ_SCOPES、EXCLUDED_PROGRAM_EDGES、EXCLUDED_CALL_EDGES）每一行的函数都定义在登记的文件里")
# 测试服务代码不进键（算键库文件头「测试服务代码不进键」）的三格
TEST_SERVICE_CODE_TABLE_CELL = ("真仓：测试服务代码登记表 scripts/gate/test-service-code-paths.tsv 与 D13（验证路线） 已定项 17 三类代码表测试服务代码那一行逐项对上"
                                "（反引号里的每一项都登了、登的每一行都出自那一行、仓里的 crash_judge*.wgsl 逐份登了、登的 .wgsl 都在仓里、登的路径都在）")
TEST_SERVICE_CODE_REAL_CELL = ("真仓：is_test_service_code 判 crates/singlefs-unit-graph/src 下一份 .rs、一份 GPU 判器 .wgsl、登的 .rs 拆出的子模块是测试服务代码，"
                               "产品包里的一份与登的 .rs 同目录没登的一份不是")
TEST_SERVICE_CODE_UNKEYED_CELL = "键：闭包里测试服务代码的单元不进 closure_key（按条复用的测试函数键）：改测试服务代码里的一个单元键不变，改闭包里的产品代码键变"
# D13（验证路线） 已定项 17「三类代码」表测试服务代码那一行：(决策文件，那一行的行首)；第三列（包含）里反引号的每一项要在登记表里有一行
TEST_SERVICE_CODE_DECISION = (".claude/kb/decisions/13-验证路线.md", "| 测试服务代码 |")
# 仓里 GPU 判器的 WGSL 内核在哪几处找（名字以 crash_judge 起头的 .wgsl）：找到的每一份要登了（按文件，或落在登的前缀底下），登的 .wgsl 都要在找到的里面
TEST_SERVICE_CODE_KERNEL_PLACES = ("crates", "gpu-verification-server")
TEST_SERVICE_CODE_KERNEL_PREFIX = "crash_judge"
# 产品包（判「不是测试服务代码」的那一份从这里取）
TEST_SERVICE_CODE_PRODUCT_SOURCE = "crates/singlefs-core/src"
# 一个包里入口经 judge_on_the_card（夹具里登成测试服务代码的那一份）与 product 两个模块各取一个数
TEST_SERVICE_CODE_FIXTURE = {
    "crates/fservice/Cargo.toml": '[package]\nname = "fservice"\n',
    "crates/fservice/src/lib.rs": "mod judge_on_the_card;\nmod product;\n\npub fn entry_point() -> u32 {\n    judge_on_the_card::judged() + product::value()\n}\n",
    "crates/fservice/src/judge_on_the_card.rs": "pub fn judged() -> u32 {\n    11\n}\n",
    "crates/fservice/src/product.rs": "pub fn value() -> u32 {\n    21\n}\n",
}
TEST_SERVICE_CODE_FIXTURE_ENTRY = "crates/fservice/src/lib.rs::entry_point"
TEST_SERVICE_CODE_FIXTURE_ROWS = {"crates/fservice/src/judge_on_the_card.rs": "夹具：只在 TEST_SERVICE_CODE_UNKEYED_CELL 跑的那一会儿登记"}
# (改哪份, 原文, 改成, 期望键变没变)：逐个累加
TEST_SERVICE_CODE_FIXTURE_EDITS = (
    ("crates/fservice/src/judge_on_the_card.rs", "    11\n", "    12\n", "same"),
    ("crates/fservice/src/product.rs", "    21\n", "    22\n", "changed"),
)
# 算键库运行期数据文件的登记（RUNTIME_DATA_FILES，lib_function_units_registry.py）与算键库各份里以 LIBRARY_ROOT 拼出的读法逐项对上
RUNTIME_DATA_FILES_CELL = ("真仓：算键库各份（scripts/gate/lib_function_units*.py，不含自证模块与夹具）里以 LIBRARY_ROOT 拼出的读法，每一处要么是 RUNTIME_DATA_FILES 登的数据文件、"
                           "要么写在 RUNTIME_LIBRARY_ROOT_READS_UNLISTED（理由）里；登的每一项仓里有文件、也被读到；不登的每一行也被读到")
# 算键库不放进 RUNTIME_DATA_FILES 的 LIBRARY_ROOT 读法：键是拼出来的仓内路径（路径折不出的写「<算键库那份文件>::<函数名>」）→ 为什么不登。
# 这些都不是算键必读的数据文件：只给命令行 cross-run-caches、--judging 与 Rust 产品行为视图，样本仓里缺了不影响 Python 与 shell 判定的键
RUNTIME_LIBRARY_ROOT_READS_UNLISTED = {
    "scripts/gate/lib_function_units_shell_text.py::shell_assignment_value":
        "读 cross_run_cache_registry 点名的几份 shell 脚本（lkmm.sh、fetch-deps.sh、test-environment.sh）里算缓存位置的那一行，只给命令行 cross-run-caches 与自证，不进键",
    ".claude/gate.d/test-environment.sh": "同上：test_environment_stage_run_times_file 读它取上一趟各阶段用时放哪，只给 cross-run-caches 与自证",
    ".claude/hooks/lib_work_root.py": "代码、不是数据：work_root_library 载入它取数据根，只给 cross-run-caches 与自证",
    "scripts/gate/admission.py": "代码、不是数据：judging_excluded_files 只在命令行 closure、key 带 --judging 时载入它，算键调用方（admission.py 自己、门禁）不经这一条",
    "scripts/gate/lib_function_units_unit_graph.py::unit_graph_directory":
        "Rust 产品行为视图编 unit-graph 要 crates/singlefs-unit-graph 的源码目录与 Cargo.lock（目录与源码、不是数据文件），拿不到抛 ProductViewUnavailable 由 rust_workspace 处置",
}
# 被 source 的库读别的文件里赋的变量、库里的 $0（算键库文件头「shell 里被 source 的库读别的文件里赋的变量」）
SHELL_SOURCED_FIXTURE = {
    "g/owned_entry.sh": ('#!/usr/bin/env bash\nSTAGE_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"\n'
                         'STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/.." && pwd -P)"\nsource "$STAGE_DIRECTORY/lib/cells.sh"\ncell_owned\n'),
    "g/started_entry.sh": '#!/usr/bin/env bash\nsource "$(dirname "$0")/lib/cells.sh"\ncell_started\n',
    "g/sibling_entry.sh": '#!/usr/bin/env bash\nsource "$(dirname "$0")/lib/assign.sh"\nsource "$(dirname "$0")/lib/cells.sh"\ncell_sibling\n',
    "g/lib/assign.sh": 'SIBLING_SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/tools/sibling.py"\n',
    "g/lib/cells.sh": ('cell_owned() {\n  python3 "$STAGE_REPOSITORY/tools/owned.py"\n}\n\n'
                       'cell_started() {\n  python3 "$(cd "$(dirname "$0")/.." && pwd)/tools/started.py"\n}\n\n'
                       'cell_sibling() {\n  python3 "$SIBLING_SCRIPT"\n}\n'),
    "tools/owned.py": "print(1)\n",
    "tools/started.py": "print(2)\n",
    "tools/sibling.py": "print(3)\n",
}
SHELL_SOURCED_OWNER_CELL = ("shell：被 source 的库里的函数读入口赋的 STAGE_REPOSITORY（值里有 ${BASH_SOURCE[0]}），照入口那一份折，"
                            "连上它起的脚本，改那份脚本键变")
SHELL_SOURCED_ZERO_CELL = "shell：被 source 的库里的 $0 是 source 它的入口，$(dirname \"$0\")/.. 折到入口往上一层，连上它起的脚本，改那份脚本键变"
SHELL_SOURCED_SIBLING_CELL = "shell：一份库读同一个入口 source 的另一份库赋的变量（SIBLING_SCRIPT），照那一份折，连上它起的脚本，改那份脚本键变"
# 拆出的模块 import 的名字照 import 它的入口插的 sys.path 解（算键库文件头「import 它的模块加的 sys.path」）
PYTHON_IMPORTER_FIXTURE = {
    "tool/entry.py": ("import os\nimport sys\n\nsys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), \"..\", \"shared\"))\n"
                      "from entry_part import judge  # noqa: E402\n\n\nif __name__ == \"__main__\":\n    sys.exit(judge())\n"),
    "tool/entry_part.py": "from helper import verdict\n\n\ndef judge():\n    return verdict()\n",
    "shared/helper.py": "def verdict():\n    return 0\n",
}
PYTHON_IMPORTER_SYS_PATH_CELL = ("python：拆出的模块自己不插 sys.path，它 import 的模块照同目录里 import 它的入口插的目录解，不拒算 import-unresolved，"
                                 "改那份模块键变")
# 模块级 from x import a, b：载入时只执行 x，绑定的名字有单元读到才连（算键库文件头「import 时执行的模块级语句」）
PYTHON_IMPORT_BINDING_FIXTURE = {
    "tool/user.py": "from helpers import used, unused\n\n\ndef judge():\n    return used()\n",
    "tool/helpers.py": "def used():\n    return 1\n\n\ndef unused():\n    return 2\n",
}
PYTHON_IMPORT_BINDING_UNUSED_CELL = "python：模块级 from helpers import used, unused 只因载入收进来时不连 unused，没人读它，改它不作废"
PYTHON_IMPORT_BINDING_USED_CELL = "python：模块级 from helpers import used, unused 里有单元读到的 used 照连，改它作废"
# 入口是脚本里的一个函数、库里的函数调同一个脚本 source 的另一份库里的函数（算键库文件头「shell 里被 source 的库读别的文件里赋的变量」，visible_roots）
SHELL_REACHED_FIXTURE = {
    "g/reach_entry.sh": ('#!/usr/bin/env bash\nsource "$(dirname "$0")/lib/reach_first.sh"\nsource "$(dirname "$0")/lib/reach_second.sh"\n'
                         'reach_main() {\n  reach_first\n}\nreach_main\n'),
    "g/lib/reach_first.sh": "reach_first() {\n  reach_second\n}\n",
    "g/lib/reach_second.sh": "reach_second() {\n  echo second\n}\n",
}
SHELL_REACHED_LIBRARY_CELL = ("shell：入口是脚本里的一个函数（source 它那几份库的脚本体不在闭包里），一份库里的函数调同一个脚本 source 的另一份库里的函数，"
                              "照叫到它的那一份看得见的文件找，连上，改那个函数键变")
# 拆出的模块里的函数（方法）的参数照 import 它的同目录模块里的调用处折（算键库文件头「拆出的模块里的函数，参数照 import 它的模块里的调用处折」）
PYTHON_IMPORTER_CALL_SITES_FIXTURE = {
    "tool/load.py": ('import os\n\nfrom load_helper import load_by_path\n\n\ndef run():\n'
                          '    module = load_by_path(os.path.join(os.path.dirname(os.path.abspath(__file__)), "load_target.py"))\n    return module.answer()\n'),
    "tool/load_helper.py": ('import importlib.util\n\n\ndef load_by_path(path):\n    specification = importlib.util.spec_from_file_location("loaded_target", path)\n'
                            '    module = importlib.util.module_from_spec(specification)\n    specification.loader.exec_module(module)\n    return module\n'),
    "tool/load_target.py": "def answer():\n    return 1\n",
}
PYTHON_IMPORTER_CALL_SITES_CELL = "python：拆出的模块里按参数载入的路径（spec_from_file_location(…, path)）照 import 它的模块里调它的那几处折，连上载入的那份，改它键变"
# 方法拿参数当载入的路径（spec_from_file_location）：路径折不出当场拒算，只有照拆分族里的调用处折才认得出。拿参数当起的脚本判不出这一条：
# 起子进程那一处折不出时收闭包照闭包里每一处调它的实参往上认（ScriptClosure.caller_launch_refusals），不经这一条也连得上
PYTHON_IMPORTER_METHOD_SITES_FIXTURE = {
    "tool/mixed.py": "from mixed_load import Loading\nfrom mixed_steps import Steps\n\n\nclass Runner(Loading, Steps):\n    pass\n\n\ndef run():\n    return Runner().go()\n",
    "tool/mixed_steps.py": ('import os\n\n\nclass Steps:\n    def go(self):\n'
                            '        return self.load(os.path.join(os.path.dirname(os.path.abspath(__file__)), "mixed_target.py")).answer()\n'),
    "tool/mixed_load.py": ('import importlib.util\n\n\nclass Loading:\n    def load(self, path):\n'
                           '        specification = importlib.util.spec_from_file_location("mixed_loaded", path)\n'
                           '        module = importlib.util.module_from_spec(specification)\n        specification.loader.exec_module(module)\n        return module\n'),
    "tool/mixed_target.py": "def answer():\n    return 1\n",
}
PYTHON_IMPORTER_METHOD_SITES_CELL = ("python：拆成混入类的方法的参数照拆分族里调它的那几处折（几个混入类由同一份模块 import 进来拼成一个类）：按参数载入的路径"
                                     "（spec_from_file_location(…, path)）连上载入的那份，改它键变")
# 拆分族外 import 同一份库的脚本交的实参不折进库里函数的参数（算键库文件头「import 它的模块加的 sys.path」拆分族那几句）：
# stage_launch.py 是 stage.py 拆出的部分，other.py 是同目录里另一份调它的脚本；收 stage.py::run 只连 stage.py 交的 first.sh
PYTHON_IMPORTER_OUTSIDE_FAMILY_FIXTURE = {
    "tool/stage.py": 'from stage_launch import start\n\n\ndef run():\n    return start("tool/first.sh")\n',
    "tool/stage_launch.py": 'import subprocess\n\n\ndef start(script):\n    return subprocess.run(["bash", script], check=False)\n',
    "tool/other.py": 'from stage_launch import start\n\n\ndef other():\n    return start("tool/second.sh")\n',
    "tool/first.sh": "#!/usr/bin/env bash\necho first\n",
    "tool/second.sh": "#!/usr/bin/env bash\necho second\n",
}
PYTHON_IMPORTER_OUTSIDE_FAMILY_CELL = ("python：库里函数的参数不照拆分族外 import 它的模块里的调用处折（同目录别的脚本交的那份程序不连进经这份库的闭包），"
                                       "改那份程序键不变")
# 按部分文件 exec（算键库文件头「按部分文件 exec」）：parted.py 在模块级逐份读部分文件、compile(<读出的源码>, <路径>, "exec")、再逐个 exec 进自己的命名空间；
# checked.py 在模块级只 compile 一份文件（核语法）、不 exec
COMPILED_PART_EXEC_FIXTURE = {
    "tool/parted.py": ('import os\n\nPART_NAMES = ("parted_first.py",)\nPART_DIRECTORY = os.path.dirname(os.path.abspath(__file__))\n\n'
                       'part_codes = []\nfor part_name in PART_NAMES:\n    part_path = os.path.join(PART_DIRECTORY, part_name)\n'
                       '    with open(part_path, encoding="utf-8") as part_handle:\n        part_codes.append(compile(part_handle.read(), part_path, "exec"))\n'
                       'for part_code in part_codes:\n    exec(part_code)\n\nif __name__ == "__main__":\n    main()\n'),
    "tool/parted_first.py": "def main():\n    return 1\n",
    "tool/checked.py": ('import os\n\nCHECKED_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "parted_first.py")\n'
                        'with open(CHECKED_PATH, encoding="utf-8") as checked_handle:\n    compile(checked_handle.read(), CHECKED_PATH, "exec")\n'),
}
COMPILED_PART_EXEC_CELL = ("python：模块级把读出的部分文件 compile(<源码>, <路径>, \"exec\") 再 exec 进自己的命名空间：exec 进来的是哪份文件的哪些名字折不出，"
                           "拒算 compiled-part-exec")
COMPILED_PART_SYNTAX_CHECK_CELL = "python：模块级只 compile 一份读出的文件（核语法）、不 exec 的不拒算"
# exec 进模块对象（算键库文件头「exec 进模块对象」）：module_exec_text.sh 的内联 python 把 shell 变量交进来的一段库正文 exec 进一个 types.ModuleType、
# module_exec_file.sh 的把按参数路径读出的 exec_part.py exec 进一个，再各按属性调；module_exec_unfolded.sh 交进来的正文是 "$1"（调用方给的，折不出）
MODULE_OBJECT_EXEC_FIXTURE = {
    "s/module_exec_text.sh": ('#!/usr/bin/env bash\nLIBRARY_TEXT="$(cat <<\'PY_LIBRARY\'\nimport subprocess\n\n\ndef run_helper():\n'
                              '    return subprocess.run(["bash", "s/exec_helper.sh"], check=False)\nPY_LIBRARY\n)"\n'
                              'python3 - "$LIBRARY_TEXT" <<\'PY\'\nimport sys, types\n'
                              'text_library = types.ModuleType("lib_text_library")\n'
                              'exec(compile(sys.argv[1], "text-library", "exec"), text_library.__dict__)\n'
                              'text_library.run_helper()\nPY\n'),
    "s/module_exec_file.sh": ('#!/usr/bin/env bash\npython3 - "s/exec_part.py" <<\'PY\'\nimport sys, types\n'
                              'file_library = types.ModuleType("lib_file_library")\n'
                              'exec(compile(open(sys.argv[1], encoding="utf-8").read(), sys.argv[1], "exec"), file_library.__dict__)\n'
                              'file_library.used()\nPY\n'),
    "s/exec_part.py": ('import subprocess\n\n\ndef used():\n    return subprocess.run(["bash", "s/exec_file_helper.sh"], check=False)\n\n\n'
                       'def unused():\n    return subprocess.run(["bash", "s/exec_unused.sh"], check=False)\n'),
    "s/exec_helper.sh": "#!/usr/bin/env bash\necho helper\n",
    "s/exec_file_helper.sh": "#!/usr/bin/env bash\necho file helper\n",
    "s/exec_unused.sh": "#!/usr/bin/env bash\necho unused\n",
    "s/module_exec_unfolded.sh": ('#!/usr/bin/env bash\npython3 - "$1" <<\'PY\'\nimport sys, types\n'
                                  'positional_library = types.ModuleType("lib_positional_library")\n'
                                  'exec(compile(sys.argv[1], "positional-library", "exec"), positional_library.__dict__)\n'
                                  'positional_library.run_helper()\nPY\n'),
}
MODULE_OBJECT_EXEC_TEXT_CELL = ("python：exec(compile(<shell 变量交进来的库正文>, <标签>, \"exec\"), <模块对象>.__dict__) 照载入那段正文连："
                                "按属性调到的函数起的脚本连上，改它键变")
MODULE_OBJECT_EXEC_FILE_CELL = ("python：exec(compile(open(<折得出的路径>).read(), <路径>, \"exec\"), <模块对象>.__dict__) 照载入那份文件连："
                                "按属性调到的函数起的脚本连上，改它键变")
MODULE_OBJECT_EXEC_UNCALLED_CELL = "python：exec 进模块对象的那份文件里没按属性调到的函数不连：它起的脚本改了键不变"
MODULE_OBJECT_EXEC_UNFOLDED_CELL = "python：exec 进模块对象的源码折不出（调用方交的 \"$1\"）拒算 compiled-part-exec"
# exec 进模块对象的库正文的几种来路（算键库文件头「起代码的写法逐种认」$(函数) 与 read 那两条、「exec 进模块对象」）：
# printed_library.sh 交 "$(不带参数、printf '%s\n' 打两个变量的函数)"，printed_unfolded.sh 的函数打的一个变量没赋过；
# read_library.sh 的库正文由 IFS= read -r -d '' 从 here-doc 读进变量、经环境变量交进去，read_unfolded.sh 的从 "$1" 那份文件读；
# loader_sites.sh 的库正文里 load(path) 按参数路径载入，路径是载入方 <模块对象>.load(…) 那一处给的，describe(path) 只拿路径拼字，
# loader_sites_unfolded.sh 给的是调用方交的 "$1"；located_entry.sh 交的库正文写在它 source 的 lib/located_library.sh 里（read 读进变量），正文里有一处 globals().get
MODULE_OBJECT_TEXT_SOURCES_FIXTURE = {
    "s/printed_library.sh": ('#!/usr/bin/env bash\nIMPORTS_LIBRARY="$(cat <<\'PY_IMPORTS\'\nimport subprocess\nPY_IMPORTS\n)"\n'
                             'FUNCTIONS_LIBRARY="$(cat <<\'PY_FUNCTIONS\'\n\n\ndef used():\n    return subprocess.run(["bash", "s/printed_used.sh"], check=False)\n\n\n'
                             'def unused():\n    return subprocess.run(["bash", "s/printed_unused.sh"], check=False)\nPY_FUNCTIONS\n)"\n'
                             'printed_library_code() { printf \'%s\\n\' "$IMPORTS_LIBRARY" "$FUNCTIONS_LIBRARY"; }\n'
                             'python3 - "$(printed_library_code)" <<\'PY\'\nimport sys, types\n'
                             'printed_library = types.ModuleType("lib_printed_library")\n'
                             'exec(compile(sys.argv[1], "printed-library", "exec"), printed_library.__dict__)\n'
                             'printed_library.used()\nPY\n'),
    "s/printed_unfolded.sh": ('#!/usr/bin/env bash\nIMPORTS_LIBRARY="$(cat <<\'PY_IMPORTS\'\nimport subprocess\nPY_IMPORTS\n)"\n'
                              'unfolded_library_code() { printf \'%s\\n\' "$IMPORTS_LIBRARY" "$CALLER_LIBRARY"; }\n'
                              'python3 - "$(unfolded_library_code)" <<\'PY\'\nimport sys, types\n'
                              'unfolded_library = types.ModuleType("lib_unfolded_library")\n'
                              'exec(compile(sys.argv[1], "unfolded-library", "exec"), unfolded_library.__dict__)\n'
                              'unfolded_library.used()\nPY\n'),
    "s/printed_used.sh": "#!/usr/bin/env bash\necho printed used\n",
    "s/printed_unused.sh": "#!/usr/bin/env bash\necho printed unused\n",
    "s/read_library.sh": ('#!/usr/bin/env bash\nIFS= read -r -d \'\' READ_LIBRARY <<\'PY_READ\' || true\nimport subprocess\n\n\n'
                          'def used():\n    return subprocess.run(["bash", "s/read_used.sh"], check=False)\n\n\n'
                          'def unused():\n    return subprocess.run(["bash", "s/read_unused.sh"], check=False)\nPY_READ\n'
                          'READ_LIBRARY="$READ_LIBRARY" python3 - <<\'PY\'\nimport os, types\n'
                          'read_library = types.ModuleType("lib_read_library")\n'
                          'exec(compile(os.environ["READ_LIBRARY"], "read-library", "exec"), read_library.__dict__)\n'
                          'read_library.used()\nPY\n'),
    "s/read_unfolded.sh": ('#!/usr/bin/env bash\nIFS= read -r -d \'\' FILE_LIBRARY < "$1" || true\n'
                           'FILE_LIBRARY="$FILE_LIBRARY" python3 - <<\'PY\'\nimport os, types\n'
                           'file_library = types.ModuleType("lib_read_file_library")\n'
                           'exec(compile(os.environ["FILE_LIBRARY"], "read-file-library", "exec"), file_library.__dict__)\n'
                           'file_library.used()\nPY\n'),
    "s/read_used.sh": "#!/usr/bin/env bash\necho read used\n",
    "s/read_unused.sh": "#!/usr/bin/env bash\necho read unused\n",
    "s/loader_sites.sh": ('#!/usr/bin/env bash\nSITES_LIBRARY="$(cat <<\'PY_SITES\'\nimport importlib.util\n\n\n'
                          'def load(path):\n    specification = importlib.util.spec_from_file_location("loaded_by_site", path)\n'
                          '    module = importlib.util.module_from_spec(specification)\n    specification.loader.exec_module(module)\n    return module\n\n\n'
                          'def describe(path):\n    return "described " + path\nPY_SITES\n)"\n'
                          'python3 - "$SITES_LIBRARY" "s/site_loaded.py" "s/site_described.py" <<\'PY\'\nimport sys, types\n'
                          'sites_library = types.ModuleType("lib_sites_library")\n'
                          'exec(compile(sys.argv[1], "sites-library", "exec"), sites_library.__dict__)\n'
                          'loaded = sites_library.load(sys.argv[2])\nprint(sites_library.describe(sys.argv[3]), loaded.VALUE)\nPY\n'),
    "s/loader_sites_unfolded.sh": ('#!/usr/bin/env bash\nSITES_LIBRARY="$(cat <<\'PY_SITES\'\nimport importlib.util\n\n\n'
                                   'def load(path):\n    specification = importlib.util.spec_from_file_location("loaded_by_site", path)\n'
                                   '    module = importlib.util.module_from_spec(specification)\n    specification.loader.exec_module(module)\n    return module\nPY_SITES\n)"\n'
                                   'python3 - "$SITES_LIBRARY" "$1" <<\'PY\'\nimport sys, types\n'
                                   'sites_library = types.ModuleType("lib_sites_library")\n'
                                   'exec(compile(sys.argv[1], "sites-library", "exec"), sites_library.__dict__)\n'
                                   'print(sites_library.load(sys.argv[2]).VALUE)\nPY\n'),
    "s/site_loaded.py": "VALUE = 1\n",
    "s/site_described.py": "VALUE = 3\n",
    "s/located_entry.sh": ('#!/usr/bin/env bash\nsource "$(dirname "$0")/lib/located_library.sh"\n'
                           'python3 - "$LOCATED_LIBRARY" <<\'PY\'\nimport sys, types\n'
                           'located_library = types.ModuleType("lib_located_library")\n'
                           'exec(compile(sys.argv[1], "located-library", "exec"), located_library.__dict__)\n'
                           'located_library.lookup("sample")\nPY\n'),
    "s/lib/located_library.sh": ('IFS= read -r -d \'\' LOCATED_LIBRARY <<\'PY_LOCATED\' || true\n\n\ndef lookup(name):\n'
                                 '    return globals().get(name + "_handler")\nPY_LOCATED\n'),
}
MODULE_OBJECT_PRINTED_TEXT_CELL = ("shell：内联 python 的参数是 \"$(不带参数的函数)\"、函数体只有一条 printf '%s\\n' 打两个变量：两个变量的正文逐行连起来当库正文"
                                   " exec 进模块对象，按属性调到的函数起的脚本连上，改它键变")
MODULE_OBJECT_PRINTED_UNCALLED_CELL = "shell：printf 打出的库正文里没按属性调到的函数不连：它起的脚本改了键不变"
MODULE_OBJECT_PRINTED_UNFOLDED_CELL = "shell：printf 打的变量有一个没赋过（折不出），库正文照旧折不出，拒算 compiled-part-exec"
MODULE_OBJECT_READ_TEXT_CELL = ("shell：IFS= read -r -d '' <变量> <<词 从 here-doc 读进变量、经环境变量交进内联 python exec 进模块对象：照 here-doc 的内容折，"
                                "按属性调到的函数起的脚本连上，改它键变")
MODULE_OBJECT_READ_UNCALLED_CELL = "shell：read -d '' 读进的库正文里没按属性调到的函数不连：它起的脚本改了键不变"
MODULE_OBJECT_READ_UNFOLDED_CELL = "shell：read -d '' 从 \"$1\" 那份文件读进变量的不认（折不出），拒算 compiled-part-exec"
MODULE_OBJECT_LOADER_SITES_CELL = ("python：exec 进模块对象的库正文里模块级函数 load(path) 按参数路径 spec_from_file_location 载入，路径照载入方 <模块对象>.load(…) 那一处折："
                                   "载入的那一份连上，改它键变")
MODULE_OBJECT_LOADER_SITES_OTHER_CELL = "python：载入方交给库正文里另一个函数（describe，不载入）的路径不当 load 的参数：那份文件改了键不变"
MODULE_OBJECT_LOADER_SITES_UNFOLDED_CELL = "python：载入方 <模块对象>.load(…) 交的是调用方给的 \"$1\"：折不出，照旧拒算 spec-path-unfolded"
MODULE_OBJECT_REFUSAL_LINES_CELL = ("python：exec 进模块对象的库正文写在被 source 的库里（read -d '' 读进变量、经参数交进来）：正文里的拒算落在那份库里正文那一行，"
                                    "算在那份文件上")
# 内联 python 的正文写在入口里、由被 source 的库起（算键库文件头「内联正文出自别的文件」）
INLINE_SOURCED_BODY_FIXTURE = {
    "s/sourced_body.sh": ('#!/usr/bin/env bash\nLOOKUP_PROGRAM="$(cat <<\'PY_LOOKUP\'\nimport sys\n\n\ndef lookup(name):\n'
                          '    return globals().get(name + "_handler")\n\n\nlookup(sys.argv[1])\nPY_LOOKUP\n)"\n'
                          'source "$(dirname "$0")/lib/sourced_body_run.sh"\nrun_lookup "$1"\n'),
    "s/lib/sourced_body_run.sh": 'run_lookup() {\n  python3 -c "$LOOKUP_PROGRAM" "$1"\n}\n',
}
INLINE_SOURCED_BODY_CELL = "内联 python 的正文写在入口里（装进变量）、由被 source 的库起：拒算落在入口里正文那一行，按文件查的登记表照入口查"
# 先给变量赋字面空串、再 read 从 "$1" 那份文件读进它（算键库文件头「起代码的写法逐种认」read 那一条）：赋值点里有 read 记的那一处，折不出
MODULE_OBJECT_READ_AFTER_LITERAL_FIXTURE = {
    "s/read_after_literal.sh": ('#!/usr/bin/env bash\nFILE_LIBRARY=""\nIFS= read -r -d \'\' FILE_LIBRARY < "$1" || true\n'
                                'FILE_LIBRARY="$FILE_LIBRARY" python3 - <<\'PY\'\nimport os, types\n'
                                'file_library = types.ModuleType("lib_read_file_library")\n'
                                'exec(compile(os.environ["FILE_LIBRARY"], "read-file-library", "exec"), file_library.__dict__)\n'
                                'file_library.used()\nPY\n'),
}
MODULE_OBJECT_READ_AFTER_LITERAL_CELL = ("shell：先给变量赋字面空串、再 IFS= read -r -d '' 从 \"$1\" 那份文件读进它、经环境变量交进内联 python exec 进模块对象："
                                         "read 改写的值折不出，拒算 compiled-part-exec")
# 按（文件, 函数）登记认限定名（算键库文件头「不拒算的程序词」）：同一份文件里两个类各有一个 run(self, command) 起调用方交的命令，只登 TripLauncher.run
PROGRAM_WORDS_QUALIFIED_FIXTURE = {
    "tool/launchers.py": ('import os\nimport subprocess\n\n\nclass TripLauncher:\n    def run(self, command):\n        return subprocess.run(command, check=False)\n\n\n'
                          'class OtherLauncher:\n    def run(self, command):\n        return subprocess.run(command, check=False)\n\n\n'
                          'def trip():\n    return TripLauncher().run(os.environ["TRIP_COMMAND"].split())\n\n\n'
                          'def other():\n    return OtherLauncher().run(os.environ["OTHER_COMMAND"].split())\n'),
}
PROGRAM_WORDS_QUALIFIED_ENTRIES = ("tool/launchers.py::trip", "tool/launchers.py::other")
PROGRAM_WORDS_QUALIFIED_ROWS = {("tool/launchers.py", "TripLauncher.run", "command"): "夹具"}
PROGRAM_WORDS_QUALIFIED_CELL = "不拒算的程序词按限定名认（夹具）：登了 TripLauncher.run 的词不拒算，同一份文件里 OtherLauncher.run 起折不出的命令照拒算"
# 入口所在的那份也登成测试服务代码（算键库文件头「测试服务代码不进键」）：lib.rs 里两个服务代码自己的入口闭包相同，同一份里另有一个测试函数当入口；
# lib.rs 与 judge_on_the_card.rs 临时登记
TEST_SERVICE_CODE_ENTRY_FIXTURE = {
    "crates/fservice/Cargo.toml": '[package]\nname = "fservice"\n',
    "crates/fservice/src/lib.rs": ("mod judge_on_the_card;\nmod product;\n\npub fn entry_point() -> u32 {\n    judge_on_the_card::judged() + product::value()\n}\n\n"
                                   "pub fn other_point() -> u32 {\n    judge_on_the_card::judged() + product::value()\n}\n\n"
                                   "#[cfg(test)]\nmod tests {\n    #[test]\n    fn entry_point_sums_the_judged_and_the_product_values() {\n"
                                   "        assert_eq!(super::entry_point(), 32);\n    }\n}\n"),
    "crates/fservice/src/judge_on_the_card.rs": "pub fn judged() -> u32 {\n    11\n}\n",
    "crates/fservice/src/product.rs": "pub fn value() -> u32 {\n    21\n}\n",
}
TEST_SERVICE_CODE_ENTRY_FIXTURE_ENTRIES = ("crates/fservice/src/lib.rs::entry_point", "crates/fservice/src/lib.rs::other_point")
# 同一份里的测试函数当入口：改它的正文键要变（测试照进键）
TEST_SERVICE_CODE_ENTRY_FIXTURE_TEST_ENTRY = "crates/fservice/src/lib.rs::tests::entry_point_sums_the_judged_and_the_product_values"
TEST_SERVICE_CODE_ENTRY_FIXTURE_TEST_EDIT = ("crates/fservice/src/lib.rs", "assert_eq!(super::entry_point(), 32);", "assert_eq!(32, super::entry_point());")
TEST_SERVICE_CODE_ENTRY_FIXTURE_ROWS = {"crates/fservice/src/lib.rs": "夹具：只在 TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL 跑的那一会儿登记",
                                       "crates/fservice/src/judge_on_the_card.rs": "夹具：只在 TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL 跑的那一会儿登记"}
# (改哪份, 原文, 改成, 入口 entry_point 的键期望变没变)：逐个累加
TEST_SERVICE_CODE_ENTRY_FIXTURE_EDITS = (
    ("crates/fservice/src/lib.rs", "pub fn entry_point() -> u32 {\n    judge_on_the_card::judged() + product::value()\n}",
     "pub fn entry_point() -> u32 {\n    product::value() + judge_on_the_card::judged()\n}", "same"),
    ("crates/fservice/src/product.rs", "    21\n", "    22\n", "changed"),
)
TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL = ("键：入口自己落在测试服务代码里：服务代码自己的入口 entry 行只记是哪一个，改它的正文键不变、改产品代码键变，同一份里另一个入口的键不同；"
                                       "同一份里的测试函数当入口照记指纹，改它的正文键变")
# 按字面尾巴读（算键库文件头「运行期读的数据文件」一）：os.path.join 拼的尾巴按整段路径比，+ 拼的照字符串后缀比；data/remove_stub.tsv 以 b.tsv 收尾
PATH_TAILS_FIXTURE = {
    "pkg/stub_reader.py": ('import os\nimport sys\n\n\ndef read_named(directory):\n    with open(os.path.join(directory, "b.tsv"), encoding="utf-8") as handle:\n'
                           '        return handle.read()\n\n\ndef read_concatenated(directory):\n    with open(directory + "b.tsv", encoding="utf-8") as handle:\n'
                           '        return handle.read()\n\n\ndef joined():\n    return read_named(sys.argv[1])\n\n\ndef concatenated():\n'
                           '    return read_concatenated(sys.argv[1])\n'),
    "data/b.tsv": "b\n",
    "data/remove_stub.tsv": "stub\n",
}
PATH_TAILS_JOINED_CELL = 'python：读 os.path.join(<折不出的目录>, "b.tsv") 按整段路径比：同目录的 remove_stub.tsv 不收，改它键不变'
PATH_TAILS_JOINED_KEPT_CELL = 'python：读 os.path.join(<折不出的目录>, "b.tsv")：仓里的 data/b.tsv 照收，改它键变'
PATH_TAILS_CONCATENATED_CELL = 'python：读 <折不出的目录> + "b.tsv" 照字符串后缀比：data/remove_stub.tsv 照收，改它键变'

BREAK_CELLS = (
    ("rust-open-entry-traits-ignored", ("开放入口：参数是 dyn", "R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl")),
    ("rust-dyn-traits-open-every-implementation", ("dyn 分派：闭包里的 dyn 不连没被点名的另一个 impl，改它不作废",)),
    ("rust-constants-dropped", ("关联常量（经泛型）", "函数指针表", "数组长度里的常量")),
    ("rust-macros-dropped", ("声明宏展开出的函数", "带路径的条目级宏调用定义的枚举：改了作废")),
    ("rust-trait-impl-edges-dropped", ("运算符重载", "Drop 隐式调用", "关联常量（经泛型）")),
    ("rust-verus-segments-ignored", ("verus! 里 ensures 带花括号：下一个函数照样切开，改它作废", "verus! 里够不到的函数改了不作废")),
    ("rust-cfg-ignored", ("cfg(feature) 关着的单元改了不作废", "认不出的 cfg 谓词拒算")),
    ("rust-manifest-directory-reads-unrefused", (MANIFEST_ROOT_CELL,)),
    ("rust-manifest-directories-refused", (MANIFEST_DIRECTORY_CELL,)),
    ("rust-manifest-binding-refused-whole", (MANIFEST_BINDING_CELL,)),
    ("shell-data-sources-unfollowed", (SHELL_DATA_SOURCE_CELL, SHELL_DATA_DOT_CELL, SHELL_DATA_NESTED_CELL, SHELL_DATA_UNFOLDED_CELL)),
    ("shell-data-sources-unsited", (SHELL_DATA_SOURCE_CELL,)),
    ("shell-data-sources-nested-unfollowed", (SHELL_DATA_NESTED_CELL,)),
    ("shell-dot-source-unrecognized", (SHELL_DATA_DOT_CELL,)),
    ("shell-data-source-refusals-dropped", (SHELL_DATA_UNFOLDED_CELL,)),
    ("rust-current-exe-refused", (CURRENT_EXE_CELL,)),
    ("rust-current-exe-children-unlinked", (CURRENT_EXE_CELL,)),
    ("rust-current-exe-constants-same-file-only", (CURRENT_EXE_CONSTANT_CELL,)),
    ("rust-current-exe-path-only-refused", (CURRENT_EXE_PATH_ONLY_CELL,)),
    ("rust-current-exe-spawn-unrefused", (CURRENT_EXE_SPAWN_CELL,)),
    ("closure-directory-lines-span-files", (DIRECTORY_CLOSURE_LINE_CELL,)),
    ("rust-lex-cache-unverified", (RUST_LEX_CACHE_CELL,)),
    ("rust-doc-comments-kept", ("只改文档注释不作废", "改文档注释里的代码块：入口 fn 不作废")),
    ("wgsl-raw-text", ("WGSL 只改注释不作废", "WGSL 够不到的函数改了不作废")),
    ("python-comments-kept", ("python：只改注释不作废",)),
    ("python-docstrings-kept", ("python：改文档串不作废",)),
    ("python-docstring-readers-ignored", ("python：有单元读 __doc__ 时改文档串作废",)),
    ("python-getattr-allowed", ("python：getattr 变量名拒算",)),
    ("python-globals-subscripts-refused", ("python：globals()[固定名字] 读连到那个名字的单元，不拒算（改它键变）", "python：globals()[折不出的名字] 读连本模块每个名字，不拒算（改其中一个键变）", "python：globals()[键] = 值、del 与 globals().update(…) 只写不读：不连、不拒算")),
    ("python-spec-paths-ignored", ("python：spec_from_file_location 字面路径折叠出跨模块边",)),
    ("python-attributes-unnarrowed", ("python：obj.方法 按闭包里出现过的类收窄",)),
    ("loaded-attributes-closure-wide", (LOADED_ATTRIBUTES_CELL,)),
    ("shell-heredoc-split", ("shell：here-doc 正文里有一行 `};` 不许被切开",)),
    ("shell-names-command-position-only", ("shell：函数名当参数传照样连上",)),
    ("shell-unfolded-paths-ignored", ("shell：`bash \"$x\"` 折不出拒算",)),
    ("shell-raw-text", ("shell：脚本体只改注释不作废",)),
    ("external-programs-refused", ("键：闭包点名了 cargo 不拒算，外部程序的版本不进键：不给 externals 也算得出键", LAUNCH_ARGUMENT_CELL)),
    ("key-entry-units-dropped", (ENTRY_UNIT_KEY_CELL,)),
    ("key-entry-doctest-owner-chained", ("R-结构：文档测试不连所挂的条目与那个模块的 use",)),
    ("unit-graph-key-entry-lines-dropped", (ENTRY_LINES_SHAPE_CELL, ENTRY_LINES_RUST_KEYS_CELL)),
    ("rust-doctest-entry-alone", (ENTRY_LINES_SHAPE_CELL, ENTRY_LINES_RUST_KEYS_CELL)),
    ("cross-run-cache-under-tmpdir", (CROSS_RUN_CACHE_CELL,)),
    ("cross-run-cache-git-directory-only", (CROSS_RUN_CACHE_CELL,)),
    ("cross-run-cache-run-times-old-spelling", (CROSS_RUN_CACHE_CELL,)),
    ("cross-run-cache-mentions-unscanned", (CROSS_RUN_CACHE_SCAN_CELL,)),
    ("rust-format-captures-ignored", ("format! 串里内联捕获的常量改了作废", "WGSL 够得到的函数改了作废")),
    ("shell-variable-first-line-only", ("shell：顶层变量里 here-doc 的正文改了，读它的函数作废",)),
    ("tree-copy-tails-ignored", (TREE_COPY_EXACT_CELL, "python：os.path.join(<折不出的目录>, <仓里的相对路径>) 连到仓里那一份")),
    ("git-repository-paths-unfolded", ("shell：git rev-parse --git-common-dir 折成根，经它 source 的库连上",
                                       "python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算",
                                       "preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算")),
    ("python-call-returns-unfolded", ("python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算",
                                      "preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算")),
    ("shell-redirect-targets-loaded", ("shell：重定向到变量路径不拒算",)),
    ("shell-path-arguments-loaded", ("shell：把路径当参数传给自己的函数不拒算",)),
    ("python-command-arguments-loaded", ("python：写数据的路径与传给 subprocess 的参数不拒算",)),
    ("shell-generated-scripts-ignored", ("shell：自己用 cat <<词 写出来再起的脚本不拒算、正文里装的连上",)),
    ("shell-inline-positionals-ignored", ("shell：bash -c '…\"$1\"…' <名字> <路径> 里的 $1 换成那个路径",)),
    ("shell-inline-positionals-escapes-ignored", (NESTED_POSITIONALS_CELL,)),
    ("shell-copied-scripts-ignored", ("shell：自己 cp 出来再起的脚本连到源那一份",)),
    ("shell-enclosing-locals-ignored", ("shell：套在外层函数里的函数看得见外层的 local",)),
    ("shell-cd-first-directory-only", ("shell：cd 过去的目录折出几个，相对路径在每一个底下都连",)),
    ("shell-pwd-unfolded", (SHELL_PWD_EXACT_CELL,)),
    ("shell-function-bounds-quote-blind", (SHELL_QUOTED_BRACE_CELL,)),
    ("shell-script-tail-groups-ignored", (SHELL_TAIL_GROUP_CELL, SHELL_TAIL_OUTSIDE_CELL)),
    ("shell-program-arguments-unnarrowed", (SHELL_ARGUMENTS_SKIPPED_CELL, SHELL_ARGUMENTS_OTHER_CASE_CELL)),
    ("shell-entry-arguments-unnarrowed", (SHELL_ENTRY_ARGUMENTS_CELL, SHELL_ENTRY_TABLE_CELL, SHELL_ENTRY_EXIT_TABLE_CELL)),
    ("shell-entry-undecided-branches-dropped", (SHELL_ENTRY_UNDECIDED_CELL,)),
    ("shell-preflight-arguments-moved", (SHELL_ARGUMENTS_PREFLIGHT_CELL,)),
    ("shell-exit-branches-followed", (SHELL_ARGUMENTS_EXIT_CELL, SHELL_ENTRY_EXIT_TABLE_CELL)),
    ("shell-table-rows-unselected", (SHELL_TABLE_UNSELECTED_CELL, SHELL_ENTRY_TABLE_CELL)),
    ("shell-table-row-items-unmatched", (SHELL_TABLE_ITEM_CELL,)),
    ("shell-table-launch-without-assignment-counted", (SHELL_TABLE_SELFTEST_ENTRY_CELL,)),
    ("shell-tree-copy-launch-words-dropped", (SHELL_TABLE_TREE_COPY_CELL,)),
    ("shell-row-driver-mentions-linked", (SHELL_TABLE_COPY_CELL,)),
    ("shell-row-driver-plain-words-dropped", (SHELL_TABLE_COPY_CALLED_CELL, SHELL_TABLE_COPY_PLAIN_WORD_CELL)),
    ("shell-row-driver-whole-launch-dropped", (SHELL_TABLE_COPY_WHOLE_CELL, SHELL_TABLE_MIXED_CELL, SHELL_TABLE_ENTRY_CELL, SHELL_ENTRY_TABLE_CELL)),
    ("shell-row-driver-selected-rows-dropped", (SHELL_TABLE_COPY_REAL_ROW_CELL,)),
    ("python-shell-launch-arguments-unfolded", (PYTHON_SHELL_ARGUMENTS_CELL,)),
    ("shell-pattern-removals-unfolded", (SHELL_PATTERN_REMOVAL_CELL,)),
    ("shell-directory-copies-ignored", (SHELL_DIRECTORY_COPY_CELL,)),
    ("shell-relative-tree-copies-ignored", (SHELL_RELATIVE_COPY_CELL,)),
    ("shell-directory-wildcards-ignored", (SHELL_DIRECTORY_WILDCARD_CELL,)),
    ("shell-positional-scripts-refused", (SHELL_POSITIONAL_SCRIPT_CELL,)),
    ("shell-relative-directories-unrooted", (SHELL_RELATIVE_CD_CELL,)),
    ("python-dispatch-tables-ignored", ("python：分派表（字典字面量）里的函数改了作废",)),
    ("python-module-constants-unfolded", ("python：import 进来的模块常量（module.NAME）折进路径，插进 sys.path 的目录连上",)),
    ("python-module-sequence-elements-unfolded", (DATA_READ_MODULE_SEQUENCE_CELL, E171_CLOSURE_CELL)),
    ("fold-limit-reads-skipped", (FOLD_LIMIT_READ_REFUSED_CELL,)),
    ("fold-limit-launches-skipped", (FOLD_LIMIT_LAUNCH_REFUSED_CELL,)),
    ("fold-limit-shell-paths-skipped", (FOLD_LIMIT_SHELL_TAIL_CELL,)),
    ("fold-limit-unmatched-tails-refused", (FOLD_LIMIT_UNMATCHED_TAILS_CELL,)),
    ("shell-inline-quoted-words-unfolded", (SHELL_QUOTED_INLINE_CELL,)),
    ("shell-case-subjects-launched", (CASE_SUBJECT_CELL,)),
    ("shell-refusal-lines-by-substring", (REFUSAL_WORD_LINES_CELL,)),
    ("shell-refusal-lines-positional-unmatched", (REFUSAL_WORD_LINES_CELL,)),
    ("shell-prefix-assignments-unscoped", (PREFIX_SCOPED_CELL,)),
    ("shell-temporary-sites-folded", (TEMPORARY_SITE_CELL,)),
    ("unit-graph-failure-step-unnamed", (UNIT_GRAPH_FAILED_STEP_CELL,)),
    ("unit-graph-present-program-rebuilt", (UNIT_GRAPH_PRESENT_PROGRAM_CELL,)),
    ("unit-graph-directories-not-shared", (UNIT_GRAPH_SHARED_DIRECTORIES_CELL,)),
    ("python-getattr-stdlib-refused", ("python：getattr(<标准库模块>, 变量名) 不拒算（够不到仓里的代码）",)),
    ("python-subscript-targets-rebind", ("python：表[路径] = … 不当成重新绑定路径，spec 载入照样折得出",)),
    ("python-environment-defaults-unfolded", ("python：os.environ.get(…) or <缺省路径> 按缺省折，连到那份脚本",)),
    ("python-written-scripts-ignored", ("python：自己写出字面文本再起的脚本不拒算",)),
    ("python-loop-variables-unfolded", ("python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连",)),
    ("python-imports-first-directory-only", ("python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连",)),
    ("python-import-names-unfolded", ("python：importlib.import_module(<模块级常量>) 折出模块名",)),
    ("python-path-tables-unfolded", ("python：路径表（字典字面量）[键] 取遍表里的路径，spec 载入连上",)),
    ("python-comprehension-scope-leaks", ("python：推导式里的循环变量不算函数里的赋值，for 取遍元组拼出的脚本连上",)),
    ("python-directory-wildcards-ignored", ("python：os.path.join(<仓里的目录>, <折不出的文件名>) 起的脚本连到那个目录里每一份",)),
    ("inline-scripts-ignored", ("shell：python3 -c '<字面串>' 当内联脚本切，sys.argv 折成后面的词，连到它 import 的仓内模块",
                                "shell：python3 - <<'PY' 喂的 here-doc 当内联脚本切，连到它 import 的仓内模块",
                                "shell：python3 -c / bash -c 的串带折不出的展开拒算 inline-script-not-literal",
                                "shell：bash -c \"$串\" 的串折得出就切开再认，连到它起的脚本",
                                "python：[sys.executable, \"-c\", <字面串>, …] 当内联脚本切，连到它 import 的仓内模块",
                                "python：-c 的串折不出（出处是环境变量）拒算 inline-script-not-literal")),
    ("shell-later-interpreters-ignored", ("shell：一条命令里第二个解释器（bash 包装脚本 N python3 x.py）照第一个的规则认，连到 x.py",)),
    ("shell-substitutions-ignored", ("shell：起同目录脚本连边（写在命令替换里：x=\"$(bash 同目录/y.sh)\"）",
                                     "shell：< <(命令 \"$x\") 里的命令照命令认，前面那个 < 不吞掉它的第一个词")),
    ("external-program-words-unfiltered", ("shell：die 消息里出现 qemu-system-x86_64 不算外部程序",)),
    ("python-modules-ignored", ("shell：python3 -m <仓内模块> 连到那份模块的程序入口", "python：-m unittest <测试模块> 整份连上（测试方法改了作废）")),
    ("shell-unfolded-commands-allowed", ("shell：命令位置上折不出的程序拒算 shell-command-unfolded",)),
    ("shell-wrapped-commands-ignored", ("shell：包装命令（timeout 60）后面直接执行的脚本连上",)),
    ("shell-command-structure-ignored", ("shell：[[ … || … ]]、(( … ))、case 分支的模式、数组字面量里的词不当命令",
                                         "shell：{ python3 /dev/fd/3 …; } 3<<'PY'（重定向写在组后面）照内联脚本切")),
    ("shell-subscript-assignment-split", ("shell：名字[下标]=值、名字[下标]+=值（下标里带空格与展开）照赋值认、不当命令",)),
    ("shell-array-commands-unfolded", ("shell：\"${数组[@]}\" 当命令时按数组的字面赋值摊开，连到它起的脚本",)),
    ("shell-heredoc-captures-unfolded", ("shell：python3 -c \"$(函数体是一条 cat <<'词' … 词 的函数)\" 折出那份 here-doc 的正文当内联脚本",)),
    ("declared-checks-ignored", ("shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本",
                                 "python：上游 preflight 跑文件头登记的判据（bash -c declaration.command）不拒算，判据从声明它的脚本连")),
    ("inline-argv-commands-ignored", ("shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认",)),
    ("python-unfolded-heads-allowed", ("python：变量解释器折不出（出处是环境变量）拒算 subprocess-command-unfolded",)),
    ("python-commands-literal-only", ("python：命令是 [解释器] + 参数（按调用处给的列表折）连到那份脚本", "python：认不出的命令（对象的属性）拒算 subprocess-command-unfolded")),
    ("python-path-programs-refused", ("python：shutil.which(…) 与 os.path.join(<折不出的目录>, \"bin/x\") 起的根外程序不拒算、不当工具链询问",
                                      "python：内联脚本起 sys.argv[k]（取的是起它那份脚本的 \"$1\"）不拒算、不当工具链询问：程序是调用处给的词")),
    ("python-command-dictionaries-unfolded", ("python：命令里的 *选项[\"键\"] 折回调用处给的字典字面量，连到那份脚本",)),
    ("python-dispatch-calls-ignored", ("python：经分派表 表[键](…) 调到的函数，命令参数按那一处调用折",)),
    ("python-nested-scopes-unfolded", (FOLD_NESTED_CELL,)),
    ("python-command-concatenation-unfolded", (FOLD_CONCATENATION_CELL,)),
    ("python-command-slices-unfolded", (FOLD_SLICE_CELL,)),
    ("python-command-heads-split-ignored", (FOLD_VERSIONS_CELL,)),
    ("python-method-parameters-unfolded", (FOLD_VERSIONS_CELL,)),
    ("python-comprehension-variables-unfolded", (FOLD_VERSIONS_CELL,)),
    ("python-element-calls-unfolded", (FOLD_VERSIONS_CELL,)),
    ("python-sequence-indexes-unfolded", (FOLD_LOADED_CELL,)),
    ("python-loaded-module-constants-unfolded", (FOLD_LOADED_CELL,)),
    ("python-script-tail-groups-ignored", (FOLD_TAIL_GROUP_CELL, FOLD_OUTSIDE_SCRIPT_CELL, FOLD_CWD_GROUP_CELL)),
    ("python-submitted-functions-unfolded", (FOLD_SUBMITTED_CELL,)),
    ("python-environment-assignments-unfolded", (FOLD_ENVIRONMENT_CELL,)),
    ("python-data-reads-ignored", (DATA_READ_CHANGED_CELL, DATA_READ_TAIL_CELL, DATA_READ_REFUSED_CELL, DATA_READ_DIRECTORY_CELL, DATA_READ_LIBRARY_CELL,
                                   DATA_READ_ATTRIBUTE_CELL)),
    ("python-parsed-arguments-unread", (DATA_READ_PARSED_CELL,)),
    ("python-directory-reads-refused", (DATA_READ_DIRECTORY_CELL,)),
    ("python-unlisted-directory-reads-whole", (DATA_READ_UNLISTED_DIRECTORY_CELL,)),
    ("python-returned-listings-ignored", (DATA_READ_RETURNED_LISTING_CELL,)),
    ("python-glob-listings-ignored", (PATTERN_LISTING_GLOB_CELL,)),
    ("python-pattern-listing-transforms-accepted", (PATTERN_LISTING_GLOB_TRANSFORMED_CELL,)),
    ("shell-find-listings-ignored", (PATTERN_LISTING_FIND_CELL,)),
    ("shell-find-listing-operators-unchecked", (PATTERN_LISTING_FIND_OPERATORS_CELL,)),
    ("python-root-joins-refused", (DATA_READ_LISTING_CELL, DATA_READ_COPY_CELL)),
    ("python-parameter-callers-unresolved", (DATA_READ_LIBRARY_CELL,)),
    ("python-attribute-reads-unfolded", (DATA_READ_ATTRIBUTE_CELL,)),
    ("python-caller-launches-refused", (CALLER_LAUNCH_CELL,)),
    ("python-launch-arguments-unfolded", (LAUNCH_ARGUMENT_CELL, LAUNCH_ARGUMENT_SCRIPT_CELL)),
    ("python-tail-reads-unanchored", (TAIL_ANCHOR_SCRATCH_CELL, TAIL_ANCHOR_JUDGE_CELL)),
    ("python-tail-reads-samples-included", (TAIL_SAMPLE_CELL,)),
    ("python-call-sites-whole-view", (TAIL_VIEW_CELL,)),
    ("python-data-reads-follow-build-outputs", (BUILD_OUTPUT_CELL,)),
    ("built-tool-sources-ignored", (TOOL_SOURCE_CELL, BUILT_TOOL_LAUNCH_CELL)),
    ("built-tool-launches-refused", (BUILT_TOOL_LAUNCH_CELL,)),
    ("caller-supplied-commands-refused", (CALLER_SUPPLIED_REGISTERED_CELL,)),
    ("caller-supplied-commands-callee-wide", (CALLER_SUPPLIED_UNREGISTERED_CELL,)),
    ("python-call-sites-matched-by-name", (CALL_SITE_SHADOWED_CELL, CALL_SITE_ATTRIBUTE_CELL)),
    ("python-partial-call-sites-ignored", (CALL_SITE_PARTIAL_CELL,)),
    ("unit-graph-text-busy-not-waited", (UNIT_GRAPH_TEXT_BUSY_CELL,)),
    ("shell-nested-commands-in-outer", (NESTED_PROGRAM_WORDS_REGISTERED_CELL,)),
    ("inline-refusal-lines-at-launch", (INLINE_REFUSAL_LINE_CELL,)),
    ("excluded-files-followed", (EXCLUDED_FILE_DATA_CELL, EXCLUDED_FILE_PROGRAM_CELL, JUDGING_CLOSURE_CELL)),
    ("not-judging-files-drifted", (NOT_JUDGING_FILES_CELL,)),
    ("closure-judging-exclusions-dropped", (JUDGING_CLOSURE_CELL,)),
    ("shell-deferred-heredocs-unmoved", (DEFERRED_HEREDOC_BODY_CELL, DEFERRED_HEREDOC_THEN_CELL)),
    ("python-text-pieces-unfolded", (TEXT_PIECES_EXECUTABLE_CELL, TEXT_PIECES_INTEGER_CELL)),
    ("shell-open-tail-unnarrowed", (OPEN_TAIL_SKIPPED_CELL, OPEN_TAIL_PYTHON_CELL)),
    ("program-words-refused", (PROGRAM_WORDS_CELL,)),
    ("launched-programs-refused", (LAUNCHED_PROGRAMS_CELL, LAUNCHED_SHELL_CELL)),
    ("shell-refusal-lines-by-first-mention", (REFUSAL_LINES_CELL,)),
    ("shell-env-array-assignments-kept", (ENV_ARRAY_CELL,)),
    ("shell-process-substitution-redirects-dropped", (PROCESS_SUBSTITUTION_HEREDOC_CELL,)),
    ("shell-process-substitution-splits-command", (PROCESS_SUBSTITUTION_ARGUMENT_CELL,)),
    ("python-registered-inline-launches-analyzed", (REGISTERED_INLINE_LAUNCH_CELL,)),
    ("python-imported-sys-path-ignored", (IMPORTED_SYS_PATH_CELL,)),
    ("shell-registered-missing-paths-refused", (REGISTERED_MISSING_PATH_CELL,)),
    ("registered-words-drifted", (REGISTERED_WORDS_REAL_CELL,)),
    ("python-registry-command-texts-refused", (REGISTRY_COMMAND_TEXTS_CELL, REGISTRY_COMMAND_TABLE_CELL)),
    ("spec-path-arguments-refused", (SPEC_PATH_ARGUMENTS_CELL,)),
    ("spec-path-tuple-first-only", (SPEC_PATH_ARGUMENTS_CELL,)),
    ("tail-read-scopes-ignored", (TAIL_READ_SCOPES_CELL, TAIL_READ_SCOPES_REAL_CELL, TAIL_READS_RECORDS_REAL_CELL)),
    ("tail-reads-under-records-unrecorded", (TAIL_READS_RECORDS_REAL_CELL,)),
    ("tail-read-scopes-copies-unscoped", (TAIL_READ_SCOPES_CELL,)),
    ("tail-read-scopes-drifted", (TAIL_READ_SCOPES_REAL_CELL,)),
    ("tail-read-scopes-inline-unread", (TAIL_READ_SCOPES_REAL_CELL,)),
    ("tail-read-scopes-root-ignored", (TAIL_READ_ROOT_SCOPE_CELL,)),
    ("python-getattr-names-unfolded", (GETATTR_NAMES_CELL,)),
    ("shell-function-outputs-unfolded", (FUNCTION_PRINTED_WORD_CELL,)),
    ("shell-quoted-assignments-unread", (QUOTED_ASSIGNMENT_ENV_CELL, QUOTED_ASSIGNMENT_ARRAY_CELL)),
    ("python-local-names-resolved", (LOCAL_NAME_SHADOW_CELL,)),
    ("stage-cell-functions-drifted", (STAGE_CELL_FUNCTIONS_REAL_CELL,)),
    ("python-module-body-commands-unfolded", (MODULE_BODY_COMMAND_CELL,)),
    ("shell-exported-variables-unread", (EXPORTED_VARIABLE_CELL, PYTHON_SPLIT_PART_ENVIRONMENT_CELL)),
    ("python-guard-names-unfolded", (GUARD_NAME_PATH_CELL,)),
    ("shell-heredoc-empty-rows-spanned", (HEREDOC_BLANK_ROW_CELL,)),
    ("shell-narrowed-views-unshared", (NARROWED_VIEW_SHARED_CELL,)),
    ("rust-own-program-restart-refused", (OWN_PROGRAM_KEYED_CELL,)),
    ("rust-own-program-entry-unchecked", (OWN_PROGRAM_ENTRY_CELL,)),
    ("rust-own-program-path-unchecked", (OWN_PROGRAM_PATH_CELL,)),
    ("rust-own-program-uses-any-block", (OWN_PROGRAM_BLOCK_CELL,)),
    ("rust-own-program-divergence-any-block", (OWN_PROGRAM_DIVERGENCE_CELL,)),
    ("rust-extern-listed-calls-refused", (EXTERN_LISTED_CELL,)),
    ("rust-extern-any-block-keyed", (EXTERN_REFUSED_CELL,)),
    ("rust-extern-link-first-attribute-only", (EXTERN_LINK_ATTRIBUTE_CELL,)),
    ("rust-extern-other-block-link-unchecked", (EXTERN_OTHER_BLOCK_LINK_CELL, EXTERN_OTHER_TARGET_LINK_CELL, EXTERN_DEPENDENCY_LINK_CELL)),
    ("rust-extern-build-script-link-unchecked", (EXTERN_BUILD_SCRIPT_LINK_CELL, EXTERN_DEPENDENCY_LINK_CELL, EXTERN_LINK_ARGUMENT_CELL)),
    ("rust-workspace-members-glob-unescaped", (WORKSPACE_BRACKET_CELL,)),
    ("rust-workspace-dot-member-unnormalized", (WORKSPACE_DOT_MEMBER_CELL,)),
    ("rust-same-named-member-directories-merged", (WORKSPACE_SAME_NAME_CELL,)),
    ("rust-workspace-multiline-backslash-read", (WORKSPACE_MULTILINE_CELL,)),
    ("rust-extern-link-of-other-targets-unchecked", (EXTERN_OTHER_TARGET_LINK_CELL,)),
    ("rust-extern-link-of-dependencies-unchecked", (EXTERN_DEPENDENCY_LINK_CELL, EXTERN_LINK_ARGUMENT_CELL)),
    ("rust-extern-link-arguments-unchecked", (EXTERN_LINK_ARGUMENT_CELL,)),
    ("rust-extern-two-colon-instructions-ignored", (EXTERN_LINK_ARGUMENT_CELL,)),
    ("rust-extern-dependency-dev-dependencies-unfollowed", (EXTERN_DEPENDENCY_LINK_CELL,)),
    ("rust-extern-instruction-words-bare", (EXTERN_INSTRUCTION_LITERAL_CELL,)),
    ("rust-extern-library-literals-of-any-package", (EXTERN_INSTRUCTION_LITERAL_CELL,)),
    ("rust-extern-external-dependencies-unchecked", (EXTERN_EXTERNAL_DEPENDENCY_CELL,)),
    ("rust-extern-system-library-list-short", (EXTERN_SYSTEM_LIBRARY_LIST_CELL,)),
    ("shell-nested-functions-by-any-word", (SHELL_NESTED_WORD_CELL,)),
    ("shell-python-launch-arguments-unfolded", (SHELL_PYTHON_ARGUMENTS_SKIPPED_CELL,)),
    ("closure-round-limit-unchecked", (CLOSURE_ROUND_LIMIT_CELL,)),
    ("closure-call-index-frozen", (CLOSURE_CALL_INDEX_CELL,)),
    ("data-units-process-wide", (DATA_READ_CHANGED_CELL, DATA_READ_DIRECTORY_CELL)),
    ("judging-entry-index-not-shared", (SHARED_SCRIPT_INDEX_CELL,)),
    ("python-toolchain-queries-refused", (TOOLCHAIN_QUERY_CELL,)),
    ("python-toolchain-queries-keyed", (TOOLCHAIN_QUERY_CELL,)),
    ("python-registry-programs-ignored", (REGISTRY_PROGRAMS_CELL,)),
    ("python-registry-programs-linked", (REGISTRY_PROGRAMS_CELL,)),
    ("python-membership-tables-followed", (PROGRAM_ARGUMENTS_CELL,)),
    ("row-keyed-tables-whole", (ROW_KEYED_CELL,)),
    ("python-program-arguments-unnarrowed", (PROGRAM_ARGUMENTS_CELL, PYTHON_LAUNCH_WITHOUT_ENTRY_ARGUMENTS_CELL)),
    ("python-import-time-tables-followed", (PROGRAM_ARGUMENTS_CELL,)),
    ("python-slice-unpacking-unpaired", ("shell：exec python3 -B /dev/fd/3 … 3<<'PY' 里 (目录, 标签) = sys.argv[1:3] 拆包逐个对上，sys.path.insert(0, 目录) 折得出、连到它 import 的仓内模块",)),
    ("rust-kind-names-hyphenated", ("dump：种类名照 crates/singlefs-unit-graph 的拼法（macro_rules、item_macro、mod、mod_head、wgsl_compute）",)),
    ("rust-qualified-names-file-local", ("dump：限定名带 crate、目标与模块路径前缀，文档测试写 <模块>::<doctest line 行>",)),
    ("rust-doctest-fingerprint-from-rows", ("文档测试：代码块里只改注释指纹不变、改代码指纹变",)),
    ("rust-module-heads-dropped", ("模块头：#![…] 与行内模块头切成单元，模块里的单元连它，改内属性作废",)),
    ("rust-data-dump-unlisted", ("dump：include_str! 读的数据文件按读它的那一处列一行 data",)),
    ("wgsl-external-names-by-long-identifiers", ("WGSL 外名：首段连 _ 落在 macro_rules! 体或串字面量里有着落", "WGSL 外名：只在别的单元的长标识符里出现的拒算")),
    ("wgsl-external-name-sources-unlinked", ("WGSL 外名的着落做成边：给名字的 Rust 单元改了 WGSL 入口作废",)),
    ("rust-escaped-brace-captures-skipped", ("format! 串里 {{{名}}}（转义的 {{ 后紧跟的捕获）改了作废",)),
    ("rust-union-as-keyword", ("条目级宏调用里的 union 算它可能定义的名字",)),
    ("rust-module-heads-unlinked", ("R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废", "R-结构：改行内模块头的内属性，只那个模块里的入口作废")),
    ("rust-module-declarations-unlinked", ("R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废",)),
    ("rust-use-names-resolved", ("R-结构：use 引进的名字不往下解，没用到的那个改了不作废",)),
    ("rust-use-aliases-unresolved", ("R-结构：use a::b as c 之后调 c()，按 b 再解",)),
    ("rust-trait-impls-by-trait-arguments", ("R-隐式：只点名 trait 参数里的类型，那个 trait impl 不进闭包",)),
    ("rust-macro-generated-types-unnamed", (MACRO_TYPE_EDGE_CELL,)),
    ("rust-macro-rules-literal-names-unlinked", (MACRO_LITERAL_EDGE_CELL,)),
    ("rust-global-allocators-unlinked", (ALLOCATOR_EDGE_CELL,)),
    ("rust-foreign-self-trait-impls-unlinked", (FOREIGN_IMPL_EDGE_CELL,)),
    ("rust-build-environment-unlinked", (BUILD_ENVIRONMENT_EDGE_CELL,)),
    ("rust-excluded-path-dependencies-dropped", (EXCLUDED_PATH_DEPENDENCY_EDGE_CELL,)),
    ("rust-outside-crate-names-linked-unimported", (OUTSIDE_IMPORT_CELL,)),
    ("rust-outside-crate-names-linked-across-modules", (OUTSIDE_MODULE_CELL,)),
    ("rust-crates-names-linked-across-modules", (CRATES_MODULE_PRIVATE_CELL,)),
    ("rust-let-shadowed-names-resolved", (LET_SHADOW_BOUND_CELL, LET_SHADOW_PARAMETER_CELL, LET_SHADOW_CAPTURE_CELL, LET_SHADOW_LISTED_MACRO_CELL)),
    ("rust-macro-arguments-all-local", (LET_SHADOW_UNLISTED_MACRO_CELL, LET_SHADOW_NESTED_MACRO_CELL)),
    ("rust-macro-list-short", (STANDARD_MACRO_LIST_CELL,)),
    ("rust-use-paths-flattened", (USE_PATH_EXACT_CELL,)),
    ("rust-use-path-conditional-candidates-narrowed", (USE_PATH_CONDITIONAL_CELL,)),
    ("rust-use-path-re-exporting-modules-narrowed", (USE_PATH_RE_EXPORT_CELL,)),
    ("rust-use-path-first-segment-imported-narrowed", (USE_PATH_FIRST_SEGMENT_CELL, USE_PATH_SAME_USE_CELL, USE_PATH_SELF_IMPORT_CELL, USE_PATH_ITEM_MACRO_CELL)),
    ("rust-use-path-first-segment-same-use-excluded", (USE_PATH_SAME_USE_CELL, USE_PATH_SELF_IMPORT_CELL)),
    ("rust-use-path-first-segment-item-macros-ignored", (USE_PATH_ITEM_MACRO_CELL,)),
    ("rust-use-alias-leaf-followed-to-the-original", (USE_PATH_ALIAS_LEAF_CELL,)),
    ("rust-cfg-atoms-unnarrowed", ("cfg 成分：闭包没引到 gpu 的入口，带不带 gpu 特性两种目标下键相同",)),
    ("rust-cfg-macro-atoms-ignored", ("cfg 成分：闭包里有 cfg!、语句上的 #[cfg] 或 #[cfg_attr] 引到 gpu 的入口，带不带 gpu 特性键不同",)),
    ("wgsl-defined-name-suffixes-ignored", ("WGSL 外名：条目级宏调用定义的名字以 _ 加外名收尾就有着落",
                                            "WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变")),
    ("wgsl-prefix-remainders-unchecked", ("WGSL 外名：首段只落在无关的串里、其余那一段哪儿都没有的拒算",)),
    ("wgsl-name-suppliers-not-pulled-in", ("WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变",)),
    ("wgsl-name-suppliers-include-test-code", (WGSL_TEST_SUPPLIER_CELL,)),
    ("wgsl-modules-per-naming-unit", ("WGSL：闭包读到的几份 .wgsl 合成一个模块，名字跨文件解",)),
    ("rust-trait-self-paths-ignored", ("R-路径：trait 成员里的 Self::x 落到这个 trait 自己的成员 x",)),
    ("rust-doctest-structure-linked", ("R-结构：文档测试不连所挂的条目与那个模块的 use",)),
    ("rust-use-self-imports-ignored", ("R-结构：use a::{self} 引进名字 a，只写 a::f() 的单元连上这条 use",)),
    ("rust-open-by-trait-arguments", ("R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl",)),
    ("rust-blanket-impls-ignored", ("R-方法：blanket impl 够得到的方法算边，改它作废",)),
    ("rust-underscore-items-named", ("R-裸名：const _ 没有名字，let _ 与 _ => 不把 gpu 特性下的 const _ 拉进闭包，带不带 gpu 特性键相同",)),
    ("rust-includes-outside-root-refused", ("include_str! 读根外的文件：读得到收成数据单元、改它键变，读不到拒算 include-missing",)),
    ("rust-concatenated-includes-skipped", (CONCATENATED_INCLUDES_CELL,)),
    ("rust-blanket-self-parameter-only", ("R-方法：自身类型里出现泛型参数的 impl（(T, u32)、[T]、[T; N]、Box<T>、Option<Box<T>>、Sealed<T>）都按 blanket 放行，改 Sealed<T> 那一份作废",)),
    ("wgsl-other-file-definitions-unlinked", ("WGSL 外名在链接集里别的 include_str! 读进来的 .wgsl 里有定义：读它的 Rust 单元与那份 .wgsl 拉进闭包，改那个常量作废",)),
    ("rust-assume-specifications-unrecognized", (ASSUME_SPECIFICATION_FORMS_CELL, ASSUME_SPECIFICATION_KEY_CELL)),
    ("rust-assume-specification-names-unlinked", (ASSUME_SPECIFICATION_FORMS_CELL, ASSUME_SPECIFICATION_KEY_CELL)),
    ("rust-dependency-features-from-every-section", (FEATURE_SECTIONS_ENTRY_CRATE_CELL, FEATURE_SECTIONS_LINKED_CRATE_CELL)),
    ("rust-verus-semicolon-split-off", (VERUS_SEMICOLON_CELL,)),
    ("python-import-time-statements-ignored", (IMPORT_TIME_EXIT_CELL,)),
    ("python-def-heads-ignored", (IMPORT_TIME_EXIT_CELL,)),
    ("python-import-time-classes-named", (IMPORT_TIME_CLASS_CELL,)),
    ("python-main-guard-conjunctions-unrecognized", (MAIN_GUARD_IMPORT_CELL, MAIN_GUARD_REAL_CELL)),
    ("python-main-guards-last-only", (MAIN_GUARD_PROGRAM_CELL,)),
    ("python-main-guard-else-dropped", (MAIN_GUARD_ELSE_CELL,)),
    ("program-edge-exclusions-ignored", (EXCLUDED_EDGE_PYTHON_CELL, EXCLUDED_EDGE_SHELL_CELL, ADMISSION_EDGE_REACH_CELL)),
    ("program-edge-exclusions-shell-ignored", (EXCLUDED_EDGE_SHELL_CELL, ADMISSION_EDGE_REACH_CELL)),
    ("root-ignored-scripts-linked", (ROOT_IGNORED_SCRIPTS_CELL,)),
    ("excluded-edge-exit-unmeasured", (EXCLUDED_EDGE_EXIT_CELL,)),
    ("admission-edge-admitted-sample", (ADMISSION_EDGE_ANOMALY_CELL,)),
    ("admission-edge-header-unstripped", (ADMISSION_EDGE_STRIP_CELL,)),
    ("admission-edge-reach-unmeasured", (ADMISSION_EDGE_REACH_CELL,)),
    ("card-lease-client-words-granted-sample", (CARD_LEASE_CLIENT_WORDS_ANOMALY_CELL,)),
    ("card-lease-client-words-return-output-kept", (CARD_LEASE_CLIENT_WORDS_STRIP_CELL,)),
    ("card-lease-client-words-other-launches-unchecked", (CARD_LEASE_CLIENT_WORDS_REACH_CELL,)),
    ("judging-entry-refusals-ignored", (REAL_JUDGING_CLOSURES_CELL,)),
    ("e171-selftest-table-at-module-level", (E171_CLOSURE_CELL,)),
    ("e171-admission-selftest-at-module-level", (E171_CLOSURE_CELL,)),
    ("e171-list-check-imports-reuse-library", (E171_CLOSURE_CELL,)),
    ("judging-entry-workers-head-word", (JUDGING_ENTRY_WORKERS_CELL,)),
    ("product-view-kept-formal", (PRODUCT_VIEW_LEMMA_CELL, PRODUCT_VIEW_ENSURES_CELL)),
    ("product-view-strips-executable-code", (PRODUCT_VIEW_EXECUTABLE_CELL,)),
    ("product-view-edges-from-the-written-tokens", (PRODUCT_VIEW_EDGES_CELL,)),
    ("product-view-blanking-unverified", (PRODUCT_VIEW_UNVERIFIED_CELL,)),
    ("product-view-verifier-attributes-kept", (PRODUCT_VIEW_ATTRIBUTE_INSIDE_CELL, PRODUCT_VIEW_ATTRIBUTE_OUTSIDE_CELL)),
    ("selftest-lost-results-unchecked", (FORKED_GATHER_CELL,)),
    ("rust-lex-cache-namespace-lexer-only", (RUST_LEX_NAMESPACE_CHANGED_CELL,)),
    ("rust-lex-cache-namespace-whole-module", (RUST_LEX_NAMESPACE_KEPT_CELL,)),
    ("rust-lex-cache-namespace-module-placement-read", (RUST_LEX_NAMESPACE_KEPT_CELL,)),
    ("rust-lex-cache-namespace-reads-own-file", (RUST_LEX_NAMESPACE_CLOSURE_CELL,)),
    ("python-docstring-writes-counted", (DOCSTRING_WRITE_CELL,)),
    ("python-docstring-augmented-assignment-not-read", (DOCSTRING_AUGMENTED_CELL,)),
    ("python-docstring-call-readers-ignored", (DOCSTRING_GETATTR_CELL, DOCSTRING_GETDOC_CELL)),
    ("call-edge-exclusions-ignored", (EXCLUDED_CALL_EDGE_CELL, UNIT_GRAPH_INPUTS_CELL)),
    ("unit-graph-digest-skips-lock-file", (UNIT_GRAPH_DIGEST_CELL,)),
    ("selftest-fixtures-loaded-at-import", (SELFTEST_FIXTURES_OUTSIDE_CELL,)),
    ("selftest-fixtures-runtime-name-moved", (SELFTEST_FIXTURE_NAMES_CELL,)),
    ("selftest-fixture-names-receivers-ignored", (SELFTEST_FIXTURE_NAMES_CELL,)),
    ("selftest-modules-loaded-at-import", (SELFTEST_MODULES_OUTSIDE_CELL,)),
    ("shell-sourced-values-folded-in-reader", (SHELL_SOURCED_OWNER_CELL,)),
    ("shell-sourced-zero-own-file", (SHELL_SOURCED_ZERO_CELL,)),
    ("shell-sourced-siblings-unread", (SHELL_SOURCED_SIBLING_CELL,)),
    ("python-importer-sys-path-ignored", (PYTHON_IMPORTER_SYS_PATH_CELL,)),
    ("python-import-time-bindings-followed", (PYTHON_IMPORT_BINDING_UNUSED_CELL,)),
    ("registered-functions-drifted", (REGISTERED_FUNCTIONS_REAL_CELL,)),
    ("unit-graph-sources-top-level-only", (EXTERN_SYSTEM_LIBRARY_LIST_CELL, STANDARD_MACRO_LIST_CELL)),
    ("judging-entry-literals-split-unfollowed", (REAL_JUDGING_CLOSURES_CELL,)),
    ("shell-name-reach-context-dropped", (SHELL_REACHED_LIBRARY_CELL,)),
    ("python-importer-call-sites-ignored", (PYTHON_IMPORTER_CALL_SITES_CELL, PYTHON_IMPORTER_METHOD_SITES_CELL)),
    ("inline-bodies-launch-file-only", (INLINE_SOURCED_BODY_CELL,)),
    ("python-importer-sites-outside-family", (PYTHON_IMPORTER_OUTSIDE_FAMILY_CELL,)),
    ("python-split-part-environment-unread", (PYTHON_SPLIT_PART_ENVIRONMENT_CELL,)),
    ("shell-self-assignments-counted", (SHELL_SELF_ASSIGNMENT_CELL,)),
    ("python-compiled-part-exec-allowed", (COMPILED_PART_EXEC_CELL,)),
    ("python-module-object-exec-refused", (MODULE_OBJECT_EXEC_TEXT_CELL, MODULE_OBJECT_EXEC_FILE_CELL, MODULE_OBJECT_EXEC_UNCALLED_CELL,
                                           MODULE_OBJECT_EXEC_UNFOLDED_CELL)),
    ("python-module-object-exec-unfolded-allowed", (MODULE_OBJECT_EXEC_UNFOLDED_CELL,)),
    ("shell-function-printed-words-single", (MODULE_OBJECT_PRINTED_TEXT_CELL, MODULE_OBJECT_PRINTED_UNCALLED_CELL)),
    ("shell-read-heredoc-assignments-ignored", (MODULE_OBJECT_READ_TEXT_CELL, MODULE_OBJECT_READ_UNCALLED_CELL, MODULE_OBJECT_REFUSAL_LINES_CELL)),
    ("python-module-object-loader-call-sites-ignored", (MODULE_OBJECT_LOADER_SITES_CELL, MODULE_OBJECT_LOADER_SITES_OTHER_CELL)),
    ("python-module-object-refusal-lines-at-exec", (MODULE_OBJECT_REFUSAL_LINES_CELL, REGISTERED_FUNCTIONS_REAL_CELL)),
    ("registered-functions-module-object-texts-unread", (REGISTERED_FUNCTIONS_REAL_CELL,)),
    ("test-service-code-table-row-dropped", (TEST_SERVICE_CODE_TABLE_CELL,)),
    ("test-service-code-split-families-unrecognized", (TEST_SERVICE_CODE_REAL_CELL,)),
    ("test-service-code-keyed", (TEST_SERVICE_CODE_UNKEYED_CELL,)),
    ("runtime-data-files-short", (RUNTIME_DATA_FILES_CELL,)),
    ("registered-functions-short-names", (PROGRAM_WORDS_QUALIFIED_CELL,)),
    ("shell-read-assignments-unrecorded", (MODULE_OBJECT_READ_AFTER_LITERAL_CELL,)),
    ("test-service-code-entry-keyed", (TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL,)),
    ("test-service-code-test-entry-unkeyed", (TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL,)),
    ("python-joined-tails-unanchored", (PATH_TAILS_JOINED_CELL,)),
    ("profile-unkeyed-scan-sample", (PROFILE_UNKEYED_CELL,)),
    ("unit-graph-inputs-lock-undeclared", (UNIT_GRAPH_INPUTS_CELL,)),
)

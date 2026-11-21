# 算键库 scripts/gate/lib_function_units.py 的自证夹具的一份（总说明在 lib_function_units_selftest_fixtures.py 文件头）：
# 只由算键库的 load_selftest_fixtures 在自证开跑前按路径载入，读已载入的几份与自证各模块此刻的名字、不被 import。

# 只在弄坏开关打开时走到的那一支读自己的原文
EXCLUDED_FILE_DATA_CELL = "排除文件：只经它读到的数据不进键（只改判法文件一行注释键不变）"
EXCLUDED_FILE_PROGRAM_CELL = "排除文件：只经它起的程序不进键（改那个程序键不变）"
EXCLUDED_FILE_KEPT_CELL = "排除文件：判法自己的单元照进键（改它键变）"
EXCLUDED_FILE_UNSET_CELL = ("排除文件：不给排除时它读的数据照进键（它读的判法文件只改一行注释，键也变）；不给排除的闭包里（命令行 closure 不带 --judging），"
                            "算键库照进键而不读自己的原文，只改算键库一行注释键不变（RUST_LEX_NAMESPACE_CLOSURE_CELL）")
NOT_JUDGING_FILES_CELL = "真仓：lib_function_units.py 抄录的 NOT_JUDGING_FILES 与 lib_verdict_library.py 那一份相同"
JUDGING_CLOSURE_CELL = ("命令行 closure --judging 照 admission.py 的 not_judging_files_of_entry 排除：只经 scripts/gate/lib_function_units.py 起到的 pkg/only_via_role.py "
                        "与它自己都不进闭包、别的路起的 pkg/direct.py 照进；不带 --judging 三份都进；入口就是排除文件的不排")
# 夹具里那份 scripts/gate/lib_function_units.py 只是占着 NOT_JUDGING_FILES 的路径：判定算键时它按角色排除
JUDGING_CLOSURE_FIXTURE = {
    "pkg/entry.py": '''import os
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def main():
    subprocess.run(["python3", os.path.join(ROOT, "pkg/direct.py")], check=False)
    return subprocess.run(["python3", os.path.join(ROOT, "scripts/gate/lib_function_units.py")], check=False)


if __name__ == "__main__":
    main()
''',
    "scripts/gate/lib_function_units.py": '''import os
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def main():
    return subprocess.run(["python3", os.path.join(ROOT, "pkg/only_via_role.py")], check=False)


if __name__ == "__main__":
    main()
''',
    "pkg/direct.py": 'def main():\n    return 501\n\n\nif __name__ == "__main__":\n    main()\n',
    "pkg/only_via_role.py": 'def main():\n    return 502\n\n\nif __name__ == "__main__":\n    main()\n',
}
LOADED_ATTRIBUTES_CELL = ("python：按路径以非 __main__ 名载入、值不出载入方（函数里的局部名字、内联脚本模块级的容器）的模块只连载入方调到的 wanted 与 import 时执行的，"
                          "别处有 x.main() 也不连它的 main；交给调用方的、以 __main__ 名载入的照整个闭包连")
# pkg/library.py 的 main 进闭包就拒算（globals() 查找）；pkg/other.py 的 call 在别的文件里调 thing.main()，让闭包里见过 .main
LOADED_ATTRIBUTES_FIXTURE = {
    "pkg/library.py": ('import sys\n\n\ndef wanted():\n    return 601\n\n\ndef main():\n    return globals().get(sys.argv[1])()\n\n\n'
                       'if __name__ == "__main__":\n    sys.exit(main())\n'),
    "pkg/other.py": 'def call(thing):\n    return thing.main()\n',
    "pkg/other_main.py": 'import other\n\nif __name__ == "__main__":\n    other.call(None)\n',
    "pkg/loader.py": '''import importlib.util
import os

import other

HERE = os.path.dirname(os.path.abspath(__file__))


def entry():
    specification = importlib.util.spec_from_file_location("library_for_loader", os.path.join(HERE, "library.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module.wanted() + other.call(None)
''',
    "pkg/escaping.py": '''import importlib.util
import os

import other

HERE = os.path.dirname(os.path.abspath(__file__))


def load():
    specification = importlib.util.spec_from_file_location("library_for_escaping", os.path.join(HERE, "library.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def entry():
    return load().wanted() + other.call(None)
''',
    "pkg/as_main.py": '''import importlib.util
import os

import other

HERE = os.path.dirname(os.path.abspath(__file__))


def entry():
    specification = importlib.util.spec_from_file_location("__main__", os.path.join(HERE, "library.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module.wanted() + other.call(None)
''',
    "s/run.sh": '''#!/usr/bin/env bash
python3 pkg/other_main.py
python3 - <<'PY'
import importlib.util

_LOADED = []


def wanted_value():
    if not _LOADED:
        specification = importlib.util.spec_from_file_location("library_for_inline", "pkg/library.py")
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _LOADED.append(module)
    return _LOADED[0].wanted()


print(wanted_value())
PY
''',
}
EXCLUDED_FILE_FIXTURE = {
    "pkg/role.py": '''import os
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def launch():
    return subprocess.run(["python3", os.path.join(ROOT, "pkg/only_via_role.py")], check=False)


def helper(word):
    launch()
    with open(os.path.join(ROOT, "pkg/judge.py"), encoding="utf-8") as handle:
        return handle.read().count(word)
''',
    "pkg/only_via_role.py": 'def main():\n    return 401\n\n\nif __name__ == "__main__":\n    main()\n',
    "pkg/judge.py": '''import role
# 只改这一行注释


def kept():
    return 402


def judge():
    return role.helper("kept") + kept()
''',
}
LOCAL_NAME_SHADOW_CELL = "python：参数、局部赋值、循环变量、套着的函数里的局部量与模块里的函数同名：不连（改那个函数键不变）"
LOCAL_NAME_REAL_CALL_CELL = "python：函数里没有绑这个名字：照连模块里的函数（改它键变）"
LOCAL_NAME_GLOBAL_CELL = "python：global 声明过的名字不算局部：照连模块里的函数（改它键变）"
LOCAL_NAME_FIXTURE = {
    "pkg/shadow.py": '''def main():
    return 501


def declared():
    return 502


def by_parameter(main=False):
    return bool(main)


def by_assignment():
    main = [1]
    return len(main)


def by_loop():
    total = 0
    for main in range(3):
        total += main
    return total


def by_nested_loop():
    def inner(items):
        for main in items:
            yield main
    return list(inner([1]))


def by_closure():
    main = 7

    def inner():
        return main
    return inner()


def by_comprehension():
    return [main for main in range(3)]


def shadowing():
    return by_parameter(), by_assignment(), by_loop(), by_nested_loop(), by_closure(), by_comprehension()


def real_call():
    return main()


def by_global():
    global declared
    return declared()
''',
}
# 闭包的轮数与调用处索引：check 起的 pkg/later_program.py 下一轮才收进来，它调 rows.read_rows(<表>)，read_rows 读的是参数
CLOSURE_ROUND_LIMIT_CELL = "闭包轮数超过上限：抛 FunctionUnitsError，点名入口与轮数"
CLOSURE_CALL_INDEX_CELL = "闭包的调用处索引增量：头一轮之后折出的程序里的调用处照样认得（它读的数据文件改了键变）"
CLOSURE_ROUNDS_FIXTURE = {
    "pkg/__init__.py": "",
    "pkg/table.tsv": "alpha\n",
    "pkg/rows.py": 'def read_rows(path):\n    with open(path, encoding="utf-8") as handle:\n        return handle.read()\n',
    "pkg/later_program.py": '''import os

import rows

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    return rows.read_rows(os.path.join(HERE, "table.tsv"))


if __name__ == "__main__":
    main()
''',
    "pkg/launch.py": '''import os
import subprocess


def run_tool(program, arguments):
    return subprocess.Popen([program, *arguments])


def check(options):
    run_tool(os.path.join(options["tool_root"], "tool-binary"), ["--check"])
    return run_tool("pkg/later_program.py", [])
''',
}
SHARED_SCRIPT_INDEX_CELL = "判法入口拒算并行：closure_of 用并行之前父进程建好的脚本索引（SHARED_SCRIPT_INDICES）"
LET_SHADOW_BOUND_CELL = "R-裸名：局部 let 绑的名字不连同名函数（let sample = …; sample + 1），改那个函数键不变"
LET_SHADOW_PARAMETER_CELL = "R-裸名：参数名不连同名函数（fn f(sample: u32)），改那个函数键不变"
LET_SHADOW_CAPTURE_CELL = "R-裸名：串字面量里的 {名} 是局部 let 绑的，不连同名函数，改那个函数键不变"
LET_SHADOW_BEFORE_CELL = "R-裸名：let 这条语句的等号右边还在作用域外（let sample = sample();），连同名函数，改它键变"
LET_SHADOW_OTHER_BLOCK_CELL = "R-裸名：let 所在的块收尾之后同名照解，连同名函数，改它键变"
LET_SHADOW_CONSTANT_CELL = "R-裸名：let 写成常量的名字是拿它当模式，只遮函数，同名常量照连，改它键变"
USE_PATH_EXACT_CELL = "R-裸名：use 路径点到的模块里没有这个名字（经 pub use 转出来的），退回全连，转出的那一份改了键变"
USE_PATH_EXACT_FIXTURE = {
    "crates/fprelude/Cargo.toml": '[package]\nname = "fprelude"\n',
    "crates/fprelude/src/lib.rs": "pub mod prelude;\npub mod deep;\nmod user;\n\npub const WIDTH: usize = 1;\n",
    "crates/fprelude/src/prelude.rs": "pub use crate::deep::WIDTH;\n",
    "crates/fprelude/src/deep.rs": "pub const WIDTH: usize = 64;\n",
    "crates/fprelude/src/user.rs": "use crate::prelude::*;\n\npub fn width() -> usize {\n    WIDTH + 1\n}\n",
}
PROFILE_UNKEYED_CELL = ("构建档与线程数不进键：crates/singlefs-unit-graph/src 与这份库的源码里都没有编译档的行（PROFILE_SETTING_KEYS、profile.incremental、"
                        "依赖一侧的 codegen-units）")
USE_PATH_CONDITIONAL_CELL = "R-裸名：use 路径点到的候选带条件编译（另一支 cfg 下名字由 pub use 转来），退回全连，转出的那一份改了键变"
USE_PATH_RE_EXPORT_CELL = "R-裸名：use 路径点到的候选所在模块另有 use 按名字引进同名，退回全连，别的模块里同名的那一份改了键变"
USE_PATH_FIRST_SEGMENT_CELL = "R-裸名：use 的相对路径首段是别的 use 按名字（as 改名）引进的模块，退回全连，那个模块里的那一份改了键变"
USE_PATH_ALIAS_LEAF_CELL = "R-裸名：use 原名 as 名 那一支退回全连，别的模块里同名的那一份改了键变"
USE_PATH_SAME_USE_CELL = "R-裸名：同一条 use 里先 as 改名、再用改名后的模块走相对路径（use {crate::deep::real as short, short::helper_total}），退回全连，改名前那个模块里的那一份改了键变"
USE_PATH_SELF_IMPORT_CELL = "R-裸名：use foo::{self, helper_total}（foo 是本模块的子模块）的相对路径首段 foo 被这条 use 自己引进，退回全连，别的模块里同名的那一份改了键变"
USE_PATH_ITEM_MACRO_CELL = "R-裸名：本作用域有条目级宏调用（宏体里 use … as 改名），相对路径的首段指的模块看不出，退回全连，改名前那个模块里的那一份改了键变"
# 下面四份夹具：点到的候选带条件编译（USE_PATH_CONDITIONAL_FIXTURE：只在 cfg 开着时才有；另一支 cfg 下的写法不在样本里）、候选所在模块另有 use 引进同名
# （USE_PATH_RE_EXPORT_FIXTURE：候选自己不带条件；unrelated 里同名的那一份不经 use 也不经候选自己按名字连到，只有退回全连才连上：候选 backend::checksum 自己的名字
# 会经它所在模块的 pub use 连到 portable::checksum，所以改 portable 的那一份在点准与退回全连下键都变，分不出这一道）、相对路径首段是别的 use 的 as 改名、use 原名 as 名（USE_PATH_*_CELL）
USE_PATH_CONDITIONAL_FIXTURE = {
    "crates/fswap/Cargo.toml": '[package]\nname = "fswap"\n\n[features]\nfast = []\n',
    "crates/fswap/src/lib.rs": '''pub mod backend {
    #[cfg(feature = "fast")]
    pub fn checksum(data: &[u8]) -> u32 {
        data.len() as u32
    }
}

pub mod portable {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte)).sum()
    }
}

use crate::backend::checksum;

pub fn entry_point(data: &[u8]) -> u32 {
    checksum(data)
}
''',
}
USE_PATH_RE_EXPORT_FIXTURE = {
    "crates/freexport/Cargo.toml": '[package]\nname = "freexport"\n',
    "crates/freexport/src/lib.rs": '''pub mod backend {
    pub fn checksum(data: &[u8]) -> u32 {
        data.len() as u32
    }

    pub use crate::portable::checksum;
}

pub mod portable {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte)).sum()
    }
}

pub mod unrelated {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte) * 2).sum()
    }
}

use crate::backend::checksum;

pub fn entry_point(data: &[u8]) -> u32 {
    checksum(data)
}
''',
}
USE_PATH_FIRST_SEGMENT_FIXTURE = {
    "crates/falias/Cargo.toml": '[package]\nname = "falias"\n',
    "crates/falias/src/lib.rs": '''pub mod deep {
    pub mod real {
        pub fn helper_total() -> u32 {
            7
        }
    }
}

pub mod other {
    pub mod short {
        pub fn helper_total() -> u32 {
            8
        }
    }
}

use crate::deep::real as short;
use short::helper_total;

pub fn entry_point() -> u32 {
    helper_total()
}
''',
}
USE_PATH_ALIAS_LEAF_FIXTURE = {
    "crates/fdiverge/Cargo.toml": '[package]\nname = "fdiverge"\n',
    "crates/fdiverge/src/lib.rs": '''pub mod first {
    pub fn old_total() -> u32 {
        1
    }

    pub fn helper_total() -> u32 {
        2
    }
}

pub mod second {
    pub fn helper_total() -> u32 {
        3
    }
}

use crate::first::old_total as helper_total;

pub fn entry_point() -> u32 {
    helper_total()
}
''',
}
USE_PATH_SAME_USE_FIXTURE = {
    "crates/fsameuse/Cargo.toml": '[package]\nname = "fsameuse"\n',
    "crates/fsameuse/src/lib.rs": '''pub mod deep {
    pub mod real {
        pub fn helper_total() -> u32 {
            7
        }
    }
}

pub mod other {
    pub mod short {
        pub fn helper_total() -> u32 {
            8
        }
    }
}

macro_rules! bring_in_the_real_module_as_short {
    () => {
        use crate::deep::real as short;
    };
}

pub mod through_self_import {
    pub mod foo {
        pub fn helper_total() -> u32 {
            9
        }
    }

    use foo::{self, helper_total};

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

pub mod through_one_use {
    use {crate::deep::real as short, short::helper_total};

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

pub mod through_a_macro {
    bring_in_the_real_module_as_short!();
    use short::helper_total;

    pub fn entry_point() -> u32 {
        helper_total()
    }
}
''',
}
LET_SHADOW_FIXTURE = {
    "crates/fshadow/Cargo.toml": '[package]\nname = "fshadow"\n',
    "crates/fshadow/src/lib.rs": '''pub fn sample() -> u32 {
    11
}

pub fn bound() -> u32 {
    let sample = 2;
    sample + 1
}

pub fn parameter(sample: u32) -> u32 {
    sample + 1
}

pub fn captured() -> String {
    let sample = 3;
    format!("{sample}")
}

pub fn before() -> u32 {
    let sample = sample();
    sample + 1
}

pub fn other_block() -> u32 {
    {
        let sample = 4;
        let _unused = sample;
    }
    sample()
}

pub const LIMIT: u32 = 21;

pub fn constant_pattern() -> u32 {
    let LIMIT = 5;
    LIMIT
}

pub fn named_in_a_listed_macro() -> String {
    let sample = 6;
    format!("{}", sample)
}

pub fn named_in_a_macro_outside_the_list() -> u32 {
    let sample = 7;
    dbg!(sample)
}

pub fn named_in_a_listed_macro_inside_an_unlisted_macro() -> u32 {
    let sample = 8;
    dbg!(format!("{}", sample).len() as u32)
}
''',
}
LET_SHADOW_LISTED_MACRO_CELL = "R-裸名：局部 let 绑的名字写在名单里的宏（format!）的括号里，不连同名函数，改那个函数键不变"
LET_SHADOW_UNLISTED_MACRO_CELL = "R-裸名：局部 let 绑的名字写在名单之外的宏（dbg!）的括号里，照连同名函数，改那个函数键变"
LET_SHADOW_NESTED_MACRO_CELL = "R-裸名：名单里的宏（format!）套在名单之外的宏（dbg!）的括号里，里层的名字也照连同名函数，改那个函数键变"
STANDARD_MACRO_LIST_CELL = "R-裸名：宏名单与 crates/singlefs-unit-graph 的 STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE 逐项相同（读那份源码比）"
LAUNCH_ARGUMENT_CELL = ("python：命令是别的函数的参数（subprocess.Popen(command)、[program, *arguments]）：照闭包里每个调用点的实参折"
                        "（字面程序、根外程序、外层函数的变量、调仓内函数的返回值），不拒算")
LAUNCH_ARGUMENT_SCRIPT_CELL = "python：命令参数在调用点折出根里的脚本（run_tool(\"pkg/tool.sh\", …)）：连上那一份，改它键变"
LAUNCH_ARGUMENT_REFUSED_CELL = "python：命令参数在调用点给的是环境变量（os.environ[…]）：折不出，照旧拒算 subprocess-command-unfolded"
LAUNCH_ARGUMENT_FIXTURE = {
    "pkg/__init__.py": "",
    "pkg/tool.sh": "#!/bin/bash\necho tool\n",
    "pkg/launch.py": '''import os
import subprocess


def run_group(command, **options):
    return subprocess.Popen(command, stdout=subprocess.PIPE, start_new_session=True, **options)


def run_tool(program, arguments):
    return run_group([program, *arguments], cwd="/")


def tool_program(executable, wrapped):
    if not wrapped:
        return executable
    return os.path.join(os.path.dirname(executable), "tool-wrapper")


def installed_tool(directory):
    executable = os.path.join(directory, "tool-binary")
    return executable, "1.0"


def prepared(options):
    executable, _version = installed_tool(options["tool_root"])
    return options, executable


def check(root, options):
    _options, executable = prepared(options)
    run_tool("cargo", ["build"])

    def one(wrapped):
        return run_tool(tool_program(executable, wrapped), ["--check"])
    return [one(True), one(False)]


def check_script(root, options):
    _options, executable = prepared(options)
    run_tool("pkg/tool.sh", ["--check"])
    return run_tool(tool_program(executable, True), [])


def check_unfolded(root):
    return run_tool(os.environ["TOOL_PROGRAM"], ["--check"])


def selftest_sample():
    run_group(["bash", "-c", "echo sample"])
    return tool_program("/opt/tool/bin/tool-binary", False)
''',
}

# R-结构（模块头、mod 声明、use）与 R-隐式按自身类型认 trait impl 的小工作区（与 crates/singlefs-unit-graph 的 closure.rs 同一口径）
CLOSURE_RULES_FIXTURE = {
    "crates/frules/Cargo.toml": '[package]\nname = "frules"\n',
    "crates/frules/src/lib.rs": '''#![allow(dead_code)]
mod sub;
pub mod inline_area {
    #![allow(unused)]
    pub fn inside() -> u32 {
        1
    }
}
pub fn outside() -> u32 {
    2
}
pub mod tools {
    pub fn used() -> u32 {
        3
    }
    pub fn unused() -> u32 {
        4
    }
    pub fn original() -> u32 {
        5
    }
}
use tools::*;
pub fn glob_caller() -> u32 {
    used()
}
pub mod listing {
    use crate::tools::{used, unused};
    pub fn list_caller() -> u32 {
        used()
    }
}
pub mod renaming {
    use crate::tools::original as renamed;
    pub fn alias_caller() -> u32 {
        renamed()
    }
}
pub struct Inner;
pub struct Outer;
impl From<Inner> for Outer {
    fn from(_value: Inner) -> Outer {
        Outer
    }
}
pub fn make_inner() -> Inner {
    Inner
}
pub fn make_outer() -> Outer {
    Outer
}
''',
    "crates/frules/src/sub.rs": "pub fn in_sub() -> u32 {\n    6\n}\n",
}
CLOSURE_RULES_FILE = "crates/frules/src/lib.rs"

# 闭包外的四类边（文件头 Rust 闭包「闭包外的四类边」）：每格一个小工作区，改的那一处补边前不在入口的闭包里（键不变），补边后在（键变）；
# 每格配一个弄坏开关（BREAK_CELLS 里 rust-macro-generated-types-unnamed 到 rust-excluded-path-dependencies-dropped 那六项）退回补边前；
# 工作区外的 crate 按调用关系连的两格（OUTSIDE_IMPORT_CELL、OUTSIDE_MODULE_CELL）各配 rust-outside-crate-* 一个开关，退回按同名连
MACRO_EDGE_FIXTURE = {
    "crates/fmacro/Cargo.toml": '[package]\nname = "fmacro"\n',
    "crates/fmacro/src/lib.rs": '''macro_rules! listed_enumeration {
    ($(#[$attribute:meta])* $visibility:vis enum $name:ident { $($variant:ident,)+ }) => {
        $(#[$attribute])*
        $visibility enum $name { $($variant,)+ }
    };
}
listed_enumeration! {
    pub enum Weight {
        Light,
        Heavy,
    }
}
impl Weight {
    pub fn amount(self) -> u32 {
        match self {
            Weight::Light => 1,
            Weight::Heavy => 9,
        }
    }
}
pub fn heavy_amount() -> u32 {
    Weight::Heavy.amount()
}
macro_rules! code_table {
    ($($constant:ident = $code:literal,)*) => {
        $(pub const $constant: u32 = $code;)*
        pub const EVERY_CODE: &[u32] = &[$($constant,)*];
    };
}
code_table! {
    CODE_NONE = 0,
    CODE_FULL = 7,
}
pub fn code_count() -> usize {
    EVERY_CODE.len()
}
''',
}
MACRO_EDGE_FILE = "crates/fmacro/src/lib.rs"
ALLOCATOR_EDGE_FIXTURE = {
    "crates/falloc/Cargo.toml": '[package]\nname = "falloc"\n',
    "crates/falloc/src/main.rs": '''use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static GROWTHS: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        GROWTHS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn growths() -> usize {
    let mut values = Vec::new();
    values.push(1_u8);
    GROWTHS.load(Ordering::Relaxed)
}

fn main() {
    println!("{}", growths());
}
''',
}
FOREIGN_IMPL_EDGE_FIXTURE = {
    "crates/fforeign/Cargo.toml": '[package]\nname = "fforeign"\n',
    "crates/fforeign/src/lib.rs": '''pub trait Measured {
    fn measure(&self) -> u64;
}

impl Measured for std::time::Duration {
    fn measure(&self) -> u64 {
        self.as_secs() + 1
    }
}

pub fn measure_of<Value>(value: &Value) -> u64
where
    Value: Measured,
{
    value.measure()
}

pub fn measured_second() -> u64 {
    measure_of(&std::time::Duration::from_secs(1))
}
''',
}
BUILD_ENVIRONMENT_EDGE_FIXTURE = {
    "crates/fbuild/Cargo.toml": '[package]\nname = "fbuild"\n',
    "crates/fbuild/build.rs": '''fn reported_features() -> String {
    String::from("plain")
}

fn main() {
    println!("cargo::rustc-env=FBUILD_REPORTED_FEATURES={}", reported_features());
}
''',
    "crates/fbuild/src/lib.rs": '''pub fn reported() -> &'static str {
    env!("FBUILD_REPORTED_FEATURES")
}
''',
}
EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/fuser"]\nexclude = ["fservice"]\n',
    "crates/fuser/Cargo.toml": '[package]\nname = "fuser"\n\n[dependencies]\nfservice = { path = "../../fservice" }\n',
    "crates/fuser/src/lib.rs": "pub fn stop_count() -> u32 {\n    fservice::stopped_connections()\n}\n",
    "fservice/Cargo.toml": '[package]\nname = "fservice"\n\n[workspace]\n',
    "fservice/src/lib.rs": "pub fn stopped_connections() -> u32 {\n    3\n}\n",
}
# 工作区外的 crate 按调用关系连（文件头 Rust 闭包「工作区外的 path 依赖」后半）：fuser 只在字段名里写着 halted；fservice 里 log 与 bench 两个模块各有一个 Log，
# bench 模块里的 fn launch 与 Worker::launch 同名
OUTSIDE_CRATE_NAME_FIXTURE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/fuser"]\nexclude = ["fservice"]\n',
    "crates/fuser/Cargo.toml": '[package]\nname = "fuser"\n\n[dependencies]\nfservice = { path = "../../fservice" }\n',
    "crates/fuser/src/lib.rs": '''use fservice::control;
use fservice::restarts;

pub struct Counters {
    pub halted: u32,
}

pub fn served() -> u32 {
    let counters = Counters { halted: 1 };
    fservice::stopped_connections() + restarts() + control::paused() + counters.halted
}
''',
    "fservice/Cargo.toml": '[package]\nname = "fservice"\n\n[workspace]\n',
    "fservice/src/lib.rs": '''pub mod control {
    pub fn paused() -> u32 {
        5
    }
}

pub mod bench {
    pub struct Log {
        pub entries: u64,
    }

    pub fn launch() -> u32 {
        7
    }
}

pub mod log {
    pub struct Log {
        pub entries: u32,
    }
}

use log::Log;

pub struct Worker;

impl Worker {
    fn launch(&self) -> u32 {
        2
    }
}

pub fn stopped_connections() -> u32 {
    Worker.launch() + Log { entries: 1 }.entries
}

pub fn restarts() -> u32 {
    4
}

pub fn halted() -> u32 {
    6
}
''',
}
OUTSIDE_CRATE_NAME_ENTRY = "crates/fuser/src/lib.rs::served"
OUTSIDE_CRATE_NAME_FILE = "fservice/src/lib.rs"
OUTSIDE_IMPORT_CELL = ("工作区外的 crate 按引进的名字连：别的包里只是同名的字段 halted 不连到它的 fn halted，改它键不变；"
                       "use 引进的 restarts、use 引进的模块下的 control::paused、写成路径的 stopped_connections 照连，改它们键变")
OUTSIDE_MODULE_CELL = ("工作区外的 crate 里按模块连：use 引进 log::Log 的只连它、不连 bench 模块里同名的 Log，Worker::launch 自己的名字不连 bench::launch；"
                       "改 bench 模块里这两样键不变，改 log::Log、改 Worker::launch 键变")
CRATES_MODULE_PRIVATE_CELL = "R-裸名：crates/ 下的包同一个目标里按模块连：经 use 引进 format::SLOT_BYTES 的单元不连 gpu_checks 里私有的同名常量，改它键不变"
CRATES_MODULE_IMPORTED_CELL = "R-裸名：crates/ 下的包同一个目标里按模块连：经 use 引进的 format::SLOT_BYTES 照连，改它键变"
CRATES_MODULE_FIXTURE = {
    "crates/fmodules/Cargo.toml": '[package]\nname = "fmodules"\n',
    "crates/fmodules/src/lib.rs": "pub mod format;\nmod gpu_checks;\nmod cpu_checks;\n",
    "crates/fmodules/src/format.rs": "pub const SLOT_BYTES: usize = 64;\n",
    "crates/fmodules/src/gpu_checks.rs": "const SLOT_BYTES: usize = 32;\n\npub fn gpu_slots() -> usize {\n    SLOT_BYTES * 2\n}\n",
    "crates/fmodules/src/cpu_checks.rs": "use crate::format::SLOT_BYTES;\n\npub fn cpu_slots() -> usize {\n    SLOT_BYTES + 1\n}\n",
}

MACRO_TYPE_EDGE_CELL = "R-宏：条目级宏调用里 enum 定义的类型算被点名，.方法() 连到写给它的 impl，改方法作废"
MACRO_LITERAL_EDGE_CELL = "R-宏：macro_rules! 体里按字面写出的常量名登到这个宏的每次调用名下，点名它的入口连上调用，改调用体作废"
ALLOCATOR_EDGE_CELL = "#[global_allocator]：链接集里的分配器 static 进闭包，连上它的 GlobalAlloc impl，改 realloc 作废"
FOREIGN_IMPL_EDGE_CELL = "R-方法：本仓 trait 写在外部类型上的 impl（impl Tr for std::time::Duration）经泛型约束调到，方法连上，改它作废"
BUILD_ENVIRONMENT_EDGE_CELL = "构建脚本经 cargo::rustc-env 交的值：用 env! 读它的单元连到本 crate 构建脚本的 main，改 main 里算的值作废"
EXCLUDED_PATH_DEPENDENCY_EDGE_CELL = "工作区外的 path 依赖（根清单 exclude 里）：顺 path 依赖收进来，调到它的函数连上，改它作废"

# 4-3d：键的 cfg 成分收窄、WGSL 外名的 _ 后缀与供名字的边、trait 成员里的 Self::x、文档测试不连结构、use a::{self} 的小工作区；
# 闭包成员与 crates/singlefs-unit-graph 的 unit-graph closure 逐行核过（报告里有命令与原样输出）
CFG_ATOMS_FIXTURE = {
    "crates/fcfg/Cargo.toml": '[package]\nname = "fcfg"\n\n[features]\ngpu = []\n',
    "crates/fcfg/src/lib.rs": '''pub fn plain_entry() -> u32 {
    helper() + 1
}
fn helper() -> u32 {
    2
}
pub fn macro_entry() -> u32 {
    if cfg!(feature = "gpu") { 3 } else { 4 }
}
pub fn statement_entry() -> u32 {
    #[cfg(feature = "gpu")]
    let value = 8;
    #[cfg(not(feature = "gpu"))]
    let value = 9;
    value
}
#[cfg_attr(feature = "gpu", inline)]
pub fn attribute_only_entry() -> u32 {
    7
}
''',
}
CFG_ATOMS_FILE = "crates/fcfg/src/lib.rs"

WGSL_TEST_SUPPLIER_CELL = "WGSL 外名的着落不收测试函数：串里有外名的 #[test] fn 不并进闭包，改它的函数体键不变，改真给名字的宏调用键变"

TRAIT_SELF_FIXTURE = {
    "crates/ftrait/Cargo.toml": '[package]\nname = "ftrait"\n',
    "crates/ftrait/src/lib.rs": '''pub trait Shape {
    fn area(&self) -> u32 {
        Self::scale() * 2
    }
    fn scale() -> u32 {
        3
    }
}
pub struct Square;
impl Shape for Square {}
pub fn measure(square: &Square) -> u32 {
    square.area()
}
''',
}

DOCTEST_STRUCTURE_FIXTURE = {
    "crates/fdoc/Cargo.toml": '[package]\nname = "fdoc"\n',
    "crates/fdoc/src/lib.rs": '''use std::fmt::Debug;
/// ```
/// assert_eq!(fdoc::shown(), 1);
/// ```
pub fn owner() -> u32 {
    10
}
pub fn shown() -> u32 {
    1
}
pub fn debug_text(value: &dyn Debug) -> String {
    format!("{value:?}")
}
''',
}

USE_SELF_FIXTURE = {
    "crates/fuse/Cargo.toml": '[package]\nname = "fuse"\n',
    "crates/fuse/src/lib.rs": '''pub mod store {
    pub fn open() -> u32 {
        1
    }
}
pub mod user {
    use crate::store::{self};
    pub fn reader() -> u32 {
        store::open()
    }
}
''',
}

OPEN_TRAIT_ARGUMENT_FIXTURE = {
    "crates/fopen/Cargo.toml": '[package]\nname = "fopen"\n',
    "crates/fopen/src/lib.rs": '''pub trait Sink {
    fn level(&self) -> u32;
}
pub trait Wrap<T: ?Sized> {
    fn wrap(&self) -> u32;
}
pub struct Meter;
impl Sink for Meter {
    fn level(&self) -> u32 {
        1
    }
}
pub struct Holder;
impl Wrap<dyn Sink> for Holder {
    fn wrap(&self) -> u32 {
        2
    }
}
pub fn feed(sink: &dyn Sink) -> u32 {
    sink.level()
}
''',
}
OPEN_TRAIT_ARGUMENT_FILE = "crates/fopen/src/lib.rs"

BLANKET_FIXTURE = {
    "crates/fblanket/Cargo.toml": '[package]\nname = "fblanket"\n',
    "crates/fblanket/src/lib.rs": '''pub trait Shout {
    fn shout(&self) -> u32;
}
impl<T: Copy> Shout for T {
    fn shout(&self) -> u32 {
        6
    }
}
pub fn yell(value: &u32) -> u32 {
    value.shout()
}
''',
}
BLANKET_FILE = "crates/fblanket/src/lib.rs"

# 4-3k：自身类型里任何深度出现 impl 的泛型参数就是 blanket（Box<T>、Option<Box<T>> 旧判法认不出：头一个名字不是泛型参数）；
# 同一份写法在 code-tooling 的 function-units-two-implementations-agree-green 样本里与 unit-graph 对拍
BLANKET_SHAPES_FIXTURE = {
    "crates/fshapes/Cargo.toml": '[package]\nname = "fshapes"\n',
    "crates/fshapes/src/lib.rs": '''pub trait Weigh {
    fn weigh(&self) -> u32;
}
impl<T> Weigh for (T, u32) {
    fn weigh(&self) -> u32 {
        self.1
    }
}
impl<T> Weigh for [T] {
    fn weigh(&self) -> u32 {
        self.len() as u32
    }
}
impl<T, const N: usize> Weigh for [T; N] {
    fn weigh(&self) -> u32 {
        N as u32
    }
}
impl<T: Weigh> Weigh for Box<T> {
    fn weigh(&self) -> u32 {
        (**self).weigh() + 1
    }
}
impl<T: Weigh> Weigh for Option<Box<T>> {
    fn weigh(&self) -> u32 {
        match self {
            Some(inner) => inner.weigh() + 2,
            None => 0,
        }
    }
}
pub struct Sealed<T>(pub T);
impl<T> Weigh for Sealed<T> {
    fn weigh(&self) -> u32 {
        5
    }
}
pub fn heaviest(pair: &(u8, u32)) -> u32 {
    pair.weigh()
}
''',
}
BLANKET_SHAPES_FILE = "crates/fshapes/src/lib.rs"
# Box、Option 与 u32 在 PRIMITIVE_TYPES 里、算被点名，这五份总是进闭包；Sealed<T> 没被点名，只有按 blanket 放行才进
BLANKET_SHAPES_SELF_TYPES = ("(T, u32)", "[T]", "[T; N]", "Box<T>", "Option<Box<T>>", "Sealed<T>")

# 4-3k：WGSL 外名在同一链接集里别的 include_str! 读进来的 .wgsl 里有定义（入口只读 kernel.wgsl，SHARED_LIMIT 在 shared.wgsl 里）
WGSL_OTHER_FILE_FIXTURE = {
    "crates/fkernels/Cargo.toml": '[package]\nname = "fkernels"\n',
    "crates/fkernels/src/lib.rs": '''pub const KERNEL_SOURCE: &str = include_str!("kernel.wgsl");
pub const SHARED_SOURCE: &str = include_str!("shared.wgsl");
pub fn kernel_text() -> &'static str {
    KERNEL_SOURCE
}
''',
    "crates/fkernels/src/kernel.wgsl": '''@compute @workgroup_size(1)
fn main() {
    let limit = SHARED_LIMIT + 1u;
}
''',
    "crates/fkernels/src/shared.wgsl": "const SHARED_LIMIT: u32 = 7u;\n",
}
WGSL_OTHER_FILE_RUST = "crates/fkernels/src/lib.rs"

# 4-3h：const _ 不是 R-裸名的着落（gpu 特性下才编的 const _ 编译期断言被 let _ = …、_ => 里的 _ 拉进闭包，带不带 gpu 的两趟键就分了）；
# include_str! 读根外的文件（工作区在根下一层，读仓根下的登记文件）照 crates/singlefs-unit-graph 收成数据单元
UNDERSCORE_FIXTURE = {
    "crates/funderscore/Cargo.toml": '[package]\nname = "funderscore"\n\n[features]\ngpu = []\n',
    "crates/funderscore/src/lib.rs": '''pub const LIMIT: u32 = 4;
#[cfg(feature = "gpu")]
const _: () = assert!(LIMIT == 4);
pub fn pick(value: u32) -> u32 {
    let _ = value;
    match value {
        0 => 1,
        _ => LIMIT,
    }
}
''',
}
UNDERSCORE_FILE = "crates/funderscore/src/lib.rs"

OUTSIDE_INCLUDE_FIXTURE = {
    "workspace/crates/finclude/Cargo.toml": '[package]\nname = "finclude"\n',
    "workspace/crates/finclude/src/lib.rs": '''pub fn threads() -> usize {
    include_str!("../../../../thread-default").trim().len()
}
pub fn missing() -> usize {
    include_str!("../../../../not-there").len()
}
''',
    "thread-default": "31\n",
}
OUTSIDE_INCLUDE_FILE = "crates/finclude/src/lib.rs"

# 一个 concat!(…) 里的几个 include_str! 各切一个数据单元（crates/singlefs-unit-graph 的 reads_and_refusals_of：concat! 里的词照样逐个看）：
# 两份文件各读在自己那一行（lib.rs 第 2、3 行）
CONCATENATED_INCLUDES_FIXTURE = {
    "crates/fconcat/Cargo.toml": '[package]\nname = "fconcat"\n',
    "crates/fconcat/src/lib.rs": '''pub const SOURCES: &str = concat!(
    include_str!("header.txt"),
    include_str!("body.txt"),
);
pub fn sources() -> usize {
    SOURCES.len()
}
''',
    "crates/fconcat/src/header.txt": "header\n",
    "crates/fconcat/src/body.txt": "body\n",
}
CONCATENATED_INCLUDES_FILE = "crates/fconcat/src/lib.rs"
CONCATENATED_INCLUDES_LINES = (("crates/fconcat/src/header.txt", 2, b"header\n"), ("crates/fconcat/src/body.txt", 3, b"body\n"))
CONCATENATED_INCLUDES_CELL = ("concat!(include_str!(…), include_str!(…)) 里每个 include_str! 各切一个数据单元：dump 按读它的那一行各列一行 data，"
                              "改其中 body.txt 那一份入口的键变")

# Rust 闭包读到的 shell 脚本（文件头「Rust 闭包读到的 shell 脚本」）：入口照 gpu_card_configuration.rs 起配置判法的写法拼出脚本路径
SHELL_DATA_FIXTURE = {
    "crates/fshell/Cargo.toml": '[package]\nname = "fshell"\n',
    "crates/fshell/src/lib.rs": '''pub fn runs_the_sourcing_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/sourcing.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_plain_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/plain.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_dotted_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/dotted.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_unfolded_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/unfolded.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
''',
    "scripts/sourcing.sh": '''#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib/first.sh"
first_function
''',
    "scripts/lib/first.sh": '''#!/usr/bin/env bash
first_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$first_directory/second.sh"
first_function() {
  echo first
}
''',
    "scripts/lib/second.sh": '''#!/usr/bin/env bash
second_function() {
  echo second
}
''',
    "scripts/dotted.sh": '''#!/usr/bin/env bash
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIRECTORY/lib/dot-only.sh"
dot_function
''',
    "scripts/lib/dot-only.sh": '''#!/usr/bin/env bash
dot_function() {
  echo dot
}
''',
    "scripts/plain.sh": '''#!/usr/bin/env bash
echo plain
''',
    "scripts/unfolded.sh": '''#!/usr/bin/env bash
library="$1"
source "$library"
''',
}
SHELL_DATA_FILE = "crates/fshell/src/lib.rs"
SHELL_DATA_HOST_LINE = "host: x86_64-unknown-linux-gnu"
SHELL_DATA_SOURCE_CELL = ("Rust 闭包读到的 shell 脚本：它 source 进来的文件收成 data-file 单元，改一处读这份脚本的入口键变、读另一份不 source 它的脚本的入口键不变；"
                          "被 source 的每一份按读这份脚本的那一处记一条数据位置，unit_graph_key（与 crates/singlefs-unit-graph 对拍的那一把）随它变")
SHELL_DATA_DOT_CELL = "Rust 闭包读到的 shell 脚本：. <路径> 照 source 收（路径经 $(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd) 赋的变量），改被引的文件键变"
SHELL_DATA_NESTED_CELL = "Rust 闭包读到的 shell 脚本：两层（脚本 source 的库再 source 一份）逐层收，改里层那份键变"
SHELL_DATA_UNFOLDED_CELL = "Rust 闭包读到的 shell 脚本：source 的路径折不出拒算 shell-path-unfolded"

MANIFEST_ROOT_CELL = "Rust：清单目录往上拼出仓根（目录）拒算 runtime-path；点名清单目录里一份文件再往上取仓根的不拒算、那份文件的字节进键"
# 实验准入入口（scripts/experiment-apparatus/lib.rs::admit_experiment_run_or_exit）取仓根的两种写法
MANIFEST_ROOT_FIXTURE = {
    "crates/fbench/Cargo.toml": '[package]\nname = "fbench"\n',
    "crates/fbench/src/lib.rs": '''// 只是注释
pub fn root_by_parent_directories() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}
pub fn root_by_named_source() -> std::path::PathBuf {
    let this_source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    this_source.ancestors().nth(4).expect("lib.rs 在 <根>/crates/fbench/src/ 下").to_path_buf()
}
''',
}

CURRENT_EXE_FIXTURE = {
    "crates/fexe/Cargo.toml": '[package]\nname = "fexe"\n',
    "crates/fexe/src/lib.rs": '''#[cfg(test)]
mod tests {
    use std::process::Command;
    const CHILD_NAME: &str = "the_child_runs_alone";
    const CHILD_TEST: &str = "tests::the_child_runs_alone";
    fn run_the_child_by_name(name: &str) {
        let output = Command::new(std::env::current_exe().expect("路径")).args(["--exact", name, "--nocapture"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_parent_starts_the_child_by_a_constant() {
        let output = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", CHILD_TEST]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_parent_starts_the_child_through_a_parameter() {
        run_the_child_by_name(CHILD_TEST);
    }
    #[test]
    fn the_parent_names_the_child_by_a_format_of_literals_and_constants() {
        let output = Command::new(std::env::current_exe().expect("路径")).arg("--exact").arg(format!("{}::{CHILD_NAME}", "tests")).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_child_runs_alone() {
        let value = 1;
        assert_eq!(value, 1);
    }
    #[test]
    fn the_unrelated_test() {
        let note = 5;
        assert_eq!(note, 5);
    }
    #[test]
    fn the_parent_asks_for_a_name_nobody_defines() {
        let _ = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::missing"]).output();
    }
    #[test]
    fn the_parent_names_itself_before_the_exact_flag_by_a_module_path() {
        let test_name = format!("{}::x", module_path!().split_once("::").expect("包名").1);
        let _ = Command::new(std::env::current_exe().expect("路径")).args([test_name.as_str(), "--exact"]).output();
    }
}
''',
}
CURRENT_EXE_CELL = "Rust：current_exe() 交给 Command::new 起子进程、--exact 后面点名的测试（常量、参数、字面与常量拼的）连到那条 #[test]，改它的函数体键变、改别的测试键不变；点名不存在的、测试名写在 --exact 前面的照旧拒算 current-exe"
CURRENT_EXE_KEYED = ("the_parent_starts_the_child_by_a_constant", "the_parent_starts_the_child_through_a_parameter",
                     "the_parent_names_the_child_by_a_format_of_literals_and_constants")
CURRENT_EXE_REFUSED = ("the_parent_asks_for_a_name_nobody_defines", "the_parent_names_itself_before_the_exact_flag_by_a_module_path")

CURRENT_EXE_CONSTANT_FIXTURE = {
    "crates/fexec/Cargo.toml": '[package]\nname = "fexec"\n',
    "crates/fexec/src/lib.rs": '''#[cfg(test)]
mod names;
#[cfg(test)]
mod other_names;
#[cfg(test)]
mod tests {
    use crate::names::{AMBIGUOUS_TEST, CHILD_TEST};
    #[test]
    fn the_parent_names_the_child_by_a_constant_from_another_file() {
        let output = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", CHILD_TEST]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_parent_names_a_constant_defined_twice_with_different_values() {
        let _ = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", AMBIGUOUS_TEST]).output();
    }
    #[test]
    fn the_child_runs_alone() {
        let value = 1;
        assert_eq!(value, 1);
    }
    #[test]
    fn the_unrelated_test() {
        let note = 5;
        assert_eq!(note, 5);
    }
}
''',
    "crates/fexec/src/names.rs": 'pub const CHILD_TEST: &str = "tests::the_child_runs_alone";\npub const AMBIGUOUS_TEST: &str = "tests::the_child_runs_alone";\n',
    "crates/fexec/src/other_names.rs": 'pub const AMBIGUOUS_TEST: &str = "tests::missing";\n',
}
CURRENT_EXE_CONSTANT_CELL = ("Rust：current_exe() 的测试名里的常量定义在同包另一份文件里，--exact 后面用它：连到那条 #[test]、不拒算，改它的函数体键变、改别的测试键不变；"
                             "同名常量在同包两份文件里值不同的照旧拒算 current-exe")

CURRENT_EXE_PATH_ONLY_FIXTURE = {
    "crates/fexp/Cargo.toml": '[package]\nname = "fexp"\n',
    "crates/fexp/src/lib.rs": '''#[cfg(test)]
mod tests {
    #[test]
    fn it_only_asks_whether_it_has_a_path() {
        assert!(std::env::current_exe().is_ok());
    }
    #[test]
    fn it_reads_the_size_of_the_file_it_was_started_from() {
        let started_from = std::env::current_exe().expect("路径");
        let size = std::fs::metadata(&started_from).expect("读得到").len();
        assert!(size > 0);
    }
    #[test]
    fn the_unrelated_test() {
        let note = 5;
        assert_eq!(note, 5);
    }
}
''',
}
CURRENT_EXE_PATH_ONLY_CELL = "Rust：current_exe() 只问路径、同一函数里不起 Command 的不拒算：键算得出，改它的函数体键变、改别的测试键不变"
CURRENT_EXE_PATH_ONLY_ENTRIES = ("it_only_asks_whether_it_has_a_path", "it_reads_the_size_of_the_file_it_was_started_from")

CURRENT_EXE_SPAWN_FIXTURE = {
    "crates/fexs/Cargo.toml": '[package]\nname = "fexs"\n',
    "crates/fexs/src/lib.rs": '''#[cfg(test)]
mod tests {
    use std::process::Command;
    #[test]
    fn it_starts_itself_and_names_the_child() {
        let output = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::the_child_runs_alone"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_starts_itself_without_naming_a_test() {
        let output = Command::new(std::env::current_exe().expect("路径")).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_takes_the_path_first_and_hands_it_to_command() {
        let program = std::env::current_exe().expect("路径");
        let output = Command::new(program).args(["--exact", "tests::the_child_runs_alone"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_starts_a_sibling_of_its_own_program_file() {
        let program = std::env::current_exe().expect("路径").with_file_name("other");
        let output = Command::new(program).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_names_a_test_nobody_defines() {
        let _ = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::missing"]).output();
    }
    #[test]
    fn the_child_runs_alone() {
        let value = 1;
        assert_eq!(value, 1);
    }
}
''',
}
CURRENT_EXE_SPAWN_CELL = ("Rust：同一函数里起 Command 的 current_exe() 照旧按起自己判：折得出测试名的连边，"
                          "折不出的（没有 --exact、路径先存进变量再交 Command::new、路径改过、点名不存在的）拒算 current-exe")
CURRENT_EXE_SPAWN_KEYED = "it_starts_itself_and_names_the_child"
CURRENT_EXE_SPAWN_REFUSED = ("it_starts_itself_without_naming_a_test", "it_takes_the_path_first_and_hands_it_to_command", "it_starts_a_sibling_of_its_own_program_file",
                             "it_names_a_test_nobody_defines")

MANIFEST_DIRECTORY_FIXTURE = {
    "crates/fdir/Cargo.toml": '[package]\nname = "fdir"\n',
    "crates/fdir/rules/first.txt": "rule one\n",
    "crates/fdir/rules/nested/second.txt": "rule two\n",
    "crates/fdir/src/lib.rs": '''use std::path::Path;
const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");
pub fn unrelated_value() -> u32 {
    1
}
pub fn count_rules_directly() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("rules")).expect("列得了").count()
}
pub fn count_rules_by_variable() -> usize {
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    std::fs::read_dir(&rules).expect("列得了").count()
}
pub fn count_rules_by_constant() -> usize {
    let root = Path::new(MANIFEST);
    let nested = root.join("rules").join("nested");
    std::fs::read_dir(nested).expect("列得了").count()
}
pub fn count_every_crate() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("..")).expect("列得了").count()
}
pub fn count_by_climbing_from_a_variable() -> usize {
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    std::fs::read_dir(rules.parent().expect("有上一级")).expect("列得了").count()
}
''',
}
MANIFEST_DIRECTORY_CELL = "Rust：清单目录折出 crates/ 之下的目录收成数据目录（直接写、绑 let 变量、经私有常量），目录里一份文件改了键变、闭包外的改了键不变；crates/ 整个与变量上接别的方法的拒算 runtime-path"
MANIFEST_DIRECTORY_ENTRIES = ("count_rules_directly", "count_rules_by_variable", "count_rules_by_constant")

# prune-server/src/preflight.rs 的写法：清单目录绑给 let，一处 .join(参数) 折不出，一处 .join("…") 折出一份文件（crates/singlefs-unit-graph 的 fold_path_chain 逐处折）
MANIFEST_BINDING_FIXTURE = {
    "crates/fpre/Cargo.toml": '[package]\nname = "fpre"\n',
    "crates/fpre/tools/check.py": "print('check')\n",
    "crates/fpre/src/lib.rs": '''use std::path::Path;
pub fn check_from_the_manifest_directory(source_relative_to_manifest: &str) -> usize {
    let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = manifest_directory.join(source_relative_to_manifest);
    let script = manifest_directory
        .join("tools")
        .join("check.py");
    source.as_os_str().len() + script.as_os_str().len()
}
pub fn check_from_the_crates_directory(source_relative_to_manifest: &str) -> usize {
    let crates_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let source = crates_directory.join(source_relative_to_manifest);
    let script = crates_directory.join("fpre").join("tools/check.py");
    source.as_os_str().len() + script.as_os_str().len()
}
pub fn check_through_an_alias() -> usize {
    let tools: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tools");
    let renamed = tools;
    let script = Path::new(renamed).join("check.py");
    script.as_os_str().len()
}
''',
}
MANIFEST_BINDING_CELL = ("Rust：清单目录绑给 let 变量的，一处 .join(变量) 折不出、一处折出文件：这个单元拒算 runtime-path、那份文件照记一条 data 读"
                         "（绑的是清单目录本身与 crates/ 整个都一样）；别名再绑一层、Path::new(别名) 接的链照跟")
# 入口 → (env! 那一行, 要不要拒算)：每个入口都只读 crates/fpre/tools/check.py 一份，读记在 env! 那一行
MANIFEST_BINDING_ENTRIES = {"check_from_the_manifest_directory": (3, True), "check_from_the_crates_directory": (11, True), "check_through_an_alias": (17, False)}

RUST_LEX_CACHE_CELL = "Rust 词法单元的磁盘缓存：同一份文本第二次取缓存、与现切的逐项相同；改一个字符（长度不变）不沿用、切出来的随文本变；缓存文件坏了照常现切"

DIRECTORY_CLOSURE_LINE_CELL = "命令行 closure：数据目录单元按读它的那一处列（data-directory 加目录、读它的文件与行、目录指纹），不打成 <目录>:1-<文件数>"

LAUNCHED_PROGRAMS_CELL = "登记的被起程序（夹具）：登记了的两处（路径折不出的、路径折得出而根里没有的）不拒算、被起的那一份整份连上（改它键变），不登记时都拒算"
LAUNCHED_PROGRAMS_FIXTURE = {
    "tool/driver.py": ('import os\nimport subprocess\nimport sys\n\n\ndef drive():\n    return 7\n\n\n'
                       'def launch_self():\n    return subprocess.run([sys.executable, os.environ["DRIVER_PATH"], "drive"], check=False)\n\n\n'
                       'def launch_template():\n    return subprocess.run([sys.executable, "{tree}/tool/driver.py", "drive"], check=False)\n\n\n'
                       'if __name__ == "__main__":\n    drive()\n'),
}
LAUNCHED_PROGRAMS_FIXTURE_ROWS = {
    ("tool/driver.py", "launch_self", "[sys.executable, os.environ['DRIVER_PATH'], 'drive']"): ("tool/driver.py",),
    ("tool/driver.py", "launch_template", "[sys.executable, '{tree}/tool/driver.py', 'drive']"): ("tool/driver.py",),
}

IMPORTED_SYS_PATH_CELL = "python：import 进来的仓内模块在载入时往 sys.path 加的目录，对引它的模块同样生效（那个目录里的模块连得上，改它键变）"
IMPORTED_SYS_PATH_FIXTURE = {
    "pkg/side.py": 'import os\nimport sys\n\nsys.path.insert(1, os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib"))\n',
    "pkg/lib/helper_mod.py": 'def helper():\n    return 3\n',
    "pkg/main.py": 'import side\nimport helper_mod\n\n\ndef entry():\n    return side is not None and helper_mod.helper()\n',
}
REGISTERED_INLINE_LAUNCH_CELL = "python：整条登记过的 bash -c，正文字面而 $2 换不进去（带引号的多行串）：不登记拒算，登记了不拆开认、不拒算"
REGISTERED_INLINE_LAUNCH_FIXTURE = {
    "tool/threads.py": ('import subprocess\nimport sys\n\nPROGRAM = """\nimport sys\nprint("x", sys.argv[1])\n"""\n\n\n'
                        'def launch():\n    return subprocess.run(["bash", "-c", \'"$1" -c "$2" "$3" & wait\', "name", sys.executable, PROGRAM, "one"], check=False)\n'),
}
REGISTERED_INLINE_LAUNCH_ROWS = {("tool/threads.py", "launch", """['bash', '-c', '"$1" -c "$2" "$3" & wait', 'name', sys.executable, PROGRAM, 'one']"""): "夹具"}

PROCESS_SUBSTITUTION_ARGUMENT_CELL = 'shell：python3 - <(命令) "$y" <<\'PY\'：进程替换不切断外面那条命令，here-doc 照旧是 python3 的脚本，不拒算'
PROCESS_SUBSTITUTION_ARGUMENT_FIXTURE = {
    "s/psub.sh": ('#!/usr/bin/env bash\npython3 - <(printf \'%s\\n\' "$x") "$y" <<\'PY\'\nimport sys\nprint(sys.argv)\nPY\n'),
}
LAUNCHED_SHELL_CELL = '登记的被起程序（夹具，shell）：source 的路径折不出，登记了连到那一份（改它键变），不登记拒算'
LAUNCHED_SHELL_FIXTURE = {
    "tool/source_lib.sh": '#!/usr/bin/env bash\nsource "${LIB_PROGRAM%/*}/lib.sh"\nlib_hello\n',
    "tool/lib.sh": 'lib_hello() {\n  echo 7\n}\n',
}
LAUNCHED_SHELL_FIXTURE_ROWS = {("tool/source_lib.sh", "<script-body>", "${LIB_PROGRAM%/*}/lib.sh"): ("tool/lib.sh",)}

REFUSAL_LINES_CELL = 'shell：拒算落在真起它的那一行（bash -c "$词" 那一行、命令位置上折不出的词跟在 && 之后的那一行），不落在头一处提到 bash 或那个词的行'
REFUSAL_LINES_FIXTURE = {
    "s/inline_word.sh": '#!/usr/bin/env bash\nnote="bash is mentioned here"\necho "run: bash tool.sh"\nbash -c "$TEXT_FROM_OUTSIDE"\n',
    "s/command_word.sh": '#!/usr/bin/env bash\nname="$NAME_FROM_OUTSIDE"\necho "$name"\n( true && "$name" )\n',
}
REFUSAL_WORD_LINES_CELL = ("shell：拒算落在真起它的那一行：词按整个词找（bash join 不落在前面 bash s/tools/joined.sh 那一行），"
                           "剪过的正文里解释器是代进来的 \"$1\" 的落在原文 \"$1\" join 那一行（不落在前面 echo join 那一行）")
REFUSAL_WORD_LINES_FIXTURE = {
    "s/tools/joined.sh": "#!/usr/bin/env bash\necho joined\n",
    "s/word_bounds.sh": "#!/usr/bin/env bash\nbash s/tools/joined.sh\nbash join\n",
    "s/positional_interpreter.sh": '#!/usr/bin/env bash\necho join\n"$1" join\n',
    "s/positional_caller.sh": "#!/usr/bin/env bash\nbash s/positional_interpreter.sh bash\n",
}
CASE_SUBJECT_CELL = "shell：bash s/dispatch.sh bash：剪过的正文里 case \"$1\" in 代成 case bash in，in 不当成 bash 起的脚本：不拒算"
CASE_SUBJECT_FIXTURE = {
    "s/child.sh": "#!/usr/bin/env bash\necho child\n",
    "s/tools/one.py": 'def main():\n    return 1\n\n\nif __name__ == "__main__":\n    main()\n',
    "s/tools/two.py": 'def main():\n    return 2\n\n\nif __name__ == "__main__":\n    main()\n',
    "s/dispatch.sh": ('#!/usr/bin/env bash\nif [[ "${1:-}" == --selftest ]]; then\n  bash s/child.sh\n  exit $?\nfi\ncase "$1" in\n  run)\n'
                      '    python3 s/tools/one.py\n    ;;\n  *)\n    python3 s/tools/two.py\n    ;;\nesac\n'),
    "s/case_caller.sh": "#!/usr/bin/env bash\nbash s/dispatch.sh bash\n",
}
ENV_ARRAY_CELL = 'shell：env 起头的数组里 NAME="$值" 是 env 的赋值，"${数组[@]}" "$@" 不拒算'
ENV_ARRAY_FIXTURE = {
    "s/env_array.sh": ('#!/usr/bin/env bash\nscene_command_for() {\n  local scene="$work/$1"\n  scene_command=(env -u SOME_VARIABLE\n'
                       '    PATH="$scene/local-bin:$PATH" OTHER_SETTING="$scene/other")\n}\n'
                       'scene_environment() {\n  scene_command_for "$1"\n  shift\n  "${scene_command[@]}" "$@"\n}\nscene_environment one true\n'),
}
PROCESS_SUBSTITUTION_HEREDOC_CELL = 'shell：< <(python3 - "$x" <<\'PY\' …) 进程替换里的 here-doc 照旧是 python3 的脚本，不拒算'
PROCESS_SUBSTITUTION_HEREDOC_FIXTURE = {
    "s/psub_heredoc.sh": ('#!/usr/bin/env bash\nwhile IFS= read -r line; do echo "$line"; done < <(python3 - "$x" <<\'PY\'\nimport sys\nprint(sys.argv)\nPY\n)\n'),
}

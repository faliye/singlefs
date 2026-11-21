# 算键库 scripts/gate/lib_function_units.py 的自证才读的模块级常量：夹具文本、格名、开关到格的对照表（BREAK_CELLS）；分在三份里，
# 按次序载入：本份、lib_function_units_selftest_fixtures_closure_rules.py、lib_function_units_selftest_fixtures_real_repo.py（每份读排在它之前载入的几份定义的名字）。
# 不单独起、不被 import：只由算键库的 load_selftest_fixtures 在自证开跑前按路径载入；载入前先把自证各模块此刻的名字放进这份模块，
# 这里的式子照那些名字求值，求出来的名字再放回自证各模块（格函数与 selftest 照名字读它们）。平常 import 算键库不载入这几份，
# 这里的常量不进它 import 时执行的语句，也就不进经它收的各条判法闭包（D13（验证路线） 已定项 17）。
# 放进来的只是运行期函数一个都不读的那几样；运行期也读的、正文里写着自证起法的那几样留在算键库各模块里（算键库文件头「自证夹具」）。

RUST_FIXTURE = {
    "crates/fcore/Cargo.toml": '[package]\nname = "fcore"\n',
    "crates/fcore/src/lib.rs": '''pub const WIDTH: usize = 4;
pub trait Device {
    fn read(&self) -> u32;
    fn doubled(&self) -> u32 {
        self.read() * 2
    }
}
pub fn recover(device: &dyn Device) -> u32 {
    device.read() + 1
}
pub fn recover_generic<D: Device>(device: &D) -> u32 {
    device.doubled()
}
pub struct Money(pub u32);
impl std::ops::Add for Money {
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money(self.0 + other.0)
    }
}
pub struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = 1;
    }
}
macro_rules! make_constant_function {
    ($name:ident) => {
        pub fn $name() -> u32 {
            5
        }
    };
}
make_constant_function!(five);
pub trait Limit {
    const LIMIT: u32;
}
pub fn limit_of<T: Limit>() -> u32 {
    T::LIMIT
}
pub fn helper() -> u32 {
    3
}
pub static TABLE: [fn() -> u32; 1] = [helper];
pub fn unrelated() -> u32 {
    99
}
pub struct Buffer {
    pub bytes: [u8; WIDTH],
}
''',
    "crates/fharness/Cargo.toml": '[package]\nname = "fharness"\n[dependencies]\nfcore = { path = "../fcore" }\n',
    "crates/fharness/src/lib.rs": '''use fcore::{Device, Limit};
pub struct Memory;
impl Device for Memory {
    fn read(&self) -> u32 {
        7
    }
}
pub struct Small;
impl Limit for Small {
    const LIMIT: u32 = 10;
}
pub struct Other;
impl Device for Other {
    fn read(&self) -> u32 {
        8
    }
}
''',
    "crates/ftier/Cargo.toml": '[package]\nname = "ftier"\n[dependencies]\nfcore = { path = "../fcore" }\nfharness = { path = "../fharness" }\nfextra = { path = "../fextra" }\n',
    "crates/ftier/src/lib.rs": '''use fcore::{recover, recover_generic, Device, Money};
use fharness::{Memory, Small};
pub fn judge_through_dyn() -> u32 {
    recover(&Memory)
}
pub fn judge_through_generic() -> u32 {
    recover_generic(&Memory)
}
pub fn judge_operator() -> u32 {
    (Money(1) + Money(2)).0
}
pub fn judge_drop() -> u32 {
    let _guard = fcore::Guard;
    0
}
pub fn judge_macro() -> u32 {
    fcore::five()
}
pub fn judge_associated_constant() -> u32 {
    fcore::limit_of::<Small>()
}
pub fn judge_table() -> u32 {
    (fcore::TABLE[0])()
}
pub fn judge_open(device: &dyn Device) -> u32 {
    device.read()
}
pub fn judge_array_length() -> usize {
    std::mem::size_of::<fcore::Buffer>()
}
pub fn judge_listing() -> u32 {
    fextra::shade_count()
}
pub fn judge_speed() -> u32 {
    fextra::speed()
}
pub fn judge_proven() -> u32 {
    fextra::proven()
}
pub fn judge_documented() -> u32 {
    fextra::documented()
}
pub fn judge_table_rows() -> usize {
    fextra::table_rows()
}
pub fn judge_kernel() -> String {
    fextra::kernel_source()
}
pub fn judge_greeting() -> String {
    fextra::greet()
}
''',
    "crates/fextra/Cargo.toml": '[package]\nname = "fextra"\n[features]\ndefault = ["fast"]\nfast = []\nslow = []\n',
    "crates/fextra/src/lib.rs": '''#[macro_export]
macro_rules! listing {
    ($($item:item)*) => { $($item)* };
}
crate::listing! {
    pub enum Shade {
        Light,
        Dark,
    }
}
pub fn shade_count() -> u32 {
    Shade::Dark as u32 + 1
}
#[cfg(feature = "fast")]
pub fn speed() -> u32 {
    11
}
#[cfg(feature = "slow")]
pub fn speed() -> u32 {
    22
}
verus! {
pub fn bounded(x: u32) -> (r: u32)
    ensures
        if x > 0 { r > 0 } else { r == 0 },
{
    x
}
pub fn proven() -> u32 {
    9
}
} // verus!
/// 文档说明。
///
/// ```
/// assert_eq!(fextra::documented(), 4);
/// ```
pub fn documented() -> u32 {
    4
}
pub const TABLE_TEXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/table.txt"));
pub fn table_rows() -> usize {
    TABLE_TEXT.lines().count()
}
pub const KERNEL_COMMON: &str = include_str!("common.wgsl");
pub const KERNEL_MAIN: &str = include_str!("main.wgsl");
pub fn prelude() -> String {
    format!("const GENERATED_{}: u32 = 1u;", "LIMIT")
}
pub fn kernel_source() -> String {
    format!("{}\\n{KERNEL_COMMON}\\n{KERNEL_MAIN}", prelude())
}
const GREETING: &str = "hi";
pub fn greet() -> String {
    format!("{GREETING}!")
}
''',
    "crates/fextra/data/table.txt": "row\nrow\n",
    "crates/fextra/src/common.wgsl": '''// 共用的那一半
fn helper_used(x: u32) -> u32 {
    return x + GENERATED_LIMIT;
}
fn helper_unused(x: u32) -> u32 {
    return x * 7u;
}
''',
    "crates/fextra/src/main.wgsl": '''@group(0) @binding(0) var<storage, read_write> output: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    output[id.x] = helper_used(id.x);
}
''',
}
RUST_FIXTURE_ENTRY_FILE = "crates/ftier/src/lib.rs"
RUST_FIXTURE_CASES = (
    ("dyn 分派到下游 crate 的 impl", "judge_through_dyn", "crates/fharness/src/lib.rs", "    fn read(&self) -> u32 {\n        7", "    fn read(&self) -> u32 {\n        70", "changed"),
    ("dyn 分派：闭包里的 dyn 不连没被点名的另一个 impl，改它不作废", "judge_through_dyn", "crates/fharness/src/lib.rs", "        8\n", "        80\n", "same"),
    ("泛型约束分派", "judge_through_generic", "crates/fharness/src/lib.rs", "    fn read(&self) -> u32 {\n        7", "    fn read(&self) -> u32 {\n        70", "changed"),
    ("trait 默认方法", "judge_through_generic", "crates/fcore/src/lib.rs", "self.read() * 2", "self.read() * 3", "changed"),
    ("运算符重载", "judge_operator", "crates/fcore/src/lib.rs", "Money(self.0 + other.0)", "Money(self.0 * other.0)", "changed"),
    ("Drop 隐式调用", "judge_drop", "crates/fcore/src/lib.rs", "let _ = 1;", "let _ = 2;", "changed"),
    ("声明宏展开出的函数", "judge_macro", "crates/fcore/src/lib.rs", "        pub fn $name() -> u32 {\n            5", "        pub fn $name() -> u32 {\n            6", "changed"),
    ("关联常量（经泛型）", "judge_associated_constant", "crates/fharness/src/lib.rs", "const LIMIT: u32 = 10;", "const LIMIT: u32 = 11;", "changed"),
    ("函数指针表", "judge_table", "crates/fcore/src/lib.rs", "pub fn helper() -> u32 {\n    3", "pub fn helper() -> u32 {\n    4", "changed"),
    ("开放入口：参数是 dyn", "judge_open", "crates/fharness/src/lib.rs", "    fn read(&self) -> u32 {\n        7", "    fn read(&self) -> u32 {\n        70", "changed"),
    ("数组长度里的常量", "judge_array_length", "crates/fcore/src/lib.rs", "pub const WIDTH: usize = 4;", "pub const WIDTH: usize = 5;", "changed"),
    ("阴性：够不到的函数", "judge_macro", "crates/fcore/src/lib.rs", "    99\n", "    98\n", "same"),
    ("阴性：只改注释", "judge_operator", "crates/fcore/src/lib.rs", "        Money(self.0 + other.0)", "        // 只加注释\n        Money(self.0 + other.0)", "same"),
    ("带路径的条目级宏调用定义的枚举：改了作废", "judge_listing", "crates/fextra/src/lib.rs", "        Light,\n        Dark,", "        Dark,\n        Light,", "changed"),
    ("verus! 里 ensures 带花括号：下一个函数照样切开，改它作废", "judge_proven", "crates/fextra/src/lib.rs", "    9\n", "    10\n", "changed"),
    ("verus! 里够不到的函数改了不作废", "judge_proven", "crates/fextra/src/lib.rs", "{\n    x\n}", "{\n    x + 0\n}", "same"),
    ("cfg(feature) 关着的单元改了不作废", "judge_speed", "crates/fextra/src/lib.rs", "    22\n", "    23\n", "same"),
    ("cfg(feature) 开着的单元改了作废", "judge_speed", "crates/fextra/src/lib.rs", "    11\n", "    12\n", "changed"),
    ("只改文档注释不作废", "judge_documented", "crates/fextra/src/lib.rs", "/// 文档说明。", "/// 文档说明，改了措辞。", "same"),
    ("改文档注释里的代码块：入口 fn 不作废", "judge_documented", "crates/fextra/src/lib.rs", "documented(), 4);", "documented(), 5);", "same"),
    ("include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), …)) 读的文件改了作废", "judge_table_rows", "crates/fextra/data/table.txt", "row\nrow\n", "row\nrow\nrow\n", "changed"),
    ("WGSL 只改注释不作废", "judge_kernel", "crates/fextra/src/common.wgsl", "// 共用的那一半", "// 共用的那一半，改了措辞", "same"),
    ("WGSL 够不到的函数改了不作废", "judge_kernel", "crates/fextra/src/common.wgsl", "x * 7u", "x * 8u", "same"),
    ("WGSL 够得到的函数改了作废", "judge_kernel", "crates/fextra/src/common.wgsl", "x + GENERATED_LIMIT", "x - GENERATED_LIMIT", "changed"),
    ("format! 串里内联捕获的常量改了作废", "judge_greeting", "crates/fextra/src/lib.rs", 'GREETING: &str = "hi";', 'GREETING: &str = "hello";', "changed"),
)

# 与 crates/singlefs-unit-graph 对齐的那几条（dump 写法、模块头、文档测试指纹、数据单元、WGSL 外名着落、{{{名}}}、union）的小工作区
ALIGNMENT_FIXTURE = {
    "crates/falign/Cargo.toml": '[package]\nname = "falign"\n',
    "crates/falign/src/lib.rs": '''#![allow(dead_code)]
//! 模块说明。
//!
//! ```
//! // 只是注释
//! assert_eq!(falign::width(), 4);
//! ```
pub mod nested {
    #![allow(unused)]
    pub fn inside() -> u32 {
        1
    }
}
mod helpers;
pub const WIDTH: u32 = 4;
pub fn width() -> u32 {
    WIDTH
}
macro_rules! declare_named {
    ($name:ident) => {
        pub fn $name() -> u32 {
            2
        }
    };
}
declare_named!(union);
pub fn call_union() -> u32 {
    union()
}
pub fn wrapped() -> String {
    format!("{{{WIDTH}}}")
}
pub const TABLE_TEXT: &str = include_str!("table.txt");
pub fn table() -> usize {
    TABLE_TEXT.len()
}
pub const KERNEL: &str = include_str!("kernel.wgsl");
macro_rules! failure_name {
    ($name:ident) => {
        concat!(stringify!(FAILURE_), stringify!($name))
    };
}
pub fn kernel_source() -> String {
    format!("const {}: u32 = 0u;\\nconst T_{}: u32 = 1u;\\n{KERNEL}", failure_name!(POINT_NONE), "HEADER")
}
pub const KERNEL_COUNT: &str = include_str!("kernel_count.wgsl");
pub fn count_kernel_source() -> String {
    let COUNT_LIMIT = 3;
    format!("const X: u32 = {}u;\\n{KERNEL_COUNT}", COUNT_LIMIT)
}
''',
    "crates/falign/src/helpers.rs": "pub fn help() -> u32 {\n    5\n}\n",
    "crates/falign/src/main.rs": "fn main() {\n    println!(\"{}\", falign::width());\n}\n",
    "crates/falign/src/bin/tool.rs": "fn main() {}\n",
    "crates/falign/src/table.txt": "row\n",
    "crates/falign/src/kernel.wgsl": '''@group(0) @binding(0) var<storage, read_write> output: array<u32>;
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    output[id.x] = T_HEADER + FAILURE_POINT_NONE;
}
''',
    "crates/falign/src/kernel_count.wgsl": '''@compute @workgroup_size(1)
fn main() {
    _ = COUNT_LIMIT;
}
''',
}
ALIGNMENT_ENTRY_FILE = "crates/falign/src/lib.rs"

PYTHON_FIXTURE = {
    "pkg/a.py": '''"""模块文档串"""
import importlib.util
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("b_module", os.path.join(HERE, "b.py"))
b_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b_module)


def helper():
    """helper 的文档串"""
    # 只是注释
    return 1


def entry():
    return helper() + b_module.value()


def described():
    return helper.__doc__


def dynamic(name):
    return getattr(b_module, name)()


def launch():
    return subprocess.run([sys.executable, os.path.join(HERE, "c.py")], check=False)


class Reader:
    def read(self):
        return 1


class Writer:
    def read(self):
        return 2


def use_reader():
    return Reader().read()


if __name__ == "__main__":
    print(entry())
''',
    "pkg/b.py": '''def value():
    return 2


def unused():
    return 3
''',
    "pkg/c.py": '''def main():
    return 5


if __name__ == "__main__":
    main()
''',
    "pkg/d.py": '''import os
import runpy
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def handle_alpha():
    return 11


def handle_beta():
    return 12


HANDLERS = {"alpha": handle_alpha, "beta": handle_beta}


def dispatch(name):
    return HANDLERS[name]()


def run_copy(tree):
    return runpy.run_path(os.path.join(tree, "pkg", "c.py"), run_name="__main__")


def write_data(tree):
    target = os.path.join(tree, "pkg", "out.py")
    with open(target, "w", encoding="utf-8") as handle:
        handle.write("data")
    return subprocess.run([sys.executable, os.path.join(HERE, "c.py"), target], check=False)


def common_directory():
    return subprocess.run(["git", "-C", HERE, "rev-parse", "--path-format=absolute", "--git-common-dir"], capture_output=True, text=True).stdout.strip()


def main_repository_value():
    sys.path.insert(0, os.path.join(os.path.dirname(common_directory()), "pkg"))
    import b
    return b.value()
''',
    "pkg/constants.py": '''TOOLS_RELATIVE = "pkg/tools"
''',
    "pkg/g.py": '''import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
CACHE = {}


def cached_b():
    path = os.path.join(HERE, "b.py")
    if path not in CACHE:
        specification = importlib.util.spec_from_file_location("cached_b", path)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        CACHE[path] = module
    return CACHE[path].value()
''',
    "pkg/k.py": '''import os
import subprocess
import sys


def run_tool_named(root, name):
    return subprocess.run([sys.executable, os.path.join(root, "pkg", "tools", name)], check=False)
''',
    "pkg/j.py": '''import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
MODULE_PATHS = {"b": os.path.join(HERE, "b.py"), "c": os.path.join(HERE, "c.py")}


def load_named(name):
    specification = importlib.util.spec_from_file_location(name, MODULE_PATHS[name])
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module.value()
''',
    "pkg/h.py": '''import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STOP_SCRIPT = os.environ.get("FIXTURE_STOP_SCRIPT") or os.path.join(HERE, "c.py")
STUB = "print('stub')\\n"


def stop():
    return subprocess.run([sys.executable, STOP_SCRIPT, "stop"], check=False)


def run_stub():
    stub = os.path.join(os.environ["STUB_DIRECTORY"], os.environ["STUB_NAME"])
    with open(stub, "w") as handle:
        handle.write(STUB)
    return subprocess.run([sys.executable, stub], check=False)
''',
    "pkg/f.py": '''import os


def patched_os_function(name, replacement):
    original = getattr(os, name)
    setattr(os, name, replacement)
    return original
''',
    "pkg/e.py": '''import os
import sys

import constants


def load_tool(repository):
    sys.path.insert(0, os.path.join(repository, constants.TOOLS_RELATIVE))
    import tool
    return tool.run()
''',
    "pkg/tools/tool.py": '''def run():
    return 7
''',
    "pkg/more_tools/tool.py": '''def run():
    return 8
''',
    "pkg/tools/main_tool.py": '''def main():
    return 9


if __name__ == "__main__":
    main()
''',
    "pkg/i.py": '''import importlib
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOL_DIRECTORIES = (os.path.join(HERE, "tools"), os.path.join(HERE, "more_tools"))
TOOL_MODULE = "tool"


def load_named(name):
    for directory in TOOL_DIRECTORIES:
        if os.path.basename(directory) == name:
            sys.path.insert(0, directory)
            break
    import tool
    return tool.run()


def load_by_constant(name):
    for directory in TOOL_DIRECTORIES:
        if os.path.basename(directory) == name:
            sys.path.insert(0, directory)
    return importlib.import_module(TOOL_MODULE).run()


def load_each():
    values = []
    for name in MODULE_FILES:
        specification = importlib.util.spec_from_file_location("each", os.path.join(HERE, name))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        values.append(module.value())
    return ",".join(name for name, _code in [("x", 1)])


MODULE_FILES = ("b.py",)
''',
    "pkg/m.py": '''import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def run_inline():
    return subprocess.run([sys.executable, "-c", "import sys; sys.path.insert(0, sys.argv[1]); import b; b.value()", HERE], check=False)


def run_dynamic(code):
    return subprocess.run([sys.executable, "-c", code], check=False)


def run_unknown(interpreter):
    return subprocess.run([interpreter, os.path.join(HERE, "c.py")], check=False)


def run_dynamic_from_environment():
    return run_dynamic(os.environ["SNIPPET"])


def run_unknown_from_environment():
    return run_unknown(os.environ["INTERPRETER"])


def launch_with(extra):
    return subprocess.run([sys.executable] + extra, check=False)


def launch_tool():
    return launch_with([os.path.join(HERE, "c.py")])


def launch_job(job):
    return subprocess.run(job.command, check=False)


def run_tests():
    return subprocess.run([sys.executable, "-m", "unittest", "test_sample"], cwd=HERE, check=False)


def version_of(name, toolchain):
    found = subprocess.run([shutil.which(name), "--version"], check=False)
    return found, subprocess.run([os.path.join(toolchain, "bin", "llvm-cov"), "--version"], check=False)


def run_with_options(options):
    return subprocess.run([*options["launcher"], os.path.join(HERE, "c.py")], check=False)


def run_default():
    return run_with_options({"launcher": ["python3"]})


def launch_listed(command):
    return subprocess.run(command, check=False)


LAUNCHERS = {"listed": launch_listed}


def launch_from_table():
    return LAUNCHERS["listed"]([sys.executable, os.path.join(HERE, "c.py")])
''',
    "pkg/test_sample.py": '''import unittest


class SampleTest(unittest.TestCase):
    def test_one(self):
        return 3
''',
    ".claude/singlefs-ai-sop/scripts/preflight.py": '''import subprocess


def run_check(declaration):
    return subprocess.run(["bash", "-c", declaration.command], check=False)
''',
}

SHELL_FIXTURE = {
    "s/main.sh": '''#!/usr/bin/env bash
# admission: always 样本
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$here/lib.sh"
emit() {
  cat <<'EOF'
first
    };
second
EOF
}
cell_alpha() { echo alpha; }
cell_beta() { echo beta; }
LIBRARY_TEXT="$(cat <<'PY'
print("library")
PY
)"
use_library() { printf '%s\\n' "$LIBRARY_TEXT"; }
run_cell() { "$1"; }
run_cell cell_alpha
bash "$here/child.sh"
''',
    "s/lib.sh": '''#!/usr/bin/env bash
lib_function() { echo lib; }
''',
    "s/child.sh": '''#!/usr/bin/env bash
echo child
''',
    "s/dynamic.sh": '''#!/usr/bin/env bash
x="$1"
bash "$x"
''',
    "s/broken.sh": '''#!/usr/bin/env bash
if then fi (
''',
    "s/tools.sh": '''#!/usr/bin/env bash
build() {
  cargo build
}
build
''',
    "s/copies.sh": '''#!/usr/bin/env bash
tree="$1"
bash "$tree/s/child.sh"
''',
    "s/versions.sh": '''#!/usr/bin/env bash
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
run_in() {
  local directory
  case "$1" in
    alpha) directory="$here/alpha" ;;
    beta) directory="$here/beta" ;;
  esac
  (cd "$directory" && bash tool.sh)
}
run_in alpha
run_in beta
''',
    "s/alpha/tool.sh": '''#!/usr/bin/env bash
echo alpha tool
''',
    "s/beta/tool.sh": '''#!/usr/bin/env bash
echo beta tool
''',
    "s/nested.sh": '''#!/usr/bin/env bash
outer() {
  local helper
  helper="$(dirname "${BASH_SOURCE[0]}")/child.sh"
  inner() { bash "$helper"; }
  inner
}
outer
''',
    "s/selfcopy.sh": '''#!/usr/bin/env bash
copy="$1/run.sh"
cp "$(dirname "${BASH_SOURCE[0]}")/child.sh" "$copy"
bash "$copy"
''',
    "s/common.sh": '''#!/usr/bin/env bash
if common="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then
  source "${common%/*}/s/lib.sh"
fi
lib_function
''',
    "s/redirect.sh": '''#!/usr/bin/env bash
out="$1"
grep -v '^#' "$2" > "$out/filtered.sh"
''',
    "s/arguments.sh": '''#!/usr/bin/env bash
write_stage() { printf '%s\\n' "$2" | tee "$1" >/dev/null; }
write_stage "$1/stage.sh" 'echo stage'
''',
    "s/generator.sh": '''#!/usr/bin/env bash
out="$1"
cat > "$out/run.sh" <<'RUN'
bash s/child.sh
RUN
bash "$out/run.sh"
''',
    "s/inline.sh": '''#!/usr/bin/env bash
tree="$1"
bash -c 'source "$1"; lib_function' inline-case "$tree/s/lib.sh"
''',
    "s/py/helper_mod.py": '''def run():
    return 1
''',
    "s/py/tool.py": '''def main():
    return 2


if __name__ == "__main__":
    main()
''',
    "s/inline_python.sh": '''#!/usr/bin/env bash
python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import helper_mod; helper_mod.run()' "$(dirname "${BASH_SOURCE[0]}")/py"
''',
    "s/heredoc_in_if.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
if python3 - "$here/py" <<'PY'
import sys
sys.path.insert(0, sys.argv[1])
import helper_mod
helper_mod.run()
PY
then
  bash s/child.sh
  echo done
fi
''',
    "s/heredoc_python.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
python3 - "$here/py" <<'PY'
import sys
sys.path.insert(0, sys.argv[1])
import helper_mod
helper_mod.run()
PY
''',
    "s/inline_expanded.sh": '''#!/usr/bin/env bash
code="$1"
python3 -c "$code"
bash -c "$2"
''',
    "s/inline_bash_folded.sh": '''#!/usr/bin/env bash
step="bash $(dirname "${BASH_SOURCE[0]}")/child.sh"
bash -c "$step"
''',
    "s/second.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
bash "$here/lib.sh" 2 python3 -B "$here/py/tool.py"
''',
    "s/substitution.sh": '''#!/usr/bin/env bash
check_output="$(bash "$(dirname "${BASH_SOURCE[0]}")/child.sh" 2>&1)" || exit 1
printf '%s\\n' "$check_output"
''',
    "s/process_substitution.sh": '''#!/usr/bin/env bash
file="$(pick_file)"
mapfile -t lines < <(read_lines "$file")
''',
    "s/message.sh": '''#!/usr/bin/env bash
die() { echo "$1" >&2; exit 1; }
command -v true >/dev/null || die "qemu-system-x86_64 缺失" "装上 qemu-system-x86 再跑"
[[ -n "${1:-}" ]] || die "缺 qemu-system-x86_64" "装上 qemu-system-x86 再跑"
''',
    "s/module.sh": '''#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/py" && python3 -m tool
''',
    "s/head.sh": '''#!/usr/bin/env bash
runner="$(find_runner)"
"$runner" --flag
''',
    "s/wrapped.sh": '''#!/usr/bin/env bash
timeout 60 "$(dirname "${BASH_SOURCE[0]}")/child.sh"
''',
    "s/subscript.sh": '''#!/usr/bin/env bash
declare -A item
key="$1" address="$2" value="$3"
item[$key $address]="$value"
item[$key]+=" more"
echo "${item[$key $address]}"
''',
    "s/structure.sh": '''#!/usr/bin/env bash
x="$(pick_x)" y="$(pick_y)"
if [[ -n "$x" || "$y" == ok ]]; then echo yes; fi
case "$1" in
  "$x") echo first ;;
  *) echo other ;;
esac
pair=( "$x" "$y" )
(( $# > 0 )) && echo "${pair[0]}"
''',
    "s/array.sh": '''#!/usr/bin/env bash
command_words=(bash "$(dirname "${BASH_SOURCE[0]}")/child.sh")
"${command_words[@]}"
''',
    "s/capture.sh": '''#!/usr/bin/env bash
code_of() {
  cat <<'PY'
import sys
sys.path.insert(0, sys.argv[1])
import helper_mod
helper_mod.run()
PY
}
code="$(code_of)"
python3 -c "$code" "$(dirname "${BASH_SOURCE[0]}")/py"
''',
    "s/nested_inline_positionals.sh": '''#!/usr/bin/env bash
code_of() {
  printf '%s\\n' "import sys" "sys.path.insert(0, sys.argv[1])" "import helper_mod" "helper_mod.run()"
}
code="$(code_of)"
here="$(dirname "${BASH_SOURCE[0]}")"
bash -c 'bash -c "python3 -c \\"\\$1\\" \\"\\$2\\"" inner "$2" "$1"' outer "$here/py" "$code"
''',
    "s/declared.sh": '''#!/usr/bin/env bash
# run-condition: check bash "$PREFLIGHT_SCRIPT_DIRECTORY/child.sh" :: 先让同目录的 child.sh 跑通
echo declared
''',
    "s/argv_exec.sh": '''#!/usr/bin/env bash
extra=(SAMPLE=1)
python3 -c 'import os, sys; os.execvp(sys.argv[1], sys.argv[1:])' env "${extra[@]}" bash "$(dirname "${BASH_SOURCE[0]}")/child.sh"
''',
    "s/positional_tool.sh": '''#!/usr/bin/env bash
tool_path="$1"
python3 - "$tool_path" <<'PY'
import subprocess
import sys
tool = sys.argv[1]
subprocess.run([tool, "--version"], check=False)
PY
''',
    "s/argv_slice.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
watch_helper() {
  exec python3 -B /dev/fd/3 "$here/py" "$1" 3<<'PY'
import sys
(directory, label) = sys.argv[1:3]
sys.path.insert(0, directory)
import helper_mod
helper_mod.run()
PY
}
watch_helper sample
''',
    "s/argv_slice_mismatched.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
watch_helper() {
  exec python3 -B /dev/fd/3 "$here/py" "$1" 3<<'PY'
import sys
(directory, label) = sys.argv[1:4]
sys.path.insert(0, directory)
import helper_mod
helper_mod.run()
PY
}
watch_helper sample
''',
    "s/group.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
{
  python3 /dev/fd/3 "$here/py"
} 3<<'PY'
import sys
sys.path.insert(0, sys.argv[1])
import helper_mod
helper_mod.run()
PY
''',
}

# 起子进程的几种折法与运行期读的数据文件的夹具（文件头「起代码的写法逐种认」python 的命令、「运行期读的数据文件」）：每个函数对一格
FOLDS_FIXTURE = {
    "pkg/folds.py": '''import concurrent.futures
import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile

import rows

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
EDGE = ("pkg/c.py", "plan", "pkg/b.py")
TABLE = os.path.join(HERE, "table.tsv")


def nested_launch():
    script = os.path.join(HERE, "c.py")

    def run(extra):
        command = [sys.executable, script] + extra
        return subprocess.run(command, check=False)
    return run(["--quiet"])


def concatenated(steps):
    git = ["git", "-C", HERE]
    for step in steps:
        subprocess.run(git + step, check=False)


def concatenated_caller():
    return concatenated([["status"], ["log"]])


def launch_plan():
    return 4, [sys.executable, os.path.join(HERE, "c.py"), "--", "marker", "test"]


def sliced_launch():
    command = launch_plan()[1]
    launched = command[:command.index("marker")] + ["--flag"]
    return subprocess.run(launched, check=False)


def interpreter_lines(language):
    return {"python": ["python3"], "shell": ["bash"]}.get(language, [])


class Versions:
    def interpreter(self, name):
        return subprocess.run([name, "--version"], check=False)

    def every(self, language):
        return [self.interpreter(name) for name in interpreter_lines(language)]


def loaded_constant():
    path = os.path.join(os.path.dirname(HERE), EDGE[2])
    specification = importlib.util.spec_from_file_location("loaded_b", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return subprocess.run(["sh", "-c", module.COMMAND], check=False)


def run_from_copy(tree):
    return subprocess.run([sys.executable, os.path.join(tree, "tools", "runner.py")], check=False)


def run_generated():
    return subprocess.run([sys.executable, os.path.join(os.environ["GENERATED_DIRECTORY"], "generated", "made.py")], check=False)


def run_in_copy(copy):
    return subprocess.run(["bash", "setup.sh"], cwd=copy, check=False)


def submitted_run(command):
    return subprocess.run(["bash", "-c", command], check=False)


def submit_all():
    with concurrent.futures.ThreadPoolExecutor() as pool:
        return pool.submit(submitted_run, "python3 pkg/c.py").result()


def environment_launch(threads):
    return subprocess.run(["env", "-u", "UNSET_ME", f"WORKERS={threads}", sys.executable, os.path.join(HERE, "c.py")], check=False)


def table_rows():
    with open(TABLE, encoding="utf-8") as handle:
        return handle.read().splitlines()


def rows_under(root):
    return open(os.path.join(root, "pkg", "table.tsv"), encoding="utf-8").read()


def rows_from_argument():
    return rows_under(sys.argv[1])


def scratch_rows():
    work = tempfile.mkdtemp()
    with open(os.path.join(work, "notes.txt"), "w") as handle:
        handle.write("x")
    return open(os.path.join(work, "notes.txt")).read()


def listed_rows():
    with open(os.path.join(HERE, "index.txt"), encoding="utf-8") as listing:
        return [open(os.path.join(ROOT, line.strip())).read() for line in listing]


def template_rows():
    chosen = sorted(os.listdir(os.path.join(HERE, "templates")))[0]
    return open(os.path.join(HERE, "templates", chosen)).read()


def template_names(directory):
    return sorted(name for name in os.listdir(directory) if name.endswith(".txt"))


def template_hits(directory, wanted):
    hits = []
    for name in template_names(directory):
        with open(os.path.join(directory, name)) as handle:
            if wanted in handle.read():
                hits.append(name)
    return hits


def templates_through_a_listing_function():
    return template_hits(os.path.join(HERE, "templates"), "second")


def template_named_in_a_file():
    with open(os.path.join(HERE, "index.txt"), encoding="utf-8") as listing:
        name = listing.read().strip()
    return open(os.path.join(HERE, "templates", name)).read()


def rows_through_library(request):
    return rows.read_rows(TABLE) + rows.read_rows(request)


class TableReader:
    def __init__(self):
        self.path = TABLE

    def lines(self):
        with open(self.path, encoding="utf-8") as handle:
            return handle.read().splitlines()


def compiler_version():
    program = os.environ["RUSTC"]
    return subprocess.run([program, "-V"], capture_output=True, check=False).stdout


def judge_file(path):
    return open(path).read().strip()


def parsed_rows():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--table")
    arguments = parser.parse_args()
    return open(arguments.table).read()


def listed_members(units):
    members = {line.split("\\t")[0] for line in units} | {"pkg/table.tsv"}
    return [open(os.path.join(ROOT, relative)).read() for relative in sorted(members)]


def copy_listed(work):
    with open(os.path.join(HERE, "index.txt"), encoding="utf-8") as listing:
        for line in listing:
            shutil.copyfile(os.path.join(ROOT, line.strip()), os.path.join(work, line.strip()))
''',
    "pkg/b.py": '''COMMAND = "python3 pkg/c.py"


def value():
    return 2
''',
    "pkg/c.py": '''def main():
    return 5


if __name__ == "__main__":
    main()
''',
    "pkg/tools/runner.py": '''def run():
    return 7


if __name__ == "__main__":
    run()
''',
    "pkg/sample/setup.sh": "#!/usr/bin/env bash\necho prepared\n",
    "pkg/table.tsv": "alpha\t1\n",
    "pkg/other.tsv": "gamma\t2\n",
    "pkg/index.txt": "pkg/table.tsv\n",
    "pkg/templates/first.txt": "first\n",
    "pkg/templates/second.txt": "second\n",
    "pkg/rows.py": """def read_rows(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read().splitlines()
""",
}

SHELL_ARGUMENTS_EXIT_CELL = "shell：bash s/dispatch_exit.sh --only：走到的那一支以 exit 收尾，fi 之后的 s/tools/two.py 不连（改它键不变）"
SHELL_ARGUMENTS_EXIT_TAKEN_CELL = "shell：bash s/dispatch_exit.sh --only：走到的那一支里的 s/child.sh 连上（改它键变）"
PYTHON_SHELL_ARGUMENTS_CELL = "python：[\"bash\", <脚本>, *<调用处给的字面列表>] 起 shell：脚本体照字面参数剪（改走不到的 s/child.sh 键不变）"
SHELL_NESTED_WORD_CELL = "shell：外面的单元里一个词与套在别的函数里的函数同名：不连（改它起的 s/child.sh 键不变）"
SHELL_NESTED_INSIDE_CELL = "shell：外层函数里调套着的函数：连上（改它起的 s/child.sh 键变）"
SHELL_NESTED_OUTSIDE_COMMAND_CELL = "shell：外面的单元在命令位置上调套在别的函数里的函数：连上（改它起的 s/child.sh 键变）"

SHELL_PYTHON_ARGUMENTS_TAKEN_CELL = "shell：python3 <脚本> alpha \"$1\"：分派表里 alpha 那一项走到（改它键变）"

PYTHON_LAUNCH_WITHOUT_ENTRY_ARGUMENTS_CELL = "python：入口没带参数，它起的 python 带字面参数：被起的那份照它们剪（改走不到的函数键不变）"

SHELL_TABLE_UNSELECTED_CELL = "shell：bash <复跑表脚本> E1：表里别的实验的驱动函数走不到，它起的 s/two.sh 不连（改它键不变）"
SHELL_TABLE_SELECTED_CELL = "shell：bash <复跑表脚本> E1：表里这个实验的驱动函数连上，它起的 s/one.sh 改了键变"
SHELL_TABLE_REAL_CELL = ("真仓：replay.sh 的表还是按第一列选行的写法（TABLE、ADMISSION_ROW_KEYS 两张 here-doc，参数进 ONLY、want 按实验号挑行；"
                         "--item 后面的词是行项，行序号按「实验号|二进制|参数」数第一列以 E 起头的行，同 table_rows_selected）")
SHELL_TABLE_MIXED_CELL = "shell：同一个闭包里 <复跑表脚本> 既按实验号起、又有一处起法折不出：读表连整张，别的实验的驱动照连（改 s/two.sh 键变）"
SHELL_TABLE_ENTRY_CELL = "shell：入口就是 <复跑表脚本>：读表连整张（改 s/two.sh 键变）"
# 带 --item 起复跑表脚本（文件头「shell 起法带字面参数」）：行项 <实验号>.<行序号> 认到复跑表里那一行；E1 两行、E2 一行，各行的驱动各起一份自己的脚本，
# 准入键表里 E1 那一行抄的是 E1.2 那一行的前三列
SHELL_TABLE_ITEM_FIXTURE = {
    "scripts/experiment-apparatus/replay.sh": ("#!/usr/bin/env bash\ndriver_one_grid() {\n  bash s/one_grid.sh\n}\ndriver_one_sweep() {\n  bash s/one_sweep.sh\n}\n"
                                               "driver_two() {\n  bash s/two.sh\n}\n"
                                               "TABLE=$(cat <<'TSV'\nE1|@driver_one_grid||one-grid.out|exact\nE1|@driver_one_sweep||one-sweep.out|exact\n"
                                               "E2|@driver_two||two.out|exact\nTSV\n)\n"
                                               "ADMISSION_ROW_KEYS=$(cat <<'TSV'\nE1/sweep\tE1|@driver_one_sweep|\treplay\nTSV\n)\n"
                                               "printf '%s\\n' \"$TABLE\" \"$ADMISSION_ROW_KEYS\"\n"),
    "s/one_grid.sh": "#!/usr/bin/env bash\necho one-grid\n",
    "s/one_sweep.sh": "#!/usr/bin/env bash\necho one-sweep\n",
    "s/two.sh": "#!/usr/bin/env bash\necho two\n",
}
SHELL_TABLE_ITEM_CELL = ("shell 入口是 <复跑表脚本> 带 --item E1.2：E1.2 那一行的驱动连上（改 s/one_sweep.sh 键变），"
                         "E1.1 那一行与 E2 的驱动走不到（改 s/one_grid.sh、s/two.sh 键不变）")

SHELL_TABLE_TREE_COPY_CELL = ('shell：cd 到折不出的目录（仓副本）之后 bash <复跑表脚本> E1：'
                              '参数照记、照 E1 剪，表里别的实验的驱动函数走不到（改 s/two.sh 键不变），E1 那一行的连上（改 s/one.sh 键变）')

SHELL_ENTRY_UNDECIDED_CELL = "shell 入口带参数：if [[ \"${1:-}\" == \"$SELFTEST_WORD\" ]] 判不出真假，两支都收（改 then 那一支起的 s/child.sh 键变）"

# 拷进临时目录、表换成桩的副本（文件头「shell 起法带字面参数」表行之外写到的驱动函数）：写法同 scripts/gate/admission.py 的 run_replay_cells、replay_copy_text。
# 真表每行一个驱动，各起一份自己的脚本；表行之外的写法各占一个驱动：declare -A 的键、[[ == @<名> ]]、eval 里的定义、命令位置、放进变量。
SHELL_TABLE_COPY_RUNNER = "scripts/experiment-apparatus/replay.sh"
SHELL_TABLE_COPY_LAUNCHER = """import os
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
RUNNER = "scripts/experiment-apparatus/replay.sh"
TABLE_START = "TABLE=$(cat <<'TSV'\\n"
TABLE_END = "\\nTSV\\n)\\n"
STUB_DRIVER = "driver_stub() {\\n  echo stub\\n}\\n"
STUB_ROWS = ["E900|@driver_stub||e900.out|exact"]


def run_quietly(command):
    return subprocess.run(command, capture_output=True, text=True).returncode


def write_text(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def copy_text(rows):
    with open(os.path.join(HERE, "..", "experiment-apparatus", "replay.sh"), encoding="utf-8") as handle:
        text = handle.read()
    start = text.index(TABLE_START) + len(TABLE_START)
    end = text.index(TABLE_END, start)
    return text[:start] + "\\n".join(rows) + TABLE_END + STUB_DRIVER + text[end + len(TABLE_END):]


"""
SHELL_TABLE_COPY_WITH_EXPERIMENT = """def judge():
    work = tempfile.mkdtemp()

    def replay(experiment):
        write_text(os.path.join(work, RUNNER), copy_text(STUB_ROWS))
        return run_quietly(["bash", os.path.join(work, RUNNER), experiment])

    def replay_with_arguments(arguments):
        write_text(os.path.join(work, RUNNER), copy_text(STUB_ROWS))
        return run_quietly(["bash", os.path.join(work, RUNNER), *arguments])

    return replay("E900"), replay("E901"), replay_with_arguments(["E902"]), replay_with_arguments(["--admission-only", "E902"])
"""
SHELL_TABLE_COPY_WITHOUT_EXPERIMENT = """def judge():
    work = tempfile.mkdtemp()
    write_text(os.path.join(work, RUNNER), copy_text(STUB_ROWS))
    return run_quietly(["bash", os.path.join(work, RUNNER)])
"""
SHELL_TABLE_COPY_FIXTURE = {
    SHELL_TABLE_COPY_RUNNER: ("#!/usr/bin/env bash\n"
                              "driver_row_only() {\n  bash s/row_only.sh\n}\n"
                              "driver_deferred() {\n  bash s/deferred.sh\n}\n"
                              "driver_compared() {\n  bash s/compared.sh\n}\n"
                              "driver_replaced() {\n  bash s/replaced.sh\n}\n"
                              "driver_called() {\n  bash s/called.sh\n}\n"
                              "driver_plain_word() {\n  bash s/plain_word.sh\n}\n"
                              "driver_shared() {\n  bash s/shared.sh\n}\n"
                              "framework_helper() {\n  bash s/framework.sh\n}\n"
                              "framework_judge() {\n  [[ -n \"${DEFERRED_ROOT[\"$1\"]+set}\" ]] && framework_helper\n}\n"
                              "TABLE=$(cat <<'TSV'\n"
                              "E1|@driver_row_only||e1.out|exact\nE2|@driver_deferred||e2.out|exact\nE3|@driver_compared||e3.out|exact\n"
                              "E4|@driver_replaced||e4.out|exact\nE5|@driver_called||e5.out|exact\nE6|@driver_plain_word||e6.out|exact\n"
                              "E7|@driver_shared||e7.out|exact\nE8|@driver_shared||e8.out|exact\nTSV\n)\n"
                              "if [[ \"${REPLAY_BROKEN:-}\" == replaced ]]; then\n  eval 'driver_replaced() { (echo replaced); }'\nfi\n"
                              "declare -A DEFERRED_ROOT=(\n  [\"E2|@driver_deferred|\"]=root\n)\n"
                              "while IFS='|' read -r exp bin args _rest; do\n"
                              "  [[ \"$bin\" == @driver_compared ]] && echo compared\n"
                              "  framework_judge \"$exp|$bin|$args\"\n"
                              "done <<<\"$TABLE\"\n"
                              "driver_called\n"
                              "selected_step=driver_plain_word\n"
                              "printf '%s\\n' \"$selected_step\"\n"),
    "s/row_only.sh": "#!/usr/bin/env bash\necho row-only\n",
    "s/deferred.sh": "#!/usr/bin/env bash\necho deferred\n",
    "s/compared.sh": "#!/usr/bin/env bash\necho compared\n",
    "s/replaced.sh": "#!/usr/bin/env bash\necho replaced\n",
    "s/called.sh": "#!/usr/bin/env bash\necho called\n",
    "s/plain_word.sh": "#!/usr/bin/env bash\necho plain-word\n",
    "s/shared.sh": "#!/usr/bin/env bash\necho shared\n",
    "s/framework.sh": "#!/usr/bin/env bash\necho framework\n",
    "scripts/gate/copy_launcher.py": SHELL_TABLE_COPY_LAUNCHER + SHELL_TABLE_COPY_WITH_EXPERIMENT,
    "scripts/gate/copy_launcher_real_row.py": SHELL_TABLE_COPY_LAUNCHER + SHELL_TABLE_COPY_WITH_EXPERIMENT.replace('replay("E900")', 'replay("E7")'),
    "scripts/gate/copy_launcher_whole.py": SHELL_TABLE_COPY_LAUNCHER + SHELL_TABLE_COPY_WITHOUT_EXPERIMENT,
}
SHELL_TABLE_COPY_CELL = "python：带实验号起拷进临时目录、表换成桩的副本：只被真表点名的驱动函数不进闭包（表行之外写成 @<名>、<名>() 的也不连），副本里留下的框架函数与它读的变量进闭包"
SHELL_TABLE_COPY_FRAMEWORK_CELL = "python：带实验号起表换成桩的副本：副本里留下的框架函数连上（改它起的 s/framework.sh 键变）"
SHELL_TABLE_COPY_CALLED_CELL = "python：带实验号起表换成桩的副本：脚本体在命令位置上调的驱动函数照连（改它起的 s/called.sh 键变）"
SHELL_TABLE_COPY_PLAIN_WORD_CELL = "python：带实验号起表换成桩的副本：写成不带 @ 的词的驱动函数照连（改它起的 s/plain_word.sh 键变）"
SHELL_TABLE_COPY_WHOLE_CELL = "python：不带实验号起表换成桩的副本：认不出表换没换，真表里的驱动函数照连（改它起的 s/row_only.sh 键变）"
SHELL_TABLE_COPY_REAL_ROW_CELL = "python：带实验号起表换成桩的副本，那个实验号在真表里有行：那几行的驱动函数照连，别的实验那几行也写到它也算（改它起的 s/shared.sh 键变）"
SHELL_PWD_CELL = "shell：$PWD 照脚本体顶层那一处 cd 折（cd \"$(dirname \"$0\")/..\"），函数里 bash \"$PWD/<文件>\" 连到那一份"
SHELL_PWD_EXACT_CELL = "shell：$PWD 折出来只连折出的那一份，别处同名的不连（改它键不变）"
TREE_COPY_EXACT_CELL = "shell：`bash \"$tree/<仓里的相对路径>\"` 只连根里同一相对路径那一份，别处以这一截收尾的不连（改它键不变）"
SHELL_PATTERN_REMOVAL_CELL = "shell：${名%词} 与 ${名##词} 的词折得出就照 bash 去掉后缀、前缀，连到折出的那一份"
SHELL_DIRECTORY_COPY_CELL = "shell：cd \"$目录/<根里的目录>\" 之后的相对路径照根里那个目录认，连到那一份"
SHELL_RELATIVE_COPY_CELL = "shell：cd 到折不出的目录之后 bash <根里的相对路径>（至少两段）连到根里那一份"
SHELL_DIRECTORY_WILDCARD_CELL = "shell：python3 \"<根里的目录>/$1\" 连到那个目录里每一份 .py"
SHELL_POSITIONAL_SCRIPT_CELL = "shell：函数里 python3 \"$@\" 的脚本照每一处调用给的那个词认，连到那一份"
SHELL_RELATIVE_CD_CELL = "shell：cd <根里的相对目录> 之后 python3 <那个目录里的文件> 连到那一份（不报不在）"
SHELL_QUOTED_BRACE_CELL = "shell：函数里多行 awk 程序带一行与函数收尾同缩进的 }：函数照引号外的 } 收尾，awk 之后的 bash s/child.sh 连上（改它键变）"
SHELL_TAIL_GROUP_CELL = "shell：bash \"$<折不出的目录>/$<折得出的变量>/driver.sh\" 连到仓里以 .sample/driver.sh 收尾的那一组（改它键变）"
SHELL_TAIL_OUTSIDE_CELL = "shell：bash \"$<折不出的目录>/.nowhere/run.sh\" 仓里没有以它收尾的脚本：不是仓里的脚本，不拒算"

SHELL_ARGUMENTS_TAKEN_CELL = "shell：bash s/dispatch.sh run：case \"$1\" 里 run) 那一支走到，它起的 s/tools/one.py 连上（改它键变）"
SHELL_ARGUMENTS_OTHER_CASE_CELL = "shell：bash s/dispatch.sh run：case \"$1\" 里 *) 那一支走不到，它起的 s/tools/two.py 不连（改它键不变）"

OPEN_TAIL_TAKEN_CELL = "shell：bash s/dispatch.sh run \"$@\"：case 里 run) 那一支走到（改 s/tools/one.py 键变）"

DEFERRED_HEREDOC_BODY_CELL = "shell：if python3 - <<'PY' 的 here-doc 照内联脚本切（declare -f 把正文打在 then 那一串头一条命令之后），连到它 import 的仓内模块"
DEFERRED_HEREDOC_THEN_CELL = "shell：if python3 - <<'PY' 之后 then 那一串的头一条命令不算 here-doc 正文，它起的 s/child.sh 连上（改它键变）"
TEXT_PIECES_EXECUTABLE_CELL = "python：bash -c f'\"{sys.executable}\" <脚本>' 折得出，连到那份脚本（改它键变）"
TEXT_PIECES_INTEGER_CELL = "python：bash -c 的串里 f 串的 {<整数常量>} 与 \" \".join(<字面元组>) 折得出，串里起的脚本连上（改它键变）"
TEXT_PIECES_FIXTURE = {
    "pkg/launch.py": ('import subprocess\nimport sys\n\nEXIT_CODE = 4\nNAMES = ("one", "two")\n\n\n'
                      'def by_executable():\n    return subprocess.run(["bash", "-c", f\'"{sys.executable}" pkg/child.py; echo done\'], check=False)\n\n\n'
                      'def by_integer():\n    text = (\'bash pkg/child.sh || exit 1; for name in \' + " ".join(NAMES)\n'
                      '            + f\'; do echo "$name"; done; exit {EXIT_CODE}\')\n'
                      '    return subprocess.run(["bash", "-c", text], check=False)\n'),
    "pkg/child.py": 'def child():\n    return 1\n\n\nif __name__ == "__main__":\n    child()\n',
    "pkg/child.sh": "#!/usr/bin/env bash\necho child\n",
}
PROGRAM_WORDS_CELL = "不拒算的程序词（夹具）：登记了的 shell 词与 python 词不拒算，同一份文件里没登记的一处照拒算，不登记时都拒算"
REGISTERED_WORDS_REAL_CELL = "真仓：UNKEYED_PROGRAM_WORDS、REGISTRY_COMMAND_TEXTS、SPEC_PATH_ARGUMENTS、TAIL_READ_SCOPES 登记的文件、函数、词与点名的文件都还在"
# 夹具：seam.sh 与 launch.py 各有一处自证期间登记的词、一处不登记的词
PROGRAM_WORDS_FIXTURE = {
    "tool/seam.sh": '#!/usr/bin/env bash\nrun_seam() {\n  "$SEAM_PROGRAM" --flag\n}\nrun_other() {\n  "$OTHER_PROGRAM" --flag\n}\n',
    "tool/launch.py": ('import os\nimport subprocess\n\n\ndef launch_seam():\n    return subprocess.run(["bash", "-c", os.environ["SEAM_TEXT"]], check=False)\n\n\n'
                       'def launch_other():\n    return subprocess.run(["bash", "-c", os.environ["OTHER_TEXT"]], check=False)\n'),
}
PROGRAM_WORDS_FIXTURE_ROWS = {("tool/seam.sh", "run_seam", "$SEAM_PROGRAM"): "夹具", ("tool/launch.py", "launch_seam", "os.environ['SEAM_TEXT']"): "夹具"}
# 外层函数里套一个内层函数（文件头「shell 里套在函数里的函数」外层只认自己那一层）：内层起一个折不出的程序、再起 tool/child.sh，外层只在命令位置上调内层
NESTED_PROGRAM_WORDS_FIXTURE = {
    "tool/nested_seam.sh": ('#!/usr/bin/env bash\nrun_cells() {\n  run_inner_seam() {\n    "$NESTED_SEAM_PROGRAM" --flag\n    bash tool/child.sh\n  }\n'
                            '  run_inner_seam\n}\n'),
    "tool/child.sh": "#!/usr/bin/env bash\necho child\n",
}
NESTED_PROGRAM_WORDS_INNER_ROWS = {("tool/nested_seam.sh", "run_inner_seam", "$NESTED_SEAM_PROGRAM"): "夹具"}
# 别的程序换掉的路径模板（{tree} 是那一边拷出来的树）：折得出、指到不在的文件；按函数短名登了就不拒算，同一个函数里起的别的脚本照连
REGISTERED_MISSING_PATH_FIXTURE = {
    "tool/template_launch.sh": '#!/usr/bin/env bash\nrun_cells() {\n  bash "{tree}/tool/template_launch.sh" --only checks\n  bash tool/child.sh\n}\nrun_cells\n',
    "tool/child.sh": "#!/usr/bin/env bash\necho child\n",
}
REGISTERED_MISSING_PATH_ROWS = {("tool/template_launch.sh", "run_cells", "{tree}/tool/template_launch.sh"): "夹具"}
REGISTERED_MISSING_PATH_CELL = ('shell：登了 UNKEYED_PROGRAM_WORDS 的路径词 "{tree}/…" 折得出、指到不在的文件：'
                                '不拒算，同一个函数里别的起法照连（改 tool/child.sh 键变）；不登记照拒算 shell-path-missing')
NESTED_PROGRAM_WORDS_OUTER_ROWS = {("tool/nested_seam.sh", "run_cells", "$NESTED_SEAM_PROGRAM"): "夹具"}
NESTED_PROGRAM_WORDS_REGISTERED_CELL = ("shell：外层函数里套着的函数起折不出的程序、按内层的短名登了 UNKEYED_PROGRAM_WORDS：外层只认自己那一层，"
                                        "外层当入口不拒算，内层起的 tool/child.sh 连上（改它键变）")
NESTED_PROGRAM_WORDS_UNREGISTERED_CELL = ("shell：外层函数里套着的函数起折不出的程序、没按内层的短名登记：拒算落在内层名下"
                                          "（内层当入口拒算，外层当入口照外层的短名登记了也照拒算）")
REGISTRY_COMMAND_TEXTS_CELL = "python：bash -c 的串来自登记表：登记表那一列的命令照内联 bash 认，改它起的脚本键变"
REGISTRY_COMMAND_TABLE_CELL = "python：bash -c 的串来自登记表：登记表收成数据单元，改表里一行键变"
# 照真仓的样子：门禁入口 source 格函数库，内联 python（登记在 REGISTRY_COMMAND_TEXTS 那一行的文件上）写在库里的格函数里
REGISTRY_COMMAND_FIXTURE = {
    ".claude/gate.d/doc-registries.sh": ('#!/usr/bin/env bash\nSTAGE_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"\n'
                                         'source "$STAGE_DIRECTORY/lib/doc-registries-reference-and-invariant-cells.sh"\ncheck_counted_claims\n'),
    ".claude/gate.d/lib/doc-registries-reference-and-invariant-cells.sh": (
        "check_counted_claims() {\n  python3 - <<'PY'\nimport subprocess\n\n\ndef counting_command_result(command):\n"
        "    return subprocess.run([\"bash\", \"-o\", \"pipefail\", \"-c\", command], check=False)\n\n\n"
        "with open('.claude/gate.d/counted-claims.tsv', encoding='utf-8') as handle:\n    for row in handle:\n        counting_command_result(row)\nPY\n}\n"),
    ".claude/gate.d/counted-claims.tsv": "# 注释那一行\nREADME.md\t(1)\tbash s/child.sh | wc -l\t说明\n",
    "s/child.sh": "#!/usr/bin/env bash\necho child\n",
}
SPEC_PATH_ARGUMENTS_CELL = "python：spec_from_file_location 的路径折不出而登记了实际载入的文件：不拒算、连到那一份（改它键变）"
SPEC_PATH_FIXTURE = {
    "tool/check.py": ('import importlib.util\nimport sys\n\nanchors_path = sys.argv[1]\n\n\ndef load():\n'
                      '    specification = importlib.util.spec_from_file_location("anchors", anchors_path)\n'
                      '    module = importlib.util.module_from_spec(specification)\n    specification.loader.exec_module(module)\n    return module\n'),
    "tool/anchors.py": "VALUE = 1\n",
    "tool/anchors_other.py": "VALUE = 3\n",
}
TAIL_READ_SCOPES_CELL = "python：按字面尾巴读而登记了前缀（TAIL_READ_SCOPES）：前缀外多一份同名文件键不变、前缀里改一份键变；登记成调用方输入的那一处一份都不收"
TAIL_READ_SCOPES_REAL_CELL = "真仓：TAIL_READ_SCOPES 每一行那一处读还在、算键时问到的就是这一行，收进来的文件都在登记的前缀下"
# 两处按尾巴读 Manifest.toml：scan 走 members 底下（os.walk 的目录变量），climb 从参数给的目录往上找第一份（经 parent 换一手，推不到参数上）；
# rows 读 <参数给的树>/tables/rows.tsv（tree_copy_file 那一种：照根里同一相对路径那一份收）
TAIL_READ_SCOPES_FIXTURE = {
    "members/one/Manifest.toml": "name = one\n",
    "elsewhere/Manifest.toml": "name = elsewhere\n",
    "pkg/scoped.py": '''import os
import sys

ROOTS = ("members",)


def member_texts(repository):
    texts = []
    for root in ROOTS:
        for directory, _subdirectories, files in os.walk(os.path.join(repository, root)):
            if "Manifest.toml" in files:
                with open(os.path.join(directory, "Manifest.toml"), encoding="utf-8") as handle:
                    texts.append(handle.read())
    return texts


def nearest(start):
    current = os.path.abspath(start)
    while True:
        manifest = os.path.join(current, "Manifest.toml")
        if os.path.isfile(manifest):
            with open(manifest, encoding="utf-8") as handle:
                return handle.read()
        parent = os.path.dirname(current)
        if parent == current:
            return None
        current = parent


def scan():
    return member_texts(sys.argv[1])


def climb():
    return nearest(sys.argv[1])


def table_rows(tree):
    with open(os.path.join(tree, "tables", "rows.tsv"), encoding="utf-8") as handle:
        return handle.read()


def rows():
    return table_rows(sys.argv[1])
''',
    "tables/rows.tsv": "row\n",
}
TAIL_READ_ROOT_SCOPE_CELL = "python：按字面尾巴读而前缀登了 .（TAIL_READ_SCOPES 里被判那棵树根上的那一份）：根上那一份改了键变，research/ 下多一份、改别处同名的一份键不变；登空元组时改根上那一份键不变"
# root_manifest 读 <参数给的树>/Manifest.toml（head 交 sys.argv[1]）：照尾巴收的是根上的、members/ 底下的与 research/ 下后放进来的那几份
TAIL_READ_ROOT_SCOPE_FIXTURE = {
    "Manifest.toml": "name = root\n",
    "members/one/Manifest.toml": "name = one\n",
    "pkg/rooted.py": '''import os
import sys


def root_manifest(tree):
    with open(os.path.join(tree, "Manifest.toml"), encoding="utf-8") as handle:
        return handle.read()


def head():
    return root_manifest(sys.argv[1])
''',
}
GETATTR_NAMES_CELL = "python：getattr(<对象>, <推导式变量>)，变量取遍模块级字典字面量的值里的字面名：不拒算"
GETATTR_NAMES_FIXTURE = {
    "pkg/options.py": ('TABLE = {"keys": ("copy", "build"), "plan": ("copy",)}\n\n\n'
                       'def missing(holder, command):\n    return [name for name in TABLE[command] if getattr(holder, name) is None]\n'),
}
FUNCTION_PRINTED_WORD_CELL = "shell：函数里 python3 \"$@\"，调用处的脚本是 \"$(只打一个词的函数 <字面词>)/<文件>\"：照函数打的词折，连到那一份（改它键变）"
QUOTED_ASSIGNMENT_ENV_CELL = "shell：env \"X=$y\" nice -n 19 bash <脚本>：带引号的赋值不算命令词，连到那份脚本（改它键变）"
QUOTED_ASSIGNMENT_ARRAY_CELL = "shell：env ${e[@]+\"${e[@]}\"} bash <脚本>，e=(\"X=$(f)\")：数组里的赋值不算命令词，连到那份脚本（改它键变）"
FOLD_NESTED_CELL = "python：套在函数里的函数起子进程，它的局部变量与外层的变量照那一层作用域折，连到那份脚本"
FOLD_CONCATENATION_CELL = "python：git + step（step 是循环里取的、折不出）命令位置上是 git，不拒算"
FOLD_SLICE_CELL = "python：f(…)[k] 取返回的元组那一项、command[:command.index(<字面词>)] 照切，连到那份脚本"
FOLD_VERSIONS_CELL = "python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码，不当工具链询问）"
FOLD_LOADED_CELL = "python：spec_from_file_location 载入的模块的常量（module.NAME）与模块级元组[整数] 折得出，sh -c 的串连到它起的脚本"
FOLD_TAIL_GROUP_CELL = "python：os.path.join(<折不出的目录>, \"tools\", \"runner.py\") 连到仓里以 tools/runner.py 收尾的那一份"
FOLD_OUTSIDE_SCRIPT_CELL = "python：os.path.join(<折不出的目录>, \"generated\", \"made.py\") 仓里没有以它收尾的文件：不是仓里的脚本，不拒算"
FOLD_CWD_GROUP_CELL = "python：[\"bash\", \"setup.sh\"] 在折不出的 cwd= 里起：仓里以 /setup.sh 收尾的整组连上"
FOLD_SUBMITTED_CELL = "python：pool.submit(f, 实参) 算一处调用，f 的命令参数按它折，连到那份脚本"
FOLD_ENVIRONMENT_CELL = "python：env -u X NAME={折不出} 之后的命令照命令位置认，连到那份脚本"
DATA_READ_CHANGED_CELL = "python：运行期 open 读的仓内文件（路径折得出）进键：改它键变"
DATA_READ_UNRELATED_CELL = "python：运行期 open 读的仓内文件进键：改不相干的文件键不变"
DATA_READ_TAIL_CELL = "python：读 <sys.argv 给的目录>/pkg/table.tsv 照字面尾巴收仓里那一份：改它键变"
DATA_READ_SCRATCH_CELL = "python：读这一趟自己在 mkdtemp 建的目录里写的文件（仓里没有这一截尾巴）不拒算"
DATA_READ_REFUSED_CELL = "python：读 os.path.join(<根本身>, <读进来的登记行点名的一截>) 拒算 data-path-unfolded"
DATA_READ_DIRECTORY_CELL = "python：读 os.path.join(<根里的目录>, <从那个目录的清单里挑的一截>) 收那个目录整棵：改目录里的一份键变"
DATA_READ_RETURNED_LISTING_CELL = ("python：读 os.path.join(<根里的目录>, <本模块函数返回的那个目录的清单里挑的一截>)（for 名字 in 列清单的函数(目录)）"
                                   "收那个目录整棵：改目录里的一份键变")
DATA_READ_UNLISTED_DIRECTORY_CELL = "python：读 os.path.join(<根里的目录>, <出处推不出、不是从清单里挑的一截>) 拒算 data-path-unfolded，不收那个目录整棵"
DATA_READ_MODULE_SEQUENCE_CELL = ("python：读 os.path.join(<参数缺省的目录>, <for 名字 in 别的模块.NAMES 取的一项>)（读的函数在另一份模块里）收那几份文件："
                                  "改其中一份键变，不拒算")
# 读文件的函数在另一份模块里、只照调用处认路径（ScriptClosure.resolve_parameter_reads）；调用处的文件名取自 import 进来的模块的模块级元组
# （PythonModule.fold_elements 认 module.NAMES），目录是参数、照它的缺省折（E171 removal_effect.py 的 files_of_the_rows 那一种写法）
MODULE_SEQUENCE_FIXTURE = {
    "pkg/table_names.py": 'TABLES = ("alpha.tsv", "beta.tsv")\n',
    "pkg/table_rows.py": '''def read_rows(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read().splitlines()
''',
    "pkg/tables.py": '''import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import table_names  # noqa: E402
from table_rows import read_rows  # noqa: E402


def rows_of_the_tables(directory=HERE):
    return {row for name in table_names.TABLES for row in read_rows(os.path.join(directory, name))}


def all_rows():
    return rows_of_the_tables()
''',
    "pkg/alpha.tsv": "alpha\n",
    "pkg/beta.tsv": "beta\n",
}
FOLD_LIMIT_READ_REFUSED_CELL = "python：读 os.path.join(<根本身>, <取遍 65 项字面元组的循环变量>)（取值超过 FOLD_LIMIT 种）拒算 fold-limit-exceeded，不当调用方的输入不收"
FOLD_LIMIT_READ_UNDER_CELL = "python：读 os.path.join(<根本身>, <取遍 64 项字面元组的循环变量>)（取值恰好 FOLD_LIMIT 种）收那 64 份：改其中一份键变"
FOLD_LIMIT_LAUNCH_REFUSED_CELL = "python：起 [\"python3\", os.path.join(<根本身>, <取遍 65 项字面元组的循环变量>)]（取值超过 FOLD_LIMIT 种）拒算 fold-limit-exceeded"
FOLD_LIMIT_SHELL_TAIL_CELL = ("shell：bash \"$dir/tools/run.sh\"，dir 取遍 for 里的 65 个目录（取值超过 FOLD_LIMIT 种）：不照根里同一相对路径那一份猜，"
                              "照尾巴整组收，改 s/a07/tools/run.sh 键变")
# 超过 FOLD_LIMIT（64）种取值的读与起法（文件头「拒算」公共一行 fold-limit-exceeded）：照钩子库 own_library_digest 的写法缩成几份模块
# （for 名字 in <字面元组>：if 名字 == 要的：读 os.path.join(<根本身>, 名字)，读在另一个函数里、照参数认）；64 项那一份是上限本身，照收。
# 读与起法各在自己的模块里：同一个读文件的函数被两处调用时照每一处的实参认，混在一份里 64 项那一格也会连到 65 项那一处
FOLD_LIMIT_FIXTURE = {
    "pkg/limits_over.py": ('import os\n\nROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))\n'
                           'TABLES = (' + ", ".join(f'"pkg/t{number:02d}.tsv"' for number in range(65)) + ')\n\n\n'
                           'def file_digest(path):\n    with open(path, "rb") as handle:\n        return handle.read()\n\n\n'
                           'def table_digest(wanted):\n    for listed in TABLES:\n        if listed == wanted:\n'
                           '            return file_digest(os.path.join(ROOT, listed))\n    return None\n'),
    "pkg/limits_under.py": ('import os\n\nROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))\n'
                            'TABLES = (' + ", ".join(f'"pkg/t{number:02d}.tsv"' for number in range(64)) + ')\n\n\n'
                            'def file_digest(path):\n    with open(path, "rb") as handle:\n        return handle.read()\n\n\n'
                            'def table_digest(wanted):\n    for listed in TABLES:\n        if listed == wanted:\n'
                            '            return file_digest(os.path.join(ROOT, listed))\n    return None\n'),
    "pkg/limits_launch.py": ('import os\nimport subprocess\n\nROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))\n'
                             'SCRIPTS = (' + ", ".join(f'"tools/s{number:02d}.py"' for number in range(65)) + ')\n\n\n'
                             'def run_all():\n    for name in SCRIPTS:\n        subprocess.run(["python3", os.path.join(ROOT, name)], check=False)\n'),
    "s/limits_tail.sh": ("#!/usr/bin/env bash\nfor dir in " + " ".join(f"s/a{number:02d}" for number in range(65)) + "; do\n"
                         '  bash "$dir/tools/run.sh"\ndone\n'),
    "tools/run.sh": "#!/usr/bin/env bash\necho root copy\n",
    "s/a07/tools/run.sh": "#!/usr/bin/env bash\necho a07\n",
    **{f"pkg/t{number:02d}.tsv": f"row {number}\n" for number in range(65)},
}
FOLD_LIMIT_UNMATCHED_TAILS_CELL = ("python：读 os.path.join(<参数>, \"dump\", <3 种流>, <5 种文件名>)，参数照调用处取 5 种尾巴（合起来 75 种、超过 FOLD_LIMIT）："
                                   "折得出的那一截（dump/<流>/<名>）在仓里一份都对不上，整组都不是仓里的文件，不拒算")
FOLD_LIMIT_MATCHED_TAILS_CELL = ("python：同一处读，仓里有一份以 /dump/<流>/<名> 收尾的文件：那一截对得上，照旧拒算 fold-limit-exceeded"
                                 "（不照那一截猜整组）")
# 照 E171 rows_orchestrator.py 的 dump_differences 缩写：两个行目录下 dump/<流>/<名> 逐个比，目录是参数、照几处调用取尾巴
# （一处 os.path.join(<参数>, "base")，另一处 os.path.join(<参数>, <取遍 5 项字面元组的循环变量>)）；改了的那一份多放一个对得上那一截的文件
FOLD_LIMIT_TAILS_SOURCE = ('import os\n\nSTREAMS = ("s0", "s1", "s2")\nNAMES = ("a.tsv", "b.tsv", "c.tsv", "d.tsv", "e.tsv")\n'
                           'PARTS = ("same", "device", "dump", "o3", "row")\n\n\n'
                           'def differences(first, second):\n    differing = []\n    for stream in STREAMS:\n        for name in NAMES:\n'
                           '            left, right = os.path.join(first, "dump", stream, name), os.path.join(second, "dump", stream, name)\n'
                           '            if open(left, "rb").read() != open(right, "rb").read():\n                differing.append(name)\n'
                           '    return differing\n\n\n'
                           'def compare_all(scratch):\n    return [differences(os.path.join(scratch, "base"), os.path.join(scratch, part)) for part in PARTS]\n')
FOLD_LIMIT_TAILS_FIXTURE = {"pkg/dump_compare.py": FOLD_LIMIT_TAILS_SOURCE}
FOLD_LIMIT_MATCHED_TAILS_FIXTURE = {"pkg/dump_compare.py": FOLD_LIMIT_TAILS_SOURCE, "results/kept/dump/s1/c.tsv": "kept\n"}
SHELL_QUOTED_INLINE_CELL = ("shell：bash -c \"touch $(printf '%q' \"$work/ran\"); bash s/child.sh\"（work 折不出）：printf %q 那一处在内联正文里是一个取值折不出的词，"
                            "正文照切，不拒算，连到 s/child.sh（改它键变）")
SHELL_QUOTED_INLINE_LAUNCH_CELL = ("shell：bash -c \"bash $(printf '%q' \"$work/tool.sh\")\"（work 折不出）：内联正文里起的脚本就是那个折不出的词，"
                                   "照拒算 shell-path-unfolded")
# 照 scripts/hosts/multi-host-run-selftest.sh 的写法缩写：bash -c "<字面命令> $(printf '%q' "<折不出的目录>/<文件>")"
SHELL_QUOTED_INLINE_FIXTURE = {
    "s/child.sh": "#!/usr/bin/env bash\necho child\n",
    "s/quoted_inline.sh": '#!/usr/bin/env bash\nwork="$1"\nbash -c "touch $(printf \'%q\' "$work/ran"); bash s/child.sh"\n',
    "s/quoted_launch.sh": '#!/usr/bin/env bash\nwork="$1"\nbash -c "bash $(printf \'%q\' "$work/tool.sh")"\n',
}
PREFIX_SCOPED_CELL = ("shell：命令前的 tool=<折不出> plan 只管 plan 那一次调用：plan 走不到的 merge 里 bash \"$tool\" 照别处的赋值折，"
                      "连到 s/child.sh（改它键变）")
TEMPORARY_SITE_CELL = ("shell：命令前的 tool=\"$work/fake.sh\" plan（work 是 mktemp -d）管得到 plan 里的 bash \"$tool\"，但起的是这一趟自己写的临时文件："
                       "这一处赋值不折，照别处的赋值连到 s/child.sh（改它键变）")
# 照 scripts/mutation/mutation-shard-run.sh 的写法缩成两份：合并那一步的函数读全局的 rows_module，自证里换成临时目录下的假模块是在别的函数的调用前赋的
# （prefix_scoped.sh 的 merge）；计划那一步的函数本身就是自证换了假模块去调的那一个（temporary_site.sh 的 plan）
PREFIX_SCOPED_FIXTURE = {
    "s/child.sh": "#!/usr/bin/env bash\necho child\n",
    "s/prefix_scoped.sh": ('#!/usr/bin/env bash\nmerge() {\n  bash "$tool"\n}\nplan() {\n  echo plan\n}\nselftest() {\n  tool="$1/fake.sh" plan\n}\n'
                           'tool="s/child.sh"\nmerge\n'),
    "s/temporary_site.sh": ('#!/usr/bin/env bash\nplan() {\n  bash "$tool"\n}\nselftest() {\n  work="$(mktemp -d)"\n'
                            "  printf '%s\\n' 'echo fake' > \"$work/fake.sh\"\n"
                            '  tool="$work/fake.sh" plan\n}\ntool="s/child.sh"\nplan\n'),
}
# 按名字模式列的清单里取文件名、到一个目录里读（PythonModuleDataReads.pattern_listed_files）：python 的 glob 与 shell 交进内联 python 的 find 清单，各一绿一红
PATTERN_LISTING_FIXTURE = {
    "pkg/listings.py": '''import glob
import os

HERE = os.path.dirname(os.path.abspath(__file__))
STAGES = sorted(os.path.basename(path) for path in glob.glob(os.path.join(HERE, "stages", "*.sh")) if os.path.isfile(path))


def stage_texts():
    return [open(os.path.join(HERE, "stages", stage), encoding="utf-8").read() for stage in STAGES]


def stage_tables():
    texts = []
    for path in glob.glob(os.path.join(HERE, "stages", "*.sh")):
        table = os.path.splitext(os.path.basename(path))[0] + ".tsv"
        texts.append(open(os.path.join(HERE, "stages", table), encoding="utf-8").read())
    return texts
''',
    "pkg/stages/build.sh": "echo build\n",
    "pkg/stages/check.sh": "echo check\n",
    "pkg/stages/build.tsv": "table build\n",
    "pkg/stages/notes.txt": "notes\n",
    "s/find_listing.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
listing="$(find "$here/stages" -maxdepth 1 -name '*.sh' \\( -type f -o -type l \\) | sort)"
python3 - "$here/stages" "$listing" <<'PY'
import os
import sys

ordered = [os.path.basename(path) for path in sys.argv[2].split("\\n") if path.strip()]
for position, name in enumerate(ordered):
    with open(os.path.join(sys.argv[1], name), encoding="utf-8") as handle:
        print(position, len(handle.read()))
PY
''',
    "s/find_listing_any.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
listing="$(find "$here/stages" -maxdepth 1 -name '*.sh' -o -type f)"
python3 - "$here/stages" "$listing" <<'PY'
import os
import sys

ordered = [os.path.basename(path) for path in sys.argv[2].split("\\n") if path.strip()]
for position, name in enumerate(ordered):
    with open(os.path.join(sys.argv[1], name), encoding="utf-8") as handle:
        print(position, len(handle.read()))
PY
''',
    "s/stages/build.sh": "echo build\n",
    "s/stages/check.sh": "echo check\n",
    "s/stages/notes.txt": "notes\n",
}
PATTERN_LISTING_GLOB_CELL = ("python：读 os.path.join(<根里的目录>, <glob 按 *.sh 列出、os.path.basename 取的文件名>) 只收目录里合 *.sh 的几份："
                             "改其中一份键变，改目录里别的文件键不变，不拒算")
PATTERN_LISTING_GLOB_TRANSFORMED_CELL = ("python：读 os.path.join(<根里的目录>, <glob 列出的文件名换了后缀>)（读的那一份不一定合模式）"
                                         "拒算 data-path-unfolded")
PATTERN_LISTING_FIND_CELL = ("shell：$(find <目录> -maxdepth 1 -name '*.sh' \\( -type f -o -type l \\) | sort) 交给内联 python，"
                             "读 os.path.join(<根里的目录>, <清单一行的 os.path.basename>) 只收目录里合 *.sh 的几份：改其中一份键变，改目录里别的文件键不变，不拒算")
PATTERN_LISTING_FIND_OPERATORS_CELL = ("shell：$(find <目录> -maxdepth 1 -name '*.sh' -o -type f) 交给内联 python（顶层的 -o 让不合模式的行也进清单），"
                                       "读 os.path.join(<根里的目录>, <清单一行的 os.path.basename>) 拒算 data-path-unfolded")
DATA_READ_LIBRARY_CELL = "python：别的模块里的函数读它参数给的文件：闭包里的调用处给仓里的常量收进键（改它键变），给入口参数的不收也不拒算"
DATA_READ_ATTRIBUTE_CELL = "python：读 self.<属性>：照本类方法里给它的赋值认，赋的是仓里的常量就收进键（改它键变）"
TOOLCHAIN_QUERY_CELL = "python：[<环境给的编译器>, \"-V\"] 是工具链询问：不拒算、算得出键，询问打的版本不进键：闭包里没有工具链单元，换一版假 rustc 键不变"
CALLER_LAUNCH_CELL = "python：判法入口的参数给的命令（-c 的串、解释器）是调用方交进来的：不拒算"
REGISTRY_PROGRAMS_CELL = "python：登记表点名的程序（REGISTRY_LISTED_PROGRAMS）是环境探针：不连、不拒算，改登记表点名的一份探针，起探针的入口键不变"
ROW_KEYED_CELL = "按行进键的登记表：判法经登记的取用函数按入口查表里一行，改表里与这一条无关的行键不变、改它查的那一行键变；读整张表的不换"
PROGRAM_ARGUMENTS_CELL = "python：同一程序入口带两组参数：只改另一组参数才走到的函数，这一组键不变、改这一组走到的键变；起它的脚本把参数带过去照样剪"
DATA_READ_INPUT_CELL = "python：入口函数读它参数给的文件（调用方的输入）不拒算"
DATA_READ_PARSED_CELL = "python：读 argparse 解出来的路径（调用方的输入）不拒算"
DATA_READ_LISTING_CELL = "python：读 os.path.join(<根本身>, <算出来的名单里的一截>)（不是读根里的文件得来的）：被判的输入，不拒算"
DATA_READ_COPY_CELL = "python：拷文件的源 os.path.join(<根本身>, <读进来的登记行点名的一截>)：拷副本，不拒算"
TAIL_ANCHOR_SCRATCH_CELL = "python：读 <参数>/<字面尾巴>，闭包里的调用处给的是这一趟的临时目录：仓里这一截尾巴的那一组不收，改其中一份键不变"
TAIL_ANCHOR_JUDGE_CELL = "python：读 <判法入口的参数>/<字面尾巴>（被判的输入）：仓里这一截尾巴的那一组不收，改其中一份键不变"
TAIL_ANCHOR_ARGUMENT_CELL = "python：读 <参数>/<字面尾巴>，闭包里的调用处给的是 sys.argv：照尾巴收仓里那一组，改其中一份键变"
TAIL_ANCHOR_FIXTURE = {
    "pkg/__init__.py": "",
    "fixtures/sample/Manifest.toml": "name = sample\n",
    "pkg/manifest_reader.py": '''import os
import sys
import tempfile


def package_name(tree, relative):
    directory = os.path.dirname(os.path.join(tree, relative))
    while True:
        manifest = os.path.join(directory, "Manifest.toml")
        if os.path.isfile(manifest):
            return open(manifest, encoding="utf-8").read()
        if directory == tree:
            return None
        directory = os.path.dirname(directory)


def judge(tree):
    return package_name(tree, "src/lib.rs")


def scratch_run():
    with tempfile.TemporaryDirectory() as work:
        return package_name(os.path.join(work, "tree"), "src/lib.rs")


def from_argument():
    return package_name(sys.argv[1], "src/lib.rs")
''',
}
# 读 <参数>/<字面尾巴> 那两格的样本：门禁样本档底下多一份同名文件；入口带 --selftest 时 main 里走不到的那一处调用
TAIL_SAMPLE_CELL = "python：按字面尾巴读 Manifest.toml：门禁样本档底下多一份同名文件、改它都不动键"

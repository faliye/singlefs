# 算键库 scripts/gate/lib_function_units.py 的自证才读的模块级常量：夹具文本、格名、开关到格的对照表（BREAK_CELLS）。
# 不单独起、不被 import：只由算键库的 load_selftest_fixtures 在自证开跑前按路径载入；载入前先把算键库此刻的名字放进这份模块，
# 这里的式子照那些名字求值，求出来的名字再放回算键库（格函数与 selftest 照名字读它们）。平常 import 算键库不载入这一份，
# 这里的常量不进它 import 时执行的语句，也就不进经它收的各条判法闭包（D13（验证路线） 已定项 17）。
# 搬进来的只是运行期函数一个都不读的那几样；运行期也读的、正文里写着自证起法的那几样留在算键库里（算键库文件头「自证夹具」）。

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
  printf '%s\\n' "import sys" "sys.path.insert(0, sys.argv[1])" "import helper_mod" "helper_mod.run()"
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
REGISTRY_COMMAND_FIXTURE = {
    ".claude/gate.d/doc-registries.sh": ("#!/usr/bin/env bash\npython3 - <<'PY'\nimport subprocess\n\n\ndef counting_command_result(command):\n"
                                         "    return subprocess.run([\"bash\", \"-o\", \"pipefail\", \"-c\", command], check=False)\n\n\n"
                                         "with open('.claude/gate.d/counted-claims.tsv', encoding='utf-8') as handle:\n    for row in handle:\n        counting_command_result(row)\nPY\n"),
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
DATA_READ_DIRECTORY_CELL = "python：读 os.path.join(<根里的目录>, <出处推不出的一截>) 收那个目录整棵：改目录里的一份键变"
DATA_READ_LIBRARY_CELL = "python：别的模块里的函数读它参数给的文件：闭包里的调用处给仓里的常量收进键（改它键变），给入口参数的不收也不拒算"
DATA_READ_ATTRIBUTE_CELL = "python：读 self.<属性>：照本类方法里给它的赋值认，赋的是仓里的常量就收进键（改它键变）"
TOOLCHAIN_QUERY_CELL = "python：[<环境给的编译器>, \"-V\"] 是工具链询问：不拒算，工具链单元进键：换一版假 rustc 键变，闭包里没有询问的换 rustc 键不变"
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

# 只在弄坏开关打开时走到的那一支读自己的原文
EXCLUDED_FILE_DATA_CELL = "排除文件：只经它读到的数据不进键（只改判法文件一行注释键不变）"
EXCLUDED_FILE_PROGRAM_CELL = "排除文件：只经它起的程序不进键（改那个程序键不变）"
EXCLUDED_FILE_KEPT_CELL = "排除文件：判法自己的单元照进键（改它键变）"
EXCLUDED_FILE_UNSET_CELL = ("排除文件：不给排除时它读的数据照进键（它读的判法文件只改一行注释，键也变）；crates 变异判法那一路不给排除，"
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
''',
}
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

NESTED_PROGRAM_WORDS_OUTER = "tool/nested_seam.sh::run_cells"
NESTED_PROGRAM_WORDS_INNER = "tool/nested_seam.sh::run_cells::run_inner_seam"

PYTHON_STATEMENT_UNIT_NAMES = ("<module-body>", "<main-guard>")
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
    "bound-diverging-fallback": '''fn restart() {
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
    "scripts/gate/lib_function_units_selftest_fixtures.py": "算键库的自证夹具，自证开跑前才载入；这个名字只出现在格名、扫描用的常量与夹具文本里，不放缓存",
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
SELFTEST_FIXTURES_IMPORT_EDIT = ("\n\nif __name__ == '__main__':\n", "\n\nSELFTEST_FIXTURES_PROBE = load_selftest_fixtures\n\n\nif __name__ == '__main__':\n")
SELFTEST_FIXTURES_APPENDED = '\nSELFTEST_FIXTURES_APPENDED_PROBE = "自证在这一份末尾加的一句"\n'

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
    ("external-programs-unchecked", ("键：点名了 cargo 而调用方没给 --external 拒算、给了出键",)),
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
    ("shell-heredoc-captures-unfolded", ("shell：python3 -c \"$(打印正文的函数)\" 折出函数打的那几行当内联脚本",)),
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
    ("tail-read-scopes-copies-unscoped", (TAIL_READ_SCOPES_CELL, TAIL_READ_SCOPES_REAL_CELL)),
    ("tail-read-scopes-drifted", (TAIL_READ_SCOPES_REAL_CELL,)),
    ("tail-read-scopes-inline-unread", (TAIL_READ_SCOPES_REAL_CELL,)),
    ("tail-read-scopes-root-ignored", (TAIL_READ_ROOT_SCOPE_CELL,)),
    ("python-getattr-names-unfolded", (GETATTR_NAMES_CELL,)),
    ("shell-function-outputs-unfolded", (FUNCTION_PRINTED_WORD_CELL,)),
    ("shell-quoted-assignments-unread", (QUOTED_ASSIGNMENT_ENV_CELL, QUOTED_ASSIGNMENT_ARRAY_CELL)),
    ("python-local-names-resolved", (LOCAL_NAME_SHADOW_CELL,)),
    ("stage-cell-functions-drifted", (STAGE_CELL_FUNCTIONS_REAL_CELL,)),
    ("python-module-body-commands-unfolded", (MODULE_BODY_COMMAND_CELL,)),
    ("shell-exported-variables-unread", (EXPORTED_VARIABLE_CELL,)),
    ("python-guard-names-unfolded", (GUARD_NAME_PATH_CELL,)),
    ("shell-heredoc-empty-rows-spanned", (HEREDOC_BLANK_ROW_CELL,)),
    ("shell-narrowed-views-unshared", (NARROWED_VIEW_SHARED_CELL,)),
    ("rust-own-program-restart-refused", (OWN_PROGRAM_KEYED_CELL,)),
    ("rust-own-program-entry-unchecked", (OWN_PROGRAM_ENTRY_CELL,)),
    ("rust-own-program-path-unchecked", (OWN_PROGRAM_PATH_CELL,)),
    ("rust-extern-listed-calls-refused", (EXTERN_LISTED_CELL,)),
    ("rust-extern-any-block-keyed", (EXTERN_REFUSED_CELL,)),
    ("shell-nested-functions-by-any-word", (SHELL_NESTED_WORD_CELL,)),
    ("shell-python-launch-arguments-unfolded", (SHELL_PYTHON_ARGUMENTS_SKIPPED_CELL,)),
    ("closure-round-limit-unchecked", (CLOSURE_ROUND_LIMIT_CELL,)),
    ("closure-call-index-frozen", (CLOSURE_CALL_INDEX_CELL,)),
    ("data-units-process-wide", (DATA_READ_CHANGED_CELL, DATA_READ_DIRECTORY_CELL)),
    ("judging-entry-index-not-shared", (SHARED_SCRIPT_INDEX_CELL,)),
    ("python-toolchain-queries-refused", (TOOLCHAIN_QUERY_CELL,)),
    ("python-toolchain-queries-unkeyed", (TOOLCHAIN_QUERY_CELL,)),
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
    ("rust-let-shadowed-names-resolved", (LET_SHADOW_BOUND_CELL, LET_SHADOW_PARAMETER_CELL, LET_SHADOW_CAPTURE_CELL)),
    ("rust-use-paths-flattened", (USE_PATH_EXACT_CELL,)),
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
    ("excluded-edge-exit-unmeasured", (EXCLUDED_EDGE_EXIT_CELL,)),
    ("admission-edge-admitted-sample", (ADMISSION_EDGE_ANOMALY_CELL,)),
    ("admission-edge-header-unstripped", (ADMISSION_EDGE_STRIP_CELL,)),
    ("admission-edge-reach-unmeasured", (ADMISSION_EDGE_REACH_CELL,)),
    ("card-lease-client-words-granted-sample", (CARD_LEASE_CLIENT_WORDS_ANOMALY_CELL,)),
    ("card-lease-client-words-return-output-kept", (CARD_LEASE_CLIENT_WORDS_STRIP_CELL,)),
    ("card-lease-client-words-other-launches-unchecked", (CARD_LEASE_CLIENT_WORDS_REACH_CELL,)),
    ("judging-entry-literals-unfolded", (REAL_ENTRIES_CELL, REAL_JUDGING_CLOSURES_CELL)),
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
)

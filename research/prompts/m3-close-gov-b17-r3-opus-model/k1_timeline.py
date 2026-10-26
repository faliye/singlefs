#!/usr/bin/env python3
"""K1 第三条路：在一份脚本副本（--scripts，里面的 lib_function_units.py 可能改过）上，用 crates-mutation-rows.py 自证里的假 cargo / 假 llvm 工具 /
假内存包装，对一棵自造的小仓跑一趟 run_stage（不起真 cargo、不编译），打出这一行的判定、几把键与库里的覆盖记录。
用法：k1_timeline.py --scripts <副本根> --fixture <小仓目录> --state S0|S1 --library <库目录> --work <工作目录> [--break <GATE_MUTATION_BREAK 的值>]
S0：建小仓（已在就不动）；S1、S3：把 add 的函数体 { left + right } 改成 { right + left }（点名测试 helps 在没改坏的源码上就红）；S2：改回来。"""
import argparse, contextlib, importlib.util, io, json, os, sys

SOURCE = '''// K1 攻方样本 crate（假 cargo 按下面几行判）
// TEST helps NEEDS value + 7 RUNS helps,add,helper
// TEST helps NEEDS left + right RUNS helps,add,helper
pub const OFFSET: u32 = 95;
pub fn add(left: u32, right: u32) -> u32 { left + right }

pub fn helper(value: u32) -> u32 { value + 7 }

#[cfg(test)]
mod tests {
    #[test]
    fn helps() { assert_eq!(super::add(super::helper(0), 3), 10); }
}
'''
ROWS = ["helper-becomes-eight\tcrates/tiny/src/lib.rs\t{ value + 7 }\t{ value + 8 }\t-p tiny --lib -- helps\thelps"]


def main():
    parser = argparse.ArgumentParser()
    for name in ("--scripts", "--fixture", "--state", "--library", "--work"):
        parser.add_argument(name, required=True)
    parser.add_argument("--break", dest="breaks", default="")
    arguments = parser.parse_args()
    if arguments.breaks:
        os.environ["GATE_MUTATION_BREAK"] = arguments.breaks
    else:
        os.environ.pop("GATE_MUTATION_BREAK", None)
    os.environ.pop("CRATES_MUTATION_ROWS_BREAK", None)
    path = os.path.join(arguments.scripts, "scripts", "mutation", "crates-mutation-rows.py")
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_k1", path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    fixture = arguments.fixture
    if not os.path.isdir(fixture):
        module.build_selftest_repository(fixture, ROWS, source=SOURCE)
    lib_rs = os.path.join(fixture, "crates", "tiny", "src", "lib.rs")
    text = open(lib_rs, encoding="utf-8").read()
    if arguments.state in ("S1", "S3") and "{ left + right }" in text:
        with open(lib_rs, "w", encoding="utf-8") as handle:
            handle.write(text.replace("{ left + right }", "{ right + left }"))
    if arguments.state == "S2" and "{ right + left }" in text:
        with open(lib_rs, "w", encoding="utf-8") as handle:
            handle.write(text.replace("{ right + left }", "{ left + right }"))
    os.makedirs(arguments.work, exist_ok=True)
    selftest = module.Selftest(arguments.work)
    changes = dict(GATE_MUTATION_RECORDS_HOME=arguments.library)
    if arguments.breaks:
        changes["GATE_MUTATION_BREAK"] = arguments.breaks
    exit_code, stdout, stderr, invocations = selftest.run(fixture, judging_repository=arguments.scripts, **changes)
    print(f"name=run state={arguments.state} break={arguments.breaks or '-'} exit={exit_code}")
    for line in module.selftest_verdict_view(stdout).splitlines():
        if line.strip():
            print(f"name=verdict_line {line}")
    print(f"name=cargo_calls total={len(invocations)} coverage={len(module.coverage_invocations(invocations))} plain={len(module.plain_invocations(invocations))}")
    environment = dict(selftest.base_environment, **changes)
    context = module.KeyContext(fixture, environment, "1G", 60, arguments.scripts)
    row = module.read_table(os.path.join(fixture, module.TABLE))[0][0]
    components = context.components(row)
    print(f"name=keys judging={components.judging[:16]} row_key={(components.row_key or '-')[:16]} baseline_key={(components.baseline_key or '-')[:16]} "
          f"coverage_id={(components.coverage_id or '-')[:16]} test_closure={(components.test_closure or '-')[:16]} "
          f"mutated={','.join(r.kind + ':' + r.name for r in components.mutated or [])}")
    library = os.path.join(arguments.library, module.LIBRARY_DIRECTORY_NAME)
    coverage_directory = os.path.join(library, "coverage")
    for name in sorted(os.listdir(coverage_directory)) if os.path.isdir(coverage_directory) else []:
        record = module.mutation_units.Library(library).read_json("coverage", name.split(".")[0])
        executed = ",".join(f"{item[0]}:{item[2]}" for item in record.get("executed", []))
        print(f"name=coverage_record id={name[:16]} complete={record.get('complete')} executed={executed}")
    print(f"name=stderr_tail {' | '.join(stderr.strip().splitlines()[-3:])}")


if __name__ == "__main__":
    main()

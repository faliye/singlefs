#!/usr/bin/env bash
# Q6：快照树的 crash-case-check.py 对各种「全量枚举用例」写法认不认。用法：bash q6_fixture.sh [快照目录] [草稿目录]
set -uo pipefail
S="${1:-/tmp/claude-1000/three-way-attack/verification-split-r1/snap}"
F="${2:-/tmp/claude-1000/three-way-attack/verification-split-r1/q6-rerun}/fixture"
rm -rf "${F:?}"; mkdir -p "$F/crates/singlefs-harness/tests" "$F/crates/singlefs-checker/tests" "$F/.claude/gate.d"
printf '# fixture\ncrash-case:ok\tcrates/\ttest=singlefs-checker:registered:registered_full\t# x\n' > "$F/.claude/gate.d/stage-inputs.tsv"
w() { printf '%s' "$2" > "$F/crates/$1"; }
w singlefs-checker/tests/registered.rs $'#[test]\n#[ignore]\nfn registered_full() {\n    enumerate_layer0_versions(&stream);\n}\n'
w singlefs-harness/tests/f1_same_file_helper.rs $'fn run_everything() {\n    enumerate_layer0(&big);\n}\n\n#[test]\nfn f1_full_enumeration_through_a_helper() {\n    run_everything();\n}\n'
w singlefs-harness/tests/f2_one_line.rs $'#[test] fn f2_full_enumeration_on_one_line() { enumerate_layer0(&big); }\n'
w singlefs-harness/tests/f3_bracket_in_ignore_reason.rs $'#[test]\n#[ignore = "全量 [层 0]：平时不跑"]\nfn f3_full_enumeration_with_a_bracket_in_the_reason() {\n    enumerate_layer0(&big);\n}\n'
w singlefs-checker/tests/f3b_bracket_unregistered.rs $'#[test]\n#[ignore = "全量 [层 0]：平时不跑"]\nfn f3b_checker_full_unregistered() {\n    enumerate_layer0(&big);\n}\n'
w singlefs-harness/tests/f4_macro.rs $'macro_rules! case { ($n:ident) => { #[test] fn $n() { enumerate_layer0(&big); } }; }\ncase!(f4_generated);\n'
w singlefs-harness/tests/f5_waiver_abuse.rs $'// crash-case-check:not-a-crash-case 很快的流\n#[test]\nfn f5_full_enumeration_with_a_waiver() {\n    enumerate_layer0_in_state_slices(&b, &w, &s, r, &v, &full_expansion, p, None, &resume);\n}\n'
w singlefs-checker/tests/f6_ignored_waived.rs $'// crash-case-check:not-a-crash-case 很快的流\n#[test]\n#[ignore]\nfn f6_ignored_and_waived_never_registered() {\n    enumerate_layer0(&big);\n}\n'
w singlefs-checker/tests/f7_big_unignored.rs $'#[test]\nfn f7_full_enumeration_of_a_big_stream_in_the_quick_tier() {\n    enumerate_layer0_versions(&sixty_seven_million_states);\n}\n'
w singlefs-harness/tests/f8_own_enumerator.rs $'#[test]\nfn f8_every_subset_of_an_eighteen_write_segment() {\n    enumerate_every_subset_of_one_segment(&segment);\n}\n'
w singlefs-harness/tests/f9_renamed_import.rs $'use singlefs_harness::crash::enumerate_layer0 as run_all;\n#[test]\nfn f9_full_enumeration_under_another_name() {\n    run_all(&big);\n}\n'
w singlefs-harness/tests/f10_control_direct.rs $'#[test]\nfn f10_control_direct_full_in_harness() {\n    enumerate_layer0(&big);\n}\n'
w singlefs-harness/tests/f11_cfg_attr_ignore.rs $'#[test]\n#[cfg_attr(not(feature = "full"), ignore)]\nfn f11_cfg_attr() {\n    enumerate_layer0(&big);\n}\n'
(cd "$S" && python3 research/scripts/crash-case-check.py "$F"; echo "rc=$?")
python3 - "$S" "$F" <<'PY'
import importlib.util, sys
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("ccc", sys.argv[1] + "/research/scripts/crash-case-check.py"); m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
print("recognized:", [t[2] for t in m.crash_enumeration_tests(sys.argv[2])])
PY

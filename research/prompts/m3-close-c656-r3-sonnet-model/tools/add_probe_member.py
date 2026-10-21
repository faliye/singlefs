"""把辩方原型 crate 加进仓副本的工作区成员表，并给几个包加 release 档覆盖（编得快一些；照 m3-close-c656-r2 攻方那一份，只在副本里）。
用法：python3 add_probe_member.py <副本的 Cargo.toml>"""
import sys

path = sys.argv[1]
text = open(path).read()
anchor = '    "crates/singlefs-unit-graph",\n'
assert text.count(anchor) == 1, "工作区成员表的写法变了"
text = text.replace(anchor, anchor + '    "crates/c656-r3-defense-probe",\n', 1)
for name, level in [("singlefs-core", 1), ("singlefs-checker", 1), ("singlefs-checker-tier", 0), ("singlefs-harness", 1), ("c656-r3-defense-probe", 1)]:
    text += f"\n[profile.release.package.{name}]\nopt-level = {level}\ndebug = false\ncodegen-units = 256\n"
open(path, "w").write(text)
print("added probe member to", path)

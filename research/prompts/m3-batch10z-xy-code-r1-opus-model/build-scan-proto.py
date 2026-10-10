#!/usr/bin/env python3
"""K3（skip=名字 点名的条目不抽，它们引 crate 里别的东西、与扫源码无关）：把冻结副本 checker 档一份源文件 `mod tests` 里「读自己这份源文件」的扫源码用例连同它们（传递地）引用的条目机械抽出来，拼成独立小程序
（不是 checker 档包，不跑它的测试）：build-scan-proto.py <源文件> <输出 main.rs> [probe=函数名 …]。
条目按 `mod tests` 里四格缩进的行切（前面的属性与文档一起带走）；起点 = 函数体里有 include_str!("<这份文件>") 的 #[test]；闭包 = 条目文本里出现的别的条目名。
include_str!("<这份文件>") 换成读环境变量 SRC 指的那一份；#[test] 去掉，别的一字不改。main 逐条在 catch_unwind 里跑，打「名字: GREEN / RED 消息头」；
probe=函数名 点名的（签名收 `&str`）另跑一次，打它交回的 Debug 头或 panic 消息头。"""
import re
import sys

src_path, out_path = sys.argv[1], sys.argv[2]
probes = [a[6:] for a in sys.argv[3:] if a.startswith("probe=")]
skips = [a[5:] for a in sys.argv[3:] if a.startswith("skip=")]
file_name = src_path.rsplit("/", 1)[-1]
lines = open(src_path, encoding="utf-8").read().split("\n")
start = lines.index("mod tests {")
end = next(i for i in range(start + 1, len(lines)) if lines[i] == "}")
body = lines[start + 1:end]
head_re = re.compile(r"^    (?:pub(?:\([a-z]+\))? )?(fn|const|static|struct|enum|impl|type|use)\b\s*(?:<[^>]*>)?\s*([A-Za-z_][A-Za-z_0-9]*)?")
items = []
i = 0
while i < len(body):
    match = head_re.match(body[i])
    if not match:
        i += 1
        continue
    kind, name = match.group(1), match.group(2)
    first = i
    while first > 0 and (body[first - 1].startswith("    #[") or body[first - 1].startswith("    //")):
        first -= 1
    last = i
    if kind in ("const", "static", "type", "use"):
        balance = 0
        while True:
            balance += sum(body[last].count(c) for c in "([{") - sum(body[last].count(c) for c in ")]}")
            if balance == 0 and body[last].rstrip().endswith(";"):
                break
            last += 1
    elif not (body[i].rstrip().endswith("}") and body[i].count("{") == body[i].count("}") and "{" in body[i]):
        while body[last] != "    }":
            last += 1
    text = body[first:last + 1]
    is_test = any(line.strip() == "#[test]" for line in text)
    if kind == "impl":
        name = "impl:" + body[i].strip()
    items.append((name, kind, text, is_test))
    i = last + 1
by_name = {item[0]: item for item in items if item[1] != "use"}
include_re = re.compile(r'include_str!\(\s*"%s"\s*\)' % re.escape(file_name))
wanted = [item[0] for item in items if item[3] and include_re.search("\n".join(item[2]))] + probes
chosen, queue = [], list(wanted)
while queue:
    name = queue.pop(0)
    if name in chosen or name not in by_name or name in skips:
        continue
    chosen.append(name)
    text = "\n".join(by_name[name][2])
    for other in by_name:
        if other not in chosen and not other.startswith("impl:") and re.search(r"\b" + re.escape(other) + r"\b", text):
            queue.append(other)
for name in list(by_name):
    if name.startswith("impl:"):
        target = re.match(r"impl(?:<[^>]*>)?\s+(?:[A-Za-z_:<>]+\s+for\s+)?([A-Za-z_][A-Za-z_0-9]*)", name[5:])
        if target and target.group(1) in chosen and name not in chosen:
            chosen.append(name)
order = [item[0] for item in items if item[0] in chosen]
uses = [item for item in items if item[1] == "use" and not re.search(r"super|crate::|singlefs_|wgpu", " ".join(item[2]))]
text = "\n".join(line for name in order for line in by_name[name][2] + [""] if line.strip() != "#[test]")
text = "\n".join(line for item in uses for line in item[2]) + "\n" + include_re.sub("source_text()", text)
tests = [name for name in order if by_name[name][3]]
main = [
    "#![allow(dead_code, unused, warnings)]",
    "// 抄自冻结副本 " + src_path + " 的 mod tests（机械抽取，#[test] 去掉、include_str! 换成读 SRC）；main 是攻方加的。",
    'fn source_text() -> &\'static str { Box::leak(std::fs::read_to_string(std::env::var("SRC").expect("SRC")).expect("读得到 SRC").into_boxed_str()) }',
    text,
    "fn panic_head(payload: Box<dyn std::any::Any + Send>) -> String {",
    "    payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().replace('\\n', \" \").chars().take(220).collect()",
    "}",
    "fn main() {",
    "    std::panic::set_hook(Box::new(|_| {}));",
    "    let wanted: Vec<String> = std::env::args().skip(1).collect();",
    "    let tests: Vec<(&str, fn())> = vec![" + ", ".join('("%s", %s)' % (t, t) for t in tests) + "];",
    "    for (name, test) in tests {",
    "        if !wanted.is_empty() && !wanted.iter().any(|w| w == name) { continue; }",
    "        match std::panic::catch_unwind(test) { Ok(()) => println!(\"{name}: GREEN\"), Err(payload) => println!(\"{name}: RED {}\", panic_head(payload)) }",
    "    }",
]
for probe in probes:
    main.append('    match std::panic::catch_unwind(|| format!("{:?}", %s(source_text())).chars().take(80).collect::<String>()) { Ok(text) => println!("probe %s: ok len_of_debug={} head={text:?}", format!("{:?}", %s(source_text())).len()), Err(payload) => println!("probe %s: PANIC {}", panic_head(payload)) }' % (probe, probe, probe, probe))
main.append("}")
open(out_path, "w", encoding="utf-8").write("\n".join(main) + "\n")
print(f"{file_name}: 抽出 {len(order)} 个条目（用例 {len(tests)} 条）：{' '.join(tests)}")

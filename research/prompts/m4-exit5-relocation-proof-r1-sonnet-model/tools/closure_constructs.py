import re, sys, collections
# 用法：closure_constructs.py <冻结副本根> <closure 输出>：对闭包里每个 fn 单元取源码行区间，数含有 Verus 要另给规约或根本不支持的写法的 fn 个数（静态扫，按子串；一个 fn 可计入多项）
root, path = sys.argv[1:3]
patterns = {
    "BTreeMap/BTreeSet/HashMap/HashSet": r"\b(BTreeMap|BTreeSet|HashMap|HashSet)\b",
    "dyn trait": r"\bdyn\s+\w+",
    "泛型设备/读者 trait 约束": r"\b(Device|Reader)\s*:\s*(BlockDevice|PoolReader)",
    "闭包 |..|": r"\|[^|\n]*\|\s*(\{|[a-z_!&*(])",
    "迭代器链 .iter()/.map(/.filter(/.collect": r"\.(iter|into_iter|map|filter|collect|find_map|all|any)\(",
    "format!/panic!/assert!/unreachable!/expect(": r"(format!|panic!|assert!|assert_eq!|unreachable!|\.expect\()",
    "String / format 文字": r"\bString\b",
    "Vec<": r"\bVec<",
    "u64/usize 算术（+ - * 的粗数不计，只数 checked_/saturating_/try_from）": r"(checked_\w+|saturating_\w+|try_from)\(",
}
count = collections.Counter(); total = 0; cache = {}
for row in open(path, encoding="utf-8"):
    parts = row.rstrip("\n").split("\t")
    if len(parts) < 3 or parts[0] != "fn": continue
    file, span = parts[2].rsplit(":", 1); a, b = map(int, span.split("-"))
    if file not in cache: cache[file] = open(f"{root}/{file}", encoding="utf-8").read().split("\n")
    text = "\n".join(cache[file][a-1:b])
    total += 1
    for name, pat in patterns.items():
        if re.search(pat, text): count[name] += 1
print(f"fn 总数={total}")
for name in patterns: print(f"{count[name]:5d}  {name}")

# 按 (shape, borrow) 累加 name=counts 行里的计数（带 _peak 的取最大）
/name=counts/ {
  delete v
  for (i = 1; i <= NF; i++) { split($i, kv, "="); v[kv[1]] = kv[2] }
  g = v["shape"] " " v["borrow"]
  groups[g] = 1
  for (key in v) {
    if (key ~ /^(R2|name|shape|geometry|borrow|retirement_at_rollback|seed)$/) continue
    if (key ~ /_peak$/) { if (v[key] + 0 > mx[g, key]) mx[g, key] = v[key] + 0; keys[key] = 1 }
    else { sm[g, key] += v[key]; keys[key] = 1 }
  }
}
END {
  for (g in groups) for (key in keys) {
    if (key ~ /_peak$/) printf "%s %s=%d\n", g, key, mx[g, key]
    else printf "%s %s=%d\n", g, key, sm[g, key]
  }
}

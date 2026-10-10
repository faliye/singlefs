# 按 (scenario, reader, tree) 聚合 name=demo 行：放开扫的后缀数、r / m 为正的后缀数、峰值的最大值
/name=demo / && !/prefix_failed/ {
  delete v
  for (i = 1; i <= NF; i++) { split($i, kv, "="); v[kv[1]] = kv[2] }
  k = v["scenario"] " " v["reader"] " " v["tree"]
  n[k]++
  if (v["r_peak"] + 0 > 0) rp[k]++
  if (v["m_peak"] + 0 > 0) mp[k]++
  if (v["r_peak"] + 0 > R[k]) R[k] = v["r_peak"] + 0
  if (v["m_peak"] + 0 > M[k]) M[k] = v["m_peak"] + 0
}
/prefix_failed/ { print "PREFIX_FAILED " $0 }
END { for (k in n) printf "%s suffixes=%d suffixes_r_pos=%d suffixes_m_pos=%d r_max=%d m_max=%d\n", k, n[k], rp[k] + 0, mp[k] + 0, R[k], M[k] }

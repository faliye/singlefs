# 按 (shape, geometry, borrow, reader, tree) 聚合 name=cell 行：段数、r / m 峰值的最大值、r / m 为正的段数
/name=cell/ {
  delete v
  for (i = 1; i <= NF; i++) { split($i, kv, "="); v[kv[1]] = kv[2] }
  k = v["shape"] " " v["geometry"] " " v["borrow"] " " v["retirement_at_rollback"] " " v["reader"] " " v["tree"]
  n[k]++
  if (v["r_peak"] + 0 > rp[k]) rp[k] = v["r_peak"] + 0
  if (v["m_peak"] + 0 > mp[k]) mp[k] = v["m_peak"] + 0
  if (v["r_peak"] + 0 > 0) rh[k]++
  if (v["m_peak"] + 0 > 0) mh[k]++
}
END { for (k in n) printf "%s histories=%d r_peak_max=%d histories_r_pos=%d m_peak_max=%d histories_m_pos=%d\n", k, n[k], rp[k], rh[k] + 0, mp[k], mh[k] + 0 }

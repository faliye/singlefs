# 按 (几何, 政策, 读法, 树) 取全部历史上 r_peak、m_peak、r_out_peak 的最大值，并数 r_peak>0、m_peak>0 的历史数
/name=leaf_cell/ {
  delete v; for (i=2;i<=NF;i++){ split($i,kv,"="); v[kv[1]]=kv[2] }
  k = v["geometry"] " " v["policy"] " " v["reader"] " " v["tree"]
  n[k]++
  if (v["r_peak"]+0 > r[k]) r[k]=v["r_peak"]+0
  if (v["m_peak"]+0 > m[k]) m[k]=v["m_peak"]+0
  if (v["r_out_peak"]+0 > o[k]) o[k]=v["r_out_peak"]+0
  if (v["r_peak"]+0 > 0) rp[k]++
  if (v["m_peak"]+0 > 0) mp[k]++
}
END { for (k in n) printf "%s histories=%d r_peak_max=%d r_out_peak_max=%d histories_r_pos=%d m_peak_max=%d histories_m_pos=%d\n", k, n[k], r[k], o[k], rp[k]+0, m[k], mp[k]+0 }

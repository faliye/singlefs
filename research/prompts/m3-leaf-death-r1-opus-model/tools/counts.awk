/name=leaf_counts/ {
  delete v; for (i=2;i<=NF;i++){ split($i,kv,"="); v[kv[1]]=kv[2] }
  k = v["geometry"] " " v["policy"]
  n[k]++; pub[k]+=v["publishes"]; ret[k]+=v["retirements"]; pr[k]+=v["publishes_with_a_retirement"]
  mer[k]+=v["merges"]; bor[k]+=v["borrows"]; zv[k]+=v["zero_versions"]; od[k]+=v["oneshot_drops"]; rd[k]+=v["record_deaths"]
  rp[k]+=v["removal_publishes"]; cr[k]+=v["containers_rewritten_in_removals"]
  if (v["most_containers_rewritten_in_a_removal"]+0 > mx[k]) mx[k]=v["most_containers_rewritten_in_a_removal"]+0
  if (v["third_sentence_false_pairs_peak"]+0 > tf[k]) tf[k]=v["third_sentence_false_pairs_peak"]+0
  tv[k]+=v["versions_with_a_third_sentence_false_pair"]
  if (v["misrouted_records_peak"]+0 > mr[k]) mr[k]=v["misrouted_records_peak"]+0
  ev[k]+=v["versions_with_an_empty_container"]; if (v["empty_containers_peak"]+0 > ep[k]) ep[k]=v["empty_containers_peak"]+0
  if (v["stopped_at"]!="none") st[k]++
}
END { for (k in n) printf "%s runs=%d publishes=%d record_deaths=%d removal_publishes=%d containers_rewritten_in_removals=%d most_in_one_removal=%d retirements=%d publishes_with_a_retirement=%d merges=%d borrows=%d zero_versions=%d oneshot_drops=%d third_false_pairs_peak=%d versions_with_third_false=%d misrouted_peak=%d versions_with_empty_container=%d empty_peak=%d stopped=%d\n", k,n[k],pub[k],rd[k],rp[k],cr[k],mx[k],ret[k],pr[k],mer[k],bor[k],zv[k],od[k],tf[k],tv[k],mr[k],ev[k],ep[k],st[k]+0 }

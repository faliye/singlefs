BEGIN{FS="\t"; OFS="\t"}
$1=="publish"&&$6=="same_mount"&&$7=="W1"&&$8==1&&$9==0{print $4,$3,"R="$10,"A_final="$11,"W="$12,"Rel_sum="$13,"Rel_final="$14,"map="$16,"recs="$18,"b8="$24,sprintf("b8/Rs=%.1f b8/W=%.1f b8/(Rs+W)=%.1f b8/Rf=%.1f b8/Af=%.1f b8/(Rf+Af)=%.1f",$24/$13,$24/$12,$24/($13+$12),$24/$14,$24/$11,$24/($14+$11))}

BEGIN{FS="\t"; OFS="\t"}
$1=="publish"{k=$4 FS $3; if(!(k in mb)||$24>mb[k]){mb[k]=$24;mp[k]=$5;ms[k]=$13;mw[k]=$12;mf[k]=$14;ma[k]=$11;mr[k]=$10;mm[k]=$16}
 if($13>0){r=$24/$13; if(r>rx[k]){rx[k]=r;rxp[k]=$5;rxs[k]=$13;rxb[k]=$24}} else z[k]++
 r2=$24/($14+$11); if(r2>ry[k]){ry[k]=r2;ryp[k]=$5}
 n[k]++}
END{for(k in mb)print k,"pubs="n[k],"peak="mb[k],mp[k],"R="mr[k],"Rs="ms[k],"W="mw[k],"Rf="mf[k],"Af="ma[k],"map="mm[k],sprintf("peak/Rs=%.1f peak/(Rs+W)=%.1f peak/(Rf+Af)=%.1f",mb[k]/ms[k],mb[k]/(ms[k]+mw[k]),mb[k]/(mf[k]+ma[k])),sprintf("maxb8/Rs=%.1f",rx[k]),rxp[k],"at_Rs="rxs[k],"b8="rxb[k],sprintf("maxb8/(Rf+Af)=%.1f",ry[k]),ryp[k],"zeroRs="z[k]+0}

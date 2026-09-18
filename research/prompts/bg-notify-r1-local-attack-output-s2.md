1 Column 1 yes because the command contains nohup and disown which are forbidden by the ban on those words. Column 2 yes because the command contains nohup and disown as whole words. Column 3 yes because nohup and disown detach the process so it continues after shell exit. All three agree. This would be overturned if the command did not include nohup or disown.

2 Column 1 yes because the command contains nohup and ends with a trailing &. Column 2 yes because the command contains nohup as a whole word. Column 3 yes because nohup ensures the process continues after shell exit. All three agree. This would be overturned if the command did not include nohup.

3 Column 1 no because there are no forbidden elements (no nohup setsid disown or trailing &). Column 2 no because there are no lone &s and no forbidden words. Column 3 no because it is a foreground command and the shell exits after the work finishes. All three agree. This would be overturned if the command included a trailing & or any of nohup setsid or disown.

4 Column 1 no because there are no forbidden elements. Column 2 no because the only & is inside 2>&1 so not a lone &. Column 3 no because it is a foreground command and the shell exits after the work finishes. All three agree. This would be overturned if the command contained a trailing & or a lone & without wait after.

5 Column 1 no because there are no forbidden elements. Column 2 no because the & characters are inside 2>&1 and && so not lone &s. Column 3 no because it is a foreground command and the shell exits after the work finishes. All three agree. This would be overturned if the command included a trailing & or a lone & without wait after.

6 Column 1 no because there are no forbidden words and it does not end with a trailing & (ends with wait). Column 2 no because both lone &s have wait after them. Column 3 no because the wait command ensures the shell exits only after all background jobs finish. All three agree. This would be overturned if the command had a lone & without wait after it.

7 Column 1 yes because it ends with a trailing &. Column 2 yes because the lone & at the end has no wait after it. Column 3 no because the shell exits immediately and kills the background process without disown or nohup. Columns 1 and 2 differ from column 3. This would be overturned if the command included disown or nohup.

8 Column 1 no because there are no forbidden elements. Column 2 no because the & is inside |& so not a lone &. Column 3 no because it is a foreground command and the shell exits after the work finishes. All three agree. This would be overturned if the command contained a lone & without wait after.

9 Column 1 no because the & is inside quotes so not a trailing &. Column 2 no because the & is inside quotes so not a lone &. Column 3 no because it is a foreground command and the shell exits after the work finishes. All three agree. This would be overturned if the & was outside quotes making it a lone &.

10 Column 1 yes because the command contains setsid which is forbidden. Column 2 yes because the command contains setsid as a whole word. Column 3 yes because setsid detaches the process so it continues after shell exit. All three agree. This would be overturned if the command did not include setsid.

11 Column 1 no because the & is inside &> so not a trailing &. Column 2 no because the & is inside &> so not a lone &. Column 3 no because it is a foreground command and the shell exits after the work finishes. All three agree. This would be overturned if the & was not inside &> making it a lone &.

12 Column 1 no because there are no forbidden words and it does not end with a trailing & (ends with wait). Column 2 no because the lone & has wait after it. Column 3 no because the wait command ensures the shell exits only after the background job finishes. All three agree. This would be overturned if the command had a lone & without wait after it.

13 Column 1 yes because the command contains nohup as a whole word inside quotes which violates the ban. Column 2 yes because the command contains nohup as a whole word inside quotes. Column 3 no because it is a foreground command and the shell exits after the work finishes. Columns 1 and 2 differ from column 3. This would be overturned if the nohup was removed from the string.

14 Column 1 no because the & is inside single quotes so not a trailing &. Column 2 no because the & is inside single quotes so not a lone &. Column 3 no because the inner shell exits immediately and kills the background sleep process. All three agree. This would be overturned if the & was outside quotes making it a lone &.

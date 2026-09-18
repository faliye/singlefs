1 column 1 yes because the command contains nohup, disown, and a trailing & which are forbidden by Rule 1. column 2 yes because the command contains nohup and disown as whole words. column 3 yes because the trailing & causes the shell to exit immediately while the command continues running. all three agree. This would be overturned if the command did not include nohup, disown, or a trailing &.

2 column 1 yes because the command contains nohup and a trailing & which are forbidden by Rule 1. column 2 yes because the command contains nohup as a whole word. column 3 yes because the trailing & causes the shell to exit immediately while the command continues running. all three agree. This would be overturned if the command did not include nohup or a trailing &.

3 column 1 no because the command does not contain nohup, setsid, disown, or a trailing &. column 2 no because the command has no nohup/setsid/disown words and no lone &. column 3 no because the command runs in the foreground and the shell exits only after it completes. all three agree. This would be overturned if the command included a trailing & or nohup/setsid/disown.

4 column 1 no because the command does not contain nohup, setsid, disown, or a trailing &. column 2 no because the command has no nohup/setsid/disown words and no lone &. column 3 no because the command runs in the foreground and the shell exits only after it completes. all three agree. This would be overturned if the command included a trailing & or nohup/setsid/disown.

5 column 1 no because the command does not contain nohup, setsid, disown, or a trailing &. column 2 no because the command has no nohup/setsid/disown words and no lone &. column 3 no because the command runs in the foreground and the shell exits only after it completes. all three agree. This would be overturned if the command included a trailing & or nohup/setsid/disown.

6 column 1 no because Rule 1 allows starting multiple background jobs followed by wait. column 2 no because for each lone &, the word wait appears later in the command. column 3 no because the shell waits for the background jobs before exiting. all three agree. This would be overturned if the command did not include wait after the background jobs.

7 column 1 yes because the command has a trailing & which is forbidden by Rule 1. column 2 yes because the lone & has no wait after it. column 3 yes because the shell exits immediately after starting the background job. all three agree. This would be overturned if the command included wait after the background job.

8 column 1 no because the command does not contain nohup, setsid, disown, or a trailing &. column 2 no because the command has no nohup/setsid/disown words and no lone &. column 3 no because the command runs in the foreground and the shell exits only after it completes. all three agree. This would be overturned if the command included a trailing & or nohup/setsid/disown.

9 column 1 no because the command does not contain nohup, setsid, disown, or a trailing &. column 2 no because the & is inside quotes and not a lone &. column 3 no because the command runs in the foreground and the shell exits only after it completes. all three agree. This would be overturned if the command included a trailing & or nohup/setsid/disown.

10 column 1 yes because the command contains setsid which is forbidden by Rule 1. column 2 yes because the command contains setsid as a whole word. column 3 yes because the shell exits immediately after starting setsid while the command continues running. all three agree. This would be overturned if the command did not include setsid.

11 column 1 no because the command does not contain nohup, setsid, disown, or a trailing &. column 2 no because the & is part of &> and not a lone &. column 3 no because the command runs in the foreground and the shell exits only after it completes. all three agree. This would be overturned if the command included a trailing & or nohup/setsid/disown.

12 column 1 no because Rule 1 allows starting a background job followed by wait. column 2 no because the word wait appears after the lone &. column 3 no because the shell waits for the background job before exiting. all three agree. This would be overturned if the command did not include wait after the background job.

13 column 1 no because the nohup is inside quotes and not used as a command. column 2 yes because the literal text contains the word nohup as a whole word. column 3 no because the command runs in the foreground and the shell exits immediately after echo. column 2 disagrees with columns 1 and 3. This would be overturned if the nohup were not present in the string or if the hook did not check for words inside quotes.

14 column 1 no because the command does not contain nohup, setsid, disown, or a trailing & in the outer shell command. column 2 no because the & is inside quotes and not a lone &. column 3 yes because the outer shell exits after starting the inner shell while the sleep continues running. column 3 disagrees with columns 1 and 2. This would be overturned if the inner shell waited for the background job or the & was not inside quotes.

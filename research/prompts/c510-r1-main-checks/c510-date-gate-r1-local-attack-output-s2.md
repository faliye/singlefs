CATEGORY_A.shape a file in the external/ directory named technical_manual_2026-04-16.pdf  
This would be refuted by: A file named with an external date outside the allowed window existing in the repository and being accepted by the check.  

CATEGORY_A.legitimacy It is legitimate to archive copies of external documents for reference, even if their dates predate the project's history.  
This would be refuted by: A file named with an external date outside the allowed window existing in the repository and being accepted by the check.  

CATEGORY_A.today does not yet exist but would foreseeably be created by ordinary project work  
This would be refuted by: A file named with an external date outside the allowed window existing in the repository and being accepted by the check.  

CATEGORY_B.shape historical/legacy_report_2026-08-18.txt  
This would be refuted by: A file with a date before the lower bound existing in the repository and being accepted by the check.  

CATEGORY_B.legitimacy The project may need to archive historical data from before its own git history began, such as documents from prior projects or external sources that are part of the project's legacy.  
This would be refuted by: A file with a date before the lower bound existing in the repository and being accepted by the check.  

CATEGORY_B.today does not yet exist but would foreseeably be created by ordinary project work  
This would be refuted by: A file with a date before the lower bound existing in the repository and being accepted by the check.  

CATEGORY_C.shape external/blogs/blog_post_2019-02-08.md  
This would be refuted by: A file with an externally supplied date outside the allowed window existing in the repository and being accepted by the check.  

CATEGORY_C.legitimacy The project may generate artifacts that include external dates as part of their naming, such as when processing or archiving external content, which is legitimate for reference.  
This would be refuted by: A file with an externally supplied date outside the allowed window existing in the repository and being accepted by the check.  

CATEGORY_C.today does not yet exist but would foreseeably be created by ordinary project work  
This would be refuted by: A file with an externally supplied date outside the allowed window existing in the repository and being accepted by the check.  

CATEGORY_D.shape a shallow clone or non-toplevel invocation after the repository's first commit has been rewritten to a date later than the shared toolkit's first commit  
This would be refuted by: The generic function correctly returning the first commit date for shallow clones or non-toplevel directories, so the check does not use the hardcoded fallback.  

CATEGORY_D.legitimacy The fallback mechanism uses a hardcoded date from a different repository which may not match the current repository's history after changes.  
This would be refuted by: The generic function correctly returning the first commit date for shallow clones or non-toplevel directories, so the check does not use the hardcoded fallback.  

CATEGORY_D.today could occur after some future change to the repository's history  
This would be refuted by: The generic function correctly returning the first commit date for shallow clones or non-toplevel directories, so the check does not use the hardcoded fallback.  

CATEGORY_E.shape There is no concrete filename or directory that could cause a timezone-related rejection of a genuine today's date  
This would be refuted by: A date that is confirmed to be the current date in some time zone being rejected by the check because it falls outside the allowed window.  

CATEGORY_E.legitimacy The check's upper bound calculation accounts for all time zones by using UTC+14, and the lower bound is a fixed date unaffected by time zones.  
This would be refuted by: A date that is confirmed to be the current date in some time zone being rejected by the check because it falls outside the allowed window.  

CATEGORY_E.today not applicable  
This would be refuted by: A date that is confirmed to be the current date in some time zone being rejected by the check because it falls outside the allowed window.  

CATEGORY_F.shape No static fixture for a different automated check could legitimately require an out-of-range date in its filename  
This would be refuted by: A static fixture for a different automated check that has a filename with an out-of-range date and is accepted as legitimate by the project's standards.  

CATEGORY_F.legitimacy The project's guidelines (FACT 7) explicitly state that fixtures requiring out-of-range dates should use on-the-fly generation, so static fixtures with such dates are not legitimate.  
This would be refuted by: A static fixture for a different automated check that has a filename with an out-of-range date and is accepted as legitimate by the project's standards.  

CATEGORY_F.today does not exist in the repository  
This would be refuted by: A static fixture for a different automated check that has a filename with an out-of-range date and is accepted as legitimate by the project's standards.

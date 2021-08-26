from typing import Optional


def has_diffs(
    file_a: str,
    file_b: str,
    ignore_comments: Optional[str] = None,
    suspend_on: Optional[str] = None,
    resume_on: Optional[str] = None,
    ignore_blank_lines: bool = True,
) -> bool:
    """
    This function compares the two ASCII files at the given paths and returns True if any
    differences are found between them.

    Blank lines will be ignored by default, so additional blank lines in one file will
    not result in a diff being reported if they are otherwise the same.

    Differences in comments can be ignored by specifying the comment char(s) to be used,
    and also start and end sequences for C-style block comments

    # Examples

    ```python
    # Ignore Python style comments
    has_diffs("file_a.py", "file_b.py", ignore_comments="#")

    # Ignore C++ style comments, including blocks
    has_diffs("file_a.cpp", "file_b.cpp", ignore_comments="//", suspend_on="/*", resume_on="*/")

    ```
    """
    ...

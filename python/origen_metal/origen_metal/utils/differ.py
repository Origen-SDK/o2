from typing import Optional, Tuple, List


def has_diffs(
    file_a: str,
    file_b: str,
    ignore_comments: Optional[List[str]] = None,
    ignore_blocks: Optional[List[Tuple[str, str]]] = None,
    ignore_blank_lines: bool = True,
) -> bool:
    """
    This function compares the two ASCII files at the given paths and returns True if any
    differences are found between them.

    Blank lines will be ignored by default, so additional blank lines in one file will
    not result in a diff being reported if they are otherwise the same.

    Differences in comments can be ignored by specifying the comment char(s) to be used,
    and blocks of content can be ignored (e.g. C-style block comments) by specifying start
    and end characters.
    
    # Examples

    ```python
    # Ignore Python style comments
    has_diffs("file_a.py", "file_b.py", ignore_comments=["#"])

    # Ignore C++ style comments, including blocks
    has_diffs("file_a.cpp", "file_b.cpp", ignore_comments=["//"], ignore_blocks=[("/*", "*/")])
    
    # Multiple entries can be given to both the ignore_comments and ignore_blocks arguments
    has_diffs("file_a.cpp",
              "file_b.cpp",
              ignore_comments=["//", "#"],
              ignore_blocks=[("/*", "*/"), ("{{", "}}")])
    ```
    """
    ...

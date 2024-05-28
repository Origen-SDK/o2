import pytest
from pathlib import Path

# Have pytest's assert rewriting take over:
# https://docs.pytest.org/en/stable/writing_plugins.html#assertion-rewriting
pytest.register_assert_rewrite("tests.shared")
pytest.register_assert_rewrite("tests.framework.users")

test_dir = Path(__file__).parent
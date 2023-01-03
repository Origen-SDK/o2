import pytest
from tests.test_configs import Common as ConfigCommon

class CLICommon(ConfigCommon):
    @pytest.fixture
    def cmd(self):
        return self._cmd

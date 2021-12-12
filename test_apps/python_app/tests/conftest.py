import os
from ._shared import tmp_dir

# Move the session store into a local test directory
os.environ['origen_session__user_root'] = str(tmp_dir())
os.environ['origen_app_app_session_root'] = str(tmp_dir())

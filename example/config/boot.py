import sys
import origen
from pathlib import Path

# Add app's lib directory to the load path
app_lib = Path(__file__).absolute().parent.parent.joinpath("lib")
sys.path.insert(0, str(app_lib))

# Load application
import example

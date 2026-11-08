"""E2E steps share the common bindings through one explicit import."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "steps"))
import common  # noqa: F401,E402

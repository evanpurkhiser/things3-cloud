import os
import time
from collections.abc import Callable

import pytest

os.environ["TZ"] = "UTC"
time.tzset()

from things_cloud.store import ThingsStore

from tests.helpers import build_store_from_journal


@pytest.fixture
def store_from_journal() -> Callable[[list[dict]], ThingsStore]:
    return build_store_from_journal

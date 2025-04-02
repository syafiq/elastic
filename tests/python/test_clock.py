import time
import os
import unittest
from dataclasses import dataclass
from typing import Optional, Union

@dataclass
class ClockError:
    type: str
    message: str

class Clock:
    def __init__(self):
        self.monotonic_start: Optional[float] = None

    def read_current_time(self) -> Union[int, ClockError]:
        try:
            return int(time.time())
        except Exception as e:
            return ClockError("system-time-error", str(e))

    def read_timezone(self) -> Union[str, ClockError]:
        try:
            return os.environ.get('TZ', 'UTC')
        except Exception as e:
            return ClockError("timezone-error", str(e))

    def start_monotonic(self) -> Union[None, ClockError]:
        try:
            self.monotonic_start = time.monotonic()
            return None
        except Exception as e:
            return ClockError("monotonic-clock-error", str(e))

    def stop_monotonic(self) -> Union[int, ClockError]:
        if self.monotonic_start is None:
            return ClockError("monotonic-clock-error", "Monotonic clock not started")
        try:
            elapsed = (time.monotonic() - self.monotonic_start) * 1000  # Convert to milliseconds
            self.monotonic_start = None
            return int(elapsed)
        except Exception as e:
            return ClockError("monotonic-clock-error", str(e))

    def read_monotonic(self) -> Union[int, ClockError]:
        if self.monotonic_start is None:
            return ClockError("monotonic-clock-error", "Monotonic clock not started")
        try:
            elapsed = (time.monotonic() - self.monotonic_start) * 1000  # Convert to milliseconds
            return int(elapsed)
        except Exception as e:
            return ClockError("monotonic-clock-error", str(e))

class TestClock(unittest.TestCase):
    def setUp(self):
        self.clock = Clock()

    def test_read_current_time(self):
        result = self.clock.read_current_time()
        self.assertIsInstance(result, int)
        self.assertGreater(result, 0)

    def test_read_timezone(self):
        result = self.clock.read_timezone()
        self.assertIsInstance(result, str)
        self.assertIn(result, ['UTC', 'GMT', 'EST', 'PST'])  # Common timezones

    def test_monotonic_clock(self):
        # Test start
        result = self.clock.start_monotonic()
        self.assertIsNone(result)

        # Test read while running
        time.sleep(0.1)  # Sleep for 100ms
        result = self.clock.read_monotonic()
        self.assertIsInstance(result, int)
        self.assertGreaterEqual(result, 100)  # Should be at least 100ms

        # Test stop
        result = self.clock.stop_monotonic()
        self.assertIsInstance(result, int)
        self.assertGreaterEqual(result, 100)  # Should be at least 100ms

    def test_monotonic_clock_errors(self):
        # Test read without start
        result = self.clock.read_monotonic()
        self.assertIsInstance(result, ClockError)
        self.assertEqual(result.type, "monotonic-clock-error")

        # Test stop without start
        result = self.clock.stop_monotonic()
        self.assertIsInstance(result, ClockError)
        self.assertEqual(result.type, "monotonic-clock-error")

if __name__ == '__main__':
    unittest.main() 
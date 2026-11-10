"""The comparison contract must fail closed and require deterministic generation."""

import tempfile
import unittest
from pathlib import Path

from PIL import Image
from screenshots import Screenshots

COMPARISON = {
    "channel_delta": 0,
    "max_changed_pixels": 0,
    "determinism_channel_delta": 0,
    "determinism_max_changed_pixels": 0,
}


class ScreenshotContractTests(unittest.TestCase):
    def test_verify_does_not_create_a_missing_golden(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runner = Screenshots(
                root / "actual", root / "goldens", "verify", COMPARISON
            )
            frame = Image.new("RGB", (40, 40), "black")

            runner.compare({"id": "missing"}, frame, frame.copy())

            with self.assertRaisesRegex(AssertionError, "Missing reviewed golden"):
                runner.finish()
            self.assertFalse((root / "goldens").exists())

    def test_generate_rejects_even_one_nondeterministic_pixel(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runner = Screenshots(
                root / "actual", root / "goldens", "generate", COMPARISON
            )
            first = Image.new("RGB", (40, 40), "black")
            repeat = first.copy()
            repeat.putpixel((1, 1), (1, 0, 0))

            runner.compare({"id": "unstable"}, first, repeat)

            with self.assertRaisesRegex(AssertionError, "Nondeterministic"):
                runner.finish()
            self.assertFalse((root / "goldens").exists())

    def test_generate_then_verify_rejects_one_changed_pixel_without_overwriting(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            frame = Image.new("RGB", (40, 40), "black")
            generate = Screenshots(
                root / "actual", root / "goldens", "generate", COMPARISON
            )
            generate.compare({"id": "icon"}, frame, frame.copy())
            generate.finish()
            golden = root / "goldens" / generate.platform / "icon.png"
            original = golden.read_bytes()
            changed = frame.copy()
            changed.putpixel((1, 1), (1, 0, 0))
            verify = Screenshots(
                root / "actual", root / "goldens", "verify", COMPARISON
            )

            verify.compare({"id": "icon"}, changed, changed.copy())

            with self.assertRaisesRegex(AssertionError, "Visual regression"):
                verify.finish()
            self.assertEqual(golden.read_bytes(), original)
            self.assertTrue(
                (root / "actual" / verify.platform / "icon.diff.png").exists()
            )


if __name__ == "__main__":
    unittest.main()

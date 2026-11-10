"""The comparison contract must fail closed and require deterministic generation."""

import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw
from screenshots import Screenshots

COMPARISON = {
    "perceptual_threshold": 0.01,
    "include_antialiasing": False,
    "max_changed_pixels": 0,
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
            repeat.putpixel((1, 1), (255, 255, 255))

            runner.compare({"id": "unstable"}, first, repeat)

            with self.assertRaisesRegex(AssertionError, "Nondeterministic"):
                runner.finish()
            self.assertFalse((root / "goldens").exists())

    def test_verify_rejects_pixel_icon_and_size_changes_without_overwriting(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            frame = Image.new("RGB", (40, 40), "black")
            generate = Screenshots(
                root / "actual", root / "goldens", "generate", COMPARISON
            )
            ImageDraw.Draw(frame).rectangle((12, 8, 14, 30), fill="white")
            generate.compare({"id": "icon"}, frame, frame.copy())
            generate.finish()
            golden = root / "goldens" / generate.platform / "icon.png"
            original = golden.read_bytes()
            pixel = frame.copy()
            pixel.putpixel((1, 1), (255, 255, 255))
            translated = Image.new("RGB", frame.size, "black")
            ImageDraw.Draw(translated).rectangle((13, 8, 15, 30), fill="white")

            for changed in (pixel, translated, frame.resize((41, 40))):
                with self.subTest(size=changed.size):
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

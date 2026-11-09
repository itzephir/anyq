"""Check the receiver's archive boundary, atomic rollback and stand isolation."""

import io
import stat
import tempfile
import unittest
import zipfile
from pathlib import Path

from receive_website import REQUIRED, publish, unpack


def bundle(text=b"first", extra=None):
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w") as archive:
        for name in sorted(REQUIRED):
            archive.writestr(name, text if name == "index.html" else b"asset")
        if extra:
            archive.writestr(*extra)
    return output.getvalue()


class ReceiverWebsiteTest(unittest.TestCase):
    def test_atomic_updates_keep_previous_release_and_other_stand(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def ready(*_args):
                return None

            publish("dev", "a" * 40, bundle(), root, ready)
            publish("prod", "b" * 40, bundle(b"production"), root, ready)
            publish("dev", "c" * 40, bundle(b"next"), root, ready)

            self.assertEqual((root / "dev/current/index.html").read_bytes(), b"next")
            self.assertEqual(
                (root / "prod/current/index.html").read_bytes(), b"production"
            )
            self.assertEqual(
                (root / "dev/releases" / ("a" * 40) / "index.html").read_bytes(),
                b"first",
            )

            def failed(*_args):
                raise RuntimeError("HTTP failed")

            with self.assertRaises(RuntimeError):
                publish("dev", "d" * 40, bundle(b"broken"), root, failed)
            self.assertEqual((root / "dev/current/index.html").read_bytes(), b"next")
            self.assertEqual(
                (root / "prod/current/index.html").read_bytes(), b"production"
            )

    def test_repeated_revision_is_idempotent_but_cannot_change_contents(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            publish("dev", "a" * 40, bundle(), root, lambda *_: None)
            publish("dev", "a" * 40, bundle(), root, lambda *_: None)
            with self.assertRaises(ValueError):
                publish("dev", "a" * 40, bundle(b"different"), root, lambda *_: None)
            self.assertEqual((root / "dev/current/index.html").read_bytes(), b"first")

    def test_invalid_archives_never_escape_or_replace_existing_files(self):
        for name in (
            "../escape",
            "/absolute",
            "assets/../../escape",
            ".ssh/authorized_keys",
            "revision.txt",
            "assets\\escape.ttf",
        ):
            with self.subTest(name=name), self.assertRaises(ValueError):
                unpack(bundle(extra=(name, b"bad")))

        link = zipfile.ZipInfo("assets/link.ttf")
        link.create_system = 3
        link.external_attr = (stat.S_IFLNK | 0o777) << 16
        with self.assertRaises(ValueError):
            unpack(bundle(extra=(link, b"/etc/passwd")))
        with self.assertRaises(ValueError):
            unpack(bundle(extra=("assets/large.ttf", b"x" * (16 * 1024 * 1024))))

    def test_failed_first_release_leaves_no_active_stand(self):
        with tempfile.TemporaryDirectory() as directory:

            def failed(*_args):
                raise RuntimeError("HTTP failed")

            with self.assertRaises(RuntimeError):
                publish("dev", "a" * 40, bundle(), Path(directory), failed)
            self.assertFalse((Path(directory) / "dev/current").exists())
            self.assertFalse((Path(directory) / "prod").exists())

    def test_invalid_stand_and_revision_are_rejected_before_writing(self):
        with tempfile.TemporaryDirectory() as directory:
            for stand, revision in (("../prod", "a" * 40), ("dev", "../../bad")):
                with self.assertRaises(ValueError):
                    publish(stand, revision, bundle(), Path(directory), lambda *_: None)
            self.assertEqual(list(Path(directory).iterdir()), [])


if __name__ == "__main__":
    unittest.main()

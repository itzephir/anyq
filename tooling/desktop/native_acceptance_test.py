"""Reject the native acceptance false positives found during PR review."""

import json
import unittest
from pathlib import Path

import document_observation
import popup_geometry


class NativeAcceptanceContractTests(unittest.TestCase):
    def test_popup_rejects_tiny_size_wrong_anchor_and_wrong_dpi(self):
        monitor = {
            "x": 0,
            "y": 0,
            "width": 1920,
            "height": 1040,
            "scale": 1,
            "bounds": {"x": 0, "y": 0, "width": 1920, "height": 1080},
        }
        icon = {"x": 1800, "y": 1040, "width": 24, "height": 40}
        valid = {
            "client_position": {"x": 1532, "y": 677},
            "client_size": {"width": 380, "height": 355},
            "scale": 1,
        }
        popup_geometry.verify(valid, icon, [monitor], logical_coordinates=False)

        for mutation in (
            {"client_size": {"width": 1, "height": 1}},
            {"client_position": {"x": 8, "y": 8}},
            {"client_position": {"x": 1540, "y": 678}},
            {"scale": 1.5},
        ):
            with self.subTest(mutation=mutation):
                with self.assertRaises(AssertionError):
                    popup_geometry.verify(
                        valid | mutation, icon, [monitor], logical_coordinates=False
                    )

    def test_popup_accepts_scaled_negative_origin_and_retina_coordinates(self):
        physical = {
            "x": -1920,
            "y": -80,
            "width": 1920,
            "height": 1040,
            "scale": 1.5,
            "bounds": {"x": -1920, "y": -120, "width": 1920, "height": 1080},
        }
        icon = {"x": -1040, "y": -120, "width": 40, "height": 40}
        window = {
            "client_position": {"x": -1305, "y": -68},
            "client_size": {"width": 570, "height": 533},
            "scale": 1.5,
        }
        popup_geometry.verify(window, icon, [physical], logical_coordinates=False)

        retina = {
            "x": 0,
            "y": 24,
            "width": 1280,
            "height": 936,
            "scale": 2,
            "bounds": {"x": 0, "y": 0, "width": 1280, "height": 960},
        }
        icon = {"x": 1100, "y": 0, "width": 34, "height": 24}
        window = {
            "client_position": {"x": 1784, "y": 64},
            "client_size": {"width": 760, "height": 710},
            "scale": 2,
        }
        popup_geometry.verify(window, icon, [retina], logical_coordinates=True)

    def test_installed_document_rejects_blank_stale_and_wrong_slot_states(self):
        root = Path(__file__).resolve().parents[2]
        plan = json.loads((root / "tests/desktop/config.json").read_text())[
            "package_observation"
        ]
        prior = ""
        for index, stage in enumerate(plan["stages"]):
            expected = stage | {"required": plan["required"] + stage["required"]}
            document = "\n".join(expected["required"])
            document_observation.verify(document, expected)

            for invalid in (
                "",
                "anyq daemon is running",
                prior,
                document.replace(stage["required"][-1], "99/99"),
                document.replace("package-active", "missing-task"),
            ):
                with self.subTest(stage=index, invalid=invalid):
                    with self.assertRaises(AssertionError):
                        document_observation.verify(invalid, expected)
            prior = document

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "chunk_fragmentation.py"


def load_module():
    scripts_dir = str(MODULE_PATH.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location("chunk_fragmentation", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ChunkFragmentationTests(unittest.TestCase):
    def test_fragment_chunk_uses_item_lengths_to_avoid_repeated_payload_serialization(self) -> None:
        module = load_module()

        chunk = {
            "id": "0_0",
            "terrain": {
                "cellSizeStuds": 2,
                "width": 64,
                "depth": 64,
                "heights": list(range(256)),
                "materials": ["Grass"] * 256,
            },
            "roads": [
                {"kind": "residential", "points": [{"x": float(index), "y": 0.0, "z": 0.0}]}
                for index in range(32)
            ],
        }

        payload_len_calls = 0
        item_len_calls = 0

        def payload_len(payload: object) -> int:
            nonlocal payload_len_calls
            payload_len_calls += 1
            return len(json.dumps(payload, separators=(",", ":")).encode("utf-8"))

        def item_len(value: object) -> int:
            nonlocal item_len_calls
            item_len_calls += 1
            return len(json.dumps(value, separators=(",", ":")).encode("utf-8"))

        fragments = module.fragment_chunk_for_lua_shards(
            chunk,
            400,
            lua_len_fn=payload_len,
            lua_value_len_fn=item_len,
            chunk_label="runtime chunk",
        )

        self.assertGreater(len(fragments), 4)
        self.assertLessEqual(
            payload_len_calls,
            6,
            f"expected payload serialization to stay bounded, got {payload_len_calls} calls",
        )
        self.assertGreater(item_len_calls, 32)


if __name__ == "__main__":
    unittest.main()

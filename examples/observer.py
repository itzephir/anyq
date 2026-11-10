#!/usr/bin/env python3
"""Minimal explicitly enabled protocol-v1 observer; Python is example tooling."""

import json
import sys


def send(frame):
    print(json.dumps(frame, separators=(",", ":")), flush=True)


def main():
    count = 0
    for line in sys.stdin:
        frame = json.loads(line)
        if frame["type"] == "hello":
            for key, value in frame["state"]["object"]:
                if key == "observed":
                    count = int(value["number"])
            send(
                {
                    "type": "hello",
                    "version": 1,
                    "subscriptions": ["task.running"],
                    "capabilities": ["state"],
                }
            )
        elif frame["type"] == "event":
            count += 1
            send(
                {
                    "type": "ack",
                    "delivery_id": frame["delivery_id"],
                    "state": {"object": [["observed", {"number": str(count)}]]},
                    "timers": [],
                }
            )
        elif frame["type"] == "stop":
            return


if __name__ == "__main__":
    main()

#!/usr/bin/env python3

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Dict, Iterable, Set


DiffLines = Dict[str, Set[int]]
CoverageLines = Dict[str, Set[int]]


def parse_diff(diff_text: str) -> DiffLines:
    diff_lines: DiffLines = {}
    current_file = None
    for line in diff_text.splitlines():
        if line.startswith("+++ b/"):
            current_file = line[6:]
            diff_lines.setdefault(current_file, set())
            continue
        if not line.startswith("@@") or current_file is None:
            continue

        match = re.search(r"\+(\d+)(?:,(\d+))?", line)
        if match is None:
            continue
        start = int(match.group(1))
        count = int(match.group(2) or "1")
        if count == 0:
            continue
        for line_number in range(start, start + count):
            diff_lines[current_file].add(line_number)
    return diff_lines


def parse_uncovered_lines(lcov_text: str, targets: Iterable[str]) -> CoverageLines:
    target_set = set(targets)
    uncovered: CoverageLines = {target: set() for target in target_set}
    current_target = None

    for raw_line in lcov_text.splitlines():
        line = raw_line.strip()
        if line.startswith("SF:"):
            source_path = line[3:]
            current_target = next(
                (target for target in target_set if source_path.endswith(target)),
                None,
            )
            continue
        if current_target is not None and line.startswith("DA:"):
            line_number_text, hits_text = line[3:].split(",", 1)
            if int(hits_text) == 0:
                uncovered[current_target].add(int(line_number_text))
            continue
        if line == "end_of_record":
            current_target = None

    return uncovered


def nearest_context(file_path: Path, line_number: int) -> str:
    lines = file_path.read_text(encoding="utf-8").splitlines()
    for index in range(min(line_number - 1, len(lines) - 1), -1, -1):
        candidate = lines[index].strip()
        if candidate.startswith("fn ") or " fn " in candidate:
            return candidate
        if candidate.startswith("if ") or candidate.startswith("match "):
            return candidate
    return "Unknown context"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Intersect changed diff lines with uncovered LCOV lines for patch coverage triage.",
    )
    parser.add_argument(
        "--lcov",
        default="lcov.info",
        help="Path to the LCOV report. Defaults to lcov.info.",
    )
    parser.add_argument(
        "--diff-file",
        help="Optional path to a diff file. If omitted, the script reads the diff from stdin.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit machine-readable JSON instead of text output.",
    )
    parser.add_argument(
        "files",
        nargs="+",
        help="Repository-relative files to inspect.",
    )
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    lcov_path = Path(args.lcov)
    if not lcov_path.exists():
        parser.error(f"LCOV file not found: {args.lcov}")

    if args.diff_file:
        diff_path = Path(args.diff_file)
        if not diff_path.exists():
            parser.error(f"Diff file not found: {args.diff_file}")
        diff_text = diff_path.read_text(encoding="utf-8")
    else:
        diff_text = sys.stdin.read()

    diff_lines = parse_diff(diff_text)
    uncovered_lines = parse_uncovered_lines(
        lcov_path.read_text(encoding="utf-8"),
        args.files,
    )

    results = []
    for target in args.files:
        intersections = sorted(diff_lines.get(target, set()) & uncovered_lines.get(target, set()))
        file_result = {
            "file": target,
            "uncovered_patch_lines": [
                {
                    "line": line_number,
                    "context": nearest_context(Path(target), line_number),
                }
                for line_number in intersections
            ],
        }
        results.append(file_result)

    if args.json:
        print(json.dumps(results, indent=2))
        return 0

    for result in results:
        if not result["uncovered_patch_lines"]:
            continue
        print(f"\nFile: {result['file']}")
        for item in result["uncovered_patch_lines"]:
            print(f"Line {item['line']}: {item['context']}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

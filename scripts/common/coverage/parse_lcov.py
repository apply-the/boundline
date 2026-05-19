#!/usr/bin/env python3

import argparse
from pathlib import Path
from typing import Dict


LineHits = Dict[int, int]
CoverageMap = Dict[str, LineHits]


def parse_lcov(lcov_path: Path, targets: list[str]) -> CoverageMap:
    target_suffixes = tuple(targets)
    coverage: CoverageMap = {target: {} for target in targets}
    current_target = None

    with lcov_path.open("r", encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if line.startswith("SF:"):
                current_path = line[3:]
                current_target = next(
                    (target for target in target_suffixes if current_path.endswith(target)),
                    None,
                )
                continue
            if line.startswith("DA:") and current_target is not None:
                line_number_text, hits_text = line[3:].split(",", 1)
                line_number = int(line_number_text)
                hits = int(hits_text)
                current_hits = coverage[current_target]
                current_hits[line_number] = current_hits.get(line_number, 0) + hits
                continue
            if line == "end_of_record":
                current_target = None

    return coverage


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Print per-file LCOV coverage for selected repository files.",
    )
    parser.add_argument("lcov_file", help="Path to the LCOV report.")
    parser.add_argument(
        "targets",
        nargs="+",
        help="Repository-relative file paths to inspect.",
    )
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    lcov_path = Path(args.lcov_file)
    if not lcov_path.exists():
        parser.error(f"LCOV file not found: {args.lcov_file}")

    coverage = parse_lcov(lcov_path, args.targets)
    for target in args.targets:
        line_hits = coverage[target]
        total_lines = len(line_hits)
        if total_lines == 0:
            print(f"{target}: no coverage data found")
            continue
        covered_lines = sum(1 for hits in line_hits.values() if hits > 0)
        percentage = covered_lines / total_lines * 100.0
        print(f"{target}: {percentage:.2f}% ({covered_lines}/{total_lines})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

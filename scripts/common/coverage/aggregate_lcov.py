#!/usr/bin/env python3

import argparse
from pathlib import Path
from typing import Dict

CoverageMap = Dict[str, Dict[int, int]]


def parse_lcov_file(lcov_path: Path, coverage: CoverageMap) -> None:
    current_file = None
    with lcov_path.open("r", encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if line.startswith("SF:"):
                current_file = str(Path(line[3:]).resolve())
                coverage.setdefault(current_file, {})
                continue
            if line.startswith("DA:") and current_file is not None:
                line_number_text, hits_text = line[3:].split(",", 1)
                line_number = int(line_number_text)
                hits = int(hits_text)
                file_coverage = coverage.setdefault(current_file, {})
                file_coverage[line_number] = file_coverage.get(line_number, 0) + hits
                continue
            if line == "end_of_record":
                current_file = None


def summarize(coverage: CoverageMap, targets: list[str]) -> list[tuple[str, int, int, float]]:
    rows = []
    for target in targets:
        target_path = str(Path(target).resolve())
        merged_hits: Dict[int, int] = {}
        for candidate_path, candidate_hits in coverage.items():
            if candidate_path == target_path or candidate_path.endswith(target):
                for line_number, hits in candidate_hits.items():
                    merged_hits[line_number] = merged_hits.get(line_number, 0) + hits

        total_lines = len(merged_hits)
        covered_lines = sum(1 for hits in merged_hits.values() if hits > 0)
        percentage = (covered_lines / total_lines * 100.0) if total_lines else 0.0
        rows.append((target, covered_lines, total_lines, percentage))
    return rows


def lcov_output_path(source_path: str, repo_root: Path) -> str:
    path = Path(source_path)
    try:
        return str(path.relative_to(repo_root))
    except ValueError:
        return source_path


def write_merged_lcov(coverage: CoverageMap, output_path: Path, repo_root: Path) -> None:
    with output_path.open("w", encoding="utf-8") as handle:
        for source_path in sorted(coverage):
            handle.write(f"SF:{lcov_output_path(source_path, repo_root)}\n")
            for line_number, hits in sorted(coverage[source_path].items()):
                handle.write(f"DA:{line_number},{hits}\n")
            handle.write("end_of_record\n")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Aggregate one or more LCOV reports and summarize coverage for target files.",
    )
    parser.add_argument(
        "lcov_files",
        nargs="+",
        help="One or more LCOV files to aggregate.",
    )
    parser.add_argument(
        "targets",
        nargs="*",
        help="Optional repository-relative file paths to summarize.",
    )
    parser.add_argument(
        "--output-lcov",
        help="Optional path to write a merged LCOV file built from the input reports.",
    )
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    if not args.targets and not args.output_lcov:
        parser.error("provide at least one target to summarize or set --output-lcov")

    coverage: CoverageMap = {}
    for lcov_file in args.lcov_files:
        lcov_path = Path(lcov_file)
        if not lcov_path.exists():
            parser.error(f"LCOV file not found: {lcov_file}")
        parse_lcov_file(lcov_path, coverage)

    if args.output_lcov:
        write_merged_lcov(coverage, Path(args.output_lcov), Path.cwd().resolve())

    if args.targets:
        rows = summarize(coverage, args.targets)
        print(f"{'File':<50} {'Covered':>8} {'Total':>8} {'Percent':>8}")
        print("-" * 80)
        for target, covered, total, percentage in rows:
            print(f"{target:<50} {covered:>8} {total:>8} {percentage:>7.2f}%")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

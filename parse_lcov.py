import sys
import os

target_files = [
    "src/cli/run.rs",
    "src/domain/governance.rs",
    "src/domain/reasoning.rs",
    "src/fixture.rs",
    "src/orchestrator/engine.rs",
    "src/orchestrator/governance.rs",
    "src/orchestrator/review_trace.rs",
    "src/orchestrator/session_runtime.rs"
]

coverage_data = {f: {} for f in target_files}

def parse_lcov(file_path):
    current_file = None
    with open(file_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith("SF:"):
                path = line[3:]
                current_file = None
                for target in target_files:
                    if path.endswith(target):
                        current_file = target
                        break
            elif line.startswith("DA:") and current_file:
                parts = line[3:].split(',')
                if len(parts) >= 2:
                    line_num = int(parts[0])
                    hits = int(parts[1])
                    if line_num not in coverage_data[current_file]:
                        coverage_data[current_file][line_num] = 0
                    coverage_data[current_file][line_num] += hits
            elif line == "end_of_record":
                current_file = None

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python parse_lcov.py <lcov_file>")
        sys.exit(1)
    
    parse_lcov(sys.argv[1])
    
    for file in target_files:
        lines = coverage_data[file]
        total_lines = len(lines)
        if total_lines == 0:
            print(f"{file}: No coverage data found")
            continue
        covered_lines = sum(1 for hits in lines.values() if hits > 0)
        percentage = (covered_lines / total_lines) * 100
        print(f"{file}: {percentage:.2f}% ({covered_lines}/{total_lines})")

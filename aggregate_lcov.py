import sys
import os

def aggregate(lcov_path, target_files):
    if not os.path.exists(lcov_path):
        print(f"Error: {lcov_path} not found")
        return

    current_file = None
    # Map from absolute path to {line_number: hits}
    coverage_data = {}

    with open(lcov_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('SF:'):
                current_file = line[3:]
                # Resolve relative paths or normalized paths if needed,
                # but lcov.info usually has relative or absolute paths.
                # We'll normalize to absolute paths for mapping.
                current_file = os.path.abspath(current_file)
                if current_file not in coverage_data:
                    coverage_data[current_file] = {}
            elif line.startswith('DA:'):
                if current_file:
                    parts = line[3:].split(',')
                    line_num = int(parts[                        hits = int(parts[1])
                    coverage_data[current_file][line_num] = coverage_data[current_file].get(line_num, 0) + hits
            elif line == 'end_of_record':
                current_file = None

    results = []
    base_dir = os.getcwd()
    for target in target_files:
        # Construct absolute path for target
        abs_target = os.path.abspath(target)
        
        # Sometimes paths in lcov.info might be relative to workspace or crate roots.
        # We search coverage_data keys for the target suffix.
        matching_data = {}
        for path, data in coverage_data.items():
            if path.endswith(target):
                matching_data.update(data)
        
        if not matching_data:
            results.append((target, 0, 0, 0.0))
            continue
            
        total_lines = len(matching_data)
        covered_lines = sum(1 for hits in matching_data.values() if hits > 0)
        percentage = (covered_lines / total_lines * 100) if total_lines > 0 else 0.0
        results.append((target, covered_lines, total_lines, percentage))

    print(f"{'File':<40} {'Covered':<10} {'Total':<10} {'%':<10}")
    print("-" * 75)
    for res in results:
        print(f"{res[0]:<40} {res[1]:<10} {res[2]:<10} {res[3]:<10.2f}")

if __name__ == "__main__":
    targets = [
        "src/cli/run.rs",
        "src/domain/governance.rs",
        "src/domain/reasoning.rs",
        "src/fixture.rs",
        "src/orchestrator/engine.rs",
        "src/orchestrator/governance.rs",
        "src/orchestrator/review_trace.rs",
        "src/orchestrator/session_runtime.rs"
    ]
    aggregate("lcov.info", targets)

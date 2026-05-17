import sys
import os

def parse_lcov(filename):
    files = []
    current_file = None
    with open(filename, "r") as f:
        for line in f:
            line = line.strip()
            if line.startswith("SF:"):
                # Handle the weird pathing in the lcov file
                path = line[3:]
                if "../../../" in path:
                    path = path.split("../../../")[-1]
                current_file = {"path": path, "hits": 0, "total": 0, "uncovered": []}
            elif line.startswith("DA:"):
                line_info = line[3:].split(",")
                line_num = int(line_info[0])
                hits = int(line_info[1])
                current_file["total"] += 1
                if hits > 0:
                    current_file["hits"] += 1
                else:
                    current_file["uncovered"].append(line_num)
            elif line == "end_of_record":
                if current_file:
                    files.append(current_file)
                current_file = None
    return files

coverage_threshold = 0.95
files = parse_lcov("lcov.info")

# deduplicate by path (keep the one with more coverage or first)
unique_files = {}
for f in files:
    path = f["path"]
    if path not in unique_files or unique_files[path]["hits"]/unique_files[path]["total"] < f["hits"]/f["total"]:
        unique_files[path] = f

for path, f in sorted(unique_files.items()):
    if f["total"] > 0:
        pct = f["hits"] / f["total"]
        if pct < coverage_threshold:
            uncovered_range = ""
            if f["uncovered"]:
                nums = sorted(f["uncovered"])
                ranges = []
                if nums:
                    start = nums[0]
                    prev = nums[0]
                    for n in nums[1:]:
                        if n == prev + 1:
                            prev = n
                        else:
                            ranges.append(f"{start}-{prev}" if start != prev else f"{start}")
                            start = n
                            prev = n
                    ranges.append(f"{start}-{prev}" if start != prev else f"{start}")
                uncovered_range = ",".join(ranges)
            
            print(f"{path} | {pct:.2%} | {uncovered_range}")

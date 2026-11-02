#!/usr/bin/env python3
"""Convert llvm-cov JSON export to a Cobertura XML report.

The JSON exporter keeps branch instrumentation data, while cargo-llvm-cov's
lcov-based Cobertura conversion crashes on branch regions with LLVM 23.
"""

import json
import sys
import xml.etree.ElementTree as ET


def line_hits(file_entry):
    hits = {}
    for segment in file_entry.get("segments", []):
        line, _column, count, has_count, is_region_entry, _gap = segment[:6]
        if not (has_count and is_region_entry):
            continue
        hits[line] = max(hits.get(line, 0), int(count))
    return hits


def branch_counts(file_entry):
    per_line = {}
    for branch in file_entry.get("branches", []):
        line = int(branch[0])
        executed = int(branch[4])
        fallthrough = int(branch[5])
        valid, covered = per_line.get(line, (0, 0))
        per_line[line] = (
            valid + 2,
            covered + int(executed > 0) + int(fallthrough > 0),
        )
    return per_line


def main(source, output):
    payload = json.load(open(source))
    data = payload["data"][0]

    packages = {}
    total_valid = 0
    total_covered = 0
    branch_valid = 0
    branch_covered = 0

    for entry in data.get("files", []):
        filename = entry["filename"]
        hits = line_hits(entry)
        if not hits:
            continue
        line_branches = branch_counts(entry)
        valid = sum(item[0] for item in line_branches.values())
        covered = sum(item[1] for item in line_branches.values())
        total_valid += len(hits)
        total_covered += sum(1 for value in hits.values() if value > 0)
        branch_valid += valid
        branch_covered += covered
        package_name = filename.rsplit("/", 1)[0] if "/" in filename else "."
        packages.setdefault(package_name, []).append(
            (filename, hits, line_branches, valid, covered)
        )

    if total_valid == 0:
        raise SystemExit("json_to_cobertura: no line coverage found")

    root = ET.Element(
        "coverage",
        {
            "version": "1.9",
            "timestamp": "0",
            "lines-valid": str(total_valid),
            "lines-covered": str(total_covered),
            "line-rate": f"{total_covered / total_valid:.6f}",
            "branches-valid": str(branch_valid),
            "branches-covered": str(branch_covered),
            "branch-rate": f"{branch_covered / branch_valid:.6f}" if branch_valid else "0",
        },
    )
    sources = ET.SubElement(root, "sources")
    ET.SubElement(sources, "source").text = "."
    packages_node = ET.SubElement(root, "packages")

    for package_name in sorted(packages):
        entries = packages[package_name]
        package_valid = sum(len(hits) for _, hits, _, _, _ in entries)
        package_covered = sum(
            1 for _, hits, _, _, _ in entries for value in hits.values() if value > 0
        )
        package_branch_valid = sum(valid for _, _, _, valid, _ in entries)
        package_branch_covered = sum(covered for _, _, _, _, covered in entries)
        package = ET.SubElement(
            packages_node,
            "package",
            {
                "name": package_name,
                "line-rate": f"{package_covered / package_valid:.6f}",
                "branch-rate": (
                    f"{package_branch_covered / package_branch_valid:.6f}"
                    if package_branch_valid
                    else "0"
                ),
                "complexity": "0",
            },
        )
        classes = ET.SubElement(package, "classes")
        for filename, hits, line_branches, valid, covered in sorted(entries):
            file_covered = sum(1 for value in hits.values() if value > 0)
            cls = ET.SubElement(
                classes,
                "class",
                {
                    "name": filename,
                    "filename": filename,
                    "line-rate": f"{file_covered / len(hits):.6f}",
                    "branch-rate": f"{covered / valid:.6f}" if valid else "0",
                    "complexity": "0",
                },
            )
            ET.SubElement(cls, "methods")
            lines = ET.SubElement(cls, "lines")
            for number in sorted(hits):
                attributes = {
                    "number": str(number),
                    "hits": str(hits[number]),
                    "branch": "false",
                }
                branch_valid, branch_covered = line_branches.get(number, (0, 0))
                if branch_valid:
                    attributes["branch"] = "true"
                    attributes["condition-coverage"] = (
                        f"{100 * branch_covered // branch_valid}% "
                        f"({branch_covered}/{branch_valid})"
                    )
                ET.SubElement(lines, "line", attributes)

    ET.ElementTree(root).write(output, encoding="utf-8", xml_declaration=True)
    print(f"cobertura: {total_covered}/{total_valid} lines -> {output}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: json_to_cobertura.py <llvm-cov.json> <cobertura.xml>")
    main(sys.argv[1], sys.argv[2])

#!/usr/bin/env bash
# Merge the regular and @e2e Cobertura reports into one union report.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
non_e2e="${GIVN_COVERAGE_NON_E2E_PATH:-$root/coverage/non-e2e-cobertura.xml}"
e2e="${GIVN_COVERAGE_E2E_PATH:-$root/coverage/e2e-cobertura.xml}"
output="${GIVN_COVERAGE_OUTPUT_PATH:-$root/coverage/cobertura-coverage.xml}"
mkdir -p "$(dirname "$output")"

python3 - "$non_e2e" "$e2e" "$output" <<'PY'
import re
import sys
import xml.etree.ElementTree as ET

non_e2e_path, e2e_path, output_path = sys.argv[1:4]
condition = re.compile(r"\((\d+)/(\d+)\)")

# filename -> line -> [hits, branch_covered, branch_valid]
lines = {}
sources = []

for path in (non_e2e_path, e2e_path):
    root = ET.parse(path).getroot()
    for source in root.findall("./sources/source"):
        if source.text and source.text not in sources:
            sources.append(source.text)
    for cls in root.findall("./packages/package/classes/class"):
        filename = cls.get("filename")
        if not filename:
            continue
        for line in cls.findall("./lines/line"):
            number = int(line.get("number"))
            hits = int(line.get("hits", "0"))
            covered = valid = 0
            match = condition.search(line.get("condition-coverage", ""))
            if match:
                covered, valid = int(match.group(1)), int(match.group(2))
            slot = lines.setdefault(filename, {}).setdefault(number, [0, 0, 0])
            slot[0] = max(slot[0], hits)
            slot[1] = max(slot[1], covered)
            slot[2] = max(slot[2], valid)

if not lines:
    raise SystemExit("merge: no line coverage found in input reports")

total = sum(len(file_lines) for file_lines in lines.values())
covered = sum(
    1 for file_lines in lines.values() for value in file_lines.values() if value[0] > 0
)
branch_valid = sum(value[2] for file_lines in lines.values() for value in file_lines.values())
branch_covered = sum(
    value[1] for file_lines in lines.values() for value in file_lines.values()
)

root = ET.Element(
    "coverage",
    {
        "version": "1.9",
        "timestamp": "0",
        "lines-valid": str(total),
        "lines-covered": str(covered),
        "line-rate": f"{covered / total:.6f}",
        "branches-valid": str(branch_valid),
        "branches-covered": str(branch_covered),
        "branch-rate": f"{branch_covered / branch_valid:.6f}" if branch_valid else "0",
    },
)
sources_node = ET.SubElement(root, "sources")
for source in sources or ["."]:
    ET.SubElement(sources_node, "source").text = source
packages = ET.SubElement(root, "packages")

by_package = {}
for filename in lines:
    package = filename.rsplit("/", 1)[0] if "/" in filename else "."
    by_package.setdefault(package, []).append(filename)

for package_name, files in sorted(by_package.items()):
    package_covered = sum(
        1
        for filename in files
        for value in lines[filename].values()
        if value[0] > 0
    )
    package_total = sum(len(lines[filename]) for filename in files)
    package_branch_valid = sum(
        value[2] for filename in files for value in lines[filename].values()
    )
    package_branch_covered = sum(
        value[1] for filename in files for value in lines[filename].values()
    )
    package = ET.SubElement(
        packages,
        "package",
        {
            "name": package_name,
            "line-rate": f"{package_covered / package_total:.6f}",
            "branch-rate": (
                f"{package_branch_covered / package_branch_valid:.6f}"
                if package_branch_valid
                else "0"
            ),
            "complexity": "0",
        },
    )
    classes = ET.SubElement(package, "classes")
    for filename in sorted(files):
        file_lines = lines[filename]
        file_covered = sum(1 for value in file_lines.values() if value[0] > 0)
        file_branch_valid = sum(value[2] for value in file_lines.values())
        file_branch_covered = sum(value[1] for value in file_lines.values())
        cls = ET.SubElement(
            classes,
            "class",
            {
                "name": filename,
                "filename": filename,
                "line-rate": f"{file_covered / len(file_lines):.6f}",
                "branch-rate": (
                    f"{file_branch_covered / file_branch_valid:.6f}"
                    if file_branch_valid
                    else "0"
                ),
                "complexity": "0",
            },
        )
        ET.SubElement(cls, "methods")
        lines_node = ET.SubElement(cls, "lines")
        for number in sorted(file_lines):
            hits, branch_covered_line, branch_valid_line = file_lines[number]
            attributes = {
                "number": str(number),
                "hits": str(hits),
                "branch": "false",
            }
            if branch_valid_line:
                attributes["branch"] = "true"
                attributes["condition-coverage"] = (
                    f"{100 * branch_covered_line // branch_valid_line}% "
                    f"({branch_covered_line}/{branch_valid_line})"
                )
            ET.SubElement(lines_node, "line", attributes)

ET.ElementTree(root).write(output_path, encoding="utf-8", xml_declaration=True)
print(
    f"merged coverage: {covered}/{total} lines, "
    f"{branch_covered}/{branch_valid} branches -> {output_path}"
)
PY

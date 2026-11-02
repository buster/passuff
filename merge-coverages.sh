#!/usr/bin/env bash
# Merge the regular and @e2e Cobertura reports into one union report.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
non_e2e="${GIVN_COVERAGE_NON_E2E_PATH:-$root/coverage/non-e2e-cobertura.xml}"
e2e="${GIVN_COVERAGE_E2E_PATH:-$root/coverage/e2e-cobertura.xml}"
output="${GIVN_COVERAGE_OUTPUT_PATH:-$root/coverage/cobertura-coverage.xml}"
mkdir -p "$(dirname "$output")"

python3 - "$non_e2e" "$e2e" "$output" <<'PY'
import sys
import xml.etree.ElementTree as ET

non_e2e_path, e2e_path, output_path = sys.argv[1:4]

# filename -> line number -> hits
lines: dict[str, dict[int, int]] = {}
sources: list[str] = []

for path in (non_e2e_path, e2e_path):
    tree = ET.parse(path)
    root = tree.getroot()
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
            existing = lines.setdefault(filename, {})
            existing[number] = max(existing.get(number, 0), hits)

if not lines:
    raise SystemExit("merge: no line coverage found in input reports")

covered = sum(1 for hits in lines.values() for value in hits.values() if value > 0)
total = sum(len(hits) for hits in lines.values())

root = ET.Element(
    "coverage",
    {
        "version": "1.9",
        "timestamp": "0",
        "lines-valid": str(total),
        "lines-covered": str(covered),
        "line-rate": f"{covered / total:.6f}",
        "branches-valid": "0",
        "branches-covered": "0",
        "branch-rate": "0",
    },
)
sources_node = ET.SubElement(root, "sources")
for source in sources or ["."]:
    ET.SubElement(sources_node, "source").text = source
packages = ET.SubElement(root, "packages")

by_package: dict[str, list[str]] = {}
for filename in lines:
    package = filename.rsplit("/", 1)[0] if "/" in filename else "."
    by_package.setdefault(package, []).append(filename)

for package_name, files in sorted(by_package.items()):
    package_covered = sum(
        1
        for filename in files
        for hits in lines[filename].values()
        if hits > 0
    )
    package_total = sum(len(lines[filename]) for filename in files)
    package = ET.SubElement(
        packages,
        "package",
        {
            "name": package_name,
            "line-rate": f"{package_covered / package_total:.6f}",
            "branch-rate": "0",
            "complexity": "0",
        },
    )
    classes = ET.SubElement(package, "classes")
    for filename in sorted(files):
        file_lines = lines[filename]
        file_covered = sum(1 for hits in file_lines.values() if hits > 0)
        file_total = len(file_lines)
        cls = ET.SubElement(
            classes,
            "class",
            {
                "name": filename,
                "filename": filename,
                "line-rate": f"{file_covered / file_total:.6f}",
                "branch-rate": "0",
                "complexity": "0",
            },
        )
        ET.SubElement(cls, "methods")
        lines_node = ET.SubElement(cls, "lines")
        for number in sorted(file_lines):
            ET.SubElement(
                lines_node,
                "line",
                {
                    "number": str(number),
                    "hits": str(file_lines[number]),
                    "branch": "false",
                },
            )

ET.ElementTree(root).write(output_path, encoding="utf-8", xml_declaration=True)
print(f"merged coverage: {covered}/{total} lines -> {output_path}")
PY

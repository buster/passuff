#!/usr/bin/env python3
"""Reject no-op step bodies in the durable step trees."""

import ast
import pathlib
import sys

STEP_DECORATORS = {"given", "when", "then", "step", "given_step", "when_step", "then_step"}


def is_noop(body):
    meaningful = []
    for node in body:
        if isinstance(node, ast.Pass):
            continue
        if isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant):
            continue
        if isinstance(node, ast.Return):
            if node.value is None or isinstance(node.value, ast.Constant):
                continue
        meaningful.append(node)
    return not meaningful


def decorated(node):
    for decorator in node.decorator_list:
        target = decorator.func if isinstance(decorator, ast.Call) else decorator
        if isinstance(target, ast.Name) and target.id in STEP_DECORATORS:
            return True
        if isinstance(target, ast.Attribute) and target.attr in STEP_DECORATORS:
            return True
    return False


import argparse

parser = argparse.ArgumentParser()
parser.add_argument("--path", action="append", default=[])
arguments = parser.parse_args()
roots = arguments.path or ["givn/steps", "givn/e2e_steps"]

failures = []
for root in roots:
    root_path = pathlib.Path(root)
    paths = [root_path] if root_path.is_file() else sorted(root_path.glob("*.py"))
    for path in paths:
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError as exc:
            failures.append(f"{path}: syntax error: {exc}")
            continue
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and decorated(node):
                if is_noop(node.body):
                    failures.append(f"{path}:{node.lineno}: no-op step {node.name}")

if failures:
    print("\n".join(failures))
    sys.exit(1)
print("check_steps: no no-op step bodies")

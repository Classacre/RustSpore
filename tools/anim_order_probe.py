"""Independent native flat-child-table reference on synthetic runtime trees.

No installed assets are read. Inputs explicitly supply post-reroot runtime parent
ordinals and block+0x150 masks; they do not assert a CreatureData/runtime mapping.
"""
from __future__ import annotations

import argparse
import hashlib
import heapq
import itertools
import json
import struct
from pathlib import Path


def native_reference(parents: list[int | None], masks: list[int], root: int) -> dict:
    n = len(parents)
    counts = [0] * n
    for parent in parents:
        if parent is not None:
            counts[parent] += 1
    starts = []
    cursor = 1
    for count in counts:
        starts.append(cursor)
        cursor += count
    assert cursor == n
    used = [0] * n
    flat = [0] * n
    for block, parent in enumerate(parents):
        if parent is None:
            continue
        position = starts[parent] + used[parent]
        flat[position] = block
        if used[parent] and masks[block] & 8:
            first = starts[parent]
            if parents[parent] is None and masks[flat[first]] & 8:
                if used[parent] > 1:
                    flat[first + 1], flat[position] = flat[position], flat[first + 1]
            else:
                flat[first], flat[position] = flat[position], flat[first]
        used[parent] += 1
    order = [root]
    child_ranges = [None] * n
    descendant_counts = [0] * n

    def visit(block: int) -> int:
        children = flat[starts[block]:starts[block] + counts[block]]
        child_ranges[block] = (len(order), len(children))
        order.extend(children)
        descendant_counts[block] = len(children)
        for child in children:
            descendant_counts[block] += visit(child)
        return descendant_counts[block]

    visit(root)
    inverse = [0] * n
    assert sorted(order) == list(range(n))
    for new, old in enumerate(order):
        inverse[old] = new
    result = []
    for old in order:
        start, count = child_ranges[old]
        parent = parents[old]
        result.append((old, None if parent is None else inverse[parent], start,
                       count, start, descendant_counts[old]))
        assert set(order[start:start + count]) == {
            i for i, value in enumerate(parents) if value == old
        }
        expected_descendants = set()
        for candidate in range(n):
            at = parents[candidate]
            while at is not None:
                if at == old:
                    expected_descendants.add(candidate)
                    break
                at = parents[at]
        assert set(order[start:start + descendant_counts[old]]) == expected_descendants
        if parent is not None:
            assert inverse[parent] < inverse[old]
    return {"new_to_old": order, "old_to_new": inverse, "blocks": result}


def digest(parents: list[int | None], masks: list[int], root: int, ordered: dict) -> str:
    feed = bytearray(b"runtime-block-order-v1\0")

    def word(value: int | None) -> None:
        feed.extend(struct.pack("<I", 0xFFFFFFFF if value is None else value))

    word(len(parents))
    word(root)
    for parent, mask in zip(parents, masks):
        word(parent)
        word(mask)
    for field in ("new_to_old", "old_to_new"):
        for value in ordered[field]:
            word(value)
    for block in ordered["blocks"]:
        for value in block:
            word(value)
    value = 0xCBF29CE484222325
    for byte in feed:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016X}"


def edges_from_prufer(n: int, code: tuple[int, ...]) -> list[tuple[int, int]]:
    if n == 1:
        return []
    degree = [1] * n
    for value in code:
        degree[value] += 1
    leaves = [i for i, value in enumerate(degree) if value == 1]
    heapq.heapify(leaves)
    edges = []
    for value in code:
        leaf = heapq.heappop(leaves)
        edges.append((leaf, value))
        degree[value] -= 1
        if degree[value] == 1:
            heapq.heappush(leaves, value)
    edges.append(tuple(leaves))
    return edges


def rooted_parents(n: int, edges: list[tuple[int, int]], root: int) -> list[int | None]:
    adjacency = [[] for _ in range(n)]
    for first, second in edges:
        adjacency[first].append(second)
        adjacency[second].append(first)
    parents = [None] * n
    work = [root]
    seen = {root}
    for at in work:
        for child in adjacency[at]:
            if child not in seen:
                parents[child] = at
                seen.add(child)
                work.append(child)
    assert len(seen) == n
    return parents


def fixtures():
    for n in range(1, 6):
        codes = itertools.product(range(n), repeat=max(0, n - 2))
        for code in codes:
            edges = edges_from_prufer(n, code)
            for root in range(n):
                parents = rooted_parents(n, edges, root)
                for spine_bits in range(1 << n):
                    # The extra bits prove that only the native spine bit matters.
                    masks = [(0xFFFFFFF7 if (spine_bits + i) & 1 else 0)
                             | (8 if spine_bits & (1 << i) else 0) for i in range(n)]
                    yield "exhaustive", parents, masks, root
    n = 255
    cases = (
        ("chain", [(i, i + 1) for i in range(n - 1)], 127),
        ("star", [(254, i) for i in range(n - 1)], 254),
        ("ternary", [((i - 1) // 3, i) for i in range(1, n)], 173),
    )
    for name, edges, root in cases:
        parents = rooted_parents(n, edges, root)
        masks = [0xFFFFFFFF if i % 3 else 0xFFFFFFF7 for i in range(n)]
        yield name, parents, masks, root


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--digest-out", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--rust-digest", type=Path)
    args = parser.parse_args()
    for path in (args.fixtures, args.digest_out, args.out):
        path.parent.mkdir(parents=True, exist_ok=True)
    # Hand-computed ordering includes root and nonroot sibling promotion.
    golden_parents = [None] + [0] * 5 + [1] * 5
    golden_masks = [0, 0, 8, 8, 0, 8, 0, 8, 8, 0, 8]
    assert native_reference(golden_parents, golden_masks, 0)["new_to_old"] == [
        0, 2, 5, 1, 4, 3, 10, 6, 7, 9, 8
    ]
    lines = []
    counts = {}
    total_blocks = 0
    max_blocks = 0
    with args.fixtures.open("w", encoding="utf-8", newline="\n") as output:
        output.write("# Synthetic post-reroot runtime inputs: id, root, parents, masks\n")
        for index, (kind, parents, masks, root) in enumerate(fixtures()):
            case_id = f"{index:06d}-{kind}"
            ordered = native_reference(parents, masks, root)
            lines.append(f"{case_id}\t{digest(parents, masks, root, ordered)}")
            p = ",".join("-" if value is None else str(value) for value in parents)
            m = ",".join(f"{value:08X}" for value in masks)
            output.write(f"{case_id}\t{root}\t{p}\t{m}\n")
            key = f"exhaustive_n{len(parents)}" if kind == "exhaustive" else kind
            counts[key] = counts.get(key, 0) + 1
            total_blocks += len(parents)
            max_blocks = max(max_blocks, len(parents))
    normalized = "\n".join(lines) + "\n"
    args.digest_out.write_text(normalized, encoding="utf-8", newline="\n")
    comparison = None
    problems = []
    if args.rust_digest:
        rust = args.rust_digest.read_text(encoding="utf-8").splitlines()
        if rust != lines:
            problems = [{"line": i + 1, "expected": a, "rust": b}
                        for i, (a, b) in enumerate(itertools.zip_longest(lines, rust)) if a != b]
        comparison = {"rust_digest": str(args.rust_digest), "exact_match": not problems,
                      "expected_records": len(lines), "rust_records": len(rust)}
    report = {
        "scope": "synthetic explicit post-reroot runtime topologies; not installed creature rigs",
        "native_procedures": ["9C3150 ordering region 9C4213..9C4380", "9C2FE0"],
        "cases": len(lines), "supplied_blocks": total_blocks, "max_blocks": max_blocks,
        "case_counts": counts, "normalized_digest_sha256": hashlib.sha256(normalized.encode()).hexdigest(),
        "rust_comparison": comparison, "problems": problems[:20], "problem_count": len(problems),
    }
    args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in ("cases", "supplied_blocks", "max_blocks",
                                                "normalized_digest_sha256", "problem_count")}))
    if problems:
        raise SystemExit(1)


if __name__ == "__main__":
    main()

"""Index saved REA dossiers without admitting unrelated fallback procedures.

This reads existing tool results only. It does not start a provider, execute a
target, or treat a successful tool status as evidence of address agreement.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re


def number(value):
    try:
        return int(value, 16) if isinstance(value, str) else None
    except ValueError:
        return None


def envelope(result):
    for content in result.get("content", []):
        if content.get("type") == "text":
            try:
                return json.loads(content["text"])
            except (ValueError, TypeError):
                continue
    return {}


def record(path, query, evidence):
    parameters = evidence.get("parameters") or {}
    requested = parameters.get("procedure")
    raw = evidence.get("raw_result") or {}
    value = raw.get("value") or evidence.get("normalized_result") or {}
    procedure = value.get("procedure") or {}
    actual = procedure.get("address")
    requested_number, actual_number = number(requested), number(actual)
    body = procedure.get("body") or {}
    ranges = body.get("ranges") or []
    contains = requested_number is not None and any(
        number(part.get("start")) is not None
        and number(part.get("end")) is not None
        and number(part["start"]) <= requested_number <= number(part["end"])
        for part in ranges
    )
    error = evidence.get("error") or {}
    if error:
        status = "explicit_error"
        # An error envelope can omit parameters. Retain its stated address as
        # diagnostic context, without promoting that string to a successful read.
        match = re.search(r"requested address: (0x[0-9a-fA-F]+)",
                          json.dumps(error.get("details", {}).get("issues", [])))
        if requested is None and match:
            requested = match[1]
    elif requested_number is not None and requested_number == actual_number:
        status = "exact_entry"
    elif contains:
        status = "containing_body"
    elif requested_number is not None and actual_number is not None:
        status = "mismatched_entry_quarantined"
    else:
        status = "unresolved"
    source = evidence.get("subject") or {}
    return dict(
        query=query, file=path.name, requested=requested, actual=actual,
        address_status=status, exact_entry=status == "exact_entry",
        requested_in_reported_body=bool(contains),
        has_pseudocode=bool(value.get("pseudocode")),
        body_available=bool(body.get("available")),
        reported_body_bytes=body.get("total_bytes"),
        source_sha256=(source.get("digest") or {}).get("sha256"),
        error_code=error.get("code"),
        error_message=error.get("message"),
        tool_result_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("session", type=Path)
    parser.add_argument("--through", type=int, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    rows = []
    for path in args.session.glob("*-analyze_function.json"):
        prefix = path.name.split("-", 1)[0]
        if not prefix.isdigit() or int(prefix) > args.through:
            continue
        result = json.loads(path.read_text(encoding="utf-8"))
        rows.append(record(path, int(prefix), envelope(result)))
    rows.sort(key=lambda row: row["query"])
    counts = Counter(row["address_status"] for row in rows)
    exact = [row for row in rows if row["exact_entry"]]
    report = dict(
        schema="rea-function-dossier-address-index-v1",
        through_query=args.through,
        analysis_operation_records=len(rows),
        address_status_counts=dict(sorted(counts.items())),
        unique_exact_entries=len({(row["source_sha256"], row["actual"]) for row in exact}),
        limitations=[
            "Counts describe saved analyze_function operations, not all target functions.",
            "Exact entry agreement does not establish complete CFG, indirect targets or data closure.",
            "Containing-body results identify a reported body, not the requested procedure entry.",
            "Mismatched entries are quarantined; explicit errors do not supply function evidence.",
            "Batch decompilation is excluded and requires its separate per-item address audit.",
        ],
        records=rows,
    )
    with args.out.open("x", encoding="utf-8", newline="\n") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    print(json.dumps({key: value for key, value in report.items() if key != "records"}))


if __name__ == "__main__":
    main()

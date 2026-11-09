"""Fail-closed placeholder: no native host has been constructed or admitted."""
import sys

if __name__ == '__main__':
    print('BLOCKED: no independently reviewed linked host and runner exist; no native process launched.', file=sys.stderr)
    sys.exit(2)

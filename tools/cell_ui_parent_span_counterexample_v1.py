"""Read-only arithmetic regression for the manager-root flooring boundary."""
import math
import struct


def f32(v):
    return struct.unpack('<f', struct.pack('<f', v))[0]


def main():
    descriptor = struct.unpack('<f', bytes.fromhex('ffff7f3f'))[0]
    authored_origin = 25.0
    committed_manager_span = float(math.floor(descriptor))
    mistaken = math.floor(f32(f32(authored_origin + descriptor) - authored_origin))
    correct = math.floor(f32(f32(authored_origin + committed_manager_span) - authored_origin))
    assert descriptor < 1.0 and mistaken == 1 and committed_manager_span == 0 and correct == 0
    print('3F7FFFFF: raw-descriptor fixture yields 1; laid-out-parent-span fixture yields 0')
    print('This is a boundary regression, not a runtime resolution observation.')

if __name__ == '__main__':
    main()

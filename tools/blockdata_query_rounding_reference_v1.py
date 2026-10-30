"""Independent binary32 reference for the 2x2 ordered interpolation fixture.

No Rust, game, GPU, or native FP control-state is invoked. Inputs and output
are bit-pinned. Explicit packing rounds after each operation. This does not
prove native parity under all FP modes or implement the production query.
"""
import struct

def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]

def from_bits(bits):
    return struct.unpack('<f', struct.pack('<I', bits))[0]

def bits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]

def lerp(a, b, t):
    delta = f32(b - a)
    product = f32(delta * t)
    return f32(a + product)

base = from_bits(0xc408a5a6)
scale = from_bits(0x4259d573)
t0 = from_bits(0x3e015ce3)
t1 = from_bits(0x3f346ee2)
values = [f32(base - f32(n * scale)) for n in (2, 4, 7, 6)]
# Native static trace processes the last dimension inside the first.
ordered = lerp(lerp(values[0], values[1], t1), lerp(values[2], values[3], t1), t0)
reversed_order = lerp(lerp(values[0], values[2], t0), lerp(values[1], values[3], t0), t1)
assert bits(ordered) == 0xc43c07e8
assert bits(reversed_order) == 0xc43c07e9
print(f'ordered={bits(ordered):08x} reversed={bits(reversed_order):08x}')

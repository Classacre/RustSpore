# Cell feeding candidate producer E6A450: ordered decompiler trace

Static evidence for x86 SporeApp.exe SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`: q487 `ev_83ad54e5a1a5eb26edb1d0b01da42140ddf31159660cd405c481090bc0df4c48`; arbitration q493 `ev_ca9d95f9fe0f313cabe5c27eec324c49da02888e50e70e66447be17def314e8e`; relation q494 `ev_b85740dfa6f1dbef601b133803dcbe0792eb6bc8dbed9005357be0eaed92ff58`. See [selector trace](cell-feeding-selector-source-trace-v1.md) for context. This is a transcription of Ghidra control flow, not original names, runtime reachability, or a gameplay specification. It omits no producer branch, but does not resolve x87 state, aliasing, or external call effects.

## Notation and observation boundary

`A=param_5`, `B=param_9`, `a=param_7`, `b=param_13`, `D=param_6`, `d=param_10`, `m=param_11`, `r=param_12`; `param_8` and `param_14` are feature indices. `A.id=*A`, `B.id=*B`, `P=*(global+411c)`, `A.flag=byte A+112`, `B.flag=byte B+112`, `B.kind=B[42]`. `r` is the separately computed E57260 relation, normally 0..4 with its fallback observed; comparisons below are signed. `submit(n)` means E59B40 with original geometry, current IDs and feature indices; `submit*(n)` uses both indices `-1`. `end` returns without E6A2B0. `tail` calls E6A2B0 with geometry, A, B, d, r and returns. A submission is not necessarily a winning update. Re-read memory at the native branch/call sites: external calls and arbitration can affect state or alias it. Character checks are specifically `==0`, `!=0` or `==1` as stated.

The ordinary submission depth and tail argument are not consistently established by pseudocode: Ghidra represents several as casts from `extraout_ST0` or the return of E59B40, including across E571C0. E59B40 itself must be checked in assembly under its real calling convention; do not equate a decompiler `float10` with its integer return or silently replace all depths with `param_4`. The geometry values, current player ID, any observed per-site depth/x87 state, mutable field reads, and external-call outcomes/effects must be supplied at the time each operation occurs. Calls can mutate the shared selection, actors, player state or other state; no purity is proven. Implement an ordered interaction protocol, not a single precomputed predicate bundle. Failed or missing observations must stop the model at the relevant boundary.

## Ordered prefix

1. If `a==9` or `b==9`: end.
2. If `d==7`: submit `19` if `A.id==P`, otherwise `0`; end. If `D==7`: end.
3. If `d==6`: submit `18` if `A.id==P`, otherwise `1`; **continue**.
4. Call E673A0 with current IDs, `param_2`, distance derived from A/B floats at +90,+94,+98 and the value at `*(global+5190)+74`. Then call E59CD0(B,21). If its nonzero result and `a==2 && r==1 && d==1`, submit 21; end.
5. Call E4CB40(19). If `B.kind` equals its result, `a==7`, `b==12`, E5F3D0(B,param_2) returns exactly 1, E527C0 returns nonzero, and E571C0(B)>2: submit 8; end. Calls after the kind check are short-circuited in this order.
6. Call E4CB40(20). If `B.kind` equals result, `r>2`, `a` is 7 or 4, `b==11`, and E6A0C0(B,param_14) returns exactly 1: submit 20; end.
7. Call E4CB40(25). If `B.kind` equals result, `r>2`, `b` is 10, 11 or 12, and `a==4`: submit 13; end.
8. If `d==4`, call 007B2350, resolve A.kind with thunk E823D0, inspect the resolved definition's +B4: if 4 submit* 2; otherwise submit* 3 when `r` is 0, 1 or 2 and (`m!=2` or `A.id==P`), else submit* 1. Call E82160 and continue.
9. If `a==3`, submit 9 only if `d==3 && r==1 && A.flag==0`; end in either case. If `b==3`: end.

## Ordered remainder

All paths below end at `tail` unless marked `end`. Multiple submissions listed in a row occur in order; predicates must be evaluated where they appear, not simultaneously.

- `a==2`, `d==2`:
  - `r==0`: if both flags zero submit 10; tail.
  - `r==1`: if B is player and both flags zero submit 7. Then if B is not player and A.flag is zero submit 10; tail.
  - `r==2`: if both flags zero and `b==2`, submit 15. If A.flag is nonzero or `b==2`, tail. Otherwise call E571C0(B); if result <2 call E527C0; if that result is zero submit 1 and tail. Otherwise submit 7; tail.
  - `r==3`: if A.flag is zero, call E503A0(A); submit 4 if its result is zero, otherwise 7. Tail. Other r: tail.
- `a==2`, `d!=2`: if `(m & 1)!=0 && r==1`, then if B is not player and A.flag is zero submit 10. Tail.
- `a==7`: first, if `(m & 1)!=0 && r<=1`, submit 8. Continue with:
  - `d==2`, `r==2`: if A.flag zero, submit 14 when `b==7`, otherwise 8. Tail.
  - `d==2`, `r==1`: if both flags zero, call E571C0(B), and if result >1 submit 8. Tail.
  - `d==2`, `r==3`: if `b==2` and A.flag zero, submit 5; if A.flag is nonzero, tail. If B.flag zero submit 8. Then if A.flag zero and B.flag exactly 1 submit 5. Tail. Other r: tail.
  - `d==3`, `r==2`: if A.flag zero call E571C0(B); if result >=2 submit 6. Tail. `d==3`, `r==4`: if A.flag zero submit 5. Other r: tail.
  - `d==4`: submit 3 and tail. Other d: tail.
- `a==4`, `d==2`: if r is 1 or 2, call E573E0(B); if nonzero and both flags zero submit 16 when `b==4`, else 12. If `r==3` and A.flag zero submit 4. Tail.
- `a==4`, `d==3`: if A.flag zero, submit 11 for `r==1` or 4 for `r==3`. Tail.
- Any other a/d combination: tail.

Note especially that `a==3` and `b==3` exit without the tail, whereas most other cases invoke it even without submissions. E6A2B0 is a separate producer with potentially relevant effects; it is not represented by the table. Each E59B40 submission can change shared selection before later calls. External results listed here are ordered per invocation, including repeated calls to the same function; they are not justified as stable properties.

## Implementation status

No candidate-producer Rust model is claimed from this evidence. A complete safe boundary needs instruction-level confirmation of x87 depth at each call, a defined protocol for external calls and mutations, and analysis of E6A2B0. The existing E59B40 model accepts supplied candidates and remains a narrower arbitration model. The trace supports planning those observations but is not evidence that any candidate is reachable in a live game.


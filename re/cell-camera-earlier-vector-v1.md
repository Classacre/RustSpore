# Conditional earlier camera vectors

The reviewed original `E60750..E60818` slice and original E51230/1041E40
helpers matched four controlled rows with supplied state, optional synthetic
actor and alpha. The source copies state +48/+4C/+50 to B and +6C/+70 plus
`+74 * +908` to C; when the supplied actor is non-null it uses the original
ramp to interpolate B.x/B.y toward the copied state from actor +4C/+50.
The fixture stops before the following mode/actor/provider dispatch. Native
exit was zero and its non-switched desktop closed after reaping. Receipt
`target/cell-camera-earlier-vector-result-v1/receipt.json` SHA-256
`fb6d08d0055716fe649d3e667fb14fba6d350f3bbe79407b60cd81d6d869f0dc`.
The original host retained ramp results as measurements rather than comparing
them to an independent exact helper oracle. It did not establish a live actor,
current state, successful E51760 provider or gameplay camera.

`cell_camera_earlier_vector_v1` separately projects the copied vectors and
the optional actor branch using the existing checked retained-ramp model, with
explicit stable/disjoint state, CW 027F and MXCSR 1F80 admissions. It leaves
later dispatch pending. Its comparator checks all four retained vectors, ramp
or alpha, and FP/control words. This narrows one source producer boundary but
does not establish its scheduling or connect the synthetic actor to a live one.

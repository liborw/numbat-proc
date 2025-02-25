# Schmitt Trigger

let r1 = 10 kiloohm
let r2 = 47 kiloohm
let r3 = 10 kiloohm
let v_ref = 3.3 V

let r23 = (r2 * r3) / (r2 + r3)

let v_lb = (r23 / (r1 + r23)) * v_ref
v_lb #= 1.49135 V

let v_ub = (r2 / (r2 + r23)) * v_ref
v_ub #= 2.80746 V


# LED forward current range

let v_in = 3.3 V
let v_fwd = 1.2 V
let a_fwd_max = 50 mA

let R8 = (v_in - v_fwd) / a_fwd_max
R8 -> ohm #= 42 Ω

let R8 = 50 ohm
let a_fwd = (v_in - v_fwd) / R8
a_fwd -> mA #= 42 mA

let R8 = 2.1 kiloohm
let a_fwd = (v_in - v_fwd) / R8
a_fwd -> mA #= 1 mA

;; Tests for i32x4 trunc sat conversions from float.

(module
  (func (export "i32x4.trunc_sat_f32x4_s") (param v128) (result v128) (i32x4.trunc_sat_f32x4_s (local.get 0)))
  (func (export "i32x4.trunc_sat_f32x4_u") (param v128) (result v128) (i32x4.trunc_sat_f32x4_u (local.get 0)))
)


;; i32x4.trunc_sat_f32x4_s
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0.0 0.0 0.0 0.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0.0 -0.0 -0.0 -0.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 1.5 1.5 1.5 1.5))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -1.5 -1.5 -1.5 -1.5))
                                                 (v128.const i32x4 -1 -1 -1 -1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 1.9 1.9 1.9 1.9))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 2.0 2.0 2.0 2.0))
                                                 (v128.const i32x4 2 2 2 2))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -1.9 -1.9 -1.9 -1.9))
                                                 (v128.const i32x4 -1 -1 -1 -1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -2.0 -2.0 -2.0 -2.0))
                                                 (v128.const i32x4 -2 -2 -2 -2))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 2147483520.0 2147483520.0 2147483520.0 2147483520.0))
                                                 (v128.const i32x4 2147483520 2147483520 2147483520 2147483520))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -2147483520.0 -2147483520.0 -2147483520.0 -2147483520.0))
                                                 (v128.const i32x4 -2147483520 -2147483520 -2147483520 -2147483520))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 2147483648.0 2147483648.0 2147483648.0 2147483648.0))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -2147483648.0 -2147483648.0 -2147483648.0 -2147483648.0))
                                                 (v128.const i32x4 -2147483648 -2147483648 -2147483648 -2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 4294967294.0 4294967294.0 4294967294.0 4294967294.0))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -4294967294.0 -4294967294.0 -4294967294.0 -4294967294.0))
                                                 (v128.const i32x4 -2147483648 -2147483648 -2147483648 -2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 2147483647.0 2147483647.0 2147483647.0 2147483647.0))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -2147483647.0 -2147483647.0 -2147483647.0 -2147483647.0))
                                                 (v128.const i32x4 -2147483648 -2147483648 -2147483648 -2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 4294967294.0 4294967294.0 4294967294.0 4294967294.0))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 4294967295.0 4294967295.0 4294967295.0 4294967295.0))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 4294967296.0 4294967296.0 4294967296.0 4294967296.0))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1p-149 0x1p-149 0x1p-149 0x1p-149))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1p-149 -0x1p-149 -0x1p-149 -0x1p-149))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1p-126 0x1p-126 0x1p-126 0x1p-126))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1p-126 -0x1p-126 -0x1p-126 -0x1p-126))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1p-1 0x1p-1 0x1p-1 0x1p-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1p-1 -0x1p-1 -0x1p-1 -0x1p-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1p+0 0x1p+0 0x1p+0 0x1p+0))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1p+0 -0x1p+0 -0x1p+0 -0x1p+0))
                                                 (v128.const i32x4 -1 -1 -1 -1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1.19999ap+0 0x1.19999ap+0 0x1.19999ap+0 0x1.19999ap+0))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1.19999ap+0 -0x1.19999ap+0 -0x1.19999ap+0 -0x1.19999ap+0))
                                                 (v128.const i32x4 -1 -1 -1 -1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2))
                                                 (v128.const i32x4 6 6 6 6))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2))
                                                 (v128.const i32x4 -6 -6 -6 -6))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127))
                                                 (v128.const i32x4 -2147483648 -2147483648 -2147483648 -2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1.ccccccp-1 0x1.ccccccp-1 0x1.ccccccp-1 0x1.ccccccp-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1.ccccccp-1 -0x1.ccccccp-1 -0x1.ccccccp-1 -0x1.ccccccp-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1.fffffep-1 0x1.fffffep-1 0x1.fffffep-1 0x1.fffffep-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1.fffffep-1 -0x1.fffffep-1 -0x1.fffffep-1 -0x1.fffffep-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2))
                                                 (v128.const i32x4 6 6 6 6))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2))
                                                 (v128.const i32x4 -6 -6 -6 -6))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127))
                                                 (v128.const i32x4 -2147483648 -2147483648 -2147483648 -2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 +inf +inf +inf +inf))
                                                 (v128.const i32x4 2147483647 2147483647 2147483647 2147483647))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -inf -inf -inf -inf))
                                                 (v128.const i32x4 -2147483648 -2147483648 -2147483648 -2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 +nan +nan +nan +nan))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -nan -nan -nan -nan))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 nan:0x444444 nan:0x444444 nan:0x444444 nan:0x444444))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -nan:0x444444 -nan:0x444444 -nan:0x444444 -nan:0x444444))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 42 42 42 42))
                                                 (v128.const i32x4 42 42 42 42))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 -42 -42 -42 -42))
                                                 (v128.const i32x4 -42 -42 -42 -42))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 0123456792.0 0123456792.0 0123456792.0 0123456792.0))
                                                 (v128.const i32x4 123456792 123456792 123456792 123456792))
(assert_return (invoke "i32x4.trunc_sat_f32x4_s" (v128.const f32x4 01234567890.0 01234567890.0 01234567890.0 01234567890.0))
                                                 (v128.const i32x4 1234567936 1234567936 1234567936 1234567936))

;; i32x4.trunc_sat_f32x4_u
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0.0 0.0 0.0 0.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0.0 -0.0 -0.0 -0.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 1.5 1.5 1.5 1.5))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -1.5 -1.5 -1.5 -1.5))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 1.9 1.9 1.9 1.9))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 2.0 2.0 2.0 2.0))
                                                 (v128.const i32x4 2 2 2 2))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -1.9 -1.9 -1.9 -1.9))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -2.0 -2.0 -2.0 -2.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 2147483520.0 2147483520.0 2147483520.0 2147483520.0))
                                                 (v128.const i32x4 2147483520 2147483520 2147483520 2147483520))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -2147483520.0 -2147483520.0 -2147483520.0 -2147483520.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 2147483648.0 2147483648.0 2147483648.0 2147483648.0))
                                                 (v128.const i32x4 2147483648 2147483648 2147483648 2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -2147483648.0 -2147483648.0 -2147483648.0 -2147483648.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 4294967294.0 4294967294.0 4294967294.0 4294967294.0))
                                                 (v128.const i32x4 4294967295 4294967295 4294967295 4294967295))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -4294967294.0 -4294967294.0 -4294967294.0 -4294967294.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 2147483647.0 2147483647.0 2147483647.0 2147483647.0))
                                                 (v128.const i32x4 2147483648 2147483648 2147483648 2147483648))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -2147483647.0 -2147483647.0 -2147483647.0 -2147483647.0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 4294967294.0 4294967294.0 4294967294.0 4294967294.0))
                                                 (v128.const i32x4 4294967295 4294967295 4294967295 4294967295))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 4294967295.0 4294967295.0 4294967295.0 4294967295.0))
                                                 (v128.const i32x4 4294967295 4294967295 4294967295 4294967295))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 4294967296.0 4294967296.0 4294967296.0 4294967296.0))
                                                 (v128.const i32x4 4294967295 4294967295 4294967295 4294967295))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1p-149 0x1p-149 0x1p-149 0x1p-149))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1p-149 -0x1p-149 -0x1p-149 -0x1p-149))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1p-126 0x1p-126 0x1p-126 0x1p-126))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1p-126 -0x1p-126 -0x1p-126 -0x1p-126))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1p-1 0x1p-1 0x1p-1 0x1p-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1p-1 -0x1p-1 -0x1p-1 -0x1p-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1p+0 0x1p+0 0x1p+0 0x1p+0))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1p+0 -0x1p+0 -0x1p+0 -0x1p+0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1.19999ap+0 0x1.19999ap+0 0x1.19999ap+0 0x1.19999ap+0))
                                                 (v128.const i32x4 1 1 1 1))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1.19999ap+0 -0x1.19999ap+0 -0x1.19999ap+0 -0x1.19999ap+0))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2))
                                                 (v128.const i32x4 6 6 6 6))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127))
                                                 (v128.const i32x4 4294967295 4294967295 4294967295 4294967295))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1.ccccccp-1 0x1.ccccccp-1 0x1.ccccccp-1 0x1.ccccccp-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1.ccccccp-1 -0x1.ccccccp-1 -0x1.ccccccp-1 -0x1.ccccccp-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1.fffffep-1 0x1.fffffep-1 0x1.fffffep-1 0x1.fffffep-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1.fffffep-1 -0x1.fffffep-1 -0x1.fffffep-1 -0x1.fffffep-1))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2 0x1.921fb6p+2))
                                                 (v128.const i32x4 6 6 6 6))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2 -0x1.921fb6p+2))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127 0x1.fffffep+127))
                                                 (v128.const i32x4 4294967295 4294967295 4294967295 4294967295))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127 -0x1.fffffep+127))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 +inf +inf +inf +inf))
                                                 (v128.const i32x4 4294967295 4294967295 4294967295 4294967295))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -inf -inf -inf -inf))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 +nan +nan +nan +nan))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -nan -nan -nan -nan))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 nan:0x444444 nan:0x444444 nan:0x444444 nan:0x444444))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -nan:0x444444 -nan:0x444444 -nan:0x444444 -nan:0x444444))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 42 42 42 42))
                                                 (v128.const i32x4 42 42 42 42))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 -42 -42 -42 -42))
                                                 (v128.const i32x4 0 0 0 0))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 0123456792.0 0123456792.0 0123456792.0 0123456792.0))
                                                 (v128.const i32x4 123456792 123456792 123456792 123456792))
(assert_return (invoke "i32x4.trunc_sat_f32x4_u" (v128.const f32x4 01234567890.0 01234567890.0 01234567890.0 01234567890.0))
                                                 (v128.const i32x4 1234567936 1234567936 1234567936 1234567936))

;; type check
(assert_invalid (module (func (result v128) (i32x4.trunc_sat_f32x4_s (i32.const 0)))) "type mismatch")
(assert_invalid (module (func (result v128) (i32x4.trunc_sat_f32x4_u (i32.const 0)))) "type mismatch")

;; Test operation with empty argument

(assert_invalid
  (module
    (func $i32x4.trunc_sat_f32x4_s-arg-empty (result v128)
      (i32x4.trunc_sat_f32x4_s)
    )
  )
  "type mismatch"
)
(assert_invalid
  (module
    (func $i32x4.trunc_sat_f32x4_u-arg-empty (result v128)
      (i32x4.trunc_sat_f32x4_u)
    )
  )
  "type mismatch"
)


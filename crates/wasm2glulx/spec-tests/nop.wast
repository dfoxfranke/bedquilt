;; Test `nop` operator.

(module
  ;; Auxiliary definitions
  (func $dummy)
  (func $3-ary (param i32 i32 i32) (result i32)
    local.get 0 local.get 1 local.get 2 i32.sub i32.add
  )
  (memory 1)

  (func (export "as-func-first") (result i32)
    (nop) (i32.const 1)
  )
  (func (export "as-func-mid") (result i32)
    (call $dummy) (nop) (i32.const 2)
  )
  (func (export "as-func-last") (result i32)
    (call $dummy) (i32.const 3) (nop)
  )
  (func (export "as-func-everywhere") (result i32)
    (nop) (nop) (call $dummy) (nop) (i32.const 4) (nop) (nop)
  )

  (func (export "as-drop-first") (param i32)
    (nop) (local.get 0) (drop)
  )
  (func (export "as-drop-last") (param i32)
    (local.get 0) (nop) (drop)
  )
  (func (export "as-drop-everywhere") (param i32)
    (nop) (nop) (local.get 0) (nop) (nop) (drop)
  )

  (func (export "as-select-first") (param i32) (result i32)
    (nop) (local.get 0) (local.get 0) (local.get 0) (select)
  )
  (func (export "as-select-mid1") (param i32) (result i32)
    (local.get 0) (nop) (local.get 0) (local.get 0) (select)
  )
  (func (export "as-select-mid2") (param i32) (result i32)
    (local.get 0) (local.get 0) (nop) (local.get 0) (select)
  )
  (func (export "as-select-last") (param i32) (result i32)
    (local.get 0) (local.get 0) (local.get 0) (nop) (select)
  )
  (func (export "as-select-everywhere") (param i32) (result i32)
    (nop) (local.get 0) (nop) (nop) (local.get 0)
    (nop) (nop) (local.get 0) (nop) (nop) (select)
  )

  (func (export "as-block-first") (result i32)
    (block (result i32) (nop) (i32.const 2))
  )
  (func (export "as-block-mid") (result i32)
    (block (result i32) (call $dummy) (nop) (i32.const 2))
  )
  (func (export "as-block-last") (result i32)
    (block (result i32) (nop) (call $dummy) (i32.const 3) (nop))
  )
  (func (export "as-block-everywhere") (result i32)
    (block (result i32)
      (nop) (nop) (call $dummy) (nop) (i32.const 4) (nop) (nop)
    )
  )

  (func (export "as-loop-first") (result i32)
    (loop (result i32) (nop) (i32.const 2))
  )
  (func (export "as-loop-mid") (result i32)
    (loop (result i32) (call $dummy) (nop) (i32.const 2))
  )
  (func (export "as-loop-last") (result i32)
    (loop (result i32) (call $dummy) (i32.const 3) (nop))
  )
  (func (export "as-loop-everywhere") (result i32)
    (loop (result i32)
      (nop) (nop) (call $dummy) (nop) (i32.const 4) (nop) (nop)
    )
  )

  (func (export "as-if-condition") (param i32)
    (local.get 0) (nop) (if (then (call $dummy)))
  )
  (func (export "as-if-then") (param i32)
    (if (local.get 0) (then (nop)) (else (call $dummy)))
  )
  (func (export "as-if-else") (param i32)
    (if (local.get 0) (then (call $dummy)) (else (nop)))
  )

  (func (export "as-br-first") (param i32) (result i32)
    (block (result i32) (nop) (local.get 0) (br 0))
  )
  (func (export "as-br-last") (param i32) (result i32)
    (block (result i32) (local.get 0) (nop) (br 0))
  )
  (func (export "as-br-everywhere") (param i32) (result i32)
    (block (result i32) (nop) (nop) (local.get 0) (nop) (nop) (br 0))
  )

  (func (export "as-br_if-first") (param i32) (result i32)
    (block (result i32) (nop) (local.get 0) (local.get 0) (br_if 0))
  )
  (func (export "as-br_if-mid") (param i32) (result i32)
    (block (result i32) (local.get 0) (nop) (local.get 0) (br_if 0))
  )
  (func (export "as-br_if-last") (param i32) (result i32)
    (block (result i32) (local.get 0) (local.get 0) (nop) (br_if 0))
  )
  (func (export "as-br_if-everywhere") (param i32) (result i32)
    (block (result i32)
      (nop) (nop) (local.get 0) (nop) (nop) (local.get 0) (nop) (nop)
      (br_if 0)
    )
  )

  (func (export "as-br_table-first") (param i32) (result i32)
    (block (result i32) (nop) (local.get 0) (local.get 0) (br_table 0 0))
  )
  (func (export "as-br_table-mid") (param i32) (result i32)
    (block (result i32) (local.get 0) (nop) (local.get 0) (br_table 0 0))
  )
  (func (export "as-br_table-last") (param i32) (result i32)
    (block (result i32) (local.get 0) (local.get 0) (nop) (br_table 0 0))
  )
  (func (export "as-br_table-everywhere") (param i32) (result i32)
    (block (result i32)
      (nop) (nop) (local.get 0) (nop) (nop) (local.get 0) (nop) (nop)
      (br_table 0 0)
    )
  )

  (func (export "as-return-first") (param i32) (result i32)
    (nop) (local.get 0) (return)
  )
  (func (export "as-return-last") (param i32) (result i32)
    (local.get 0) (nop) (return)
  )
  (func (export "as-return-everywhere") (param i32) (result i32)
    (nop) (nop) (local.get 0) (nop) (nop) (return)
  )

  (func (export "as-call-first") (param i32 i32 i32) (result i32)
    (nop) (local.get 0) (local.get 1) (local.get 2) (call $3-ary)
  )
  (func (export "as-call-mid1") (param i32 i32 i32) (result i32)
    (local.get 0) (nop) (local.get 1) (local.get 2) (call $3-ary)
  )
  (func (export "as-call-mid2") (param i32 i32 i32) (result i32)
    (local.get 0) (local.get 1) (nop) (local.get 2) (call $3-ary)
  )
  (func (export "as-call-last") (param i32 i32 i32) (result i32)
    (local.get 0) (local.get 1) (local.get 2) (nop) (call $3-ary)
  )
  (func (export "as-call-everywhere") (param i32 i32 i32) (result i32)
    (nop) (nop) (local.get 0) (nop) (nop) (local.get 1)
    (nop) (nop) (local.get 2) (nop) (nop) (call $3-ary)
  )

  (func (export "as-unary-first") (param i32) (result i32)
    (nop) (local.get 0) (i32.ctz)
  )
  (func (export "as-unary-last") (param i32) (result i32)
    (local.get 0) (nop) (i32.ctz)
  )
  (func (export "as-unary-everywhere") (param i32) (result i32)
    (nop) (nop) (local.get 0) (nop) (nop) (i32.ctz)
  )

  (func (export "as-binary-first") (param i32) (result i32)
    (nop) (local.get 0) (local.get 0) (i32.add)
  )
  (func (export "as-binary-mid") (param i32) (result i32)
    (local.get 0) (nop) (local.get 0) (i32.add)
  )
  (func (export "as-binary-last") (param i32) (result i32)
    (local.get 0) (local.get 0) (nop) (i32.add)
  )
  (func (export "as-binary-everywhere") (param i32) (result i32)
    (nop) (local.get 0) (nop) (nop) (local.get 0) (nop) (nop) (i32.add)
  )

  (func (export "as-test-first") (param i32) (result i32)
    (nop) (local.get 0) (i32.eqz)
  )
  (func (export "as-test-last") (param i32) (result i32)
    (local.get 0) (nop) (i32.eqz)
  )
  (func (export "as-test-everywhere") (param i32) (result i32)
    (nop) (nop) (local.get 0) (nop) (nop) i32.eqz
  )

  (func (export "as-compare-first") (param i32) (result i32)
    (nop) (local.get 0) (local.get 0) (i32.ne)
  )
  (func (export "as-compare-mid") (param i32) (result i32)
    (local.get 0) (nop) (local.get 0) (i32.ne)
  )
  (func (export "as-compare-last") (param i32) (result i32)
    (local.get 0) (local.get 0) (nop) (i32.lt_u)
  )
  (func (export "as-compare-everywhere") (param i32) (result i32)
    (nop) (local.get 0) (nop) (nop) (local.get 0) (nop) (nop) (i32.le_s)
  )

  (func (export "as-memory.grow-first") (param i32) (result i32)
    (nop) (local.get 0) (memory.grow)
  )
  (func (export "as-memory.grow-last") (param i32) (result i32)
    (local.get 0) (nop) (memory.grow)
  )
  (func (export "as-memory.grow-everywhere") (param i32) (result i32)
    (nop) (nop) (local.get 0) (nop) (nop) (memory.grow)
  )

  (func $func (param i32 i32) (result i32) (local.get 0))
  (type $check (func (param i32 i32) (result i32)))
  (table funcref (elem $func))
  (func (export "as-call_indirect-first") (result i32)
    (block (result i32)
      (nop) (i32.const 1) (i32.const 2) (i32.const 0)
      (call_indirect (type $check))
    )
  )
  (func (export "as-call_indirect-mid1") (result i32)
    (block (result i32)
      (i32.const 1) (nop) (i32.const 2) (i32.const 0)
      (call_indirect (type $check))
    )
  )
  (func (export "as-call_indirect-mid2") (result i32)
    (block (result i32)
      (i32.const 1) (i32.const 2) (nop) (i32.const 0)
      (call_indirect (type $check))
    )
  )
  (func (export "as-call_indirect-last") (result i32)
    (block (result i32)
      (i32.const 1) (i32.const 2) (i32.const 0) (nop)
      (call_indirect (type $check))
    )
  )
  (func (export "as-call_indirect-everywhere") (result i32)
    (block (result i32)
      (nop) (nop) (i32.const 1) (nop) (nop) (i32.const 2) (nop) (nop) (i32.const 0) (nop) (nop)
      (call_indirect (type $check))
    )
  )

  (func (export "as-local.set-first") (param i32) (result i32)
    (nop) (i32.const 2) (local.set 0) (local.get 0)
  )
  (func (export "as-local.set-last") (param i32) (result i32)
    (i32.const 2) (nop) (local.set 0) (local.get 0)
  )
  (func (export "as-local.set-everywhere") (param i32) (result i32)
    (nop) (nop) (i32.const 2) (nop) (nop) (local.set 0) (local.get 0)
  )

  (func (export "as-local.tee-first") (param i32) (result i32)
    (nop) (i32.const 2) (local.tee 0)
  )
  (func (export "as-local.tee-last") (param i32) (result i32)
    (i32.const 2) (nop) (local.tee 0)
  )
  (func (export "as-local.tee-everywhere") (param i32) (result i32)
    (nop) (nop) (i32.const 2) (nop) (nop) (local.tee 0)
  )

  (global $a (mut i32) (i32.const 0))
  (func (export "as-global.set-first") (result i32)
    (nop) (i32.const 2) (global.set $a) (global.get $a)
  )
  (func (export "as-global.set-last") (result i32)
    (i32.const 2) (nop) (global.set $a) (global.get $a)
  )
  (func (export "as-global.set-everywhere") (result i32)
    (nop) (nop) (i32.const 2) (nop) (nop) (global.set 0)
    (global.get $a)
  )

  (func (export "as-load-first") (param i32) (result i32)
    (nop) (local.get 0) (i32.load)
  )
  (func (export "as-load-last") (param i32) (result i32)
    (local.get 0) (nop) (i32.load)
  )
  (func (export "as-load-everywhere") (param i32) (result i32)
    (nop) (nop) (local.get 0) (nop) (nop) (i32.load)
  )

  (func (export "as-store-first") (param i32 i32)
    (nop) (local.get 0) (local.get 1) (i32.store)
  )
  (func (export "as-store-mid") (param i32 i32)
    (local.get 0) (nop) (local.get 1) (i32.store)
  )
  (func (export "as-store-last") (param i32 i32)
    (local.get 0) (local.get 1) (nop) (i32.store)
  )
  (func (export "as-store-everywhere") (param i32 i32)
    (nop) (nop) (local.get 0) (nop) (nop) (local.get 1) (nop) (nop) (i32.store)
  )
)

(assert_return (invoke "as-func-first") (i32.const 1))
(assert_return (invoke "as-func-mid") (i32.const 2))
(assert_return (invoke "as-func-last") (i32.const 3))
(assert_return (invoke "as-func-everywhere") (i32.const 4))

(assert_return (invoke "as-drop-first" (i32.const 0)))
(assert_return (invoke "as-drop-last" (i32.const 0)))
(assert_return (invoke "as-drop-everywhere" (i32.const 0)))

(assert_return (invoke "as-select-first" (i32.const 3)) (i32.const 3))
(assert_return (invoke "as-select-mid1" (i32.const 3)) (i32.const 3))
(assert_return (invoke "as-select-mid2" (i32.const 3)) (i32.const 3))
(assert_return (invoke "as-select-last" (i32.const 3)) (i32.const 3))
(assert_return (invoke "as-select-everywhere" (i32.const 3)) (i32.const 3))

(assert_return (invoke "as-block-first") (i32.const 2))
(assert_return (invoke "as-block-mid") (i32.const 2))
(assert_return (invoke "as-block-last") (i32.const 3))
(assert_return (invoke "as-block-everywhere") (i32.const 4))

(assert_return (invoke "as-loop-first") (i32.const 2))
(assert_return (invoke "as-loop-mid") (i32.const 2))
(assert_return (invoke "as-loop-last") (i32.const 3))
(assert_return (invoke "as-loop-everywhere") (i32.const 4))

(assert_return (invoke "as-if-condition" (i32.const 0)))
(assert_return (invoke "as-if-condition" (i32.const -1)))
(assert_return (invoke "as-if-then" (i32.const 0)))
(assert_return (invoke "as-if-then" (i32.const 4)))
(assert_return (invoke "as-if-else" (i32.const 0)))
(assert_return (invoke "as-if-else" (i32.const 3)))

(assert_return (invoke "as-br-first" (i32.const 5)) (i32.const 5))
(assert_return (invoke "as-br-last" (i32.const 6)) (i32.const 6))
(assert_return (invoke "as-br-everywhere" (i32.const 7)) (i32.const 7))

(assert_return (invoke "as-br_if-first" (i32.const 4)) (i32.const 4))
(assert_return (invoke "as-br_if-mid" (i32.const 5)) (i32.const 5))
(assert_return (invoke "as-br_if-last" (i32.const 6)) (i32.const 6))
(assert_return (invoke "as-br_if-everywhere" (i32.const 7)) (i32.const 7))

(assert_return (invoke "as-br_table-first" (i32.const 4)) (i32.const 4))
(assert_return (invoke "as-br_table-mid" (i32.const 5)) (i32.const 5))
(assert_return (invoke "as-br_table-last" (i32.const 6)) (i32.const 6))
(assert_return (invoke "as-br_table-everywhere" (i32.const 7)) (i32.const 7))

(assert_return (invoke "as-return-first" (i32.const 5)) (i32.const 5))
(assert_return (invoke "as-return-last" (i32.const 6)) (i32.const 6))
(assert_return (invoke "as-return-everywhere" (i32.const 7)) (i32.const 7))

(assert_return (invoke "as-call-first" (i32.const 3) (i32.const 1) (i32.const 2)) (i32.const 2))
(assert_return (invoke "as-call-mid1" (i32.const 3) (i32.const 1) (i32.const 2)) (i32.const 2))
(assert_return (invoke "as-call-mid2" (i32.const 0) (i32.const 3) (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-call-last" (i32.const 10) (i32.const 9) (i32.const -1)) (i32.const 20))
(assert_return (invoke "as-call-everywhere" (i32.const 2) (i32.const 1) (i32.const 5)) (i32.const -2))

(assert_return (invoke "as-unary-first" (i32.const 30)) (i32.const 1))
(assert_return (invoke "as-unary-last" (i32.const 30)) (i32.const 1))
(assert_return (invoke "as-unary-everywhere" (i32.const 12)) (i32.const 2))

(assert_return (invoke "as-binary-first" (i32.const 3)) (i32.const 6))
(assert_return (invoke "as-binary-mid" (i32.const 3)) (i32.const 6))
(assert_return (invoke "as-binary-last" (i32.const 3)) (i32.const 6))
(assert_return (invoke "as-binary-everywhere" (i32.const 3)) (i32.const 6))

(assert_return (invoke "as-test-first" (i32.const 0)) (i32.const 1))
(assert_return (invoke "as-test-last" (i32.const 0)) (i32.const 1))
(assert_return (invoke "as-test-everywhere" (i32.const 0)) (i32.const 1))

(assert_return (invoke "as-compare-first" (i32.const 3)) (i32.const 0))
(assert_return (invoke "as-compare-mid" (i32.const 3)) (i32.const 0))
(assert_return (invoke "as-compare-last" (i32.const 3)) (i32.const 0))
(assert_return (invoke "as-compare-everywhere" (i32.const 3)) (i32.const 1))

(assert_return (invoke "as-memory.grow-first" (i32.const 0)) (i32.const 1))
(invoke "as-memory.grow-first" (i32.const 0))
(assert_return (invoke "as-memory.grow-last" (i32.const 2)) (i32.const 1))
(invoke "as-memory.grow-last" (i32.const 2))
(assert_return (invoke "as-memory.grow-everywhere" (i32.const 12)) (i32.const 3))

(assert_return (invoke "as-call_indirect-first") (i32.const 1))
(assert_return (invoke "as-call_indirect-mid1") (i32.const 1))
(assert_return (invoke "as-call_indirect-mid2") (i32.const 1))
(assert_return (invoke "as-call_indirect-last") (i32.const 1))
(assert_return (invoke "as-call_indirect-everywhere") (i32.const 1))

(assert_return (invoke "as-local.set-first" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-local.set-last" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-local.set-everywhere" (i32.const 1)) (i32.const 2))

(assert_return (invoke "as-local.tee-first" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-local.tee-last" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-local.tee-everywhere" (i32.const 1)) (i32.const 2))

(assert_return (invoke "as-global.set-first") (i32.const 2))
(assert_return (invoke "as-global.set-last") (i32.const 2))
(assert_return (invoke "as-global.set-everywhere") (i32.const 2))

(assert_return (invoke "as-load-first" (i32.const 100)) (i32.const 0))
(assert_return (invoke "as-load-last" (i32.const 100)) (i32.const 0))
(assert_return (invoke "as-load-everywhere" (i32.const 100)) (i32.const 0))

(assert_return (invoke "as-store-first" (i32.const 0) (i32.const 1)))
(assert_return (invoke "as-store-mid" (i32.const 0) (i32.const 2)))
(assert_return (invoke "as-store-last" (i32.const 0) (i32.const 3)))
(assert_return (invoke "as-store-everywhere" (i32.const 0) (i32.const 4)))

(assert_invalid
  (module (func $type-i32 (result i32) (nop)))
  "type mismatch"
)
(assert_invalid
  (module (func $type-i64 (result i64) (nop)))
  "type mismatch"
)
(assert_invalid
  (module (func $type-f32 (result f32) (nop)))
  "type mismatch"
)
(assert_invalid
  (module (func $type-f64 (result f64) (nop)))
  "type mismatch"
)

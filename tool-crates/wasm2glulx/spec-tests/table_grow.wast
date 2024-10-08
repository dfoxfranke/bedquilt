;; Reject growing to size outside i32 value range
(module
  (table $t 0x10 funcref)
  (elem declare func $f)
  (func $f (export "grow") (result i32)
    (table.grow $t (ref.func $f) (i32.const 0xffff_fff0))
  )
)

(assert_return (invoke "grow") (i32.const -1))


(module
  (table $t 0 externref)
  (func (export "grow") (param i32) (result i32)
    (table.grow $t (ref.null extern) (local.get 0))
  )
)

(assert_return (invoke "grow" (i32.const 0)) (i32.const 0))
(invoke "grow" (i32.const 0))
(assert_return (invoke "grow" (i32.const 1)) (i32.const 0))
(invoke "grow" (i32.const 1))
(assert_return (invoke "grow" (i32.const 0)) (i32.const 1))
(invoke "grow" (i32.const 0))
(assert_return (invoke "grow" (i32.const 2)) (i32.const 1))
(invoke "grow" (i32.const 2))
(assert_return (invoke "grow" (i32.const 800)) (i32.const 3))


(module
  (table $t 0 10 externref)
  (func (export "grow") (param i32) (result i32)
    (table.grow $t (ref.null extern) (local.get 0))
  )
)

(assert_return (invoke "grow" (i32.const 0)) (i32.const 0))
(invoke "grow" (i32.const 0))
(assert_return (invoke "grow" (i32.const 1)) (i32.const 0))
(invoke "grow" (i32.const 1))
(assert_return (invoke "grow" (i32.const 1)) (i32.const 1))
(invoke "grow" (i32.const 1))
(assert_return (invoke "grow" (i32.const 2)) (i32.const 2))
(invoke "grow" (i32.const 2))
(assert_return (invoke "grow" (i32.const 6)) (i32.const 4))
(invoke "grow" (i32.const 6))
(assert_return (invoke "grow" (i32.const 0)) (i32.const 10))
(invoke "grow" (i32.const 0))
(assert_return (invoke "grow" (i32.const 1)) (i32.const -1))
(invoke "grow" (i32.const 1))
(assert_return (invoke "grow" (i32.const 0x10000)) (i32.const -1))


(module
  (table $t 10 funcref)
  (func (export "grow") (param i32) (result i32)
    (table.grow $t (ref.null func) (local.get 0))
  )
  (elem declare func 1)
  (func (export "check-table-null") (param i32 i32) (result funcref)
    (local funcref)
    (local.set 2 (ref.func 1))
    (block
      (loop
        (local.set 2 (table.get $t (local.get 0)))
        (br_if 1 (i32.eqz (ref.is_null (local.get 2))))
        (br_if 1 (i32.ge_u (local.get 0) (local.get 1)))
        (local.set 0 (i32.add (local.get 0) (i32.const 1)))
        (br_if 0 (i32.le_u (local.get 0) (local.get 1)))
      )
    )
    (local.get 2)
  )
)

(assert_return (invoke "check-table-null" (i32.const 0) (i32.const 9)) (ref.null func))
(assert_return (invoke "grow" (i32.const 10)) (i32.const 10))
(invoke "grow" (i32.const 10))
(assert_return (invoke "check-table-null" (i32.const 0) (i32.const 19)) (ref.null func))
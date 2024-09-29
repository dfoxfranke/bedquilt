(module
    (func (export "ret0") (result i64) (i64.const 4294967298)))

(assert_return (invoke "ret0") (i64.const 4294967298))

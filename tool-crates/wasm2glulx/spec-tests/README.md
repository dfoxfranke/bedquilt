# WebAssembly Test Suite

This directory is a fork of <https://github.com/WebAssembly/testsuite/>. It was
necessary to fork the test suite, rather than use it unmodified, because the
code which translates these tests into Wasm2Glulx integration tests handles only
a subset of WAST and some tests had to be adapted to conform to that subset. In
particular: 

1. We don't have a linker, so some modules had to be linked by hand.

2. Each assertion generates a separate test. Any invoke statements which precede
the assertion are run before the assertion is evaluated, but previous assertions
are not. This is different from WAST's intended semantics in which all state
carries over to later statements, even if there was a trap along the way. This
means that when an assertion also has a side effect that later assertions depend
on, that assertion needs to be duplicated as an invocation so that later
assertions see those effects.
# ankoku

[![Rust CI](https://github.com/ankoku-lang/ankoku/actions/workflows/ci.yml/badge.svg)](https://github.com/ankoku-lang/ankoku/actions/workflows/ci.yml)

Ankoku is a small scripting language written in Rust, designed for performance, expressiveness, and ease of embedding.

## Features

(Note: most not implemented yet)

-   [x] not Lua
-   [ ] good FFI with Rust
-   [ ] good performance
-   [ ] fun to write
-   [ ] similar-ish syntax to rust
-   [ ] low footprint (zero dependencies!)

## Name

[Pronounced](https://translate.google.com/?sl=ja&tl=en&text=ankoku&op=translate) aan ko ku. If you can read IPA it's ã̠ŋko̞kɯ̟ᵝ, but nobody can read random Unicode. Japanese for "darkness", because the language is emo.

## MSRV

Currently 1.65.0 because of using std::backtrace::Backtrace, which was [recently stabilized](https://github.com/rust-lang/rust/pull/99573), however this could be fixed by using the backtrace crate.

## todo

-   [x] implement chunks
-   [x] implement start of bytecode interpreter
-   [x] implement hashmaps
-   [x] implement global variables
-   [ ] implement local variables
-   [ ] implement control flow
-   [ ] implement functions
-   [ ] implement closures
-   [ ] implement classes

### parsing

-   [x] implement panic mode and proper errors in parser
-   [x] fix error handling to not suck

### chores

-   [ ] switch to github flow branching and protect `master`

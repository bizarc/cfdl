//! Stub crate for CFDL lexer.
//! Implementation will follow @docs/compiler_spec_v0_1.md.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}
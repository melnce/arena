//! Headless match engine. Card JSON under `cards/` is data; this crate
//! interprets it. Must stay free of `web-sys`, `js-sys`, and `wasm-bindgen`.
#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {
        assert_eq!(2 + 2, 4);
    }
}

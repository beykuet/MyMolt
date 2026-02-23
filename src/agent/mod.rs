// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

pub mod loop_;

pub use loop_::run;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_reexport_exists<F>(_value: F) {}

    #[test]
    fn run_function_is_reexported() {
        assert_reexport_exists(run);
        assert_reexport_exists(loop_::run);
    }
}

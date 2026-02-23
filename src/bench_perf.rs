// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

/// MyMolt Performance Benchmarks
/// Measures real latency of core security/identity operations for patent documentation.
/// Run with: cargo test bench_ -- --nocapture
#[cfg(test)]
mod bench {
    use std::time::{Duration, Instant};

    use crate::identity::soul::{Soul, TrustLevel};
    use crate::security::policy::SecurityPolicy;

    const ITERATIONS: u64 = 100_000;

    fn mean_ns(total: Duration, n: u64) -> f64 {
        (total.as_nanos() as f64) / (n as f64)
    }

    // ── 1. Trust level check ─────────────────────────────────────────────────
    #[test]
    fn bench_check_trust() {
        let mut policy = SecurityPolicy::default();
        policy.set_trust_level(TrustLevel::High);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = policy.check_trust(TrustLevel::High);
        }
        let elapsed = start.elapsed();

        let ns = mean_ns(elapsed, ITERATIONS);
        println!(
            "\n[BENCH] check_trust()          {:>10.2} ns/op  ({} iterations)",
            ns, ITERATIONS
        );
        assert!(ns < 1_000.0, "check_trust must be < 1µs, got {:.2}ns", ns);
    }

    // ── 2. Command allowlist check ───────────────────────────────────────────
    #[test]
    fn bench_is_command_allowed() {
        let policy = SecurityPolicy::default();

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = policy.is_command_allowed("git status");
        }
        let elapsed = start.elapsed();

        let ns = mean_ns(elapsed, ITERATIONS);
        println!(
            "\n[BENCH] is_command_allowed()   {:>10.2} ns/op  ({} iterations)",
            ns, ITERATIONS
        );
        assert!(ns < 5_000.0, "command check must be < 5µs");
    }

    // ── 3. Command risk classification ───────────────────────────────────────
    #[test]
    fn bench_command_risk_level() {
        let policy = SecurityPolicy::default();
        let cmds = [
            "ls -la",
            "git commit -m 'test'",
            "rm -rf /tmp/x",
            "curl http://example.com",
        ];

        let start = Instant::now();
        for i in 0..ITERATIONS {
            let _ = policy.command_risk_level(cmds[(i % 4) as usize]);
        }
        let elapsed = start.elapsed();

        let ns = mean_ns(elapsed, ITERATIONS);
        println!(
            "\n[BENCH] command_risk_level()   {:>10.2} ns/op  ({} iterations)",
            ns, ITERATIONS
        );
        assert!(ns < 10_000.0, "risk classification must be < 10µs");
    }

    // ── 4. Path validation ───────────────────────────────────────────────────
    #[test]
    fn bench_is_path_allowed() {
        let policy = SecurityPolicy::default();

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = policy.is_path_allowed("src/main.rs");
        }
        let elapsed = start.elapsed();

        let ns = mean_ns(elapsed, ITERATIONS);
        println!(
            "\n[BENCH] is_path_allowed()      {:>10.2} ns/op  ({} iterations)",
            ns, ITERATIONS
        );
        assert!(ns < 2_000.0, "path check must be < 2µs");
    }

    // ── 5. Soul load (cold, from tempdir) ────────────────────────────────────
    #[test]
    fn bench_soul_load() {
        let tmp = tempfile::tempdir().unwrap();

        // Pre-create with some bindings
        {
            let mut soul = Soul::new(tmp.path());
            soul.load().unwrap();
            soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
                .unwrap();
            soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
                .unwrap();
        }

        let iterations = 10_000u64;
        let start = Instant::now();
        for _ in 0..iterations {
            let mut soul = Soul::new(tmp.path());
            soul.load().unwrap();
        }
        let elapsed = start.elapsed();

        let ns = mean_ns(elapsed, iterations);
        let us = ns / 1_000.0;
        println!(
            "\n[BENCH] soul_load()            {:>10.2} µs/op  ({} iterations)",
            us, iterations
        );
    }

    // ── 6. Soul identity resolution (in-memory) ──────────────────────────────
    #[test]
    fn bench_soul_max_trust_level() {
        let tmp = tempfile::tempdir().unwrap();
        let mut soul = Soul::new(tmp.path());
        soul.load().unwrap();
        soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
            .unwrap();
        soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
            .unwrap();

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = soul.max_trust_level();
        }
        let elapsed = start.elapsed();

        let ns = mean_ns(elapsed, ITERATIONS);
        println!(
            "\n[BENCH] soul.max_trust_level() {:>10.2} ns/op  ({} iterations)",
            ns, ITERATIONS
        );
        assert!(ns < 500.0, "trust resolution must be < 500ns");
    }

    // ── 7. Full security gate stack (simulate one agent tool call check) ─────
    #[test]
    fn bench_full_gate_stack() {
        let mut policy = SecurityPolicy::default();
        policy.set_trust_level(TrustLevel::High);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            // Simulate: skill allowed? → trust check → command allowed? → risk?
            let _ = policy.is_skill_allowed("calendar_read");
            let _ = policy.check_trust(crate::identity::soul::TrustLevel::Low);
            let _ = policy.is_command_allowed("ls");
            let _ = policy.command_risk_level("ls");
        }
        let elapsed = start.elapsed();

        let ns = mean_ns(elapsed, ITERATIONS);
        let us = ns / 1_000.0;
        println!(
            "\n[BENCH] full_gate_stack()      {:>10.2} µs/op  ({} iterations)",
            us, ITERATIONS
        );
        assert!(us < 10.0, "full gate stack must be < 10µs, got {:.2}µs", us);
    }

    // ── Summary ──────────────────────────────────────────────────────────────
    #[test]
    fn bench_summary() {
        println!("\n");
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║         MyMolt Core — Performance Benchmark Summary          ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Run individual bench_ tests with --nocapture for numbers.  ║");
        println!("║  Target: full security gate < 1.2µs per agent interaction   ║");
        println!("║  Baseline: Rust release build, single-threaded, no I/O      ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
    }
}

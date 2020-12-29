use ::rsp2_integration_test::{CliTest, filetypes, resource, cli_test, Result};

// Check rsp2 against dynmats produced by phonopy.
//
// The workflow to update the test outputs is a bit rough:
//
//    * Check out this hacked version of phonopy (preferrably in a venv):
//        https://github.com/exphp-forks/phonopy/tree/write-dm
//    * phonopy -d --dim "13 13 1" --amplitude 1e-2
//    * Make band.conf:
//          EIGENVECTORS = .TRUE.
//          DIM = 13 13 1
//          BAND = 0 0 0   1/3 1/3 0   1/2 0 0  -1/2 0 0  0 0 0
//          BAND_POINTS = 1
//    * Phonopy's FORCE_SETS are written to low precision, so get one at high precision by
//      setting `RUST_LOG=rsp2_tasks::special::phonopy_force_sets=trace` while running this test.
//    * phonopy band.conf
//    * Assuming you got the hacked version above, `dynmat-${n}.npz` files will have been written.

#[ignore] // This test is expensive; use `cargo test -- --ignored` to run it!
#[test]
fn dynmat_at_q() -> Result<()> {
    let env = cli_test::Environment::init();

    for &(ref expected_outfile, kpoint) in &[
        (resource("dynmat-at-q/001-0-a-gamma.dynmat.npz"), "0 0 0"),
        (resource("dynmat-at-q/001-0-a-k.dynmat.npz"), "1/3 1/3 0"),
        (resource("dynmat-at-q/001-0-a-m.dynmat.npz"), "0.5 0 0"),
        (resource("dynmat-at-q/001-0-a-m-neg.dynmat.npz"), "-0.5 0 0"),
    ] {
        println!("Testing kpoint {}", kpoint);
        CliTest::cargo_binary(&env, "rsp2-dynmat-at-q")
            .arg("-c").arg(resource("dynmat-at-q/settings.yaml"))
            .arg(resource("dynmat-at-q/001-0-a.structure"))
            .arg("--qpoint").arg(kpoint)
            .arg("-o").arg("dynmat.npz")
            .check_file::<filetypes::Dynmat>(
                "dynmat.npz".as_ref(),
                expected_outfile.as_ref(),
                filetypes::DynmatTolerances {
                    rel_tol: 1e-4,
                    abs_tol: 1e-7,
                },
            )
            .run()?;
    }
    Ok(())
}

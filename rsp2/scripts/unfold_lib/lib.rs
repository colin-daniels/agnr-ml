use rsp2_array_types::{V3};
use rsp2_soa_ops::{Perm, Permute};

use std::os::raw::c_char;
use std::ffi::CStr;

use slice_of_array::prelude::*;
use rayon::prelude::*;
use num_complex::Complex64;

#[macro_use] extern crate rsp2_util_macros;
#[macro_use] extern crate rsp2_assert_close;

#[no_mangle]
pub extern "C" fn rsp2c_unfold_all(
    num_quotient: i64,
    num_sites: i64,
    num_evecs: i64,
    gamma_only: u8,
    progress_prefix: *const c_char, // NUL-terminated UTF-8, possibly NULL
    site_phases: *const Complex64, // shape (sites,)
    gpoint_sfracs: *const f64, // shape (quotient, 3)
    qpoint_sfrac: *const f64, // shape (3,)
    eigenvectors: *const Complex64, // shape (evecs, sites, 3)
    translation_sfracs: *const f64, // shape (quotient, 3)
    translation_deperms: *const i32, // shape (quotient, sites)
    translation_phases: *const Complex64, // shape (quotient, sites)
    output: *mut f64, // shape (evecs, quotient)
    // return is nonzero on error
) -> i32 {
    match std::panic::catch_unwind(|| {
        let num_quotient = num_quotient as usize;
        let num_sites = num_sites as usize;
        let num_evecs = num_evecs as usize;

        use std::slice::{from_raw_parts, from_raw_parts_mut};
        unsafe {
            let site_phases = from_raw_parts(site_phases, num_sites);
            let gpoint_sfracs = from_raw_parts(gpoint_sfracs, num_quotient * 3).nest();
            let qpoint_sfrac = from_raw_parts(qpoint_sfrac, 3).as_array();
            let eigenvectors = from_raw_parts(eigenvectors, num_evecs * num_sites * 3).nest();
            let translation_sfracs = from_raw_parts(translation_sfracs, num_quotient * 3).nest();
            let translation_deperms = from_raw_parts(translation_deperms, num_quotient * num_sites);
            let translation_phases = from_raw_parts(translation_phases, num_quotient * num_sites);
            let output = from_raw_parts_mut(output, num_evecs * num_quotient);
            let gamma_only = gamma_only != 0;

            let progress_prefix = if progress_prefix.is_null() {
                None
            } else {
                Some(CStr::from_ptr(progress_prefix).to_str().unwrap())
            };

            unfold_all(
                progress_prefix,
                site_phases,
                gpoint_sfracs,
                qpoint_sfrac,
                eigenvectors,
                translation_sfracs,
                translation_deperms,
                translation_phases,
                gamma_only,
                output,
            );
        }
    }) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

fn unfold_all(
    progress_prefix: Option<&str>,
    site_phases: &[Complex64],
    gpoint_sfracs: &[V3],
    qpoint_sfrac: &V3,
    eigenvectors: &[V3<Complex64>],
    translation_sfracs: &[V3],
    translation_deperms: &[i32],
    translation_phases: &[Complex64],
    gamma_only: bool,
    output: &mut [f64],
) {
    let num_sites = site_phases.len();
    let num_quotient = translation_sfracs.len();

    let ref translation_deperms: Vec<Perm> = {
        translation_deperms.chunks(num_sites)
            .map(|perm| Perm::from_vec(perm.iter().map(|&i| i as usize).collect()).expect("invalid perm!"))
            .collect()
    };

    let ref translation_phases: Vec<&[Complex64]> = {
        translation_phases.chunks(num_sites).collect()
    };

    let overwrite_line = atty::is(atty::Stream::Stdout);
    let progress = progress_prefix.map(|prefix| {
        let end = if overwrite_line { "\r" } else { "\n" };
        move |done, total| print!("{}Unfolded {:>5} of {} eigenvectors{}", prefix, done, total, end)
    });

    let iter = zip_eq!(eigenvectors.chunks(num_sites), output.chunks_mut(num_quotient)).enumerate();
    let num_total = iter.len();
    for (num_complete, (eigenvector, output)) in iter {
        if let Some(progress) = progress.as_ref() {
            progress(num_complete, num_total);
        }

        let dense_row = unfold_one(
            site_phases,
            translation_sfracs,
            translation_deperms,
            translation_phases,
            gpoint_sfracs,
            qpoint_sfrac,
            eigenvector,
            gamma_only,
        );
        output.copy_from_slice(&dense_row);
    }

    if let Some(progress) = progress {
        progress(num_total, num_total);
        if overwrite_line {
            println!();
        }
    }
}

/// See `unfold_one` in the python code for documentation of the arguments.
fn unfold_one(
    site_phases: &[Complex64],
    translation_sfracs: &[V3],
    translation_deperms: &[Perm],
    translation_phases: &[&[Complex64]],
    gpoint_sfracs: &[V3],
    qpoint_sfrac: &V3,
    eigenvector: &[V3<Complex64>],
    gamma_only: bool,
) -> Vec<f64> {
    let num_quotient = translation_sfracs.len();

    // our "Frankenstein bloch function" with the magnitudes of the eigenvector and
    // the phases of the displacement vector.
    let ref bloch_function: Vec<_> = {
        zip_eq!(eigenvector, site_phases)
            .map(|(&v3, phase)| v3.map(|x| x * phase))
            .collect()
    };

    // Expectation value of each translation operator.
    let inner_prods: Vec<_> = {
        translation_deperms.par_iter().zip_eq(translation_phases).map(|(perm, &image_phases)| {
            let permuted = bloch_function.to_vec().permuted_by(perm);
            zip_eq!(bloch_function, image_phases, permuted)
                .map(|(orig_v3, &image_phase, perm_v3)| {
                    let true_translated_v3 = perm_v3.map(|x| x * image_phase);
                    inner_prod_v3(orig_v3, &true_translated_v3)
                })
                .sum::<Complex64>()
        }).collect()
    };

    // Expectation value of each P(Q -> Q + G) projection operator.
    let compute_at_g = |g: &V3| {
        let phases: Vec<_> = {
            translation_sfracs.iter()
                // Phases from Allen Eq 3.  Due to our differing phase conventions,
                // we have exp(+i...) rather than exp(-i...).
                .map(|t| exp_i2pi(V3::dot(&(qpoint_sfrac + g), t)))
                .collect()
        };
        let prob = zip_eq!(&inner_prods, phases).map(|(a, b)| a * b).sum::<Complex64>() / num_quotient as f64;

        // analytically, these are all real, positive numbers
        //
        // numerically, however, cancellation may cause issues
        assert!(f64::abs(prob.im) < 1e-7);
        assert!(-1e-7 < prob.re);

        f64::max(prob.re, 0.0)
    };

    let gpoint_probs: Vec<_> = if gamma_only {
        assert_eq!(gpoint_sfracs[0], V3::zero());
        let mut gpoint_probs = vec![0.0; gpoint_sfracs.len()];
        gpoint_probs[0] = compute_at_g(&gpoint_sfracs[0]);
        gpoint_probs
    } else {
        gpoint_sfracs.par_iter().map(compute_at_g).collect()
    };

    if !gamma_only {
        assert_close!(
            abs=1e-7,
            gpoint_probs.iter().sum::<f64>(),
            inner_prod_ev(eigenvector, eigenvector).norm(),
        );
    }
    gpoint_probs
}

const TWO_PI_I: Complex64 = Complex64 { re: 0.0, im: 2.0 * std::f64::consts::PI };
fn exp_i2pi(x: f64) -> Complex64 {
    Complex64::exp(&(TWO_PI_I * x))
}

fn inner_prod_ev(a: &[V3<Complex64>], b: &[V3<Complex64>]) -> Complex64 {
    zip_eq!(a, b).map(|(a, b)| inner_prod_v3(a, b)).sum()
}

fn inner_prod_v3(a: &V3<Complex64>, b: &V3<Complex64>) -> Complex64 {
    V3::from_fn(|i| a[i].conj() * b[i]).0.iter().sum()
}

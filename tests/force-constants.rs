#[macro_use] extern crate rsp2_assert_close;
#[macro_use] extern crate rsp2_integration_test;

#[macro_use] extern crate serde_derive;


type FailResult<T> = Result<T, ::failure::Error>;

use rsp2_integration_test::{resource, filetypes::Primitive};
use rsp2_dynmat::SuperForceConstants;
use rsp2_array_types::{M33, V3, Unvee};
use rsp2_structure::{Coords, supercell::SupercellToken};
use rsp2_soa_ops::{Permute};

#[derive(Deserialize)]
pub struct ForceSets {
    #[serde(rename =       "sc-dims")] sc_dims: [u32; 3],
    #[serde(rename =    "force-sets")] orig_super_force_sets: Vec<Vec<(usize, V3)>>, // [disp] -> [(super, V3)]
    #[serde(rename = "displacements")] orig_displacements: Vec<(usize, V3)>, // [disp] -> (super, V3)
    #[serde(rename =     "structure")] orig_super_coords: Coords,
}
impl_json!{ (ForceSets)[load] }

#[derive(Deserialize)]
struct InputForceConstants {
    #[serde(rename =   "sc-dims")] sc_dims: [u32; 3],
    #[serde(rename =     "dense")] orig_force_constants: Vec<Vec<M33>>, // [super][super]
    #[serde(rename = "structure")] orig_super_coords: Coords,
}
impl_json!{ (InputForceConstants)[load] }

#[derive(Deserialize)]
struct OutputForceConstants {
    #[serde(rename = "dense")] orig_force_constants: Vec<Vec<M33>>, // [super][super]
}
impl_json!{ (OutputForceConstants)[load] }

#[derive(Deserialize)]
struct OutputDynMat {
    #[serde(rename = "real")] expected_dynmat_real: Vec<Vec<M33>>, // [super][super]
    #[serde(rename = "imag")] expected_dynmat_imag: Vec<Vec<M33>>, // [super][super]
}
impl_json!{ (OutputDynMat)[load] }

#[derive(Copy, Clone)]
struct Tolerances {
    relative: f64,
    absolute: f64,
}

fn check(
    prim_info_relpath: &str,
    super_info_relpath: &str,
    expected_fc_relpath: Option<&str>,
    expected_dynmat_relpath: &str,
    qpoint_frac: V3,
    tol: Tolerances,
) -> FailResult<()> {
    let Primitive {
        cart_ops,
        masses: prim_masses,
        coords: prim_coords,
    } = Primitive::load(resource(prim_info_relpath))?;

    let ForceSets {
        sc_dims, orig_super_coords, orig_super_force_sets, orig_displacements,
    } = ForceSets::load(resource(super_info_relpath))?;

    let orig_expected_fcs = expected_fc_relpath.map(|relpath| {
        let matrix = OutputForceConstants::load(resource(relpath)).unwrap().orig_force_constants;
        SuperForceConstants::from_dense_matrix(matrix)
    });

    let OutputDynMat {
        expected_dynmat_real,
        expected_dynmat_imag,
    } = OutputDynMat::load(resource(expected_dynmat_relpath))?;

    let (super_coords, sc) = ::rsp2_structure::supercell::diagonal(sc_dims).build(&prim_coords);

    // permute expected output into correct order for the current supercell convention
    // (this is to be robust to changes in supercell ordering convention)
    let coperm_from_orig = orig_super_coords.perm_to_match(&super_coords, 1e-10)?;
    let deperm_from_orig = coperm_from_orig.inverted();

    let super_force_sets: Vec<_> = {
        orig_super_force_sets
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|(atom, v)| (deperm_from_orig.permute_index(atom), v))
                    .collect()
            })
            .collect()
    };

    let super_displacements: Vec<_> = {
        orig_displacements.into_iter()
            .map(|(orig_atom, v3)| {
                let atom = deperm_from_orig.permute_index(orig_atom);
                (atom, v3)
            })
            .collect()
    };

    let super_sg_deperms: Vec<_> = {
        ::rsp2_structure::find_perm::spacegroup_deperms(
            &super_coords,
            &cart_ops,
            1e-1,
        )?
    };

    let cart_rots = cart_ops.iter().map(|c| c.cart_rot()).collect::<Vec<_>>();

    // ---------------------------------
    // ------- Force constants ---------
    // ---------------------------------

    let force_constants = rsp2_dynmat::ForceConstants::compute_required_rows(
        &super_displacements,
        &super_force_sets,
        &cart_rots,
        &super_sg_deperms,
        &sc,
    )?;

    if let Some(orig_expected_fcs) = orig_expected_fcs {
        let expected = orig_expected_fcs.permuted_by(&deperm_from_orig);
        let actual = force_constants.to_super_force_constants_with_zeroed_rows(&sc);
        assert_fcs_close(&sc, &actual, &expected, tol);
    }

    // ----------------------------------
    // ------- Dynamical Matrix ---------
    // ----------------------------------

    let dynamical_matrix = {
        let qpoint_cart = qpoint_frac * &prim_coords.lattice().reciprocal();

        force_constants
            .dynmat_at_cart_q(&super_coords, qpoint_cart, &sc, &prim_masses)
            .hermitianize()
    };

    {
        let real = dynamical_matrix.0.to_coo().map(|c| c.0).into_dense();
        let imag = dynamical_matrix.0.to_coo().map(|c| c.1).into_dense();
        assert_eq!(real.len(), sc.num_primitive_atoms());
        assert_eq!(real[0].len(), sc.num_primitive_atoms());
        for r in 0..sc.num_primitive_atoms() {
            for c in 0..sc.num_primitive_atoms() {
                assert_close!(
                    rel=tol.relative, abs=tol.absolute,
                    real[r][c].unvee(),
                    expected_dynmat_real[r][c].unvee(),
                    "{:?}", (real, imag),
                );
                assert_close!(
                    rel=tol.relative, abs=tol.absolute,
                    imag[r][c].unvee(),
                    expected_dynmat_imag[r][c].unvee(),
                    "{:?}", (real, imag),
                );
            }
        }
    }
    Ok(())
}

fn check_translational_invariance(
    prim_info_relpath: &str,
    initial_fc_relpath: &str,
    expected_fc_relpath: &str,
    level: u32,
    tol: Tolerances,
) -> FailResult<()> {
    let Primitive {
        cart_ops: _,
        masses: _,
        coords: prim_coords,
    } = Primitive::load(resource(prim_info_relpath))?;

    let InputForceConstants {
        orig_force_constants: orig_input,
        sc_dims, orig_super_coords,
    } = InputForceConstants::load(resource(initial_fc_relpath))?;

    let OutputForceConstants {
        orig_force_constants: orig_output,
    } = OutputForceConstants::load(resource(expected_fc_relpath))?;

    let orig_input = SuperForceConstants::from_dense_matrix(orig_input);
    let orig_output = SuperForceConstants::from_dense_matrix(orig_output);

    let (super_coords, sc) = rsp2_structure::supercell::diagonal(sc_dims).build(&prim_coords);

    // permute expected output into correct order for the current supercell convention
    // (this is to be robust to differences in supercell ordering convention)
    let coperm_from_orig = orig_super_coords.perm_to_match(&super_coords, 1e-10)?;
    let deperm_from_orig = coperm_from_orig.inverted();

    let input = orig_input.permuted_by(&deperm_from_orig);
    let expected_output = orig_output.permuted_by(&deperm_from_orig);

    let output = {
        input.drop_non_designated_rows(&sc)
            .impose_translational_invariance(&sc, level)
            .to_super_force_constants_with_zeroed_rows(&sc)
    };

    assert_fcs_close(&sc, &output, &expected_output, tol);
    Ok(())
}

fn assert_fcs_close(
    sc: &SupercellToken,
    left: &SuperForceConstants,
    right: &SuperForceConstants,
    tol: Tolerances,
) {
    let left = left.to_dense_matrix();
    let right = right.to_dense_matrix();
    assert_eq!(left.len(), sc.num_supercell_atoms());
    assert_eq!(left[0].len(), sc.num_supercell_atoms());

    println!("{:?}", left);
    for r in 0..sc.num_supercell_atoms() {
        for c in 0..sc.num_supercell_atoms() {
            assert_close!(
                rel=tol.relative, abs=tol.absolute,
                left[r][c].unvee(),
                right[r][c].unvee(),
            );
        }
    }
}

#[test]
fn graphene_denseforce_771() {
    check(
        // * Graphene
        "primitive/graphene.json",
        // * Dense force sets
        // * [7, 7, 1] supercell
        "force-constants/graphene-771-dense.super.json",
        Some("force-constants/graphene-771.fc.json.xz"),
        "force-constants/graphene-gamma.dynmat.json",
        V3::zero(),
        Tolerances { relative: 1e-10, absolute: 1e-12 },
    ).unwrap()
}

#[test]
fn graphene_denseforce_111() {
    check(
        // * Graphene
        "primitive/graphene.json",
        // * Dense force sets
        // * [1, 1, 1] supercell
        "force-constants/graphene-111-dense.super.json",
        Some("force-constants/graphene-111.fc.json"),
        // * same dynmat output as for 771; a supercell should not be necessary
        //   for the dynamical matrix at gamma
        //
        // FIXME:
        //   That is not true! If multiple images of an atom move in graphene,
        //   it will affect the bond angle terms.
        //   What's going on here?  How were these force sets obtained?
        "force-constants/graphene-gamma.dynmat.json",
        V3::zero(),
        Tolerances { relative: 1e-10, absolute: 1e-12 },
    ).unwrap()
}

#[test]
fn graphene_sparseforce_771() {
    check(
        // * Graphene
        "primitive/graphene.json",
        // * Sparse force sets (clipped to zero at 1e-12, ~70% sparse)
        // * [7, 7, 1] supercell
        "force-constants/graphene-771-sparse.super.json",
        Some("force-constants/graphene-771.fc.json.xz"),

        // I don't have data computed specifically for this, just use
        // the existing data with a more relaxed tolerance
        "force-constants/graphene-gamma.dynmat.json",
        V3::zero(),
        Tolerances { relative: 1e-6, absolute: 1e-7 },
    ).unwrap()
}

#[test]
fn blg_force_sets_gamma() {
    check(
        "primitive/blg.json",
        "force-constants/blg.super.json",
        None, // no fc file
        "force-constants/blg-gamma.dynmat.json",
        V3::zero(),
        Tolerances { relative: 1e-10, absolute: 1e-12 },
    ).unwrap()
}

#[test]
fn blg_force_sets_k() {
    check(
        "primitive/blg.json",
        "force-constants/blg.super.json",
        None, // no fc file
        "force-constants/blg-k.dynmat.json",
        V3([1.0/3.0, 1.0/3.0, 0.0]),
        Tolerances { relative: 1e-10, absolute: 1e-12 },
    ).unwrap()
}

#[test]
fn blg_force_sets_m() {
    check(
        "primitive/blg.json",
        "force-constants/blg.super.json",
        None, // no fc file
        "force-constants/blg-m.dynmat.json",
        V3([0.5, 0.0, 0.0]),
        Tolerances { relative: 1e-10, absolute: 1e-12 },
    ).unwrap()
}

//------------------------------------------------------------
// Translational invariance tests

// TO CREATE THE INPUT FILES:
//
// Create `POSCAR` from `resources/primitive/blg.json`, then run phonopy with a 3x3x1 unit cell
// and give it some arbitrary `FORCE_SETS`. (e.g. run rsp2 on `SPOSCAR` with
// `RUST_LOG=rsp2_tasks::special::phonopy_force_sets=trace` and `disp-finder: {phonopy: {}}`).
// The values in the `FORCE_SETS` will be ignored, but they must have the correct atom numbers and
// sizes for phonopy's displacements.
//
// Then check out this commit of phonopy:
// https://github.com/exphp-forks/phonopy/tree/debug-symmetrize
//
// and do the following:
//
// $ phonopy --fc-symmetry --dim '3 3 1' # generates the json files
// $ python3
// >>> from pymatgen.io.vasp import Poscar
// >>> structure = Poscar.from_file('SPOSCAR').structure
// >>> dense=np.array(json.load(open('output-2-rsp2.json')))
// >>> json.dump({ 'dense': dense.tolist() }, open('symmetrize-output-2.fc.json', 'w'))
// >>> dense=np.array(json.load(open('output-0-rsp2.json')))
// >>> json.dump({ 'dense': dense.tolist() }, open('symmetrize-output-0.fc.json', 'w'))
// >>> dense=np.array(json.load(open('input-rsp2.json')))
// >>> json.dump({
// ... 'dense': dense.tolist(), "sc-dims": [3, 3, 1],
// ... "structure": {"lattice": structure.lattice.matrix.tolist(),
// ... "carts": structure.cart_coords.tolist()}}, open('symmetrize-input.json', 'w'))

const TRANS_INVARIANCE_TOLS: Tolerances = Tolerances { relative: 1e-10, absolute: 1e-12 };
const TRANS_INVARIANCE_TOLS_WEAK: Tolerances = Tolerances { relative: 1e-4, absolute: 1e-5 };

#[test]
fn impose_translational_invariance_0() {
    check_translational_invariance(
        "primitive/blg.json",
        "force-constants/symmetrize-input.json.xz",
        "force-constants/symmetrize-output-0.fc.json.xz",
        // level = 0 is effectively equivalent to a call to `impose_translational_symmetry`,
        // allowing us to test that function in isolation.
        0,
        TRANS_INVARIANCE_TOLS,
    ).unwrap()
}

#[test]
fn impose_translational_invariance_2() {
    check_translational_invariance(
        "primitive/blg.json",
        "force-constants/symmetrize-input.json.xz",
        "force-constants/symmetrize-output-2.fc.json.xz",
        2, // level
        TRANS_INVARIANCE_TOLS,
    ).unwrap()
}

// try matching the level with the wrong output file to verify that the test would fail
#[test]
#[should_panic(expected = "not nearly equal")]
fn impose_translational_invariance_specificity() {
    check_translational_invariance(
        "primitive/blg.json",
        "force-constants/symmetrize-input.json.xz",
        "force-constants/symmetrize-output-0.fc.json.xz",
        2, // level
        TRANS_INVARIANCE_TOLS_WEAK,
    ).unwrap()
}

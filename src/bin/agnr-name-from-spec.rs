use agnr_gen::generate::spec::AGNRSpec;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let mut spec: Vec<(i32, i32)> = serde_json::from_str(&line?)?;
        // ensure min is zero
        let min = spec.iter().map(|s| s.0).min().unwrap();
        for s in &mut spec {
            *s = (s.0 - min, s.1 - min);
        }

        let mut temp = spec.clone();
        let width = spec.iter().map(|s| s.1).max().unwrap();

        // get the "minimum" spec out of all possible images of the GNR
        for &_x_mirror in &[false, true] {
            for &_y_mirror in &[false, true] {
                for _shift in 0..temp.len() {
                    // update min
                    if temp < spec {
                        spec.copy_from_slice(&temp);
                    }

                    // translations
                    temp.rotate_right(1);
                }
                // y mirror plane
                temp.reverse();
            }
            // x mirror plane
            for v in &mut temp {
                *v = (width - v.1, width - v.0);
            }
        }

        // build gnr "name" from spec
        let spec = AGNRSpec {
            spec,
            structure: None,
            symmetries: vec![],
        };
        println!("{}", spec.name().unwrap());
    }

    Ok(())
}

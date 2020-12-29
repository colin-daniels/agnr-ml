use agnr_gen::generate::spec::AGNRSpec;
use std::io::{self, BufRead};
use vasp_poscar::Poscar;

fn poscar_from_spec_line(line: &str) -> Result<Poscar, Box<dyn std::error::Error>> {
    let spec = AGNRSpec {
        spec: serde_json::from_str(line)?,
        structure: None,
        symmetries: vec![],
    };

    Ok(spec.to_poscar())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let poscar_str = format!("{}", poscar_from_spec_line(&line?)?);
        // convert to json string and output in one line
        println!("{}", serde_json::to_string(&poscar_str)?);
    }

    Ok(())
}

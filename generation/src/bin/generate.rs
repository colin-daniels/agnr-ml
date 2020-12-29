use agnr_gen::generate::generate_gnrs_json;

fn main() {
    // generate_gnrs_json(1..10, 2..6, true, 5, "gnrs-symm-max-w5.json");
    // generate_gnrs_json(1..8, 2..7, false, 6, "gnrs-no-symm-max-w6.json");
    generate_gnrs_json(1..9, 2..7, false, 6, "gnrs-no-symm-max-w6-spec-only.json");
}

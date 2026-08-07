fn main() {
    match tabletennis_tournament::simulation::run_standard_scenarios() {
        Ok(reports) => {
            for report in reports {
                println!("{report}");
            }
        }
        Err(error) => {
            eprintln!("simulation failed: {error}");
            std::process::exit(1);
        }
    }
}

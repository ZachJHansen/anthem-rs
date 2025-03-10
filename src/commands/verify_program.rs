pub fn run<P1, P2>(program_path: P1, specification_paths: &[P2],
	proof_direction: crate::problem::ProofDirection, no_simplify: bool,
	color_choice: crate::output::ColorChoice, time_limit: u64, cores: u64)
where
	P1: AsRef<std::path::Path>,
	P2: AsRef<std::path::Path>,
{
	let mut problem = crate::Problem::new(color_choice);

	for specification_path in specification_paths
	{
		log::info!("reading specification file “{}”", specification_path.as_ref().display());

		let specification_content = match std::fs::read_to_string(specification_path.as_ref())
		{
			Ok(specification_content) => specification_content,
			Err(error) =>
			{
				log::error!("could not access specification file: {}", error);
				std::process::exit(1)
			},
		};

		// TODO: rename to read_specification
		if let Err(error) = crate::input::parse_specification(&specification_content, &mut problem)
		{
			log::error!("could not read specification file: {}", error);
			std::process::exit(1)
		}

		log::info!("read specification “{}”", specification_path.as_ref().display());
	}

	problem.process_output_predicates();

	log::info!("reading input program “{}”", program_path.as_ref().display());

	// TODO: make consistent with specification call (path vs. content)
	if let Err(error) = crate::translate::verify_properties::Translator::new(&mut problem)
		.translate(program_path)
	{
		log::error!("could not translate input program: {}", error);
		std::process::exit(1)
	}

	if let Err(error) = problem.check_consistency(proof_direction)
	{
		match error.kind
		{
			// In forward proofs, it’s okay to use private predicates in the specification, but
			// issue a warning regardless
			crate::error::Kind::PrivatePredicateInSpecification(_)
				if !proof_direction.requires_backward_proof() => log::warn!("{}", error),
            crate::error::Kind::ProgramNotTight(_) => 
            {
                log::warn!("{}", error);
                std::process::exit(0)
            },
			_ =>
			{
				log::error!("{}", error);
				std::process::exit(1)
			},
		}
	}

	if !no_simplify
	{
		if let Err(error) = problem.simplify()
		{
			log::error!("could not simplify translated program: {}", error);
			std::process::exit(1)
		}
	}

	if let Err(error) = problem.prove(proof_direction, time_limit, cores)
	{
		log::error!("could not verify program: {}", error);
		std::process::exit(1)
	}
}

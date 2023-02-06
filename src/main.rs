use structopt::StructOpt as _;

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "anthem", about = "Use first-order theorem provers with answer set programs.")]
enum Command
{
	#[structopt(about = "Verifies a logic program against a specification")]
	#[structopt(aliases = &["vprog"])]
	VerifyProgram
	{
		/// ASP input program file path
		#[structopt(name = "program", parse(from_os_str), required(true))]
		program_path: std::path::PathBuf,

		/// One or more specification file paths
		#[structopt(name = "specification", parse(from_os_str), required(true))]
		specification_paths: Vec<std::path::PathBuf>,

		/// Proof direction (forward, backward, both)
		#[structopt(long, default_value = "both")]
		proof_direction: anthem::problem::ProofDirection,

		/// Do not simplify translated program
		#[structopt(long)]
		no_simplify: bool,

		/// Whether to use colors in the output (auto, always, never)
		#[structopt(name = "color", long, default_value = "auto")]
		color_choice: anthem::output::ColorChoice,

        /// Time limit for Vampire (seconds)
        #[structopt(long, default_value = "300")]
        time_limit: u64,
	}
}

fn main()
{
	pretty_env_logger::init_custom_env("ANTHEM_LOG");

	let command = Command::from_args();

	match command
	{
		Command::VerifyProgram
		{
			program_path,
			specification_paths,
			proof_direction,
			no_simplify,
			color_choice,
            time_limit,
		}
			=> anthem::commands::verify_program::run(&program_path, specification_paths.as_slice(),
				proof_direction, no_simplify, color_choice, time_limit),
	}
}

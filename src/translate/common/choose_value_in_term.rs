pub(crate) fn choose_value_in_primitive(term: Box<crate::Term>,
	variable_declaration: std::rc::Rc<crate::VariableDeclaration>)
	-> crate::Formula
{
	let variable = crate::Term::variable(variable_declaration);

	crate::Formula::equal(Box::new(variable), term)
}

pub(crate) fn choose_value_in_term<C>(term: &clingo::ast::Term,
	variable_declaration: std::rc::Rc<crate::VariableDeclaration>, context: &C,
	variable_declaration_stack: &crate::VariableDeclarationStackLayer)
	-> Result<crate::Formula, crate::Error>
where
	C: foliage::FindOrCreateFunctionDeclaration<crate::FoliageFlavor>,
{
	match term.term_type()
	{
		clingo::ast::TermType::Symbol(symbol) => match symbol.symbol_type()
			.map_err(|error| crate::Error::new_logic("clingo error").with(error))?
		{
			clingo::SymbolType::Number => Ok(choose_value_in_primitive(
				Box::new(crate::Term::integer(symbol.number()
					.map_err(|error| crate::Error::new_logic("clingo error").with(error))?)),
				variable_declaration)),
			clingo::SymbolType::Infimum => Ok(choose_value_in_primitive(
				Box::new(crate::Term::infimum()), variable_declaration)),
			clingo::SymbolType::Supremum => Ok(choose_value_in_primitive(
				Box::new(crate::Term::supremum()), variable_declaration)),
			clingo::SymbolType::String => Ok(choose_value_in_primitive(
				Box::new(crate::Term::string(symbol.string()
					.map_err(|error| crate::Error::new_logic("clingo error").with(error))?
					.to_string())),
				variable_declaration)),
			clingo::SymbolType::Function =>
			{
				let arguments = symbol.arguments()
					.map_err(|error| crate::Error::new_logic("clingo error").with(error))?;

				// Functions with arguments are represented as clingo::ast::Function by the parser.
				// At this point, we only have to handle (0-ary) constants
				if !arguments.is_empty()
				{
					return Err(crate::Error::new_logic(
						"unexpected arguments, expected (0-ary) constant symbol"));
				}

				let constant_name = symbol.name()
					.map_err(|error| crate::Error::new_logic("clingo error").with(error))?;

				let constant_declaration =
					context.find_or_create_function_declaration(constant_name, 0);
				let function = crate::Term::function(constant_declaration, vec![]);

				Ok(choose_value_in_primitive(Box::new(function), variable_declaration))
			}
		},
		clingo::ast::TermType::Variable(variable_name) =>
		{
			let other_variable_declaration = match variable_name
			{
				// Every occurrence of anonymous variables is treated as if it introduced a fresh
				// variable declaration
				"_" => variable_declaration_stack.free_variable_declarations_do_mut(
					|free_variable_declarations|
					{
						// TODO: check domain type
						let variable_declaration = std::rc::Rc::new(
							crate::VariableDeclaration::new_generated(crate::Domain::Program));

						free_variable_declarations.push(std::rc::Rc::clone(&variable_declaration));

						variable_declaration
					}),
				_ => variable_declaration_stack.find_or_create(variable_name),
			};
			let other_variable = crate::Term::variable(other_variable_declaration);

			Ok(choose_value_in_primitive(Box::new(other_variable), variable_declaration))
		},
		clingo::ast::TermType::BinaryOperation(binary_operation) =>
		{
			let operator = super::translate_binary_operator(binary_operation.binary_operator())?;

			match operator
			{
				foliage::BinaryOperator::Add
				| foliage::BinaryOperator::Subtract
				| foliage::BinaryOperator::Multiply
					=>
				{
					let parameters = (0..2).map(
						|_| std::rc::Rc::new(crate::VariableDeclaration::new_generated(
							crate::Domain::Integer)))
						.collect::<crate::VariableDeclarations>();
					let parameters = std::rc::Rc::new(parameters);

					let parameter_1 = &parameters[0];
					let parameter_2 = &parameters[1];

					let translated_binary_operation = crate::Term::binary_operation(operator,
						Box::new(crate::Term::variable(std::rc::Rc::clone(&parameter_1))),
						Box::new(crate::Term::variable(std::rc::Rc::clone(&parameter_2))));

					let equals = crate::Formula::equal(
						Box::new(crate::Term::variable(variable_declaration)),
						Box::new(translated_binary_operation));

					let choose_value_from_left_argument = choose_value_in_term(
						binary_operation.left(), std::rc::Rc::clone(&parameter_1), context,
						variable_declaration_stack)?;

					let choose_value_from_right_argument = choose_value_in_term(
						binary_operation.right(), std::rc::Rc::clone(&parameter_2), context,
							variable_declaration_stack)?;

					let and = crate::Formula::And(vec![equals, choose_value_from_left_argument,
						choose_value_from_right_argument]);

					Ok(crate::Formula::exists(parameters, Box::new(and)))
				},
				foliage::BinaryOperator::Divide
				| foliage::BinaryOperator::Modulo
					=>
				{
					let parameters = (0..4).map(
						|_| std::rc::Rc::new(crate::VariableDeclaration::new_generated(
							crate::Domain::Integer)))
						.collect::<crate::VariableDeclarations>();
					let parameters = std::rc::Rc::new(parameters);

					let parameter_i = &parameters[0];
					let parameter_j = &parameters[1];
					let parameter_q = &parameters[2];
					let parameter_r = &parameters[3];

					let j_times_q = crate::Term::multiply(
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_j))),
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_q))));

					let j_times_q_plus_r = crate::Term::add(Box::new(j_times_q),
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_r))));

					let i_equals_j_times_q_plus_r = crate::Formula::equal(
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_i))),
						Box::new(j_times_q_plus_r));

					let choose_i_in_t1 = choose_value_in_term(binary_operation.left(),
						std::rc::Rc::clone(parameter_i), context, variable_declaration_stack)?;

					let choose_j_in_t2 = choose_value_in_term(binary_operation.right(),
						std::rc::Rc::clone(parameter_j), context, variable_declaration_stack)?;

					let j_not_equal_to_0 = crate::Formula::not_equal(
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_j))),
						Box::new(crate::Term::integer(0)));

					let r_greater_or_equal_to_0 = crate::Formula::greater_or_equal(
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_r))),
						Box::new(crate::Term::integer(0)));

					let r_less_than_j = crate::Formula::less(
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_r))),
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_j))));

					let z_equal_to_q = crate::Formula::equal(
						Box::new(
							crate::Term::variable(std::rc::Rc::clone(&variable_declaration))),
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_q))));

					let z_equal_to_r = crate::Formula::equal(
						Box::new(crate::Term::variable(variable_declaration)),
						Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_r))));

					let last_argument = match operator
					{
						foliage::BinaryOperator::Divide => z_equal_to_q,
						foliage::BinaryOperator::Modulo => z_equal_to_r,
						_ => return Err(crate::Error::new_logic("unreachable code")),
					};

					let and = crate::Formula::and(vec![i_equals_j_times_q_plus_r, choose_i_in_t1,
						choose_j_in_t2, j_not_equal_to_0, r_greater_or_equal_to_0, r_less_than_j,
						last_argument]);

					Ok(crate::Formula::exists(parameters, Box::new(and)))
				},
				foliage::BinaryOperator::Exponentiate =>
					Err(crate::Error::new_unsupported_language_feature("exponentiation")),
			}
		},
		clingo::ast::TermType::UnaryOperation(unary_operation) =>
		{
			match unary_operation.unary_operator()
			{
				clingo::ast::UnaryOperator::Absolute =>
					return Err(crate::Error::new_unsupported_language_feature("absolute value")),
				clingo::ast::UnaryOperator::Minus =>
				{
					let parameter_z_prime = std::rc::Rc::new(
						crate::VariableDeclaration::new_generated(crate::Domain::Integer));

					let negative_z_prime = crate::Term::negative(Box::new(
						crate::Term::variable(std::rc::Rc::clone(&parameter_z_prime))));
					let equals = crate::Formula::equal(
						Box::new(crate::Term::variable(variable_declaration)),
						Box::new(negative_z_prime));

					let choose_z_prime_in_t_prime = choose_value_in_term(unary_operation.argument(),
						std::rc::Rc::clone(&parameter_z_prime), context,
						variable_declaration_stack)?;

					let and = crate::Formula::and(vec![equals, choose_z_prime_in_t_prime]);

					let parameters = std::rc::Rc::new(vec![parameter_z_prime]);

					Ok(crate::Formula::exists(parameters, Box::new(and)))
				},
				_ => Err(crate::Error::new_not_yet_implemented("todo")),
			}
		},
		clingo::ast::TermType::Interval(interval) =>
		{
			let parameters = (0..3).map(
				|_| std::rc::Rc::new(crate::VariableDeclaration::new_generated(
					crate::Domain::Integer)))
				.collect::<crate::VariableDeclarations>();
			let parameters = std::rc::Rc::new(parameters);

			let parameter_i = &parameters[0];
			let parameter_j = &parameters[1];
			let parameter_k = &parameters[2];

			let choose_i_in_t_1 = choose_value_in_term(interval.left(),
				std::rc::Rc::clone(parameter_i), context, variable_declaration_stack)?;

			let choose_j_in_t_2 = choose_value_in_term(interval.right(),
				std::rc::Rc::clone(parameter_j), context, variable_declaration_stack)?;

			let i_less_than_or_equal_to_k = crate::Formula::less_or_equal(
				Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_i))),
				Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_k))));

			let k_less_than_or_equal_to_j = crate::Formula::less_or_equal(
				Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_k))),
				Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_j))));

			let z_equals_k = crate::Formula::equal(
				Box::new(crate::Term::variable(variable_declaration)),
				Box::new(crate::Term::variable(std::rc::Rc::clone(parameter_k))));

			let and = crate::Formula::and(vec![choose_i_in_t_1, choose_j_in_t_2,
				i_less_than_or_equal_to_k, k_less_than_or_equal_to_j, z_equals_k]);

			Ok(crate::Formula::exists(parameters, Box::new(and)))
		},
		clingo::ast::TermType::Function(_) =>
			Err(crate::Error::new_unsupported_language_feature("symbolic functions")),
		clingo::ast::TermType::Pool(_) =>
			Err(crate::Error::new_unsupported_language_feature("pools")),
		clingo::ast::TermType::ExternalFunction(_) =>
			Err(crate::Error::new_unsupported_language_feature("external functions")),
	}
}

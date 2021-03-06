use super::*;

#[test]
fn with_number_atom_reference_function_port_pid_or_tuple_returns_true() {
    TestRunner::new(Config::with_source_file(file!()))
        .run(
            &strategy::process().prop_flat_map(|arc_process| {
                (
                    strategy::term::map(arc_process.clone()),
                    strategy::term::number_atom_reference_function_port_pid_or_tuple(
                        arc_process.clone(),
                    ),
                )
            }),
            |(left, right)| {
                prop_assert_eq!(erlang::is_greater_than_or_equal_2(left, right), true.into());

                Ok(())
            },
        )
        .unwrap();
}

#[test]
fn with_smaller_map_right_returns_true() {
    is_greater_than_or_equal(
        |_, process| {
            process
                .map_from_slice(&[(atom_unchecked("a"), process.integer(1).unwrap())])
                .unwrap()
        },
        true,
    );
}

#[test]
fn with_same_size_map_with_greater_keys_returns_true() {
    is_greater_than_or_equal(
        |_, process| {
            process
                .map_from_slice(&[
                    (atom_unchecked("a"), process.integer(2).unwrap()),
                    (atom_unchecked("b"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        true,
    );
}

#[test]
fn with_same_size_map_with_same_keys_with_greater_values_returns_true() {
    is_greater_than_or_equal(
        |_, process| {
            process
                .map_from_slice(&[
                    (atom_unchecked("b"), process.integer(2).unwrap()),
                    (atom_unchecked("c"), process.integer(2).unwrap()),
                ])
                .unwrap()
        },
        true,
    );
}

#[test]
fn with_same_value_map_returns_true() {
    is_greater_than_or_equal(
        |_, process| {
            process
                .map_from_slice(&[
                    (atom_unchecked("b"), process.integer(2).unwrap()),
                    (atom_unchecked("c"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        true,
    );
}

#[test]
fn with_same_size_map_with_same_keys_with_greater_values_returns_false() {
    is_greater_than_or_equal(
        |_, process| {
            process
                .map_from_slice(&[
                    (atom_unchecked("b"), process.integer(3).unwrap()),
                    (atom_unchecked("c"), process.integer(4).unwrap()),
                ])
                .unwrap()
        },
        false,
    );
}

#[test]
fn with_same_size_map_with_greater_keys_returns_false() {
    is_greater_than_or_equal(
        |_, process| {
            process
                .map_from_slice(&[
                    (atom_unchecked("c"), process.integer(2).unwrap()),
                    (atom_unchecked("d"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        false,
    );
}

#[test]
fn with_greater_size_map_returns_false() {
    is_greater_than_or_equal(
        |_, process| {
            process
                .map_from_slice(&[
                    (atom_unchecked("a"), process.integer(1).unwrap()),
                    (atom_unchecked("b"), process.integer(2).unwrap()),
                    (atom_unchecked("c"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        false,
    );
}

#[test]
fn with_list_or_bitstring_returns_false() {
    TestRunner::new(Config::with_source_file(file!()))
        .run(
            &strategy::process().prop_flat_map(|arc_process| {
                (
                    strategy::term::map(arc_process.clone()),
                    strategy::term::list_or_bitstring(arc_process.clone()),
                )
            }),
            |(left, right)| {
                prop_assert_eq!(
                    erlang::is_greater_than_or_equal_2(left, right),
                    false.into()
                );

                Ok(())
            },
        )
        .unwrap();
}

fn is_greater_than_or_equal<R>(right: R, expected: bool)
where
    R: FnOnce(Term, &Process) -> Term,
{
    super::is_greater_than_or_equal(
        |process| {
            process
                .map_from_slice(&[
                    (atom_unchecked("b"), process.integer(2).unwrap()),
                    (atom_unchecked("c"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        right,
        expected,
    );
}

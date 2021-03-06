use super::*;

use proptest::strategy::Strategy;

#[test]
fn without_list_errors_badarg() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(&strategy::term::is_not_list(arc_process.clone()), |list| {
                prop_assert_eq!(erlang::list_to_existing_atom_1(list), Err(badarg!().into()));

                Ok(())
            })
            .unwrap();
    });
}

#[test]
fn with_empty_list() {
    let list = Term::NIL;

    // as `""` can only be entered into the global atom table, can't test with non-existing atom
    let existing_atom = atom_unchecked("");

    assert_eq!(erlang::list_to_existing_atom_1(list), Ok(existing_atom));
}

#[test]
fn with_improper_list_errors_badarg() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &strategy::term::list::improper(arc_process.clone()),
                |list| {
                    prop_assert_eq!(erlang::list_to_existing_atom_1(list), Err(badarg!().into()));

                    Ok(())
                },
            )
            .unwrap();
    });
}

#[test]
fn with_list_without_existing_atom_errors_badarg() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &any::<String>().prop_map(|suffix| {
                    let string = strategy::term::non_existent_atom(&suffix);
                    let codepoint_terms: Vec<Term> = string
                        .chars()
                        .map(|c| arc_process.integer(c).unwrap())
                        .collect();

                    arc_process.list_from_slice(&codepoint_terms).unwrap()
                }),
                |list| {
                    prop_assert_eq!(erlang::list_to_existing_atom_1(list), Err(badarg!().into()));

                    Ok(())
                },
            )
            .unwrap();
    });
}

#[test]
// collisions due to Unicode escapes.  Could be a normalization/canonicalization issue?
#[ignore]
fn with_list_with_existing_atom_returns_atom() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &any::<String>().prop_map(|string| {
                    let codepoint_terms: Vec<Term> = string
                        .chars()
                        .map(|c| arc_process.integer(c).unwrap())
                        .collect();

                    (
                        arc_process.list_from_slice(&codepoint_terms).unwrap(),
                        atom_unchecked(&string),
                    )
                }),
                |(list, atom)| {
                    prop_assert_eq!(erlang::list_to_existing_atom_1(list), Ok(atom));

                    Ok(())
                },
            )
            .unwrap();
    });
}

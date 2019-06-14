use super::*;

mod with_arity_2;

#[test]
fn without_arity_2_errors_badarg() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &(
                    strategy::term::tuple(arc_process.clone())
                        .prop_filter("Tuple must not be arity 2", |start_length| {
                            start_length.len() != 2
                        }),
                    strategy::term(arc_process.clone()),
                    valid_options(arc_process.clone()),
                ),
                |(destination, message, options)| {
                    prop_assert_eq!(
                        erlang::send_3(destination, message, options, &arc_process),
                        Err(badarg!())
                    );

                    Ok(())
                },
            )
            .unwrap();
    });
}

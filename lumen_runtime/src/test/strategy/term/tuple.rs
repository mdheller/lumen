use std::convert::TryInto;
use std::sync::Arc;

use num_bigint::BigInt;

use proptest::collection::SizeRange;
use proptest::strategy::{BoxedStrategy, Just, Strategy};

use liblumen_alloc::erts::process::alloc::heap_alloc::HeapAlloc;
use liblumen_alloc::erts::term::{Boxed, Term, Tuple};
use liblumen_alloc::erts::Process;

pub fn intermediate(
    element: BoxedStrategy<Term>,
    size_range: SizeRange,
    arc_process: Arc<Process>,
) -> BoxedStrategy<Term> {
    proptest::collection::vec(element, size_range)
        .prop_map(move |vec| arc_process.tuple_from_slice(&vec).unwrap())
        .boxed()
}

pub fn with_index(arc_process: Arc<Process>) -> BoxedStrategy<(Vec<Term>, usize, Term, Term)> {
    (Just(arc_process), 1_usize..=4_usize)
        .prop_flat_map(|(arc_process, len)| {
            (
                Just(arc_process.clone()),
                proptest::collection::vec(super::super::term(arc_process), len..=len),
                0..len,
            )
        })
        .prop_map(|(arc_process, element_vec, zero_based_index)| {
            let mut heap = arc_process.acquire_heap();

            (
                element_vec.clone(),
                zero_based_index,
                heap.tuple_from_slice(&element_vec).unwrap(),
                heap.integer(zero_based_index + 1).unwrap(),
            )
        })
        .boxed()
}

pub fn without_index(arc_process: Arc<Process>) -> BoxedStrategy<(Term, Term)> {
    (super::tuple(arc_process.clone()), super::super::term(arc_process.clone()))
        .prop_filter("Index either needs to not be an integer or not be an integer in the index range 1..=len", |(tuple, index)| {
            let index_big_int_result: std::result::Result<BigInt, _> = (*index).try_into();

            match index_big_int_result {
                Ok(index_big_int) => {
                    let tuple_tuple: Boxed<Tuple> = (*tuple).try_into().unwrap();
                    let min_index: BigInt = 1.into();
                    let max_index: BigInt = tuple_tuple.len().into();

                    !((min_index <= index_big_int) && (index_big_int <= max_index))
                }
                _ => true,
            }
        }).boxed()
}

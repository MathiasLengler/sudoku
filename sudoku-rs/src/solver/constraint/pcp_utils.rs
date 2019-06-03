use gcollections::VectorStack;
use pcp::propagation::CStoreFD;
use pcp::search::branching::{BinarySplit, Brancher, FirstSmallestVar, MiddleVal};
use pcp::search::engine::one_solution::OneSolution;
use pcp::search::propagation::Propagation;
use pcp::search::*;
use pcp::variable::VStoreFD;

pub type VStore = VStoreFD;
type CStore = CStoreFD<VStore>;
pub type FDSpace = Space<VStore, CStore, NoRecomputation<VStore, CStore>>;

pub fn one_solution_engine_interval() -> impl SearchTreeVisitor<FDSpace> {
    concrete_one_solution_engine_interval()
}

pub fn boxed_one_solution_engine_interval() -> Box<dyn SearchTreeVisitor<FDSpace>> {
    Box::new(concrete_one_solution_engine_interval())
}

fn concrete_one_solution_engine_interval() -> pcp::search::engine::one_solution::OneSolution<
    pcp::search::propagation::Propagation<
        pcp::search::branching::brancher::Brancher<
            pcp::search::branching::first_smallest_var::FirstSmallestVar,
            pcp::search::branching::middle_val::MiddleVal,
            pcp::search::branching::binary_split::BinarySplit,
        >,
    >,
    gcollections::stack::Stack<
        gcollections::wrappers::vector::Vector<
            pcp::search::branching::branch::Branch<
                pcp::search::space::Space<
                    pcp::variable::store::Store<
                        pcp::variable::memory::trail_memory::TrailMemory<
                            pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                                interval::interval::Interval<i32>,
                            >,
                            interval::interval::Interval<i32>,
                        >,
                        pcp::propagation::events::FDEvent,
                    >,
                    pcp::propagation::store::Store<
                        pcp::variable::store::Store<
                            pcp::variable::memory::trail_memory::TrailMemory<
                                pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                                    interval::interval::Interval<i32>,
                                >,
                                interval::interval::Interval<i32>,
                            >,
                            pcp::propagation::events::FDEvent,
                        >,
                        pcp::propagation::events::FDEvent,
                        pcp::propagation::reactors::indexed_deps::IndexedDeps,
                        pcp::propagation::schedulers::relaxed_fifo::RelaxedFifo,
                    >,
                    pcp::search::recomputation::no_recomputation::NoRecomputation<
                        pcp::variable::store::Store<
                            pcp::variable::memory::trail_memory::TrailMemory<
                                pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                                    interval::interval::Interval<i32>,
                                >,
                                interval::interval::Interval<i32>,
                            >,
                            pcp::propagation::events::FDEvent,
                        >,
                        pcp::propagation::store::Store<
                            pcp::variable::store::Store<
                                pcp::variable::memory::trail_memory::TrailMemory<
                                    pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                                        interval::interval::Interval<i32>,
                                    >,
                                    interval::interval::Interval<i32>,
                                >,
                                pcp::propagation::events::FDEvent,
                            >,
                            pcp::propagation::events::FDEvent,
                            pcp::propagation::reactors::indexed_deps::IndexedDeps,
                            pcp::propagation::schedulers::relaxed_fifo::RelaxedFifo,
                        >,
                    >,
                >,
            >,
        >,
        gcollections::ops::sequence::ordering::Back,
    >,
    pcp::search::space::Space<
        pcp::variable::store::Store<
            pcp::variable::memory::trail_memory::TrailMemory<
                pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                    interval::interval::Interval<i32>,
                >,
                interval::interval::Interval<i32>,
            >,
            pcp::propagation::events::FDEvent,
        >,
        pcp::propagation::store::Store<
            pcp::variable::store::Store<
                pcp::variable::memory::trail_memory::TrailMemory<
                    pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                        interval::interval::Interval<i32>,
                    >,
                    interval::interval::Interval<i32>,
                >,
                pcp::propagation::events::FDEvent,
            >,
            pcp::propagation::events::FDEvent,
            pcp::propagation::reactors::indexed_deps::IndexedDeps,
            pcp::propagation::schedulers::relaxed_fifo::RelaxedFifo,
        >,
        pcp::search::recomputation::no_recomputation::NoRecomputation<
            pcp::variable::store::Store<
                pcp::variable::memory::trail_memory::TrailMemory<
                    pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                        interval::interval::Interval<i32>,
                    >,
                    interval::interval::Interval<i32>,
                >,
                pcp::propagation::events::FDEvent,
            >,
            pcp::propagation::store::Store<
                pcp::variable::store::Store<
                    pcp::variable::memory::trail_memory::TrailMemory<
                        pcp::variable::memory::trail::timestamp_trail::TimestampTrail<
                            interval::interval::Interval<i32>,
                        >,
                        interval::interval::Interval<i32>,
                    >,
                    pcp::propagation::events::FDEvent,
                >,
                pcp::propagation::events::FDEvent,
                pcp::propagation::reactors::indexed_deps::IndexedDeps,
                pcp::propagation::schedulers::relaxed_fifo::RelaxedFifo,
            >,
        >,
    >,
> {
    OneSolution::<_, VectorStack<_>, FDSpace>::new(Propagation::new(Brancher::new(
        FirstSmallestVar,
        MiddleVal,
        BinarySplit,
    )))
}

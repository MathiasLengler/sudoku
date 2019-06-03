use gcollections::VectorStack;
use pcp::propagation::CStoreFD;
use pcp::search::branching::{BinarySplit, Brancher, FirstSmallestVar, MiddleVal};
use pcp::search::engine::one_solution::OneSolution;
use pcp::search::propagation::Propagation;
use pcp::search::*;
use pcp::variable::VStoreFD;

pub type VStore = VStoreFD;
pub type CStore = CStoreFD<VStore>;
pub type FDSpace = Space<VStore, CStore, NoRecomputation<VStore, CStore>>;

pub fn one_solution_engine_interval() -> impl SearchTreeVisitor<FDSpace> {
    OneSolution::<_, VectorStack<_>, FDSpace>::new(Propagation::new(Brancher::new(
        FirstSmallestVar,
        MiddleVal,
        BinarySplit,
    )))
}

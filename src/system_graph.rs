use smallvec::SmallVec;

use shred::Resources;
use shred::RunNow;

use crate::stage::Stage;

/// The system graph struct, allowing
/// systems to be executed in parallel.
pub struct SystemGraph<'systems> {
    stages: Vec<Stage<'systems>>,
    #[cfg(feature = "parallel")]
    thread_pool: ::std::sync::Arc<::rayon::ThreadPool>,
}

impl<'systems> SystemGraph<'systems> {
    /// Sets up all the systems which means they are gonna add default values for the resources
    /// they need.
    pub fn setup(&mut self, res: &mut Resources) {
        for stage in &mut self.stages {
            stage.setup(res);
        }
    }

    /// Dispatch all the systems with given resources and context
    /// and then run thread local systems.
    ///
    /// This function automatically redirects to
    ///
    /// * [`dispatch_seq`] in case the `parallel` feature is disabled
    ///
    /// Otherwise, it simply runs in parallel.
    ///
    /// Please note that this method assumes that no resource
    /// is currently borrowed. Otherwise, it panics.
    ///
    /// [`dispatch_par`]: struct.Dispatcher.html#method.dispatch_par
    /// [`dispatch_seq`]: struct.Dispatcher.html#method.dispatch_seq
    pub fn dispatch(&mut self, res: &Resources) {
        #[cfg(feature = "parallel")]
        self.dispatch_par(res);

        #[cfg(not(feature = "parallel"))]
        self.dispatch_seq(res);
    }

    #[cfg(feature = "parallel")]
    fn dispatch_par(&mut self, res: &Resources) {
        let stages = &mut self.stages;

        self.thread_pool.install(move || {
            for stage in stages {
                stage.execute(res);
            }
        });
    }

    /// Dispatches the systems (except thread local systems) sequentially.
    ///
    /// This is useful if parallel overhead is
    /// too big or the platform does not support multi-threading.
    ///
    /// Please note that this method assumes that no resource
    /// is currently borrowed. If that's the case, it panics.
    pub fn dispatch_seq(&mut self, res: &Resources) {
        for stage in &mut self.stages {
            stage.execute_seq(res);
        }
    }

    /// This method returns the largest amount of threads this dispatcher
    /// can make use of. This is mainly for debugging purposes so you can see
    /// how well your systems can make use of multi-threading.
    #[cfg(feature = "parallel")]
    pub fn max_threads(&self) -> usize {
        self.stages
            .iter()
            .map(|s| s.max_threads())
            .fold(0, |highest, value| highest.max(value))
    }
}

impl<'a, 'systems> RunNow<'a> for SystemGraph<'systems> {
    fn run_now(&mut self, res: &Resources) {
        self.dispatch(res);
    }

    fn setup(&mut self, res: &mut Resources) {
        self.setup(res);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SystemId(pub usize);

pub type SystemExecSend<'b> = Box<for<'a> RunNow<'a> + Send + 'b>;
pub type ThreadLocal<'a> = SmallVec<[Box<for<'b> RunNow<'b> + 'a>; 4]>;

#[cfg(feature = "parallel")]
pub fn new_dispatcher(
    stages: Vec<Stage>,
    thread_pool: ::std::sync::Arc<::rayon::ThreadPool>,
) -> SystemGraph {
    SystemGraph {
        stages,
        thread_pool,
    }
}

#[cfg(not(feature = "parallel"))]
pub fn new_dispatcher(stages: Vec<Stage>) -> SystemGraph {
    SystemGraph { stages }
}

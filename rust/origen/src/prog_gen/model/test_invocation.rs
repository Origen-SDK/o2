/// Represents an individual test being called/invoked by a test program flow, it associates
/// a test with additional parameterization that is relevant to a particular invocation of it
/// within a flow.
/// The invocation parameters can be very simple, e.g. only really to capture things like the
/// test name column in a Teradyne platform, or more elaborate as in an Advantest example where
/// it is used to model test suites.
#[derive(Debug)]
pub struct TestInvocation {
    /// The ID of the Test to be invoked, i.e. the test instance/test method
    pub test_id: usize,
    /// The parameters associated with this particular call from the flow, i.e. the test suite
    pub test_inv_id: usize,
}

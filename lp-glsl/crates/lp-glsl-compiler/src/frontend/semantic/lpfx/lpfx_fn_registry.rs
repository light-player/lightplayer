//! LPFX Function Registry


/// Registry for Lpfx functions, contains util code for looking them up, resolving signatures, etc.
struct LpfxFnRegistry {}
impl LpfxFnRegistry {
    pub fn get_fn(&self, id: LpfxFnId) -> Option<&LpfxFn> {
        self.fns.get(&id)
    }
}
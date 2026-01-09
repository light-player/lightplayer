pub struct ProjectHandle(pub i32);
impl ProjectHandle {
    pub const NONE: Self = Self(-1);

    pub fn new(id: i32) -> Self {
        Self(id)
    }
}

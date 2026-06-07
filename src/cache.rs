// TODO: Phase 3 — SQLite metadata cache via rusqlite.

#[allow(dead_code)]
pub struct Cache;

#[allow(dead_code)]
impl Cache {
    pub fn open() -> anyhow::Result<Self> {
        Ok(Cache)
    }
}

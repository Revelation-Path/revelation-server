use revelation_bible::{BibleRepository, BibleSearch, ReadingPlan};
use revelation_songbook::SongRepository;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool:  PgPool,
    pub bible: BibleService,
    pub songs: SongRepository
}

#[derive(Clone)]
pub struct BibleService {
    pool: PgPool
}

impl BibleService {
    pub fn repository(&self) -> BibleRepository {
        BibleRepository::new(self.pool.clone())
    }

    pub fn search(&self) -> BibleSearch {
        BibleSearch::new(self.pool.clone())
    }

    pub fn reading_plan(&self) -> ReadingPlan {
        ReadingPlan::new(self.pool.clone())
    }
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            bible: BibleService {
                pool: pool.clone()
            },
            songs: SongRepository::new(pool.clone()),
            pool
        }
    }
}

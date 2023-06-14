use crate::models::competitions_list::CompetitionsListElement;

pub trait CompetitionID {
    fn competition_id(&self) -> u32;
}

impl CompetitionID for u32 {
    fn competition_id(&self) -> u32 {
        *self
    }
}

impl CompetitionID for CompetitionsListElement {
    fn competition_id(&self) -> u32 {
        self.id
    }
}
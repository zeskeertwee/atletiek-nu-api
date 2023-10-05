use tokio;
use crate::{
    get_competition_registrations,
    get_athlete_event_result
};
use crate::models::athlete_event_result::EventResultItem;

#[tokio::test]
async fn test_get_participant_list_39657() {
    let participants = get_competition_registrations(&39657)
        .await
        .unwrap();

    assert_eq!(participants.len(), 220);
}

#[tokio::test]
async fn test_get_participant_list_38681() {
    let participants = get_competition_registrations(&38681)
        .await
        .unwrap();

    assert_eq!(participants.len(), 111);
}

#[tokio::test]
async fn test_get_results_combined_event_with_dnf_1793090() {
    let results = get_athlete_event_result(1793090)
        .await
        .unwrap();

    assert_eq!(results.get_total_points(), Some(1436));
    assert_eq!(results.results.len(), 5);

    for i in results.results {
        if i.event_name == "800m" {
            // check if DNF case is being handled well
            assert!(i.items.contains(&EventResultItem::Points { amount: 0 }));
            assert_eq!(i.items.len(), 1);
        }
    }
}
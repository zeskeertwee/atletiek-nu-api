use std::collections::HashMap;
use tokio;
use regex::Regex;
use crate::{
    get_competition_registrations,
    get_competition_registrations_web,
    get_athlete_event_result
};
use crate::models::athlete_event_result::{DnfReason, EventResultItem};
use crate::models::registrations_list_web::EventStatus;

#[tokio::test]
async fn test_get_participant_list_39657() {
            let participants = get_competition_registrations_web(&39657)
        .await
        .unwrap();

    assert_eq!(participants.len(), 220);
}

#[tokio::test]
async fn test_get_participant_list_38681() {
    let participants = get_competition_registrations_web(&38681)
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
            assert_eq!(i.items.len(), 2);
            assert!(i.items.contains(&EventResultItem::Measurement {
                wind_speed: None,
                result: 9999998.0,
                dnf: true,
                dnf_reason: Some(DnfReason::DataAboveThreshold {
                    threshold: 10000.0
                })
            }));
        }
    }
}

#[tokio::test]
async fn test_get_results_dnf_1734217() {
    let results = get_athlete_event_result(1734217)
        .await
        .unwrap();

    assert_eq!(results.results.len(), 3); // LJ, SP, DT
    let mut expected_dnf_counts: HashMap<String, usize> = HashMap::new();
    expected_dnf_counts.insert("Ver".to_string(), 2);
    expected_dnf_counts.insert("Kogel".to_string(), 0);
    expected_dnf_counts.insert("Discus".to_string(), 4);

    for result in results.results {
        let dnf_count = result.items.iter().filter(|v| match v {
            EventResultItem::Measurement { dnf, dnf_reason, .. } => if *dnf {
                assert_eq!(*dnf_reason.as_ref().unwrap(), DnfReason::DataBelowZero); // in this case, they are all -2
                true
            } else { false },
            _ => false,
        }).count();

        let expected_count = expected_dnf_counts.get(&result.event_name).unwrap();
        assert_eq!(dnf_count, *expected_count);
    }
}

#[tokio::test]
async fn test_multiple_event_registrations_40258() {
    let registrations = get_competition_registrations_web(&40258).await.unwrap();

    let re = Regex::new("\\+[0-9] onderdelen").unwrap();

    for i in registrations {
        for event in i.events {
            if re.is_match(&event.0) {
                panic!("Test failed on participant {}, event '{:?}'", i.participant_id, event);
            }
        }
    }
}

#[tokio::test]
async fn test_event_status_38436() {
    let registrations = get_competition_registrations_web(&38436).await.unwrap();

    for i in registrations {
        for (_, status) in &i.events {
            // we shouldn't see any other status than these
            assert!(status == &EventStatus::CheckedIn || status == &EventStatus::Cancelled || status == &EventStatus::Rejected);
        }

        match i.bib_number {
            Some(44) => {
                assert!(i.events.contains(&("400m".to_string(), EventStatus::CheckedIn)));
                assert!(i.events.contains(&("400m_f".to_string(), EventStatus::CheckedIn)));
                assert_eq!(i.events.len(), 2);
            },
            Some(45) => {
                assert!(i.events.contains(&("60m".to_string(), EventStatus::Rejected)));
                assert!(i.events.contains(&("60mH".to_string(), EventStatus::CheckedIn)));
                assert!(i.events.contains(&("SP".to_string(), EventStatus::CheckedIn)));
            },
            _ => (),
        }
    }
}

#[tokio::test]
async fn test_get_participant_list_38679() {
    let registrations =  get_competition_registrations_web(&38679).await.unwrap();

    for i in registrations {
        if i.name.is_empty() {
            panic!("Empty name for participant id {}!", i.participant_id);
        }
    }
}
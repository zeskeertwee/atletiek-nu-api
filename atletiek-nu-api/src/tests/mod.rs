use std::collections::HashMap;
use std::ops::Add;
use std::time::Duration;
use chrono::NaiveDate;
use tokio;
use regex::Regex;
use tokio::time::Instant;
use crate::{get_competition_registrations_web, get_athlete_event_result, get_athlete_profile};
use crate::models::athlete_event_result::{DnfReason, EventResultItem};
use crate::models::athlete_profile::EventAttribute;
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

#[tokio::test]
async fn test_relay_teams_38406() {
    let registrations = get_competition_registrations_web(&38406).await.unwrap();

    for i in registrations {
        match i.bib_number {
            Some(276) | Some(336) => assert_eq!(i.relay_teams.len(), 1),
            Some(353) => assert_eq!(i.relay_teams.len(), 4),
            _ => (),
        }
    }
}

#[tokio::test]
async fn test_profile_921275() {
    let profile = get_athlete_profile(921275).await.unwrap();

    assert_eq!(profile.name, "Marith Siekman");

    for i in profile.personal_bests {
        match i.event.as_str() {
            "60 meters" => {
                assert_eq!(i.performance, 9.62);
                assert_eq!(i.hand_measured, false);
                assert_eq!(i.date, NaiveDate::from_ymd_opt(2016, 06, 09).unwrap());
                assert_eq!(i.country, "NLD");
                assert_eq!(i.location, "Venlo");
                assert_eq!(i.attribute, None);
            },
            "Shot put" => {
                assert_eq!(i.performance, 5.98);
                assert_eq!(i.hand_measured, false);
                assert_eq!(i.date, NaiveDate::from_ymd_opt(2016, 06, 09).unwrap());
                assert_eq!(i.country, "NLD");
                assert_eq!(i.location, "Venlo");
                assert_eq!(i.attribute, Some(EventAttribute::Weight(2.0)));
            },
            "Long jump" => {
                if i.wind_speed.is_some() {
                    assert_eq!(i.performance, 3.36);
                    assert_eq!(i.hand_measured, false);
                    assert_eq!(i.date, NaiveDate::from_ymd_opt(2016, 06, 25).unwrap());
                    assert_eq!(i.country, "NLD");
                    assert_eq!(i.location, "Weert");
                    assert_eq!(i.attribute, None);
                } else {
                    assert_eq!(i.performance, 3.44);
                    assert_eq!(i.hand_measured, false);
                    assert_eq!(i.date, NaiveDate::from_ymd_opt(2016, 06, 09).unwrap());
                    assert_eq!(i.country, "NLD");
                    assert_eq!(i.location, "Venlo");
                    assert_eq!(i.attribute, None);
                }
            },
            _ => (),
        }
    }

    for graph in profile.graphs {
        match graph.event.as_str() {
            "Long jump" => {
                assert_eq!(graph.points.len(), 2);
                assert_eq!(graph.specification, EventAttribute::All);
                assert!(graph.points.contains(&(NaiveDate::from_ymd_opt(2016, 6, 9).unwrap(), 3.44)));
                assert!(graph.points.contains(&(NaiveDate::from_ymd_opt(2016, 6, 25).unwrap(), 3.36)))
            },
            _ => (),
        }
    }
}

#[tokio::test]
async fn test_profile_parsing() {
    let profiles = [862577, 876749, 871514, 862980, 871317];

    for i in profiles {
        let start = Instant::now();
        println!("Parsing {}", i);
        let _ = get_athlete_profile(i).await.unwrap();
        tokio::time::sleep_until(start.add(Duration::from_secs(1))).await;
    }
}

#[tokio::test]
async fn test_multiday_event_parsing_2418938() {
    let _ = get_athlete_event_result(2418938).await.unwrap();
}
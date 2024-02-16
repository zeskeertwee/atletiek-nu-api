use atletiek_nu_api::models::{competitions_list_web, athlete_profile};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scraper::Html;

const COMPETITIONS_LIST_WEB_SHORT_HTML: &'static str =
    include_str!("../../test-cli/test-data/feeder-short.html");

const COMPETITIONS_LIST_WEB_LONG_HTML: &'static str =
    include_str!("../../test-cli/test-data/feeder-long.html");

const PROFILE_862577_HTML: &'static str =
    include_str!("../../test-cli/test-data/profile_862577.html");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("competitions_list_web (3 items)", |b| {
        b.iter(|| {
            competitions_list_web::parse(Html::parse_document(COMPETITIONS_LIST_WEB_SHORT_HTML))
        })
    });

    c.bench_function("competitions_list_web (alot of items)", |b| {
        b.iter(|| {
            competitions_list_web::parse(Html::parse_document(COMPETITIONS_LIST_WEB_LONG_HTML))
        })
    });

    c.bench_function("athlete_profile 862577", |b| {
        b.iter(|| {
            athlete_profile::parse(Html::parse_document(PROFILE_862577_HTML))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

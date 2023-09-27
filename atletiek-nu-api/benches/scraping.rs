use atletiek_nu_api::models::competitions_list_web;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scraper::Html;

const COMPETITIONS_LIST_WEB_SHORT_HTML: &'static str =
    include_str!("../../test-cli/test-data/feeder-short.html");

const COMPETITIONS_LIST_WEB_LONG_HTML: &'static str =
    include_str!("../../test-cli/test-data/feeder-long.html");

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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

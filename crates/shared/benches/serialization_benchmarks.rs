use criterion::{Criterion, black_box, criterion_group, criterion_main};
use shared::export::{export_to_compressed_bson, export_to_csv, export_to_json};
use shared::models::{AppState, Item, ItemCollection, Priority};

fn bench_serialization(c: &mut Criterion) {
    let mut state = AppState::default();
    let mut collection = ItemCollection::new();
    for i in 0..100 {
        collection.add(Item::new(
            format!("item-{}", i),
            format!("Item Title {}", i),
            format!("Detailed description for item number {}", i),
            Priority::Medium,
        ));
    }
    state.collection = collection;

    c.bench_function("export_to_json_100_items", |b| {
        b.iter(|| {
            let res = export_to_json(black_box(&state));
            let _ = black_box(res);
        })
    });

    c.bench_function("export_to_csv_100_items", |b| {
        b.iter(|| {
            let res = export_to_csv(black_box(&state.collection));
            let _ = black_box(res);
        })
    });

    c.bench_function("export_to_compressed_bson_100_items", |b| {
        b.iter(|| {
            let res = export_to_compressed_bson(black_box(&state));
            let _ = black_box(res);
        })
    });
}

criterion_group!(benches, bench_serialization);
criterion_main!(benches);

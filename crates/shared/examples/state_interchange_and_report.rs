//! Demonstrates UndoStack, JSON/CSV/BSON interchange, shareable URL hash, SVG chart, and HTML5 report generation.

use shared::{
    AppState, Item, ItemCollection, ParametricPreviewConfig, Priority, UndoStack,
    decode_shareable_url_hash, encode_shareable_url_hash, export_to_compressed_bson, export_to_csv,
    export_to_json, export_to_printable_html, export_to_svg, import_from_compressed_bson,
    import_from_csv, import_from_json,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = AppState::new();
    let mut undo_stack: UndoStack<ItemCollection> = UndoStack::new(32);

    undo_stack.push(state.collection.clone());
    state.collection.add(Item::new(
        "example-5",
        "Publish Static Site & Sitemap",
        "Verify SVG chart and printable HTML5 generation",
        Priority::High,
    ));
    assert_eq!(state.collection.total_count(), 5);

    if let Some(prev) = undo_stack.undo(state.collection.clone()) {
        state.collection = prev;
        assert_eq!(state.collection.total_count(), 4);
    }
    if let Some(next) = undo_stack.redo(state.collection.clone()) {
        state.collection = next;
        assert_eq!(state.collection.total_count(), 5);
    }

    let json = export_to_json(&state)?;
    let from_json = import_from_json(&json)?;
    assert_eq!(state, from_json);

    let csv = export_to_csv(&state.collection)?;
    let from_csv = import_from_csv(&csv)?;
    assert_eq!(state.collection.total_count(), from_csv.total_count());

    let bson = export_to_compressed_bson(&state)?;
    let from_bson = import_from_compressed_bson(&bson)?;
    assert_eq!(state, from_bson);

    let url_hash = encode_shareable_url_hash(&state)?;
    let from_hash = decode_shareable_url_hash(&url_hash)?;
    assert_eq!(state, from_hash);

    let svg = export_to_svg(&state);
    let html = export_to_printable_html(&state);
    let polyline = ParametricPreviewConfig::default().svg_polyline_points(600.0, 220.0, 64);

    println!(
        "Verified interchange: {} items | share hash len={} | svg={}B | html={}B | curve pts len={}",
        state.collection.total_count(),
        url_hash.len(),
        svg.len(),
        html.len(),
        polyline.len()
    );
    Ok(())
}

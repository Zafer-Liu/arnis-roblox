#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (f64, f64, u32)| {
    let (lat, lon, zoom_raw) = data;

    // Clamp to valid ranges
    let zoom = zoom_raw % 23; // zoom 0-22
    let lat = if lat.is_finite() {
        lat.clamp(-85.05112878, 85.05112878)
    } else {
        return;
    };
    let lon = if lon.is_finite() { lon.clamp(-180.0, 180.0) } else { return };

    // Forward: lat/lon -> tile
    let (x, y) = arbx_geo::tiles::lat_lon_to_tile(lat, lon, zoom);

    // Tile coordinates must be within valid range
    let max_tile = 1u32 << zoom;
    assert!(x < max_tile, "x={x} >= max_tile={max_tile} for zoom={zoom}");
    assert!(y < max_tile, "y={y} >= max_tile={max_tile} for zoom={zoom}");

    // Roundtrip: tile -> lat/lon should be close to original
    let (lat2, lon2) = arbx_geo::tiles::tile_to_lat_lon(x, y, zoom);
    assert!(lat2.is_finite(), "roundtrip lat not finite");
    assert!(lon2.is_finite(), "roundtrip lon not finite");

    // The roundtrip should land in the same or adjacent tile (±1 is acceptable
    // due to floating-point precision at tile boundaries)
    let (x2, y2) = arbx_geo::tiles::lat_lon_to_tile(lat2, lon2, zoom);
    assert!(x.abs_diff(x2) <= 1, "roundtrip x off by >1: ({lat},{lon}) z={zoom} -> ({x},{y}) -> ({lat2},{lon2}) -> ({x2},{y2})");
    assert!(y.abs_diff(y2) <= 1, "roundtrip y off by >1");

    // meters_per_pixel should be positive and finite
    let mpp = arbx_geo::tiles::meters_per_pixel(lat, zoom);
    assert!(mpp > 0.0 && mpp.is_finite(), "mpp={mpp} for lat={lat} zoom={zoom}");

    // bbox coverage should be non-empty for any valid bbox
    if lat.abs() < 85.0 && lon.abs() < 179.5 {
        let tiles = arbx_geo::tiles::tiles_for_bbox(
            lat - 0.001,
            lon - 0.001,
            lat + 0.001,
            lon + 0.001,
            zoom.min(18),
        );
        assert!(!tiles.is_empty(), "tiles_for_bbox empty for valid bbox");
    }
});

//! ESRI satellite tile coordinate math, URL builder, and disk-cached fetcher.
//!
//! Uses standard Web Mercator / Slippy Map conventions:
//!   x = floor((lon + 180) / 360 * 2^zoom)
//!   y = floor((1 - ln(tan(lat_rad) + 1/cos(lat_rad)) / pi) / 2 * 2^zoom)
//!
//! Tiles are fetched from ESRI World Imagery (free, no API key) and cached
//! locally at `{cache_dir}/{z}/{x}/{y}.jpg`.

use std::path::{Path, PathBuf};

/// Convert lat/lon to slippy map tile coordinates at a given zoom level.
pub fn lat_lon_to_tile(lat: f64, lon: f64, zoom: u32) -> (u32, u32) {
    let n = 2_u64.pow(zoom) as f64;
    let x = ((lon + 180.0) / 360.0 * n) as u32;
    let lat_rad = lat.to_radians();
    let y = ((1.0
        - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)
        / 2.0
        * n) as u32;
    (x, y)
}

/// Convert tile coordinates back to lat/lon (northwest corner of the tile).
pub fn tile_to_lat_lon(x: u32, y: u32, zoom: u32) -> (f64, f64) {
    let n = 2_u64.pow(zoom) as f64;
    let lon = x as f64 / n * 360.0 - 180.0;
    let lat_rad = (std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n)).sinh().atan();
    (lat_rad.to_degrees(), lon)
}

/// Get all tiles that cover a bounding box at a given zoom level.
///
/// Returns `(x, y)` pairs for every tile whose area intersects the bbox.
/// The bbox corners are `(min_lat, min_lon)` (southwest) and
/// `(max_lat, max_lon)` (northeast).
pub fn tiles_for_bbox(
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    zoom: u32,
) -> Vec<(u32, u32)> {
    // NW corner has max_lat, min_lon; SE corner has min_lat, max_lon.
    // In slippy map coords, higher lat => lower y.
    let (x_min, y_min) = lat_lon_to_tile(max_lat, min_lon, zoom);
    let (x_max, y_max) = lat_lon_to_tile(min_lat, max_lon, zoom);

    let mut tiles = Vec::new();
    for x in x_min..=x_max {
        for y in y_min..=y_max {
            tiles.push((x, y));
        }
    }
    tiles
}

/// Compute the ground resolution in meters per pixel at a given latitude and
/// zoom level.  Standard 256-pixel tiles assumed.
pub fn meters_per_pixel(lat: f64, zoom: u32) -> f64 {
    let lat_rad = lat.to_radians();
    156543.03392 * lat_rad.cos() / (2_u64.pow(zoom) as f64)
}

/// Build the ESRI World Imagery tile URL for the given tile coordinates.
///
/// ESRI uses `z/y/x` ordering (not `z/x/y`).
pub fn esri_tile_url(x: u32, y: u32, zoom: u32) -> String {
    format!(
        "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
        zoom, y, x
    )
}

/// Return the on-disk cache path for a tile: `{cache_dir}/{z}/{x}/{y}.jpg`.
pub fn tile_cache_path(cache_dir: &Path, x: u32, y: u32, zoom: u32) -> PathBuf {
    cache_dir.join(format!("{}/{}/{}.jpg", zoom, x, y))
}

/// Fetch a single satellite tile, returning raw JPEG bytes.
///
/// If the tile is already cached on disk, the cached copy is returned
/// immediately. Otherwise the tile is downloaded from ESRI World Imagery,
/// written to the cache directory, and then returned.
///
/// A 200 ms politeness delay is inserted before each network request to
/// respect ESRI's free-tier rate expectations. The request identifies itself
/// with a descriptive User-Agent string.
pub fn fetch_tile(cache_dir: &Path, x: u32, y: u32, zoom: u32) -> Result<Vec<u8>, String> {
    let cached = tile_cache_path(cache_dir, x, y, zoom);

    // Fast path: return from disk cache.
    if cached.exists() {
        return std::fs::read(&cached).map_err(|e| format!("cache read failed: {e}"));
    }

    // Politeness delay before hitting the remote server.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let url = esri_tile_url(x, y, zoom);

    let resp = ureq::agent()
        .get(&url)
        .header(
            "User-Agent",
            "arnis-roblox/1.0 (open-source educational project)",
        )
        .call()
        .map_err(|e| format!("HTTP fetch failed for {url}: {e}"))?;

    let status = resp.status();
    if status != 200 {
        return Err(format!("HTTP {status} for {url}"));
    }

    let bytes = resp
        .into_body()
        .read_to_vec()
        .map_err(|e| format!("body read failed: {e}"))?;

    // Ensure parent directories exist, then write cache file.
    if let Some(parent) = cached.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create_dir_all failed: {e}"))?;
    }
    std::fs::write(&cached, &bytes).map_err(|e| format!("cache write failed: {e}"))?;

    Ok(bytes)
}

/// Composite satellite tiles into a single 512x512 RGB image covering the
/// given bounding box.
///
/// For each output pixel the function maps back to lat/lon, identifies the
/// source tile pixel, and copies the colour across.  Missing or corrupt tiles
/// are silently skipped (the output defaults to black).
pub fn composite_chunk_texture(
    cache_dir: &Path,
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    zoom: u32,
    output_size: u32,
) -> Result<image::RgbImage, String> {
    use image::{GenericImageView, RgbImage};

    let mut output = RgbImage::new(output_size, output_size);

    let tiles = tiles_for_bbox(min_lat, min_lon, max_lat, max_lon, zoom);
    if tiles.is_empty() {
        return Ok(output);
    }

    // Pre-load all needed tiles into memory.
    let mut tile_images: std::collections::HashMap<(u32, u32), image::DynamicImage> =
        std::collections::HashMap::new();
    for &(tx, ty) in &tiles {
        match fetch_tile(cache_dir, tx, ty, zoom) {
            Ok(bytes) => {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    tile_images.insert((tx, ty), img);
                }
            }
            Err(_) => {
                // Skip unavailable tiles; output stays black for those pixels.
            }
        }
    }

    let n = 2_u64.pow(zoom) as f64;

    for py in 0..output_size {
        // Map output pixel row to latitude (top = max_lat, bottom = min_lat).
        let lat = max_lat - (py as f64 / output_size as f64) * (max_lat - min_lat);
        let lat_rad = lat.to_radians();
        let tile_y_frac =
            (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;

        for px in 0..output_size {
            // Map output pixel column to longitude (left = min_lon, right = max_lon).
            let lon = min_lon + (px as f64 / output_size as f64) * (max_lon - min_lon);
            let tile_x_frac = (lon + 180.0) / 360.0 * n;

            let tx = tile_x_frac as u32;
            let ty = tile_y_frac as u32;

            if let Some(img) = tile_images.get(&(tx, ty)) {
                let (tw, th) = img.dimensions();
                let src_x = ((tile_x_frac - tx as f64) * tw as f64) as u32;
                let src_y = ((tile_y_frac - ty as f64) * th as f64) as u32;
                if src_x < tw && src_y < th {
                    let pixel = img.get_pixel(src_x, src_y);
                    output.put_pixel(px, py, image::Rgb([pixel[0], pixel[1], pixel[2]]));
                }
            }
        }
    }

    Ok(output)
}

/// Build the output path for a chunk's terrain texture PNG.
///
/// Layout: `{out_dir}/{scene}-terrain-tiles/{chunk_id}.png`
pub fn chunk_texture_path(out_dir: &Path, scene: &str, chunk_id: &str) -> PathBuf {
    out_dir
        .join(format!("{}-terrain-tiles", scene))
        .join(format!("{}.png", chunk_id))
}

/// Build the output path for a chunk's raw RGBA terrain texture.
///
/// Layout: `{out_dir}/{scene}-terrain-tiles/{chunk_id}.rgba`
pub fn chunk_texture_rgba_path(out_dir: &Path, scene: &str, chunk_id: &str) -> PathBuf {
    out_dir
        .join(format!("{}-terrain-tiles", scene))
        .join(format!("{}.rgba", chunk_id))
}

/// Convert an RGB image to raw RGBA bytes (alpha = 255) and write to disk.
pub fn save_rgba_raw(img: &image::RgbImage, path: &Path) -> Result<(), String> {
    let (w, h) = img.dimensions();
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for pixel in img.pixels() {
        rgba.push(pixel[0]);
        rgba.push(pixel[1]);
        rgba.push(pixel[2]);
        rgba.push(255);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir failed for {}: {e}", parent.display()))?;
    }
    std::fs::write(path, &rgba)
        .map_err(|e| format!("RGBA write failed for {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lat_lon_to_tile_known_values() {
        // Austin downtown at zoom 17
        let (x, y) = lat_lon_to_tile(30.267, -97.743, 17);
        assert_eq!(x, 29948);
        assert_eq!(y, 53964);
    }

    #[test]
    fn tile_roundtrip() {
        let (x, y) = lat_lon_to_tile(30.267, -97.743, 17);
        let (lat, lon) = tile_to_lat_lon(x, y, 17);
        assert!(
            (lat - 30.267).abs() < 0.01,
            "lat roundtrip delta too large: got {lat}"
        );
        assert!(
            (lon - (-97.743)).abs() < 0.01,
            "lon roundtrip delta too large: got {lon}"
        );
    }

    #[test]
    fn tiles_for_bbox_covers_area() {
        let tiles = tiles_for_bbox(30.26, -97.75, 30.27, -97.74, 17);
        assert!(!tiles.is_empty(), "bbox should produce at least one tile");
        // ~0.01 degree bbox at z17 produces a small grid (around 20 tiles).
        assert!(
            tiles.len() <= 25,
            "small bbox at z17 should not need {n} tiles",
            n = tiles.len()
        );
    }

    #[test]
    fn tiles_for_bbox_contains_expected_tile() {
        // The center of the bbox should map to one of the returned tiles.
        let center_lat = (30.26 + 30.27) / 2.0;
        let center_lon = (-97.75 + -97.74) / 2.0;
        let center_tile = lat_lon_to_tile(center_lat, center_lon, 17);
        let tiles = tiles_for_bbox(30.26, -97.75, 30.27, -97.74, 17);
        assert!(
            tiles.contains(&center_tile),
            "bbox tiles should include the center tile {center_tile:?}"
        );
    }

    #[test]
    fn meters_per_pixel_at_equator_zoom_17() {
        let mpp = meters_per_pixel(0.0, 17);
        assert!(
            (mpp - 1.194).abs() < 0.1,
            "expected ~1.19 m/px at equator z17, got {mpp}"
        );
    }

    #[test]
    fn meters_per_pixel_decreases_at_higher_latitude() {
        let mpp_equator = meters_per_pixel(0.0, 17);
        let mpp_austin = meters_per_pixel(30.267, 17);
        assert!(
            mpp_austin < mpp_equator,
            "higher latitude should have smaller m/px"
        );
    }

    #[test]
    fn esri_tile_url_format() {
        let url = esri_tile_url(29948, 53964, 17);
        assert!(url.contains("/17/53964/29948"), "URL should be z/y/x: {url}");
        assert!(url.starts_with("https://"), "URL should be HTTPS: {url}");
    }

    #[test]
    fn tile_cache_path_layout() {
        let p = tile_cache_path(Path::new("data/esri-tiles"), 29442, 54787, 17);
        assert_eq!(
            p,
            PathBuf::from("data/esri-tiles/17/29442/54787.jpg")
        );
    }

    #[test]
    fn fetch_tile_returns_cached_file() {
        // Write a fake cached file and verify fetch_tile reads it back.
        let tmp = std::env::temp_dir().join("arbx_geo_tile_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let fake_bytes = b"not-a-real-jpeg";
        let path = tile_cache_path(&tmp, 1, 2, 3);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, fake_bytes).unwrap();

        let result = fetch_tile(&tmp, 1, 2, 3).expect("cached fetch should succeed");
        assert_eq!(result, fake_bytes);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn chunk_texture_path_construction() {
        let p = chunk_texture_path(Path::new("out"), "Austin", "2_3");
        assert_eq!(p, PathBuf::from("out/Austin-terrain-tiles/2_3.png"));
    }

    #[test]
    fn chunk_texture_path_nested_out_dir() {
        let p = chunk_texture_path(Path::new("/tmp/export/scenes"), "Downtown", "-1_0");
        assert_eq!(
            p,
            PathBuf::from("/tmp/export/scenes/Downtown-terrain-tiles/-1_0.png")
        );
    }

    #[test]
    fn tiles_for_bbox_deterministic_small() {
        // A tight bbox that should produce exactly 1 tile at zoom 17.
        // Pick a point and compute its tile, then build a bbox inside that tile.
        let (tx, ty) = lat_lon_to_tile(30.267, -97.743, 17);
        let (nw_lat, nw_lon) = tile_to_lat_lon(tx, ty, 17);
        let (se_lat, se_lon) = tile_to_lat_lon(tx + 1, ty + 1, 17);
        // Shrink bbox to be strictly inside the tile.
        let min_lat = se_lat + 0.0001;
        let max_lat = nw_lat - 0.0001;
        let min_lon = nw_lon + 0.0001;
        let max_lon = se_lon - 0.0001;
        let tiles = tiles_for_bbox(min_lat, min_lon, max_lat, max_lon, 17);
        assert_eq!(tiles.len(), 1, "tight bbox should produce exactly 1 tile");
        assert_eq!(tiles[0], (tx, ty));
    }

    #[test]
    fn composite_chunk_texture_returns_correct_size_with_fake_tiles() {
        // Seed the cache with a fake JPEG tile, then verify compositing
        // produces the correct output dimensions.
        let tmp = std::env::temp_dir().join("arbx_geo_composite_fake");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // Pre-populate cache with a real (tiny) 256x256 red PNG encoded as JPEG
        // so `image::load_from_memory` will parse it.
        let red_img = image::RgbImage::from_fn(256, 256, |_, _| image::Rgb([255, 0, 0]));
        let tiles = tiles_for_bbox(30.26, -97.75, 30.27, -97.74, 17);
        for &(tx, ty) in &tiles {
            let path = tile_cache_path(&tmp, tx, ty, 17);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            // Save as JPEG bytes.
            let mut buf = std::io::Cursor::new(Vec::new());
            red_img
                .write_to(&mut buf, image::ImageFormat::Jpeg)
                .unwrap();
            std::fs::write(&path, buf.into_inner()).unwrap();
        }

        let img = composite_chunk_texture(&tmp, 30.26, -97.75, 30.27, -97.74, 17, 512)
            .expect("compositing with fake cached tiles should succeed");
        assert_eq!(img.width(), 512);
        assert_eq!(img.height(), 512);
        // Every pixel should be reddish (JPEG compression loses some precision).
        let sample = img.get_pixel(256, 256);
        assert!(
            sample[0] > 200,
            "red channel should be high, got {}",
            sample[0]
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn chunk_texture_rgba_path_construction() {
        let p = chunk_texture_rgba_path(Path::new("out"), "Austin", "2_3");
        assert_eq!(p, PathBuf::from("out/Austin-terrain-tiles/2_3.rgba"));
    }

    #[test]
    fn save_rgba_raw_writes_correct_bytes() {
        let tmp = std::env::temp_dir().join("arbx_geo_rgba_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // Create a 2x2 RGB image: red, green, blue, white
        let mut img = image::RgbImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        img.put_pixel(1, 0, image::Rgb([0, 255, 0]));
        img.put_pixel(0, 1, image::Rgb([0, 0, 255]));
        img.put_pixel(1, 1, image::Rgb([255, 255, 255]));

        let out_path = tmp.join("test.rgba");
        save_rgba_raw(&img, &out_path).expect("save should succeed");

        let bytes = std::fs::read(&out_path).unwrap();
        // 2x2 * 4 = 16 bytes
        assert_eq!(bytes.len(), 16);
        // Red pixel: R=255, G=0, B=0, A=255
        assert_eq!(&bytes[0..4], &[255, 0, 0, 255]);
        // Green pixel
        assert_eq!(&bytes[4..8], &[0, 255, 0, 255]);
        // Blue pixel
        assert_eq!(&bytes[8..12], &[0, 0, 255, 255]);
        // White pixel
        assert_eq!(&bytes[12..16], &[255, 255, 255, 255]);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

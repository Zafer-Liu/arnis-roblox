use arbx_pipeline::BuildingFeature;

/// A resolved building part with inherited + overridden tags.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingPart {
    pub id: String,
    pub footprint: Vec<(f64, f64)>,
    pub holes: Vec<Vec<(f64, f64)>>,
    pub base_y: f64,
    pub height: f64,
    pub min_height: Option<f64>,
    pub levels: Option<u32>,
    pub roof_shape: String,
    pub roof_height: Option<f64>,
    pub roof_direction: Option<f64>,
    pub roof_angle: Option<f64>,
    pub material_tag: Option<String>,
    pub colour: Option<String>,
    pub roof_material: Option<String>,
    pub roof_colour: Option<String>,
    pub usage: Option<String>,
    pub name: Option<String>,
}

/// Given a parent building and its building:part children, resolve tag
/// inheritance and return a list of BuildingParts ready for mesh generation.
///
/// Each part inherits ALL tags from the parent building UNLESS the part has
/// its own value for that tag.  Parts define their own `min_height` and
/// `height` for vertical stacking.
///
/// If no parts exist, the parent itself becomes a single BuildingPart.
///
/// The parent outline should NOT render where parts cover it (handled by
/// the caller: if parts exist, skip parent mesh, generate per-part meshes).
pub fn resolve_building_parts(
    parent: &BuildingFeature,
    parts: &[BuildingFeature],
) -> Vec<BuildingPart> {
    if parts.is_empty() {
        return vec![feature_to_part(parent, None)];
    }

    parts
        .iter()
        .map(|part| feature_to_part(part, Some(parent)))
        .collect()
}

/// Convert a `BuildingFeature` into a `BuildingPart`, optionally inheriting
/// missing tags from a parent building.
fn feature_to_part(feature: &BuildingFeature, parent: Option<&BuildingFeature>) -> BuildingPart {
    let footprint: Vec<(f64, f64)> = feature
        .footprint
        .points
        .iter()
        .map(|p| (p.x, p.y))
        .collect();
    let holes: Vec<Vec<(f64, f64)>> = feature
        .holes
        .iter()
        .map(|h| h.points.iter().map(|p| (p.x, p.y)).collect())
        .collect();

    BuildingPart {
        id: feature.id.clone(),
        footprint,
        holes,
        base_y: feature.base_y,
        height: feature.height,
        min_height: feature.min_height,
        levels: feature.levels.or_else(|| parent.and_then(|p| p.levels)),
        roof_shape: if feature.roof.is_empty() {
            parent
                .map(|p| p.roof.clone())
                .unwrap_or_else(|| "flat".to_string())
        } else {
            feature.roof.clone()
        },
        roof_height: feature
            .roof_height
            .or_else(|| parent.and_then(|p| p.roof_height)),
        roof_direction: feature
            .roof_direction
            .or_else(|| parent.and_then(|p| p.roof_direction)),
        roof_angle: feature
            .roof_angle
            .or_else(|| parent.and_then(|p| p.roof_angle)),
        material_tag: feature
            .material_tag
            .clone()
            .or_else(|| parent.and_then(|p| p.material_tag.clone())),
        colour: feature
            .colour
            .clone()
            .or_else(|| parent.and_then(|p| p.colour.clone())),
        roof_material: feature
            .roof_material
            .clone()
            .or_else(|| parent.and_then(|p| p.roof_material.clone())),
        roof_colour: feature
            .roof_colour
            .clone()
            .or_else(|| parent.and_then(|p| p.roof_colour.clone())),
        usage: feature
            .usage
            .clone()
            .or_else(|| parent.and_then(|p| p.usage.clone())),
        name: feature
            .name
            .clone()
            .or_else(|| parent.and_then(|p| p.name.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbx_geo::Vec2;
    use arbx_geo::Footprint;

    fn square_footprint() -> Footprint {
        Footprint::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ])
    }

    fn make_parent() -> BuildingFeature {
        BuildingFeature {
            id: "parent_1".to_string(),
            footprint: square_footprint(),
            holes: vec![],
            indices: None,
            base_y: 0.0,
            height: 30.0,
            height_m: Some(30.0),
            levels: Some(10),
            roof_levels: None,
            min_height: Some(0.0),
            roof: "gabled".to_string(),
            usage: Some("commercial".to_string()),
            colour: Some("beige".to_string()),
            material_tag: Some("concrete".to_string()),
            roof_colour: Some("grey".to_string()),
            roof_material: Some("metal".to_string()),
            roof_height: Some(3.0),
            roof_direction: Some(90.0),
            roof_angle: Some(30.0),
            name: Some("Test Tower".to_string()),
            facade_style: None,
            structure_type: None,
        }
    }

    #[test]
    fn parent_with_no_parts_yields_single_building_part() {
        let parent = make_parent();
        let parts = resolve_building_parts(&parent, &[]);

        assert_eq!(parts.len(), 1);
        let p = &parts[0];
        assert_eq!(p.id, "parent_1");
        assert_eq!(p.height, 30.0);
        assert_eq!(p.base_y, 0.0);
        assert_eq!(p.levels, Some(10));
        assert_eq!(p.roof_shape, "gabled");
        assert_eq!(p.colour.as_deref(), Some("beige"));
        assert_eq!(p.material_tag.as_deref(), Some("concrete"));
        assert_eq!(p.name.as_deref(), Some("Test Tower"));
    }

    #[test]
    fn two_parts_base_and_tower_with_correct_heights() {
        let parent = make_parent();

        let base = BuildingFeature {
            id: "part_base".to_string(),
            footprint: square_footprint(),
            holes: vec![],
            indices: None,
            base_y: 0.0,
            height: 10.0,
            height_m: Some(10.0),
            levels: Some(3),
            roof_levels: None,
            min_height: Some(0.0),
            roof: "flat".to_string(),
            usage: Some("retail".to_string()),
            colour: None,
            material_tag: None,
            roof_colour: None,
            roof_material: None,
            roof_height: None,
            roof_direction: None,
            roof_angle: None,
            name: None,
            facade_style: None,
            structure_type: None,
        };

        let tower = BuildingFeature {
            id: "part_tower".to_string(),
            footprint: Footprint::new(vec![
                Vec2::new(2.0, 2.0),
                Vec2::new(8.0, 2.0),
                Vec2::new(8.0, 8.0),
                Vec2::new(2.0, 8.0),
            ]),
            holes: vec![],
            indices: None,
            base_y: 10.0,
            height: 20.0,
            height_m: Some(30.0),
            levels: Some(7),
            roof_levels: None,
            min_height: Some(10.0),
            roof: "flat".to_string(),
            usage: Some("office".to_string()),
            colour: None,
            material_tag: None,
            roof_colour: None,
            roof_material: None,
            roof_height: None,
            roof_direction: None,
            roof_angle: None,
            name: None,
            facade_style: None,
            structure_type: None,
        };

        let parts = resolve_building_parts(&parent, &[base, tower]);

        assert_eq!(parts.len(), 2);

        // Base part: 0-10m
        assert_eq!(parts[0].id, "part_base");
        assert_eq!(parts[0].base_y, 0.0);
        assert_eq!(parts[0].height, 10.0);
        assert_eq!(parts[0].levels, Some(3));
        assert_eq!(parts[0].usage.as_deref(), Some("retail"));

        // Tower part: 10-30m
        assert_eq!(parts[1].id, "part_tower");
        assert_eq!(parts[1].base_y, 10.0);
        assert_eq!(parts[1].height, 20.0);
        assert_eq!(parts[1].min_height, Some(10.0));
        assert_eq!(parts[1].levels, Some(7));
        assert_eq!(parts[1].usage.as_deref(), Some("office"));
    }

    #[test]
    fn tag_inheritance_part_without_roof_shape_inherits_parent() {
        let parent = make_parent(); // roof = "gabled"

        let part = BuildingFeature {
            id: "part_inherit".to_string(),
            footprint: square_footprint(),
            holes: vec![],
            indices: None,
            base_y: 0.0,
            height: 15.0,
            height_m: Some(15.0),
            levels: None,
            roof_levels: None,
            min_height: Some(0.0),
            // Empty roof → should inherit from parent
            roof: String::new(),
            usage: None,
            colour: None,
            material_tag: None,
            roof_colour: None,
            roof_material: None,
            roof_height: None,
            roof_direction: None,
            roof_angle: None,
            name: None,
            facade_style: None,
            structure_type: None,
        };

        let parts = resolve_building_parts(&parent, &[part]);
        assert_eq!(parts.len(), 1);
        // Should inherit parent's "gabled" roof
        assert_eq!(parts[0].roof_shape, "gabled");
        // Should also inherit other tags from parent
        assert_eq!(parts[0].colour.as_deref(), Some("beige"));
        assert_eq!(parts[0].material_tag.as_deref(), Some("concrete"));
        assert_eq!(parts[0].roof_material.as_deref(), Some("metal"));
        assert_eq!(parts[0].roof_colour.as_deref(), Some("grey"));
        assert_eq!(parts[0].roof_height, Some(3.0));
        assert_eq!(parts[0].roof_direction, Some(90.0));
        assert_eq!(parts[0].name.as_deref(), Some("Test Tower"));
        assert_eq!(parts[0].levels, Some(10)); // inherited from parent
    }

    #[test]
    fn tag_override_part_with_own_roof_shape_overrides_parent() {
        let parent = make_parent(); // roof = "gabled"

        let part = BuildingFeature {
            id: "part_override".to_string(),
            footprint: square_footprint(),
            holes: vec![],
            indices: None,
            base_y: 0.0,
            height: 20.0,
            height_m: Some(20.0),
            levels: Some(6),
            roof_levels: None,
            min_height: Some(0.0),
            roof: "hipped".to_string(),
            usage: Some("residential".to_string()),
            colour: Some("white".to_string()),
            material_tag: Some("brick".to_string()),
            roof_colour: Some("red".to_string()),
            roof_material: Some("tile".to_string()),
            roof_height: Some(5.0),
            roof_direction: Some(180.0),
            roof_angle: Some(45.0),
            name: Some("Part Name".to_string()),
            facade_style: None,
            structure_type: None,
        };

        let parts = resolve_building_parts(&parent, &[part]);
        assert_eq!(parts.len(), 1);
        let p = &parts[0];
        // All should be the part's own values, not the parent's
        assert_eq!(p.roof_shape, "hipped");
        assert_eq!(p.levels, Some(6));
        assert_eq!(p.colour.as_deref(), Some("white"));
        assert_eq!(p.material_tag.as_deref(), Some("brick"));
        assert_eq!(p.roof_colour.as_deref(), Some("red"));
        assert_eq!(p.roof_material.as_deref(), Some("tile"));
        assert_eq!(p.roof_height, Some(5.0));
        assert_eq!(p.roof_direction, Some(180.0));
        assert_eq!(p.roof_angle, Some(45.0));
        assert_eq!(p.usage.as_deref(), Some("residential"));
        assert_eq!(p.name.as_deref(), Some("Part Name"));
    }
}

use super::*;

#[test]
fn thai_distribution_is_font_independent_and_identical_through_both_app_routes() {
    let ctx = egui::Context::default();
    ctx.set_fonts(egui::FontDefinitions::empty());
    for disabled in [false, true] {
        let mut outputs = Vec::new();
        for document in [true, false] {
            ctx.begin_pass(egui::RawInput::default());
            let p = ctx.layer_painter(egui::LayerId::background());
            let r = Rect::from_min_size(Pos2::ZERO, vec2(20.0, 20.0));
            if document {
                assert!(paint_named(&p, r, "thaiDistribute", Color32::GRAY, false, disabled));
            } else {
                deck::paint(&p, r, deck::Icon::ThaiDistribute, Color32::GRAY, disabled);
            }
            let mut output = ctx.end_pass();
            output.textures_delta.clear();
            let meshes: Vec<_> = ctx
                .tessellate(output.shapes, output.pixels_per_point)
                .into_iter()
                .filter_map(|primitive| match primitive.primitive {
                    egui::epaint::Primitive::Mesh(mesh) => Some((mesh.vertices, mesh.indices)),
                    _ => None,
                })
                .collect();
            assert!(!meshes.is_empty());
            outputs.push(meshes);
        }
        assert_eq!(outputs.first(), outputs.get(1));
    }
}

#[test]
fn balanced_roles_preserve_caller_alpha_and_drawing_opacity() {
    assert_eq!(gold(Color32::WHITE, GoldRole::Primary, 1.0).a(), 230);
    assert_eq!(gold(Color32::WHITE, GoldRole::Surface, 1.0).a(), 107);
    for alpha in [0, 32, 64, 128, 255] {
        let ink = Color32::from_white_alpha(alpha);
        for role in [GoldRole::Primary, GoldRole::Surface] {
            let full = gold(ink, role, 1.0);
            let half = gold(ink, role, 0.5);
            assert!(full.a() <= alpha);
            assert!(half.a() <= full.a());
            assert!(half.a().abs_diff((f32::from(full.a()) * 0.5).round() as u8) <= 1);
            assert_eq!(gold(ink, role, 0.0).a(), 0);
            assert_eq!(gold(ink, role, -1.0).a(), 0);
            assert_eq!(gold(ink, role, f32::NAN).a(), 0);
            assert_eq!(gold(ink, role, 2.0), full);
        }
    }
}

#[test]
fn catalog_preserves_transparency_and_monochrome_states() {
    let ctx = egui::Context::default();
    for name in catalog() {
        for alpha in [64, 255] {
            for (flat, disabled) in [(false, false), (true, false), (false, true), (true, true)] {
                ctx.begin_pass(egui::RawInput::default());
                let p = ctx.layer_painter(egui::LayerId::background());
                let ink = Color32::from_white_alpha(alpha);
                assert!(paint_named(&p, Rect::from_min_size(Pos2::ZERO, vec2(24.0, 24.0)), &name, ink, flat, disabled));
                let mut output = ctx.end_pass();
                output.textures_delta.clear();
                let ceiling = if disabled { ink.gamma_multiply(0.35).a() } else { alpha };
                for primitive in ctx.tessellate(output.shapes, output.pixels_per_point) {
                    if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                        for vertex in mesh.vertices {
                            assert!(vertex.color.a() <= ceiling, "opacity increased: {name}, flat={flat}, disabled={disabled}");
                            if flat || disabled {
                                let [r, g, b, _] = vertex.color.to_array();
                                assert_eq!([r, g], [g, b], "gold leaked into monochrome state: {name}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn complete_catalog_resolves_and_renders_at_working_sizes() {
    let ctx = egui::Context::default();
    let names = catalog();
    assert!(names.len() > 400);
    for size in [16.0, 20.0, 24.0, 32.0] {
        for (flat, disabled) in [(false, false), (true, false), (false, true)] {
            ctx.begin_pass(egui::RawInput::default());
            let painter = ctx.layer_painter(egui::LayerId::background());
            for name in &names {
                assert!(
                    paint_named(&painter, Rect::from_min_size(Pos2::ZERO, vec2(size, size)), name, Color32::GRAY, flat, disabled),
                    "missing {name}"
                );
            }
            let mut output = ctx.end_pass();
            output.textures_delta.clear();
            let shapes = ctx.tessellate(output.shapes, output.pixels_per_point);
            assert!(!shapes.is_empty());
            for shape in shapes {
                if let egui::epaint::Primitive::Mesh(mesh) = shape.primitive {
                    assert!(mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()));
                }
            }
        }
    }
}

#[test]
fn clipboard_actions_use_identical_geometry_across_app_spellings() {
    for (left, right) in [("painter", "format-painter"), ("strike", "strikethrough"), ("fontcolor", "font-color"), ("alignLeft", "align-left")] {
        assert_eq!(canonical_name(left), canonical_name(right));
    }
}

#[test]
fn app_adapters_produce_identical_clipboard_and_formatting_meshes() {
    for (name, sheet_icon, deck_icon) in [
        ("paste", sheet::Icon::Paste, deck::Icon::Paste),
        ("cut", sheet::Icon::Cut, deck::Icon::Cut),
        ("copy", sheet::Icon::Copy, deck::Icon::Copy),
        ("painter", sheet::Icon::Brush, deck::Icon::FormatPainter),
        ("bold", sheet::Icon::Bold, deck::Icon::Bold),
        ("italic", sheet::Icon::Italic, deck::Icon::Italic),
        ("underline", sheet::Icon::Underline, deck::Icon::Underline),
    ] {
        let ctx = egui::Context::default();
        let mut outputs = Vec::new();
        for app in 0..3 {
            ctx.begin_pass(egui::RawInput::default());
            let p = ctx.layer_painter(egui::LayerId::background());
            let r = Rect::from_min_size(Pos2::ZERO, vec2(24.0, 24.0));
            match app {
                0 => {
                    paint_named(&p, r, name, Color32::GRAY, false, false);
                }
                1 => sheet::paint(&p, r, sheet_icon, Color32::GRAY),
                _ => deck::paint(&p, r, deck_icon, Color32::GRAY, false),
            }
            let mut output = ctx.end_pass();
            output.textures_delta.clear();
            let meshes: Vec<_> = ctx
                .tessellate(output.shapes, output.pixels_per_point)
                .into_iter()
                .filter_map(|shape| {
                    if let egui::epaint::Primitive::Mesh(mesh) = shape.primitive {
                        Some((
                            mesh.vertices.iter().map(|v| ([v.pos.x, v.pos.y], [v.uv.x, v.uv.y], v.color.to_array())).collect::<Vec<_>>(),
                            mesh.indices,
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            outputs.push(meshes);
        }
        assert_eq!(outputs.first(), outputs.get(1), "sheet differs: {name}");
        assert_eq!(outputs.first(), outputs.get(2), "deck differs: {name}");
    }
}

#[test]
fn arbitrary_names_and_hostile_rectangles_do_not_create_fallback_letter_tiles() {
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput::default());
    let p = ctx.layer_painter(egui::LayerId::background());
    assert!(!paint_named(&p, Rect::from_min_size(Pos2::ZERO, vec2(24.0, 24.0)), "not-an-icon", Color32::WHITE, false, false));
    for size in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        assert!(!paint_named(&p, Rect::from_min_size(Pos2::ZERO, vec2(size, size)), "paste", Color32::WHITE, false, false));
    }
    let mut output = ctx.end_pass();
    output.textures_delta.clear();
    assert!(output.shapes.is_empty());
}

#[test]
fn every_app_icon_routed_to_the_line_set_is_drawn_there() {
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput::default());
    let p = ctx.layer_painter(egui::LayerId::background());
    let r = Rect::from_min_size(Pos2::ZERO, vec2(20.0, 20.0));
    let mut routed = 0;
    let deck_names = deck::Icon::ALL.iter().map(|i| i.name());
    let sheet_names = sheet::Icon::ALL.iter().map(|i| i.name());
    for (app, name, line) in deck_names
        .filter_map(|n| line::deck::line_name(n).map(|l| ("deck", n, l)))
        .chain(sheet_names.filter_map(|n| line::sheet::line_name(n).map(|l| ("sheet", n, l))))
    {
        assert!(line::paint(&p, r, line, Color32::GRAY, false), "{app} `{name}` is routed to `{line}`, which the line set does not draw");
        routed += 1;
    }
    assert!(routed > 100, "only {routed} icons routed");
    // PhutKrub: every deck icon has a line drawing (routed or its own in `line/deck.rs`), save
    // Thai Distribution, which is the shared Thai mark in every app.
    for &icon in deck::Icon::ALL {
        if icon != deck::Icon::ThaiDistribute {
            for flat in [false, true] {
                assert!(line::deck::paint(&p, r, icon.name(), Color32::GRAY, false, flat), "deck `{}` has no line drawing", icon.name());
            }
            assert!(line::deck::paint(&p, Rect::NOTHING, icon.name(), Color32::GRAY, true, false), "deck `{}` unknown at an empty size", icon.name());
        }
    }
    // KhitKrub: every sheet icon has a line drawing, routed or its own (`line/sheet.rs`).
    for icon in sheet::Icon::ALL {
        assert!(line::sheet::paint(&p, r, *icon, Color32::GRAY, false), "sheet `{}` has no line drawing", icon.name());
        assert!(line::sheet::paint(&p, r, *icon, Color32::GRAY, true), "sheet `{}` has no flat line drawing", icon.name());
    }
    for name in line::sheet::OWN {
        let icon = sheet::Icon::ALL.iter().find(|i| i.name() == *name);
        assert!(icon.is_some_and(|i| line::sheet::line_name(i.name()).is_none()), "`{name}` is not a sheet icon drawn in line/sheet.rs");
    }
    ctx.end_pass().textures_delta.clear();
}

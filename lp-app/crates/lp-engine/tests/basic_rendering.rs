use lp_engine::ProjectRuntime;
use lp_shared::fs::LpFsMemory;

#[test]
fn test_basic_rendering() {
    // Create a minimal project with fixture
    let mut fs = LpFsMemory::new();

    // Create project.json
    let project_json = r#"{
        "uid": "test-project",
        "name": "Test Project"
    }"#;
    fs.write_file_mut("/project.json", project_json.as_bytes())
        .unwrap();

    // Create a texture node (new config format)
    let texture_json = r#"{
        "width": 100,
        "height": 100
    }"#;
    fs.write_file_mut("/src/test.texture/node.json", texture_json.as_bytes())
        .unwrap();

    // Create an output node
    let output_json = r#"{
        "GpioStrip": {
            "pin": 18
        }
    }"#;
    fs.write_file_mut("/src/test.output/node.json", output_json.as_bytes())
        .unwrap();

    // Create a fixture node (new config format with color_order)
    let fixture_json = r#"{
        "output_spec": "/src/test.output",
        "texture_spec": "/src/test.texture",
        "mapping": "linear",
        "lamp_type": "rgb",
        "color_order": "Rgb",
        "transform": [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    }"#;
    fs.write_file_mut("/src/test.fixture/node.json", fixture_json.as_bytes())
        .unwrap();

    // Create project runtime
    let mut runtime = ProjectRuntime::new(Box::new(fs)).unwrap();

    // Load and initialize nodes
    runtime.load_nodes().unwrap();
    runtime.initialize_nodes().unwrap();

    // Get initial frame_id
    let initial_frame = runtime.frame_id;

    // Tick to advance frame (16ms delta for ~60fps)
    runtime.tick(16);
    assert_eq!(runtime.frame_id.as_i64(), initial_frame.as_i64() + 1);
    assert_eq!(runtime.frame_time.delta_ms, 16);
    assert_eq!(runtime.frame_time.total_ms, 16);

    // Render frame
    runtime.render().unwrap();

    // Verify fixture status (should be Ok or Error, not Created)
    let fixture_entry = runtime
        .nodes
        .values()
        .find(|e| e.kind == lp_model::NodeKind::Fixture)
        .unwrap();
    assert!(!matches!(
        fixture_entry.status,
        lp_engine::project::NodeStatus::Created
    ));

    // Verify texture was initialized (check via get_changes to get actual state)
    let texture_entry = runtime
        .nodes
        .values()
        .find(|e| e.kind == lp_model::NodeKind::Texture)
        .unwrap();

    // Get texture state via sync API
    let response = runtime
        .get_changes(
            lp_model::FrameId::default(),
            &lp_model::project::api::ApiNodeSpecifier::All,
        )
        .unwrap();

    match response {
        lp_model::project::api::ProjectResponse::GetChanges { node_details, .. } => {
            // Find texture node detail by path
            let texture_detail = node_details
                .values()
                .find(|d| d.path.as_str() == "/src/test.texture")
                .expect("Texture node should be in details");

            // Verify texture state has data (should be initialized with zeros)
            match &texture_detail.state {
                lp_model::project::api::NodeState::Texture(tex_state) => {
                    // Texture should be 100x100 RGBA8 = 40,000 bytes
                    assert_eq!(tex_state.texture_data.len(), 100 * 100 * 4);
                }
                _ => panic!("Expected texture state"),
            }
        }
    }
}

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
    fs.write_file_mut("/project.json", project_json.as_bytes()).unwrap();
    
    // Create a texture node
    let texture_json = r#"{
        "Memory": {
            "width": 100,
            "height": 100
        }
    }"#;
    fs.write_file_mut("/src/test.texture/node.json", texture_json.as_bytes()).unwrap();
    
    // Create an output node
    let output_json = r#"{
        "GpioStrip": {
            "pin": 18
        }
    }"#;
    fs.write_file_mut("/src/test.output/node.json", output_json.as_bytes()).unwrap();
    
    // Create a fixture node
    let fixture_json = r#"{
        "output_spec": "/src/test.output",
        "texture_spec": "/src/test.texture",
        "mapping": "linear",
        "lamp_type": "rgb",
        "transform": [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    }"#;
    fs.write_file_mut("/src/test.fixture/node.json", fixture_json.as_bytes()).unwrap();
    
    // Create project runtime
    let mut runtime = ProjectRuntime::new(Box::new(fs)).unwrap();
    
    // Load and initialize nodes
    runtime.load_nodes().unwrap();
    runtime.initialize_nodes().unwrap();
    
    // Get initial frame_id
    let initial_frame = runtime.frame_id;
    
    // Tick to advance frame
    runtime.tick();
    assert_eq!(runtime.frame_id.as_i64(), initial_frame.as_i64() + 1);
    
    // Render frame
    runtime.render().unwrap();
    
    // Verify fixture status (should be Ok or Error, not Created)
    let fixture_entry = runtime.nodes.values()
        .find(|e| e.kind == lp_model::NodeKind::Fixture)
        .unwrap();
    assert!(!matches!(fixture_entry.status, lp_engine::project::NodeStatus::Created));
}

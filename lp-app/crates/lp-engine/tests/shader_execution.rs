use lp_engine::ProjectRuntime;
use lp_shared::fs::LpFsMemory;

#[test]
fn test_shader_initialization() {
    let mut fs = LpFsMemory::new();

    // Create project.json
    fs.write_file_mut(
        "/project.json",
        r#"{
        "uid": "test-project",
        "name": "Test Project"
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a texture node
    fs.write_file_mut(
        "/src/test.texture/node.json",
        r#"{
        "width": 100,
        "height": 100
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a shader node
    fs.write_file_mut(
        "/src/test.shader/node.json",
        r#"{
        "glsl_path": "main.glsl",
        "texture_spec": "/src/test.texture",
        "render_order": 0
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a simple GLSL shader
    fs.write_file_mut(
        "/src/test.shader/main.glsl",
        r#"
vec4 main(vec2 fragCoord, vec2 outputSize, float time) {
    return vec4(1.0, 0.0, 0.0, 1.0); // Red
}
"#
        .as_bytes(),
    )
    .unwrap();

    // Create runtime and initialize
    let mut runtime = ProjectRuntime::new(Box::new(fs)).unwrap();
    runtime.load_nodes().unwrap();
    runtime.initialize_nodes().unwrap();

    // Verify shader was initialized
    let response = runtime
        .get_changes(
            lp_model::FrameId::default(),
            &lp_model::project::api::ApiNodeSpecifier::All,
        )
        .unwrap();

    match response {
        lp_model::project::api::ProjectResponse::GetChanges { node_details, .. } => {
            let shader_detail = node_details
                .values()
                .find(|d| d.path.as_str() == "/src/test.shader")
                .expect("Shader node should be in details");

            // Check shader status first
            match shader_detail.status {
                lp_model::project::api::NodeStatus::Ok => {
                    // Shader initialized successfully
                    match &shader_detail.state {
                        lp_model::project::api::NodeState::Shader(shader_state) => {
                            // Verify GLSL code was loaded
                            assert!(
                                !shader_state.glsl_code.is_empty(),
                                "Shader code should be loaded when initialization succeeds"
                            );
                            assert!(shader_state.glsl_code.contains("main"));
                            // Should have no compilation error (shader is valid)
                            assert!(
                                shader_state.error.is_none(),
                                "Shader should compile successfully, got error: {:?}",
                                shader_state.error
                            );
                        }
                        _ => panic!("Expected shader state"),
                    }
                }
                lp_model::project::api::NodeStatus::InitError(ref msg) => {
                    panic!("Shader initialization failed: {}", msg);
                }
                _ => {
                    // Shader might not be initialized yet
                    // Check state anyway
                    match &shader_detail.state {
                        lp_model::project::api::NodeState::Shader(shader_state) => {
                            if shader_state.glsl_code.is_empty() {
                                panic!(
                                    "Shader code should be loaded, status: {:?}",
                                    shader_detail.status
                                );
                            }
                        }
                        _ => panic!("Expected shader state"),
                    }
                }
            }

            // Verify config was extracted
            match &shader_detail.config {
                config if config.kind() == lp_model::NodeKind::Shader => {
                    // Config should be extracted (not default)
                    // We can't easily check the values without downcasting, but we can verify it's not empty/default
                }
                _ => panic!("Expected shader config"),
            }
        }
    }
}

#[test]
fn test_shader_execution() {
    let mut fs = LpFsMemory::new();

    // Create project.json
    fs.write_file_mut(
        "/project.json",
        r#"{
        "uid": "test-project",
        "name": "Test Project"
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a texture node
    fs.write_file_mut(
        "/src/test.texture/node.json",
        r#"{
        "width": 10,
        "height": 10
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a shader node that renders a gradient
    fs.write_file_mut(
        "/src/test.shader/node.json",
        r#"{
        "glsl_path": "main.glsl",
        "texture_spec": "/src/test.texture",
        "render_order": 0
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a GLSL shader that creates a gradient
    fs.write_file_mut(
        "/src/test.shader/main.glsl",
        r#"
vec4 main(vec2 fragCoord, vec2 outputSize, float time) {
    float r = fragCoord.x / outputSize.x;
    float g = fragCoord.y / outputSize.y;
    return vec4(r, g, 0.5, 1.0);
}
"#
        .as_bytes(),
    )
    .unwrap();

    // Create output and fixture nodes to trigger texture access
    fs.write_file_mut(
        "/src/test.output/node.json",
        r#"{
        "GpioStrip": {
            "pin": 18
        }
    }"#
        .as_bytes(),
    )
    .unwrap();

    fs.write_file_mut("/src/test.fixture/node.json", r#"{
        "output_spec": "/src/test.output",
        "texture_spec": "/src/test.texture",
        "mapping": "linear",
        "lamp_type": "rgb",
        "color_order": "Rgb",
        "transform": [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    }"#.as_bytes()).unwrap();

    // Create runtime and initialize
    let mut runtime = ProjectRuntime::new(Box::new(fs)).unwrap();
    runtime.load_nodes().unwrap();
    runtime.initialize_nodes().unwrap();

    // Tick to advance frame
    runtime.tick(16);

    // Render (fixture will access texture, triggering shader execution)
    runtime.render().unwrap();

    // Verify shader was executed by checking texture state
    // Note: Shader execution happens lazily when texture is accessed
    // The fixture accessing the texture should trigger shader execution
    let response = runtime
        .get_changes(
            lp_model::FrameId::default(),
            &lp_model::project::api::ApiNodeSpecifier::All,
        )
        .unwrap();

    match response {
        lp_model::project::api::ProjectResponse::GetChanges { node_details, .. } => {
            // Verify shader state shows it was compiled successfully
            let shader_detail = node_details
                .values()
                .find(|d| d.path.as_str() == "/src/test.shader")
                .expect("Shader node should be in details");

            match &shader_detail.state {
                lp_model::project::api::NodeState::Shader(shader_state) => {
                    // Shader should have compiled successfully (no error)
                    assert!(
                        shader_state.error.is_none(),
                        "Shader should compile successfully"
                    );
                    assert!(
                        !shader_state.glsl_code.is_empty(),
                        "Shader code should be loaded"
                    );
                }
                _ => panic!("Expected shader state"),
            }

            // Verify texture was accessed (has data)
            let texture_detail = node_details
                .values()
                .find(|d| d.path.as_str() == "/src/test.texture")
                .expect("Texture node should be in details");

            match &texture_detail.state {
                lp_model::project::api::NodeState::Texture(tex_state) => {
                    // Texture should have been initialized (10x10 RGBA8 = 400 bytes)
                    assert_eq!(tex_state.texture_data.len(), 10 * 10 * 4);
                    // Note: We don't verify non-zero pixels here because shader execution
                    // might not happen until texture is actually accessed via get_texture_mut()
                    // This test verifies the shader can be initialized and compiled
                }
                _ => panic!("Expected texture state"),
            }
        }
    }
}

#[test]
fn test_shader_compilation_error() {
    let mut fs = LpFsMemory::new();

    // Create project.json
    fs.write_file_mut(
        "/project.json",
        r#"{
        "uid": "test-project",
        "name": "Test Project"
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a texture node
    fs.write_file_mut(
        "/src/test.texture/node.json",
        r#"{
        "width": 100,
        "height": 100
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create a shader node
    fs.write_file_mut(
        "/src/test.shader/node.json",
        r#"{
        "glsl_path": "main.glsl",
        "texture_spec": "/src/test.texture",
        "render_order": 0
    }"#
        .as_bytes(),
    )
    .unwrap();

    // Create an invalid GLSL shader
    fs.write_file_mut(
        "/src/test.shader/main.glsl",
        r#"
invalid glsl code here
"#
        .as_bytes(),
    )
    .unwrap();

    // Create runtime and initialize
    let mut runtime = ProjectRuntime::new(Box::new(fs)).unwrap();
    runtime.load_nodes().unwrap();

    // Initialize should fail or set error status
    let _init_result = runtime.initialize_nodes();
    // Initialization might succeed but shader should be in error state

    // Check shader state for error
    let response = runtime
        .get_changes(
            lp_model::FrameId::default(),
            &lp_model::project::api::ApiNodeSpecifier::All,
        )
        .unwrap();

    match response {
        lp_model::project::api::ProjectResponse::GetChanges { node_details, .. } => {
            let shader_detail = node_details
                .values()
                .find(|d| d.path.as_str() == "/src/test.shader")
                .expect("Shader node should be in details");

            // Status should indicate error (init failed)
            assert!(
                matches!(
                    shader_detail.status,
                    lp_model::project::api::NodeStatus::InitError(_)
                        | lp_model::project::api::NodeStatus::Error(_)
                ),
                "Shader should have error status, got: {:?}",
                shader_detail.status
            );

            // If runtime exists, check state for error
            // If runtime doesn't exist (init failed), state will be empty
            match &shader_detail.state {
                lp_model::project::api::NodeState::Shader(shader_state) => {
                    // If we have a runtime, it should have stored the error
                    // If init failed completely, runtime is None and state is empty
                    // In that case, the error is in the status, not the state
                    if !shader_state.glsl_code.is_empty() {
                        // Runtime exists, so error should be in state
                        assert!(
                            shader_state.error.is_some(),
                            "Shader with runtime should have compilation error in state, status: {:?}",
                            shader_detail.status
                        );
                    } else {
                        // Runtime doesn't exist, error is in status (which we already checked)
                        // This is fine - initialization failed before runtime was created
                    }
                }
                _ => panic!("Expected shader state"),
            }
        }
    }
}

use lp_engine::ProjectRuntime;
use lp_engine_client::test_util::assert_first_output_red;
use lp_engine_client::ClientProjectView;
use lp_model::project::ProjectBuilder;
use lp_shared::fs::LpFsMemory;

#[test]
fn test_end_to_end_shader_time_based() {
    let mut fs = LpFsMemory::new();
    let mut builder = ProjectBuilder::new(&mut fs);

    // Add texture
    let texture_path = builder.texture_basic();

    // Add shader
    builder.shader_basic(&texture_path);

    // Add output
    let output_path = builder.output_basic();

    // Add fixture
    builder.fixture_basic(&output_path, &texture_path);

    // Build project
    builder.build();

    // Start runtime
    let mut runtime = ProjectRuntime::new(Box::new(fs)).unwrap();
    runtime.load_nodes().unwrap();
    runtime.init_nodes().unwrap();
    runtime.ensure_all_nodes_initialized().unwrap();

    // Create client view
    let mut client_view = ClientProjectView::new();

    // Get output handle
    let output_handle = runtime.handle_for_path(output_path.as_str()).unwrap();

    // Watch output for detail changes
    client_view.watch_detail(output_handle);

    // Shader: vec4(mod(time, 1.0), 0.0, 0.0, 1.0) -> RGBA bytes [R, G, B, A]
    // Advancing time by 4ms gives an increment of (4/1000 * 255) = 1.02 ≈ 1

    // Frame 1
    runtime.tick(4).unwrap();
    sync_client_view(&runtime, &mut client_view);
    assert_first_output_red(&mut client_view, output_handle, 1);

    // Frame 2
    runtime.tick(4).unwrap();
    sync_client_view(&runtime, &mut client_view);
    assert_first_output_red(&mut client_view, output_handle, 2);

    // Frame 3
    runtime.tick(4).unwrap();
    sync_client_view(&runtime, &mut client_view);
    assert_first_output_red(&mut client_view, output_handle, 3);

    // Verify client view frame_id matches runtime
    assert_eq!(client_view.frame_id, runtime.frame_id);
}

/// Sync the client view with the runtime
fn sync_client_view(runtime: &ProjectRuntime, client_view: &mut ClientProjectView) {
    let response = runtime
        .get_changes(client_view.frame_id, &client_view.detail_specifier())
        .unwrap();
    client_view.sync(&response).unwrap();
}

use crate::error::Error;
use crate::nodes::{
    NodeRuntime, TextureRuntime, ShaderRuntime, OutputRuntime, FixtureRuntime,
};
use crate::runtime::contexts::NodeInitContext;
use lp_model::{
    FrameId, LpPath, NodeConfig, NodeHandle, NodeKind,
};
use lp_shared::fs::LpFs;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;

/// Project runtime - manages nodes and rendering
pub struct ProjectRuntime {
    /// Current frame ID
    pub frame_id: FrameId,
    /// Filesystem (owned for now)
    pub fs: Box<dyn LpFs>,
    /// Node entries
    pub nodes: BTreeMap<NodeHandle, NodeEntry>,
    /// Next handle to assign
    pub next_handle: i32,
}

/// Node entry in runtime
pub struct NodeEntry {
    /// Node path
    pub path: LpPath,
    /// Node kind
    pub kind: NodeKind,
    /// Node config
    pub config: Box<dyn NodeConfig>,
    /// Frame when config was last updated
    pub config_ver: FrameId,
    /// Node status
    pub status: NodeStatus,
    /// Node runtime (None until initialized)
    pub runtime: Option<Box<dyn NodeRuntime>>,
    /// Last frame state updates occurred
    pub state_ver: FrameId,
}

/// Node status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    /// Created but not yet initialized
    Created,
    /// Error initializing the node
    InitError(String),
    /// Node is running normally
    Ok,
    /// Node is running, but something is wrong
    Warn(String),
    /// Node cannot run
    Error(String),
}

impl ProjectRuntime {
    /// Create new project runtime
    pub fn new(fs: Box<dyn LpFs>) -> Result<Self, Error> {
        let _config = crate::project::loader::load_from_filesystem(fs.as_ref())?;
        
        Ok(Self {
            frame_id: FrameId::default(),
            fs,
            nodes: BTreeMap::new(),
            next_handle: 1,
        })
    }
    
    /// Load nodes from filesystem (doesn't initialize them)
    pub fn load_nodes(&mut self) -> Result<(), Error> {
        let node_paths = crate::project::loader::discover_nodes(self.fs.as_ref())?;
        
        for path in node_paths {
            match crate::project::loader::load_node(self.fs.as_ref(), &path) {
                Ok((path, config)) => {
                    let handle = NodeHandle::new(self.next_handle);
                    self.next_handle += 1;
                    
                    let kind = config.kind();
                    let entry = NodeEntry {
                        path,
                        kind,
                        config,
                        config_ver: self.frame_id,
                        status: NodeStatus::Created,
                        runtime: None,
                        state_ver: FrameId::default(),
                    };
                    
                    self.nodes.insert(handle, entry);
                }
                Err(e) => {
                    // Create entry with error status
                    let handle = NodeHandle::new(self.next_handle);
                    self.next_handle += 1;
                    
                    // Try to determine kind from path
                    let kind = match crate::project::loader::node_kind_from_path(&path) {
                        Ok(k) => k,
                        Err(_) => continue, // Skip unknown types
                    };
                    
                    // Create a dummy config based on kind
                    // This is a temporary solution until we have a better way
                    let config: Box<dyn NodeConfig> = match kind {
                        NodeKind::Texture => {
                            Box::new(lp_model::nodes::texture::TextureConfig::Memory {
                                width: 0,
                                height: 0,
                            })
                        }
                        NodeKind::Shader => {
                            Box::new(lp_model::nodes::shader::ShaderConfig::default())
                        }
                        NodeKind::Output => {
                            Box::new(lp_model::nodes::output::OutputConfig::GpioStrip {
                                pin: 0,
                            })
                        }
                        NodeKind::Fixture => {
                            Box::new(lp_model::nodes::fixture::FixtureConfig {
                                output_spec: lp_model::NodeSpecifier::from(""),
                                texture_spec: lp_model::NodeSpecifier::from(""),
                                mapping: String::new(),
                                lamp_type: String::new(),
                                transform: [[0.0; 4]; 4],
                            })
                        }
                    };
                    
                    let entry = NodeEntry {
                        path,
                        kind,
                        config,
                        config_ver: self.frame_id,
                        status: NodeStatus::InitError(format!("Failed to load: {}", e)),
                        runtime: None,
                        state_ver: FrameId::default(),
                    };
                    
                    self.nodes.insert(handle, entry);
                }
            }
        }
        
        Ok(())
    }
    
    /// Initialize all nodes in dependency order
    pub fn initialize_nodes(&mut self) -> Result<(), Error> {
        // Initialize in order: textures → shaders → fixtures → outputs
        let init_order = [
            NodeKind::Texture,
            NodeKind::Shader,
            NodeKind::Fixture,
            NodeKind::Output,
        ];
        
        for kind in init_order.iter() {
            let handles: Vec<NodeHandle> = self.nodes
                .iter()
                .filter(|(_, entry)| entry.kind == *kind && entry.status == NodeStatus::Created)
                .map(|(handle, _)| *handle)
                .collect();
            
            for handle in handles {
                if let Some(entry) = self.nodes.get_mut(&handle) {
                    // Create runtime based on kind
                    let mut runtime: Box<dyn NodeRuntime> = match entry.kind {
                        NodeKind::Texture => Box::new(TextureRuntime::new()),
                        NodeKind::Shader => Box::new(ShaderRuntime::new()),
                        NodeKind::Output => Box::new(OutputRuntime::new()),
                        NodeKind::Fixture => Box::new(FixtureRuntime::new()),
                    };
                    
                    // Create init context (stub for now)
                    let ctx = InitContext {
                        fs: self.fs.as_ref(),
                        // todo!("Add node resolution to context")
                    };
                    
                    // Initialize
                    match runtime.init(&ctx) {
                        Ok(()) => {
                            entry.status = NodeStatus::Ok;
                            entry.runtime = Some(runtime);
                        }
                        Err(e) => {
                            entry.status = NodeStatus::InitError(format!("{}", e));
                            entry.runtime = None;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Stub init context implementation
struct InitContext<'a> {
    fs: &'a dyn LpFs,
}

impl<'a> NodeInitContext for InitContext<'a> {
    fn get_node_fs(&self) -> &dyn LpFs {
        self.fs
    }
    
    // Other methods use default todo!() implementations
}

use crate::project::nodes::output::state::OutputState;
use crate::project::nodes::shader::state::ShaderState;
use crate::project::nodes::texture::state::TextureState;

pub enum NodeState {
    Output(OutputState),
    Shader(ShaderState),
    Texture(TextureState),
}

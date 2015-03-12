use gfx::traits::{Device, DeviceExt, FactoryExt, ToSlice};
use gfx;
use draw_state;
use vecmath::Matrix4;

static VERTEX: &'static [u8] = b"
    #version 150 core
    uniform mat4 projection, view;

    in vec2 tex_coord;
    in vec3 color, position;

    out vec2 v_tex_coord;
    out vec3 v_color;

    void main() {
        v_tex_coord = tex_coord;
        v_color = color;
        gl_Position = projection * view * vec4(position, 1.0);
    }
";

static FRAGMENT: &'static [u8] = b"
    #version 150 core
    out vec4 out_color;

    uniform sampler2D s_texture;

    in vec2 v_tex_coord;
    in vec3 v_color;

    void main() {
        vec4 tex_color = texture(s_texture, v_tex_coord);
        if(tex_color.a == 0.0) // Discard transparent pixels.
            discard;
        out_color = tex_color * vec4(v_color, 1.0);
    }
";

/*#[shader_param]
#[derive(Clone)]
struct ShaderParam<R: gfx::Resources> {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub s_texture: gfx::shade::TextureParam<R>,
}

#[vertex_format]
#[derive(Copy)]
pub struct Vertex {
    #[name="position"]
    pub xyz: [f32; 3],
    #[name="tex_coord"]
    pub uv: [f32; 2],
    #[name="color"]
    pub rgb: [f32; 3],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        *self
    }
}*/

pub struct Buffer<R: gfx::Resources> {
    batch: gfx::batch::RefBatch<ShaderParam<R>>,
}

pub struct Renderer<D: Device> {
    graphics: gfx::Graphics<D>,
    params: ShaderParam<D::Resources>,
    frame: gfx::Frame<D::Resources>,
    cd: gfx::ClearData,
    prog: gfx::ProgramHandle<D::Resources>,
    drawstate: gfx::DrawState
}

impl<R: gfx::device::Resources, C: gfx::device::draw::CommandBuffer<R>, D: gfx::device::Factory<R> + gfx::Device<Resources=R, CommandBuffer=C>> Renderer<D> {
    pub fn new(mut device: D, frame: gfx::Frame<D::Resources>,
               tex: gfx::TextureHandle<D::Resources>) -> Renderer<D> {
        let sampler = device.create_sampler(
                gfx::tex::SamplerInfo::new(
                    gfx::tex::FilterMethod::Scale,
                    gfx::tex::WrapMode::Tile
                )
            );

        let mut graphics = device.into_graphics();

        let params = ShaderParam {
            projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            s_texture: (tex, Some(sampler))
        };
        let prog = graphics.device.link_program(VERTEX.clone(), FRAGMENT.clone()).ok().unwrap();
        let mut drawstate = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
        drawstate.primitive.front_face = draw_state::block::FrontFace::Clockwise;

        Renderer {
            graphics: graphics,
            params: params,
            frame: frame,
            cd: gfx::ClearData {
                color: [0.81, 0.8, 1.0, 1.0],
                depth: 1.0,
                stencil: 0,
            },
            prog: prog,
            drawstate: drawstate,
        }
    }

    pub fn set_projection(&mut self, proj_mat: Matrix4<f32>) {
        self.params.projection = proj_mat;
    }

    pub fn set_view(&mut self, view_mat: Matrix4<f32>) {
        self.params.view = view_mat;
    }

    pub fn clear(&mut self) {
        self.graphics.clear(self.cd, gfx::COLOR | gfx::DEPTH, &self.frame);
    }

    pub fn create_buffer(&mut self, data: &[Vertex]) -> Buffer<D::Resources> {
        let buf = self.graphics.device.create_buffer(data.len(), gfx::BufferUsage::Static);
        self.graphics.device.update_buffer(&buf, data, 0);
        let mesh = gfx::Mesh::from_format(buf, data.len() as u32);
        Buffer {
            batch: self.graphics.make_batch(
                    &self.prog,
                    self.params.clone(),
                    &mesh,
                    mesh.to_slice(gfx::PrimitiveType::TriangleList),
                    &self.drawstate
                ).unwrap()
        }
    }

    pub fn render(&mut self, buffer: &mut Buffer<D::Resources>) {
        buffer.batch.params = self.params.clone();
        self.graphics.draw(&buffer.batch, &self.frame).unwrap();
    }

    pub fn end_frame(&mut self) {
        self.graphics.end_frame();
    }
}

#[derive(Clone, Debug)]
    struct ShaderParam<R: gfx::Resources> {
        pub projection: [[f32; 4]; 4],
        pub view: [[f32; 4]; 4],
        pub s_texture: gfx::shade::TextureParam<R>,
    }
#[derive(Clone, Debug)]
    struct _ShaderParamLink {
        pub projection: Option<gfx::shade::VarUniform>,
        pub view: Option<gfx::shade::VarUniform>,
        pub s_texture: Option<gfx::shade::VarTexture>,
    }
    impl ::std::marker::Copy for _ShaderParamLink { }
    impl <R: gfx::Resources> gfx::shade::ShaderParam
     for ShaderParam<R> {type
        Resources
        =
        R;type
        Link
        =
        _ShaderParamLink;
        fn create_link(_: Option<&ShaderParam<R>>,
                       params: &gfx::ProgramInfo)
         ->
             Result<_ShaderParamLink,
                    gfx::shade::ParameterError> {
            {
                let mut out =
                    _ShaderParamLink{projection: ::std::option::Option::None,
                                     view: ::std::option::Option::None,
                                     s_texture: ::std::option::Option::None,};
                {
                    let result =
                        match ::std::iter::IntoIterator::into_iter(params.uniforms.iter().enumerate())
                            {
                            mut iter =>
                            loop  {
                                match ::std::iter::Iterator::next(&mut iter) {
                                    ::std::option::Option::Some((i, u)) => {
                                        let _ = i;
                                        match &u.name[..] {
                                            "projection" => {
                                                out.projection =
                                                    Some(i as
                                                             gfx::shade::VarUniform)
                                            }
                                            "view" => {
                                                out.view =
                                                    Some(i as
                                                             gfx::shade::VarUniform)
                                            }
                                            _ =>
                                            return Err(gfx::shade::ParameterError::MissingUniform(u.name.clone())),
                                        }
                                    }
                                    ::std::option::Option::None => break ,
                                }
                            },
                        };
                    result
                }
                {
                    let result =
                        match ::std::iter::IntoIterator::into_iter(params.blocks.iter().enumerate())
                            {
                            mut iter =>
                            loop  {
                                match ::std::iter::Iterator::next(&mut iter) {
                                    ::std::option::Option::Some((i, b)) => {
                                        let _ = i;
                                        match &b.name[..] {
                                            _ =>
                                            return Err(gfx::shade::ParameterError::MissingBlock(b.name.clone())),
                                        }
                                    }
                                    ::std::option::Option::None => break ,
                                }
                            },
                        };
                    result
                }
                {
                    let result =
                        match ::std::iter::IntoIterator::into_iter(params.textures.iter().enumerate())
                            {
                            mut iter =>
                            loop  {
                                match ::std::iter::Iterator::next(&mut iter) {
                                    ::std::option::Option::Some((i, t)) => {
                                        let _ = i;
                                        match &t.name[..] {
                                            "s_texture" => {
                                                out.s_texture =
                                                    Some(i as
                                                             gfx::shade::VarTexture)
                                            }
                                            _ =>
                                            return Err(gfx::shade::ParameterError::MissingTexture(t.name.clone())),
                                        }
                                    }
                                    ::std::option::Option::None => break ,
                                }
                            },
                        };
                    result
                }
                Ok(out)
            }
        }
        fn fill_params(&self, link: &_ShaderParamLink,
                       out:
                           &mut gfx::ParamStorage<R>)
         -> () {
            use gfx::shade::ToUniform;
            out.uniforms.reserve(3usize);
            out.blocks.reserve(3usize);
            out.textures.reserve(3usize);
            /*link.projection.map_or((), |id| {
                                   if out.uniforms.len() <= id as usize {
                                       unsafe {
                                           out.uniforms.set_len(id as usize +
                                                                    1)
                                       }
                                   }
                                   *out.uniforms.get_mut(id as usize).unwrap()
                                       = self.projection.to_uniform() });
            link.view.map_or((), |id| {
                             if out.uniforms.len() <= id as usize {
                                 unsafe {
                                     out.uniforms.set_len(id as usize + 1)
                                 }
                             }
                             *out.uniforms.get_mut(id as usize).unwrap() =
                                 self.view.to_uniform() });
            link.s_texture.map_or((), |id| {
                                  if out.textures.len() <= id as usize {
                                      unsafe {
                                          out.textures.set_len(id as usize +
                                                                   1)
                                      }
                                  }
                                  *out.textures.get_mut(id as usize).unwrap()
                                      = { self.s_texture.clone() } });*/
            out.uniforms.push(self.projection.to_uniform());
            out.uniforms.push(self.view.to_uniform());
            out.textures.push(self.s_texture.clone());
        }
    }
#[derive(Copy, Clone, Debug)]
    pub struct Vertex {
        #[name = "position"]
        pub xyz: [f32; 3],
        #[name = "tex_coord"]
        pub uv: [f32; 2],
        #[name = "color"]
        pub rgb: [f32; 3],
    }
    impl gfx::VertexFormat for Vertex {
        fn generate<R: gfx::Resources>(__arg_0:
                                                                    Option<&Vertex>,
                                                                __arg_1:
                                                                    gfx::RawBufferHandle<R>)
         -> Vec<gfx::Attribute<R>> {
            {
                let mut attributes = Vec::with_capacity(3usize);
                {
                    attributes.push(gfx::Attribute{name:
                                                                                "position".to_string(),
                                                                            buffer:
                                                                                __arg_1.clone(),
                                                                            format:
                                                                                gfx::attrib::Format{elem_count:
                                                                                                                                 3,
                                                                                                                             elem_type:
                                                                                                                                 gfx::attrib::Type::Float(gfx::attrib::FloatSubType::Default,
                                                                                                                                                                                   gfx::attrib::FloatSize::F32),
                                                                                                                             offset:
                                                                                                                                 unsafe
                                                                                                                                 {
                                                                                                                                     let x:
                                                                                                                                             Vertex =
                                                                                                                                         ::std::mem::uninitialized();
                                                                                                                                     let offset =
                                                                                                                                         (&x.xyz
                                                                                                                                              as
                                                                                                                                              *const _
                                                                                                                                              as
                                                                                                                                              usize)
                                                                                                                                             -
                                                                                                                                             (&x
                                                                                                                                                  as
                                                                                                                                                  *const _
                                                                                                                                                  as
                                                                                                                                                  usize);
                                                                                                                                     ::std::mem::forget(x);
                                                                                                                                     offset
                                                                                                                                         as
                                                                                                                                         gfx::attrib::Offset
                                                                                                                                 },
                                                                                                                             stride:
                                                                                                                                 {
                                                                                                                                     use std::mem;
                                                                                                                                     mem::size_of::<Vertex>()
                                                                                                                                         as
                                                                                                                                         gfx::attrib::Stride
                                                                                                                                 },
                                                                                                                             instance_rate:
                                                                                                                                 0u8,},});
                }
                {
                    attributes.push(gfx::Attribute{name:
                                                                                "tex_coord".to_string(),
                                                                            buffer:
                                                                                __arg_1.clone(),
                                                                            format:
                                                                                gfx::attrib::Format{elem_count:
                                                                                                                                 2,
                                                                                                                             elem_type:
                                                                                                                                 gfx::attrib::Type::Float(gfx::attrib::FloatSubType::Default,
                                                                                                                                                                                   gfx::attrib::FloatSize::F32),
                                                                                                                             offset:
                                                                                                                                 unsafe
                                                                                                                                 {
                                                                                                                                     let x:
                                                                                                                                             Vertex =
                                                                                                                                         ::std::mem::uninitialized();
                                                                                                                                     let offset =
                                                                                                                                         (&x.uv
                                                                                                                                              as
                                                                                                                                              *const _
                                                                                                                                              as
                                                                                                                                              usize)
                                                                                                                                             -
                                                                                                                                             (&x
                                                                                                                                                  as
                                                                                                                                                  *const _
                                                                                                                                                  as
                                                                                                                                                  usize);
                                                                                                                                     ::std::mem::forget(x);
                                                                                                                                     offset
                                                                                                                                         as
                                                                                                                                         gfx::attrib::Offset
                                                                                                                                 },
                                                                                                                             stride:
                                                                                                                                 {
                                                                                                                                     use std::mem;
                                                                                                                                     mem::size_of::<Vertex>()
                                                                                                                                         as
                                                                                                                                         gfx::attrib::Stride
                                                                                                                                 },
                                                                                                                             instance_rate:
                                                                                                                                 0u8,},});
                }
                {
                    attributes.push(gfx::Attribute{name:
                                                                                "color".to_string(),
                                                                            buffer:
                                                                                __arg_1.clone(),
                                                                            format:
                                                                                gfx::attrib::Format{elem_count:
                                                                                                                                 3,
                                                                                                                             elem_type:
                                                                                                                                 gfx::attrib::Type::Float(gfx::attrib::FloatSubType::Default,
                                                                                                                                                                                   gfx::attrib::FloatSize::F32),
                                                                                                                             offset:
                                                                                                                                 unsafe
                                                                                                                                 {
                                                                                                                                     let x:
                                                                                                                                             Vertex =
                                                                                                                                         ::std::mem::uninitialized();
                                                                                                                                     let offset =
                                                                                                                                         (&x.rgb
                                                                                                                                              as
                                                                                                                                              *const _
                                                                                                                                              as
                                                                                                                                              usize)
                                                                                                                                             -
                                                                                                                                             (&x
                                                                                                                                                  as
                                                                                                                                                  *const _
                                                                                                                                                  as
                                                                                                                                                  usize);
                                                                                                                                     ::std::mem::forget(x);
                                                                                                                                     offset
                                                                                                                                         as
                                                                                                                                         gfx::attrib::Offset
                                                                                                                                 },
                                                                                                                             stride:
                                                                                                                                 {
                                                                                                                                     use std::mem;
                                                                                                                                     mem::size_of::<Vertex>()
                                                                                                                                         as
                                                                                                                                         gfx::attrib::Stride
                                                                                                                                 },
                                                                                                                             instance_rate:
                                                                                                                                 0u8,},});
                };
                attributes
            }
        }
    }

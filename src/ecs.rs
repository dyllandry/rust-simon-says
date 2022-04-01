use std::{
    borrow::Cow,
    cell::{RefCell, RefMut},
};

use glium::{
    implement_vertex, program, texture::Texture2d, uniform, Display, Frame, Program, Surface,
};
use rusttype::{gpu_cache::Cache, point, vector, Font, PositionedGlyph, Rect, Scale};

// Our components.
pub struct Health(pub i32);
pub struct Name(pub &'static str);

pub struct Text {
    pub text: String,
}

// World to store component vectors and entity count.
pub struct World {
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities_count: 0,
            component_vecs: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }
        self.entities_count += 1;
        entity_id
    }

    // ComponentType must be static to support downcasting Any -> ComponentType
    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        // Try to find existing component_vec for ComponentType. Insert component if component_vec
        // is found.
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = Some(component);
                return;
            }
        }

        // If component_vec not found, create a new vector & insert the component.
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);
        for _ in 0..self.entities_count {
            new_component_vec.push(None);
        }
        new_component_vec[entity] = Some(component);
        self.component_vecs
            .push(Box::new(RefCell::new(new_component_vec)));
    }

    pub fn borrow_component_vec<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }
}

pub struct TextSystem {
    font: Font<'static>,
    glyph_cache: Cache<'static>,
    glyph_cache_texture: Texture2d,
    shader_program: Program,
}

impl TextSystem {
    pub fn new(display: &Display) -> Self {
        // Initialize font.
        let font_data = include_bytes!("Roboto-Regular.ttf");
        let font = Font::try_from_bytes(font_data).unwrap();

        // Initialize gpu cache
        let scale = display.gl_window().window().scale_factor();
        let (cache_width, cache_height) = ((512.0 * scale) as u32, (512.0 * scale) as u32);
        let mut glyph_cache: Cache<'static> = Cache::builder()
            .dimensions(cache_width, cache_height)
            .build();

        // Initialize shader program.
        let shader_program = program!(
            display,
            140 => {
                vertex: "
                #version 140

                in vec2 position;
                in vec2 tex_coords;
                in vec4 colour;

                out vec2 v_tex_coords;
                out vec4 v_colour;

                void main() {
                    gl_Position = vec4(position, 0.0, 1);
                    v_tex_coords = tex_coords;
                    v_colour = colour;
                }
            ",
                fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                in vec4 v_colour;
                out vec4 f_colour;

                void main() {
                    f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
                }
            "
            }
        )
        .unwrap();

        // Init gpu cache texture.
        let glyph_cache_texture = Texture2d::with_format(
            display,
            glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
                width: cache_width,
                height: cache_height,
                format: glium::texture::ClientFormat::U8,
            },
            glium::texture::UncompressedFloatFormat::U8,
            glium::texture::MipmapsOption::NoMipmap,
        )
        .unwrap();

        TextSystem {
            font,
            glyph_cache,
            shader_program,
            glyph_cache_texture,
        }
    }
    pub fn draw(&mut self, frame: &mut Frame, display: &Display, text: &Text) {
        let scale = display.gl_window().window().scale_factor() as f32;
        let (width, _): (u32, _) = display.gl_window().window().inner_size().into();

        // Get glyphs and queue in cache
        let glyphs =
            self.layout_paragraph(&self.font, Scale::uniform(24.0 * scale), width, &text.text);
        for glyph in &glyphs {
            self.glyph_cache.queue_glyph(0, glyph.clone());
        }
        self.glyph_cache
            .cache_queued(|rect, data| {
                // Write a glyph's image to a part rect in the texture.
                self.glyph_cache_texture.main_level().write(
                    glium::Rect {
                        left: rect.min.x,
                        bottom: rect.min.y,
                        width: rect.width(),
                        height: rect.height(),
                    },
                    glium::texture::RawImage2d {
                        data: Cow::Borrowed(data),
                        width: rect.width(),
                        height: rect.height(),
                        format: glium::texture::ClientFormat::U8,
                    },
                );
            })
            .unwrap();

        // Init texture uniform from cache texture.
        let uniforms = uniform! {
            tex: self.glyph_cache_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        // Init vertex buffer for paragraph rect vertices including position, texture coordinates,
        // and colour.
        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
                tex_coords: [f32; 2],
                colour: [f32; 4],
            }
            implement_vertex!(Vertex, position, tex_coords, colour);
            let colour = [0.0, 0.0, 0.0, 1.0];
            let (screen_width, screen_height) = {
                let (w, h) = display.get_framebuffer_dimensions();
                (w as f32, h as f32)
            };
            let origin = point(0.0, 0.0);
            let vertices: Vec<Vertex> = glyphs
                .iter()
                // Get the rect for a glyph
                .filter_map(|g| self.glyph_cache.rect_for(0, g).ok().flatten())
                // uv_rect is where the glyph is in the cache texture
                // screen_rect is where the glyph is going to be drawn on the screen in pixel space
                .flat_map(|(uv_rect, screen_rect)| {
                    // This converts the pixel-space coordinate system of the glyphs (where the
                    // top left is (0,0) and bottom right is (512,512)) to opengl's coordinate
                    // system, where the top left is (-1, 1) and the bottom right is (1, -1).
                    let gl_rect = Rect {
                        min: origin
                            + (vector(
                                screen_rect.min.x as f32 / screen_width - 0.5,
                                1.0 - screen_rect.min.y as f32 / screen_height - 0.5,
                            )) * 2.0,
                        max: origin
                            + (vector(
                                screen_rect.max.x as f32 / screen_width - 0.5,
                                1.0 - screen_rect.max.y as f32 / screen_height - 0.5,
                            )) * 2.0,
                    };

                    // Each set of 6 vertices represents two triangles forming a rectangle
                    // around one glyph.
                    vec![
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.max.y],
                            tex_coords: [uv_rect.min.x, uv_rect.max.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.min.y],
                            tex_coords: [uv_rect.min.x, uv_rect.min.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.min.y],
                            tex_coords: [uv_rect.max.x, uv_rect.min.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.min.y],
                            tex_coords: [uv_rect.max.x, uv_rect.min.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.max.y],
                            tex_coords: [uv_rect.max.x, uv_rect.max.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.max.y],
                            tex_coords: [uv_rect.min.x, uv_rect.max.y],
                            colour,
                        },
                    ]
                })
                .collect();

            glium::VertexBuffer::new(display, &vertices).unwrap()
        };

        // Draw the text's vertices with uniforms and shader program
        frame
            .draw(
                &vertex_buffer,
                glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.shader_program,
                &uniforms,
                &glium::DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
    }

    fn layout_paragraph<'a>(
        &self,
        font: &Font<'a>,
        scale: Scale,
        width: u32,
        text: &str,
    ) -> Vec<PositionedGlyph<'a>> {
        let mut result = Vec::new();
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        // This is the insertion point we use to position characters.
        let mut caret = point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;
        for c in text.chars() {
            // Handle control characters
            if c.is_control() {
                match c {
                    '\r' => {
                        // If a newline is entered, reset the insertion point the start and one linedown.
                        caret = point(0.0, caret.y + advance_height);
                    }
                    '\n' => {}
                    _ => {}
                }
                continue;
            }

            let base_glyph = font.glyph(c);

            // If there's a previous glyph, add any kerning (additional h spacing) necessary between the
            // previous and next glyph.
            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, base_glyph.id());
            }
            last_glyph_id = Some(base_glyph.id());

            // Position the glyph.
            let mut glyph = base_glyph.scaled(scale).positioned(caret);
            // If we accidentally positioned it past the maximum width of the paragaph, move the
            // insertion caret to the next line and put the glyph there instead.
            if let Some(bb) = glyph.pixel_bounding_box() {
                if bb.max.x > width as i32 {
                    caret = point(0.0, caret.y + advance_height);
                    glyph.set_position(caret);
                    last_glyph_id = None;
                }
            }
            // Position the caret ahead where the next glyph would go.
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            result.push(glyph);
        }
        result
    }
}

// as_any lets us downcast from ComponentVec -> Any -> concrete component type
trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    // Each entity gets an index, all of their components are found at the same index
    // in each component vector. So entity 0's components are found at the 0 index in
    // each component vector. If the entity doesn't have that kind of component, then
    // at that index the vector contains None. Every ComponentVec type must support
    // push_none.
    fn push_none(&mut self);
}

// Casting as Any requires T to be static. Casting as Any supports downcasting
// Any -> concrete component type.
impl<T: 'static> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    fn push_none(&mut self) {
        self.get_mut().push(None)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

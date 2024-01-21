pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
        #version 460

        layout(location = 0) in vec2 position;

        layout(location = 0) out vec2 tex_coords;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
            tex_coords = position + vec2(0.5);
        }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
        #version 460

        layout(location = 0) in vec2 tex_coords;
        layout(location = 0) out vec4 f_color;

        layout(set = 0, binding = 0) uniform sampler s;
        layout(set = 0, binding = 1) uniform texture2D tex;

        void main() {
            f_color = texture(sampler2D(tex, s), tex_coords);
        }
        ",
    }
}

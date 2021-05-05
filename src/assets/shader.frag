#version 300 es
precision mediump float;

uniform bool use_tex;
uniform sampler2D sampler;

in vec4 frag_color;
in vec2 frag_uv;

out vec4 color;

void main() {
    if (use_tex) {
        color = texture(sampler, frag_uv);
    } else {
        color = frag_color;
    }
    color.rgb = pow(color.rgb, vec3(1.0 / 1.8));
}

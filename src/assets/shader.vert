#version 300 es

uniform bool use_tex;
uniform vec2 screen_dp;

layout(location = 0) in vec2 p;
layout(location = 1) in vec4 c;
layout(location = 2) in vec2 uv;
out vec4 frag_color;
out vec2 frag_uv;

void main() {
    if (use_tex) {
        frag_uv = uv;
    } else {
        frag_color = c;
    }
    gl_Position = vec4((p.xy / screen_dp - vec2(0.5)) * vec2(2.0, -2.0), 0.0, 1.0);
}

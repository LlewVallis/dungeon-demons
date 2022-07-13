#version 300 es

layout(location=0) in vec2 position;

out vec2 uv;

void main() {
    uv = position + 0.5;
    gl_Position = vec4(position * 2.0, 0.0, 1.0);
}
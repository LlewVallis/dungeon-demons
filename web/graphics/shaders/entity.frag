#version 300 es

precision highp float;

uniform sampler2D sampler;

in vec2 uv;
out vec4 color;

void main() {
    color = texture(sampler, uv);
}
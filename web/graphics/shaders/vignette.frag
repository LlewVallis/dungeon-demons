#version 300 es

precision highp float;

uniform vec3 vignetteColor;
uniform float vignetteScale;

in vec2 uv;
out vec4 color;

#define POWER 0.2

void main() {
    vec2 centered = uv * (1.0 - uv);
    float centrality = pow(centered.x * centered.y * vignetteScale, POWER);
    float intensity = 1.0 - centrality;

    color = vec4(vignetteColor, intensity * 0.9);
}

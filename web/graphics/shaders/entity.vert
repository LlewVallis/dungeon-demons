#version 300 es

uniform mat3 transform;
uniform mat3 view;
uniform mat3 textureTransform;

layout(location=0) in vec2 position;
out vec2 uv;

#define BLEED 0.001

void main() {
    mat3 transformation = view * transform;

    vec2 baseUv = position * vec2(1.0, -1.0) + 0.5;
    vec2 adjustedUv = baseUv * (1.0 - 2.0 * BLEED) + BLEED;
    uv = (textureTransform * vec3(adjustedUv, 1.0)).xy;

    vec2 transformed = (transformation * vec3(position, 1.0)).xy;
    gl_Position = vec4(transformed, 0.0, 1.0);
}
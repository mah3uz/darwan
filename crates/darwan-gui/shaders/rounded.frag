#version 440
// A picture clipped to a rounded rectangle in one pass: a signed distance to the corners gives a one-pixel soft
// edge, so no offscreen layer or mask texture is needed. uvRect crops the texture the way PreserveAspectCrop would.
layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec2 size;
    float radius;
    vec4 uvRect;
};
layout(binding = 1) uniform sampler2D source;

void main() {
    vec2 half_size = size * 0.5;
    vec2 q = abs(qt_TexCoord0 * size - half_size) - (half_size - vec2(radius));
    float d = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
    float inside = clamp(0.5 - d, 0.0, 1.0);
    fragColor = texture(source, uvRect.xy + qt_TexCoord0 * uvRect.zw) * inside * qt_Opacity;
}

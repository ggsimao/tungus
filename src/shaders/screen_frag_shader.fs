#version 430 core
in vec2 texCoords;

out vec4 fragColor;

uniform sampler2D screenTexture;
uniform bool applyFilter;

const float offset = 1.0 / 600.0;

const float kernel[3][3] = float[][](
    float[](2,2,2),
    float[](2,-15,2),
    float[](2,2,2));

void main() {
    fragColor = vec4(0);
    if (applyFilter) {
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                fragColor += texture(screenTexture, texCoords.st + vec2(i-1, j-1) * offset) * kernel[i][j];
            }
        }
    } else {
        fragColor = texture(screenTexture, texCoords.st);
    }
}
#version 430 core
in vec2 texCoords;

out vec4 fragColor;

uniform sampler2DMS screenTexture;
uniform int sampleCount;
uniform bool applySobel, applyMSAA;
uniform float gamma;

const float offset = 1.0 / 600.0;

const float kernel[3][3] = float[][](
    float[](2,2,2),
    float[](2,-15,2),
    float[](2,2,2));

void main() {
    fragColor = vec4(0);
    if (applySobel && applyMSAA) {
        for (int s = 0; s < sampleCount; s++) {
            vec4 sampleColor = vec4(0);
            for (int i = 0; i < 3; i++) {
                for (int j = 0; j < 3; j++) {
                    ivec2 texelCoords = ivec2((texCoords + ivec2(i - 1, j - 1) * offset) * textureSize(screenTexture));
                    sampleColor += texelFetch(screenTexture, texelCoords, s) * kernel[i][j];
                }
            }
            fragColor += sampleColor / sampleCount;
        }

    } else if (applySobel && !applyMSAA) {
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                ivec2 texelCoords = ivec2((texCoords + ivec2(i - 1, j - 1) * offset) * textureSize(screenTexture));
                fragColor += texelFetch(screenTexture, texelCoords, 0) * kernel[i][j];
            }
        }
    } else if (applyMSAA) {
        for (int s = 0; s < sampleCount; s++) {
            ivec2 texelCoords = ivec2(texCoords * textureSize(screenTexture));
            vec4 sampleColor = texelFetch(screenTexture, texelCoords, s);
            fragColor += sampleColor / sampleCount;
        }
    } else {
        ivec2 texelCoords = ivec2(texCoords * textureSize(screenTexture));
        fragColor = texelFetch(screenTexture, texelCoords, 0);
    }
    fragColor.rgb = pow(fragColor.rgb, vec3(1.0/gamma));
}
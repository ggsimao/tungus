#version 430 core
in VERTEX {
    vec3 pos;
    vec3 normal;
    vec2 texCoords;
    vec4 dirLightSpaceFragPos;
}
fs_in;

out vec4 fragColor;

#define NR_DIFFUSE_TEXTURES 3
#define NR_SPECULAR_TEXTURES 3

struct Material {
    sampler2D Diffuse[NR_DIFFUSE_TEXTURES];
    sampler2D Specular[NR_SPECULAR_TEXTURES];
    float shininess;
    int loadedDiffuse;
    int loadedSpecular;
};

uniform Material material;
uniform vec3 outlineColor;

void main() {
    float texture_alpha = 0.0;
    for (int i = 0; i < material.loadedDiffuse; i++) {
        texture_alpha =
            max(texture_alpha, texture(material.Diffuse[i], fs_in.texCoords).a);
    }

    if (texture_alpha == 0) {
        discard;
    } else {
        fragColor = vec4(outlineColor, 1.0);
    }
}

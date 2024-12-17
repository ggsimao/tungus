#version 430 core
in VERTEX {
    vec3 pos;
    vec3 normal;
    vec2 texCoords;
} fs_in;

#define NR_DIFFUSE_TEXTURES 3
#define NR_SPECULAR_TEXTURES 3

struct Material {
    sampler2D diffuseTextures[NR_DIFFUSE_TEXTURES];
    sampler2D specularTextures[NR_SPECULAR_TEXTURES];
    float shininess;
    int loadedDiffuse;
    int loadedSpecular;
};

struct DirLight {
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;
};

struct Spotlight {
    vec3 position;
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float phiCos;
    float gammaCos;
};

layout (std140, binding = 0) uniform Matrices {
    mat4 modelMat;
    mat4 viewMat;
    mat4 projMat;
};

#define NR_POINT_LIGHTS 4
uniform DirLight dirLight;
uniform PointLight pointLights[NR_POINT_LIGHTS];
uniform Spotlight spotlight;

uniform Material material;

out vec4 fragColor;

vec4 diff_tex_values[NR_DIFFUSE_TEXTURES];
vec4 spec_tex_values[NR_SPECULAR_TEXTURES];

vec4 calculateLightValue(float diff_str, float spec_str, vec3 amb_color, vec3 diff_color, vec3 spec_color, float shininess) {
    vec4 final_ambient = vec4(0.0);
    vec4 final_diffuse = vec4(0.0);
    vec4 final_specular = vec4(0.0);

    for (int i = 0; i < material.loadedDiffuse; i++) {
        vec4 diff_tex = diff_tex_values[i];
        vec4 ambient = vec4(amb_color, 1.0) * diff_tex;
        final_ambient.rgb += ambient.rgb;
        final_ambient.a = max(final_ambient.a, ambient.a);

        vec4 diffuse = vec4(diff_color, 1.0) * diff_str * diff_tex;
        diffuse.a = min(diffuse.a, 1.0);
        final_diffuse.rgb += diffuse.rgb;
        final_diffuse.a = max(final_diffuse.a, diffuse.a);
    }
    for (int i = 0; i < material.loadedSpecular; i++) {
        vec4 spec_tex = spec_tex_values[i];
        vec4 specular = vec4(spec_color, 1.0) * spec_str * spec_tex;
        specular.a = min(specular.a, 1.0);
        final_specular.rgb += specular.rgb;
        final_specular.a = max(final_specular.a, specular.a);
    }
    final_ambient.rgb /= material.loadedDiffuse;
    final_diffuse.rgb /= material.loadedDiffuse;
    final_specular.rgb /= material.loadedSpecular;

    vec4 final_light;
    final_light.rgb = final_ambient.rgb + final_diffuse.rgb + final_specular.rgb;
    final_light.a = max(final_ambient.a, max(final_diffuse.a, final_specular.a));

    return final_light;
}

vec4 calculateDirectionalLight(DirLight light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light.direction);
    float diff = max(dot(normal, lightDir), 0.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    vec4 directional_value = calculateLightValue(diff, spec, light.ambient, light.diffuse, light.specular, material.shininess);

    return directional_value;
}

vec4 calculatePointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    float diff = max(dot(normal, lightDir), 0.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    float dist = length(light.position - fragPos);
    float attenuation = 1.0 / ( light.constant + light.linear * dist + light.quadratic * ( dist * dist ) );

    vec4 pointlight_value = calculateLightValue(diff, spec, light.ambient, light.diffuse, light.specular, material.shininess);
    pointlight_value.rgb *= attenuation;

    return pointlight_value;
}

vec4 calculateSpotlight(Spotlight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    float diff = max(dot(normal, lightDir), 0.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    float theta = dot(lightDir, normalize(-light.direction));
    float intensity = max(( theta - light.gammaCos ) / ( light.phiCos - light.gammaCos ), 0.0);

    vec4 spotlight_value = calculateLightValue(diff, spec, light.ambient, light.diffuse, light.specular, material.shininess);
    spotlight_value.rgb *= intensity;

    return spotlight_value;
}

void main() {
    for (int i = 0; i < material.loadedDiffuse; i++)
        diff_tex_values[i] = texture(material.diffuseTextures[i], fs_in.texCoords);
    for (int i = 0; i < material.loadedSpecular; i++)
        spec_tex_values[i] = texture(material.specularTextures[i], fs_in.texCoords);

    vec3 norm = normalize(fs_in.normal);
    vec3 viewPos = vec3(viewMat[3][0], viewMat[3][1], viewMat[3][2]);
    vec3 viewDir = normalize(viewPos - fs_in.pos);

    vec4 result = calculateDirectionalLight(dirLight, norm, viewDir);

    for (int i = 0; i < NR_POINT_LIGHTS; i++) {
        vec4 pointlight_value = calculatePointLight(pointLights[i], norm, fs_in.pos, viewDir);
        result.rgb += pointlight_value.rgb;
        result.a = max(result.a, pointlight_value.a);
    }

    vec4 spotlight_value = calculateSpotlight(spotlight, norm, fs_in.pos, viewDir);
    result.rgb += spotlight_value.rgb;
    result.a = max(result.a, spotlight_value.a);

    if (result.a < 0.1) {
        discard;
    } else {
        fragColor = result;
    }
}

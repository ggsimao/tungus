#version 430 core
in VERTEX {
    vec3 pos;
    vec3 normal;
    vec2 texCoords;
} fs_in;

#define NR_DIFFUSE_TEXTURES 3
#define NR_SPECULAR_TEXTURES 3

struct Material {
    sampler2D Diffuse[NR_DIFFUSE_TEXTURES];
    sampler2D Specular[NR_SPECULAR_TEXTURES];
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
    mat3 normalMat;
};

#define NR_POINT_LIGHTS 4
uniform DirLight dirLight;
uniform PointLight pointLights[NR_POINT_LIGHTS];
uniform Spotlight spotlight;

uniform Material material;

out vec4 fragColor;

vec4 diff_tex_values[NR_DIFFUSE_TEXTURES];
vec4 spec_tex_values[NR_SPECULAR_TEXTURES];

vec4 calculateDirectionalLight(DirLight light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light . direction);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material . shininess);

    vec4 final_ambient = vec4(0.0);
    vec4 final_diffuse = vec4(0.0);
    vec4 final_specular = vec4(0.0);

    for (int i = 0; i < material.loadedDiffuse; i++) {
        vec4 ambient = vec4(light . ambient, 1.0) * diff_tex_values[i];
        vec4 diffuse = vec4(light . diffuse, 1.0) * diff * diff_tex_values[i];
        final_ambient = mix(final_ambient, ambient, 0.5);
        final_diffuse = mix(final_diffuse, diffuse, 0.5);
    }
    for (int i = 0; i < material.loadedSpecular; i++) {
        vec4 specular = vec4(light . specular, 1.0) * spec * spec_tex_values[i];
        final_specular = mix(final_specular, specular, 0.5);
    }

    return ( final_ambient + final_diffuse + final_specular );
}

vec4 calculatePointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light . position - fragPos);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material . shininess);

    float distance = length(light . position - fragPos);
    float attenuation = 1.0 / ( light . constant + light . linear * distance + light . quadratic * ( distance * distance ) );

    vec4 final_ambient = vec4(0.0);
    vec4 final_diffuse = vec4(0.0);
    vec4 final_specular = vec4(0.0);

    for (int i = 0; i < material.loadedDiffuse; i++) {
        vec4 ambient = vec4(light . ambient, 1.0) * diff_tex_values[i];
        vec4 diffuse = vec4(light . diffuse, 1.0) * diff * diff_tex_values[i];
        final_ambient = mix(final_ambient, ambient, 0.5);
        final_diffuse = mix(final_diffuse, diffuse, 0.5);
    }
    for (int i = 0; i < material.loadedSpecular; i++) {
        vec4 specular = vec4(light . specular, 1.0) * spec * spec_tex_values[i];
        final_specular = mix(final_specular, specular, 0.5);
    }

    return ( final_ambient + final_diffuse + final_specular );
}

vec4 calculateSpotlight(Spotlight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light . position - fragPos);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material . shininess);

    float theta = dot(lightDir, normalize(-light . direction));

    float intensity = max(( theta - light . gammaCos ) / ( light . phiCos - light . gammaCos ), 0.0);

    vec4 final_ambient = vec4(0.0);
    vec4 final_diffuse = vec4(0.0);
    vec4 final_specular = vec4(0.0);

    for (int i = 0; i < material.loadedDiffuse; i++) {
        vec4 ambient = vec4(light . ambient, 1.0) * diff_tex_values[i];
        vec4 diffuse = vec4(light . diffuse, 1.0) * diff * diff_tex_values[i];
        ambient *= intensity;
        diffuse *= intensity;
        final_ambient = mix(final_ambient, ambient, 0.5);
        final_diffuse = mix(final_diffuse, diffuse, 0.5);
    }
    for (int i = 0; i < material.loadedSpecular; i++) {
        vec4 specular = vec4(light . specular, 1.0) * spec * spec_tex_values[i];
        specular *= intensity;
        final_specular = mix(final_specular, specular, 0.5);
    }

    return ( final_ambient + final_diffuse + final_specular );
}

void main() {
    for (int i = 0; i < material.loadedDiffuse; i++)
        diff_tex_values[i] = texture(material . Diffuse[i], fs_in.texCoords);
    for (int i = 0; i < material.loadedSpecular; i++)
        spec_tex_values[i] = texture(material . Specular[i], fs_in.texCoords);

    vec3 norm = normalize(fs_in.normal);
    vec3 viewPos = vec3(viewMat[3][0], viewMat[3][1], viewMat[3][2]);
    vec3 viewDir = normalize(viewPos - fs_in.pos);

    vec4 result = calculateDirectionalLight(dirLight, norm, viewDir);

    for (int i = 0; i < NR_POINT_LIGHTS; i++)
        result += calculatePointLight(pointLights[i], norm, fs_in.pos, viewDir);

    result += calculateSpotlight(spotlight, norm, fs_in.pos, viewDir);

    if (result . a < 0.1) {
        discard;
    } else {
        fragColor = result;
    }
}

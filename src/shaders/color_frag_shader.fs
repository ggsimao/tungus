#version 450 core
in vec3 fragPos;
in vec3 normal;
in vec2 texCoords;

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
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

uniform DirLight dirLight;

#define NR_POINT_LIGHTS 4
uniform PointLight pointLights[NR_POINT_LIGHTS];

uniform Spotlight spotlight;

uniform Material material;
uniform vec3 viewPos;

out vec4 fragColor;

vec3 calculateDirectionalLight(DirLight light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light . direction);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material . shininess);

    vec3 ambient = light . ambient * vec3(texture(material . diffuse, texCoords));
    vec3 diffuse = light . diffuse * diff * vec3(texture(material . diffuse, texCoords));
    vec3 specular = light . specular * spec * vec3(texture(material . specular, texCoords));
    return ( ambient + diffuse + specular );
}

vec3 calculatePointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light . position - fragPos);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material . shininess);

    float distance = length(light . position - fragPos);
    float attenuation = 1.0 / ( light . constant + light . linear * distance + light . quadratic * ( distance * distance ) );

    vec3 ambient = light . ambient * vec3(texture(material . diffuse, texCoords));
    vec3 diffuse = light . diffuse * diff * vec3(texture(material . diffuse, texCoords));
    vec3 specular = light . specular * spec * vec3(texture(material . specular, texCoords));
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    return ( ambient + diffuse + specular );
}

vec3 calculateSpotlight(Spotlight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light . position - fragPos);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material . shininess);

    float theta = dot(lightDir, normalize(-light . direction));

    float intensity = max(( theta - light . gammaCos ) / ( light . phiCos - light . gammaCos ), 0.0);

    vec3 ambient = light . ambient * vec3(texture(material . diffuse, texCoords));
    vec3 diffuse = light . diffuse * diff * vec3(texture(material . diffuse, texCoords));
    vec3 specular = light . specular * spec * vec3(texture(material . specular, texCoords));
    ambient *= intensity;
    diffuse *= intensity;
    specular *= intensity;

    return ( ambient + diffuse + specular );
}

void main() {
    vec3 norm = normalize(normal);
    vec3 viewDir = normalize(viewPos - fragPos);

    vec3 result = calculateDirectionalLight(dirLight, norm, viewDir);

    for (int i = 0; i < NR_POINT_LIGHTS; i++)
        result += calculatePointLight(pointLights[i], norm, fragPos, viewDir);

    result += calculateSpotlight(spotlight, norm, fragPos, viewDir);

    fragColor = vec4(result, 1.0);
}

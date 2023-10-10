#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec3 aColor;
layout(location = 3) in vec2 aTexCoord;

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform mat3 normalMatrix;

out vec3 fragPos;
out vec3 normal;
out vec2 texCoords;

void main() {
    gl_Position = vec4(aPos, 1.0);
    gl_Position = projectionMatrix * viewMatrix * modelMatrix * gl_Position;
    fragPos = vec3(modelMatrix * vec4(aPos, 1.0));
    normal = normalMatrix * aNormal;
    texCoords = aTexCoord;
}

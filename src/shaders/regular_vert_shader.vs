#version 430 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;

layout (std140, binding = 0) uniform Matrices {
    mat4 modelMat;
    mat4 viewMat;
    mat4 projMat;
    mat3 normalMat;
};

out VERTEX {
    vec3 pos;
    vec3 normal;
    vec2 texCoords;
} vs_out;

void main() {
    gl_Position = vec4(aPos, 1.0);
    gl_Position = projMat * viewMat * modelMat * gl_Position;
    vs_out.pos = vec3(modelMat * vec4(aPos, 1.0));
    vs_out.normal = normalMat * aNormal;
    vs_out.texCoords = aTexCoord;
}

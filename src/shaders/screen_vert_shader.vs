#version 430 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoords;

layout (std140, binding = 0) uniform Matrices {
    mat4 modelMat;
    mat4 viewMat;
    mat4 projMat;
};

out vec2 texCoords;

void main() {
    gl_Position = modelMat * vec4(aPos.x, aPos.y, 0.0, 1.0);
    texCoords = aTexCoords;
}
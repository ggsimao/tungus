#version 430 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;
layout(location = 3) in mat4 aInstModel;
layout(location = 7) in mat3 aInstNormal;

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

out vec3 geo_normal;

void main() {
    gl_Position = vec4(aPos, 1.0);
    vec4 out_pos_4 = modelMat * aInstModel * gl_Position;
    gl_Position = projMat * viewMat * out_pos_4;
    vs_out.pos = vec3(out_pos_4);
    vs_out.normal = normalMat * aInstNormal * aNormal;
    geo_normal = aNormal;
    vs_out.texCoords = aTexCoord;
}

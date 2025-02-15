#version 430 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;
layout(location = 3) in mat4 aInstModel;
layout(location = 7) in mat3 aInstNormal;

layout(std140, binding = 0) uniform Matrices {
    mat4 modelMat;
    mat4 viewMat;
    mat4 projMat;
};

uniform mat4 dirLightSpaceMatrix;

out VERTEX {
    vec3 pos;
    vec3 normal;
    vec2 texCoords;
    vec4 dirLightSpaceFragPos;
}
vs_out;

out vec3 geo_normal;

mat3 extractRotation(mat4 modelMatrix) {
    // Extract the upper-left 3x3 part of the model matrix
    mat3 rotationMatrix = mat3(modelMatrix);

    // Normalize each row to remove scaling (if needed)
    rotationMatrix[0] = normalize(rotationMatrix[0]);
    rotationMatrix[1] = normalize(rotationMatrix[1]);
    rotationMatrix[2] = normalize(rotationMatrix[2]);

    return rotationMatrix;
}

void main() {
    vec4 out_pos_4 = modelMat * aInstModel * vec4(aPos, 1.0);
    vs_out.pos = vec3(out_pos_4);

    mat3 normal_mat = transpose(inverse(mat3(modelMat)));
    mat3 inst_normal_mat = transpose(inverse(mat3(aInstModel)));
    vs_out.normal = normalize(normal_mat * inst_normal_mat * aNormal);
    geo_normal =
        extractRotation(modelMat) * extractRotation(aInstModel) * aNormal;

    vs_out.texCoords = aTexCoord;

    vs_out.dirLightSpaceFragPos = dirLightSpaceMatrix * vec4(vs_out.pos, 1.0);

    gl_Position = projMat * viewMat * out_pos_4;
}
